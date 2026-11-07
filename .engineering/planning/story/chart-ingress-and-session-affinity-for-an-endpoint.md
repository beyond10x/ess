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
- serves: vision:O2
- decomposes: epic:adopter-gaps-delivery
revision: 2
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

## Evidence (0.90.0)

An adopter on 0.90.0, deploying a WebSocket push service behind an edge proxy, hand-wrote its chart
partly for this. Re-verified on current main: the chart renders no Ingress or route object at all
(the file set is fixed at `crates/generate/ess-deployment/src/environment.rs:721-727`), a provided
Service carries no `sessionAffinity` (`environment.rs:1242`), and `ProvidedEndpoint` still has no
host, path or affinity (`crates/generate/ess-deployment/src/runtime.rs:106-116`). This adopter
additionally needs, at the edge: (a) WebSocket upgrade passed through to the backend; (b) a request
or idle timeout well above common proxy defaults, for upgraded connections and for long-poll
requests that are held open by design; (c) session affinity that keeps a client on one backend pod
for the life of its session, including a bounded affinity timeout
(`sessionAffinityConfig.clientIP.timeoutSeconds` on a Service). The `Ingress` API has no portable
field for (a) or (b); they exist only as controller-specific annotations, while the Gateway API's
`HTTPRoute` has portable request timeouts — the story has to choose which object it renders, or
refuse (a) and (b) by name. The specification-side binding of a WebSocket upgrade is a separate,
already decided question (`decision-blocker:websocket-upgrade-binding-form`, cleared, for
`story:http-binding-carries-transport-framing`); this story stays the deployment half and should
not invent a second spelling of the upgrade.
