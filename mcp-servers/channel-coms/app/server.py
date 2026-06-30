"""Channel communication MCP server for the agent pipeline.

Exposes eight tools over streamable-HTTP:
  publish         — write a message to a channel
  poll            — non-blocking read of new messages since cursor
  await_message   — block until a new message arrives (long-poll)
  unread_count    — messages not yet read by any consumer
  conversation_age — seconds since the first message on a channel
  pending_for     — messages not yet read by a specific consumer
  resolve_message — mark a message as resolved (dispatcher-only)
  list_channels   — enumerate active channels
"""

from __future__ import annotations

import asyncio
import time
from typing import Any

from mcp.server.fastmcp import FastMCP

from .config import config
from .store import Store

# ── shared state ─────────────────────────────────────────────────────────
_store: Store | None = None
_events: dict[str, asyncio.Event] = {}


def _get_event(channel: str) -> asyncio.Event:
    if channel not in _events:
        _events[channel] = asyncio.Event()
    return _events[channel]


def _wake(channel: str) -> None:
    ev = _events.get(channel)
    if ev is not None:
        ev.set()
        ev.clear()


mcp = FastMCP(
    "channel-coms",
    host=config.host,
    port=config.port,
    json_response=True,
)


# ── tools ────────────────────────────────────────────────────────────────

@mcp.tool()
def publish(
    channel: str,
    from_agent: str,
    body: dict[str, Any],
    reply_to: str | None = None,
) -> dict[str, Any]:
    """Write a message to a channel. Wakes any await_message callers.

    Channel names are feature-scoped, e.g. "review:F042" or "planner:F042:contracts".
    The dispatching agent (Orchestrator/Planner) generates the channel name.

    Args:
        channel: Channel identifier. Convention: {type}:{scope}:{name}.
        from_agent: Name of the agent publishing (e.g. "implementer", "reviewer").
        body: JSON-serializable message body. Agents define their own schema.
        reply_to: Optional sequence number of a parent message this replies to.
                  If set, the parent message is automatically marked "responded".

    Returns:
        The published message, including its assigned sequence number and timestamp.
    """
    seq = _store.next_seq()
    msg = _store.write_message(channel, seq, from_agent, body, reply_to)
    if reply_to is not None:
        try:
            parent_seq = int(reply_to)
            _store.mark_responded(channel, parent_seq)
        except (ValueError, TypeError):
            pass
    _wake(channel)
    return msg


@mcp.tool()
async def await_message(
    channel: str,
    consumer: str,
    cursor: int = 0,
    timeout_ms: int = 30000,
) -> dict[str, Any] | None:
    """Block until a new message arrives on the channel, or timeout.

    Use this when an agent needs a response from another agent before continuing
    (e.g. Reviewer awaits Implementer's clarification answer).

    Args:
        channel: Channel to listen on.
        consumer: Name of the calling agent. Used to track read progress.
        cursor: Last sequence number this consumer has processed. Messages with
                seq > cursor are returned.
        timeout_ms: Maximum time to wait in milliseconds. Default 30000.

    Returns:
        The first new message, or None if the timeout expired with no message.
        The consumer's cursor is advanced to the returned message's seq.
    """
    deadline = time.monotonic() + timeout_ms / 1000.0
    event = _get_event(channel)
    stored_cursor = _store.get_cursor(consumer, channel)
    effective_cursor = max(cursor, stored_cursor)

    while True:
        msgs = _store.read_messages_after(channel, effective_cursor)
        if msgs:
            msg = msgs[0]
            seq = int(msg["seq"])
            _store.set_cursor(consumer, channel, seq)
            return msg

        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return None

        try:
            await asyncio.wait_for(event.wait(), timeout=remaining)
        except asyncio.TimeoutError:
            return None


