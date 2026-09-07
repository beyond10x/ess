---
title: CLI reference
description: The canonical ESS command, the four areas its first level is made of, and the flat spelling every verb keeps.
---

# CLI reference

`ess` is the canonical command. It exits `0` on success and non-zero on invalid input, unresolved
semantics, unsupported projection, or a failed check.

Its first level is the four areas ESS is built out of, one per crate directory, and `ess --help`
lists exactly those:

| Area | The verbs it holds |
|---|---|
| `ess specify` | `validate`, `compile`, `compose`, `inspect`, `graph`, `realization`, `runtime` |
| `ess generate` | `generate`, `synthesize`, `project`, `schema`, `build`, `component`, `release`, `stack`, `deployment` |
| `ess verify` | `bindings`, `conform`, `diff`, `impact` |
| `ess infra` | `infra`, `import` |

## Flat spellings

Every verb is also spelled flat at the top level, exactly as it was before the areas existed:
`ess validate --path .` is `ess specify validate --path .`, `ess conform run …` is
`ess verify conform run …`, and `ess import openapi …` is `ess infra import openapi …`. A flat
spelling is the same command with the same arguments, and it prints no notice of any kind: when the
command runs, both spellings produce the same stdout, the same stderr and the same exit status, so
a caller that reads the output is unaffected. For a refusal `ess` itself did not write — a missing
required argument, which the argument parser answers — and for `--help`, only the `Usage:` line
differs: it names the path you typed. It is left out of `--help` only so the listing stays the four
areas. Nothing is deprecated and no pinned caller needs changing.

Two verbs share the name of the area they sit in, so their flat spelling is one level shorter
rather than one word: `ess generate --path …` is `ess generate generate --path …`, and
`ess infra diagnose …` is `ess infra infra diagnose …`. `ess generate --help` therefore offers the
verb's options beside the area's subcommands, and the two cannot be written together —
`ess generate --path PATH synthesize` names a specification for a verb that takes one and is
refused with exit 2 rather than run against the current directory.

## `ess specify` — a system, resolved

| Command | Purpose |
|---|---|
| `ess specify validate [--path PATH] [--format text\|yaml\|json]` | Load, resolve, and validate one specification. |
| `ess specify compile [--path PATH] [--out FILE] [--format …]` | Produce canonical typed IR. |
| `ess specify compose --path PATH --service KEY=PATH… [--out FILE] [--client-plan-out FILE] [--client-rust-out DIR]` | Compile selected component surfaces into composition IR, a client plan and a Rust client with byte-buffer transport. |
| `ess specify inspect --path PATH NAME [--format …]` | Resolve and render one declaration. |
| `ess specify graph [--path PATH] [--format dot\|mermaid\|json\|yaml]` | Render the interaction graph. |
| `ess specify realization validate …` | Resolve a physical realization against one exact ESS digest. |
| `ess specify realization compile …` | Emit deterministic `ess-realization-ir/1` or `/2`, matching the authored format. |
| `ess specify realization generate …` | Render a run-mode guide from the resolved realization. |
| `ess specify runtime compile …` | Compile `ess-runtime/1` against exact semantic, realization, and build inputs. |

### Composition clients: selected operations and byte transport

Composition checks an import's exact system, specification version, **compiled-model digest** and
selected component against the supplied compiled ESS model. Commands come from that component's
`accepts`; queries come from views in domains it owns. A reference to a command that exists elsewhere
in the same model is refused with `ReferenceOutsideComponent`; a mismatched imported digest is
refused with `DigestMismatch`. The digest identifies compiled semantics, not raw YAML bytes,
client-plan bytes or the service currently running at an endpoint.

The composition IR and `ess-client-plan/1` carry selected names and imported model identity, not
complete payload definitions or codecs. Named-type traversal follows command inputs, event/error
fields and query row shapes/fields recursively. It does **not** traverse view parameters, so the
listed names do not establish complete query payload closure.

