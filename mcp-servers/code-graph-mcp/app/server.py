"""
MCP server exposing the code graph to agents.

Unchanged tools (17):
  find_symbol, fuzzy_find, get_file_symbols, get_module_api, get_module_tree,
  get_callers, get_callees, get_type_usages, get_implementors,
  get_trait_dependents, get_tests_for, get_edit_surface, get_signature,
  get_coupling_hotspots, get_cross_module_boundary, get_stats,
  index_file, index_workspace

New tool:
  list_languages  → available plugin languages and which is active
"""

from __future__ import annotations

import asyncio
import json
from contextlib import asynccontextmanager
from typing import Any, AsyncIterator

import structlog
from mcp.server.fastmcp import FastMCP

# Import plugins package to trigger @register decorators
from . import plugins  # noqa: F401

from .config import settings
from .dgraph import DGraphClient
from .lsp_client import LspClient
from .plugin_base import get_plugin, available_languages
from .indexer import (
    index_file as _index_file,
    index_workspace as _index_workspace,
)
from .watcher import watch_workspace

log = structlog.get_logger()


# ---------------------------------------------------------------------------
# Lifespan: DGraph + LSP + initial index + watcher
# ---------------------------------------------------------------------------

@asynccontextmanager
async def lifespan(server: Any) -> AsyncIterator[dict]:
    # --- DGraph ---
    dgraph = DGraphClient()
    for attempt in range(30):
        if await dgraph.health():
            break
        log.info("dgraph.waiting", attempt=attempt)
        await asyncio.sleep(2)
    else:
        raise RuntimeError("DGraph did not become healthy in time")
    await dgraph.apply_schema()

    # --- Language plugin ---
    plugin = get_plugin(settings.language, settings.workspace)
    log.info("plugin.loaded", language=plugin.name)

    # --- LSP server ---
    lsp = LspClient(plugin.lsp_command, settings.workspace)
    await lsp.start(
        init_options=plugin.lsp_init_options,
        timeout=settings.lsp_init_timeout,
    )

    # --- Full index ---
    await _index_workspace(dgraph, lsp, plugin)

    # --- File watcher ---
    watcher_task = asyncio.create_task(watch_workspace(dgraph, lsp, plugin))

    log.info("mcp.ready", language=plugin.name)
    yield {"dgraph": dgraph, "lsp": lsp, "plugin": plugin}

    # --- Shutdown ---
    watcher_task.cancel()
    try:
        await watcher_task
    except asyncio.CancelledError:
        pass
    await lsp.stop()
    await dgraph.close()


mcp = FastMCP(
    "code-graph",
    lifespan=lifespan,
    host=settings.mcp_host,
    port=settings.mcp_port,
)


def _dgraph() -> DGraphClient:
    return mcp.get_context().state["dgraph"]

def _lsp() -> LspClient:
    return mcp.get_context().state["lsp"]

def _plugin():
    return mcp.get_context().state["plugin"]


# ---------------------------------------------------------------------------
# Meta tool
# ---------------------------------------------------------------------------

@mcp.tool()
async def list_languages() -> str:
    """
    Return the available language plugins and the currently active one.

    Use to verify which language the graph server is indexing.
    """
    return json.dumps({
        "active": _plugin().name,
        "available": available_languages(),
    }, indent=2)


# ---------------------------------------------------------------------------
# Navigational tools  (unchanged signatures)
# ---------------------------------------------------------------------------

@mcp.tool()
async def find_symbol(name: str) -> str:
    """
    Exact symbol lookup by name.
    Returns all symbols with that name (there may be multiple in different modules).

    Use this first when you know the exact name of a function, struct, trait, or type.
    """
    results = await _dgraph().find_symbol(name)
    return json.dumps(results, indent=2)


@mcp.tool()
async def fuzzy_find(partial: str) -> str:
    """
    Find symbols whose name contains `partial` (case-insensitive, trigram match).
    Returns up to 20 results.

    Use when you have a partial name or aren't sure of the exact spelling.
    """
    results = await _dgraph().fuzzy_find(partial)
    return json.dumps(results, indent=2)


@mcp.tool()
async def get_file_symbols(file_path: str) -> str:
    """
    List all symbols defined in a file, with their signatures.
    Does NOT return file content — only symbol metadata.

    Useful for getting an overview of a file before deciding which symbol to edit.
    """
    results = await _dgraph().get_file_symbols(file_path)
    return json.dumps(results, indent=2)


@mcp.tool()
async def get_module_api(module_path: str) -> str:
    """
    Return all public symbols in a module path.
    Signatures only, no bodies.

    Use to understand a module's public contract before writing code that depends on it.
    """
    results = await _dgraph().get_module_api(module_path)
    return json.dumps(results, indent=2)


@mcp.tool()
async def get_module_tree() -> str:
    """
    Return the full module hierarchy of the codebase.

    Use for high-level project structure understanding (Architect / Orchestrator).
    """
    results = await _dgraph().get_module_tree()
    return json.dumps(results, indent=2)


# ---------------------------------------------------------------------------
# Impact analysis tools  (unchanged)
# ---------------------------------------------------------------------------

