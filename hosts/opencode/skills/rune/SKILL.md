---
name: rune
description: Record a hard-to-reverse engineering choice as a SpecScore Decision (context, choice, declined alternatives, consequences), lint it, and stop at gate 2.
---

# rune

Write a Decision only when a choice is expensive to undo (a dependency, a data
format, a boundary, a protocol). Otherwise skip to `plan`.

1. Search first: `sampo.search("<terms>", kind="decision")`. If an existing
   Decision covers it, cite it (`D-NNNN`) instead of writing a new one.
2. Scaffold: `specscore decision new <slug> --title "<Title>" --owner <handle>`.
   Do not pass `--supersedes` (see Superseding).
3. Fill every section: `## Context` (what forced the choice), `## Decision`
   (one or two sentences), `## Rationale`, `## Declined Alternatives` (at
   least one `### <name>` with why it lost), `## Consequences at Decision Time`
   (positive and negative). Leave `## Observed Consequences` as
   `None observed yet.`; list touched Features under `## Affected Features`.
4. `specscore spec lint` until it passes.
5. **Gate 2.** Show the Decision and stop. On approval:
   `specscore decision change-status <NNNN-slug> --to "In Review"`, then
   `--to Approved`.

Superseding (Runes are never edited):
1. Write and approve the successor as above, without `--supersedes`.
2. Then `specscore decision change-status <old> --to=Superseded --successor <new> --note "<why>"`.
   It links both ways and archives the old one. (Lesson
   `supersede-after-approving-successor`.)

After approval only `## Observed Consequences` may change, as dated bullets.
