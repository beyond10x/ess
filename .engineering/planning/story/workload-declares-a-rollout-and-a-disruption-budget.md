---
format: aep.planning-md/3
id: story:workload-declares-a-rollout-and-a-disruption-budget
kind: story
status: draft
title: A workload declares how it rolls out and how many of its pods may be disrupted, and the chart renders the strategy and a PodDisruptionBudget
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-delivery
revision: 1
---

## Outcome

A runtime's continuously running workload can declare its rolling-update limits (how many extra
pods a rollout may add, how many may be unavailable) and a disruption budget (how many of its pods
must stay up, or may be down, during voluntary disruption such as a node drain). The generated
chart renders the Deployment's `strategy` and one `policy/v1` `PodDisruptionBudget` per workload
that declares one.

## Evidence

An adopter on 0.90.0 deploying a WebSocket push service with several replicas hand-wrote the Helm
chart, among other reasons because each rollout and each node drain must keep enough pods up to
hold the open connections. On current main:

- `Workload` (`crates/generate/ess-deployment/src/runtime.rs:203-238`) has no rollout or
  disruption key.
- `render_workloads` renders `replicas`, `selector` and the template only
  (`crates/generate/ess-deployment/src/environment.rs:777-840`), so a Deployment takes the
  platform's default strategy; no `PodDisruptionBudget` is rendered anywhere in
  `ess-deployment`, and the chart's files are fixed (`environment.rs:721-727`).
- ESS already reads disruption budgets on the observed side: `ess infra` models
  `PodDisruptionBudget` with `min_available` and `max_unavailable`, each "a count or a percentage,
  as declared" (`crates/infra/infra-domain/src/policy.rs:44-55`), and can propose creating one per
  workload (`crates/infra/infra-project/src/project.rs:446-454`).

Minimal reproduction (neutral, `ess-runtime/2`), refused today with an unknown-field error:

```yaml
workloads:
  - name: push
    components: [push]
    containers: [server]
    replicas: 4
    rollout: {max_surge: 1, max_unavailable: 0}
    disruption_budget: {min_available: 3}
```

## Fit review

1. **The need, apart from the syntax.** A replicated workload has an availability floor during
   change: a rollout must not take down more pods than the service tolerates, and an operator's
   node drain must not evict more than that either. The runtime document states the replica count
   (`runtime.rs:212-215`) but neither floor. The requester's proposal (theirs): "rollout strategy
   (maxSurge/maxUnavailable) and a PodDisruptionBudget".
2. **Class: gap.** No key reaches a Deployment `strategy` or a `PodDisruptionBudget`
   (`environment.rs:777-840`, `environment.rs:721-727`).
3. **Can it already be expressed?** No. `docs/design/runtime-workload-kinds.md:35-47` lists every
   workload key. An `ess infra` specification can expect a budget and propose one
   (`crates/infra/infra-project/src/project.rs:446-454`), but that is an observation of a cluster
   held against a separate specification, not what the generated chart renders; using it as the
   answer would move the figure out of the deployment the runtime document describes.
4. **Fit.** Deployment intent; no cell of grid R, O or G2 (`docs/design/language-grids.md`).
   Vocabulary: `min_available` and `max_unavailable` are the spellings `ess infra` already uses for
   the same object (`policy.rs:52-54`), and `max_surge` follows them; values are a non-negative count
   or a percentage, as there. Placement: both are workload facts, on `Workload` beside `replicas`.
   Siblings: a StatefulSet's rolling update takes no surge and its `maxUnavailable` is behind a
   feature gate that is not on by default — I don't know whether that changed in the newest
   Kubernetes release, and the story checks before it decides — so `rollout:` is refused on a
   stateful workload, naming why; the disruption budget applies to Deployments and StatefulSets
   alike. A scheduled workload has no standing pods to roll out or protect
   (`docs/design/runtime-workload-kinds.md:137-141`), so both are refused on it. Targets: the Helm
   projection renders them; the recovery profile of `ess generate deployment reconcile` admits only
   Deployment, StatefulSet and Service (`crates/edge/ess-cli/src/recovery/model.rs:517-546`), so a
   chart with a budget is refused there by name until admission is decided, as the design leaves
   it for the `CronJob` (`docs/design/runtime-workload-kinds.md:161-163`). The budget's selector is
   the workload's own pod labels, the ones affinity already selects on (`environment.rs:1028-1036`),
   and its name follows whatever `story:chart-image-by-tag-and-overridable-names` decides for
   object names.