For a concrete example, the repository's [Todo/Usage composition][composition-example] imports two
components from one compiled model. From the ESS repository root, emit its client with:

```sh
fixture=crates/specify/ess-composition/tests/fixtures
mkdir -p target/composition-example
ess specify compose --path "$fixture/compositions/workbench.yaml" \
  --service "todo=$fixture/two-components" \
  --service "usage=$fixture/two-components" \
  --out target/composition-example/composition.json \
  --client-plan-out target/composition-example/client-plan.json \
  --client-rust-out target/composition-example/rust-client
```

The emitted Rust client exposes `service_todo::COMMAND_CREATE_LIST`. `Operation` has private fields
and a private constructor, constraining normal downstream Rust callers to emitted descriptors.
`Client::execute` takes an operation and `&[u8]`, forwards them to `Transport<Authority>`, then returns
the transport's `Vec<u8>` unchanged. It does no payload admission or response decoding.

In the [Todo declaration][composition-todo], `CreateList.details` has type `ListDetails`, whose
`title` field is the String newtype `Title`. The [executable recording-transport example][composition-boundary]
constructs a client with application providers and makes these two calls:

```rust
client.execute(
    service_todo::COMMAND_CREATE_LIST,
    br#"{"details":{"title":"Inbox"}}"#,
);
client.execute(
    service_todo::COMMAND_CREATE_LIST,
    br#"{"details":{"title":7}}"#,
);
```

| Request | Relation to the declared title type | Observed client behavior |
|---|---|---|
| `{"details":{"title":"Inbox"}}` | String title matches the declaration | Exact bytes reach `todo` / `workbench.todo.CreateList`. |
| `{"details":{"title":7}}` | Numeric title conflicts with the String declaration | The same selected operation receives these exact bytes too. |

The example independently checks the selected service identity and operation descriptor, the
separate authority argument, and the unchanged arbitrary binary response `[0, 255, 82, 10]`.
It also checks that a missing endpoint returns `ClientError::MissingEndpoint` before authority
lookup or transport execution, and a transport error returns as `ClientError::Transport`.
The [package test][composition-tests] emits the actual client, compiles it as a separate library,
then compiles and executes the downstream example. Run it with:

```sh
cargo test --locked -p ess-composition \
  generated_rust_client_executes_the_byte_transport_boundary -- --exact --nocapture
```

Endpoint, authority and transport providers are application-owned. Authority is passed separately;
the application must establish its validity. Injection does not verify authority or perform a live
endpoint/model-digest handshake. The client generates no authentication operands, but its opaque
payload can contain application-chosen data, including authentication coordinates: it is not
inspected or sanitized. Payload admission, codecs, response interpretation and live service
compatibility belong to the application and transport contract. Successful composition and byte
forwarding alone do not establish end-to-end typed payload compatibility.

[composition-example]: https://github.com/beyond10x/ess/blob/main/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml
[composition-todo]: https://github.com/beyond10x/ess/blob/main/crates/specify/ess-composition/tests/fixtures/two-components/domains/todo.yaml
[composition-boundary]: https://github.com/beyond10x/ess/blob/main/crates/specify/ess-composition/tests/fixtures/client_boundary.rs
[composition-tests]: https://github.com/beyond10x/ess/blob/main/crates/specify/ess-composition/tests/composition.rs

## `ess generate` — artifacts and explicit delivery executors

