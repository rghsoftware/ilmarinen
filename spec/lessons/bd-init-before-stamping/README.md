---
format: https://specscore.md/lesson-specification
status: Enforced
---

# Lesson: Run bd init before stamping any template, on a clean index

**Status:** Enforced
**Date:** 2026-09-25
**Owner:** rghsoftware
**Classifications:** tooling
**Legacy Provenance:** —
**Duplicate Of:** —
**Supersedes:** —
**Superseded By:** —

## Lesson

`ilmarinen init` runs `bd init` before it stamps any template, only when the git index is clean and none of the paths Beads commits by name (`AGENTS.md`, `CLAUDE.md`, `.claude/settings.json`, `.gitignore`, `.agents`, `.codex`, `.cursor`) has uncommitted changes; Beads' single init commit then holds only its own files.

## Process Gap

The first full-loop run found Ilmarinen's stamped `AGENTS.md`, `CLAUDE.md`, `.claude/settings.json` and `.gitignore` inside Beads' init commit: `bd init` stages those paths by name (`cmd/bd/init.go`, v1.3.0) and a clean-index check does not cover unstaged files. The fixture tests only checked that init succeeded, not what Beads committed.

## Tracking

- **Occurrence store:** `occurrences/`
- **Recurrence metadata:** derived from child JSON; never hand-maintained here.
- **Occurrence schema:** `https://specscore.md/new/lesson-occurrence.schema.json`

## Enforcement

**Control:** product-test: tests/fixtures.sh checks that Beads commits only .beads/ and .gitignore, and that init refuses dirty bd-committed paths
**Verification:** `tests/fixtures.sh`
**Evidence:** tests/fixtures.sh

## Open Questions

None at this time.

---
*This document follows the https://specscore.md/lesson-specification*