5. **A second adopter.** A stateless API with three replicas on a cluster whose nodes are drained
   weekly for patching: without a budget, a drain can evict all three at once. Or a service with a
   slow warm-up that must surge one new pod before retiring an old one.
6. **Cost.** New keys in a released format: `ess-runtime/3`, shared with the named-ports and
   shutdown stories (`docs/design/generic-language-features.md` § Policy, policy 2). A new chart file
   `templates/disruption-budgets.yaml`, rendered only when a workload declares a budget. New
   diagnostics: both or neither of `min_available`/`max_unavailable`, a malformed percentage, both
   rollout limits zero, a key on a workload kind that cannot take it, a budget that can never allow
   an eviction at the declared replica count.
7. **Considered.** (a) Change nothing: the adopter keeps a hand-written chart. (b) Derive the
   budget from the component's semantic replica floor (`runtime.rs:721-736`): refused, because it
   would add a budget to every existing runtime and change chart bytes nobody asked to change, and
   because the semantic floor bounds the replica count, not availability during a drain.
   (c) The requester's shape, both settings stated explicitly: taken, in `ess infra`'s spelling.
   (d) A `Recreate` strategy: not asked for; left out, and the story says so.

## Decisions

- **Accept, redesigned** (spelling only): `rollout: {max_surge, max_unavailable}` and
  `disruption_budget: {min_available | max_unavailable}` on `Workload`, each value a count or a
  `"<n>%"` percentage, the names taken from `ess infra`'s model of the same object.
- `rollout:` renders `strategy: {type: RollingUpdate, rollingUpdate: {...}}` on a Deployment and is
  refused on a StatefulSet and a scheduled workload.
- `disruption_budget:` renders one `policy/v1` `PodDisruptionBudget` per workload, refused on a
  scheduled workload; exactly one of its two keys.
- A count `min_available` at or above the declared replicas, or `max_unavailable: 0`, is refused:
  such a budget blocks every voluntary eviction. A replica count overridden per environment is not
  rechecked here; the story records that interaction with
  `story:environment-binding-sets-workload-replicas`.
- Recovery-profile admission of `PodDisruptionBudget` is not decided here; a chart carrying one is
  refused by `deployment reconcile` by name.
- Format consequence: `ess-runtime/3` compiling to `ess-runtime-ir/3`; `/1` and `/2` refuse both keys
  naming `ess-runtime/3`.

## Acceptance

- A `/3` Deployment workload with `rollout: {max_surge: 1, max_unavailable: 0}` renders
  `strategy.type: RollingUpdate` with those values; `max_surge: "25%"` renders the percentage as a
  string.
- A `/3` workload with `disruption_budget: {min_available: 3}` and `replicas: 4` renders
  `templates/disruption-budgets.yaml` with one `policy/v1` `PodDisruptionBudget` whose selector is
  the workload's pod labels and whose `minAvailable` is 3.
- Refused by name: both or neither budget key; both rollout limits zero; a percentage outside
  0-100% or not of the form `<n>%`; `rollout:` on a StatefulSet or scheduled workload;
  `disruption_budget:` on a scheduled workload; a budget that permits no eviction at the declared
  replicas.
- An unset setting changes no chart bytes and adds no file: a `/3` runtime without either key
  renders the same file set and bytes as the same runtime as `/2`.
- `/1` and `/2` documents carrying either key are refused naming `ess-runtime/3`; existing fixtures
  keep their IR digests and chart bytes.
- `deployment reconcile` refuses a chart containing a `PodDisruptionBudget` with a diagnostic naming
  the kind, and a test pins that refusal.
- `docs/design/runtime-workload-kinds.md` records the keys, the StatefulSet refusal and its reason.

## Scope (inferred)

- `crates/generate/ess-deployment/src/runtime.rs` (`Workload`), `workload_kind.rs` (key lists,
  validation), `environment.rs` (`project_helm` file set, `render_workloads`, a new budget
  renderer).
- `crates/edge/ess-cli/src/recovery/` (refusal test only; no new admitted kind).
- `crates/generate/ess-deployment/tests/workload_kinds/`.
- `docs/design/runtime-workload-kinds.md`, `website/docs/guides/deliver/build-and-chart.md`,
  `website/docs/reference/formats.md`, `CHANGELOG.md`.
- Shares `ess-runtime/3` with the named-ports and shutdown stories.
