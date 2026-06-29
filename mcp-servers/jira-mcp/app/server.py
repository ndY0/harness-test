"""
Jira MCP Bridge — FastMCP server.

Tools
-----
  get_ticket            Pull a ticket's full spec into the pipeline
  search_tickets        JQL-based backlog scanning
  get_ticket_comments   Fetch the comment thread on a ticket
  transition_ticket     Move a ticket through its workflow
  add_comment           Push a comment (clarification, summary) to Jira
  set_field             Update an arbitrary Jira field
  link_local_feature    Create/update the local jira_ref.json sidecar
  get_sync_status       Diff local snapshot vs live Jira ticket
  request_clarification Post structured questions + set Awaiting Clarification

All tools return a plain-text or JSON-serialisable result.
If Jira is not configured (JIRA_BASE_URL not set) every tool returns a
structured error object instead of raising — this keeps the pipeline
running without an Atlassian account.
"""
from __future__ import annotations

import json
import logging
from contextlib import asynccontextmanager
from functools import wraps
from typing import Any

from mcp.server.fastmcp import FastMCP

from .client import JiraClient, JiraClientError
from .config import get_config
from .sync import (
    clear_clarifications,
    get_sync_diff,
    link_local_feature,
    list_all_synced_features,
    load_sync_state,
    record_clarification,
)

logging.basicConfig(level=logging.INFO, format="%(levelname)s %(name)s: %(message)s")
logger = logging.getLogger(__name__)

config = get_config()

# ---------------------------------------------------------------------------
# Application lifespan — shared httpx client
# ---------------------------------------------------------------------------

_client: JiraClient | None = None


@asynccontextmanager
async def lifespan(server: FastMCP):
    global _client
    if config.jira_configured:
        _client = JiraClient(config)
        logger.info("Jira client initialised → %s", config.jira_base_url)
    else:
        logger.warning(
            "JIRA_BASE_URL / JIRA_USER_EMAIL / JIRA_API_TOKEN not set. "
            "Jira tools will return a 'not configured' response."
        )
    yield
    if _client:
        await _client.close()


mcp = FastMCP(
    "jira-bridge",
    lifespan=lifespan,
    host=config.host,
    port=config.port,
    json_response=True
)


# ---------------------------------------------------------------------------
# Guard helpers
# ---------------------------------------------------------------------------

def _not_configured() -> dict:
    return {
        "error": "jira_not_configured",
        "message": (
            "Jira integration is disabled. "
            "Set JIRA_BASE_URL, JIRA_USER_EMAIL, and JIRA_API_TOKEN "
            "to enable it."
        ),
    }


def _jira_required(fn):
    """Decorator: return a structured error if Jira is not configured."""
    @wraps(fn)
    async def wrapper(*args, **kwargs):
        if not config.jira_configured or _client is None:
            return json.dumps(_not_configured())
        try:
            return await fn(*args, **kwargs)
        except JiraClientError as exc:
            return json.dumps({
                "error": "jira_api_error",
                "status_code": exc.status_code,
                "message": str(exc),
            })
        except Exception as exc:
            logger.exception("Unexpected error in Jira tool")
            return json.dumps({"error": "internal_error", "message": str(exc)})
    return wrapper


# ---------------------------------------------------------------------------
# Tools — read / ingest
# ---------------------------------------------------------------------------

@mcp.tool()
@_jira_required
async def get_ticket(ticket_key: str) -> str:
    """
    Fetch a Jira ticket and return its full spec as structured JSON.

    Returned fields: key, summary, description (plain text), status, priority,
    issue_type, assignee, reporter, labels, components, fix_versions,
    story_points, parent_key, linked_issue_keys, comments, created, updated.

    Use this as the primary input for the Spec Writer when implementing a
    ticket-backed feature.
    """
    ticket = await _client.get_ticket(ticket_key)
    return ticket.model_dump_json(indent=2, exclude={"raw"})


@mcp.tool()
@_jira_required
async def search_tickets(jql: str, max_results: int = 20) -> str:
    """
    Search Jira using a JQL query and return a list of matching tickets.

    Example JQL queries:
      project = PROJ AND status = "To Do" ORDER BY priority DESC
      assignee = currentUser() AND sprint in openSprints()
      labels = "pipeline-ready" AND status != Done

    Returns: { total, issues: [ { key, summary, status, priority, issue_type } ] }
    """
    result = await _client.search_tickets(jql, max_results=max_results)
    return result.model_dump_json(indent=2)


@mcp.tool()
@_jira_required
async def get_ticket_comments(ticket_key: str) -> str:
    """
    Fetch only the comment thread for a ticket (lighter than get_ticket).

    Useful for the Reviewer to check whether a clarification question was
    already asked, or for the Spec Writer to pick up context from discussion.

    Returns: list of { id, author, body, created, updated }
    """
    ticket = await _client.get_ticket(ticket_key)
    comments_json = [c.model_dump() for c in ticket.comments]
    return json.dumps(comments_json, indent=2, default=str)


# ---------------------------------------------------------------------------
# Tools — lifecycle / write
# ---------------------------------------------------------------------------

@mcp.tool()
@_jira_required
async def transition_ticket(ticket_key: str, transition_name: str) -> str:
    """
    Move a Jira ticket to a new workflow state by transition name.

    Common transition names (case-insensitive, partial match supported):
      "In Progress", "In Review", "Done", "Blocked", "Awaiting Clarification"

    The tool fetches available transitions first and matches by name prefix,
    so you don't need to know the transition ID.

    Returns: { ticket_key, transitioned_to, transition_id }
    """
    transitions = await _client.get_transitions(ticket_key)
    name_lower = transition_name.lower()
    match = next(
        (t for t in transitions if t.name.lower().startswith(name_lower)),
        None,
    )
    if match is None:
        available = [t.name for t in transitions]
        return json.dumps({
            "error": "transition_not_found",
            "requested": transition_name,
            "available": available,
        })
    await _client.transition_ticket(ticket_key, match.id)
    return json.dumps({
        "ticket_key": ticket_key,
        "transitioned_to": match.to_status,
        "transition_id": match.id,
    })


