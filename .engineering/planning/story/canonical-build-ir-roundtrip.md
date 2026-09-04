---
format: aep.planning-md/1
id: story:canonical-build-ir-roundtrip
kind: story
status: implemented
title: Canonical build IR round-trips
summary: Generated build IR with omitted empty collections remains readable by every ESS release command.
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
  path: crates/generate/ess-deployment/src/build.rs
- confidence: cited
  path: crates/generate/ess-deployment/tests/deployment.rs
revision: 6
---
## Outcome

Canonical `ess-build-ir/1` emitted by the compiler is accepted again at every persisted IR boundary, including release verification and bundling.

## Context

An empty build-secret set is omitted from canonical JSON. The strict deserializer must restore that field to its empty value rather than reject the compiler's own output. Platforms remain required and are always present.

## Acceptance

- Canonical build IR round-trips byte-for-byte when its optional secret set is empty.
- A release manifest can be verified and bundled against the round-tripped IR.
- The full ESS gate passes and the repair is released for downstream component publishers.

## Scope

- `crates/generate/ess-deployment/src/build.rs`
- `crates/generate/ess-deployment/tests/deployment.rs`
- `Cargo.toml`
- `Cargo.lock`
- `CHANGELOG.md`
