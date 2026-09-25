---
name: verify
description: Run the deterministic gates, then have a fresh-context reviewer with no access to the implementer's reasoning check the change against the acceptance criteria only.
---

# verify

1. Gates, from the worktree root: `just check`, `just test`, and `just e2e`
   when the change touches UI. `just trace` confirms every Source Reference
   resolves. Record pass or fail for each.
2. Fresh-context review. Start a new subagent or session that receives only:
   - the ACs (`specscore feature info <id>` or the AC text), and
   - the diff (`git diff <base>...HEAD`).
   It must not see the plan, the chat or the implementer's notes. Ask it to
   report, for each AC: `met`, `not met` or `untested`, with the test name or
   line that proves it. It reports; it does not fix.
3. Report to the human as one table:

| Item | Result | Evidence |
|---|---|---|
| `just check` | pass/fail | failing step |
| `just test` | pass/fail | failing test |
| AC `<slug>` | met/not met/untested | test or line |

4. Record both gates (`<feature>` is the Feature slug; `trivial` work uses a
   short slug for the change). A gate that finds something records
   `--caught`; a gate not run records `skipped --reason <one-word>`:
   `scripts/scorecard.sh gate checks <feature> fired [--caught]` and
   `scripts/scorecard.sh gate review <feature> fired [--caught]`. Add
   `--tokens N` only when the host reports the cost.
5. Any `fail`, `not met` or `untested` goes back to `forge`. Otherwise the
   change is ready for the human's review (gate 4). Nothing merges without a
   human; never merge, and never enable auto-merge.
6. On merge, the human (or you, when asked) closes the implementing issue
   (tests-first issues were closed by `forge` when their tests landed):
   `bd close <id> --reason "<merged ref>"`. Next: `sampo`.