@mcp.tool()
@_jira_required
async def add_comment(ticket_key: str, body: str) -> str:
    """
    Post a plain-text comment to a Jira ticket.

    Use this to:
    - Push an implementation summary when a feature is done
    - Record a blocking question for the product owner
    - Note a deviation from the original spec

    Returns: { ticket_key, comment_id }
    """
    comment_id = await _client.add_comment(ticket_key, body)
    return json.dumps({"ticket_key": ticket_key, "comment_id": comment_id})


@mcp.tool()
@_jira_required
async def set_field(ticket_key: str, field: str, value: Any) -> str:
    """
    Update a single field on a Jira ticket.

    Common fields:
      story_points / customfield_10016  → number
      assignee                          → { "accountId": "..." }
      labels                            → ["label1", "label2"]
      fixVersions                       → [{ "name": "v1.2" }]

    Returns: { ticket_key, field, value }
    """
    await _client.set_field(ticket_key, field, value)
    return json.dumps({"ticket_key": ticket_key, "field": field, "value": value})


# ---------------------------------------------------------------------------
# Tools — local sync
# ---------------------------------------------------------------------------

@mcp.tool()
async def link_local_feature_tool(ticket_key: str, feature_path: str) -> str:
    """
    Link a local feature directory to a Jira ticket.

    Creates or updates a jira_ref.json sidecar file inside feature_path,
    recording the ticket key, a hash of the description, and the current
    status.  Safe to call multiple times (idempotent).

    feature_path: repo-relative directory, e.g. "docs/features/login/"

    This tool works even without Jira configured — it only requires a valid
    ticket_key if you later want to use get_sync_status.
    If Jira IS configured the ticket is fetched live to seed the snapshot.
    Returns: the written JiraSyncState as JSON.
    """
    if config.jira_configured and _client is not None:
        try:
            ticket = await _client.get_ticket(ticket_key)
        except JiraClientError as exc:
            return json.dumps({"error": "jira_api_error", "message": str(exc)})
    else:
        # Offline mode: create a minimal stub so the sidecar exists
        from .models import JiraTicket
        ticket = JiraTicket(id="", key=ticket_key)

    state = link_local_feature(ticket, feature_path)
    return state.model_dump_json(indent=2)


@mcp.tool()
@_jira_required
async def get_sync_status(feature_path: str) -> str:
    """
    Compare the local jira_ref.json snapshot with the live Jira ticket.

    Returns a SyncDiff describing:
      - description_changed: ticket description changed since last sync
      - status_changed: ticket status changed (e.g. PO re-opened it)
      - has_pending_clarifications: unanswered questions on record
      - in_sync: true only when nothing has drifted

    feature_path: repo-relative directory, e.g. "docs/features/login/"
    """
    diff = await get_sync_diff(_client, feature_path)
    return diff.model_dump_json(indent=2)


@mcp.tool()
@_jira_required
async def request_clarification(
    ticket_key: str,
    feature_path: str,
    questions: list[str],
) -> str:
    """
    Post structured clarification questions to Jira and park the feature.

    This tool:
    1. Posts a formatted comment listing the questions on the Jira ticket
    2. Transitions the ticket to "Awaiting Clarification" (if that transition exists)
    3. Records the questions in the local jira_ref.json

    The Orchestrator should treat the returned { parked: true } signal as a
    directive to move on to the next feature and not retry this one until
    the questions are resolved.

    Returns: { ticket_key, comment_id, parked, questions }
    """
    # 1. Format and post the comment
    formatted = "**Clarification requested by automated pipeline**\n\n"
    for i, q in enumerate(questions, 1):
        formatted += f"{i}. {q}\n"
    formatted += "\nPlease reply to unblock implementation."
    comment_id = await _client.add_comment(ticket_key, formatted)

    # 2. Try to transition — fail silently if transition doesn't exist
    transitions = await _client.get_transitions(ticket_key)
    clarify_transition = next(
        (t for t in transitions if "clarif" in t.name.lower()),
        None,
    )
    if clarify_transition:
        await _client.transition_ticket(ticket_key, clarify_transition.id)

    # 3. Record locally
    if feature_path:
        state = load_sync_state(feature_path)
        if state:
            record_clarification(feature_path, questions)

    return json.dumps({
        "ticket_key": ticket_key,
        "comment_id": comment_id,
        "parked": True,
        "questions": questions,
        "transition_applied": clarify_transition.name if clarify_transition else None,
    })


@mcp.tool()
async def list_synced_features() -> str:
    """
    List all local features that have a jira_ref.json sidecar file.

    Returns: list of { ticket_key, feature_path, status_at_sync, last_synced_at }
    Useful for the Orchestrator to discover what is already linked.
    This tool works without Jira configured.
    """
    states = list_all_synced_features()
    return json.dumps([
        {
            "ticket_key": s.ticket_key,
            "feature_path": s.feature_path,
            "status_at_sync": s.status_at_sync,
            "last_synced_at": s.last_synced_at.isoformat() if s.last_synced_at else None,
            "pending_clarifications": len(s.pending_clarifications),
        }
        for s in states
    ], indent=2)


# ---------------------------------------------------------------------------
# Entrypoint
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    mcp.run(transport="streamable-http")