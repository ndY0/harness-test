# PIPELINE.md — Pipeline constitution

Every agent reads this file first. All rules apply unconditionally unless an agent’s own prompt overrides with a narrower rule.

## 0. Frugality

Keep output as brief as possible without losing required information; omit all non‑essential elaboration.
write files that are as lean as possible

## 1. Identity rule
You are the agent named in your own system prompt — not the Orchestrator, not any other agent. Stay within your defined role.

## 2. Human-in-the-loop policy
On **material ambiguity** (two reasonable interpretations produce materially different outputs), stop and return `STATUS: needs_clarification`. Do not guess. Spelling, formatting, and minor phrasing are not material.

## 3. Orchestrator response protocol
Every agent response must end with exactly:

```
AGENT: <your agent name>
STATUS: <done | blocked | needs_clarification>
OUTPUT: <path(s) to file(s) written, or "none">
SUMMARY: <two to four sentences describing what was done or why you are blocked>
QUESTIONS: <numbered list of clarification questions, or "none">
```


- `needs_clarification` → write no output files.
- `blocked` → may write partial output marked incomplete at file top.
- No commentary after the block.

## 4. File ownership
Write only the files listed in your own prompt. Writing another agent’s file is a violation. If unsure, return `blocked` and name the file that needs another agent.

## 5. Single-dispatch rule
Orchestrator dispatches **exactly one** backlog feature per invocation. If you receive multiple features, process only the first and list the rest in `QUESTIONS`.

## 6. Review severity classification
Reviewer classifies every finding as **BLOCKING** or **NON_BLOCKING**.

**BLOCKING** – must resolve before advancing:
- Acceptance criterion not met
- Functional regression
- Security/data integrity issue
- Contract violation (API schema, event format, shared interface)

**NON_BLOCKING** – desirable but not blocking:
- Style/structure suggestions, naming, test expansion beyond spec, refactoring

Implementer re-loops only on BLOCKING findings. NON_BLOCKING findings are logged by the Orchestrator for future backlog items.

## 7. Iteration cap
Max **3** Implementer ↔ Reviewer cycles per feature. Tracker maintains an `iterations` counter in BACKLOG.md. If 3 cycles occur without `approved`, Orchestrator escalates to the human with:

- Current implementation state
- Full BLOCKING findings from last review
- Delta between Implementer output and Reviewer demand

Only the human can resume. No agent unilaterally breaks the escalation.

## 8. Architect tier
Two tiers:

- **Master Architect** – full system topology, cross-cutting standards, ADRs, domain charters; sole authority in cross-domain decisions.
- **Domain Architect** – owns a single bounded context; must escalate any cross-domain dependency, interface change, or standard violation to the Master Architect.

**Fast path**: a feature entirely internal to one domain (no shared contracts) goes directly to Domain Architect and Spec Writer, bypassing Master Architect.

## 9. Supersession protocol
When you write a document that replaces an existing one, **in the same turn** update the old document’s front-matter:


- `status: superseded`
- `superseded_by: <path to the new document>`
- `superseded_date: <today's date, ISO 8601>`


Do **not** delete the old document; Archivist removes it later. Every document carries front-matter:


```
---
type: <spec | adr | architecture | charter | brainstorm | review | eval>
domain: <domain name, or "global">
feature: <feature slug, or "none">
status: active
date: <ISO 8601 creation date>
superseded_by: none
superseded_date: none
complexity: simple | complex
complexity_rationale: "explanaition"
---
```


Archivist acts on `status` and `superseded_date` but never sets them. Marking supersession is mandatory.

## 10. Retrieval layer (document search)
Use the retrieval service (`doc-retrieval` MCP) instead of reading the whole `docs/` tree.

**Tools available to all agents:**
- `search(query, type?, domain?, include_deprecated?, limit?)` – returns section-level hits (path, snippet, score, blob_sha)
- `get_content(path, blob_sha?)` – full document text
- `list_active(type?, domain?)` – active document manifest

(`index_document` and `mark_deleted` are Archivist-only.)

**How to read:**
1. For discovery beyond prompt-named files, use `search`/`list_active`; then `get_content`.
2. Prompt-named input files may be read directly.
3. Default is active-only; use `include_deprecated: true` only when you deliberately need historical context — note in SUMMARY.

**Boundaries:** scope restrictions still apply (do not use retrieval to reach forbidden material). `MANIFEST.md` and `list_active` should agree; prefer `MANIFEST.md` for a quick overview.

