---
format: https://specscore.md/decision-specification
status: Approved
---
# Decision: The CLI checks; installation is delegated to mise with consent

**Status:** Approved
**Date:** 2026-09-24
**Owner:** rghsoftware
**Tags:** cli, installation, mise
**Source Idea:** —
**Supersedes:** —
**Superseded By:** —

## Context

A CLI that installs tools breaks across package managers and drifts with versions. Each tool has its own maintained installer.

## Decision

`ilmarinen doctor` checks presence and versions against `tools.toml` and MCP liveness in both hosts. If anything is missing it asks whether to run `mise install` for mise-manageable tools; if declined, or for tools mise cannot manage, it prints the install command and exits non-zero. `setup` runs `doctor` first and refuses to proceed on failure. Versions are pinned in a committed `mise.toml`.

## Rationale

Each tool already has a maintained installer; delegating to one version manager avoids breaking across package managers and drifting with versions.

## Declined Alternatives

### A CLI that installs tools itself

Breaks across package managers and drifts with versions.

## Consequences at Decision Time

- Nothing is ever installed without an explicit yes.
- Whether mise's `ubi` backend can fetch `bd`, `specscore` and `codegrapher` from GitHub releases is verified in Phase 0; if not, those fall back to printed install hints.
- Serena and Basic Memory run via `uvx` and are MCP entries, not tools to install.

## Observed Consequences

- 2026-09-24: mise's `ubi` backend is deprecated upstream in favour of `github:` (https://mise.jdx.dev/dev-tools/backends/ubi.html). `mise.toml` uses `github:` for `bd`, `specscore` and `codegrapher`; `mise install` fetched all three from GitHub releases, so no fallback to printed hints was needed.

## Affected Features

None at this time.

---
*This document follows the https://specscore.md/decision-specification*
