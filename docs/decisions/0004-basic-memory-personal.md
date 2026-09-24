# 0004 Basic Memory is the personal memory store, separate from all repos

Status: accepted · Date: 2026-09-24

## Context
Facts about the developer, their machines and preferences must be available from every host and must never leak into repos. Claude Code's native memory is per-project and host-locked. LLM-extraction stores (Mem0, Hindsight, Graphiti) put a model in the write path and compete for a 16 GB GPU.

## Decision
Use Basic Memory (Markdown notes, local SQLite index, MCP, works from Claude Code and OpenCode) as a private git repo at `$HOME/agent-memory`. A `pinned.md` of at most 30 lines is the only auto-loaded personal context. A pre-commit leak guard blocks personal facts from entering any repo.

## Consequences
- AGPL-3.0 is fine because Basic Memory is called over MCP, never distributed or vendored.
- Automatic capture is not provided; add it later only into an inbox for human approval.
- Personal memory and project knowledge are different stores with different tool names; never one store with namespaces.
