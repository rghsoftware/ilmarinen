---
format: https://specscore.md/decision-specification
status: Approved
---
# Decision: Beads is the task graph; SpecScore Plans/Tasks are not used

**Status:** Approved
**Date:** 2026-09-24
**Owner:** rghsoftware
**Tags:** tasks, beads, specscore
**Source Idea:** —
**Supersedes:** —
**Superseded By:** —

## Context

Both Beads and SpecScore can hold tasks. Keeping both requires a sync, which is a reliability problem, not a feature.

## Decision

Beads (`bd`) is the plan of record. A plan is a Beads issue graph with dependencies. Each issue description carries the `specscore:` reference(s) it serves. SpecScore's `spec/plans/` is never created. Volatile project state uses `bd remember`, never tracked Markdown.

## Rationale

One store for tasks removes the sync. Beads has the more stable API and larger ecosystem, so it is the one kept.

## Declined Alternatives

### Keep both Beads and SpecScore Plans/Tasks

Requires a sync between two task stores, which is a reliability problem, not a feature.

## Consequences at Decision Time

- Traceability chain: Requirement/AC → Beads issue → code and tests (via Source References). SpecScore owns the two ends; Beads owns the middle.
- Beads has a more stable API and larger ecosystem than SpecScore, so the task graph survives a move away from SpecScore.
- `specscore spec lint` must pass on a tree with no `spec/plans/`; verified in Phase 0 of the build.

## Observed Consequences

- 2026-09-24: Beads runs in **default mode, not stealth**. `--stealth` sets `no-git-ops`, which disables the hooks that keep `.beads/issues.jsonl` current and the `refs/dolt/data` sync between machines, the features Beads was chosen for; it also writes global gitignore/gitattributes. All five Beads git hooks are kept: an audit of v1.3.0 (`cmd/bd/hooks.go`) found none that commits, pushes or rewrites history. `bd init` makes one commit of its own (`bd init: initialize beads issue tracking`) that includes anything already staged, so `ilmarinen init` refuses to run it unless the git index is clean and then lets that commit stand. Before `bd init`, `ilmarinen init` checks `bd metrics` reports OFF and runs `bd metrics off` if not, without relying on `setup`. The `refs/dolt/data` probe on `origin` is kept because it is how a second machine bootstraps the task graph; with the network unreachable `bd init` still succeeds with a warning and a working fresh database. Rule adopted: init must succeed without network; network is used only for opt-in sync features and never for telemetry.
- 2026-09-24: `bd init` also stages and commits `AGENTS.md`, `CLAUDE.md`, `.claude/settings.json`, `.gitignore`, `.agents`, `.codex` and `.cursor` by name (`cmd/bd/init.go`), so a clean index was not enough: the first full-loop run committed Ilmarinen's stamped files inside Beads' init commit. `ilmarinen init` now runs `bd init` before stamping any template and refuses when any of those paths has uncommitted changes; the fixture tests assert that Beads' commit holds only `.beads/` and `.gitignore`.

## Affected Features

None at this time.

---
*This document follows the https://specscore.md/decision-specification*
