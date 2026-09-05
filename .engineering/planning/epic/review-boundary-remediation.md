---
format: aep.planning-md/1
id: epic:review-boundary-remediation
kind: epic
status: active
title: Preserve ESS guarantees across boundaries
relations:
- serves: vision:O2
- derived_from: specification:architecture-review-baseline
revision: 3
---
## Problem

The review at `docs/reviews/2026-09-05-architecture-review.md:27` finds incomplete propagation of validation, semantics and coverage across boundaries despite a green offline gate.

## Outcome

Address all 17 findings through reproducible fixes to demonstrated defects and explicit, reviewable contracts for architectural tradeoffs, while preserving the existing separation of semantic and infrastructure IRs.

## Acceptance

The F01–F17 traceability table in `docs/plan/2026-09-05-review-remediation.md` has a completed evidence-backed disposition for every finding, with no demonstrated defect closed solely by renaming documentation.

## Constraints

Preserve valid canonical bytes unless a separately designed and coordinated migration changes the contract; keep ESS independent of AEP. No generic facet registry, arbitrary property bag, universal interpreter, continuous deployment controller, new language target or wholesale format/crate rename is required. New persisted shapes require a binding design and compatibility fixtures before implementation; a cross-repository contract change also requires the Atlas ADR and relying-party order in `AGENTS.md`.

## Scope

All remediation implementation lives in existing ESS packages and their tests, with explanatory designs and support documentation. Exact typed scopes and conflicts are attached to stories. This epic is not a concurrent work order.

## Traceability

The complete finding ownership, sequencing and existing-backlog disposition are maintained in `docs/plan/2026-09-05-review-remediation.md` and the child stories. Tradeoffs F11, F12, F13 and F17 are not silently promoted into universal features; their present guarantees must be corrected and future semantics decided before a larger implementation.

