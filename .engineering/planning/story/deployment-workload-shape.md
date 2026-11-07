---
format: aep.planning-md/3
id: story:deployment-workload-shape
kind: story
status: draft
title: 'Deployment projection: admin port, drain, rollout, image by tag, edge and optional dependencies'
summary: 'The generated Helm chart cannot carry what a production service needs: a second port with probes, preStop and grace period, rollout strategy and PodDisruptionBudget, image by tag, stable names, an edge with affinity, optional stack dependencies.'
owner: timo
revision: 4
---
# Story: deployment-workload-shape

## Outcome

A team deploying a service from its ESS specification gets a generated Helm chart it can install in
production as it is: a second (admin) port with probes on it, a drain-aware shutdown, a rollout and
disruption policy, an image by tag, stable object names, an edge with WebSocket upgrade, long-poll
timeouts and session affinity, and optional stack dependencies wired in, so no post-renderer and no
hand-written parallel chart is needed.

## Context

Found by a downstream adopter on 2026-11-08: a service specification (a Bayeux push server with
several language implementations, ESS 0.90.0) generates one chart per runtime with
`ess generate project helm`, and its deployment had to hand-write a chart instead. Depending on the
generated chart does not hold, because a parent chart cannot change the Deployment a dependency
renders; the adopter's benchmark patches it with a Helm post-renderer. Re-checked against ESS 0.90.0
`origin/main`:

| Needed | ESS 0.90.0 today |
| --- | --- |
| admin port as a second container port | `ContainerRole` has one `http_port` (`crates/generate/ess-deployment/src/runtime.rs:156`), rendered as the only port (`environment.rs:1086-1091`) |
| readiness and liveness HTTP probes on the admin port | `readiness_path` / `liveness_path` bind to `http_port` (`runtime.rs:159-162`, `environment.rs:1092-1093`); the TCP forms (`runtime.rs:165-168`) cannot carry an HTTP path on another port |
| preStop and `terminationGracePeriodSeconds` >= preStop + drain + 10 s | no field on `ContainerRole` or `Workload` (`runtime.rs:149-190`, `:205-238`) |
| rolling update `maxSurge: 1, maxUnavailable: 0`, PodDisruptionBudget | no field; no `PodDisruptionBudget` in `crates/generate/ess-deployment/src` |
| image by tag (`<repository>:<tag>`), digest optional | `repository@digest` only (`environment.rs:1063`; `Process.image` is a build output or a digest-pinned reference, `runtime.rs:135-137`) |
| a Service with a fixed name that callers already use | `<release>-<name>` (`environment.rs:800`, `:1242`) |
| an edge: provided endpoint behind a proxy with upgrade, request timeout and cookie affinity | no ingress, edge or affinity construct |
| optional stack dependencies (a NATS cluster, Redis with Sentinel) with the configuration that wires them | `ess-stack/1` names them only as external systems |

## Acceptance

1. A runtime document can declare named ports beyond `http_port`, and readiness and liveness probes
   that name a port and an HTTP path; the generated Deployment carries both ports and both probes.
2. A runtime document can declare a drain (or `pre_stop_seconds` plus a termination grace period);
   the generated Deployment carries a preStop hook and a `terminationGracePeriodSeconds` derived
   from it.
3. A workload can declare a rollout strategy and a disruption budget; the chart renders the strategy
   and a `PodDisruptionBudget`.
4. An image can be referenced by tag with an optional digest, and the main Service's name can be
   fixed; existing digest-only specifications render byte-identically (canonical bytes preserved).
5. A provided endpoint can be exposed through an edge with WebSocket upgrade, a request timeout and
   session affinity by cookie, rendered as an edge deployment (for example Envoy) or as documented
   Ingress annotations.
6. A stack can declare NATS and Redis-with-Sentinel as optional bundled dependencies with the
   configuration slots they fill.
7. The adopter can regenerate a chart that replaces its hand-written one, checked by rendering both
   with the same values and comparing the objects.

## Out of Scope

- Edge-proxy tuning beyond upgrade, timeout and affinity.
- Monitoring objects (ServiceMonitor, PrometheusRule, dashboards).
- Applying anything to a cluster: projections never apply infrastructure.

## Ambiguities

- Whether the edge is an ESS construct or a separate projection (`requires-stakeholder-input`: the
  ESS maintainers decide; the adopter needs only acceptance 5's outcome).
- Whether items 1-6 are one story or six: written as one at the requester's choice; the ESS planner
  may split it.
