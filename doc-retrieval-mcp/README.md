# Document-retrieval MCP server

A local retrieval layer for the agent pipeline. It indexes the pipeline's
markdown docs (specs, ADRs, architecture, charters, reviews) into a vector store
and exposes hybrid search + history-aware content resolution to agents over MCP.

## What it does

- **Hybrid search** — filters by metadata (status, type, domain) first, then
  ranks by semantic similarity. Active docs only, by default.
- **History-aware retrieval** — stores each doc's git blob SHA, so a document
  archived and removed from the working tree is still resolvable from git
  history via a single `git cat-file`.
- **Index maintenance** — `index_document` / `mark_deleted` are called by the
  Archivist agent after each feature cycle.

## Architecture

```
Agents ──(MCP / streamable-http)──► doc-mcp ──► Qdrant (vectors + payload)
                                       │
                                       ├─ reads docs from /repo (read-only mount)
                                       └─ resolves deleted docs via git cat-file
```

- **Qdrant** — open-source vector store; payload filtering gives us the
  filter-then-rank hybrid in one query.
- **fastembed** — local ONNX embeddings (`bge-small-en-v1.5`, 384-dim). No
  external embedding API, no per-call cost.
- **FastMCP** — exposes the tools over streamable-HTTP at `:8000/mcp`.

## Tools

| Tool | Who calls it | Purpose |
|------|--------------|---------|
| `search` | any agent | Find relevant doc sections |
| `get_content` | any agent | Read a doc (working tree or history) |
| `index_document` | Archivist | (Re)index a created/modified doc |
| `mark_deleted` | Archivist | Tombstone a doc before `git rm` |
| `list_active` | any agent | Manifest view of active docs |

## Setup

1. Point the server at your project repo:

   ```bash
   cp .env.example .env
   # edit .env, set REPO_HOST_PATH to your project's absolute path
   ```

2. Build and start:

   ```bash
   docker compose up --build -d
   ```

   Qdrant's dashboard: http://localhost:6333/dashboard
   MCP endpoint:       http://localhost:8000/mcp

3. Wire it into Claude Code (so your agents can reach it). In your project:

   ```bash
   claude mcp add --transport http doc-retrieval http://localhost:8000/mcp
   ```

   or add to `.mcp.json`:

   ```json
   {
     "mcpServers": {
       "doc-retrieval": {
         "type": "http",
         "url": "http://localhost:8000/mcp"
       }
     }
   }
   ```

## First-time indexing

The store starts empty. To backfill the existing corpus, have an agent (or a
quick script) call `index_document` for each doc under `docs/`. After that, the
Archivist keeps it current on every cycle.

## Notes

- The repo is mounted **read-only**; the server never writes to your project.
  All git mutations (`git rm`, commits) happen on the host, done by the Archivist.
- Pin the `qdrant/qdrant` image tag in `docker-compose.yml` for reproducibility.
- To reset the index, `docker compose down -v` (drops the Qdrant volume).
