---
format: aep.planning-md/1
id: story:oci-component-release
kind: story
status: implemented
title: Execute and compose OCI component releases
summary: Make independently released component bundles the reusable ESS deployment unit.
relations:
- decomposes: epic:oci-component-delivery
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: README.md
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/generate/ess-deployment
- confidence: cited
  path: website/docs
revision: 7
---
# Story: Execute and compose OCI component releases

## Outcome

A repository-owned ESS component descriptor can be built standalone, published as a verified OCI bundle, hydrated into a release catalogue, and reconciled as one independent Helm release.

## Acceptance

- ESS validates and compiles component descriptors over existing semantic, realization, build, runtime, and release formats.
- Runtime models provided endpoints, named persistent volumes, and explicit mounts; Helm projection emits Services, claims, mounts, and workload-safe selectors.
- OCI publication and fetch are explicit credential-edge operations, idempotent by immutable digest, and admit only revalidated canonical bundle bytes to the cache.
- Stack compilation derives named internal or external endpoint bindings.
- Deployment reconciliation applies only changed releases in rollout order and refuses cross-cluster transitions or implicit removals.
