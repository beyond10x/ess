---
format: aep.planning-md/3
id: story:workload-declares-a-shutdown-drain
kind: story
status: draft
title: A workload declares how long it takes to leave rotation and drain, and the chart derives the preStop delay and the grace period from it
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-delivery
revision: 1
---

## Outcome

A runtime's workload can declare how it shuts down: how long its pods stay up after termination
starts so that load balancers stop routing to them, and how long the process needs after the stop
signal to finish in-flight work. The generated chart derives each container's `preStop` delay and
the pod's `terminationGracePeriodSeconds` from that one declaration, so the two cannot disagree.

## Evidence

An adopter on 0.90.0 deploying a WebSocket push service hand-wrote the Helm chart, among other
reasons because long-lived connections must be drained on a rollout: the pod must leave rotation
before the process is signalled, and the kubelet must not kill it before open connections close.
On current main:

- `ContainerRole` (`crates/generate/ess-deployment/src/runtime.rs:147-190`) and `Workload`
  (`runtime.rs:203-238`) have no lifecycle hook, no stop delay and no grace period.
- The pod template renders no `terminationGracePeriodSeconds`
  (`crates/generate/ess-deployment/src/environment.rs:905-1012`) and no container renders
  `lifecycle` (`environment.rs:1056-1199`), so every pod gets the platform default and an
  immediate stop signal.

Minimal reproduction (neutral, `ess-runtime/2`), refused today with an unknown-field error on
`shutdown`:

```yaml
workloads:
  - name: push
    components: [push]
    containers: [server]
    replicas: 3
    shutdown:
      deregister_seconds: 5
      drain_seconds: 30
```

## Fit review

1. **The need, apart from the syntax.** A workload that holds connections (streams, sockets,
   long polls) or in-flight work must be removed from rotation before its process is stopped, and
   must be allowed to finish before it is killed. Those are two durations of one shutdown, and the
   grace period must cover both. Nothing in the runtime document states either
   (`runtime.rs:147-238`). The requester's proposal (theirs): `lifecycle.pre_stop_seconds` plus
   `termination_grace_period_seconds`, "or better a `drain` declaration from which both derive
   (grace >= preStop + drain + 10s)".
2. **Class: gap.** No key reaches `lifecycle` or `terminationGracePeriodSeconds`
   (`environment.rs:905-1199`).
3. **Can it already be expressed?** No. `docs/design/runtime-workload-kinds.md:35-47` lists every
   pod setting; none is a shutdown setting. An entrypoint wrapper that sleeps on the stop signal
   cannot extend the platform's grace period, which the kubelet enforces from outside.
4. **Fit.** Deployment intent; reaches no cell of grid R, O or G2 (`docs/design/language-grids.md`).
   Vocabulary: durations in the runtime document are integer `_seconds` keys
   (`run_deadline_seconds`, `start_deadline_seconds`, `docs/design/runtime-workload-kinds.md:106-108`),
   so `deregister_seconds` and `drain_seconds` follow that spelling. Placement: a pod has one grace
   period, so the declaration sits on the `Workload`, like `security` and `service_account_token`
   (`docs/design/runtime-workload-kinds.md:49-52`); the `preStop` delay renders on every container
   of the pod, because each container receives the stop signal after its own hook, and a container
   left without it would stop while its neighbour still serves. Siblings: every workload kind
   renders its pod through `render_pod_template` (`environment.rs:905`), so a Deployment,
   StatefulSet and scheduled job all take it. Targets: the Helm projection renders it; the recovery
   profile reads Deployment and StatefulSet already
   (`crates/edge/ess-cli/src/recovery/model.rs:517-546`) and the new fields sit inside them;
   generated code, synthesis and `ess verify diff` do not read runtime documents.
5. **A second adopter.** An HTTP API behind a cloud load balancer whose target deregistration lags
   pod termination by several seconds, so requests fail on every rollout unless the pod waits; or a
   queue consumer that must finish the message it holds before it exits.
6. **Cost.** New keys in a released format, so `ess-runtime/3` beside `/1` and `/2`, shared with
   `story:container-declares-named-ports-and-probes-name-a-port` and
   `story:workload-declares-a-rollout-and-a-disruption-budget` (one format per decision,
   `docs/design/generic-language-features.md` § Policy, policy 2). The `preStop` delay renders as
   the native `sleep` action of `lifecycle.preStop`, not an `exec` of a `sleep` binary that a minimal
   image may lack; that action needs a cluster that supports it (on by default from Kubernetes
   1.30 — I don't know which minimum the deliver guide states today, and the story records it).
   New diagnostics: an empty `shutdown:`, a zero `drain_seconds`, a value past the field's range.
7. **Considered.** (a) Change nothing: the adopter keeps a hand-written chart. (b) The requester's
   first shape, two independent keys: refused, because the grace period would then be a second
   statement that can contradict the hook (a grace shorter than the delay kills the pod mid-hook).
   (c) The requester's preferred shape, one declaration both derive from: taken, without the fixed
   10-second margin, which is one adopter's policy; the author's `drain_seconds` is the bound the
   process is held to and includes whatever margin it needs.

## Decisions

- **Accept, redesigned.** One declaration on `Workload`, `shutdown: {deregister_seconds,
  drain_seconds}`, from which both settings derive; the requester's margin is not built in.
- `terminationGracePeriodSeconds = deregister_seconds + drain_seconds` (the platform's countdown
  starts before the hook runs, so the sum is exactly the time both phases need).
- `deregister_seconds` (optional, positive) renders `lifecycle.preStop.sleep.seconds` on every
  container of the pod; absent, no `lifecycle` is rendered.
- `drain_seconds` is required inside `shutdown:` and positive.
- Format consequence: `ess-runtime/3` compiling to `ess-runtime-ir/3`; `/1` and `/2` refuse
  `shutdown` naming `ess-runtime/3`.

## Acceptance

- A `/3` workload with `shutdown: {deregister_seconds: 5, drain_seconds: 30}` renders
  `terminationGracePeriodSeconds: 35` in its pod spec and `lifecycle.preStop.sleep.seconds: 5` on
  every container of the pod, for a Deployment, a StatefulSet and a scheduled job alike.
- `shutdown: {drain_seconds: 30}` renders `terminationGracePeriodSeconds: 30` and no `lifecycle`.
- Refused by name: `shutdown: {}`; `drain_seconds` absent or 0; `deregister_seconds: 0`; a sum past
  the Kubernetes field's integer range.
- An unset setting changes no chart bytes: a `/3` runtime without `shutdown` renders a chart
  byte-identical to the same runtime as `/2`.
- `/1` and `/2` documents carrying `shutdown` are refused naming `ess-runtime/3`; existing fixtures
  keep their IR digests and chart bytes.
- `docs/design/runtime-workload-kinds.md` records the key, the derivation and the minimum cluster
  version for the `sleep` action; the deliver guide shows it.

## Scope (inferred)

- `crates/generate/ess-deployment/src/runtime.rs` (`Workload`), `workload_kind.rs` (key lists,
  validation), `environment.rs` (`render_pod_template`, `render_container`).
- `crates/generate/ess-deployment/tests/workload_kinds/`.
- `docs/design/runtime-workload-kinds.md`, `website/docs/guides/deliver/build-and-chart.md`,
  `website/docs/reference/formats.md`, `CHANGELOG.md`.
- Shares `ess-runtime/3` with the named-ports and rollout stories.
