"""
Thin async client for the Jira Cloud REST API v3.

Design principles
-----------------
* Never raises on missing optional fields — Jira field sets vary widely.
* ADF (Atlassian Document Format) descriptions are flattened to plain text
  so agents don't have to deal with the JSON AST.
* All methods raise JiraClientError on HTTP errors; callers convert to
  MCP tool error responses.
"""
from __future__ import annotations

import hashlib
import json
import logging
from typing import Any

import httpx

from .config import Config
from .models import (
    JiraComment,
    JiraSearchResult,
    JiraTicket,
    JiraTransition,
    JiraUser,
)

logger = logging.getLogger(__name__)


class JiraClientError(Exception):
    """Raised on non-2xx Jira responses."""

    def __init__(self, status_code: int, message: str) -> None:
        super().__init__(message)
        self.status_code = status_code


def _adf_to_text(node: Any, depth: int = 0) -> str:
    """
    Recursively flatten an ADF document node to plain text.
    Handles the most common node types; unknown types are silently skipped.
    """
    if node is None:
        return ""
    if isinstance(node, str):
        return node

    node_type = node.get("type", "")
    text = node.get("text", "")

    if node_type == "text":
        return text

    children = node.get("content", [])
    inner = "".join(_adf_to_text(c, depth + 1) for c in children)

    if node_type in ("paragraph", "blockquote"):
        return inner.strip() + "\n\n"
    if node_type in ("heading",):
        level = node.get("attrs", {}).get("level", 1)
        return "#" * level + " " + inner.strip() + "\n\n"
    if node_type == "bulletList":
        return inner
    if node_type == "orderedList":
        return inner
    if node_type == "listItem":
        return "- " + inner.strip() + "\n"
    if node_type == "codeBlock":
        lang = node.get("attrs", {}).get("language", "")
        return f"```{lang}\n{inner.strip()}\n```\n\n"
    if node_type == "hardBreak":
        return "\n"
    if node_type == "rule":
        return "---\n\n"
    # doc, table, tableRow, tableCell, mediaSingle, etc. — just recurse
    return inner


def _parse_description(raw_desc: Any) -> str:
    """Convert a Jira description field (ADF or plain string) to text."""
    if raw_desc is None:
        return ""
    if isinstance(raw_desc, str):
        return raw_desc
    if isinstance(raw_desc, dict):
        return _adf_to_text(raw_desc).strip()
    return ""


def _parse_user(raw: Any) -> JiraUser:
    if not raw:
        return JiraUser()
    return JiraUser(
        account_id=raw.get("accountId", ""),
        display_name=raw.get("displayName", ""),
        email=raw.get("emailAddress", ""),
    )


def _parse_ticket(raw: dict[str, Any]) -> JiraTicket:
    fields = raw.get("fields", {})
    linked_keys: list[str] = []
    for link in fields.get("issuelinks", []):
        if inward := link.get("inwardIssue"):
            linked_keys.append(inward["key"])
        if outward := link.get("outwardIssue"):
            linked_keys.append(outward["key"])

    story_points: float | None = None
    for sp_field in ("story_points", "customfield_10016", "customfield_10028"):
        val = fields.get(sp_field)
        if val is not None:
            try:
                story_points = float(val)
                break
            except (TypeError, ValueError):
                pass

    parent_key: str | None = None
    if parent := fields.get("parent"):
        parent_key = parent.get("key")

    return JiraTicket(
        id=raw.get("id", ""),
        key=raw.get("key", ""),
        summary=fields.get("summary", ""),
        description=_parse_description(fields.get("description")),
        status=(fields.get("status") or {}).get("name", ""),
        priority=(fields.get("priority") or {}).get("name", ""),
        issue_type=(fields.get("issuetype") or {}).get("name", ""),
        assignee=_parse_user(fields.get("assignee")),
        reporter=_parse_user(fields.get("reporter")),
        labels=fields.get("labels") or [],
        components=[c.get("name", "") for c in (fields.get("components") or [])],
        fix_versions=[v.get("name", "") for v in (fields.get("fixVersions") or [])],
        story_points=story_points,
        parent_key=parent_key,
        linked_issue_keys=linked_keys,
        raw=raw,
    )


