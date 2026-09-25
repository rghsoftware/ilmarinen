---
format: https://specscore.md/decision-specification
status: Approved
---
# Decision: Fresh plugin, borrowing only the multi-host converter pattern

**Status:** Approved
**Date:** 2026-09-24
**Owner:** rghsoftware
**Tags:** plugin, converter, opencode
**Source Idea:** —
**Supersedes:** —
**Superseded By:** —

## Context

Compound Engineering's plugin proves the packaging pattern (one source tree, a converter emitting layouts for OpenCode, Pi, Codex and others) but ships 36 skills and 51 agents that encode a different workflow.

## Decision

Build the Claude Code plugin from scratch with eight skills. Write our own converter following their pattern, emitting `hosts/opencode/` as a generated, `linguist-generated` directory. Import none of their skills, agents or content.

## Rationale

The packaging pattern is proven and worth borrowing; the content encodes a different workflow (see 0001).

## Declined Alternatives

### Fork or install Compound Engineering's plugin

Ships 36 skills and 51 agents that encode a different workflow.

## Consequences at Decision Time

- Multi-host support is a build task, not a free ride, but it is small.
- Where OpenCode cannot express a Claude Code hook, the converter must report the gap rather than drop it.

## Observed Consequences

None observed yet.

## Affected Features

None at this time.

---
*This document follows the https://specscore.md/decision-specification*
