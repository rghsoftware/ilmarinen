---
name: plan
description: Decompose approved work into a Beads issue graph with dependencies, each issue carrying the specscore reference it serves; stop at gate 3 and print what is ready.
---

# plan

The plan of record is a Beads graph. Never write tasks or TODO lists in markdown.

1. Inputs: the approved Feature (`major`) or the request (`standard`), plus any
   Decisions. Read REQs and ACs with `specscore feature info <id>`.
2. One issue per independently testable slice (usually one REQ or AC):
   ```
   bd create "<imperative title>" -t task -p <0-4> --json \
     -d "<what and done-when>

   specscore:implements feature/<slug>#req:<req-slug>"
   ```
   Use `specscore:verifies ...#ac:<ac-slug>` for test-only issues. For
   `standard` work with no Feature, reference the existing Requirement it
   changes; if none exists, write `specscore: none (standard)` explicitly.
   Use `--parent <epic-id>` to group under an epic.
3. Order: `bd dep add <blocked> <blocker>` (the second blocks the first). Tests
   that pin an AC come before the implementation they verify. A tests-first
   issue is done, and closed, when its failing tests are committed; only the
   implementing issue waits for merge.
4. Check the graph: `bd graph` or `bd dep tree <id>`; every issue has a
   reference, no cycles, nothing orphaned.
5. **Gate 3.** Show the graph and stop for approval.
6. On approval, print `bd ready`. Next: `forge`.

Volatile notes about the plan go in Beads (`bd update <id> --append-notes`,
`bd remember "<insight>"`), never in tracked markdown.
