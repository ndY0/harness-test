# PIPELINE.md — Pipeline constitution

Every agent in this pipeline reads this file before doing anything else.
All rules here apply unconditionally unless an agent's own prompt explicitly
overrides one with a narrower rule for its specific role.

---

## 1. Identity rule

You are the agent named in your own system prompt. You are not the Orchestrator.
You are not any other agent. Do not act outside your defined role.

---

## 2. Human-in-the-loop policy

If you encounter a material ambiguity — something that would cause you to make
a consequential assumption about scope, technology, or behaviour — stop and
return STATUS: needs_clarification. Do not guess. Do not proceed.

A material ambiguity is one where two reasonable interpretations would lead to
meaningfully different outputs. Spelling, formatting, and minor phrasing choices
are not material.

---

## 3. Orchestrator response protocol

Every agent response must end with a response block in exactly this format:

```
AGENT: <your agent name>
STATUS: <done | blocked | needs_clarification>
OUTPUT: <path(s) to file(s) written, or "none">
SUMMARY: <two to four sentences describing what was done or why you are blocked>
QUESTIONS: <numbered list of clarification questions, or "none">
```

Rules:
- If STATUS is `needs_clarification`, do not write any output files.
- If STATUS is `blocked`, write a partial output if useful but mark it clearly
  as incomplete at the top of the file.
- Do not append commentary after the response block.

---

## 4. File ownership

Each agent owns exactly the files listed in its own prompt. Writing to a file
you do not own is a pipeline violation. When in doubt, return `blocked` and
name the file that needs to be written by another agent.

---

## 5. Single-dispatch rule

The Orchestrator dispatches exactly one backlog feature per agent invocation.
No agent should ever receive or act on more than one feature at a time.
If you receive multiple features in a single invocation, process only the first
and flag the rest in your QUESTIONS block.

---

## 6. Review severity classification

The Reviewer agent classifies every finding as either BLOCKING or NON_BLOCKING.

**BLOCKING** — must be resolved before the feature can advance:
- Acceptance criterion not met
- Functional regression introduced
- Security or data integrity issue
- Contract violation (API schema, event format, shared interface)

**NON_BLOCKING** — desirable but does not block advancement:
- Style or structural improvements
- Naming suggestions
- Test coverage expansion beyond the spec's acceptance criteria
- Refactoring opportunities

The Implementer re-loops only on BLOCKING findings.
NON_BLOCKING findings are logged by the Orchestrator for future backlog items.

---

## 7. Iteration cap

Each feature has a maximum of **3 Implementer ↔ Reviewer cycles**.

The Tracker maintains an `iterations` counter per feature in BACKLOG.md.
If `iterations` reaches 3 without the Reviewer returning `approved`, the
Orchestrator escalates to the human with:
- The current implementation state
- The full list of BLOCKING findings from the last review
- The delta between what the Implementer produced and what the Reviewer demands

The human decides whether to proceed, adjust the spec, or abandon the feature.
No agent may unilaterally break the escalation — only the human can resume.

---

## 8. Architect tier

Architecture work in this pipeline has two tiers:

**Master Architect** — owns the full system topology, cross-cutting standards,
ADRs, and domain charters. Is the sole decision authority in any cross-domain
review.

**Domain Architect** — owns a single bounded context. Designs within the
constraints defined by the Master Architect's charter for that domain.
May not make decisions that affect shared contracts without Master Architect
approval.

The fast-path rule: if a feature is entirely internal to one domain and touches
no shared contracts or cross-domain interfaces, it goes directly to the Domain
Architect and Spec Writer. The Master Architect is bypassed.

The escalation rule: if a Domain Architect identifies a cross-domain dependency,
interface change, or standard violation during design, it must escalate to the
Master Architect before producing output.

---

## 9. Supersession protocol

This rule applies to **every agent that produces a document which replaces an
existing one** — a Spec Writer writing a new version of a spec, a Master
Architect issuing an ADR that overrides a previous decision, a Domain Architect
revising a domain architecture, and so on.

When you write a document that replaces an existing document, you must, in the
same turn, update the **old** document's front-matter:

- `status: superseded`
- `superseded_by: <path to the new document>`
- `superseded_date: <today's date, ISO 8601>`

You do not delete the old document. You only mark it. Physical removal is the
Archivist's job, and it happens only after a grace period.

Every document carries front-matter from creation:

```
---
type: <spec | adr | architecture | charter | brainstorm | review | eval>
domain: <domain name, or "global">
feature: <feature slug, or "none">
status: active
date: <ISO 8601 creation date>
superseded_by: none
superseded_date: none
---
```

The Archivist acts on `status` and `superseded_date`. It never sets them — they
are set here, by the agent that supersedes the document. If you produce a
replacement and forget to mark the old document, it will linger as active truth
forever and the corpus will only grow. Marking supersession is not optional.

---

## 10. Retrieval layer (document search)

The project's documents are indexed in a retrieval service reachable over MCP as
the `doc-retrieval` connector. Use it to find and read existing project
knowledge instead of reading the whole `docs/` tree. As the project ages this is
what keeps your context bounded — searching returns only the handful of relevant
sections rather than every document ever written.

### Tools available to all agents

- `search(query, type?, domain?, include_deprecated?, limit?)` — find the
  document sections most relevant to a query. Filters by metadata first (active
  status, and optionally a `type` or `domain`), then ranks by meaning. Returns
  section-level hits with a `path`, a `section_anchor`, a `score`, a `snippet`,
  and a `blob_sha` — not full documents.
- `get_content(path, blob_sha?)` — read a document's full text. It resolves
  whether the document is on the working tree or only in git history, so it works
  even for archived/deleted docs. Pass a `blob_sha` (from a search hit) to fetch a
  specific historical version.
- `list_active(type?, domain?)` — a manifest-style list of active documents, one
  entry per document. The cheap "what is current truth" query.

(`index_document` and `mark_deleted` also exist but are maintenance tools — only
the Archivist calls them. Do not call them from any other role.)

### How to read documents

1. To discover relevant context beyond the specific files your own prompt names,
   prefer `search` or `list_active` over reading the entire tree. Then pull the
   full text of a promising hit with `get_content`.
2. The specific input files your prompt names (e.g. "read the feature spec at
   `docs/specs/<slug>.md`") you may still read directly from the filesystem.
   Search is for *discovery*, not for replacing those direct reads.
3. You see **active truth only** by default. Pass `include_deprecated: true` only
   when you deliberately need superseded or historical context — and say so in
   your SUMMARY when you do.

### Two boundaries that still bind

- **Scope restrictions are not widened by the retrieval layer.** If your own
  prompt forbids you from reading a document type (for example, the Reviewer must
  not read brainstorms or architecture), `search` does not override that — do not
  use it to reach material your role is told to ignore.
- **`MANIFEST.md` and `list_active` should agree.** `MANIFEST.md` is the
  git-tracked, always-available list the Archivist regenerates each cycle; it is
  the right first read for a quick overview and works even if the retrieval
  service is down. `list_active` is the live, filterable equivalent. Prefer the
  manifest for a glance; use `list_active` when you need to filter by type or
  domain.