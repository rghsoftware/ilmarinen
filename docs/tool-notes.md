# Tool notes

Filled during the build. Every claim below was read from current upstream docs
or source on 2026-09-24, not from memory. Sections: per-tool facts, then the
contradictions with `docs/` and the smallest change proposed for each.

Machine seen at Phase 0: Ubuntu 24.04 (WSL2), x86_64. Present: mise 2026.9.10,
just 1.58.0, uv 0.12.15, node 26, pnpm 12, cargo 1.98, Claude Code 2.1.281,
OpenCode 1.18.30, docker 29. Missing: `bd`, `specscore`, `codegrapher`,
`ast-grep`, `semgrep`, `cargo-nextest`. `/usr/bin/sg` is shadow-utils, so
ast-grep is always invoked as `ast-grep`.

## Claude Code (2.1.281)

Docs: https://code.claude.com/docs/en/plugins-reference ·
/plugin-marketplaces · /skills · /hooks · /settings · /permissions · /mcp ·
/memory · /costs · /cli-reference · /github-actions (all under
https://code.claude.com/docs/en/).

- **Plugin manifest**: `plugin/.claude-plugin/plugin.json`, only `name`
  required. Auto-discovered: `skills/<name>/SKILL.md`, `commands/*.md`,
  `agents/*.md`, `hooks/hooks.json`, `.mcp.json`. Paths must start `./` and may
  not escape the plugin root, so hook scripts live inside `plugin/`.
- **Variables**: `${CLAUDE_PLUGIN_ROOT}`, `${CLAUDE_PLUGIN_DATA}`,
  `${CLAUDE_PROJECT_DIR}`; exported to hooks and MCP processes.
- **Marketplace**: `.claude-plugin/marketplace.json` at the repo root,
  `{"name","owner":{"name"},"plugins":[{"name":"ilmarinen","source":"./plugin"}]}`.
  Install: `claude plugin marketplace add <path|owner/repo>`, then
  `claude plugin install ilmarinen@<marketplace> [-s user|project|local]`.
  Validate: `claude plugin validate .`. Local-directory marketplaces load
  relative-path plugins in place (no copy).
- **Skills**: all frontmatter optional; `name`, `description` used.
  `description` + `when_to_use` capped at 1,536 chars; body < 500 lines; only
  descriptions load at startup. Plugin skills invoke as `/ilmarinen:<skill>`.
- **Hooks**: events include SessionStart, UserPromptSubmit, PreToolUse,
  PermissionRequest, PostToolUse, PostToolUseFailure, Notification,
  SubagentStart, SubagentStop, Stop, PreCompact, PostCompact, WorktreeCreate,
  WorktreeRemove, SessionEnd (full list in /hooks). **No git pre-commit event**;
  nearest is PreToolUse with `"if": "Bash(git commit *)"`.
  `hooks/hooks.json`: `{"hooks":{"PreToolUse":[{"matcher":"Bash|Read","hooks":[{"type":"command","command":"${CLAUDE_PLUGIN_ROOT}/hooks/x.sh","timeout":30}]}]}}`.
  stdin JSON has `tool_name`, `tool_input` (`Bash: {command,...}`,
  `Read: {file_path,...}`). Exit 2 blocks and feeds stderr to Claude; exit 0
  stderr is never shown. JSON deny: `hookSpecificOutput.permissionDecision: "deny"`.
- **Settings**: `~/.claude/settings.json` < `.claude/settings.json` <
  `.claude/settings.local.json`. Rules `Bash(just *)`, `Read(./.env*)`.
  Deny beats ask beats allow. Path rules only apply to Read/Edit.
  `extraKnownMarketplaces` + `enabledPlugins` enable a plugin from project
  settings (after folder trust).
- **MCP**: `.mcp.json` `{"mcpServers":{"x":{"type":"stdio","command","args","env"}}}`
  or `{"type":"http","url","headers"}`; `${VAR:-default}` expanded. Plugin
  `.mcp.json` at plugin root; tools named `mcp__plugin_<plugin>_<server>__<tool>`.
  User scope: `claude mcp add -s user [-e K=v] name -- cmd args`.
  Liveness: `claude mcp list` prints `✔ Connected` / `✘ Failed to connect`.
- **Instructions**: user file `~/.claude/CLAUDE.md`; `@path` imports work
  (so `CLAUDE.md` = `@AGENTS.md` is valid). Since 2.1.277 Claude Code also
  reads `AGENTS.md` natively when no project `CLAUDE.md` exists.
- **Context**: `/context` measures. MCP tool schemas are deferred by default
  (tool search); disabled when `ANTHROPIC_BASE_URL` is non-first-party.
- **CI**: `anthropics/claude-code-action@v1`; model and turns go in
  `claude_args: "--model <id> --max-turns N"`; `prompt` input.

## OpenCode (1.18.30)

Docs: https://opencode.ai/docs/config · /skills · /agents · /commands ·
/mcp-servers · /plugins · /rules · /permissions · /providers.
Context7 id `/anomalyco/opencode`.

- **Config**: global `~/.config/opencode/opencode.json(c)`, project
  `opencode.json`, `.opencode/` dirs; merged, later wins.
  `$schema: https://opencode.ai/config.json`.
- **Skills**: native `SKILL.md` loading from `.opencode/skills/`,
  `.claude/skills/`, `.agents/skills/` (walks up to the worktree root) and
  `~/.config/opencode/skills/`, `~/.claude/skills/`, `~/.agents/skills/`.
  `name` must match `^[a-z0-9]+(-[a-z0-9]+)*$`, `description` required.
- **Agents / commands**: `.opencode/agents/*.md`, `.opencode/commands/*.md`
  (global under `~/.config/opencode/`).
- **MCP**: `"mcp": {"x": {"type":"local","command":[...],"environment":{},"enabled":true}}`
  or `{"type":"remote","url","headers"}` with `{env:VAR}` substitution.
  Liveness: `opencode mcp list`.
- **Plugins**: `.opencode/plugins/*.ts|js` or `~/.config/opencode/plugins/`.
  Hooks: `tool.execute.before/after`, `chat.message`, `permission.ask`,
  `shell.env`, `experimental.session.compacting`, plus bus events through
  `event` (`session.created`, `session.idle`, `session.deleted`, `file.edited`…).
  Block by throwing in `tool.execute.before`. `$` (Bun shell) can run bash.
- **What OpenCode's plugin API cannot express vs Claude Code hooks**:
  exit-code-2 / JSON decision protocol (plugin must throw itself); `prompt` and
  `agent` hook types; blocking `Stop`; `SubagentStop` (no hook);
  `SessionStart` context injection (`session.created` has no stdout channel);
  `SessionEnd` is approximated by `session.deleted`; `if:` permission-rule
  filters on hooks (must be re-implemented in JS).
- **Rules**: `AGENTS.md` project and `~/.config/opencode/AGENTS.md` global;
  falls back to `CLAUDE.md` / `~/.claude/CLAUDE.md` when no AGENTS.md exists.
- **Permissions**: `"permission": {"bash": {"*":"ask","git *":"allow"}, "read": {"*.env":"deny"}}`;
  last match wins. `.env` read deny is the documented default.
- **Provider stub**: `"provider": {"local": {"npm":"@ai-sdk/openai-compatible","options":{"baseURL":"…"},"models":{…}}}`.
- **Verify loading**: `opencode debug config`, `opencode debug skill`,
  `opencode agent list`, `opencode mcp list`.
- **Converter pattern** (EveryInc/compound-engineering-plugin,
  `src/converters/claude-to-opencode.ts`, `src/targets/opencode.ts`): skills
  copied verbatim; MCP `command`→`local`, `url`→`remote`; hooks folded into one
  generated plugin file running each hook command via `$`; unmappable events
  written as a header comment. **Not copied**: its try/catch around
  PreToolUse (converted hooks can never block) and its use of bus event names
  as top-level hook keys.

## Beads (`bd`, v1.3.0)

Docs: https://github.com/gastownhall/beads (README) ·
/docs/getting-started/installation.md · /docs/cli-reference/init.md ·
/docs/reference/git-integration.md · /docs/integrations/mcp-server.md ·
/docs/integrations/claude-code.md · https://beads.gascity.com/

- Repo moved: `steveyegge/beads` → **`gastownhall/beads`** (MIT).
- Storage is **Dolt only**; `.beads/embeddeddolt/`. **The Dolt store is not
  committed**: commit `.beads/config.yaml`, `metadata.json`, `.gitignore`
  (generated, excludes `dolt/`, `embeddeddolt/`, locks, `backup/`). Sync is
  `bd dolt push/pull` to `refs/dolt/data` on the git remote.
- `bd init`: `--non-interactive` (auto when no TTY / `CI=true`), `-p` prefix,
  `--skip-agents` (otherwise writes `AGENTS.md`), `--skip-hooks` (otherwise
  installs pre-commit/post-merge/pre-push/post-checkout/prepare-commit-msg
  shims), `--init-if-missing` (idempotent).
- **MCP exists**: PyPI `beads-mcp`. Upstream says its schemas cost 10–50k
  tokens and recommends CLI + `bd prime` for Claude Code.
- **`bd init` commits on its own** (`cmd/bd/init.go:2149-2198`, v1.3.0): it
  runs `git add .beads/ .gitignore` (plus agent files) and then
  `git -c core.hooksPath= commit --no-verify -m "bd init: initialize beads issue tracking"`.
  Anything already staged goes into that commit. Only `--stealth` skips it, and
  stealth writes global gitignore/gitattributes, so it is not used.
  **`ilmarinen init` lets that commit stand** (decision 0003, Observed
  Consequences, 2026-09-24) and instead refuses to run `bd init` unless the git
  index is clean, so the commit holds only Beads' own files. It skips `bd init`
  when `.beads/` already exists, and runs `bd metrics off` first if `bd metrics`
  does not report OFF.
- `bd init` sets `core.hooksPath` to the **absolute** path `<repo>/.beads/hooks`
  (local git config) and installs all five hooks there; `.beads/hooks/` is
  tracked. Existing `.git/hooks/*` are copied in first. Each shim is a marked
  section (`# --- BEGIN/END BEADS INTEGRATION v1.3.0 ---`); non-bd content in
  the file is kept. A fresh clone needs `bd hooks install` to re-point
  `core.hooksPath`. Subsets are impossible: `bd hooks install` always installs
  all five (`cmd/bd/hooks.go:24,694`).
- **Hook audit** (source `cmd/bd/hooks.go`, v1.3.0; kept = does not commit,
  push or rewrite history on its own):

  | Hook | What it does | Kept |
  |---|---|---|
  | pre-commit | runs `<hook>.old`; if `export.auto` **and** `export.git-add` (both default false) runs `bd export` and `git add .beads/issues.jsonl` | yes (defaults left as Beads ships them) |
  | prepare-commit-msg | appends an `Executed-By: $BD_ACTOR` trailer to the message being written, only when `BD_ACTOR` is set | yes (edits the in-progress message; no commit, no rewrite) |
  | post-merge | `bd import` into the local Dolt DB (`import.auto` default true) | yes |
  | post-checkout | same import, on branch switches only | yes |
  | pre-push | only runs `<hook>.old` | yes |

  Dolt auto-push (`refs/dolt/data`) is opt-in (`dolt.auto-push` default false)
  and no hook runs it. Our leak guard goes in its own marked section **above**
  Beads' section in `.beads/hooks/pre-commit`, so it runs first and a failure
  exits before Beads' section (Phase 2).
- `bd init` probes `origin` for `refs/dolt/data` (how a second machine
  bootstraps the task graph). **Offline test** (2026-09-24, `unshare -rn`, repo
  with an unreachable `origin`): `bd init` exits 0 with the warning
  "could not verify refs/dolt/data … treating remote as having Dolt history",
  makes its single commit, and the fresh database works offline (`bd create`,
  `bd list`); `origin` stays configured as the Dolt remote for a later
  `bd dolt pull`. No upstream issue or stopgap needed. It leaves `.beads.gate.lock` (a 0600 runtime mutex)
  at the root, gitignored by its own `*.gate.lock*` rule.
- **bd shares usage metrics by default** ("bd metrics off" to opt out); see
  Telemetry.
- Commands: `bd create "T" -t task -p 1 -d "..." --deps blocks:X --json`,
  `bd dep add <blocked> <blocker>`, `bd ready --json`,
  `bd update <id> --claim`, `bd remember "..."`, `bd version`.

## Basic Memory (v0.23.2, AGPL-3.0)

Docs: https://github.com/basicmachines-co/basic-memory (README) ·
https://docs.basicmemory.com/reference/cli-reference ·
https://docs.basicmemory.com/concepts/knowledge-format

- MCP, **pinned exactly** (no `--prerelease=allow`):
  `uvx --from basic-memory==0.23.2 --with fastmcp==4.0.0b1 --with fastmcp-slim==4.0.0b1 basic-memory mcp`.
  Why: 0.23.2 requires `fastmcp==4.0.0b1`, which requires
  `fastmcp-slim==4.0.0b1` (https://pypi.org/pypi/basic-memory/json,
  https://pypi.org/pypi/fastmcp/4.0.0b1/json). `--prerelease=allow` would let
  any dependency float to any pre-release, so each machine could resolve a
  different set. Naming both pre-releases explicitly resolves on uv 0.12.18
  **and** 0.9.12 (`uv pip compile -p 3.13`); without the two `--with` pins,
  uv 0.9.12 refuses. `ilmarinen upgrade` bumps these pins deliberately.
- Projects: `basic-memory project add <name> <path>`,
  `basic-memory project list --json`. Config `~/.basic-memory/config.json`.
- Note format: YAML frontmatter; observations `- [category] fact`; relations
  `- rel [[Target]]`. `BASIC_MEMORY_NO_PROMOS=1` quiets promos.

## Serena (v1.7.0)

Docs: https://github.com/oraios/serena · docs/02-usage/030_clients.md ·
050_configuration.md · 040_workflow.md · docs/01-about/020_programming-languages.md

- Launch, **pinned**: `uvx -p 3.13 --from serena-agent==1.7.0 serena start-mcp-server --context claude-code --project-from-cwd`
  (PyPI `serena-agent` 1.7.0 pins every dependency exactly; resolves with
  `uv pip compile -p 3.13`; the wheel provides `serena`, `start-mcp-server`,
  `--project-from-cwd` and the `claude-code` context). Replaces `git+https`.
  Contexts: `claude-code`, `ide`, `agent`, `codex`, … (no `ide-assistant`).
- Language servers: Rust needs rustup (rust-analyzer); Python uses Pyright via
  uv; TS/Vue/Svelte need Node 18+ and npm.
- `.serena/project.yml` is meant to be committed; `.serena/cache/` and
  `project.local.yml` are ignored by Serena's own `.serena/.gitignore`.
- **License is GPL-3.0-or-later** (SolidLSP part MIT).

## Context7 MCP (v4.1.1)

Docs: https://github.com/upstash/context7/blob/main/packages/mcp/README.md

- Remote http `https://mcp.context7.com/mcp` or `npx -y @upstash/context7-mcp`.
- API key env var: `CONTEXT7_API_KEY` (header `CONTEXT7_API_KEY` /
  `Authorization: Bearer`).

## Playwright MCP (v0.0.82)

Docs: https://github.com/microsoft/playwright-mcp (README)

- `npx @playwright/mcp@latest`; headed by default, so pass `--headless`;
  `--isolated` keeps the profile in memory.

## SpecScore (spec + `specscore` CLI v0.53.1)

Docs: https://specscore.md/ (specscore.org 301s here) ·
https://github.com/specscore/specscore (spec, CC-BY-4.0) ·
https://github.com/specscore/specscore-cli (Apache-2.0) ·
https://specscore.md/source-references-specification ·
https://specscore.md/decision-specification · spec repo `new/*.md` templates.
Context7 has no SpecScore entry.

- **Config**: `specscore.yaml` at repo root, first line exactly
  `# SpecScore Repo Config Schema: https://specscore.md/repo-config`. Keys used
  by Ilmarinen: `project.title/host/org/repo`. **Never set `plans_repo` to the
  repo itself**: lint then walks the missing `spec/plans/` and fails.
- **Layout**: every directory under `spec/` needs a `README.md` (rule
  `readme-exists`). Features: `spec/features/<slug>/README.md`. Decisions:
  `spec/decisions/NNNN-slug.md`, superseded ones in `archived/`. Rules: rows in
  `spec/rules/README.md` or `spec/rules/<slug>/README.md`. Lessons:
  `spec/lessons/<slug>/README.md` + `occurrences/`.
- **Formats**: YAML frontmatter `format:` + `status:`, bold header lines,
  fixed sections, footer `*This document follows the https://specscore.md/<kind>-specification*`.
  Requirements are `#### REQ: <slug>` under `## Behavior` in RFC 2119 wording;
  ACs `### AC: <slug> (verifies REQ:<slug>)` with Given/When/Then.
  Decision statuses: Draft, In Review, Approved, Rejected, Superseded, Deprecated.
  Lesson statuses: Recorded, Stated, Enforced, Withdrawn, Superseded (the
  ladder: Stated = in guidance; Enforced = deterministic control with
  `**Control:**`/`**Verification:**`/`**Evidence:**`). Promotion:
  `specscore rule promote --from-lesson <lesson> <rule> [--scope ...] [--enforcement Stated]`
  writes `**Promotes To:** rule:<rule>` into the Lesson and `lesson:<lesson>`
  into the Rule's Sources; lint R-008 checks both halves; the Lesson's Status is
  unchanged. Verified on a scratch copy (0 violations). The Lesson feature
  prose (`spec/features/lesson/README.md`) does not list `Promotes To`; the CLI
  and lint are the authority.
  `specscore.yaml` needs `lessons.classifications` once any Lesson exists.
- **Source References** (status: Amending). Published regex:
  `^\s*(//|#|--|[/*]|%|;)\s*(specscore:(implements|verifies|references)\s+|specscore:|https://specscore\.org/)`.
  Short target `feature/<slug>#req:<slug>` / `#ac:<slug>`; cross-repo
  `specscore://github.com/org/repo/feature/<slug>`; committed expanded form
  `https://specscore.org/github.com/org/repo/spec/features/<slug>#req:<slug>`.
  CLI check: `specscore code deps --check` (`spec lint` does not scan code).
- **CLI**: `specscore spec lint [--format text|json] [--severity ...]`, no path
  arg (run from repo root). Exit 0 ok, 1 violations, 3 no `specscore.yaml`.
- **`spec lint` with no `spec/plans/`: passes** when `plans_repo` is unset.
  Every plan checker returns early when the plans dir does not exist
  (`pkg/lint/plan_index.go`: `if info, err := os.Stat(plansDir); err != nil || !info.IsDir() { return nil, nil }`).
  **Confirmed by running** `specscore spec lint` 0.53.1 on this repo (no
  `spec/plans/`): `0 violations found`, exit 0.
- `--fix` leaves `.specscore-lifecycle.lock` at the repo root (gitignored).
  Index rows must mirror each Decision; empty `Affected` renders as `—`.
- **Telemetry is on by default** (PostHog usage stats, Sentry crashes).
  Opt out per run with `SPECSCORE_TELEMETRY=0` or `DO_NOT_TRACK=1`
  (`internal/telemetry/optout.go`), or persistently with
  `specscore telemetry disable all`.

## Codegrapher (v0.13.2, Apache-2.0)

Docs: https://github.com/code-grapher/codegrapher

- **Not in the SpecScore organisation**: `code-grapher/codegrapher`, same
  author, cross-linked.
- Commands: `init`, `index`, `sync`, `callers`, `callees`, `impact`,
  `affected`, `serve --mcp` (not used), `version`. Store `.codegraph/codegraph.db`.
- **Languages: Go and TypeScript/JavaScript only.** It cannot index Rust or
  Python. The trial (decision 0010) is **scoped to TypeScript repos** (the Vue
  and Svelte ones); if it never earns its place there it will not elsewhere.

## mise (2026.9.10)

Docs: https://mise.jdx.dev/dev-tools/backends/github.html ·
https://mise.jdx.dev/dev-tools/backends/ubi.html ·
https://mise.jdx.dev/dev-tools/backends/pypi.html ·
https://mise.jdx.dev/cli/ls.html · https://mise.jdx.dev/cli/trust.html

- **`ubi` backend is deprecated**; use `github:owner/repo`.
- `mise ls-remote` resolved (no install): `github:gastownhall/beads` 1.3.0,
  `github:specscore/specscore-cli` 0.53.1, `github:code-grapher/codegrapher`
  0.13.2, `ast-grep` 0.45.3, `just` 1.58.0, `pypi:semgrep` 1.177.0,
  `aqua:nextest-rs/nextest/cargo-nextest` 0.9.146. All three GitHub-release
  tools ship GoReleaser archives `<name>_<ver>_<os>_<arch>.tar.gz` with one
  binary at the root (`bd`, `specscore`, `codegrapher`), so the github backend
  auto-matches. **Confirmed**: `mise install` (with consent, Phase 0)
  installed all of `mise.toml`; `bd version` 1.3.0, `specscore version`
  0.53.1, `codegrapher version` 0.13.2 all run.
- `doctor`'s consented install writes the pinned `mise.toml` to
  `~/.config/mise/conf.d/ilmarinen.toml` (a global drop-in; the user's own
  `config.toml` still overrides it) and runs `mise install <tool>@<ver>...`.
  Verified with `MISE_CONFIG_DIR=<scratch>`: `mise config ls` lists the drop-in
  and `mise which bd` resolves. **Consented path run end to end** (2026-09-24)
  in a sandbox `HOME` with a throwaway `github:sharkdp/hyperfine@1.20.0`
  manifest (`ILMARINEN_TOOLS_TOML` / `ILMARINEN_MISE_TOML` test hooks):
  `doctor --yes` wrote the drop-in, installed the tool, and re-detected it
  through the sandbox shims; the real `~/.config/mise` was untouched.
- Detect without installing: `mise ls --missing --json`. Plain version strings
  in `mise.toml` need no `mise trust`; tool options do.

## ast-grep (0.45.3, MIT)

Docs: https://ast-grep.github.io/guide/project/project-config.html ·
https://ast-grep.github.io/reference/cli/scan.html ·
https://ast-grep.github.io/guide/quick-start.html

- `sgconfig.yml`: `ruleDirs: [rules]`. `ast-grep scan` exits 1 on an error
  rule match, **3 when no `sgconfig.yml` exists**. Call `ast-grep`, never `sg`.

## Semgrep (1.177.0, LGPL-2.1)

Docs: https://docs.semgrep.dev/running-rules · mise `pypi:semgrep`.
`semgrep scan --config <dir>`.

## just (1.58.0, CC0)

Docs: https://just.systems/man/en/ · https://github.com/casey/just/blob/master/CHANGELOG.md

- `[group('x')]`, `set shell := ["bash", "-euo", "pipefail", "-c"]`.
  Run-if-present idiom: `@if [ -f f ]; then cmd; else echo "skip: f"; fi`.

## Phase 2 verification (2026-09-24)

- `claude plugin validate .` and `claude plugin validate plugin` pass (hook
  commands quote `"${CLAUDE_PLUGIN_ROOT}/…"`, as the validator asks).
- **OpenCode loads the converted layout.** Scratch project with
  `.opencode/{skills,plugins,ilmarinen}` linked to `hosts/opencode/`:
  `opencode debug skill` lists all eight skills (no name collisions with the
  47 other skills on this machine); `opencode debug config` shows the five MCP
  servers and `plugins/ilmarinen.ts`. Same result globally after sandbox
  `setup` links them into `~/.config/opencode/`.
- **The generated plugin runs**: `bun test tests/opencode-plugin.test.ts`
  calls its `tool.execute.before/after` exactly as OpenCode does (signatures
  from `@opencode-ai/plugin` 1.18.23 `dist/index.d.ts`): force push and
  `.env` reads throw; a failing `just check rust` is appended to the tool output.
  `Bun.spawn({timeout})` enforces hook timeouts (checked on bun 1.4.2).
- **MCP liveness**, both hosts (`claude --plugin-dir plugin mcp list`,
  `opencode mcp list`): serena, context7, playwright connected. basic-memory
  fails until `setup` creates the `agent-memory` project (connected in the
  sandbox home where it exists). sampo fails until its binary exists (Phase 3).
- Claude Code lists plugin servers as `plugin:ilmarinen:<server>`; doctor
  matches that prefix because another installed plugin (`context7`) declares a
  server with the same bare name.
- This machine already has Context7 twice outside Ilmarinen (the claude.ai
  connector and the `context7` plugin). Relevant to the Phase 4 context budget.
- Serena context: `claude-code` for Claude Code; the converter rewrites it to
  `ide` for OpenCode (Serena's context for hosts with their own file tools).

- `doctor` (unless `--skip-mcp`) reads `claude plugin list --json`: fails if
  `ilmarinen@ilmarinen` is missing, disabled, or its version differs from
  `ilmarinen --version` / `ilmarinen-sampo --version`; warns when two enabled
  plugins declare the same MCP server name, printing both plugin ids.
- The package's binaries are tools like any other (`tools.toml`). Until the
  `v0.1.0` release exists their hint is `cargo install --locked --path cli`;
  the `github:rghsoftware/ilmarinen` mise pin was added once `v0.1.0` shipped
  (release assets `ilmarinen_0.1.0_{linux_amd64,darwin_arm64}.tar.gz`, both
  binaries at the archive root). `mise ls-remote` hides it for 24 h because
  of mise's default `minimum_release_age`, which filters only fuzzy version
  requests (https://mise.jdx.dev/configuration/settings.html); the exact pin
  `0.1.0` is unaffected. The `v0.1.0` tag predates its own pin (the pin can
  only follow the release). `ilmarinen-convert`
  is dev-only (`--features dev`), not in `tools.toml`, not installed.
- Hooks inside a linked worktree (tests/hooks.sh): the worktree top level is
  the `rm -rf` boundary (the main checkout counts as outside); the leak guard's
  optional extras file comes from `$ILMARINEN_CONFIG` / `$HOME`, never the
  checkout.

### Converter gaps (OpenCode cannot express)

Each gap has a type. `ACCEPTED` in `cli/src/convert.rs` holds the four below;
any other type (a new unsupported event, matcher, handler or `if`, or an
OpenCode update that drops `tool.execute.before/after` from the installed
`@opencode-ai/plugin` typings) makes `ilmarinen-convert` and `just check` fail.

Printed by `ilmarinen-convert` and written to `hosts/opencode/CONVERSION.md`:

- PostToolUse cannot block or re-prompt: `tool.execute.after` runs after the
  tool; on exit 2 the plugin appends the hook's stderr to the tool output.
- OpenCode's `patch` tool has no single file path, so Edit|Write hooks do not
  run for it.
- `if` conditions become regexes over the Bash command; only `Bash(<prefix> *)`
  and `Bash(<exact>)` convert.
- Hook JSON output (`permissionDecision`, `additionalContext`) is ignored; only
  exit 2 blocks.
- No equivalent (unused by this plugin): SessionStart context injection,
  Stop/SubagentStop blocking, UserPromptSubmit blocking, Pre/PostCompact,
  Notification, SessionEnd, PermissionRequest; `prompt`/`agent`/`http`/`mcp_tool`
  handlers.
- Not copied from EveryInc: its try/catch around PreToolUse (converted hooks
  could never block) and bus event names used as top-level hook keys.

## Hard-rule corrections (2026-09-24)

Applied after the Phase 3 report; README principle 6, decision 0013.

- **Removed**: `prod-hosts.txt`; seeding leak patterns from hostname, username
  and `$HOME`; every `setup` prompt (keyword, pinned facts, extra patterns,
  production hosts, the OpenCode `AGENTS.md` question) and the `--answers` file.
- **`setup` takes no input**: writes `opt_in_keyword = "full-depth"`, creates
  `repos.toml`, an empty `leak-patterns.txt` and an empty `pinned.md` in the
  memory project at `~/.config/ilmarinen/memory/` (moved from `~/agent-memory`
  so the package touches nothing under `$HOME` outside its own config and cache
  directories), creates `~/.config/opencode/AGENTS.md` without asking (OpenCode
  then no longer falls back to `~/.claude/CLAUDE.md`), and never prompts for
  `mise`: a missing tool fails with its install hint.
- **Deny hook** is generic: force-push, `rm -rf` outside the worktree, `.env*`,
  non-loopback database URLs and `-h`/`--host`/`--endpoint`/`-S` targets,
  `kubectl`/`helm` contexts that are not a known local cluster name
  (`kind-*`, `k3d-*`, `minikube`, `docker-desktop`, `rancher-desktop`,
  `orbstack`, `colima*`; the current context is read with `kubectl config
  current-context` and not stored), cloud CLIs, and `terraform|tofu|pulumi`
  `apply|destroy|import|up`. A test asserts the hook contains no hostnames.
- **Leak guard** is generic: absolute paths under `/home/` or `/Users/`,
  private addresses (RFC 1918 IPv4; IPv6 unique-local `fc00::/7` and
  link-local `fe80::/10`, matched only as addresses with at least two colons
  and a trailing hex digit, so range notation in prose passes), and any line
  added to a `.env*` file except
  `*.example`; optional extras in `~/.config/ilmarinen/leak-patterns.txt`
  (starts empty; never written by the package).
- **SpecScore's local event ledger** (`.specscore/events.jsonl` and
  `.specscore/event-outbox/`) is written by default and its records embed the
  absolute repo path. `events: {subscribers: []}` in `specscore.yaml` (this
  repo and the template) stops it; `.specscore/` (now only locks) is
  gitignored. Source: specscore-cli `pkg/event/config.go` (`LoadSubscribers`).
  The ledger written during this build was deleted before any commit.
- **`tools.toml` `scope`** (`dev | ci | all`): `claude`, `opencode`,
  `codegrapher`, `semgrep`, `cargo-nextest`, `docker` are `dev`.
  `doctor --skip-mcp` is CI mode: `ci`/`all` tools only, and no installed
  plugin, MCP or per-machine-state checks. CI never installs the hosts.

## Session-start context (Claude Code 2.1.282, 2026-09-25)

Measured with `claude -p "/context"` in the stamped mixed repo, plugin
installed at user scope. Headless mode reports before every MCP server has
connected (only Serena had), so the MCP line is a lower bound.

| Ilmarinen's share | Tokens | Loaded |
|---|---|---|
| 8 skill descriptions | ~490 | eagerly |
| repo `AGENTS.md` + `CLAUDE.md` | 734 | eagerly |
| user instruction block (`~/.claude/CLAUDE.md`) | ~250 | eagerly |
| `pinned.md` (Basic Memory frontmatter only) | ~25 | eagerly |
| SessionStart output (handoff note + `bd ready`) | ≤ ~150 | eagerly |
| **Eager total** | **~1.65K** | under the ~3K target |
| MCP tool schemas (Serena alone: 22 tools, 6.2K) | 8.9K+ | deferred (tool search) |

- In Claude Code the MCP schemas are deferred, so the budget holds. In a host
  that loads schemas eagerly (OpenCode) Serena alone is 6.2K, over budget by
  itself; `excluded_tools` in `.serena/project.yml` (or a leaner Serena
  context) is the lever. The Sampo server's `tools/list` is 981 bytes (~250 tokens).
- Everything else in the session (other plugins' skills and agents, the
  developer's own `CLAUDE.md`) is not Ilmarinen's and is not counted.

## 0.1.1 follow-ups (2026-09-25)

- Lessons need a Control starting with a SpecScore mechanism token
  (`product-test`, `cicd-workflow`, `spec-lint`, `claude-md-rule`, …) or
  `none-yet: <why>` (rule L-010), and an Enforced Lesson's Evidence must be a
  plain existing path, URL to a commit/blob, or `sha256:` (rule L-007,
  `pkg/lint/lesson_rules.go` `stableLessonEvidenceAtProject`); backticks fail.
- Regression tests per Phase 4 defect: Beads sweep (`tests/fixtures.sh`: Beads'
  commit content, and init refusing dirty bd-committed paths); plan/verify
  deadlock (`tests/skills.sh`, after the skill texts were aligned);
  upgrade conflicts (Seed strategy unit tests).
- `actionlint` is in `tools.toml` (dev scope) and `mise.toml`, so `doctor`
  offers it (Lesson `install-only-through-doctor`).

## Phase 4 step 5b: trial instrumentation (2026-09-25)

- `scripts/scorecard.sh` (stamped) appends v1 JSON lines to
  `artifacts/scorecard.jsonl`: `gate` (intent|decision|plan|checks|review|merge,
  fired or skipped, caught, one-word reason required for a skip, optional
  host-reported tokens), `done` (feature, depth), `hook` (disabled|bypassed).
  A skip without a one-word reason is refused.
- `just retro` (`scripts/retro.sh`) prints the scorecard and, after
  `features` finished features, KEEP or REVISIT naming each failing
  criterion. Criteria live in the stamped, committed `scorecard.toml`.
  A gate the depth requires that has no line at all counts as an unreasoned
  skip, so a silent skip is measured, not just refused.
- `hook-audit.sh` (PreToolUse Bash, never blocks) logs `git commit
  --no-verify`/`-n`, `git push --no-verify`, `git -c core.hooksPath=…`,
  `git config core.hooksPath`, `bd hooks uninstall`. `session-start.sh` logs
  `leak-guard disabled not-wired` when the repo's pre-commit lacks the guard.
  Not detectable from a hook: `disableAllHooks` in Claude Code settings
  (no hook runs at all).
- **SessionEnd hooks get 1.5 s, and plugin hook timeouts do not raise it**
  (https://code.claude.com/docs/en/hooks, SessionEnd). Since 0.1.1 there are no
  background writes: `session-end.sh` makes one `bd list --status
  open,in_progress --json` call (~0.8 s) and writes the three-line note
  synchronously to `~/.cache/ilmarinen/handoff/<repo>.md` (~0.78 s measured;
  `tests/hooks.sh` fails above 1.5 s). `session-start.sh` flushes the file into
  Beads (`bd remember --key ilmarinen-handoff`), deletes it, then prints the
  note and `bd ready`. Approximation: "waiting on dependencies" means an open
  issue with any dependency, closed or not; `bd ready` printed at start is exact.
  (0.1.0 detached four `bd` calls instead.)
- OpenCode: SessionEnd runs on `session.idle` (accepted gap
  `session-end-as-idle`); SessionStart context injection is impossible
  (accepted gap `session-start-context`), mitigated by one line in OpenCode's
  user instructions to run `bd recall ilmarinen-handoff` and `bd ready`.
- **Seed strategy**: `spec/**`, `specscore.yaml` and `scorecard.toml` are
  created once and then owned by the repo; `upgrade` no longer reports them as
  conflicts after SpecScore rewrites its indexes (found when the first real
  upgrade stalled on two index files).

## Phase 4 findings (2026-09-24)

- **`bd init` commits more than `.beads/`**: it `git add`s `AGENTS.md`,
  `CLAUDE.md`, `.claude/settings.json`, `.gitignore`, `.agents`, `.codex`,
  `.cursor` by name before its commit (`cmd/bd/init.go:2149-2198`). Found when
  the first loop run put Ilmarinen's stamped files into Beads' commit. Fixed by
  ordering (`bd init` before stamping) plus a precondition on those paths;
  asserted by `tests/fixtures.sh`. Decision 0003, Observed Consequences.
- **Basic Memory writes frontmatter into an empty `pinned.md`** when the
  project is registered (`title`, `type: note`, `permalink:
  agent-memory/pinned`). Nothing identifying; left as is.
- **SurrealDB 3.3**: readiness is `docker exec <c> /surreal is-ready` (prints
  `OK`); `SURREAL_ONLINE_VERSION_CHECK=false` disables its start-up web version
  check. `pg_isready` for Postgres. `just db-up` waits on both.
- **Duplicate MCP servers on this machine**: `context7@claude-plugins-official`
  and `playwright@claude-plugins-official` also declare `context7` and
  `playwright`; `doctor` warns. Left for the developer to decide.
- Real-home `setup` ran with no prompts; `doctor` passed with all five servers
  connected in both hosts.

## Phase 3 findings (2026-09-24)

- **TypeScript 7 breaks `vue-tsc` 3.3.11** (`ERR_PACKAGE_PATH_NOT_EXPORTED`
  for `typescript/lib/tsc`). The TS fixtures pin `typescript@^6`;
  `svelte-check` 4.7 is fine with 6. Revisit when vue-tsc supports TS 7.
- **Biome does not see Vue/Svelte template usage**: `noUnusedImports` /
  `noUnusedVariables` fire on `<script setup>` symbols used only in the
  template. The fixtures' `biome.json` turns those two rules off for
  `*.vue` / `*.svelte` only.
- **`specscore feature new` inserts a SpecScore Studio link bar** that embeds
  host/org/repo. The `specscore.yaml` template sets `studio: null` (lint
  accepts it) so generated Features carry no external links.
- **`bd init` copies an existing `.git/hooks/pre-commit` into
  `.beads/hooks/`** ("Preserving existing pre-commit hook") and appends its
  block after it. `ilmarinen init` therefore wires the leak guard into the
  hooks path *before* `bd init`, so Beads' single init commit already carries
  `.beads/hooks/pre-commit` with our section first; `init` re-checks the order
  afterwards (and on every re-run) and moves ours back to the top if needed.
  A leaking `git commit` was blocked end to end in a sandbox repo.
- **gawk aborts** when one variable is used as a scalar in one branch and an
  array in another; `trace.sh` hid that behind `|| true`. Fixed, and the
  `|| true` removed so an awk failure can no longer pass silently.
- Template delimiters are `<% %>`, because justfiles and GitHub Actions use
  `{{ }}`.
- `specscore spec lint` passes on a stamped tree with empty indexes, and on
  every fixture; `ast-grep scan` with an empty `rules/` exits 0.
- **Sampo** (`ilmarinen-sampo`, 384 lines: `cli/src/sampo.rs` +
  `cli/src/bin/ilmarinen-sampo.rs`): bundled SQLite has FTS5; MCP tool names
  cannot contain dots, so `sampo.search` is the `search` tool of the `sampo`
  server. Connected in both hosts after `cargo install --locked --path cli`.
- Versions pinned in templates (checked 2026-09-24): `actions/checkout@v7`,
  `jdx/mise-action@v4`, `anthropics/claude-code-action@v1`, devcontainer
  features `rust:1`, `node:2`, `python:1`, `postgres:18`,
  `surrealdb/surrealdb:v3.3.0`.

## Telemetry

Researched 2026-09-24 against installed versions where noted. Encoded in
`tools.toml` as `[tool.<name>.telemetry]`; `setup` applies persistent opt-outs,
`doctor` reports any it cannot confirm, and the CLI, `just`, hooks and CI set
the per-run env vars.

| Tool | Default | Per-run env | Persistent opt-out | Confirm |
|---|---|---|---|---|
| bd 1.3.0 | on: command names, version, OS, hashed machine id → gastownhall-eventsapi.com | `BD_DISABLE_METRICS=1` (DO_NOT_TRACK honoured) | `bd metrics off` (`~/.config/bd/config.yaml`) | `bd metrics` → `Anonymous usage metrics: OFF` |
| specscore 0.53.1 | on: PostHog EU usage, Sentry EU crashes; auto-off in CI | `SPECSCORE_TELEMETRY=0` (DO_NOT_TRACK=1 honoured) | `specscore telemetry disable all` (**done on this machine**) | `specscore telemetry status` → `disabled` per channel |
| mise 2026.9.10 | on after installs: tool+version, OS/arch, hashed IP → mise-versions.jdx.dev | `MISE_USE_VERSIONS_HOST_TRACK=0` | `mise settings set use_versions_host_track false` | `mise settings get use_versions_host_track` → `false` |
| Claude Code 2.1.281 | on: metrics, error reports (Pro/Max), surveys | `DISABLE_TELEMETRY=1`, `DISABLE_ERROR_REPORTING=1`, `CLAUDE_CODE_DISABLE_FEEDBACK_SURVEY=1` | same vars in `~/.claude/settings.json` `env` (`json_env`) | read back from that file (no status command) |
| semgrep 1.177.0 | only with registry rules or when logged in → metrics.semgrep.dev | `SEMGREP_SEND_METRICS=off` | none (no config key) | none: doctor reports it as unconfirmable |
| Serena 1.7.0 | one ping per start → oraios-software.de; skipped in CI | `SERENA_USAGE_REPORTING=false` | set in the MCP server `env` in `plugin/.mcp.json` (Phase 2) | read `.mcp.json` |

- `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1` is **not** used: it also
  disables feature flags, which breaks Remote Control. The three narrower vars
  cover telemetry, error reports and surveys.
- No telemetry: codegrapher 0.13.2, ast-grep, just, uv (sends "linehaul"
  metadata in its User-Agent to package indexes; no opt-out), OpenCode
  (OTel opt-in only), Basic Memory (Logfire off by default), Context7 MCP
  (request headers are the service itself), Playwright MCP.
- Sources: beads `internal/metrics/metrics.go:17-21`, `cmd/bd/metrics.go:139-190`;
  specscore-cli `internal/telemetry/optout.go`, `docs/telemetry.md`;
  mise `settings.toml:3967-3983`; semgrep `cli/src/semgrep/metrics.py:84-124`;
  https://code.claude.com/docs/en/data-usage, /env-vars; serena v1.7.0
  `src/serena/agent.py:722-735`.

## Contradictions with `docs/` and proposed smallest changes

| # | Doc says | Upstream says | Proposed change |
|---|---|---|---|
| 1 | TOOLING: Serena is MIT | GPL-3.0-or-later | **Resolved**: TOOLING.md fixed; decision 0012 supersedes 0009 with the corrected inventory |
| 2 | 0006: `ubi` backend | `ubi` deprecated | **Resolved**: `github:` in `mise.toml`; dated note in 0006's Observed Consequences |
| 3 | TOOLING/0008: Beads MCP "if it exposes MCP" | it does; 10–50k tokens of schema, upstream recommends CLI | **Resolved**: dropped by decision 0011 (five servers) |
| 4 | Beads "SQLite/JSONL"-era assumptions; `bd init` is just init | Dolt only; `bd init` writes `AGENTS.md` and git hooks by default | **Resolved**: default mode (not stealth); clean-index precondition; Beads' init commit stands; all five hooks kept; leak guard runs first (decision 0003 note) |
| 5 | TOOLING: Codegrapher same org as SpecScore; 0010 trial on a brownfield repo | different org; **Go and TS/JS only** | **Resolved**: repo confirmed; trial scoped to TS repos |
| 6 | ILMARINEN: Requirements in EARS | SpecScore uses RFC 2119 under `#### REQ:` | No conflict: EARS phrasing with uppercase `SHALL` is valid RFC 2119. Skills write EARS inside REQ blocks |
| 7 | ILMARINEN: Lessons "marked promoted", "marked stale" | Lesson statuses have no promoted/stale | **Resolved**: `Promotes To` written by `specscore rule promote --from-lesson` (verified with lint); status ladder `Stated`/`Enforced`; contradicted = `Withdrawn`; ILMARINEN.md updated |
| 8 | 0002: canonical URLs at `specscore.org` | site moved to `specscore.md`; regex and expanded form still use `specscore.org` | None; keep `specscore.org` as the regex requires |
| 9 | 0009: SpecScore spec text CC-BY-4.0 | repo LICENSE CC-BY-4.0; website footer says MIT | None; repo LICENSE governs. Templates are our own text |
| 10 | SpecScore AC spec vs template | AC spec says no Given/When/Then; template uses it | Lint is the authority. Verify in Phase 3; if lint accepts Given/When/Then, open an upstream issue about the spec/template conflict |
| 11 | 0006: Serena/Basic Memory via `uvx` | Basic Memory needs `uvx --prerelease=allow` | **Resolved**: exact pins instead of the flag (see Basic Memory) |
| 12 | Serena artifacts | `.serena/project.yml` is meant to be committed | Commit `.serena/project.yml`; ignore only `.serena/cache/` (Serena's own `.serena/.gitignore` does this) |
| 13 | not in docs | `specscore` sends telemetry by default | **Resolved**: env in `just`/hooks/CI; `specscore telemetry disable all` run on this machine; generalised as `[tool.<name>.telemetry]` in tools.toml |
