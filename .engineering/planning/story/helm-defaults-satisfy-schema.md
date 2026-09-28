---
format: aep.planning-md/3
id: story:helm-defaults-satisfy-schema
kind: story
status: implemented
title: Generate Helm defaults that satisfy their schema
summary: Ensure every configuration-neutral ESS chart passes Helm lint before environment binding.
relations:
- derived_from: epic:oci-component-delivery
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: crates/generate/ess-deployment/src/environment.rs
- confidence: cited
  path: crates/generate/ess-deployment/tests/deployment.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-04T01:42:36Z", actor: "human:timo", revision: 4, imported: true}
- {from: "proposed", to: "active", at: "2026-09-04T01:42:40Z", actor: "human:timo", revision: 5, imported: true}
- {from: "active", to: "implemented", at: "2026-09-04T01:44:30Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}, imported: true}
---
# Story: Generate Helm defaults that satisfy their schema

## Outcome

Every configuration-neutral Helm chart projected from a valid ESS runtime passes Helm lint before environment-specific values are supplied.

## Acceptance

- The generated default service account name satisfies the generated JSON Schema.
- A projection test verifies the default values and schema remain aligned.
- A real Helm lint run against an ESS-projected stateful service chart succeeds.
- ESS patch version and release notes identify the compatibility fix.

## Scope

- crates/generate/ess-deployment/src/environment.rs
- crates/generate/ess-deployment/tests/deployment.rs
- Cargo.toml
- CHANGELOG.md

## Out of Scope

Environment-specific service accounts or weakening the chart schema.

## Open Questions

None.
