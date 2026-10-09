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
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
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
