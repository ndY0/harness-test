"""
Indexer: orchestrates LSP queries to build the full dependency graph.

Strategy for full workspace index:
  1. Walk all source files (using plugin.file_patterns)
  2. For each file: send didOpen + query documentSymbol → upsert symbols
  3. For each function/method symbol: query call hierarchy → upsert call edges
  4. For each type symbol: query type hierarchy → upsert implements edges
  5. Infer test edges from naming conventions

Strategy for incremental file index:
  1. Delete existing symbols for the file
  2. Run steps 2–5 above for just that file

The LSP client is shared across all index operations (one server process).
File batching is used to avoid overwhelming the LSP server and DGraph.
"""

from __future__ import annotations

import asyncio
from pathlib import Path
from typing import TYPE_CHECKING

import structlog

from .config import settings
from .dgraph import DGraphClient
from .graph_builder import (
    symbols_from_document,
    edges_from_call_hierarchy,
    edges_from_type_hierarchy,
    infer_test_edges,
    make_symbol_id,
    path_to_uri,
    uri_to_path,
)
from .lsp_client import LspClient, LspError

if TYPE_CHECKING:
    from .plugin_base import LanguagePlugin

log = structlog.get_logger()

# Concurrency limits — LSP servers are sensitive to query floods
_FILE_BATCH_SIZE = 3
_EDGE_BATCH_SIZE = 5

# Symbol kinds that support call hierarchy (functions, methods, constructors)
_CALL_HIERARCHY_KINDS = {
    "function", "method", "constructor",
}

# Symbol kinds that support type hierarchy (classes, structs, traits, interfaces)
_TYPE_HIERARCHY_KINDS = {
    "class", "struct", "trait", "interface", "enum",
}


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

async def index_file(
    client: DGraphClient,
    lsp: LspClient,
    plugin: "LanguagePlugin",
    file_path: str,
) -> None:
    """(Re-)index a single source file."""
    log.info("indexer.indexing_file", file=file_path, language=plugin.name)
    file_uri = path_to_uri(file_path)

    # Clear stale data first
    await client.delete_file_symbols(file_path)

    symbols = await _extract_symbols(lsp, plugin, file_uri, file_path)
    if not symbols:
        log.debug("indexer.no_symbols", file=file_path)
        return

    # Upsert symbols → build lookup maps
    sym_id_map, name_to_sym_id = await _upsert_symbols(client, symbols)

    # Build and upsert edges
    edges = await _extract_edges(lsp, plugin, symbols, name_to_sym_id)
    await _upsert_edges(client, edges)

    log.info(
        "indexer.file_done",
        file=file_path,
        symbols=len(symbols),
        edges=len(edges),
    )


async def index_workspace(
    client: DGraphClient,
    lsp: LspClient,
    plugin: "LanguagePlugin",
    workspace: str | None = None,
) -> None:
    """Full workspace re-index."""
    workspace = workspace or settings.workspace
    source_files = _find_source_files(workspace, plugin)

    log.info(
        "indexer.full_index_start",
        file_count=len(source_files),
        workspace=workspace,
        language=plugin.name,
    )

    # Phase 1: extract and upsert all symbols first
    # (edges can only be resolved after all symbols exist)
    all_symbols: list[dict] = []
    name_to_sym_id: dict[str, str] = {}

    for i in range(0, len(source_files), _FILE_BATCH_SIZE):
        batch = source_files[i: i + _FILE_BATCH_SIZE]
        batch_results = await asyncio.gather(
            *[_extract_symbols(lsp, plugin, path_to_uri(str(f)), str(f)) for f in batch],
            return_exceptions=True,
        )
        for file_path, result in zip(batch, batch_results):
            if isinstance(result, Exception):
                log.warning("indexer.symbol_extract_failed", file=str(file_path), error=str(result))
                continue
            all_symbols.extend(result)

    # Upsert all symbols before building any edges
    _, name_to_sym_id = await _upsert_symbols(client, all_symbols)
    log.info("indexer.symbols_upserted", count=len(all_symbols))

    # Phase 2: extract and upsert edges for all symbols
    all_edges: list[tuple[str, str, str]] = []

    # Group symbols by file for efficient LSP batching
    by_file: dict[str, list[dict]] = {}
    for sym in all_symbols:
        by_file.setdefault(sym["file_uri"], []).append(sym)

    file_uris = list(by_file.keys())
    for i in range(0, len(file_uris), _FILE_BATCH_SIZE):
        batch_uris = file_uris[i: i + _FILE_BATCH_SIZE]
        batch_results = await asyncio.gather(
            *[
                _extract_edges(lsp, plugin, by_file[uri], name_to_sym_id)
                for uri in batch_uris
            ],
            return_exceptions=True,
        )
        for uri, result in zip(batch_uris, batch_results):
            if isinstance(result, Exception):
                log.warning("indexer.edge_extract_failed", file=uri, error=str(result))
                continue
            all_edges.extend(result)

    await _upsert_edges(client, all_edges)

    stats = await client.stats()
    log.info("indexer.full_index_done", edges=len(all_edges), **stats)


