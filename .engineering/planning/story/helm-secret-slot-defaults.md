---
format: aep.planning-md/1
id: story:helm-secret-slot-defaults
kind: story
status: implemented
title: Render valid Helm defaults for secret slots
summary: Emit every typed runtime secret slot in default values so generated charts render and lint.
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
  path: crates/generate/ess-deployment/src/environment.rs
- confidence: cited
  path: crates/generate/ess-deployment/tests/deployment.rs
revision: 5
---
## Outcome

Every secret declared by an `ess-runtime/1` container has a deterministic `{name, key}` entry in generated Helm defaults, so a configuration-neutral chart renders and lints before an environment binds an actual Secret name.

## Acceptance

- Secret slots are deduplicated by stable slot identity across containers.
- Generated `values.yaml` includes an empty environment-owned Secret name and the declared key for each slot, never credential bytes.
- The generated values schema describes the closed `{name, key}` shape.
- A regression test renders or lints a chart whose runtime declares a secret slot.

## Scope

- `crates/generate/ess-deployment/src/environment.rs`
- `crates/generate/ess-deployment/tests/deployment.rs`
- `Cargo.toml`
- `Cargo.lock`
- `CHANGELOG.md`
