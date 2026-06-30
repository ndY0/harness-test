"""Filesystem-backed message store.

Message files:  {channels_dir}/{channel}.{seq:06d}.json
Cursor files:   {channels_dir}/.cursors/{consumer}:{channel}
"""

from __future__ import annotations

import json
import os
import time
from pathlib import Path
from typing import Any


class Store:
    def __init__(self, channels_dir: str):
        self._dir = Path(channels_dir)
        self._cursors_dir = self._dir / ".cursors"
        self._dir.mkdir(parents=True, exist_ok=True)
        self._cursors_dir.mkdir(parents=True, exist_ok=True)

    # ── sequence numbers ──────────────────────────────────────────────────

    def next_seq(self) -> int:
        """Return the next available global sequence number."""
        existing = list(self._dir.glob("*.json"))
        if not existing:
            return 1
        max_seq = 0
        for f in existing:
            # filename: {channel}.{seq:06d}.json
            stem = f.stem  # e.g. "review:F042.000003"
            parts = stem.rsplit(".", 1)
            if len(parts) == 2:
                try:
                    seq = int(parts[1])
                    if seq > max_seq:
                        max_seq = seq
                except ValueError:
                    pass
        return max_seq + 1

    # ── messages ──────────────────────────────────────────────────────────

    def write_message(
        self,
        channel: str,
        seq: int,
        from_agent: str,
        body: dict[str, Any],
        reply_to: str | None = None,
    ) -> dict[str, Any]:
        ts = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        msg = {
            "seq": f"{seq:06d}",
            "channel": channel,
            "from": from_agent,
            "body": body,
            "reply_to": reply_to,
            "status": "unread",
            "resolved_by": None,
            "resolution": None,
            "ts": ts,
        }
        filename = f"{channel}.{seq:06d}.json"
        tmp = self._dir / f".{filename}.tmp"
        target = self._dir / filename
        tmp.write_text(json.dumps(msg, ensure_ascii=False))
        os.rename(tmp, target)
        return msg

    def update_message_status(
        self, channel: str, seq: int, status: str, resolved_by: str | None = None, resolution: str | None = None
    ) -> dict[str, Any] | None:
        filename = self._dir / f"{channel}.{seq:06d}.json"
        if not filename.exists():
            return None
        msg = json.loads(filename.read_text())
        msg["status"] = status
        if resolved_by is not None:
            msg["resolved_by"] = resolved_by
        if resolution is not None:
            msg["resolution"] = resolution
        tmp = self._dir / f".{channel}.{seq:06d}.json.tmp"
        tmp.write_text(json.dumps(msg, ensure_ascii=False))
        os.rename(tmp, filename)
        return msg

    def read_message(self, channel: str, seq: int) -> dict[str, Any] | None:
        filename = self._dir / f"{channel}.{seq:06d}.json"
        if not filename.exists():
            return None
        return json.loads(filename.read_text())

    def read_messages_after(
        self, channel: str, after_seq: int
    ) -> list[dict[str, Any]]:
        """Return all messages on *channel* with seq > *after_seq*, sorted."""
        prefix = f"{channel}."
        msgs = []
        for f in self._dir.glob(f"{prefix}*.json"):
            stem = f.stem
            parts = stem.rsplit(".", 1)
            if len(parts) == 2:
                try:
                    seq = int(parts[1])
                    if seq > after_seq:
                        msgs.append(json.loads(f.read_text()))
                except ValueError:
                    pass
        msgs.sort(key=lambda m: m["seq"])
        return msgs

    def all_messages(self, channel: str) -> list[dict[str, Any]]:
        return self.read_messages_after(channel, 0)

    def conversation_start(self, channel: str) -> dict[str, Any] | None:
        msgs = self.read_messages_after(channel, 0)
        return msgs[0] if msgs else None

    # ── cursors ───────────────────────────────────────────────────────────

    def _cursor_file(self, consumer: str, channel: str) -> Path:
        safe = f"{consumer}:{channel}"
        return self._cursors_dir / safe

    def get_cursor(self, consumer: str, channel: str) -> int:
        f = self._cursor_file(consumer, channel)
        if not f.exists():
            return 0
        return int(f.read_text().strip())

    def set_cursor(self, consumer: str, channel: str, seq: int) -> None:
        f = self._cursor_file(consumer, channel)
        tmp = self._cursors_dir / f".{f.name}.tmp"
        tmp.write_text(str(seq))
        os.rename(tmp, f)

    def all_consumers(self, channel: str) -> list[str]:
        suffix = f":{channel}"
        consumers = []
        for f in self._cursors_dir.iterdir():
            if f.name.endswith(suffix):
                consumers.append(f.name[: -len(suffix)])
        return consumers

    def unread_count(self, channel: str) -> int:
        """Messages not yet read by ANY consumer on this channel."""
        consumers = self.all_consumers(channel)
        if not consumers:
            return len(self.all_messages(channel))
        min_cursor = min(self.get_cursor(c, channel) for c in consumers)
        return len(self.read_messages_after(channel, min_cursor))

    def pending_for(self, consumer: str, channel: str) -> list[dict[str, Any]]:
        cursor = self.get_cursor(consumer, channel)
        return self.read_messages_after(channel, cursor)

    def mark_responded(self, channel: str, seq: int) -> None:
        self.update_message_status(channel, seq, "responded")

    def mark_resolved(
        self, channel: str, seq: int, resolved_by: str, resolution: str
    ) -> dict[str, Any] | None:
        return self.update_message_status(
            channel, seq, "resolved", resolved_by=resolved_by, resolution=resolution
        )
