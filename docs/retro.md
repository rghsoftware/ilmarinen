# Retro: building Ilmarinen 0.1.0

Written at the end of the build (2026-09-25). Details and sources for every
finding are in `docs/tool-notes.md`.

## The dry run

One deliberately tiny `major` feature on the mixed fixture (Rust, Vue,
Python): the `greeting` Feature learned to trim names. It ran the whole loop:
`route` → `blueprint` (gate 1) → `plan` (gate 3) → `forge` in three stacked
worktrees → `verify` (gates plus a fresh-context reviewer) → human review and
merge (gate 4) → `sampo`.

### What each gate caught

| Gate | Result | What it caught |
|---|---|---|
| 1 intent + AC | fired, **caught** | The first Blueprint had no rule for an all-whitespace name. The human sent it back; the Feature gained `REQ: blank-name` and its AC with an explicit decision (`Hello, !`, no error). |
| 2 decision | skipped (`no-choice`) | Nothing hard to reverse; recorded with a reason, not silently. |
| 3 plan | fired | Nothing; the graph (two tests-first issues blocking one implementation) was approved as proposed. |
| checks (`just check`/`test`/`trace`) | fired | Nothing at verify time; red tests during `forge` were test-first, not catches. |
| review (fresh context) | fired | Nothing not met. It noted "no error" is asserted explicitly only in TypeScript; in Rust and Python a panic or exception fails the equality test anyway. |
| 4 merge | fired | Nothing; merged `--no-ff` by the human's instruction. |

The dry run is the first scorecard entry: seven events, one finished feature,
one real catch, verdict pending (1 of 5). It was written with
`scripts/scorecard.sh` into the loop repo's `artifacts/scorecard.jsonl`; that
repo lives in session scratch space, so the lines are reproduced here:

```
{"v":1,"kind":"gate","gate":"intent","feature":"greeting","status":"fired","caught":true,"reason":null,"tokens":null}
{"v":1,"kind":"gate","gate":"decision","feature":"greeting","status":"skipped","caught":false,"reason":"no-choice","tokens":null}
{"v":1,"kind":"gate","gate":"plan","feature":"greeting","status":"fired","caught":false,"reason":null,"tokens":null}
{"v":1,"kind":"gate","gate":"checks","feature":"greeting","status":"fired","caught":false,"reason":null,"tokens":null}
{"v":1,"kind":"gate","gate":"review","feature":"greeting","status":"fired","caught":false,"reason":null,"tokens":null}
{"v":1,"kind":"gate","gate":"merge","feature":"greeting","status":"fired","caught":false,"reason":null,"tokens":null}
{"v":1,"kind":"done","feature":"greeting","depth":"major","tokens":null}
```

The loop itself also found three defects no gate was designed for:

1. **`bd init` commits more than `.beads/`.** It stages `AGENTS.md`,
   `CLAUDE.md`, `.claude/settings.json` and `.gitignore` by name, so Beads'
   own commit swallowed Ilmarinen's stamped files. Fixed by running `bd init`
   before stamping; asserted by the fixture tests (decision 0003).
2. **`plan` and `verify` contradicted each other.** Tests-first issues block
   the implementation, but issues close on merge, so the implementation could
   never become ready. Recorded as Lesson `close-test-issues-when-tests-land`.
3. **`upgrade` treated SpecScore-maintained files as conflicts.** Fixed with
   the Seed strategy: `spec/**`, `specscore.yaml` and `scorecard.toml` are
   created once and then owned by the repo.

## Where the tooling fought back

- **SpecScore**: `decision new --supersedes` writes a one-sided link that
  fails lint, and every status change re-runs lint and rolls back (Lesson
  `supersede-after-approving-successor`). It also wrote a local event ledger
  embedding absolute paths until `events: {subscribers: []}` stopped it, and
  its telemetry was on by default. Its AC spec contradicts its own template
  (filed upstream as https://github.com/specscore/specscore/issues/60).
- **Beads**: the init commit above, default-on metrics, and a SessionEnd
  handoff that takes ~1.9 s against Claude Code's 1.5 s budget (now detached).
- **Claude Code**: SessionEnd's budget cannot be raised by a plugin;
  `disableAllHooks` is invisible to hooks.
- **OpenCode**: no SessionStart context, no blocking PostToolUse, no hook for
  multi-file `patch` edits. Six gap types are now accepted explicitly; any new
  one fails `just check`.
- **Front-end toolchain**: TypeScript 7 breaks `vue-tsc` 3.3 (fixtures pin
  TS 6); Biome cannot see Vue/Svelte template usage.
- **This build's own process**: the first pass collected facts about the
  developer (seeded leak patterns, a production-host list, prompts for pinned
  facts). Reversed by the hard rule (README principle 6, decision 0013).
  Session scratch space was wiped between sessions, taking the first loop
  repo with it; it was rebuilt from the approved text.

## Token cost

Host-reported, from Claude Code's session transcript (Opus, 1M context):

| Scope | Output | Cache read | Cache write | Uncached input |
|---|---|---|---|---|
| Dry-run loop (route → sampo, including the scratch rebuild) | 68K | 55.7M | 1.2M | 0.2K |
| Whole build (phases 0-4) | 1.28M | 419M | 4.0M | 2K |

Cache reads dominate because the build ran in one long session whose context
grew past 500K tokens; the loop re-read that history every turn. A loop run in
a fresh session starts near the measured ~1.65K of Ilmarinen context.

## Three changes to make first

1. **Fix the `plan`/`verify` contradiction in the skill text** (promote the
   Lesson): a tests-first issue closes when its failing tests are committed.
   It deadlocks every `standard` and `major` loop today.
2. **Cut Serena's schema cost.** It is 6.2K of the 8.9K+ deferred MCP tokens,
   harmless in Claude Code but over the 3K budget on its own in any host that
   loads schemas eagerly. Exclude the memory and editing tools the workflow
   does not use (`excluded_tools`), then re-measure.
3. **Make the scorecard survive and record cost without trust.** Give
   `verify`/`sampo` the host-reported token count automatically (the
   transcript's usage fields) instead of an optional flag, and keep the trial
   repos out of scratch space, so five features can actually accumulate
   before the first KEEP/REVISIT verdict.
