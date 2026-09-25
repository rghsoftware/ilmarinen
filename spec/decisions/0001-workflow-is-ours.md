---
format: https://specscore.md/decision-specification
status: Approved
---
# Decision: The workflow is ours; tools are adopted as formats and validators only

**Status:** Approved
**Date:** 2026-09-24
**Owner:** rghsoftware
**Tags:** workflow, methodology, tooling
**Source Idea:** —
**Supersedes:** —
**Superseded By:** —

## Context

Every methodology reviewed (AWS AI-DLC, Spec-Driven Development via Spec Kit / Kiro / Tessl, Compound Engineering, OpenAI's harness engineering and Symphony, Gas Town, Archon) converges on the same skeleton: capture intent, AI proposes a plan, human approves at a few gates, isolated execution, deterministic verification, knowledge written back. They differ in ceremony and in where the human sits. Adopting any one tool's methodology imports its ceremony and its churn.

## Decision

Ilmarinen defines its own workflow (`docs/ILMARINEN.md`): adaptive depth with explicit opt-in for `major`, four gates for `major` and one for `trivial`, and the loop route → blueprint → rune → plan → forge → verify → review → sampo. Every external tool is adopted only as a file format, a validator, or a transport. No tool's "how to work" skills are installed.

## Rationale

The reviewed methodologies share one skeleton, so owning that skeleton costs little, while adopting any one of them imports its ceremony and its churn.

## Declined Alternatives

### Adopt one existing methodology wholesale

Spec Kit, Kiro, Tessl, AI-DLC, Compound Engineering, Symphony, Gas Town or Archon as-is. Declined because each imports its ceremony and its churn.

## Consequences at Decision Time

- SpecStudio, Spec Kit commands, Compound Engineering's skill set, and AI-DLC's stage library are not installed; ideas are borrowed into our eight skills.
- Replacing any tool must not require changing `docs/ILMARINEN.md`.
- The cost is writing and maintaining our own skills.

## Observed Consequences

None observed yet.

## Affected Features

None at this time.

---
*This document follows the https://specscore.md/decision-specification*
