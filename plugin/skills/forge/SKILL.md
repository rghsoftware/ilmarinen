---
name: forge
description: Implement the next ready Beads issue in its own git worktree, tests first, carrying specscore Source References, running the gates, never weakening a test.
---

# forge

1. Claim: `bd ready --json`, pick the first, `bd update <id> --claim`.
   (`trivial` work has no issue: skip claiming.)
2. Worktree: `git worktree add .worktrees/<id> -b <id>` and work only there.
3. Tests first. Write the test that pins the AC or REQ, with a Source
   Reference comment on the test symbol:
   `// specscore:verifies feature/<slug>#ac:<ac-slug>` (use the language's
   comment marker: `//`, `#`, `--`). Run it and see it fail for the right reason.
4. Implement the smallest change that passes. Mark the implementing symbol:
   `// specscore:implements feature/<slug>#req:<req-slug>`.
   Cross-repo targets use `specscore://<host>/<org>/<repo>/feature/<slug>#req:<r>`.
5. Gates: `just check` and `just test` (and `just e2e` for UI changes) until
   green. Hooks run `just check <lang>` after each edit; fix what they report.
6. Never weaken, skip or delete a test to make it pass. If a test is wrong,
   stop and say why; changing it needs the human.
7. Commit in the worktree with a message naming the issue id. Never push with
   `--force`, never touch `.env*`, and run migrations and database commands
   only against the local throwaway databases (`just db-up`). There are no
   production credentials here, by design; never go looking for them.
8. Record progress: `bd update <id> --append-notes "<state>"`. A tests-first
   issue closes now: `bd close <id> --reason "failing tests in <commit>"`, so
   the implementation it blocks becomes ready. Next: `verify` (after the
   implementing issue).

If the work turns out bigger than routed (new interface, data model, or
repo), stop and re-route.