# ---------------------------------------------------------------------------
# Symbol extraction
# ---------------------------------------------------------------------------

async def _extract_symbols(
    lsp: LspClient,
    plugin: "LanguagePlugin",
    file_uri: str,
    file_path: str,
) -> list[dict]:
    """Open a file in the LSP and query its document symbols."""
    try:
        text = Path(file_path).read_text(encoding="utf-8", errors="replace")
        await lsp.did_open(file_uri, plugin.language_id, text)

        # Small delay to let the LSP server parse the file
        await asyncio.sleep(0.1)

        raw_symbols = await lsp.document_symbols(file_uri)
        await lsp.did_close(file_uri)
    except LspError as e:
        log.warning("indexer.lsp_error", file=file_path, error=str(e))
        return []
    except Exception as e:
        log.warning("indexer.extract_error", file=file_path, error=str(e))
        return []

    if not raw_symbols:
        return []

    symbols = symbols_from_document(raw_symbols, file_uri, plugin)

    # Attach file_uri to each symbol (graph_builder already sets file_path)
    for sym in symbols:
        sym["file_uri"] = file_uri

    return symbols


# ---------------------------------------------------------------------------
# Edge extraction
# ---------------------------------------------------------------------------

async def _extract_edges(
    lsp: LspClient,
    plugin: "LanguagePlugin",
    symbols: list[dict],
    name_to_sym_id: dict[str, str],
) -> list[tuple[str, str, str]]:
    """Extract call and type hierarchy edges for a list of symbols."""
    edges: list[tuple[str, str, str]] = []

    # Call hierarchy edges (functions/methods)
    call_syms = [s for s in symbols if s["kind"] in _CALL_HIERARCHY_KINDS]
    for i in range(0, len(call_syms), _EDGE_BATCH_SIZE):
        batch = call_syms[i: i + _EDGE_BATCH_SIZE]
        results = await asyncio.gather(
            *[_call_edges_for_symbol(lsp, sym) for sym in batch],
            return_exceptions=True,
        )
        for sym, result in zip(batch, results):
            if isinstance(result, Exception):
                log.debug("indexer.call_edge_failed", sym=sym["name"], error=str(result))
                continue
            edges.extend(result)

    # Type hierarchy edges (classes/structs/traits)
    type_syms = [s for s in symbols if s["kind"] in _TYPE_HIERARCHY_KINDS]
    for i in range(0, len(type_syms), _EDGE_BATCH_SIZE):
        batch = type_syms[i: i + _EDGE_BATCH_SIZE]
        results = await asyncio.gather(
            *[_type_edges_for_symbol(lsp, sym) for sym in batch],
            return_exceptions=True,
        )
        for sym, result in zip(batch, results):
            if isinstance(result, Exception):
                log.debug("indexer.type_edge_failed", sym=sym["name"], error=str(result))
                continue
            edges.extend(result)

    # Test edges (inferred from naming convention)
    test_edges = infer_test_edges(symbols, plugin, name_to_sym_id)
    edges.extend(test_edges)

    return edges


