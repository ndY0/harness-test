"""
Configuration for the Jira MCP server.

Jira credentials are entirely optional: if JIRA_BASE_URL is not set the server
starts normally and every Jira tool returns a clear "not configured" error
rather than crashing.  This ensures the pipeline stays functional without
any Atlassian account.
"""
from __future__ import annotations

import os
from functools import lru_cache


class Config:
    # --- Jira connection (all optional) ------------------------------------
    jira_base_url: str | None       # e.g. https://myorg.atlassian.net
    jira_user_email: str | None
    jira_api_token: str | None

    # --- Local repo root (for jira_ref.json side-car files) ---------------
    repo_root: str                  # mounted read-write inside the container

    # --- Server transport ---------------------------------------------------
    host: str
    port: int

    def __init__(self) -> None:
        self.jira_base_url   = os.environ.get("JIRA_BASE_URL")
        self.jira_user_email = os.environ.get("JIRA_USER_EMAIL")
        self.jira_api_token  = os.environ.get("JIRA_API_TOKEN")
        self.repo_root       = os.environ.get("REPO_ROOT", "/repo")
        self.host            = os.environ.get("MCP_HOST", "0.0.0.0")
        self.port            = int(os.environ.get("MCP_PORT", "8001"))

    @property
    def jira_configured(self) -> bool:
        return bool(
            self.jira_base_url
            and self.jira_user_email
            and self.jira_api_token
        )

    @property
    def jira_auth(self) -> tuple[str, str]:
        """Basic-auth tuple for httpx."""
        return (self.jira_user_email or "", self.jira_api_token or "")


@lru_cache(maxsize=1)
def get_config() -> Config:
    return Config()
