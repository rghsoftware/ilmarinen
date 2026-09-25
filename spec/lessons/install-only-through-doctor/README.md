---
format: https://specscore.md/lesson-specification
status: Stated
---

# Lesson: Install tools only through doctor's consented mise path, never with mise x

**Status:** Stated
**Date:** 2026-09-25
**Owner:** rghsoftware
**Classifications:** tooling
**Legacy Provenance:** —
**Duplicate Of:** —
**Supersedes:** —
**Superseded By:** —

## Lesson

Every tool the build or the workflow needs is declared in `tools.toml` and pinned in `mise.toml`; it is installed only after `ilmarinen doctor` offers `mise install` and the developer says yes. `mise x`, `mise use` and ad-hoc installers are never used, even for a one-off check.

## Process Gap

During Phase 4 the agent ran `mise x github:rhysd/actionlint@1.7.12` to lint the workflows, which installed actionlint without consent. The rule existed only as prose in the build instructions; the tool was absent from `tools.toml`, so `doctor` had nothing to offer. Fixed by adding actionlint to `tools.toml` (dev scope) and `mise.toml`.

## Tracking

- **Occurrence store:** `occurrences/`
- **Recurrence metadata:** derived from child JSON; never hand-maintained here.
- **Occurrence schema:** `https://specscore.md/new/lesson-occurrence.schema.json`

## Enforcement

**Control:** none-yet: agent discipline; actionlint is now in tools.toml and mise.toml so doctor offers it
**Verification:** —
**Evidence:** —

## Open Questions

None at this time.

---
*This document follows the https://specscore.md/lesson-specification*
