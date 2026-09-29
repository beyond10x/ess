---
title: Resolve a stack
sidebar_position: 2
description: Turn a product's release constraints into an exact stack lock from an offline release catalogue, and check that a stack resolves.
---

# Resolve a stack

A product composed of several components does not pin their releases by hand. It states
constraints (`ess-stack/1`): which system, which semantic major, which runtime and chart release
ranges. `ess generate stack resolve` picks exact releases from a release catalogue you supply and
writes an `ess-stack-lock/1`. Nothing is fetched: the catalogue is the only input.

The commands on this page continue the example of
[Project a build and a Helm chart](./build-and-chart.md).

## The releases

A release manifest (`ess-release/1`) names one release unit, its version, and the exact build,
runtime and semantic digests it was produced from, with its artifacts and four evidence
references (provenance, SBOM, signature, conformance). Your release process writes it once the
artifacts it names are built and published; the `release-component` action in this repository
does so after `build execute` and `release publish-conformance`
([Independent component delivery](../../concepts/component-delivery.md#release-a-component-with-the-action)).
`release verify` checks a manifest against the build and runtime IR:

```shell-session
$ ess generate release verify --path release-runtime.json \
    --build-ir build-ir.json --runtime-ir runtime-ir.json
oracle-runtime 1.2.3 — 1 immutable artifact(s), consistency checked
release consistency: checked
conformance: not assessed
attachment binding: unverified
producer origin: unverified
artifact execution: unverified
signature verification: unsupported
```

The last four lines are always printed: a consistent manifest does not show that its evidence is
bound to the artifact, who produced it, that the artifact ran, or that a signature is valid
([What release evidence establishes](../../concepts/component-delivery.md#what-release-evidence-establishes)).

A component publishes two release units: the runtime (an OCI image) and the chart (a Helm chart).
Each has its own version.

## The catalogue

`ess-release-catalog/1` lists candidate releases. Each entry holds the semantic major, the
surfaces the release serves, the release manifest, and the runtime IR it was built for:

```yaml
format: ess-release-catalog/1
releases:
  - semantic_version: v1
    surfaces: [oracle.order.PlaceOrder]
    release: { … the oracle-runtime 1.2.3 manifest … }
    runtime: { … runtime-ir.json … }
  - semantic_version: v1
    surfaces: [oracle.order.PlaceOrder]
    release: { … the oracle-chart 4.5.6 manifest … }
    runtime: { … runtime-ir.json … }
```

You assemble the catalogue; ESS does not query a registry to build one.

## The stack

```yaml
format: ess-stack/1
stack: oracle-stack
composition_digest: sha256:2222222222222222222222222222222222222222222222222222222222222222
systems:
  - service: oracle
    system: oracle
    semantic_version: v1
    runtime_release: ^1.0
    chart_release: ^4.0
    surfaces: [oracle.order.PlaceOrder]
```

Each entry under `systems:` is a service of the stack. It may also list `depends_on:` (other
services that roll out first). `external_systems:` declares systems the stack uses but does not
deploy from the catalogue, with the contract, endpoints, audiences and configuration they must
provide.

## Check and resolve

```shell-session
$ ess generate stack validate --path stack.yaml --catalog catalog.json
oracle-stack — 1 exact release(s), 0 external system(s), resolved
$ ess generate stack resolve --path stack.yaml --catalog catalog.json --out stack-lock.json
oracle-stack — 1 exact release(s), 0 external system(s), resolved to stack-lock.json
```

`validate` answers whether the stack resolves completely and writes nothing; `resolve` writes the
lock. Both exit 1 when a constraint has no match:

```shell-session
$ ess generate stack validate --path stack-unmet.yaml --catalog catalog.json
lowering was refused:
[unsatisfied_constraint:Composition] no release of oracle satisfies semantic v1, release ^2.0, and required surfaces
```

For each service the resolver takes the highest runtime release that matches the system, semantic
major, runtime range and surfaces and holds an OCI image. It then takes the highest chart release
in the chart range that holds exactly one Helm chart and was made from the same runtime and
semantic digests. Equal versions are ordered by manifest digest, so the result is deterministic.
It refuses a catalogue runtime that does not match its manifest, a rollout dependency on an
undeclared service, and a dependency cycle.

## The lock

`ess-stack-lock/1` records, for each service, the exact runtime and chart release, their manifest
digests, their artifacts, and the runtime's configuration, secret and endpoint slots. Its own
`stack_digest` field is the digest of the stack document it was resolved from.

An environment binds to a lock by the lock file's own digest, which is a different value: the
SHA-256 of the `stack-lock.json` file `resolve --out` wrote (for example `sha256sum
stack-lock.json`). [Deploy an environment](./deploy-an-environment.md) uses it.
