# Vocabulary

Use these names everywhere: code, docs, skill text, commit messages.

| Term | Meaning | Concrete things |
|---|---|---|
| **Ilmarinen** | The package, the CLI, and the methodology | this repo; `ilmarinen` binary; `docs/ILMARINEN.md` |
| **Blueprints** | What to build | SpecScore Features, Requirements, Acceptance Criteria; the Beads issue graph derived from them |
| **Runes** | Durable engineering knowledge | SpecScore Rules and Decisions; promoted Lessons; ast-grep/Semgrep rules derived from them |
| **Forge** | The implementation loop | the `forge` skill; git worktrees; hooks; `just check` / `just test` |
| **Sampo** | The compounding engine | the `sampo` skill; the cross-repo index and its MCP server (`sampo.*`); `just stale`; the Lesson → Rune promotion path |
| **Artifacts** | Generated, rebuildable, never authoritative | traceability reports; surveys; the Sampo index; converted host layouts. Always under `artifacts/` or `$HOME/.cache/ilmarinen/`; always gitignored or `linguist-generated` |

## Collisions to keep straight

- SpecScore calls its spec files "artifacts". In Ilmarinen those are **Blueprints**
  or **Runes**. An **Artifact** is only ever generated output.
- Claude has an unrelated "artifacts" feature. Never use the word for it in
  Ilmarinen docs.
- The `spec/` directory keeps SpecScore's mandated layout (`spec/features/`,
  `spec/rules/`, `spec/decisions/`, `spec/lessons/`). Blueprints and Runes are
  layer names, not folder names.

## Skills, by layer

| Skill | Layer | Produces |
|---|---|---|
| `route` | Ilmarinen | a depth classification and the gates that apply |
| `blueprint` | Blueprints | a SpecScore Feature (EARS requirements, Given/When/Then ACs) |
| `rune` | Runes | a SpecScore Decision |
| `plan` | Blueprints | Beads issues with dependencies and `specscore:` references |
| `forge` | Forge | tests and code in a worktree, carrying Source References |
| `verify` | Forge | gate results and a fresh-context review against the ACs |
| `sampo` | Sampo | a Lesson and a promotion proposal |
| `survey` | Artifacts | a repo map and draft as-is Decisions for a brownfield repo |
