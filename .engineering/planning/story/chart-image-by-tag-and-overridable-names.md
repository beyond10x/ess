---
format: aep.planning-md/3
id: story:chart-image-by-tag-and-overridable-names
kind: story
status: draft
title: The chart takes an image by tag and lets its resource names be overridden
tags:
- adopter-report
- deployment-chain
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-delivery
revision: 2
---
## Outcome

The generated chart accepts an image by tag as well as by digest, and its Deployment and Service
names can be overridden instead of always being `<release>-<component>`.

## Evidence

An adopter on ess 0.55.0. On main the image renders as `repository@digest` only
(`crates/generate/ess-deployment/src/environment.rs:794`), filled by reconcile
(`crates/edge/ess-cli/src/recovery/mod.rs:1213-1216`), and names are
`{{ .Release.Name }}-<name>` (`environment.rs:751,764,917,935`) with no override.

## Acceptance

- A values file with a tag and no digest renders `repository:tag`; with a digest the digest wins;
  a production environment can still require a digest.
- A `fullnameOverride`-style value (or a per-workload name) renames Deployment and Service; the
  default names are unchanged.

## Evidence (0.90.0)

A second adopter, on 0.90.0, deploying a WebSocket push service with several interchangeable
implementations, hand-wrote its chart for the same two reasons. Re-verified on current main: a
build-output image still renders only as `repository@digest`
(`crates/generate/ess-deployment/src/environment.rs:1063`), filled by reconcile from the release's
artifact (`crates/edge/ess-cli/src/recovery/mod.rs:1213-1216`); since `ess-runtime/2` a process may
name an image pinned by digest instead (`crates/generate/ess-deployment/src/runtime.rs:133-137`,
rendered verbatim at `environment.rs:1064-1066`), which is still not a tag. Every object name is
still `{{ .Release.Name }}-<name>` with no override: workloads (`environment.rs:800`), the headless
Service and its reference (`environment.rs:813`, `environment.rs:1259`), the `CronJob`
(`environment.rs:850`), volume claims (`environment.rs:890`, `environment.rs:981`) and provided
Services (`environment.rs:1242`). This adopter additionally needs a provided endpoint's Service
named without the release prefix: the edge proxy addresses each implementation by a stable
in-cluster name that must not change when the release is renamed or the implementation is
swapped. Whatever override lands must also cover objects added by
`story:workload-declares-a-rollout-and-a-disruption-budget` (the `PodDisruptionBudget`), and must
keep the stack lock's derived in-cluster endpoint URLs pointing at the renamed Service
(`environment.rs:513-521`).