async def _call_edges_for_symbol(
    lsp: LspClient, sym: dict
) -> list[tuple[str, str, str]]:
    """Query call hierarchy for a single symbol and return edge tuples."""
    file_uri = sym["file_uri"]
    # LSP positions are 0-based
    line = sym["line"] - 1
    col = sym["col"]

    items = await lsp.prepare_call_hierarchy(file_uri, line, col)
    if not items:
        return []

    item = items[0]
    outgoing, incoming = await asyncio.gather(
        lsp.outgoing_calls(item),
        lsp.incoming_calls(item),
        return_exceptions=False,
    )

    return edges_from_call_hierarchy(item, outgoing or [], incoming or [], {})


async def _type_edges_for_symbol(
    lsp: LspClient, sym: dict
) -> list[tuple[str, str, str]]:
    """Query type hierarchy for a single symbol and return edge tuples."""
    file_uri = sym["file_uri"]
    line = sym["line"] - 1
    col = sym["col"]

    items = await lsp.prepare_type_hierarchy(file_uri, line, col)
    if not items:
        return []

    item = items[0]
    supers, subs = await asyncio.gather(
        lsp.supertypes(item),
        lsp.subtypes(item),
        return_exceptions=False,
    )

    return edges_from_type_hierarchy(item, supers or [], subs or [])


# ---------------------------------------------------------------------------
# DGraph upsert
# ---------------------------------------------------------------------------

async def _upsert_symbols(
    client: DGraphClient,
    symbols: list[dict],
) -> tuple[dict[str, str], dict[str, str]]:
    """
    Upsert all symbols to DGraph.
    Returns (sym_id → uid, name → sym_id) maps.
    """
    sym_id_map: dict[str, str] = {}
    name_to_sym_id: dict[str, str] = {}

    for sym in symbols:
        try:
            uid = await client.upsert_symbol(sym)
            sym_id_map[sym["id"]] = uid
            # Last writer wins for name → id (prefer pub symbols)
            existing = name_to_sym_id.get(sym["name"])
            if existing is None or sym.get("visibility") == "pub":
                name_to_sym_id[sym["name"]] = sym["id"]
        except Exception as e:
            log.warning("indexer.upsert_failed", sym=sym["name"], error=str(e))

    return sym_id_map, name_to_sym_id


async def _upsert_edges(
    client: DGraphClient,
    edges: list[tuple[str, str, str]],
) -> None:
    """Upsert all edges to DGraph, resolving symbol IDs to UIDs."""
    # Batch resolve symbol IDs → UIDs
    all_sym_ids = set()
    for from_id, _, to_id in edges:
        all_sym_ids.add(from_id)
        all_sym_ids.add(to_id)

    uid_map: dict[str, str] = {}
    for sym_id in all_sym_ids:
        uid = await client.resolve_uid(sym_id)
        if uid:
            uid_map[sym_id] = uid

    # Upsert edges
    for from_id, predicate, to_id in edges:
        from_uid = uid_map.get(from_id)
        to_uid = uid_map.get(to_id)
        if from_uid and to_uid and from_uid != to_uid:
            try:
                await client.add_edge(from_uid, predicate, to_uid)
            except Exception as e:
                log.debug("indexer.edge_upsert_failed", predicate=predicate, error=str(e))


# ---------------------------------------------------------------------------
# File discovery
# ---------------------------------------------------------------------------

def _find_source_files(workspace: str, plugin: "LanguagePlugin") -> list[Path]:
    """Find all source files matching the plugin's file patterns."""
    root = Path(workspace)
    exclude = set(plugin.exclude_dirs)
    files: list[Path] = []

    for pattern in plugin.file_patterns:
        for f in root.rglob(pattern):
            # Skip excluded directories anywhere in the path
            if any(part in exclude for part in f.parts):
                continue
            files.append(f)

    return sorted(set(files))  # deduplicate (a file can match multiple patterns)
