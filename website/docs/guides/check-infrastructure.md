---
title: Import and project Kubernetes infrastructure
description: Turn a sanitized Kubernetes observation into typed infrastructure IR, diagnose it, compare it, and project reviewed manifests without applying them.
---

# Import and project Kubernetes infrastructure

The Kubernetes adapter has two input modes: an existing sanitized `infra-observation/1` bundle, or
an explicitly selected live kubeconfig context. Both produce typed `infra-ir/1`. Only the adapter
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
last-applied configuration values are rejected before serialization. Keep the observation output
for review; do not treat sanitization as authority to publish operational topology.
