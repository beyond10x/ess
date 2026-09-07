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
