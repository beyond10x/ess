---
title: Import and project Kubernetes infrastructure
description: Turn a sanitized Kubernetes observation into typed infrastructure IR, diagnose it, compare it, and project reviewed manifests without applying them.
---

# Import and project Kubernetes infrastructure

The Kubernetes adapter imports existing observation bundles or reads an explicitly selected live
kubeconfig context. Legacy cluster observations produce `infra-ir/1`; the namespace topology
profile produces qualified `infra-ir/2`. Only the adapter
touches cluster authority; downstream analysis reads data.

## Import the committed observation

```shell-session
$ ess infra import kubernetes \
    --path examples/k3d-dev-cluster/observation.json \
    --out target/cluster.ir.json
```

The importer reports coverage and unresolved references. It does not invent missing ownership,
policy, or topology semantics.

## Diagnose and graph the IR

The `infra` verb sits in the `infra` area, which is why the word appears twice: `ess infra infra
diagnose` is the area path, and `ess infra diagnose` is its flat spelling — the same command, the
same bytes. See [Flat spellings](../reference/cli.md#flat-spellings).

```shell-session
$ ess infra infra diagnose --path target/cluster.ir.json
$ ess infra infra graph --path target/cluster.ir.json --format mermaid
```

Diagnosis is a report, not a deployment gate. Findings carry stable codes and the exact resources
that established them.

## Compare two observations

```shell-session
$ ess infra infra diff \
    --from examples/k3d-dev-cluster/observation.json \
    --to examples/k3d-dev-cluster/observation.drifted.json
```

Snapshot diff compares observed systems. It is distinct from projecting desired infrastructure
intent.

## Project manifests without applying them

```shell-session
$ ess generate project kubernetes \
    --spec examples/k3d-dev-cluster/expected.yaml \
    --ir target/cluster.ir.json \
    --out target/kubernetes-projection
```

The output contains manifests and explicit obligations. ESS never invokes `kubectl`, chooses a
cluster, or applies the projection.

## Scan a live cluster explicitly

```shell-session
$ ess infra import kubernetes \
    --context my-read-only-context \
    --observation-out target/observation.json \
    --out target/cluster.ir.json
```

Live access is outside the offline repository gate. Raw Secret `data`, `stringData`, and
last-applied configuration values are sanitized before serialization. Failed reads stop collection;
ESS never retries a failed cluster read in the current namespace. Keep the observation output
for review; do not treat sanitization as authority to publish operational topology.

## Collect one namespace's topology

```shell-session
$ ess infra import kubernetes \
    --context my-read-only-context --namespace app \
    --observation-out target/app.observation.json \
    --out target/app.ir.json
```

The namespace option selects a fixed topology profile. It reads namespaced collections only in
`app`, the exact Namespace object, and individual Nodes referenced by the collected Pods. It never
lists every namespace or node. A failed, incomplete or out-of-scope response prevents publication.
The reads are sequential, not an atomic snapshot of the API.

This profile omits ConfigMap and Secret keys/values, literal environment entries, probe definitions,
annotations, commands, arguments and unmodeled API fields before writing. Structural identities,
labels, images, selectors, references, resource envelopes and modeled runtime state remain.
Labels are structural data; the profile is not a general secret detector. Unsupported selector
terms, including `matchExpressions`, are refused instead of becoming match-all selectors.

The resulting `infra-observation/2` and `infra-ir/2` retain a typed `coverage` claim. The IR includes
that claim in its semantic digest. Native graph and drift exports use version 2 and retain it too.
Drift requires the same context, namespace and profile. An empty change list means no change in
the compared topology; omitted content is still unobserved. Node membership in this profile is the
set of referenced nodes, so it does not report cluster node additions or removals.
Changes outside the detailed drift rules produce `topology_digest_changed`, retaining the exact
before/after model digests instead of reporting no change.

Diagnosis reports the limited observation, intent simulation returns unknown, and manifest
projection refuses this profile. These consumers cannot treat omitted values or probes as absent.
Existing version 1 documents remain readable without reclassification as complete observations.
Upgrade readers before selecting this profile; older readers reject version 2.

## Connect implementation selections to observed workloads

`ess verify bindings` is available in the source preview; pin an exact published Git revision
containing this command. Released ESS 0.19.0 does not provide it. The service owns its semantic
model and realization; the system owns environment-specific `ess-observed-bindings/1` declarations.
Infrastructure remains a separate observed authority. A monolith may assign several components
to one implementation and bind that implementation to several named deployment roles.

Compile the service's realization to obtain its exact `realization_digest`. Existing
`ess-realization/1` requires a primary executable entrypoint. Use explicit `ess-realization/2`
when only source implementation membership is evidenced: components and implementations remain
nonempty, while entrypoints and actors are empty and conformance is absent. Do not invent an
executable interface. Version 2 has a separate digest domain; old readers reject it.

```yaml
format: ess-observed-bindings/1
id: billing-qa
realization_digest: sha256:REPLACE_WITH_COMPILED_REALIZATION_DIGEST
scope:
  context: my-read-only-context
  namespace: app
bindings:
  - id: api
    implementation: billing-binary
    workload:
      kind: deployment
      name: billing
    container: billing
    image: registry.example/billing:1.2.3
```

Component membership is inherited from the exact admitted realization. Supported workload kinds
are `deployment`, `statefulset` and `daemonset`. Optional per-binding `evidence` entries contain
`repository`, an exact 40-character Git `revision`, and relative `path`; they record attribution
without fetching or verifying those files. Unknown fields, duplicate targets, empty selections
and invalid references are refused before collection.

```shell-session
$ ess verify bindings --spec service/model --realization service/realization.yaml \
    --bindings system/qa.yaml --infra /external/qa.ir.json --format json
$ ess verify bindings --spec service/model --realization service/realization.yaml \
    --bindings system/qa.yaml --live --observation-out /external/new-qa.observation.json \
    --format json --markdown-out /external/new-qa.md
```

Output parents must exist and output files must be new. Live observations must be outside Git
checkouts. Live mode performs a fresh namespace-topology read with the document's explicit context
and namespace; collection failure produces an unknown report without falling back to an old file.
Offline mode is byte-deterministic for the same inputs and uses the observation's timestamp.
Neither mode changes a cluster or promotes observations into an accepted baseline.

The report checks scope, workload, container and exact workload-template image reference.
Missing targets within admitted scope and changed images violate the declaration. Wrong context
or missing/different namespace coverage leaves the result unknown. A digest-pinned container
artifact and identical digest-pinned template reference can establish immutable reference
agreement. Source artifacts and tag-only images leave that check unknown, even when the image
reference matches. Resolving a registry tag alone does not prove which source built it.

| Exit | Report status | Meaning |
|---|---|---|
| 0 | `satisfied` | Every supported selected check is satisfied. |
| 1 | `violated` | A check violated the declaration or the authored contract was refused. |
| 2 | `unknown` | Required evidence or acquisition is unavailable, with no known violation. |

A violation takes precedence over unknown; all applicable findings remain in the JSON report.
Syntax/argument errors can terminate at the CLI parser before a report exists. The report and
generated Markdown explicitly exclude running Pod image IDs, source-to-image build attestations,
runtime behavior, profile-specific operation exposure, health, replicas and omitted secret/config
values. A satisfied reference check is not runtime conformance. These checks can run unchanged
in a scheduled job; no AI interpretation is part of the verdict.
