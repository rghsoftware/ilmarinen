---
format: https://specscore.md/decision-specification
status: Superseded
---
# Decision: Basic Memory is the personal memory store, separate from all repos

**Status:** Superseded
**Date:** 2026-09-24
**Owner:** rghsoftware
**Tags:** memory, privacy, basic-memory
**Source Idea:** —
**Supersedes:** —
**Superseded By:** 0013-personal-memory-how-i-work

## Context

Facts about the developer, their machines and preferences must be available from every host and must never leak into repos. Claude Code's native memory is per-project and host-locked. LLM-extraction stores (Mem0, Hindsight, Graphiti) put a model in the write path and compete for a 16 GB GPU.

## Decision

Use Basic Memory (Markdown notes, local SQLite index, MCP, works from Claude Code and OpenCode) as a private git repo at `$HOME/agent-memory`. A `pinned.md` of at most 30 lines is the only auto-loaded personal context. A pre-commit leak guard blocks personal facts from entering any repo.

## Rationale

Basic Memory is plain Markdown with a local index and works from both hosts over MCP, with no model in the write path.

## Declined Alternatives

### Claude Code native memory

Per-project and host-locked.

### LLM-extraction stores (Mem0, Hindsight, Graphiti)

Put a model in the write path and compete for a 16 GB GPU.

## Consequences at Decision Time

- AGPL-3.0 is fine because Basic Memory is called over MCP, never distributed or vendored.
- Automatic capture is not provided; add it later only into an inbox for human approval.
- Personal memory and project knowledge are different stores with different tool names; never one store with namespaces.

## Observed Consequences

None observed yet.

## Affected Features

None at this time.

---

## Resolution

Personal memory holds how I work, never what I own (D-0013)
*This document follows the https://specscore.md/decision-specification*