def description_hash(description: str) -> str:
    return hashlib.sha256(description.encode()).hexdigest()


class JiraClient:
    def __init__(self, config: Config) -> None:
        self._config = config
        self._base = (config.jira_base_url or "").rstrip("/")
        self._client = httpx.AsyncClient(
            auth=config.jira_auth,
            headers={
                "Accept": "application/json",
                "Content-Type": "application/json",
            },
            timeout=15.0,
        )

    async def _get(self, path: str, params: dict | None = None) -> Any:
        url = f"{self._base}/rest/api/3{path}"
        resp = await self._client.get(url, params=params)
        if not resp.is_success:
            raise JiraClientError(resp.status_code, resp.text)
        return resp.json()

    async def _post(self, path: str, body: dict) -> Any:
        url = f"{self._base}/rest/api/3{path}"
        resp = await self._client.post(url, content=json.dumps(body))
        if not resp.is_success:
            raise JiraClientError(resp.status_code, resp.text)
        # Some POST endpoints return 204 No Content
        if resp.status_code == 204 or not resp.content:
            return {}
        return resp.json()

    async def _put(self, path: str, body: dict) -> Any:
        url = f"{self._base}/rest/api/3{path}"
        resp = await self._client.put(url, content=json.dumps(body))
        if not resp.is_success:
            raise JiraClientError(resp.status_code, resp.text)
        if resp.status_code == 204 or not resp.content:
            return {}
        return resp.json()

    # ------------------------------------------------------------------
    # Public interface
    # ------------------------------------------------------------------

    async def get_ticket(self, ticket_key: str) -> JiraTicket:
        raw = await self._get(
            f"/issue/{ticket_key}",
            params={"fields": "summary,description,status,priority,issuetype,"
                              "assignee,reporter,labels,components,fixVersions,"
                              "story_points,customfield_10016,customfield_10028,"
                              "parent,issuelinks,comment,created,updated"},
        )
        ticket = _parse_ticket(raw)
        # Embed comments
        comment_data = (raw.get("fields", {}).get("comment") or {})
        comments = []
        for c in comment_data.get("comments", []):
            comments.append(JiraComment(
                id=c.get("id", ""),
                author=_parse_user(c.get("author")),
                body=_parse_description(c.get("body")),
                created=c.get("created"),
                updated=c.get("updated"),
            ))
        ticket.comments = comments
        return ticket

    async def search_tickets(self, jql: str, max_results: int = 20) -> JiraSearchResult:
        raw = await self._post("/search", {
            "jql": jql,
            "maxResults": max_results,
            "fields": ["summary", "status", "priority", "issuetype", "assignee"],
        })
        issues = [_parse_ticket(i) for i in raw.get("issues", [])]
        return JiraSearchResult(total=raw.get("total", len(issues)), issues=issues)

    async def get_transitions(self, ticket_key: str) -> list[JiraTransition]:
        raw = await self._get(f"/issue/{ticket_key}/transitions")
        return [
            JiraTransition(
                id=t["id"],
                name=t["name"],
                to_status=(t.get("to") or {}).get("name", ""),
            )
            for t in raw.get("transitions", [])
        ]

    async def transition_ticket(self, ticket_key: str, transition_id: str) -> None:
        await self._post(
            f"/issue/{ticket_key}/transitions",
            {"transition": {"id": transition_id}},
        )

    async def add_comment(self, ticket_key: str, body_text: str) -> str:
        """Post a plain-text comment; returns the new comment ID."""
        # Wrap plain text in minimal ADF so Jira Cloud accepts it
        adf_body = {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [{"type": "text", "text": body_text}],
                }
            ],
        }
        raw = await self._post(f"/issue/{ticket_key}/comment", {"body": adf_body})
        return raw.get("id", "")

    async def set_field(self, ticket_key: str, field: str, value: Any) -> None:
        await self._put(f"/issue/{ticket_key}", {"fields": {field: value}})

    async def close(self) -> None:
        await self._client.aclose()
