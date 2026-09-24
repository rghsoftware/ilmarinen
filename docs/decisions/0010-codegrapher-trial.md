# 0010 Codegrapher is a CLI-only trial

Status: accepted · Date: 2026-09-24

## Context
Codegrapher (same organisation as SpecScore, Apache-2.0, single Go binary, SQLite symbol graph) offers `impact`, `affected`, `callers` and symbol-level attachment of Source References. Serena already provides LSP-backed navigation. Usefulness cannot be judged without trying it.

## Decision
Install it as a CLI, gitignore `.codegraph/`, never register it as an MCP server. Trial it on a brownfield repo for a few weeks and judge on whether the agent reaches for it over Serena and `rg`. Nothing may depend on its `.codegraph/` store.

## Consequences
- The line-level Source Reference scanner works without it.
- Promote to always-on only via a superseding Rune and a measured context cost.
