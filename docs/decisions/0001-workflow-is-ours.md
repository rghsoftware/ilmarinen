# 0001 The workflow is ours; tools are adopted as formats and validators only

Status: accepted · Date: 2026-09-24

## Context
Every methodology reviewed (AWS AI-DLC, Spec-Driven Development via Spec Kit / Kiro / Tessl, Compound Engineering, OpenAI's harness engineering and Symphony, Gas Town, Archon) converges on the same skeleton: capture intent, AI proposes a plan, human approves at a few gates, isolated execution, deterministic verification, knowledge written back. They differ in ceremony and in where the human sits. Adopting any one tool's methodology imports its ceremony and its churn.

## Decision
Ilmarinen defines its own workflow (`docs/ILMARINEN.md`): adaptive depth with explicit opt-in for `major`, four gates for `major` and one for `trivial`, and the loop route → blueprint → rune → plan → forge → verify → review → sampo. Every external tool is adopted only as a file format, a validator, or a transport. No tool's "how to work" skills are installed.

## Consequences
- SpecStudio, Spec Kit commands, Compound Engineering's skill set, and AI-DLC's stage library are not installed; ideas are borrowed into our eight skills.
- Replacing any tool must not require changing `docs/ILMARINEN.md`.
- The cost is writing and maintaining our own skills.
