---
format: aep.planning-md/1
id: architecture-decision-record:ess-evolution-02-semantics-before-bindings
kind: architecture-decision-record
status: proposed
title: 02 — Semantic models precede implementation bindings
relations:
- decides: initiative:ess-evolution
revision: 2
---
## Context

Domain, deployment and observed infrastructure already have distinct semantics. Approved ESS evolution revision 1 narrows the current addition to reusable service semantics and ER execution.

## Decision

Keep EssIr, composition, CLI, realization, deployment and infrastructure distinct; add focused ServiceIr lowering and explicit ER definitions/binding plans. Generic UI modeling is deferred to task:deferred-protocol-ui-bindings.

## Rejected alternatives

Universal facet registry, arbitrary metadata bags, forced ess-ir/2 and weakening semantics to fit a target.

## Compatibility and migration

Keep existing versions/defaults and typed references. Add opt-in service-runtime-ir/4 and service-realization-plan/4 with old-reader compatibility tests. Deferred UI formats do not block current completion.

## Acceptance evidence

Operator-approved revision ess-evolution-20260915/1, recorded by approval-record:ess-evolution-20260915, supersedes the broader 2026-09-10 scope. Preserve docs/design/ess-evolution/feature-preservation.md and execute acceptance.md. New runtime acceptance is not yet established.
