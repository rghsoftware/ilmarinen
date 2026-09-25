---
format: https://specscore.md/decision-specification
status: Approved
---

# Decision: License: Apache-2.0, with a corrected dependency inventory

**Status:** Approved
**Date:** 2026-09-24
**Owner:** rghsoftware
**Tags:** license
**Source Idea:** —
**Supersedes:** 0009-license
**Superseded By:** —

## Context

Decision 0009 chose Apache-2.0 from a dependency inventory that Phase 0 verification found wrong in two places: Serena is GPL-3.0-or-later (its SolidLSP component is MIT), not MIT (https://github.com/oraios/serena); and Codegrapher is in the `code-grapher` organisation, not SpecScore's (https://github.com/code-grapher/codegrapher). Decisions are not edited, so the inventory is corrected here.

| Dependency | License | How Ilmarinen uses it |
|---|---|---|
| Beads (`bd`) | MIT | CLI |
| Serena | GPL-3.0-or-later (SolidLSP MIT) | MCP server, launched by `uvx`; never vendored |
| Basic Memory | AGPL-3.0 | MCP server, launched by `uvx`; never vendored |
| specscore-cli | Apache-2.0 | CLI |
| SpecScore spec text | CC-BY-4.0 | format; our templates are our own text |
| Codegrapher | Apache-2.0 | CLI trial |
| Context7 MCP | MIT | MCP server via `npx` |
| Playwright MCP | Apache-2.0 | MCP server via `npx` |
| ast-grep / Semgrep OSS | MIT / LGPL-2.1 | CLI |
| just / mise | CC0 / MIT | CLI |
| Compound Engineering | MIT | converter pattern only |

The SpecScore website footer says "MIT license" while the spec repo's LICENSE file says CC-BY-4.0; the repo LICENSE governs, and it does not matter to us because no SpecScore text is copied.

## Decision

Apache-2.0, unchanged: explicit patent grant, clear contribution terms, matches the nearest neighbours. Copyleft dependencies (Serena GPL-3.0, Basic Memory AGPL-3.0) are only called over MCP as separate processes and are never vendored or distributed.

## Rationale

The corrected inventory adds a GPL-3.0 dependency, but the conclusion of 0009 still holds because every copyleft tool runs as a separate process the user installs; Ilmarinen ships none of their code.

## Declined Alternatives

### MIT

Used by several dependencies, but carries no explicit patent grant.

### Edit 0009 in place

Approved Decisions are immutable; supersede, never edit.

## Consequences at Decision Time

- Never vendor Serena or Basic Memory source.
- Any SpecScore template text copied verbatim must carry CC-BY attribution; prefer writing our own templates.
- This is a practical reading, not legal advice.

## Observed Consequences

None observed yet.

## Affected Features

None at this time.

---
*This document follows the https://specscore.md/decision-specification*