| Command | Purpose |
|---|---|
| `ess generate --path PATH --kind docs\|site\|docs-ir\|schema\|openapi\|asyncapi --out PATH` | Generate an explicit deterministic projection. `site` writes HTML and local assets at the output root; `docs-ir` writes `docs-ir/document.json` carrying `ess-docs/1`. Omitting `--kind` combines the five default generators, including HTML under `site/`, and excludes `docs-ir`. |
| `ess generate synthesize …` | Emit supported structural implementation artifacts plus obligations. |
| `ess generate project <adapter> …` | Project typed IR into concrete artifacts. |
| `ess generate schema validate …` | Validate adopter-owned JSON Schema contracts. |
| `ess generate build compile\|graph\|execute …` | Validate and compile `ess-build/1`, render its DAG, or explicitly execute its BuildKit projection. |
| `ess generate component compile …` | Validate a repository-owned component descriptor. |
| `ess generate release verify\|bundle\|verify-bundle\|publish\|fetch …` | Verify and bundle release records or explicitly cross the OCI credential edge. |
| `ess generate stack resolve\|validate …` | Resolve generic product stacks from an offline release catalogue. |
| `ess generate deployment compile\|diff\|reconcile …` | Bind an exact stack lock, compare deployments, or explicitly reconcile the affected Helm releases. |

