# Executable System Specification

ESS is a standalone Rust workspace for describing systems as typed data, compiling those
descriptions into deterministic IR, projecting concrete artifacts, and checking implementations or
infrastructure against what was declared.

The canonical command is `ess`, and its first level is the four areas the crates are grouped into:
`specify`, `generate`, `verify`, `infra`.

```console
cargo run --bin ess -- specify validate --path examples/billing
cargo run --bin ess -- specify compile --path examples/billing --format json
cargo run --bin ess -- specify realization validate --path realization.yaml --spec path/to/spec
cargo run --bin ess -- specify realization generate --path realization.yaml --spec path/to/spec --out running-modes.md
cargo run --bin ess -- specify runtime compile --path ess/runtime.yaml --system ess/system --realization ess/realization.yaml --build-ir generated/ess/build.json --out generated/ess/runtime.json
cargo run --bin ess -- generate --path examples/billing --kind docs --out generated
cargo run --bin ess -- generate --path examples/billing --kind site --out generated
cargo run --bin ess -- generate --path examples/billing --kind openapi --out generated
cargo run --bin ess -- generate build compile --path ess/build.yaml --out generated/ess/build.json
cargo run --bin ess -- generate build graph --path ess/build.yaml --out generated/ess/build.mmd
cargo run --bin ess -- generate project buildkit --ir generated/ess/build.json --out generated/build
cargo run --bin ess -- generate project helm --ir generated/ess/runtime.json --chart example --version 1.0.0 --out generated/chart
cargo run --bin ess -- generate project openapi --ir interface.json --out normalized-api.yaml
cargo run --bin ess -- generate stack resolve --path ess/stack.yaml --catalog releases.json --out stack.lock.json
cargo run --bin ess -- generate deployment compile --path environment.yaml --stack-lock stack.lock.json --out deployment.json
cargo run --bin ess -- generate schema validate instances --schemas schemas
cargo run --bin ess -- generate schema typescript urn:example:registry:1 --root Registry --schemas schemas
cargo run --bin ess -- verify conform synthesize --path examples/billing --out suite.json
cargo run --bin ess -- verify conform run --suite suite.json --target billing --report-out report.json
cargo run --bin ess -- infra import openapi --path api.yaml --out interface.json
cargo run --bin ess -- infra infra diagnose --path observation.json
```

Every verb is also spelled flat at the top level, exactly as it was before the areas existed:
`ess validate --path examples/billing`, `ess conform run …`, `ess schema validate …`. A flat
spelling runs the same command and prints the same bytes on both streams with the same exit status,
with no notice of any kind; it is left out of `--help` so the listing stays the four areas. Nothing
is deprecated, and a pinned caller needs no change.

## Install the command

