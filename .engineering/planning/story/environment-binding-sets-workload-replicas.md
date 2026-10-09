---
format: aep.planning-md/3
id: story:environment-binding-sets-workload-replicas
kind: story
status: draft
title: An environment's release binding sets a workload's replica count
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

An `ess-environment/1` release binding can set a workload's replica count, so several topologies of
one runtime (1, 2 or 4 replicas) are separate environments, and `deployment reconcile` writes it.

## Evidence

An adopter on ess 0.55.0 got `unknown field` for `replicas` on a release binding. On main
`ReleaseBinding` has no `replicas` and rejects unknown fields
(`crates/generate/ess-deployment/src/environment.rs:29-46`). The chart reads
`.Values.workloads.<w>.replicas` (`environment.rs:757`), defaulted from the runtime's
`Workload.replicas` (`runtime.rs:185`), but `values_document` never writes `workloads`
(`crates/edge/ess-cli/src/recovery/mod.rs:1209-1221`).

## Acceptance

- An environment binding with `replicas` for a named workload validates; one naming an unknown
  workload is refused by name.
- `deployment reconcile` writes the replica count into the values it produces; without the key
  the runtime's default stands.
- The format consequence for `ess-environment/1` is decided and recorded.
