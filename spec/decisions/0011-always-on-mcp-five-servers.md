---
format: https://specscore.md/decision-specification
status: Approved
---

# Decision: Always-on MCP servers are a closed list of five; Beads is CLI only

**Status:** Approved
**Date:** 2026-09-24
**Owner:** rghsoftware
**Tags:** mcp, context-budget, beads
**Source Idea:** —
**Supersedes:** 0008-mcp-closed-list
**Superseded By:** —

## Context

Decision 0008 listed six always-on servers, including "Beads (if it exposes MCP)". Phase 0 verification found that Beads does expose one (PyPI `beads-mcp`), and that upstream documents its schemas at 10–50k tokens and recommends the `bd` CLI plus `bd prime` for Claude Code (https://github.com/gastownhall/beads/blob/main/docs/integrations/mcp-server.md). Claude Code defers MCP schemas; OpenCode and other hosts may load them eagerly, which alone would exceed the ~3K session-start budget (decision 0005). `bd` already emits `--json` and is agent-friendly on the command line.

## Decision

Always-on MCP servers: Serena, Context7, Playwright, Basic Memory, Sampo. Beads is used through the `bd` CLI only and is never registered as an MCP server. Everything else remains CLI. Adding a server still requires a new Rune and a re-measured session-start budget.

## Rationale

Upstream recommends the CLI, the MCP schema cost breaks the context budget in any host that loads schemas eagerly, and the CLI already gives agents structured output. Nothing in the workflow needs Beads over MCP.

## Declined Alternatives

### Keep Beads MCP as the sixth server

Matches 0008 as written. Costs 10–50k tokens wherever schemas load eagerly, against a ~3K budget, for no capability the CLI lacks.

## Consequences at Decision Time

- `plugin/.mcp.json` declares five servers.
- Skills call `bd ... --json` through the shell; `bd` must be on PATH (checked by `doctor`).
- The Sampo server still exposes exactly four read-only tools; Codegrapher stays CLI-only (0010).

## Observed Consequences

- 2026-09-24: In Claude Code, `ilmarinen setup` installs the Ilmarinen plugin at user scope and the plugin's own `.mcp.json` starts the servers; it never calls `claude mcp add`, which would start each server twice. OpenCode gets the same servers written into its global `opencode.json`.

## Affected Features

None at this time.

---
*This document follows the https://specscore.md/decision-specification*
