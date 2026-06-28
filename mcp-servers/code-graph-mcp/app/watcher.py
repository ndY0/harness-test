"""
File watcher: watches the workspace for source file changes and triggers
incremental re-indexing. Language-agnostic — uses plugin.file_patterns
to determine which files to watch.
"""

from __future__ import annotations

import asyncio
from pathlib import Path
from typing import TYPE_CHECKING

import structlog
from watchfiles import awatch, Change

from .config import settings
from .dgraph import DGraphClient
from .indexer import index_file
from .lsp_client import LspClient

if TYPE_CHECKING:
    from .plugin_base import LanguagePlugin

log = structlog.get_logger()

_DEBOUNCE_SECONDS = 1.5


async def watch_workspace(
    client: DGraphClient,
    lsp: LspClient,
    plugin: "LanguagePlugin",
) -> None:
    """
    Watch workspace for source file changes.
    Runs forever as a background task.
    """
    workspace = settings.workspace
    extensions = _extensions_from_patterns(plugin.file_patterns)
    exclude_dirs = set(plugin.exclude_dirs)

    log.info(
        "watcher.started",
        workspace=workspace,
        language=plugin.name,
        extensions=list(extensions),
    )

    pending: dict[str, asyncio.TimerHandle] = {}
    loop = asyncio.get_event_loop()

    async for changes in awatch(workspace, recursive=True):
        for change_type, path in changes:
            # Filter by extension
            if not any(path.endswith(ext) for ext in extensions):
                continue

            # Filter excluded directories
            path_parts = Path(path).parts
            if any(part in exclude_dirs for part in path_parts):
                continue

            if change_type == Change.deleted:
                log.info("watcher.deleted", file=path)
                await client.delete_file_symbols(path)
                continue

            # Debounce: cancel any pending re-index for this file
            if path in pending:
                pending[path].cancel()

            def _trigger(p: str = path) -> None:
                pending.pop(p, None)
                asyncio.ensure_future(_reindex(client, lsp, plugin, p))

            handle = loop.call_later(_DEBOUNCE_SECONDS, _trigger)
            pending[path] = handle


async def _reindex(
    client: DGraphClient,
    lsp: LspClient,
    plugin: "LanguagePlugin",
    file_path: str,
) -> None:
    try:
        await index_file(client, lsp, plugin, file_path)
    except Exception as e:
        log.error("watcher.reindex_failed", file=file_path, error=str(e))


def _extensions_from_patterns(patterns: list[str]) -> set[str]:
    """
    Extract file extensions from glob patterns.
    ["*.rs", "*.toml"] → {".rs", ".toml"}
    """
    exts = set()
    for pattern in patterns:
        if "." in pattern:
            ext = "." + pattern.rsplit(".", 1)[-1]
            exts.add(ext)
    return exts