Every version release publishes checksum-pinned archives for Linux and macOS on x86-64 and ARM64.
Pick the target for your machine from the [release page](https://github.com/beyond10x/ess/releases):

| Machine | Target |
|---|---|
| Linux x86-64 | `x86_64-unknown-linux-gnu` |
| Linux ARM64 | `aarch64-unknown-linux-gnu` |
| macOS Intel | `x86_64-apple-darwin` |
| macOS Apple Silicon | `aarch64-apple-darwin` |

For example, to install the current release for Apple Silicon in the current directory:

```console
version=0.9.1
target=aarch64-apple-darwin
archive="ess-${version}-${target}.tar.gz"
base="https://github.com/beyond10x/ess/releases/download/${version}"
curl --fail --location --remote-name "${base}/${archive}"
curl --fail --location --remote-name "${base}/SHA256SUMS"
grep -F "  ${archive}" SHA256SUMS | shasum -a 256 --check
tar -xzf "${archive}"
"./ess-${version}-${target}/ess" --version
```

If none of the four targets matches your machine, build the locked source checkout instead:

```console
cargo build --locked --release --bin ess
./target/release/ess --version
```

Adapters use one explicit contract:

- `ess infra import <adapter>` reads a concrete source and reports coverage, diagnostics, and
  unresolved references while producing typed IR where the adapter supports the source.
- `ess generate project <adapter>` writes reviewable artifacts, obligations, and refusals. It never
  applies infrastructure or mutates an external system.
- Kubernetes import accepts a sanitized bundle or performs a live scan at the credential edge.
  Secret values are digested before any serialization or filesystem write.
- OpenAPI import produces `ess-service-interface/1`: typed service, operation, JSON-message, local
  reference, and interface-type structures plus coverage gaps and unresolved references. Projection
  accepts that IR or a native ESS specification. The supported subset has an IR → OpenAPI → IR
  semantic round-trip guarantee; unsupported protocol features are normalized, reported, or refused.

`EssIr`, `InfraIr`, and the adapter-specific service-interface representation intentionally remain
distinct. The compiler IRs keep compiler-minted handles, total lookups, ordered collections, and
deterministic serialization. New typed constructs are added only when an importer or projector
establishes their semantics; there is no generic property bag or facet registry.

`ess-realization/1` describes one physical implementation of an exact `EssIr` without changing the
semantic system. It binds the ESS digest to immutable implementation artifacts and typed
entrypoints, including local CLIs, loopback browser surfaces, model-backed agent loops, or
approval-required hosted interfaces. `ess specify realization compile` emits
`ess-realization-ir/1`; the
Markdown generator turns the same IR into a drift-checkable run-mode guide.

Build and deployment use another explicit lowering chain. `ess-build/1` is a typed, content-addressed
transformation DAG which projects to `BuildKit` inputs and a deterministic Mermaid graph.
Compiler APIs stay pure; explicit CLI executor commands perform credentialed BuildKit, ORAS, and
Helm operations at the edge. `ess-realization/1`
binds exact semantic components to immutable implementation artifacts and entrypoints.
`ess-runtime/1` maps those components to processes, container roles, and workloads. Executor-produced
`ess-release/1` manifests bind those inputs to immutable artifacts and evidence. Generic
`ess-stack/1` constraints resolve offline to an exact `ess-stack-lock/1`; a private
`ess-environment/1` then lowers to `ess-deployment/1`, one independently deployable Helm release per
system. Chart and runtime releases are selected and pinned separately. Secret bytes have no field in
any of these documents.

`ess-component/1` joins those documents as the release boundary owned by an implementation
repository. Runtime and chart manifests are bundled as canonical `ess-release-bundle/1`, published
and fetched through OCI by digest, then revalidated before entering the local cache. Runtime models
can expose named Services and persistent volumes, and `ess deployment reconcile` applies only the
release units changed from an optional previous deployment IR. See the
[independent component delivery concept](website/docs/concepts/component-delivery.md).

A construct is a design page before it is code. The binding designs live in `docs/design/`; entity
relations shipped in `0.5.0` and their
[design](docs/design/ess-entity-relations-design-v0.1.md) records how an entity declares what it owns
and references, what that refuses, and how one extension key (`x-ess-relation`) carries a relation
into JSON Schema, OpenAPI and Rust.

`ess generate --kind docs` emits repository Markdown and Mermaid. The `site` projection, introduced in
`0.4.0`, adds frontmatter and `sidebar.json` to the same pages so a static site generator can consume
them. It does not accept prose as its specification and does not emit HTML, a theme, or hosting.

## The crate tree

Crates sit under the area they serve, and the command surface says the same thing: one area of
`ess` per directory here. The name of a crate is its identity and no move changed one; the
directory says which half of the pipeline it belongs to.

- **`crates/specify/`** — `ess-primitives`, `ess-domain`, `ess-compiler`, `ess-composition`,
  `ess-realization`: an authored system becomes a validated, resolved IR.
- **`crates/generate/`** — `ess-gen`, `ess-synth`, `ess-openapi`, `schema-contract`,
  `ess-deployment`: that IR becomes artifacts, and nothing here applies one to a running system.
- **`crates/verify/`** — `ess-conformance`, `ess-diff`: an implementation held to the
  specification, and one revision of a specification against another.
- **`crates/infra/`** — `infra-domain`, `infra-compiler`, `infra-analyze`, `infra-spec`,
  `infra-project`, `ess-kubernetes`: the observed cluster, a separate bounded context whose only
  dependency on the rest is `ess-primitives`.
- **`crates/edge/`** — `ess-cli`, `ess-xtask`: the `ess` binary an adopter runs, and this
  repository's own tooling.

Run the complete offline gate with:

```console
task check
```

The historical ESS, infrastructure, schema, examples, suites, and generated artifacts were
extracted with filtered Git history. The Kubernetes adapter additionally carries the imported
history of the former standalone scanner.

<!-- b10x-docs:start -->
## Documentation

[ESS documentation](https://beyond10x.github.io/docs/ess/) · [Start](https://beyond10x.github.io/) · [Ecosystem](https://beyond10x.github.io/ecosystem/) · [Impact](https://beyond10x.github.io/changes/) · [Releases](https://beyond10x.github.io/releases/)
<!-- b10x-docs:end -->
