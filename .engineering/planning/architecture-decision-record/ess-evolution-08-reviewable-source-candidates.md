---
format: aep.planning-md/1
id: architecture-decision-record:ess-evolution-08-reviewable-source-candidates
kind: architecture-decision-record
status: proposed
title: 08 — Source imports produce reviewable candidates
relations:
- decides: initiative:ess-evolution
revision: 1
---
## Context
Prose and brownfield sources leave semantic questions unanswered.

## Decision
Preserve source identities, locations, accounting and unresolved questions; deterministic compiler is independent of model provider.

## Rejected alternatives
Guessed semantics or model-provider-specific compiler behavior.

## Compatibility and migration
Candidates remain reviewable proposals; compare protobuf descriptors and independent consumers before ownership migration.

## Acceptance evidence
Operator decision: supplied implementation plan, 2026-09-10. See docs/design/ess-evolution/feature-preservation.md and acceptance.md. Implementation and runtime acceptance are not yet established.
