"""
Local synchronisation state manager.

Each feature linked to a Jira ticket gets a jira_ref.json sidecar file
written next to the feature's markdown document(s).  This file is the
source of truth for the local↔remote mapping and allows offline diff
detection without hitting the Jira API.

The repo root is mounted read-write in the container (unlike the
doc-retrieval-mcp where the repo is read-only) because we need to write
these sidecar files.
"""
from __future__ import annotations

import json
import logging
from datetime import datetime, timezone
from pathlib import Path

from .client import JiraClient, description_hash
from .config import get_config
from .models import JiraSyncState, JiraTicket, SyncDiff

logger = logging.getLogger(__name__)

_SIDECAR_FILENAME = "jira_ref.json"


def _repo_root() -> Path:
    return Path(get_config().repo_root)


def _sidecar_path(feature_path: str) -> Path:
    """
    Resolve the sidecar file path.
    feature_path is relative to repo root, e.g. "docs/features/login/".
    """
    return _repo_root() / feature_path.lstrip("/") / _SIDECAR_FILENAME


# ------------------------------------------------------------------
# Read / write helpers
# ------------------------------------------------------------------

def load_sync_state(feature_path: str) -> JiraSyncState | None:
    path = _sidecar_path(feature_path)
    if not path.exists():
        return None
    try:
        return JiraSyncState.model_validate_json(path.read_text())
    except Exception as exc:
        logger.warning("Could not parse %s: %s", path, exc)
        return None


def save_sync_state(state: JiraSyncState) -> None:
    path = _sidecar_path(state.feature_path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(state.model_dump_json(indent=2))
    logger.info("Saved sync state → %s", path)


def list_all_synced_features() -> list[JiraSyncState]:
    """Walk the repo and collect all jira_ref.json files."""
    root = _repo_root()
    states: list[JiraSyncState] = []
    for sidecar in root.rglob(_SIDECAR_FILENAME):
        try:
            states.append(JiraSyncState.model_validate_json(sidecar.read_text()))
        except Exception as exc:
            logger.warning("Skipping malformed %s: %s", sidecar, exc)
    return states


# ------------------------------------------------------------------
# High-level sync operations (called by MCP tools)
# ------------------------------------------------------------------

def link_local_feature(ticket: JiraTicket, feature_path: str) -> JiraSyncState:
    """
    Create or update the sidecar file for a feature↔ticket link.
    Safe to call multiple times (idempotent on ticket_key + feature_path).
    """
    existing = load_sync_state(feature_path)
    state = JiraSyncState(
        ticket_key=ticket.key,
        ticket_id=ticket.id,
        feature_path=feature_path,
        last_synced_at=datetime.now(timezone.utc),
        description_hash=description_hash(ticket.description),
        status_at_sync=ticket.status,
        pending_clarifications=(existing.pending_clarifications if existing else []),
    )
    save_sync_state(state)
    return state


async def get_sync_diff(
    client: JiraClient,
    feature_path: str,
) -> SyncDiff:
    """
    Compare the local snapshot against the live Jira ticket.
    Returns a SyncDiff describing what (if anything) has drifted.
    """
    state = load_sync_state(feature_path)
    if state is None:
        return SyncDiff(
            ticket_key="",
            feature_path=feature_path,
            in_sync=False,
        )

    live = await client.get_ticket(state.ticket_key)
    live_hash = description_hash(live.description)

    desc_changed   = live_hash != state.description_hash
    status_changed = live.status != state.status_at_sync

    return SyncDiff(
        ticket_key=state.ticket_key,
        feature_path=feature_path,
        description_changed=desc_changed,
        status_changed=status_changed,
        previous_status=state.status_at_sync,
        current_status=live.status,
        has_pending_clarifications=bool(state.pending_clarifications),
        in_sync=not desc_changed and not status_changed,
    )


def record_clarification(feature_path: str, questions: list[str]) -> JiraSyncState:
    state = load_sync_state(feature_path)
    if state is None:
        raise ValueError(f"No jira_ref.json found for feature path: {feature_path}")
    state.pending_clarifications.extend(questions)
    save_sync_state(state)
    return state


def clear_clarifications(feature_path: str) -> JiraSyncState:
    state = load_sync_state(feature_path)
    if state is None:
        raise ValueError(f"No jira_ref.json found for feature path: {feature_path}")
    state.pending_clarifications = []
    save_sync_state(state)
    return state
