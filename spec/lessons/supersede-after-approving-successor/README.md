---
format: https://specscore.md/lesson-specification
status: Recorded
---

# Lesson: Approve the successor Decision before superseding the old one

**Status:** Recorded
**Date:** 2026-09-24
**Owner:** rghsoftware
**Classifications:** tooling
**Legacy Provenance:** —
**Duplicate Of:** —
**Supersedes:** —
**Superseded By:** —

## Lesson

When a new Decision supersedes an existing one, scaffold the successor **without** `--supersedes`, move it to `Approved` with `specscore decision change-status`, and only then run `specscore decision change-status <old> --to=Superseded --successor <new> --note "..."`. That command writes both links (`**Supersedes:**` on the successor, `**Superseded By:**` on the old Decision) and archives the old one in a single lint-checked step.

`specscore decision new <slug> --supersedes <old>` writes a one-sided link. Lint (`D-supersedes-bidirectional`) then fails on the whole tree, and every later `change-status` re-runs lint and rolls back, so neither Decision can move.

## Process Gap

The `--supersedes` flag on `decision new` suggests it is the way to supersede, and its help does not say the old Decision must already be `Superseded` for lint to pass. The loop only shows up when the next `change-status` rolls back with lint errors about the other Decision. Hit on 2026-09-24 while superseding 0008 with 0011 and 0009 with 0012 (specscore 0.53.1).

## Tracking

- **Occurrence store:** `occurrences/`
- **Recurrence metadata:** derived from child JSON; never hand-maintained here.
- **Occurrence schema:** `https://specscore.md/new/lesson-occurrence.schema.json`

## Enforcement

**Control:** none-yet: just recorded, mechanism not chosen yet
**Verification:** —
**Evidence:** —

## Open Questions

None at this time.

---
*This document follows the https://specscore.md/lesson-specification*
