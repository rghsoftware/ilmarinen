---
format: https://specscore.md/lesson-specification
status: Enforced
---

# Lesson: Close a tests-first issue when its failing tests are committed, not at merge

**Status:** Enforced
**Date:** 2026-09-25
**Owner:** rghsoftware
**Classifications:** process
**Legacy Provenance:** —
**Duplicate Of:** —
**Supersedes:** —
**Superseded By:** —

## Lesson

A Beads issue that only adds tests pinning an AC is done when those tests are committed and fail for the right reason; close it then, with the commit in the reason. Only the implementing issue closes at merge.

## Process Gap

The `plan` skill makes implementation issues depend on tests-first issues, while the `verify` skill says issues close on merge. Read together, the implementation issue can never become ready. Found in the first full loop (greeting trim, 2026-09-25): `forge` had to close `mixed-876` and `mixed-yu5` early to unblock `mixed-4o8`. Neither skill's text covers the case.

## Tracking

- **Occurrence store:** `occurrences/`
- **Recurrence metadata:** derived from child JSON; never hand-maintained here.
- **Occurrence schema:** `https://specscore.md/new/lesson-occurrence.schema.json`

## Enforcement

**Control:** product-test: tests/skills.sh checks that the plan, forge and verify skill texts agree
**Verification:** `dash tests/skills.sh`
**Evidence:** tests/skills.sh

## Open Questions

None at this time.

---
*This document follows the https://specscore.md/lesson-specification*
