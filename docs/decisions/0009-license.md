# 0009 License: Apache-2.0

Status: accepted · Date: 2026-09-24

## Context
The package is tooling. Dependencies: Beads (MIT), Serena (MIT), specscore-cli and Codegrapher (Apache-2.0), SpecScore spec text (CC-BY-4.0), Basic Memory (AGPL-3.0, called over MCP only), Compound Engineering (MIT, pattern only).

## Decision
Apache-2.0: explicit patent grant, clear contribution terms, matches the nearest neighbours.

## Consequences
- Never vendor Basic Memory source.
- Any SpecScore template text copied verbatim must carry CC-BY attribution; prefer writing our own templates.
- This is a practical reading, not legal advice.
