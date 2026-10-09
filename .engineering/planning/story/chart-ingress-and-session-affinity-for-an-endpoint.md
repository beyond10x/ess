---
format: aep.planning-md/3
id: story:chart-ingress-and-session-affinity-for-an-endpoint
kind: story
status: draft
title: The chart exposes a provided endpoint through an Ingress with session affinity
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

The generated chart can expose a provided endpoint through an Ingress (host, path) and set session
affinity on its Service, so an edge component is reachable without a hand-written manifest.

## Evidence

An adopter on ess 0.55.0 needed an ingress with session affinity for an edge. On main the chart
has no Ingress, Services carry no `sessionAffinity`
(`crates/generate/ess-deployment/src/environment.rs:917,935`), and `ProvidedEndpoint` has no host
or path (`runtime.rs:90-101`).

## Acceptance

- An environment (host is per environment) declares the ingress for a provided endpoint; the
  chart renders an Ingress and the Service's session affinity; a projection test pins the bytes.
- An endpoint without the declaration renders as today.
