---
format: aep.planning-md/3
id: architecture-decision-record:ess-evolution-10-infra-intention-observation
kind: architecture-decision-record
status: accepted
title: 10 — Infrastructure intention and observation remain distinct
relations:
- decides: initiative:ess-evolution
revision: 2
transitions:
- {from: "proposed", to: "accepted", at: "2026-10-23T19:36:20Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"approval":1}}, correlation: "ess-0690"}
---
## Context
Desired deployments do not establish actual runtime facts.

## Decision
Preserve InfraSpec, observed InfraIr and deployment plans, adding typed links from service requirements to existing checks.

## Rejected alternatives
Treat desired declarations as observations; generation that applies infrastructure.

## Compatibility and migration
Keep current infrastructure projections/refusals and add deterministic checks using rendered manifests and separately captured or synthetic observations.

## Acceptance evidence
Operator decision: supplied implementation plan, 2026-09-10. See docs/design/ess-evolution/feature-preservation.md and acceptance.md. Implementation and runtime acceptance are not yet established.
