---
name: survey
description: Map a brownfield repo and draft as-is Decisions into the Artifacts cache ($HOME/.cache/ilmarinen/<repo>/). Writes nothing in the repo and never commits.
---

# survey

Output is an Artifact: rebuildable, never authoritative, never committed.
Write only under `$HOME/.cache/ilmarinen/<repo>/` (`<repo>` is the repository
name). Do not create or modify any file inside the repo.

1. Identify: `git remote get-url origin`, languages from marker files
   (`Cargo.toml`, `package.json` + `tsconfig.json`, `pyproject.toml`), build
   and test commands (`justfile`, `package.json` scripts, `Makefile`, CI files).
2. Map with Serena (symbols overview per top-level module) and `rg`; for
   TypeScript repos also try `codegrapher` (trial: `codegrapher index`, then
   `callers`/`impact`). Note entry points, module boundaries, data stores,
   external services, and existing docs or ADRs.
3. Write `survey.md` (≤ 150 lines): stack, layout map, entry points, how to
   build/test, data model locations, hotspots, and open questions.
4. Draft as-is Decisions for choices the code already embodies (framework,
   storage, boundaries), one file each under `decisions/`, in SpecScore
   Decision shape with `**Status:** Draft` and evidence paths in Context.
5. Report the paths. The human adopts a draft through the `rune` skill.
