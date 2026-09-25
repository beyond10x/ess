---
format: aep.planning-md/2
id: architecture-decision-record:ess-evolution-08-reviewable-source-candidates
kind: architecture-decision-record
status: proposed
title: 08 — Source imports produce reviewable candidates
relations:
- decides: initiative:ess-evolution
revision: 2
---
## Context

Prose and brownfield sources leave semantic questions unanswered. Existing import and coverage behavior must remain supported during the service/runtime migration.

## Decision

Preserve source identities, locations, accounting and unresolved questions. The deterministic compiler remains independent of a model provider. Imported candidates are reviewable proposals; imports never guess.

## Rejected alternatives

Guessed semantics or model-provider-specific compiler behavior.

## Compatibility and migration

Preserve current importer semantics and explicit coverage refusals. New generic protobuf ownership migration and its descriptor/consumer comparison requirements are deferred to task:deferred-protocol-ui-bindings and do not block current ESS evolution.

## Acceptance evidence

Operator-approved revision ess-evolution-20260915/1, recorded by approval-record:ess-evolution-20260915, narrows the original 2026-09-10 completion scope. Existing import behavior remains covered by the full ESS preservation gate; no deferred capability is claimed implemented.
