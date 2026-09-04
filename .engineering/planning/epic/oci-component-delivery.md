---
format: aep.planning-md/1
id: epic:oci-component-delivery
kind: epic
status: active
title: OCI-native independent component delivery
summary: Build, bundle, cache, resolve, and reconcile independently released ESS components.
revision: 3
---
# Epic: OCI component delivery

## Outcome

ESS owns the typed and executable path from a repository component descriptor through build, OCI release bundles, cached catalogue hydration, and affected-only Helm reconciliation.

## Acceptance

- Component builds, manifests, runtime contracts, charts, evidence, and immutable artifacts remain one digest-verified chain.
- Explicit executor commands perform credentialed BuildKit, OCI, and Helm work while compiler APIs remain pure.
- A second fetch is served from the content-addressed cache and a one-component change reconciles only that Helm release.
