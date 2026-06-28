"""
Domain models for the Jira MCP bridge.

All models are optional-field-friendly: we never crash on missing Jira fields
because Jira's field set varies wildly between instances and custom schemes.
"""
from __future__ import annotations

from datetime import datetime
from typing import Any

from pydantic import BaseModel, Field


# ---------------------------------------------------------------------------
# Inbound (from Jira)
# ---------------------------------------------------------------------------

class JiraUser(BaseModel):
    account_id: str = ""
    display_name: str = ""
    email: str = ""


class JiraComment(BaseModel):
    id: str = ""
    author: JiraUser = Field(default_factory=JiraUser)
    body: str = ""
    created: datetime | None = None
    updated: datetime | None = None


class JiraTicket(BaseModel):
    """Normalised representation of a Jira issue — cloud REST v3."""

    id: str
    key: str                        # e.g. PROJ-42
    summary: str = ""
    description: str = ""           # ADF → plain text, converted server-side
    status: str = ""                # status.name
    priority: str = ""              # priority.name
    issue_type: str = ""            # issuetype.name
    assignee: JiraUser = Field(default_factory=JiraUser)
    reporter: JiraUser = Field(default_factory=JiraUser)
    labels: list[str] = Field(default_factory=list)
    components: list[str] = Field(default_factory=list)
    fix_versions: list[str] = Field(default_factory=list)
    story_points: float | None = None
    parent_key: str | None = None
    linked_issue_keys: list[str] = Field(default_factory=list)
    comments: list[JiraComment] = Field(default_factory=list)
    created: datetime | None = None
    updated: datetime | None = None
    raw: dict[str, Any] = Field(default_factory=dict, exclude=True)


class JiraTransition(BaseModel):
    id: str
    name: str
    to_status: str = ""


class JiraSearchResult(BaseModel):
    total: int
    issues: list[JiraTicket]


# ---------------------------------------------------------------------------
# Local sync state  (written as jira_ref.json alongside feature docs)
# ---------------------------------------------------------------------------

class JiraSyncState(BaseModel):
    """
    Persisted in <feature_dir>/jira_ref.json.
    Records the link between a local feature directory and a Jira ticket,
    plus a snapshot hash so we can detect drift.
    """
    ticket_key: str
    ticket_id: str
    feature_path: str               # relative path, e.g. docs/features/login/
    last_synced_at: datetime | None = None
    description_hash: str = ""      # SHA-256 of ticket description at last sync
    status_at_sync: str = ""
    pending_clarifications: list[str] = Field(default_factory=list)


class SyncDiff(BaseModel):
    ticket_key: str
    feature_path: str
    description_changed: bool = False
    status_changed: bool = False
    previous_status: str = ""
    current_status: str = ""
    has_pending_clarifications: bool = False
    in_sync: bool = True
