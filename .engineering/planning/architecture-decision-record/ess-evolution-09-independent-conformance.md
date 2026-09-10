---
format: aep.planning-md/1
id: architecture-decision-record:ess-evolution-09-independent-conformance
kind: architecture-decision-record
status: proposed
title: 09 — Conformance observes implementations independently
relations:
- decides: initiative:ess-evolution
revision: 1
---
## Context
Tests derived from implementation lowering can reproduce its defects.

## Decision
Extend existing conformance runners, Go/fault injection and versioned formats; expected behavior comes from upstream scenarios and targets return observations.

## Rejected alternatives
Generating expectations from implementation code or target plans; silent missing capabilities.

## Compatibility and migration
Retain all versions/defaults. Add opt-in UI scenarios and emitters; prove wrong-field, dropped-patch, transition, revocation and UI-binding mutations fail.

## Acceptance evidence
Operator decision: supplied implementation plan, 2026-09-10. See docs/design/ess-evolution/feature-preservation.md and acceptance.md. Implementation and runtime acceptance are not yet established.
