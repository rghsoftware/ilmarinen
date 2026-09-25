---
format: https://specscore.md/decision-specification
status: Approved
---
# Decision: Plain text in git plus a rebuildable index beats every context store today

**Status:** Approved
**Date:** 2026-09-24
**Owner:** rghsoftware
**Tags:** context, index, sampo
**Source Idea:** —
**Supersedes:** —
**Superseded By:** —

## Context

Evaluated OpenViking, OpenDeepWiki, Sibyl, Graphiti, Cognee, Hindsight, Mem0, Letta, claude-mem and others against a baseline of per-repo Markdown plus a cross-repo SQLite FTS5 index over MCP. For 2–4 related repos, none justified its footprint. Published studies (ETH Zurich, arXiv 2602.11988; Khatri 2026) show context files do not reliably improve agent outcomes and raise cost, so auto-loaded context must stay minimal.

## Decision

Authoritative knowledge is committed Markdown (Blueprints, Runes). Derived knowledge is generated into Artifacts and never committed. The Sampo index (`$HOME/.cache/ilmarinen/sampo.db`) is rebuilt from the repos in `repos.toml` and exposed through four read-only MCP tools. Session-start context target: under ~3K tokens.

## Rationale

For 2–4 related repos no evaluated store justified its footprint over the plain-text baseline, and auto-loaded context raises cost without reliably improving outcomes.

## Declined Alternatives

### A dedicated context store

OpenViking, OpenDeepWiki, Sibyl, Graphiti, Cognee, Hindsight, Mem0, Letta, claude-mem and others. For 2–4 related repos, none justified its footprint.

## Consequences at Decision Time

- Cannot infer cross-repo concept relationships or temporal validity beyond explicit `supersedes`/`affects` and Source References.
- Upgrade triggers: frequent "what was true when" questions → Graphiti fed from the same files; >~2K docs or multimodal corpus → OpenViking. The files never change.

## Observed Consequences

None observed yet.

## Affected Features

None at this time.

---
*This document follows the https://specscore.md/decision-specification*
