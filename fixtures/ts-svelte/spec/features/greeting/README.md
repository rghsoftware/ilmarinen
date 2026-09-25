---
format: https://specscore.md/feature-specification
status: Approved
---

# Feature: Greeting

**Status:** Approved
**Source Ideas:** —

## Summary

Greet a user by name. The smallest Blueprint that exercises every Ilmarinen gate.

## Problem

The fixture needs one Requirement, one implementing symbol and one verifying test.

## Behavior

### Greeting

#### REQ: greet-by-name

When a name is given, the library SHALL return the greeting `Hello, <name>!`.

## Acceptance Criteria

### AC: greets-named-user (verifies REQ:greet-by-name)

**Given** the name `Ada`
**When** the greeting is requested
**Then** the result is `Hello, Ada!`

## Open Questions

None at this time.

---
*This document follows the https://specscore.md/feature-specification*
