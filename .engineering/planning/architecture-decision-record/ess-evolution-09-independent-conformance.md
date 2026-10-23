---
format: aep.planning-md/3
id: architecture-decision-record:ess-evolution-09-independent-conformance
kind: architecture-decision-record
status: accepted
title: 09 — Conformance observes implementations independently
relations:
- decides: initiative:ess-evolution
revision: 3
transitions:
- {from: "proposed", to: "accepted", at: "2026-10-23T19:36:20Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"approval":1}}, correlation: "ess-0690"}
---
## Context

Tests derived from implementation lowering can reproduce its defects.

## Decision

Expected behavior comes from upstream scenarios and independent fixtures. Targets return observations; runners judge them. Preserve existing conformance runners/formats and exercise recorded execution, actual service behavior and Connectors independently.

## Rejected alternatives

Generating expectations from implementation code or target plans, silent missing capabilities, and treating selected-zero or skipped required lanes as success.

## Compatibility and migration

Retain existing versions/defaults. Prove sensitivity to wrong fields, dropped state/event updates, incorrect transitions and bypassed revocation. Generic UI scenarios, emitters and UI-action mutation acceptance are deferred to task:deferred-protocol-ui-bindings.

## Acceptance evidence

Operator-approved revision ess-evolution-20260915/1, recorded by approval-record:ess-evolution-20260915, supersedes the broader 2026-09-10 scope. See docs/design/ess-evolution/acceptance.md. New runtime and migration evidence remains outstanding.
