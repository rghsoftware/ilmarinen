---
format: https://specscore.md/decision-specification
status: Approved
---
# Decision: Codegrapher is a CLI-only trial

**Status:** Approved
**Date:** 2026-09-24
**Owner:** rghsoftware
**Tags:** codegrapher, trial, mcp
**Source Idea:** —
**Supersedes:** —
**Superseded By:** —

## Context

Codegrapher (same organisation as SpecScore, Apache-2.0, single Go binary, SQLite symbol graph) offers `impact`, `affected`, `callers` and symbol-level attachment of Source References. Serena already provides LSP-backed navigation. Usefulness cannot be judged without trying it.

## Decision

Install it as a CLI, gitignore `.codegraph/`, never register it as an MCP server. Trial it on a brownfield repo for a few weeks and judge on whether the agent reaches for it over Serena and `rg`. Nothing may depend on its `.codegraph/` store.

## Rationale

Usefulness cannot be judged without trying it, and a CLI trial costs no session-start context.

## Declined Alternatives

### Always-on MCP server

Overlaps Serena and costs context; promotion requires a superseding Rune and a measured context cost.

### Skip it

Its `impact`, `affected` and `callers` commands may beat Serena and `rg`; that cannot be judged without a trial.

## Consequences at Decision Time

- The line-level Source Reference scanner works without it.
- Promote to always-on only via a superseding Rune and a measured context cost.

## Observed Consequences

None observed yet.

## Affected Features

None at this time.

---
*This document follows the https://specscore.md/decision-specification*
