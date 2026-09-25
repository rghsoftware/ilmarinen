---
format: https://specscore.md/decision-specification
status: Approved
---

# Decision: Personal memory holds how I work, never what I own

**Status:** Approved
**Date:** 2026-09-24
**Owner:** rghsoftware
**Tags:** memory, privacy, basic-memory
**Source Idea:** —
**Supersedes:** 0004-basic-memory-personal
**Superseded By:** —

## Context

Decision 0004 made Basic Memory the personal store for "facts about the developer, their machines and preferences", and the first build seeded it: `setup` asked for pinned facts, derived leak patterns from the hostname, username and `$HOME`, and kept a list of production hosts. That makes the package a place that collects and stores facts about the person using it, which Ilmarinen must never be. The package is glue that installs a workflow with no typed input and no knowledge of who the developer is or what they own.

## Decision

Personal memory (Basic Memory project `agent-memory`, a private git repo with no remote at `~/.config/ilmarinen/memory/`) holds only how the developer works: preferences that are not workflow rules, recurring decisions and their reasons, tooling lessons that are not repo-specific, and standing instructions about the developer as a collaborator. It never holds what the developer owns: identity, usernames, hostnames, hardware, network addresses or ranges, paths, accounts, deployment targets, or anything credential-adjacent. `pinned.md` starts empty and fills only when the developer explicitly asks the agent to remember something. When a session needs such a fact, the agent runs a command in that session and does not persist the result.

## Rationale

A workflow package that knows nothing about its user cannot leak anything about them, needs no questions at install time, and behaves the same on every machine. How someone works is what makes an assistant better across repos; what they own is either discoverable in the session or none of the package's business.

## Declined Alternatives

### Seed personal facts at setup (0004 as first built)

Asks the developer to type facts about themselves and derives more from the machine; every such fact is a liability in a store the package controls.

### Allow-lists or deny-lists of infrastructure

A production-host list is itself a map of the developer's infrastructure. Production safety comes from absence instead: no production credentials, profiles, kubeconfigs or connection strings exist in the agent environment.

## Consequences at Decision Time

- `ilmarinen setup` takes no input and detects nothing about the machine.
- The leak guard blocks generic shapes only (home-directory paths, private addresses, `.env*` contents); its optional extras file starts empty and the package never writes it.
- The memory project moves from `~/agent-memory` to `~/.config/ilmarinen/memory/`, inside the package's own config directory.
- AGPL-3.0 remains fine: Basic Memory is called over MCP, never vendored.
- Personal memory and project knowledge stay different stores with different tool names.

## Observed Consequences

None observed yet.

## Affected Features

None at this time.

---
*This document follows the https://specscore.md/decision-specification*
