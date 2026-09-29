---
title: Project a build and a Helm chart
sidebar_position: 1
description: Compile a component's build graph and runtime model, then project BuildKit inputs and a configuration-neutral Helm chart without running either.
---

# Project a build and a Helm chart

A component repository describes how its image is built (`ess-build/1`) and how it runs
(`ess-runtime/1`). ESS compiles both into canonical IR and projects them into files other tools
consume: Dockerfile and Bake inputs for BuildKit, and a Helm chart. The projections run nothing.
[Independent component delivery](../../concepts/component-delivery.md) describes where these files
sit in the whole delivery path.

The commands on this page were run against the `examples/oracle-fixture` specification with the
documents shown here.

## Compile the build graph

A build graph is a list of nodes (a pinned base image, a source directory, a command, an image, a
file taken from a build step) and the outputs a release publishes:

```yaml
format: ess-build/1
build: oracle-runtime
platforms:
  - os: linux
    architecture: amd64
nodes:
  - id: base
    kind: oci_base
    reference: docker.io/library/alpine
    digest: sha256:0000000000000000000000000000000000000000000000000000000000000000
  - id: source
    kind: source
    path: .
    destination: /src
  - id: compile
    kind: run
    base: base
    argv: [cp, /src/oracle, /usr/local/bin/oracle]
    mounts:
      - kind: input
        from: source
        target: /src
  - id: runtime-image
    kind: image
    rootfs: compile
    config:
      entrypoint: [/usr/local/bin/oracle]
      user: "10001"
  - id: chart-file
    kind: artifact
    from: compile
    path: /src/chart.tgz
outputs:
  - name: app
    release_unit: oracle-runtime
    node: runtime-image
    kind: oci_image
    repository: registry.example/oracle
  - name: chart
    release_unit: oracle-chart
    node: chart-file
    kind: helm_chart
```

```shell-session
$ ess generate build compile --path build.yaml --out build-ir.json
oracle-runtime — 5 node(s), 2 output(s), compiled to build-ir.json
$ ess generate build graph --path build.yaml
flowchart LR
  subgraph build_graph["oracle-runtime build graph"]
    n0["base<br/><small>pinned OCI base</small>"]
    ...
```

Compilation refuses a secret mount that the graph does not declare under `secrets:`, and a cycle
between nodes. `build graph` renders the same validated graph as Mermaid source.

## Project BuildKit inputs

```shell-session
$ ess generate project buildkit --ir build-ir.json --out buildkit
3 BuildKit file(s) projected to buildkit without executing them
```

The directory holds `Dockerfile.ess`, `docker-bake.hcl` and a copy of the build IR. Every base is
pinned by digest, and a `run` step has no network unless it declares `network: sandbox`:

```dockerfile
FROM docker.io/library/alpine@sha256:0000…0000 AS base

FROM scratch AS source
COPY [".","/src"]

FROM base AS compile
RUN --network=none --mount=type=bind,from=source,source=/,target=/src,ro ["cp","/src/oracle","/usr/local/bin/oracle"]
```

`ess generate build execute` compiles, projects and then runs Docker Buildx Bake over the result,
with `--target`, `--set`, `--push` and `--load` passed to Bake. It is an explicit executor
command: it needs Docker Buildx and, with `--push`, registry credentials. It was not run for this
page.

## Compile the runtime model

The runtime model says which processes, containers and workloads run the component. It is bound
by digest to three things: the compiled specification, the realization and the build IR.

```yaml
format: ess-runtime/1
runtime: oracle-runtime
semantic_digest: sha256:7ca8e1ba…
realization_digest: sha256:527b40ae…
build_digest: sha256:8f82170b…
processes:
  - name: server
    image: app
containers:
  - name: server
    process: server
    http_port: 8080
    readiness_path: /ready
    liveness_path: /live
    config:
      - name: log-level
        environment: LOG_LEVEL
        kind: optional
    secrets:
      - name: database-password
        environment: DATABASE_PASSWORD
        key: password
    volume_mounts:
      - volume: data
        mount_path: /var/lib/oracle
workloads:
  - name: oracle
    components: [order-service, dispatch-service]
    containers: [server]
    replicas: 1
    volumes:
      - name: data
        size: 1Gi
provided_endpoints:
  - name: api
    workload: oracle
    container: server
    scheme: http
```

Where each digest comes from:

| Field | Value |
|---|---|
| `semantic_digest` | the compiled specification's digest. `ess specify realization compile` names it when a realization carries another one: `SpecificationMismatch at specification: expected oracle v1 sha256:…` |
| `realization_digest` | the `realization_digest` field of the realization IR that `ess specify realization compile --out` writes ([Record a physical realization](../record-realization.md)) |
| `build_digest` | the SHA-256 of the `build-ir.json` file `build compile --out` wrote, for example `sha256sum build-ir.json` |

```shell-session
$ ess specify runtime compile --path runtime.yaml --system examples/oracle-fixture \
    --realization realization.yaml --build-ir build-ir.json --out runtime-ir.json
oracle-runtime — 1 process(es), 1 container role(s), 1 workload(s), compiled to runtime-ir.json
```

Compilation checks that the digests match the supplied documents, that every selected component
is placed in a workload, the replica bounds, and stateful storage. It does not check that a cluster
can provision any of it.

## Project a Helm chart

```shell-session
$ ess generate project helm --ir runtime-ir.json --chart oracle --version 1.0.0 --out chart
5 Helm chart file(s) projected to chart without applying them
$ helm lint chart
1 chart(s) linted, 0 chart(s) failed
```

`--chart` is the chart's name and `--version` its SemVer, which is independent of the runtime's own
release version. The chart holds `Chart.yaml`, `values.yaml`, `values.schema.json` and templates
for the Services and workloads. A workload with volumes becomes a StatefulSet with volume claims
and mounts, with a headless Service; each provided endpoint becomes a Service.

The chart carries no environment. Every image, secret and storage class is an empty slot a
deployment fills in, and the service account defaults to `default`, so a fresh chart passes
`helm lint` on its own:

```yaml
serviceAccount:
  name: default
images:
  app:
    repository: ""
    digest: ""
config: {}
secrets:
  database-password:
    name: ""
    key: "password"
endpoints: {}
workloads:
  oracle:
    replicas: 1
    volumes:
      data:
        class: ""
        size: "1Gi"
```

The output directory also holds `.ess-output/state.json`, which records the files the projection
owns, so running the same command again rewrites them in place
([repeated generation](../generate-artifacts.md#repeated-generation-and-recovery)).

## What these projections do not do

They do not build an image, push anything, install a chart or contact a cluster. A projected chart
says nothing about whether its resources can be created or become ready. Building and publishing
are the explicit executor commands `build execute` and `release publish`; installing is
[`deployment reconcile`](./deploy-an-environment.md#reconcile-under-a-recovery-authority).

Next: [resolve a stack](./resolve-a-stack.md) from the releases these files produce.
