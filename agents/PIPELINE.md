# pipeline constitution

Every agent in this pipeline reads this file before doing anything else.
Rules here override anything in an individual agent's system prompt.

---

## Human in the loop

Never make heavy assumptions in silence. If your inputs are ambiguous on a point
that would materially change your output, stop and ask for clarification before
producing anything.

When asking, list all blocking questions at once — never one at a time across
multiple turns. Resume only after receiving answers.

Proceed without asking only when the ambiguity is minor enough to surface safely
in your output (e.g. as an open question or a flagged assumption), rather than
something that would invalidate your entire output if you got it wrong.

---

## Orchestrator protocol

You are invoked by an orchestrator that routes tasks across the pipeline. Your
immediate reader is the orchestrator, not a human.

When your work is complete — or when you need to pause — respond with this block
and nothing else around it:

```
AGENT: <your agent name>
STATUS: done | blocked | needs_clarification
OUTPUT: <path(s) written, or "none">
SUMMARY: <2–3 sentences: what you did, what the key output contains, and the most
important thing the next agent or human needs to know>
QUESTIONS: <numbered list — only present when STATUS is needs_clarification>
```

The orchestrator parses this programmatically. Do not add prose before or after it.
If STATUS is `needs_clarification`, do not write any output file — wait for answers.

---

## File system rules

| Path | Rule |
|------|------|
| `CLAUDE.md` | Read only — never modify |
| `README.md` | Read only |
| `docs/` | Planning agents write here |
| `src/` | Implementer agent only — all others read only |
| `BACKLOG.md` | Tracker agent only — all others read only |

When in doubt about whether you are allowed to write somewhere, you are not.

---

## Tool defaults

These apply to every agent unless its own system prompt explicitly overrides them:

- Bash execution — forbidden
- Git operations — forbidden
- Web search — allowed for research, forbidden for fetching code to copy