# 0003 Beads is the task graph; SpecScore Plans/Tasks are not used

Status: accepted · Date: 2026-09-24

## Context
Both Beads and SpecScore can hold tasks. Keeping both requires a sync, which is a reliability problem, not a feature.

## Decision
Beads (`bd`) is the plan of record. A plan is a Beads issue graph with dependencies. Each issue description carries the `specscore:` reference(s) it serves. SpecScore's `spec/plans/` is never created. Volatile project state uses `bd remember`, never tracked Markdown.

## Consequences
- Traceability chain: Requirement/AC → Beads issue → code and tests (via Source References). SpecScore owns the two ends; Beads owns the middle.
- Beads has a more stable API and larger ecosystem than SpecScore, so the task graph survives a move away from SpecScore.
- `specscore spec lint` must pass on a tree with no `spec/plans/`; verified in Phase 0 of the build.
