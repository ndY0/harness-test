#!/usr/bin/env python3
"""Backfill the retrieval index from the existing document corpus.

Walks the docs directory of your project repo and calls the `index_document`
tool on the running MCP server once per markdown file. Because it goes through
the server over MCP, it reuses the exact indexing logic (chunking, embedding,
blob-SHA capture, upsert) — no logic is duplicated here.

The server reads files relative to its own `/repo` mount, so this script passes
*repo-relative* paths (e.g. "docs/specs/foo.md"), computed from --repo.

Usage:
    # from your project repo root, with `docker compose up` already running:
    python scripts/backfill.py --repo .

    # or point at the repo explicitly and a non-default server:
    python scripts/backfill.py --repo /path/to/project --url http://localhost:8000/mcp

    # see what would be indexed without touching the server:
    python scripts/backfill.py --repo . --dry-run

Requires the MCP client library on the host:
    pip install "mcp>=1.27,<2"
"""
import argparse
import asyncio
import sys
from pathlib import Path


def find_docs(repo: Path, docs_subdir: str) -> list[str]:
    """Return sorted repo-relative paths of every .md file under docs_subdir."""
    docs_root = repo / docs_subdir
    if not docs_root.is_dir():
        raise SystemExit(f"error: '{docs_root}' is not a directory")
    files = sorted(
        p.relative_to(repo).as_posix()
        for p in docs_root.rglob("*.md")
        if p.is_file()
    )
    return files


async def backfill(url: str, paths: list[str]) -> int:
    # Imported here so --dry-run works without the mcp client installed.
    from mcp import ClientSession
    from mcp.client.streamable_http import streamablehttp_client

    indexed = 0
    failed: list[tuple[str, str]] = []

    async with streamablehttp_client(url) as (reader, writer, _):
        async with ClientSession(reader, writer) as session:
            await session.initialize()
            total = len(paths)
            for i, rel_path in enumerate(paths, start=1):
                try:
                    result = await session.call_tool(
                        "index_document", {"path": rel_path}
                    )
                    summary = _summarize(result)
                    print(f"[{i}/{total}] {rel_path} -> {summary}")
                    indexed += 1
                except Exception as exc:  # keep going on per-file failures
                    print(f"[{i}/{total}] {rel_path} -> FAILED: {exc}",
                          file=sys.stderr)
                    failed.append((rel_path, str(exc)))

    print(f"\nDone. Indexed {indexed}/{len(paths)} document(s).")
    if failed:
        print(f"{len(failed)} failed:", file=sys.stderr)
        for path, err in failed:
            print(f"  - {path}: {err}", file=sys.stderr)
        return 1
    return 0


def _summarize(result) -> str:
    """Pull a readable summary out of an MCP CallToolResult."""
    # FastMCP returns the tool's dict as structured content when available.
    structured = getattr(result, "structuredContent", None)
    if structured:
        chunks = structured.get("indexed_chunks")
        status = structured.get("status")
        return f"{chunks} chunk(s), status={status}"
    # Fall back to the first text content block.
    for block in getattr(result, "content", []) or []:
        text = getattr(block, "text", None)
        if text:
            return text
    return "ok"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", default=".",
                        help="Path to the project repo root (default: .)")
    parser.add_argument("--docs-subdir", default="docs",
                        help="Docs directory relative to the repo (default: docs)")
    parser.add_argument("--url", default="http://localhost:8000/mcp",
                        help="MCP server URL (default: http://localhost:8000/mcp)")
    parser.add_argument("--dry-run", action="store_true",
                        help="List the files that would be indexed, then exit")
    args = parser.parse_args()

    repo = Path(args.repo).resolve()
    paths = find_docs(repo, args.docs_subdir)

    if not paths:
        print(f"No .md files found under {repo / args.docs_subdir}")
        return

    if args.dry_run:
        print(f"Would index {len(paths)} file(s) from {repo}:")
        for p in paths:
            print(f"  {p}")
        return

    exit_code = asyncio.run(backfill(args.url, paths))
    sys.exit(exit_code)


if __name__ == "__main__":
    main()