The [current-source support matrix](../status/where-this-stands.md#support-boundaries) records
projection kinds, adapter directions and their limits separately from the dated release observation.

Run `ess generate synthesize --help` and `ess generate <command> --help` for target-specific
arguments.

Omitting `--kind` generates every projection. Omitting `--out` lists or serializes artifacts
without writing them. The repository-only `cargo xtask generate` command reconciles the committed
`generated/` projection tree; `cargo xtask generate --check` compares it without writing.

## Component delivery

| Command | Purpose |
|---|---|
| `ess generate component compile --path FILE [--out FILE]` | Validate a repository-owned component descriptor. |
| `ess generate build execute --path FILE --projection-out DIR …` | Compile and retain the BuildKit projection, then invoke Docker Buildx Bake. |
| `ess generate release bundle …` | Verify runtime and chart releases and write one canonical OCI payload. |
| `ess generate release publish --path FILE --to OCI_TAG` | Publish a verified bundle and print its immutable OCI manifest digest. |
| `ess generate release fetch --from OCI_REF@sha256:… --cache DIR` | Fetch a digest-pinned bundle, revalidate it, and cache canonical bytes. |
| `ess generate deployment reconcile --path FILE --current FILE --cache DIR` | Apply only added or changed Helm releases in rollout order. |

The compiler and projection operations stay offline. The commands that say `execute`, `publish`,
`fetch`, or `reconcile` are explicit credential edges and invoke installed Docker, ORAS, or Helm
clients. Reconciliation refuses removals unless `--allow-removals` is supplied. Use `--dry-run` to
print the affected set without contacting external systems. See
[Independent component delivery](../concepts/component-delivery.md).

### Adopter-owned schema contracts

These commands are available in the current source. Their presence does not identify which remote
release contains a particular change; see the [dated release observation](../status/where-this-stands.md).

| Command | Purpose |
|---|---|
| `ess generate schema validate PATH… --schemas DIR [--format text\|yaml\|json]` | Validate JSON instances against the offline `*.schema.json` registry they select by stable `schema` identity. |
| `ess generate schema typescript SCHEMA_ID --root TYPE --schemas DIR [--out FILE] [--check]` | Project deterministic structural TypeScript from one authoritative JSON Schema. |
| `ess generate schema import-bundle --path FILE --component NAME… --dialect draft-2020-12 [--out FILE]` | Retain and qualify a selected structural component closure without inventing an OpenAPI service. |
| `ess generate schema import-document --path FILE --root NAME [--definition NAME…] --dialect draft-2020-12 [--out FILE]` | Retain a JSON Schema document root and local definition closure in a replay-checked `/2` bundle. |
| `ess generate schema project-bundle --bundle FILE --root NAME --schema-id URI [--out FILE]` | Revalidate an import and emit one root's standalone JSON Schema and source qualification. |
| `ess generate schema validate-bundle --bundle FILE --root NAME INSTANCE…` | Validate unmodified instances against one explicitly selected component. |
| `ess generate schema types-bundle --bundle FILE --root NAME… --target typescript\|rust\|go [--package NAME] [--module PATH] --out DIR` | Emit root-selected data libraries, qualified source and target accounting. Rust/Go require package identity; Go also requires module identity. Not an application decoder. |
| `ess generate schema normalize-check --recipe FILE [--bundle FILE]… [--model PATH]… [--out FILE]` | Check bundles and compiled model selections, check every normalization branch, and emit the canonical recipe. At least one source is required. Model roots require version 3. |
| `ess generate schema normalize-generate --recipe FILE [--bundle FILE]… [--model PATH]… --target rust\|go\|typescript --package NAME [--module PATH] --out DIR [--check]` | Emit a source-pinned normalization library or check planned file bytes without writing. At least one source is required. Go requires `--module`; Rust and TypeScript refuse it. Protect model input trees. TypeScript emits a standalone JSON-text runtime with a fixed checked schema profile. |
| `ess generate schema normalize-run --recipe FILE [--bundle FILE]… [--model PATH]… --branch NAME --input FILE [--out FILE]` | Execute an explicit branch with stage input/output validation; emit only a complete result. At least one source is required. Refuse duplicate input keys, numeric precision loss and input/output aliases. |
| `ess generate types --path SPEC (--root QUALIFIED_NAME… \| --all-types) --target typescript\|rust\|go [--package NAME] [--module PATH] --out DIR` | Realize checked ESS model types using the shared wire mapping and data targets. Retains model schema selection and typed provenance; output must be outside the specification tree. |

These operations are offline. Schema identity comes from `$id`; filenames only locate documents.
`--check` compares an existing generated module byte for byte without rewriting it.

## `ess verify` — held to what was declared

| Command | Purpose |
|---|---|
| `ess verify conform synthesize …` | Generate the semantic suite required by a specification. |
| `ess verify conform run …` | Execute a suite against a supported target and emit a standalone report. |
| `ess verify diff --from PATH --to PATH [--format text\|json]` | Compare two revisions semantically. |
| `ess verify impact --from PATH --to PATH [--suite PATH] [--format …]` | Name invalidated scenarios and generated artifacts. |
| `ess verify bindings --spec PATH --realization FILE --bindings FILE (--infra FILE \| --live --observation-out FILE) [--format text\|json] [--markdown-out FILE]` | Compare an exact implementation selection with scoped workload templates; exit 0 satisfied, 1 violated/refused, 2 unknown. |

See [observed implementation bindings](../guides/check-infrastructure.md#connect-implementation-selections-to-observed-workloads)
for the authored contract, source-preview requirement and evidence limits.

Run `ess verify conform <command> --help` for target-specific arguments.

## `ess infra` — an observed cluster, and what reads one

| Command | Direction |
|---|---|
| `ess infra import openapi --path FILE [--out FILE] …` | OpenAPI 3.1 subset → `ess-openapi-import/1` with retained source, SHA-256 and durable accounting. `--format` selects terminal presentation; `--out` always writes the canonical import envelope. |
| `ess infra import kubernetes …` | sanitized bundle or explicitly selected live cluster → infrastructure IR. |
| `ess generate project openapi (--ir FILE \| --path SPEC) …` | Checked import envelope or native ESS specification → OpenAPI. `--ir` refuses semantic gaps, unresolved references or legacy interface-only input before output; reimport original OpenAPI to replace legacy files. |
| `ess generate project kubernetes …` | infrastructure intent and observation → manifests and obligations. |

The commands under `ess generate project` write artifacts only. They do not call `kubectl`, apply a
manifest, or mutate a target.

`ess infra infra` contains `diagnose`, `graph`, and `diff` operations over sanitized infrastructure
IR — the same three as `ess infra diagnose`, `ess infra graph` and `ess infra diff`, which are their
flat spellings. Live or bundle scanning is under `ess infra import kubernetes`; manifest generation
is under `ess generate project kubernetes`. Run `ess infra infra --help` for their current
arguments.
