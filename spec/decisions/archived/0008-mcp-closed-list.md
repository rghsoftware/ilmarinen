---
format: https://specscore.md/decision-specification
status: Superseded
---
# Decision: Always-on MCP servers are a closed list of six

**Status:** Superseded
**Date:** 2026-09-24
**Owner:** rghsoftware
**Tags:** mcp, context-budget
**Source Idea:** —
**Supersedes:** —
**Superseded By:** 0011-always-on-mcp-five-servers

## Context

Tool schemas cost context on every turn where they are loaded eagerly; Claude Code now defers most of it, other hosts may not. Serena and Codegrapher overlap; Playwright and Chrome DevTools MCPs overlap.

## Decision

Always-on: Serena, Context7, Playwright, Basic Memory, Beads (if it exposes MCP), Sampo. Everything else is CLI. Adding a server requires a new Rune and a re-measured session-start budget.

## Rationale

Schemas cost context wherever they load eagerly, and overlapping servers pay that cost twice for one capability.

## Declined Alternatives

### An open list of servers

Overlapping servers (Serena and Codegrapher; Playwright and Chrome DevTools) cost context on every turn where schemas load eagerly.

## Consequences at Decision Time

- Codegrapher is CLI-only (see 0010).
- The Sampo server exposes exactly four read-only tools.

## Observed Consequences

None observed yet.

## Affected Features

None at this time.

---

## Resolution

Beads MCP dropped; five servers (D-0011)
*This document follows the https://specscore.md/decision-specification*
