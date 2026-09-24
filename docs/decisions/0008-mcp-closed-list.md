# 0008 Always-on MCP servers are a closed list of six

Status: accepted · Date: 2026-09-24

## Context
Tool schemas cost context on every turn where they are loaded eagerly; Claude Code now defers most of it, other hosts may not. Serena and Codegrapher overlap; Playwright and Chrome DevTools MCPs overlap.

## Decision
Always-on: Serena, Context7, Playwright, Basic Memory, Beads (if it exposes MCP), Sampo. Everything else is CLI. Adding a server requires a new Rune and a re-measured session-start budget.

## Consequences
- Codegrapher is CLI-only (see 0010).
- The Sampo server exposes exactly four read-only tools.
