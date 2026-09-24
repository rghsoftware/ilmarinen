# 0005 Plain text in git plus a rebuildable index beats every context store today

Status: accepted · Date: 2026-09-24

## Context
Evaluated OpenViking, OpenDeepWiki, Sibyl, Graphiti, Cognee, Hindsight, Mem0, Letta, claude-mem and others against a baseline of per-repo Markdown plus a cross-repo SQLite FTS5 index over MCP. For 2–4 related repos, none justified its footprint. Published studies (ETH Zurich, arXiv 2602.11988; Khatri 2026) show context files do not reliably improve agent outcomes and raise cost, so auto-loaded context must stay minimal.

## Decision
Authoritative knowledge is committed Markdown (Blueprints, Runes). Derived knowledge is generated into Artifacts and never committed. The Sampo index (`$HOME/.cache/ilmarinen/sampo.db`) is rebuilt from the repos in `repos.toml` and exposed through four read-only MCP tools. Session-start context target: under ~3K tokens.

## Consequences
- Cannot infer cross-repo concept relationships or temporal validity beyond explicit `supersedes`/`affects` and Source References.
- Upgrade triggers: frequent "what was true when" questions → Graphiti fed from the same files; >~2K docs or multimodal corpus → OpenViking. The files never change.