@mcp.tool()
def poll(
    channel: str,
    consumer: str,
    cursor: int = 0,
) -> list[dict[str, Any]]:
    """Non-blocking read of new messages since the given cursor.

    Use this between work units to check for pending clarifications without
    blocking (e.g. Implementer polls after each stage).

    Args:
        channel: Channel to read from.
        consumer: Name of the calling agent. Cursor is advanced on return.
        cursor: Last sequence number processed. Default 0.

    Returns:
        List of new messages (may be empty). Consumer's cursor is advanced to
        the highest seq returned.
    """
    stored_cursor = _store.get_cursor(consumer, channel)
    effective_cursor = max(cursor, stored_cursor)
    msgs = _store.read_messages_after(channel, effective_cursor)
    if msgs:
        _store.set_cursor(consumer, channel, int(msgs[-1]["seq"]))
    return msgs


@mcp.tool()
def unread_count(channel: str) -> int:
    """Return the number of messages not yet read by any consumer.

    Zero means all consumers are caught up. Non-zero is a signal for the
    dispatcher to investigate — someone may not be polling.

    Args:
        channel: Channel to inspect.

    Returns:
        Count of messages no consumer has acknowledged reading.
    """
    return _store.unread_count(channel)


@mcp.tool()
def conversation_age(channel: str) -> float | None:
    """Return seconds since the first message on this channel.

    Use this to detect conversations that have been running too long without
    resolution. A high value with a non-zero unread_count may mean the
    dispatcher needs to cut the conversation short and arbitrate.

    Args:
        channel: Channel to inspect.

    Returns:
        Seconds since first message, or None if the channel is empty.
    """
    first = _store.conversation_start(channel)
    if first is None:
        return None
    msg_ts = first["ts"]
    try:
        msg_time = time.mktime(time.strptime(msg_ts, "%Y-%m-%dT%H:%M:%SZ"))
    except ValueError:
        return None
    return time.time() - msg_time


@mcp.tool()
def pending_for(consumer: str, channel: str) -> list[dict[str, Any]]:
    """Return messages on a channel not yet read by a specific consumer.

    The dispatcher uses this to decide whether an agent needs re-dispatching
    with pending messages injected into its context.

    Args:
        consumer: Agent name to check (e.g. "implementer", "reviewer").
        channel: Channel to inspect.

    Returns:
        List of unread messages for this consumer, oldest first.
    """
    return _store.pending_for(consumer, channel)


@mcp.tool()
def resolve_message(
    channel: str,
    seq: str,
    resolved_by: str,
    resolution: str,
) -> dict[str, Any] | None:
    """Mark a message as resolved. Dispatcher-only.

    Called after the dispatcher arbitrates a conversation that has timed out
    or reached a deadlock. The resolution string documents the decision.

    Args:
        channel: Channel containing the message.
        seq: Sequence number as a zero-padded string (e.g. "000003").
        resolved_by: Name of the dispatcher making the ruling.
        resolution: Human-readable description of the decision.

    Returns:
        The updated message, or None if not found.
    """
    try:
        seq_int = int(seq)
    except ValueError:
        return None
    return _store.mark_resolved(channel, seq_int, resolved_by, resolution)


@mcp.tool()
def list_channels() -> list[dict[str, Any]]:
    """List all channels with message counts and status.

    Returns one entry per channel: name, total messages, unread count,
    and conversation age.
    """
    seen: set[str] = set()
    channels: list[dict[str, Any]] = []
    for f in sorted(_store._dir.glob("*.json")):
        stem = f.stem  # e.g. "review:F042.000003"
        parts = stem.rsplit(".", 1)
        if len(parts) != 2:
            continue
        channel = parts[0]
        if channel in seen:
            continue
        seen.add(channel)
        channels.append({
            "channel": channel,
            "total": len(_store.all_messages(channel)),
            "unread": _store.unread_count(channel),
            "age_s": conversation_age(channel),
        })
    return channels


# ── entry point ──────────────────────────────────────────────────────────

def main() -> None:
    global _store
    _store = Store(config.channels_dir)
    mcp.run(transport="streamable-http")


if __name__ == "__main__":
    main()
