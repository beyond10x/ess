---
format: aep.planning-md/3
id: story:runtime-workload-probes-resources-placement-annotations
kind: story
status: draft
title: A runtime states TCP probes, resources, affinity and annotations, and the chart renders them
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

A runtime's workloads and containers can state TCP readiness and liveness probes, resource
requests and limits, affinity or anti-affinity, and annotations, and the generated chart renders
them.

## Evidence

An adopter on ess 0.55.0 found none of these in the runtime or chart. On main:

- probes: `ContainerRole` has only `readiness_path`/`liveness_path`
  (`crates/generate/ess-deployment/src/runtime.rs:142-146`), rendered as `httpGet` only
  (`environment.rs:816,819`); a TCP probe exists only as an `ess infra` remedy
  (`crates/infra/infra-spec/src/raw.rs:170,199`).
- resources: none on containers (`runtime.rs:130-162`); the only chart `resources:` is the PVC
  request (`environment.rs:885,893`).
- affinity: none on `Workload` (`runtime.rs:174-191`) or in `render_workloads`.
- annotations: metadata is name and labels only (`environment.rs:751`).

## Acceptance

- Each setting validates in the runtime document, is rendered into the chart, and is checked by
  a chart projection test; an unset setting changes no chart bytes.
- Resources and placement can be overridden per environment, or the story records why not.
- The runtime format consequence is decided and recorded.
