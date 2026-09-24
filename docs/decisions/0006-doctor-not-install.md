# 0006 The CLI checks; installation is delegated to mise with consent

Status: accepted · Date: 2026-09-24

## Context
A CLI that installs tools breaks across package managers and drifts with versions. Each tool has its own maintained installer.

## Decision
`ilmarinen doctor` checks presence and versions against `tools.toml` and MCP liveness in both hosts. If anything is missing it asks whether to run `mise install` for mise-manageable tools; if declined, or for tools mise cannot manage, it prints the install command and exits non-zero. `setup` runs `doctor` first and refuses to proceed on failure. Versions are pinned in a committed `mise.toml`.

## Consequences
- Nothing is ever installed without an explicit yes.
- Whether mise's `ubi` backend can fetch `bd`, `specscore` and `codegrapher` from GitHub releases is verified in Phase 0; if not, those fall back to printed install hints.
- Serena and Basic Memory run via `uvx` and are MCP entries, not tools to install.
