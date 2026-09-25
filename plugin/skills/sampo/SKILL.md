---
name: sampo
description: After review, record what would recur as a SpecScore Lesson, withdraw contradicted Lessons, and propose promoting validated Lessons to Rules. Use at the end of every standard or major loop. Only in `ilmarinen init` repos.
---

# sampo

Dormant unless a `.ilmarinen.version` sits at or above the working directory with no `.ilmarinen.off` beside it; outside that, act only when invoked by name.

Knowledge moves one way and gets shorter: Lesson → Rule or Decision →
ast-grep/Semgrep rule → one line in `AGENTS.md`.

1. Ask: what in this loop would recur (a gate that caught something, a tool
   that fought back, a review finding)? If nothing, say so and stop.
2. Search before writing: `sampo.search("<terms>", kind="lesson")`.
   - Existing Lesson: `specscore lesson recur <slug> --note "<what happened>"`.
   - New: `specscore lesson new <slug> --title "<imperative rule>" --owner <handle> --classification <c>`
     (classifications are in `specscore.yaml`), fill `## Lesson` and
     `## Process Gap`, then
     `specscore lesson occurrence add <slug> --capture-context=false --evidence-kind path --evidence-ref <file> --summary "<fact>"`.
3. A Lesson this loop contradicted:
   `specscore lesson change-status <slug> --to=withdrawn --note "<why>"`.
4. Propose promotion (the human decides) for a Lesson that recurred or is
   validated: `specscore rule promote --from-lesson <lesson> <rule> --scope <scope> --enforcement Stated`,
   plus, where it can be enforced, an ast-grep rule in `rules/` run by
   `just check`, and at most one line in `AGENTS.md`.
5. `just stale` lists Lessons still `Recorded` six weeks after their date
   with no `Promotes To`; propose deleting them at review.
6. `specscore spec lint`.
7. Scorecard: record each human gate of this loop with
   `scripts/scorecard.sh gate <intent|decision|plan|merge> <feature> fired [--caught]`
   (`--caught` when the human's review at that gate changed the work), or
   `skipped --reason <one-word>` when the depth or the work did not need it
   (e.g. `decision ... skipped --reason no-choice`). Then
   `scripts/scorecard.sh done <feature> <trivial|standard|major>`. There is no
   silent skip: every gate the depth requires gets a line.

Lessons are about the work, never the person. How the developer works goes to
Basic Memory only when they explicitly ask you to remember it; what they own
(identity, hosts, network, paths, accounts) is never recorded anywhere.
