# The Ilmarinen workflow

Tool-agnostic. Tools appear only where a step emits or checks a file.

## Routing

Every request is classified before any work starts:

| Depth | When | Gates |
|---|---|---|
| `trivial` | a fix or small change with no interface, data-model, or cross-repo impact | merge only |
| `standard` | a bounded change inside one repo | plan, merge |
| `major` | the opt-in keyword was used, or the change touches an interface contract, a data model, or more than one repo | intent+AC, decision, plan, merge |

The router states its classification and reason in one sentence. The human can
override it. Depth never escalates silently mid-task; if scope grows, the agent
stops and re-routes.

## Gates

A gate is a stop where the human approves before the next step. Gates are cheap
to pass and expensive to skip.

1. **Intent and acceptance criteria** (`major`): the Blueprint's Requirements
   (EARS) and Acceptance Criteria (Given/When/Then) are correct and testable.
2. **Decision** (`major`, when a hard-to-reverse choice exists): a Rune records
   the choice, alternatives, and consequences.
3. **Plan** (`standard`, `major`): the Beads issue graph is complete, ordered,
   and every issue references the Requirement or AC it serves.
4. **Merge** (all): gates green, review done, human merges. Nothing merges
   without a human.

## The loop

```
route → blueprint → rune? → plan → forge → verify → review → sampo
```

- **blueprint**: write the Feature; lint; gate 1.
- **rune**: write a Decision if one is needed; lint; gate 2.
- **plan**: decompose into Beads issues; gate 3; print what is ready.
- **forge**: claim the next ready issue; new worktree; tests first, carrying
  `specscore:verifies`; implement, carrying `specscore:implements`; run the
  gates; never weaken a test to pass it.
- **verify**: run `just check`, `just test`, `just e2e` as relevant; then a
  fresh-context reviewer with no access to the implementer's reasoning reports
  against the ACs only.
- **review**: the human reviews the diff; comments go back to the agent; gate 4.
- **sampo**: record what would recur as a Lesson; mark contradicted Lessons
  stale; propose promotion.

`trivial` work runs `forge → verify → review` only and emits no Blueprint.

## Sensors and guides

The quality ceiling is set by the sensors, not by the model. Every repo has one
`just check` that runs every deterministic check for every detected language,
plus `just trace` for spec-to-code links and `specscore spec lint` for format.
Hooks and CI call these targets; they never duplicate them.

Guides are short. `AGENTS.md` is a map of at most 60 lines. Nothing else
auto-loads. Knowledge is fetched on demand through the Sampo index and the
always-on MCP servers.

## Compounding

Knowledge moves in one direction and gets shorter as it goes:

```
Lesson (spec/lessons/)  →  validated  →  Rune (Rule or Decision)
                                      →  ast-grep / Semgrep rule
                                      →  one line in AGENTS.md
```

Once promoted, the Lesson is marked promoted. Lessons not promoted within six
weeks are flagged by `just stale` and deleted at review. Runes are never edited;
they are superseded.

## Memory boundaries

- If a fact is true for anyone who clones the repo, it belongs in the repo.
- If it mentions the developer, their machines, or their preferences, it belongs
  in personal memory and never in a repo. A pre-commit leak guard enforces this.
- Task state and in-flight notes live in Beads, never in tracked markdown.

## Context budget

At session start: the 60-line map plus a 30-line pinned personal file, and the
tool schemas of the always-on MCP servers. Target under ~3K tokens. Measure it
after any change to the plugin.