## §11 — Jira integration (optional)
Jira is never required. All workflows work locally. Bridge is a progressive enhancement.

**Detection:** call `list_synced_features`. If it returns `"error": "jira_not_configured"`, proceed as if Jira doesn’t exist.

**Key tools (jira-bridge MCP):**
- `get_ticket` (Spec Writer)
- `search_tickets` (Orchestrator)
- `get_ticket_comments` (Spec Writer, Reviewer)
- `transition_ticket` (Orchestrator, Tracker) – strings: `"In Progress"`, `"In Review"`, `"Done"`, `"Awaiting Clarification"`
- `add_comment` (Reviewer, Evaluator)
- `set_field` (Evaluator)
- `link_local_feature_tool` (Spec Writer)
- `get_sync_status` (Tracker)
- `request_clarification` (Reviewer)
- `list_synced_features` (Orchestrator)

**Clarification parking:** when `request_clarification` returns `{ "parked": true }`, Orchestrator sets feature as `PARKED / Awaiting Clarification` in BACKLOG.md, moves on, and later checks `get_sync_status` for new comments.

**Local sidecar:** `jira_ref.json` in feature dir (never edit directly).

**Offline:** never surface `jira_not_configured` errors; never block on Jira.

# §12 — Feature complexity and Planner-driven parallelism

**Complexity** (front-matter field, set once by Spec Writer):
- `simple` → dispatched to Implementer
- `complex` → dispatched to Planner

**Planner’s contract:**
- Decomposes into sub-tasks with disjoint write sets
- Creates worktrees under `.worktrees/` (in `.gitignore`), branches `feat/<subtask-id>`
- Dispatches Implementers (or child Planners) in parallel
- Reviews each sub-task, merges, resolves conflicts, runs post-merge tests
- Reports to Orchestrator like an Implementer

**Nesting depth:** max 2 (0,1,2). At depth 2, all sub-tasks are treated as `simple`.

**Worktree cleanup:** Planner removes all worktrees it created on completion.

**Artifacts:**
- Sub-task manifests: `docs/plans/<feature-id>/<subtask-id>.md`
- Post-merge test output: `docs/plans/<feature-id>/post-merge-test.txt`

**Agent read policy:** Tracker, Reviewer, Evaluator read only the merged result. Exception: Reviewer is also invoked on each sub-task branch before merge, with scope restricted to that sub-task’s write set.

# §13 — Code Graph MCP (LSP-backed)

All agents may use the Code Graph MCP for semantic code analysis. It provides
real-time LSP data without requiring agents to manually grep or recursively
read files.

**Purpose**: Resolve dependencies, inspect call hierarchies, find type usages,
and calculate safe edit surfaces before modifying code.

**Allowed Agents**: Planner, Implementer, Reviewer (read‑only for all; no
write/modify operations on the codebase itself).

**Key tools (grouped)**:

| Category | Tools |
|----------|-------|
| Discovery | `list_languages`, `get_stats`, `get_file_symbols` |
| Navigation | `find_symbol`, `fuzzy_find` |
| Structure | `get_module_tree`, `get_module_api` |
| Dependency | `get_callers`, `get_callees`, `get_cross_module_boundary` |
| Typing | `get_type_usages`, `get_implementors`, `get_trait_dependents` |
| Safety | `get_edit_surface`, `get_signature`, `get_tests_for` |
| Indexing | `index_file`, `index_workspace` (Planner/Implementer may trigger on new files only) |

**Usage rules**:
- **Before editing any existing file**, the Implementer **must** call
  `get_callers(symbol)` on the public function/type being modified to assess
  impact. If callers exist outside the current feature's write set, the
  Implementer must return `blocked` and escalate to Orchestrator.
- **During decomposition**, the Planner **must** use `get_coupling_hotspots`
  and `get_cross_module_boundary` to validate that sub-task write sets are
  truly disjoint. If two sub-tasks touch files that are tightly coupled by
  direct calls, merge those sub-tasks or mark a sequenced dependency.
- **To determine what to change**, use `get_edit_surface(file)` to receive a
  minimal set of symbols that actually need modification — honour this list.
- **Indexing**: assume the workspace is already indexed. Only call
  `index_file(path)` if the file is freshly created in your current worktree
  and `get_file_symbols` returns stale or empty data for it.
- **Language context**: call `list_languages` to verify the active LSP plugin
  matches the language of the files you are analysing.