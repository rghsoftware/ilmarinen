# `ilmarinen` CLI

Install: `mise use -g github:rghsoftware/ilmarinen@0.1.1` (both binaries). Build
from source with `just build`; test with `just test`.
Templates, `tools.toml`, `mise.toml` and `plugin/` are compiled in.

**Network rule:** init must succeed without network; network is used only for
opt-in sync features and never for telemetry.

With the network unreachable, `init` still succeeds: `bd init` only warns that
it could not probe `origin` for `refs/dolt/data`, and `bd dolt pull` bootstraps
the task graph later. Every tool that phones home is opted out per run (env)
and persistently (`setup`); see `tools.toml`.

| Command | Does |
|---|---|
| `ilmarinen doctor [--skip-mcp] [--yes]` | Checks every tool in `tools.toml` (present, `>= min`), telemetry opt-outs, the installed plugin's version and duplicate MCP names, MCP liveness in Claude Code and OpenCode for each server in `plugin/.mcp.json`, and per-machine state (`config.toml`, `repos.toml`, the memory project, the Basic Memory project). Offers `mise install` for missing mise-manageable tools; if declined or unmanageable, prints install commands and exits 1. |
| `ilmarinen setup [--skip-mcp] [--package DIR]` | No input. Runs doctor's tool check without prompting and refuses on failure. Then config only: `config.toml` (`opt_in_keyword = "full-depth"`), `repos.toml`, an empty `leak-patterns.txt`, the empty personal memory project (`~/.config/ilmarinen/memory/`, Basic Memory project `agent-memory`), the managed block in `~/.claude/CLAUDE.md` and `~/.config/opencode/AGENTS.md`, the Claude Code plugin at user scope, the OpenCode layout, telemetry opt-outs. Detects nothing about the machine. Closes with a full doctor run. |
| `ilmarinen init <repo> [--host H --org O --name R] [--no-beads]` | Stamps `templates/` (language parts only when detected), writes `.ilmarinen.version`, runs `bd init` in default mode (refuses unless the git index is clean, because Beads' single init commit includes anything staged; makes sure `bd metrics` is off first), `specscore spec lint`, and adds the repo to `repos.toml`. Idempotent. |
| `ilmarinen upgrade <repo>` | Diffs current templates against the repo using `.ilmarinen.version`; shows every diff; applies only after confirmation, with a second confirmation per locally edited file. |

## `ilmarinen-convert`

`just convert` regenerates `hosts/opencode/` from `plugin/`; `just convert-check`
(part of `just check`) fails when it is stale. Every Claude Code hook feature
OpenCode cannot express is printed and written to `hosts/opencode/CONVERSION.md`.

## Flags for CI

- `--skip-mcp` is CI mode: only `ci`/`all` scope tools from `tools.toml`, and no
  installed-plugin, MCP liveness or per-machine-state checks (CI has no hosts
  and no logins). CI runs `ilmarinen doctor --skip-mcp`.
- `--yes` is explicit consent to doctor's `mise install` offer. Without it,
  a non-interactive stdin always answers no.

## Why setup runs only doctor's tool check first

`setup` is what registers the MCP servers and creates the personal state, so
those checks cannot pass before it runs. It gates on tools, then runs the full
doctor at the end.
