# 0002 SpecScore is the file format for Blueprints and Runes

Status: accepted · Date: 2026-09-24

## Context
We need a Markdown schema for Features, Requirements, Acceptance Criteria, Decisions, Rules and Lessons, plus a code↔spec link grammar that works across 2–4 separate repos, without inventing one. Candidates: SpecScore, StrictDoc, Spec Kit templates, Scry markers, a custom frontmatter schema.

## Decision
Adopt SpecScore's format and its lint (`specscore spec lint`) as a format validator. Adopt its Source References grammar (`specscore:implements`, `specscore:verifies`, `specscore:references`; cross-repo `specscore://host/org/repo/...`) as the only traceability notation. Requirements are phrased in EARS.

## Alternatives rejected
- **StrictDoc**: `.sdoc` is not Markdown and its ceremony suits regulated work; use it end to end on a regulated project, never mixed.
- **Scry markers**: a second grammar overlapping Source References; keep only the "rebuildable index, read-only SQL" idea.
- **Custom schema**: reinvents the wheel.

## Consequences
- Small, single-organisation ecosystem; Source References were marked "Amending" at adoption. Exit cost is low because the files are plain Markdown and the detection regex is published.
- The `spec/` layout is SpecScore's; Blueprints/Runes are layer names only.
- We do not adopt SpecScore Plans/Tasks (see 0003) or SpecStudio's workflow (see 0001).
- Canonical expanded references point at `specscore.org` URLs; short form is acceptable if that is unwanted.
