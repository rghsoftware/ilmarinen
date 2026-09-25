---
name: blueprint
description: Write the SpecScore Feature for major work, with EARS requirements and Given/When/Then acceptance criteria, lint it, and stop at gate 1 (intent and acceptance criteria). Only in `ilmarinen init` repos.
---

# blueprint

Dormant unless a `.ilmarinen.version` sits at or above the working directory with no `.ilmarinen.off` beside it; outside that, act only when invoked by name.

Produces one SpecScore Feature: `spec/features/<slug>/README.md`.

1. Check for an existing Feature first: `specscore feature list`,
   `sampo.search("<terms>", kind="feature")`. Amend it instead of duplicating.
2. Scaffold: `specscore feature new --title "<Title>" --slug <slug>`
   (add `--parent <id>` for a sub-feature). Never hand-create the file.
3. Fill `## Summary` and `## Problem` in a few sentences each.
4. Under `## Behavior`, one `#### REQ: <req-slug>` per requirement, phrased in
   EARS with an uppercase `SHALL` (valid RFC 2119):
   - Ubiquitous: `The <system> SHALL <response>.`
   - Event: `When <trigger>, the <system> SHALL <response>.`
   - State: `While <state>, the <system> SHALL <response>.`
   - Unwanted: `If <condition>, then the <system> SHALL <response>.`
   - Optional: `Where <feature is present>, the <system> SHALL <response>.`
   One behaviour per REQ; no "and"-chained requirements.
5. Under `## Acceptance Criteria`, one `### AC: <ac-slug> (verifies REQ:<req-slug>)`
   per observable outcome, each with `**Given**`, `**When**`, `**Then**`
   lines. Every REQ has at least one AC; every AC is testable by a command or
   a test, not by judgement.
6. Unknowns go in `## Open Questions`; do not guess.
7. `specscore spec lint` until it passes.
8. **Gate 1.** Show the REQ and AC list and stop. Wait for explicit approval.
   On approval: `specscore feature change-status <id> --to "In Review"`, then
   `--to Approved`. Next: `rune` if a hard-to-reverse choice exists, else `plan`.

References to this Feature from code and issues use
`specscore:<verb> feature/<slug>#req:<req-slug>` or `#ac:<ac-slug>`.
