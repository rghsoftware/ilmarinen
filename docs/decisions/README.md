# Decisions

Plain-Markdown decision records for the package itself, MADR-style. Immutable
once accepted; superseded, never edited. During Phase 0 of the build these are
migrated into `spec/decisions/` in SpecScore format once that format has been
read from its current docs; this directory is then removed.

| # | Decision | Status |
|---|---|---|
| 0001 | The workflow is ours; tools are adopted as formats and validators only | accepted |
| 0002 | SpecScore is the file format for Blueprints and Runes | accepted |
| 0003 | Beads is the task graph; SpecScore Plans/Tasks are not used | accepted |
| 0004 | Basic Memory is the personal memory store, separate from all repos | accepted |
| 0005 | Plain text in git plus a rebuildable index beats every context store today | accepted |
| 0006 | The CLI checks; installation is delegated to mise with consent | accepted |
| 0007 | Fresh plugin, borrowing only the multi-host converter pattern | accepted |
| 0008 | Always-on MCP servers are a closed list of six | accepted |
| 0009 | License: Apache-2.0 | accepted |
| 0010 | Codegrapher is a CLI-only trial | accepted |
