---
format: https://specscore.md/lesson-specification
status: Enforced
---

# Lesson: Stamp files that SpecScore or the developer maintain once, then never diff them

**Status:** Enforced
**Date:** 2026-09-25
**Owner:** rghsoftware
**Classifications:** tooling
**Legacy Provenance:** —
**Duplicate Of:** —
**Supersedes:** —
**Superseded By:** —

## Lesson

Files that SpecScore rewrites (everything under `spec/`) or that the developer tunes (`specscore.yaml`, `scorecard.toml`) are stamped once when absent and are never rewritten or diffed by `ilmarinen upgrade` (the Seed strategy).

## Process Gap

The first real `ilmarinen upgrade` stalled asking to overwrite `spec/features/README.md` and `spec/rules/README.md`, whose rows SpecScore had maintained since init. The templates treated every stamped file as template-owned; nothing distinguished files the repo owns after the first stamp.

## Tracking

- **Occurrence store:** `occurrences/`
- **Recurrence metadata:** derived from child JSON; never hand-maintained here.
- **Occurrence schema:** `https://specscore.md/new/lesson-occurrence.schema.json`

## Enforcement

**Control:** product-test: unit tests for the Seed strategy (classify_changes, language_paths_and_suffixes)
**Verification:** `cargo test --manifest-path cli/Cargo.toml --features dev classify_changes`
**Evidence:** cli/src/init.rs

## Open Questions

None at this time.

---
*This document follows the https://specscore.md/lesson-specification*
