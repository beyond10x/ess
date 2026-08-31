# ess-kubernetes

Scans a Kubernetes cluster into an `infra-observation/1` bundle. **This repo holds the kubeconfig
so the analysis never has to.**

The problem it removes: to reason about a cluster you need its facts, and the obvious way to get
them hands cluster credentials to the thing doing the reasoning. ess-kubernetes is the one program
that touches the cluster. It writes a file. Everything downstream reads that file and holds no
credential.

```console
ess-kubernetes contexts
ess-kubernetes scan --context k3d-example --out cluster.observation.json
```

## Where it sits

| direction | what |
|---|---|
| produces bundles for | [aep](https://github.com/beyond10x/aep) — the pure toolchain that consumes `infra-observation/1` |
| shells out to | `kubectl`, using the operator's kubeconfig |
| mapped in | [atlas](https://github.com/beyond10x/atlas) |

The split is deliberate and is stated on the other side, in `docs/VISION.md` of
aep under "What this is deliberately not". This repository exists so that
boundary can stay.

## Status

**A working v0 spike, and small on purpose.** Version `0.1.0`, no git tag, three commits, one
source file. The whole committed surface is:

| tracked file | what |
|---|---|
| `src/main.rs` | the entire program — 201 lines: two subcommands, the kind list, the secret hasher |
| `Cargo.toml`, `Cargo.lock` | four dependencies: `clap`, `serde`, `serde_json`, `sha2` |
| `README.md` | this file |

There is no `docs/`, no test and no CI. A `scripts/` directory exists on disk but is **not
committed** — it holds bot-identity helpers, not anything a scan needs. The working agreement is
[`AGENTS.md`](AGENTS.md).

Applying anything to a cluster is not in this tool. When it arrives it will be behind an explicit
flag with a server-side dry-run first.

## Build, test, run

There is no task runner, no gate script and no test — this repo defines none. The commands are the
plain cargo ones:

```console
cargo build --release
cargo run -- contexts
cargo run -- scan --out cluster.observation.json
```

Edition 2021. `kubectl` must be on `PATH` and the kubeconfig must already work — a scan is a
sequence of `kubectl get` calls, and a failure names the invocation that failed.

## What a scan collects

Seventeen API kinds, in the order they are collected: `namespaces`, `nodes`, `deployments`,
`statefulsets`, `daemonsets`, `replicasets`, `jobs`, `cronjobs`, `pods`, `services`, `ingresses`,
`configmaps`, `secrets`, `serviceaccounts`, `persistentvolumeclaims`, `poddisruptionbudgets`,
`horizontalpodautoscalers`.

Enough to map workloads, their wiring (services, ingress, config) and their identity (service
accounts), plus the runtime facts (pods, nodes) diagnosis reads. Extending that list is a format
change and belongs in a commit that says so. Per-kind progress goes to stderr while a scan runs.

## Secrets

**Secret values never touch disk.** Every `data` and `stringData` value is replaced with its
SHA-256 and its length before the bundle is written, and the `last-applied-configuration`
annotation — which can carry values — is dropped entirely.

The *shape* survives: which secrets exist, which keys they carry. That is what dependency mapping
needs, and the values are unrecoverable. This is the reason the tool exists as a program rather
than as a `kubectl | jq` pipeline somebody edits under pressure.

## Why `kubectl` and not a Kubernetes client

Credentials, exec plugins and API skew stay `kubectl`'s problem. Linking a client would move all
three into this binary in exchange for nothing v0 needs.