@mcp.tool()
async def get_callers(symbol_id: str, depth: int = 1) -> str:
    """
    Return all symbols that call this one, up to `depth` hops transitively.

    Use before modifying a function signature to understand what will break.
    symbol_id comes from find_symbol or get_edit_surface.
    """
    results = await _dgraph().get_callers(symbol_id, depth=min(depth, 3))
    return json.dumps(results, indent=2)


@mcp.tool()
async def get_callees(symbol_id: str, depth: int = 1) -> str:
    """
    Return all symbols that this one calls, up to `depth` hops transitively.

    Use to understand what a function depends on before refactoring it.
    """
    results = await _dgraph().get_callees(symbol_id, depth=min(depth, 3))
    return json.dumps(results, indent=2)


@mcp.tool()
async def get_type_usages(symbol_id: str) -> str:
    """
    Return all symbols that reference this type (struct, enum, trait, type alias).

    Use before changing a type's fields or variants to see all affected code.
    """
    results = await _dgraph().get_type_usages(symbol_id)
    return json.dumps(results, indent=2)


@mcp.tool()
async def get_implementors(trait_symbol_id: str) -> str:
    """
    Return all structs/types that implement this trait or interface.

    Use before changing a trait definition to see all implementors.
    """
    results = await _dgraph().get_implementors(trait_symbol_id)
    return json.dumps(results, indent=2)


@mcp.tool()
async def get_trait_dependents(trait_symbol_id: str) -> str:
    """
    Full blast radius of a trait/interface change: implementors AND type users.

    Use when changing method signatures or adding required methods.
    """
    results = await _dgraph().get_trait_dependents(trait_symbol_id)
    return json.dumps(results, indent=2)


@mcp.tool()
async def get_tests_for(symbol_id: str) -> str:
    """
    Return all test functions that cover this symbol.

    Use before and after an edit to know which tests to run.
    """
    results = await _dgraph().get_tests_for(symbol_id)
    return json.dumps(results, indent=2)


# ---------------------------------------------------------------------------
# Context assembly tools  (unchanged)
# ---------------------------------------------------------------------------

@mcp.tool()
async def get_edit_surface(symbol_id: str, depth: int = 1) -> str:
    """
    The recommended first call before making any edit.

    Returns a compact pre-edit read package:
      - the symbol itself (signature, doc, file, line)
      - its direct callees
      - its direct callers
      - the types it uses
      - tests covering it

    All as signatures only — no file content read.
    """
    result = await _dgraph().get_edit_surface(symbol_id, depth=depth)
    return json.dumps(result, indent=2)


@mcp.tool()
async def get_signature(symbol_id: str) -> str:
    """
    Return just the signature and doc comment of a symbol.

    Use when you need to reference a signature in generated code without
    reading the full file.
    """
    data = await _dgraph()._query(
        f'{{ q(func: eq(symbol_id, "{symbol_id}")) '
        f'{{ symbol_id symbol_name symbol_signature symbol_doc symbol_file symbol_line }} }}'
    )
    return json.dumps(data.get("q", []), indent=2)


# ---------------------------------------------------------------------------
# Structural tools  (unchanged)
# ---------------------------------------------------------------------------

@mcp.tool()
async def get_coupling_hotspots(top_n: int = 20) -> str:
    """
    Return the most-depended-upon symbols, ranked by total in-degree.

    High in-degree = high risk to modify. Use during architecture reviews.
    """
    results = await _dgraph().get_coupling_hotspots(top_n=min(top_n, 50))
    return json.dumps(results, indent=2)


@mcp.tool()
async def get_cross_module_boundary(module_a: str, module_b: str) -> str:
    """
    Return all call and type-usage edges crossing from module_a into module_b.

    Use to understand interface contracts or detect excessive coupling.
    """
    results = await _dgraph().get_cross_module_boundary(module_a, module_b)
    return json.dumps(results, indent=2)


@mcp.tool()
async def get_stats() -> str:
    """
    Return graph size metrics: number of indexed symbols and files.

    Use to verify the index is populated before relying on graph queries.
    """
    result = await _dgraph().stats()
    result["language"] = _plugin().name
    return json.dumps(result, indent=2)


# ---------------------------------------------------------------------------
# Indexing tools  (updated signatures — now language-agnostic)
# ---------------------------------------------------------------------------

@mcp.tool()
async def index_file(file_path: str) -> str:
    """
    Re-index a single source file. Language is determined by the active plugin.

    The file watcher handles this automatically on save.
    Call manually if you suspect the index is stale for a specific file.
    """
    try:
        await _index_file(_dgraph(), _lsp(), _plugin(), file_path)
        return json.dumps({"status": "ok", "file": file_path})
    except Exception as e:
        return json.dumps({"status": "error", "file": file_path, "error": str(e)})


@mcp.tool()
async def index_workspace() -> str:
    """
    Trigger a full workspace re-index.

    Run automatically on server startup. Call manually after large refactors
    or if the graph seems inconsistent.
    """
    try:
        await _index_workspace(_dgraph(), _lsp(), _plugin())
        stats = await _dgraph().stats()
        return json.dumps({"status": "ok", **stats})
    except Exception as e:
        return json.dumps({"status": "error", "error": str(e)})