# 0007 Fresh plugin, borrowing only the multi-host converter pattern

Status: accepted · Date: 2026-09-24

## Context
Compound Engineering's plugin proves the packaging pattern (one source tree, a converter emitting layouts for OpenCode, Pi, Codex and others) but ships 36 skills and 51 agents that encode a different workflow.

## Decision
Build the Claude Code plugin from scratch with eight skills. Write our own converter following their pattern, emitting `hosts/opencode/` as a generated, `linguist-generated` directory. Import none of their skills, agents or content.

## Consequences
- Multi-host support is a build task, not a free ride, but it is small.
- Where OpenCode cannot express a Claude Code hook, the converter must report the gap rather than drop it.
