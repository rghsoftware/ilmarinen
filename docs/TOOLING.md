# Tooling

Every tool is adopted as a **format, validator, or transport**. None of them
supply the workflow. Each must be replaceable without touching
`docs/ILMARINEN.md`.

## Adopted

| Tool | Role in Ilmarinen | Adopted as | License | Replace with, if it dies |
|---|---|---|---|---|
| SpecScore (`spec/` format, `specscore` CLI) | Blueprints and Runes on disk; Source References for code↔spec links | file format + lint | spec CC-BY-4.0; CLI Apache-2.0 | keep the Markdown; ~100-line reference checker from the published regex |
| Beads (`bd`) | task graph; plan of record; volatile project memory (`bd remember`) | CLI (+ MCP if offered) | MIT | any issue tracker with dependencies |
| Basic Memory | personal/operator memory across hosts | MCP server + note format | AGPL-3.0 (called, not distributed) | markdown files + FTS index |
| Serena | code intelligence (LSP-backed symbols) | always-on MCP | MIT | LSP directly |
| Context7 | current library docs | always-on MCP | see upstream | web search |
| Playwright MCP | UI verification | always-on MCP | Apache-2.0 | Playwright CLI |
| ast-grep / Semgrep | enforceable Runes | CLI in `just check` | MIT / LGPL-2.1 (Semgrep OSS) | either one alone |
| `just` | single entry point for gates | CLI | CC0 | make |
| `mise` | tool installation and version pinning | CLI, consent-gated | MIT | manual install from `tools.toml` hints |
| Codegrapher | symbol graph, blast radius | **trial**, CLI only, never MCP | Apache-2.0 | Serena |
| EARS | requirement phrasing | convention | n/a | n/a |
| Compound Engineering | multi-host converter **pattern** only | design reference | MIT | own converter |

## Always-on MCP servers (the complete list)

Serena, Context7, Playwright, Basic Memory, Beads (if it exposes MCP), Sampo.
Anything else is CLI. Adding a server requires a Rune and a re-measured context
budget.

## Deliberately excluded (and the condition that would change it)

| Excluded | Why | Revisit when |
|---|---|---|
| SpecScore Plans/Tasks, SpecStudio skills | overlap with Beads; encode a methodology we don't use | never for tasks; templates may be borrowed |
| Spec Kit, Kiro specs, BMAD, Tessl | second spec format or spec-as-source | never; EARS phrasing is borrowed |
| OpenViking, OpenDeepWiki, Sibyl | heavier than 2–4 repos justify; LLM in the write path | corpus grows past ~2K docs or needs multimodal |
| Graphiti/Zep, Mem0, Hindsight, Letta | LLM-extracted memory; opaque stores | frequent "what was true when" questions (Graphiti); want auto-capture into an inbox (Hindsight) |
| StrictDoc | regulated-grade ceremony | a regulated or safety-critical project (use it end to end there) |
| Gas Town, claude-flow/ruflo, Symphony daemon, Stoneforge | multi-agent swarms, unattended runs, no approval gates | never as-is; Symphony's WORKFLOW-file idea is already borrowed |
| GUI ADEs (Orca, T3 Code, OpenChamber, Nimbalyst) | not needed for the baseline | after the workflow is stable for a month; use before forking |
| Local model serving | separate concern | embeddings/rerank/summaries first; never primary implementation on 16 GB |
| claude-mem | transcript sink; license churn; token promotion | never |

## Installation policy

`ilmarinen doctor` reads `tools.toml`, reports present / missing / too old, and
checks every MCP server responds from both hosts. If anything is missing it asks
whether to run `mise install` for the mise-manageable tools. If declined, or for
tools mise cannot manage, it prints the exact install command and exits
non-zero. `setup` refuses to run until `doctor` passes. Nothing is ever installed
without asking.
