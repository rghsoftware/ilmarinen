---
format: https://specscore.md/decision-specification
status: Approved
---

# Decision: Ilmarinen is dormant unless the repo is initialized

**Status:** Approved
**Date:** 2026-09-25
**Owner:** rghsoftware
**Tags:** plugin, hooks, activation
**Source Idea:** —
**Supersedes:** —
**Superseded By:** —

## Context

A Claude Code plugin's hooks and skill descriptions load in every session once the plugin is enabled, whatever the repo. Up to 0.1.1 the hooks acted everywhere: the deny hook blocked commands, SessionStart printed Beads state, SessionEnd wrote handoff notes, and the post-edit check ran `just check` after every edit, in repos that had never opted into the workflow. That collides with other workflows on the same machine. The post-edit check also ran once per edit, which is slow and noisy.

## Decision

Ilmarinen acts only in an initialized repo. A repo is active when `.ilmarinen.version` exists in the working directory or its nearest ancestor (so linked worktrees under the repo count) and no `.ilmarinen.off` sits beside it. Every hook checks this first and exits 0 with no output otherwise; SessionStart and SessionEnd do nothing outside an active repo. `ilmarinen off` writes `.ilmarinen.off` (gitignored, so per checkout) and `ilmarinen on` removes it. Every skill description says it applies only in `ilmarinen init` repos, the other skill bodies open with the same rule, and `route` refuses to start the workflow outside an active repo unless invoked by name. The per-edit check moves from PostToolUse to a Stop hook that runs `just check <lang>` once per turn for the languages with uncommitted changes.

## Rationale

Installing the plugin must not change how any other repo behaves. A marker file the CLI already writes is the cheapest, most visible opt-in: no configuration, no list of repos, and it travels with the checkout. A second file for off keeps the switch reversible without touching committed files.

## Declined Alternatives

### A list of enabled repos in the user config

It is a record of the developer's paths, which the hard rule (decision 0013) forbids, and it drifts from the checkouts it names.

### Hide the skills with `disable-model-invocation: true`

It removes the descriptions from context everywhere but also stops Claude invoking the skills in initialized repos, which is the workflow itself.

### Keep the check on PostToolUse

It runs after every edit, including mid-refactor states that are expected to fail, and multiplies the cost of every multi-file change.

## Consequences at Decision Time

- Tests prove that an uninitialized repo and an off repo get no hook output, no blocks and no scorecard lines (`tests/hooks.sh`), and that `off` lets a leaking commit through while `on` blocks it again (`tests/fixtures.sh`).
- The skill descriptions remain in context wherever the plugin is enabled; the host offers no per-repo switch for them. The dormancy clause adds about 8 tokens per skill.
- The Stop check hands failures back once per turn (exit 2) and never re-blocks when `stop_hook_active` is set; on OpenCode it runs on `session.idle` and is advisory.

## Observed Consequences

- 2026-09-25: Measured with `claude -p "/context"` in an uninitialized repo, the plugin adds no messages and no memory files; the eight skill descriptions add ~580 tokens (Skills 5.9K → 6.5K). Accepted by the human; the question is closed. `disable-model-invocation` is not a lever: it also blocks the model-invoked `route` from calling the other skills through the Skill tool (`docs/tool-notes.md`, 0.1.2).

## Affected Features

None at this time.

---
*This document follows the https://specscore.md/decision-specification*
