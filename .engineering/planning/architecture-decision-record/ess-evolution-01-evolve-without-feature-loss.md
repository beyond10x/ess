---
format: aep.planning-md/1
id: architecture-decision-record:ess-evolution-01-evolve-without-feature-loss
kind: architecture-decision-record
status: proposed
title: 01 — Evolve ESS without feature loss
relations:
- decides: initiative:ess-evolution
revision: 1
---
## Context
Existing consumers depend on package identities, formats and tested behavior.

## Decision
Preserve ESS, its repository, package identities and all implemented contracts. Migrate incrementally.

## Rejected alternatives
Replacement repository, renaming ESS and dropping features to simplify lowering.

## Compatibility and migration
Retain every public format, fixture, generated snapshot and CLI behavior. The preservation mapping names destinations; full existing gates and independent compatibility prove each move.

## Acceptance evidence
Operator decision: supplied implementation plan, 2026-09-10. See docs/design/ess-evolution/feature-preservation.md and acceptance.md. Implementation and runtime acceptance are not yet established.
