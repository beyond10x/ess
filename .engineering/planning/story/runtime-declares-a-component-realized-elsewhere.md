---
format: aep.planning-md/3
id: story:runtime-declares-a-component-realized-elsewhere
kind: story
status: draft
title: A runtime declares a component realized outside it
tags:
- adopter-report
- deployment-chain
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A runtime can declare that a component of its system is realized outside it (by another runtime,
system or team), and `ess specify runtime compile` accepts a runtime that realizes only some of the
system's components.

## Evidence

An adopter ran the deployment chain on ess 0.55.0 for a two-component system (a push server and
the backend it calls) and got `[missing_component:Runtime] semantic component backend has no
realized workload`. On main every component must be realized by this runtime's workloads
(`crates/generate/ess-deployment/src/runtime.rs:686-695`); `RuntimeSpec` (`runtime.rs:196-219`,
`deny_unknown_fields`) has no field for a component realized elsewhere. Workaround in use: listing
the external component on another workload, which states something false.

## Acceptance

- A runtime that declares the external component compiles; one that neither realizes nor declares
  it is still refused `missing_component`.
- A declared-external component gets no workload, image or chart resources; endpoints it provides
  are consumed as external.
- The runtime format change is decided and documented (new field, format consequence).
