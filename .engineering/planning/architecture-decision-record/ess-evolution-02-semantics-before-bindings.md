---
format: aep.planning-md/1
id: architecture-decision-record:ess-evolution-02-semantics-before-bindings
kind: architecture-decision-record
status: proposed
title: 02 — Semantic models precede implementation bindings
relations:
- decides: initiative:ess-evolution
revision: 1
---
## Context
Domain, deployment and observed infrastructure already have distinct semantics.

## Decision
Keep EssIr, composition, CLI, realization, deployment and infrastructure distinct; add focused ServiceIr and UiIr.

## Rejected alternatives
Universal facet registry, arbitrary metadata bags, forced ess-ir/2.

## Compatibility and migration
Keep existing versions/defaults and typed references. Add opt-in UI/runtime formats and negative old-reader compatibility cases.

## Acceptance evidence
Operator decision: supplied implementation plan, 2026-09-10. See docs/design/ess-evolution/feature-preservation.md and acceptance.md. Implementation and runtime acceptance are not yet established.
