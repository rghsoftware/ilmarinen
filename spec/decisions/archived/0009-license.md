---
format: https://specscore.md/decision-specification
status: Superseded
---
# Decision: License: Apache-2.0

**Status:** Superseded
**Date:** 2026-09-24
**Owner:** rghsoftware
**Tags:** license
**Source Idea:** —
**Supersedes:** —
**Superseded By:** 0012-license-dependency-inventory

## Context

The package is tooling. Dependencies: Beads (MIT), Serena (MIT), specscore-cli and Codegrapher (Apache-2.0), SpecScore spec text (CC-BY-4.0), Basic Memory (AGPL-3.0, called over MCP only), Compound Engineering (MIT, pattern only).

## Decision

Apache-2.0: explicit patent grant, clear contribution terms, matches the nearest neighbours.

## Rationale

Explicit patent grant, clear contribution terms, and it matches the nearest neighbours (specscore-cli, Codegrapher).

## Declined Alternatives

### MIT

Used by several dependencies, but carries no explicit patent grant.

## Consequences at Decision Time

- Never vendor Basic Memory source.
- Any SpecScore template text copied verbatim must carry CC-BY attribution; prefer writing our own templates.
- This is a practical reading, not legal advice.

## Observed Consequences

None observed yet.

## Affected Features

None at this time.

---

## Resolution

Dependency inventory corrected; conclusion unchanged (D-0012)
*This document follows the https://specscore.md/decision-specification*
