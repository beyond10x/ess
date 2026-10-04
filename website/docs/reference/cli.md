---
title: CLI reference
sidebar_position: 1
description: The canonical ESS command, the five areas its first level is made of, and the flat spelling every verb keeps.
---

# CLI reference

`ess` is the canonical command. It exits `0` on success and non-zero on invalid input, unresolved
semantics, unsupported projection, or a failed check.

Its first level is the five areas ESS is built out of, one per crate directory, and `ess --help`
lists exactly those: [`ess specify`](#ess-specify), [`ess generate`](#ess-generate),
[`ess verify`](#ess-verify), [`ess infra`](#ess-infra) and [`ess ui`](#ess-ui). The
[command reference](#command-reference) at the end of this page lists every command under them,
with every argument, its default and its help text. It is generated from the command definition
`ess` itself parses with, so it names exactly what the shipped command accepts.
What each error code and refusal name `ess` prints means, and how to repair it, is in
[Diagnostics](diagnostics.md).

The agent guidance for these commands is the `ess` plugin in
[`beyond10x/agentplugins`](https://github.com/beyond10x/agentplugins); `ess` itself carries none.

## Flat spellings

Existing verbs retain their flat spellings at the top level, exactly as before the areas existed:
`ess validate --path .` is `ess specify validate --path .`, `ess conform run …` is
`ess verify conform run …`, and `ess import openapi …` is `ess infra import openapi …`. A flat
spelling is the same command with the same arguments, and it prints no notice of any kind: when the
command runs, both spellings produce the same stdout, the same stderr and the same exit status, so
a caller that reads the output is unaffected. For a refusal `ess` itself did not write — a missing
required argument, which the argument parser answers — and for `--help`, only the `Usage:` line
differs: it names the path you typed. It is left out of `--help` only so the listing stays the five
areas. Nothing is deprecated and no pinned caller needs changing.

The additive `ess specify cli` and `ess generate cli` routes are available only
under their areas. Their shared leaf name has two distinct purposes, so neither
claims a flat `ess cli` spelling.

Two verbs share the name of the area they sit in, so their flat spelling is one level shorter
rather than one word: `ess generate --path …` is `ess generate generate --path …`, and
`ess infra diagnose …` is `ess infra infra diagnose …`. `ess generate --help` therefore offers the
verb's options beside the area's subcommands, and the two cannot be written together —
`ess generate --path PATH synthesize` names a specification for a verb that takes one and is
refused with exit 2 rather than run against the current directory.

## Directory input selection

Every command that reads a specification takes `--path` (or `--spec`, `--system`, `--from`/`--to`
where it reads two). The path is one of:

| `--path` names | What is read |
|---|---|
| a file | that file alone, whatever its extension; no parent directory is consulted |
| a directory with an `ess-inputs.yaml` | exactly the files its `specification:` list names |
| a directory with a `system.yaml` and no `ess-inputs.yaml` | every lowercase `.yaml` and `.yml` file below it |

[`ess-inputs.yaml`](formats.md#directory-input-configuration), introduced in 0.21.0, lets a
specification share its directory with generated output and authored scenarios: only the listed
files are opened, in sorted order, and nothing else in the directory is scanned. Input selection reads it only in
the directory you name; only the release pin below is looked up in parent directories. A malformed manifest is
refused rather than ignored. `--scenarios` reads the same manifest's `scenarios:` list, and
`ess specify validate` also compiles a nonempty `scenarios:` list and reports its `ESS-AUTHOR-*`
refusals.

`format: ess-inputs/2` can pin the `ess` release with `requires:`. An older `ess` refuses and a
newer one warns; the global `--strict-requires`, accepted in any position, refuses instead. See
[the release a project runs](#the-release-a-project-runs).

A manifest selects inputs; it does not permit writing generated output into the specification's
directory, and the commands that refuse to write inside their input still refuse. Inputs that are
not specifications — a committed suite given to `ess verify conform run --suite`, an OpenAPI `--ir`,
build and runtime IR, infrastructure observations — are read as given and never through a manifest.

## `ess specify` — a system, resolved

The `ess specify` commands load, resolve and validate a specification and render what it says:
`validate` also compiles the authored scenarios its `ess-inputs.yaml` lists, and `compose` compiles
selected component surfaces into composition IR, a client plan and a Rust client with byte-buffer
transport. Their arguments are listed under [`ess specify`](#ess-specify) in the command reference.

`ess specify toolchain install X.Y.Z` fetches only from `https://` or a local directory, and its
`--pin` writes `requires: ess X.Y.Z` (0.34.0 or later). `ess specify toolchain which` names the
reason for its answer: `ESS_TOOLCHAIN`, the pin, or this `ess`.

### The release a project runs

When the nearest `ess-inputs.yaml` above the working directory carries an exact
`requires: ess X.Y.Z` naming another release, or `ESS_TOOLCHAIN=X.Y.Z` is set, any `ess` runs that
release from its cache (`$XDG_CACHE_HOME/ess/toolchains/X.Y.Z/ess`, else
`~/.cache/ess/toolchains/`) with the same arguments and environment, installing it first when it is
not cached. A minor line `ess X.Y`, or no pin, runs the `ess` you called. Every delegated command
prints one note on stderr naming the dispatcher, the release it delegated to and why
(`ESS_TOOLCHAIN_QUIET=1` silences it); stdout is the delegated release's alone, and `ess --version`
prints both releases' lines. A release that cannot be installed is refused,
naming the newest cached one. This walk upwards reads only the pin; input selection still reads
only the manifest of the directory it is given. The `toolchain` commands always run in the `ess`
you called.

### CLI presentation bindings

An independently authored `ess-cli/1` document selects typed local actions,
service-forwarded operations or dynamic schema-selected calls. It declares CLI
paths, finite aliases, argument sources and process context without moving
operation ownership or changing the ESS semantic format.

```sh
ess specify cli --path model --binding cli.yaml --format json
ess generate cli --path model --binding cli.yaml --out generated/cli
ess generate cli --path model --binding cli.yaml --out generated/cli --check
```

The generator emits a standalone Rust/Clap package, resolved binding, help and
completion artifacts. The generated adapter validates input and handler output
types; dynamic schema validation and application actions remain explicit runtime
obligations. A callable's `invalid_input: <code>` names one of its declared errors
as the answer, with `{}` data, for every invalid input in place of `cli_input` and
`cli_dynamic_input`. Its default handler reports unavailable. It does not install an
adapter, store credentials or contact a provider merely because parsing succeeds.

Unsupported model projections and invalid bindings refuse before output mutation.
Output must stay outside model inputs. Generation uses the existing output
ownership protocol with family `cli-binding`. Regeneration may repair edited files
already owned by that family; unowned destinations refuse. `--check` detects drift
without writing or recovery. See [repeated generation and recovery](../guides/generate-artifacts.md#repeated-generation-and-recovery).
`--check` compares generated bytes without replacing them. See the
[binding design](https://github.com/beyond10x/ess/blob/main/docs/design/cli-presentation-binding.md)
for the complete input and handler contracts. CLI presentation bindings were
introduced in 0.21.0.

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
  --ownership-root target/composition-example \
  --out target/composition-example/composition.json \
  --client-plan-out target/composition-example/client-plan.json \
  --client-rust-out target/composition-example/rust-client
```

Writing composition outputs requires `--ownership-root` enclosing every selected file and client
directory. These outputs share one owner: omitting a previously selected output on a later run
retires its owned files. Listing composition without output destinations remains nonwriting.

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

The `ess generate` commands, with every argument, are listed under
[`ess generate`](#ess-generate) in the command reference; `--help` on any of them prints the same.

`ess generate --kind` selects one deterministic projection. `site` writes HTML and local assets at
the output root; `docs-ir` writes `docs-ir/document.json` carrying `ess-docs/1`. The
`ess generate output` commands are also available as `ess output …`.

The [current-source support matrix](../status/where-this-stands.md#support-boundaries) records
projection kinds, adapter directions and their limits separately from the dated release observation.

Omitting `--kind` combines the five default generators, including HTML under `site/`, and excludes
`docs-ir`. Omitting `--out` lists or serializes artifacts
without writing them.

`openapi` and `asyncapi` write one document per component, so a domain no component `owns` is in
neither, and a specification without components projects to `0 artifact(s)`. That is legal, and
`--kind openapi`, `--kind asyncapi`, no `--kind` and `ess generate project openapi --path …` each
print `note: no component owns <domain>; declare it in components.yaml` on stderr for every such
domain, and exit 0. `--strict` makes the same condition a refusal: the line reads `refused:`,
nothing is written, and the exit is 1. The other kinds do not read components and print no note. The repository-only `cargo xtask generate` command reconciles the committed
`generated/` projection tree; `cargo xtask generate --check` compares it without writing.

Generated tree outputs use their output root as the ownership root; a standalone generated file
uses its parent. Existing unowned destinations refuse. See [repeated generation and recovery](../guides/generate-artifacts.md#repeated-generation-and-recovery)
for adoption, stale-file retirement, and the filesystem contract. Output-management commands were introduced in 0.21.0.

## Component delivery

A component is delivered by [`ess generate component compile`](#ess-generate-component-compile),
[`ess generate build execute`](#ess-generate-build-execute), which compiles and retains the
BuildKit projection and then invokes Docker Buildx Bake, the `ess generate release` commands and
[`ess generate deployment reconcile`](#ess-generate-deployment-reconcile). `release publish`
prints the published OCI manifest digest; `release fetch` takes a digest-pinned reference,
revalidates the bundle and caches its canonical bytes.

`deployment reconcile` applies only added or changed Helm releases, in rollout order. `--current`
is an admitted baseline desired deployment, not a record of what was applied. `--authority` names
one entry of the protected recovery registry and is **required** for execution: without it the
command refuses before any external call, cache write or recovery write. `--dry-run` remains a
local unverified preview and needs no authority.

The seven release routes also have identical flat `ess release …` aliases. Existing canonical
JSON/YAML streams and output files keep their bytes. Text success describes consistency; fetch
additionally describes OCI content identity. Qualification diagnostics go to stderr. Both
publishers end stdout with `<destination> — published at <OCI manifest digest>` so the final token
remains the published manifest digest. `check-conformance` succeeds with empty stdout.

```sh
ess generate release check-conformance \
  --spec ess/model --report evidence/report.json \
  --expected-suite-input policy/expected-input.json \
  --component-ir ess/compiled/component.ir.json \
  --build-ir ess/compiled/build.ir.json --runtime-ir ess/compiled/runtime.ir.json
```

Replace `check-conformance` with `publish-conformance` and add `--to repository:tag` to qualify
and upload the exact original report bytes through ORAS in the same process. Digest destinations
are refused. Use `--expected-suite` instead of `--expected-suite-input` for an original unfiltered
suite/5. The carrier route requires every original parent. A standalone report/2 and explicit
model are required; report/1, generic logs and detailed run/2 documents refuse. Complete nonempty
all-pass selection, full model/contract identity and canonical deployment context must agree.

Existing bundle `publish` accepts the all-or-none group `--spec`, `--report` and exactly one of
`--expected-suite` / `--expected-suite-input`. It obtains deployment context from its admitted
canonical `--path` bundle. Omitting the group preserves consistency-only publication. Either
optional raw-byte pin, `--report-sha256 sha256:…` or `--expected-input-sha256 sha256:…`, requires
the qualification group. Pins cover the complete original files, not an OCI manifest or the
carrier's selected inner-suite digest. Invalid CLI groups are usage errors; admission or positive
qualification failures exit 1 before publication effects.

Success qualifies only the supplied exact declared selection. Attachment binding, producer origin
and artifact execution remain **unverified**; signature verification remains **unsupported**.
See [component delivery](../concepts/component-delivery.md#release-a-component-with-the-action)
for the breaking action inputs and old/new action/ESS compatibility.

The compiler and projection operations stay offline. The commands that say `execute`, `publish`,
`fetch`, or `reconcile` are explicit credential edges and invoke installed Docker, ORAS, or Helm
clients. Reconciliation refuses removals unless `--allow-removals` is supplied. Use `--dry-run` to
print the affected set without contacting external systems. See
[Independent component delivery](../concepts/component-delivery.md).

### Adopter-owned schema contracts

These commands are available in the current source. Their presence does not identify which remote
release contains a particular change; see the [dated release observation](../status/where-this-stands.md).

The `ess generate schema` commands and [`ess generate types`](#ess-generate-types) are listed in
the command reference. In addition to their help text:

- `import-document` retains the document root and its local definition closure in a
  replay-checked `/2` bundle; `project-bundle` also emits the root's source qualification.
- `types-bundle` emits data libraries, not an application decoder.
- The `normalize-*` commands need at least one `--bundle` or `--model` source, and model roots
  require version 3. With `normalize-generate`, protect model input trees; its TypeScript target
  emits a standalone JSON-text runtime with a fixed checked schema profile.
  `normalize-run` emits only a complete result and refuses duplicate input keys, numeric precision
  loss and input/output aliases.
- `ess generate types` realizes checked ESS model types through the shared wire mapping and data
  targets, retaining model schema selection and typed provenance; its output must be outside the
  specification tree.

These operations are offline. Schema identity comes from `$id`; filenames only locate documents.
`--check` compares an existing generated module byte for byte without rewriting it.

## `ess verify` — held to what was declared

The `ess verify` commands, with every argument and, where they have them, their exit statuses, are
listed under [`ess verify`](#ess-verify) in the command reference. In addition to their help text:

- [`ess verify conform mutate`](#ess-verify-conform-mutate) `--emit DIR` writes `mutant.json` (the
  mutant's class, site and change) and `ir.json` beside each mutant's `suite.json`, and its
  `ess-mutation-manifest/3` names each suite's synthesis refusals and each mutant's
  `unsatisfiable_guard`. `--collect` scores `ess-conformance-report/1` or `/2` reports into
  `ess-mutation-report/3`; an `ess-mutation-manifest/2` or `/1` emission still collects, naming no
  dead guard, and a `/1` one judges gained refusals by count.
- [`ess verify conform web`](#ess-verify-conform-web) with `--history` exits 0 once the page is
  rendered.
- [`ess verify bindings`](#ess-verify-bindings) exits 0 when the bindings are satisfied, 1 when
  they are violated or refused, and 2 when the result is unknown.

See [observed implementation bindings](../guides/check-infrastructure.md#connect-implementation-selections-to-observed-workloads)
for the authored contract and evidence limits.

## `ess infra` — an observed cluster, and what reads one

Sources come in through `ess infra import` and leave through
`ess generate project`:

- `ess infra import openapi` reads an OpenAPI 3.0 or 3.1 subset into `ess-openapi-import/1`, with
  the retained source, its SHA-256 and durable accounting. `--format` selects the terminal
  presentation; `--out` always writes the canonical import envelope.
- `ess infra import kubernetes` turns a sanitized bundle, or an explicitly selected live cluster,
  into infrastructure IR.
- [`ess generate project openapi`](#ess-generate-project-openapi) projects a checked import
  envelope (`--ir`) or a native ESS specification (`--path`) to OpenAPI. `--ir` refuses semantic
  gaps, unresolved references or legacy interface-only input before output; reimport the original
  OpenAPI to replace legacy files.
- [`ess generate project kubernetes`](#ess-generate-project-kubernetes) turns infrastructure intent
  and observation into manifests and obligations.

The commands under `ess generate project` write artifacts only. They do not call `kubectl`, apply a
manifest, or mutate a target.

`ess infra infra` contains `diagnose`, `graph`, and `diff` operations over sanitized infrastructure
IR — the same three as `ess infra diagnose`, `ess infra graph` and `ess infra diff`, which are their
flat spellings. Live or bundle scanning is under `ess infra import kubernetes`; manifest generation
is under `ess generate project kubernetes`. Their arguments are listed under
[`ess infra`](#ess-infra) in the command reference.

## `ess ui` — renderer-neutral UI documents

An `ess-ui/1` document ([reference](ess-ui.md)) describes an application's pages, state and
channels without naming a renderer. `ess ui load` loads one and prints how many pages and addressed
nodes it holds, or exits `1` naming the node that refuses it. `ess ui check` runs every check the
schema declares and, with `--model`, checks the document's views, commands and events against an
ESS specification; each finding names its node path, and any error exits `1`. `ess ui docs` renders the `ess-ui/1`
reference from its schema, and `--check` fails when a written copy is stale. `ess ui run --tui`
runs a document in the terminal against its fixtures, or with `--model` and `--base-url` against
the HTTP surface that specification serves, showing a refusal where the user acted; the
`Authorization` header comes from `ESS_UI_AUTHORIZATION`. `ess generate ui --target react` writes a
React + TypeScript project from one whose only runtime dependencies are `react` and `react-dom`,
with its routing generated into it and one esbuild step to build it; with `--model` the project
is bound to the HTTP surface that specification serves, reading and commanding its paths and
showing a refusal where the user acted. `ess generate ui --target tui --model` writes a Rust
terminal application crate holding the document, its binding and a clap command line
(`--base-url`, and `--screen-once <width>x<height>`, at most `1000x1000`, to print one rendered
frame without a terminal). `--screen-once` exits `0` when every read of the page it shows
answered; when one failed it still prints the frame, writes one stderr line naming each failed
read, and exits `3` (a refusal before the frame exits `1`, a usage error `2`). It holds no renderer
code: it depends on `ess-ui-tui` at the Git tag of the ESS release that generated it, so generate
with a released `ess`, whose tag exists. Each generated file opens with a `generated by ess
generate ui --target tui` line. An `--out` holding a file of the same name without that line, or a
symbolic link or a non-directory where the crate puts a file or `src/`, is refused before anything
is written; a document that opens with that line, such as the crate's own `src/ui.yaml`, is
embedded without it. None of these has a flat
spelling. Their arguments are listed under [`ess ui`](#ess-ui) and
[`ess generate ui`](#ess-generate-ui) in the command reference.

## Command reference

[ess-cli-begin]: #

This section is generated from the `ess` command definition by `cargo xtask cli-reference`. Change the command's help text and regenerate rather than editing it here.

### Global options

Accepted by every `ess` command, in any position.

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--strict-requires` |  | no |  | Refuse, rather than warn, where `ess-inputs.yaml` `requires` an older `ess` release |

### `ess specify`

Author a system, resolve what it says, and inspect the result

#### `ess specify protocol validate`

Validate an experimental ess-protospec/1 protocol document

```text
ess specify protocol validate [OPTIONS] --path <PATH>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |

#### `ess specify protocol compile`

Write canonical admitted protocol JSON (create-new output only)

```text
ess specify protocol compile [OPTIONS] --path <PATH>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--out` | `<OUT>` | no |  |  |

#### `ess specify cli`

Validate a typed CLI presentation binding against its selected ESS model

```text
ess specify cli [OPTIONS] --binding <BINDING>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--format` | `<FORMAT>` | no | `text` | Output rendering. One of `text`, `yaml`, `json`. |
| `--binding` | `<BINDING>` | yes |  | An independently authored ess-cli/1 presentation document |

#### `ess specify validate`

Validate and resolve an ESS specification

```text
ess specify validate [OPTIONS]
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--format` | `<FORMAT>` | no | `text` | Output rendering. One of `text`, `yaml`, `json`. |

#### `ess specify compile`

Compile a specification into canonical typed IR

```text
ess specify compile [OPTIONS]
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--format` | `<FORMAT>` | no | `text` | Output rendering. One of `text`, `yaml`, `json`. |
| `--out` | `<OUT>` | no |  | Where to write canonical JSON IR |

#### `ess specify compose`

Compile exact component surfaces into composition IR and generated clients

```text
ess specify compose [OPTIONS] --path <PATH> --service <KEY=PATH>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  | An `ess-composition/1`, `/2` or `/3` JSON or YAML document |
| `--service` | `<KEY=PATH>`… | yes |  | A compiled ESS source, written `service-key=path`. Repeat for every import |
| `--out` | `<OUT>` | no |  | Where to write canonical composition IR; its format echoes the input's (`/1`–`/3`) |
| `--client-plan-out` | `<CLIENT_PLAN_OUT>` | no |  | Where to write canonical `ess-client-plan/1` |
| `--client-rust-out` | `<CLIENT_RUST_OUT>` | no |  | Root for the generated dependency-free Rust composition client |
| `--ownership-root` | `<OWNERSHIP_ROOT>` | no |  | One enclosing ownership anchor for every composition output |
| `--format` | `<FORMAT>` | no | `text` | Output rendering. One of `text`, `yaml`, `json`. |

#### `ess specify inspect`

Inspect one declaration in resolved IR

```text
ess specify inspect [OPTIONS] <NAME>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `<NAME>` |  | yes |  | Fully qualified declaration name or binding/component identifier |
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--format` | `<FORMAT>` | no | `text` | Output rendering. One of `text`, `yaml`, `json`. |

#### `ess specify graph`

Render the interaction graph

```text
ess specify graph [OPTIONS]
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--format` | `<FORMAT>` | no | `mermaid` | One of `dot`, `mermaid`, `json`, `yaml`. |

#### `ess specify realization validate`

Validate and resolve an `ess-realization/1` or `/2` declaration

```text
ess specify realization validate [OPTIONS] --path <PATH> --spec <SPECIFICATION>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  | An `ess-realization/1` or `/2` JSON or YAML document |
| `--spec` | `<SPECIFICATION>` | yes |  | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--format` | `<FORMAT>` | no | `text` | Output and diagnostic rendering. One of `text`, `yaml`, `json`. |

#### `ess specify realization compile`

Compile a declaration into its versioned canonical realization IR

```text
ess specify realization compile [OPTIONS] --path <PATH> --spec <SPECIFICATION>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  | An `ess-realization/1` or `/2` JSON or YAML document |
| `--spec` | `<SPECIFICATION>` | yes |  | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--format` | `<FORMAT>` | no | `text` | Output and diagnostic rendering. One of `text`, `yaml`, `json`. |
| `--out` | `<OUT>` | no |  | Where to write canonical JSON IR |

#### `ess specify realization generate`

Generate the deterministic public-facing run-mode guide

```text
ess specify realization generate [OPTIONS] --path <PATH> --spec <SPECIFICATION> --out <OUT>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  | An `ess-realization/1` or `/2` JSON or YAML document |
| `--spec` | `<SPECIFICATION>` | yes |  | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--format` | `<FORMAT>` | no | `text` | Output and diagnostic rendering. One of `text`, `yaml`, `json`. |
| `--out` | `<OUT>` | yes |  | Markdown file to write or compare |
| `--check` |  | no |  | Compare with `--out` and fail on drift instead of writing |

#### `ess specify transport validate`

Validate and resolve an `ess-transport/1` or `/2` document

```text
ess specify transport validate [OPTIONS] --path <PATH> --spec <SPECIFICATION>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  | An `ess-transport/1` or `/2` JSON or YAML document |
| `--spec` | `<SPECIFICATION>` | yes |  | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--format` | `<FORMAT>` | no | `text` | Output and diagnostic rendering. One of `text`, `yaml`, `json`. |

#### `ess specify transport compile`

Compile a document into its canonical `ess-transport-ir/1` or `/2` form

```text
ess specify transport compile [OPTIONS] --path <PATH> --spec <SPECIFICATION>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  | An `ess-transport/1` or `/2` JSON or YAML document |
| `--spec` | `<SPECIFICATION>` | yes |  | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--format` | `<FORMAT>` | no | `text` | Output and diagnostic rendering. One of `text`, `yaml`, `json`. |
| `--out` | `<OUT>` | no |  | Where to write canonical JSON IR |

#### `ess specify runtime compile`

Compile `ess-runtime/1` against exact semantic, realization, and build inputs

```text
ess specify runtime compile [OPTIONS] --path <PATH> --system <SYSTEM> --realization <REALIZATION> --build-ir <BUILD_IR>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  | Authored runtime JSON or YAML |
| `--system` | `<SYSTEM>` | yes |  | ESS source file or directory |
| `--realization` | `<REALIZATION>` | yes |  | Authored `ess-realization/1` bound to the same ESS source |
| `--build-ir` | `<BUILD_IR>` | yes |  | Compiled `ess-build-ir/1` JSON |
| `--out` | `<OUT>` | no |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess specify toolchain install`

Download a released `ess`, verify it against the release's SHA256SUMS, and cache it.

Only published releases install: building from a git revision or a tag is not supported. A release already cached is left as it is. Never prompts.

```text
ess specify toolchain install [OPTIONS] <VERSION>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `<VERSION>` |  | yes |  | The exact release, `X.Y.Z` |
| `--pin` |  | no |  | Also write `requires: ess X.Y.Z` into the nearest `ess-inputs.yaml` |

#### `ess specify toolchain list`

List the cached releases, oldest first

```text
ess specify toolchain list [OPTIONS]
```

No arguments beyond the global options.

#### `ess specify toolchain which`

Print the release that would run here, and why: the override, the pin, or this `ess`

```text
ess specify toolchain which [OPTIONS]
```

No arguments beyond the global options.

### `ess generate`

Turn a resolved system into artifacts and cross explicit delivery executor boundaries.

The options below belong to the `generate` verb, which is spelled either way: `ess generate --path …` is `ess generate generate --path …`. They cannot be written beside one of the other verbs, which would say two things at once.

```text
ess generate [OPTIONS]
ess generate <COMMAND>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--kind` | `<KIND>` | no |  | One of `docs`, `site`, `docs-ir`, `schema`, `openapi`, `asyncapi`. |
| `--include` | `<PAGE=PATH>`… | no |  | An authored Markdown page, written `<page-id>=<path>`. Repeat for multiple pages |
| `--front-page` | `<PATH>` | no |  | Override the specification-adjacent README without changing its source directory |
| `--asset` | `<OUTPUT=PATH>`… | no |  | Publish a declared UTF-8 download verbatim, written `<output-path>=<source-path>` |
| `--strict-links` |  | no |  | Refuse unpublished local link targets before writing any output |
| `--out` | `<OUT>` | no |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |
| `--strict` |  | no |  | Refuse, writing nothing, where `openapi` or `asyncapi` has a domain no component owns.<br /><br />Without it the same condition is a note on stderr and the exit stays 0: an empty projection is legal, and the note is what tells it apart from a clean one. |
| `--transport` | `<TRANSPORT>` | no |  | An `ess-transport/1` or `ess-transport/2` document binding events to brokers, subjects and streams; only with `--kind asyncapi` |

#### `ess generate generate`

Generate deterministic documentation, schemas, and interface contracts.

`--kind site` opens on the `README.md` beside the specification, where there is one, and takes any number of `--include` pages beside the generated ones — a plan board another tool rendered, a runbook. Both are markdown somebody wrote, read into the document and styled like every other page.

```text
ess generate generate [OPTIONS]
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--kind` | `<KIND>` | no |  | One of `docs`, `site`, `docs-ir`, `schema`, `openapi`, `asyncapi`. |
| `--include` | `<PAGE=PATH>`… | no |  | An authored Markdown page, written `<page-id>=<path>`. Repeat for multiple pages |
| `--front-page` | `<PATH>` | no |  | Override the specification-adjacent README without changing its source directory |
| `--asset` | `<OUTPUT=PATH>`… | no |  | Publish a declared UTF-8 download verbatim, written `<output-path>=<source-path>` |
| `--strict-links` |  | no |  | Refuse unpublished local link targets before writing any output |
| `--out` | `<OUT>` | no |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |
| `--strict` |  | no |  | Refuse, writing nothing, where `openapi` or `asyncapi` has a domain no component owns.<br /><br />Without it the same condition is a note on stderr and the exit stays 0: an empty projection is legal, and the note is what tells it apart from a clean one. |
| `--transport` | `<TRANSPORT>` | no |  | An `ess-transport/1` or `ess-transport/2` document binding events to brokers, subjects and streams; only with `--kind asyncapi` |

#### `ess generate cli`

Generate a parser and process adapter from a typed CLI presentation binding

```text
ess generate cli [OPTIONS] --binding <BINDING> --out <OUT>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--format` | `<FORMAT>` | no | `text` | Output rendering. One of `text`, `yaml`, `json`. |
| `--binding` | `<BINDING>` | yes |  | An independently authored ess-cli/1 presentation document |
| `--out` | `<OUT>` | yes |  | Generated package destination, outside the specification inputs |
| `--check` |  | no |  | Compare generated bytes without changing output files |

#### `ess generate ui`

Generate an application from an `ess-ui/1` document

```text
ess generate ui [OPTIONS] --target <TARGET> --path <PATH> --out <OUT>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--target` | `<TARGET>` | yes |  | What to generate. One of `react`, `tui`. |
| `--path` | `<PATH>` | yes |  | The `ess-ui/1` document |
| `--out` | `<OUT>` | yes |  | Directory the project is written to |
| `--model` | `<MODEL>` | no |  | The ESS specification the document's `model:` names (a directory, its `ess-inputs.yaml`, or one file): the app is bound to the HTTP surface the specification's `reached_by: network` components serve, instead of answering from fixtures |

#### `ess generate output adopt`

Enroll only existing bytes matching a settled generated reference

```text
ess generate output adopt [OPTIONS] --ownership-root <OWNERSHIP_ROOT> --from <REFERENCE> --owner <OWNER>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--ownership-root` | `<OWNERSHIP_ROOT>` | yes |  |  |
| `--from` | `<REFERENCE>` | yes |  |  |
| `--owner` | `<OWNER>` | yes |  | Fixed generator family, such as projection:site or typescript-file |
| `--file` | `<FILE>` | no |  | One native filename; required only for a standalone file family |

#### `ess generate output recover`

Settle the recorded operation without loading specification inputs

```text
ess generate output recover [OPTIONS] --ownership-root <OWNERSHIP_ROOT>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--ownership-root` | `<OWNERSHIP_ROOT>` | yes |  |  |

#### `ess generate types`

Realize selected model types as standalone, accounted data libraries

```text
ess generate types [OPTIONS] --target <TARGET> --out <OUT> <--root <ROOT>|--all-types|--all-events>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--root` | `<ROOT>`… | no |  | Qualified model type or event root. Repeat for a shared transitive closure |
| `--all-types` |  | no |  | Explicitly select every named type in the resolved model |
| `--all-events` |  | no |  | Explicitly select every event payload in the resolved model; combines with `--all-types` |
| `--target` | `<TARGET>` | yes |  | Data library target. Unsupported language targets are not silently substituted. One of `typescript`, `rust`, `go`. |
| `--package` | `<PACKAGE>` | no |  | Native package identity, required for Rust and Go |
| `--module` | `<MODULE>` | no |  | Go module identity, required only for Go |
| `--names` | `<NAMES>` | no | `qualified` | How declarations are named: the qualified model name, or its last segment. One of `qualified`, `short`. |
| `--out` | `<OUT>` | yes |  | Library destination, outside the specification input tree |

#### `ess generate client`

Generate a typed event publisher for one component over its transport document

```text
ess generate client [OPTIONS] --component <COMPONENT> --transport <TRANSPORT> --target <TARGET> --package <PACKAGE> --out <OUT>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--component` | `<COMPONENT>` | yes |  | The component whose published events the publisher sends |
| `--transport` | `<TRANSPORT>` | yes |  | The `ess-transport/1` or `ess-transport/2` document binding those events |
| `--target` | `<TARGET>` | yes |  | The language to generate. One of `rust`, `go`. |
| `--package` | `<PACKAGE>` | yes |  | Native package identity |
| `--module` | `<MODULE>` | no |  | Go module identity, required only for Go |
| `--names` | `<NAMES>` | no | `qualified` | How declarations are named: the qualified model name, or its last segment. One of `qualified`, `short`. |
| `--out` | `<OUT>` | yes |  | Library destination, outside the specification input tree |

#### `ess generate synthesize`

Synthesize implementation artifacts and explicit obligations

```text
ess generate synthesize [OPTIONS]
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--target` | `<TARGET>` | no | `rust` | One of `rust`, `go`, `web`, `clap`. |
| `--layout` | `<LAYOUT>` | no | `workspace` | How the `rust` target lays its output out: a workspace of crates, or one crate at `--out` with its HTTP surface behind a `server` Cargo feature. Other targets refuse `crate`. One of `workspace`, `crate`. |
| `--out` | `<OUT>` | no |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess generate project buildkit`

Project canonical build IR to `BuildKit` Dockerfile and Bake inputs

```text
ess generate project buildkit [OPTIONS] --ir <IR> --out <OUT>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--ir` | `<IR>` | yes |  | Compiled `ess-build-ir/1` JSON |
| `--out` | `<OUT>` | yes |  | Output directory |

#### `ess generate project helm`

Project runtime IR into one configuration-neutral component Helm chart

```text
ess generate project helm [OPTIONS] --ir <IR> --chart <CHART> --version <VERSION> --out <OUT>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--ir` | `<IR>` | yes |  | Compiled `ess-runtime-ir/1` JSON |
| `--chart` | `<CHART>` | yes |  | Stable chart name |
| `--version` | `<VERSION>` | yes |  | Independent chart version |
| `--out` | `<OUT>` | yes |  | Output directory |

#### `ess generate project kubernetes`

Project infrastructure intent and observed IR into Kubernetes manifests and obligations

```text
ess generate project kubernetes [OPTIONS] --spec <SPEC> --ir <IR>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--spec` | `<SPEC>` | yes |  |  |
| `--ir` | `<IR>` | yes |  |  |
| `--out` | `<OUT>` | no |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess generate project openapi`

Project supported ESS service/interface structures into `OpenAPI`

```text
ess generate project openapi [OPTIONS]
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no |  | ESS specification to compile and project |
| `--ir` | `<IR>` | no |  | Checked `ess-openapi-import/1` to project; partial, unresolved or legacy input refuses |
| `--out` | `<OUT>` | no |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |
| `--strict` |  | no |  | With `--path`: refuse, writing nothing, where a domain has no owning component |

#### `ess generate schema import-bundle`

Extract a qualified structural component closure without inventing a service

```text
ess generate schema import-bundle [OPTIONS] --path <PATH> --component <COMPONENT> --dialect <DIALECT>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  | Original JSON document containing components/schemas. Its exact bytes are retained |
| `--component` | `<COMPONENT>`… | yes |  | Select a named component root. Repeat to select multiple roots and their closure |
| `--dialect` | `<DIALECT>` | yes |  | Required structural interpretation; never inferred from the envelope's version claim. One of `draft-2020-12`. |
| `--out` | `<OUT>` | no |  | Write the qualified import envelope here; omit to print it |

#### `ess generate schema import-document`

Retain a JSON Schema document root and its local definitions without inventing a service

```text
ess generate schema import-document [OPTIONS] --path <PATH> --root <ROOT> --dialect <DIALECT>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  | Original JSON Schema 2020-12 document, including its root and local $defs |
| `--root` | `<ROOT>` | yes |  | Explicit identity for the document root, distinct from every existing definition |
| `--definition` | `<DEFINITION>`… | no |  | Additional $defs roots to make independently selectable |
| `--dialect` | `<DIALECT>` | yes |  | Required interpretation; an incompatible declared dialect is refused. One of `draft-2020-12`. |
| `--out` | `<OUT>` | no |  | Write the replay-checked document bundle here; omit to print it |

#### `ess generate schema project-bundle`

Revalidate a qualified import and project one selected root as standalone JSON Schema

```text
ess generate schema project-bundle [OPTIONS] --bundle <BUNDLE> --root <ROOT> --schema-id <SCHEMA_ID>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--bundle` | `<BUNDLE>` | yes |  | Persisted component (/1) or document-root (/2) bundle to revalidate |
| `--root` | `<ROOT>` | yes |  | One root explicitly selected by that import |
| `--schema-id` | `<SCHEMA_ID>` | yes |  | Absolute identity for the standalone projected schema, without a fragment |
| `--out` | `<OUT>` | no |  | Write the qualified standalone schema here; omit to print it |

#### `ess generate schema validate-bundle`

Validate unmodified JSON instances against one selected component root

```text
ess generate schema validate-bundle [OPTIONS] --bundle <BUNDLE> --root <ROOT> <INSTANCES>...
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `<INSTANCES>`… |  | yes |  | JSON instance files. No schema selector field is added to their data |
| `--bundle` | `<BUNDLE>` | yes |  | Persisted component (/1) or document-root (/2) bundle to revalidate |
| `--root` | `<ROOT>` | yes |  | One root explicitly selected by that import |

#### `ess generate schema types-bundle`

Realize selected structural roots with source provenance and target accounting

```text
ess generate schema types-bundle [OPTIONS] --bundle <BUNDLE> --root <ROOT> --target <TARGET> --out <OUT>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--bundle` | `<BUNDLE>` | yes |  | Persisted component (/1) or document-root (/2) bundle, revalidated before type planning |
| `--root` | `<ROOT>`… | yes |  | Root explicitly selected by the import. Repeat to select a shared closure |
| `--target` | `<TARGET>` | yes |  | Data library target. Unsupported language targets are not silently substituted. One of `typescript`, `rust`, `go`. |
| `--package` | `<PACKAGE>` | no |  | Native package identity, required for Rust and Go |
| `--module` | `<MODULE>` | no |  | Go module identity, required only for Go |
| `--out` | `<OUT>` | yes |  | Directory for declarations, target accounting and the retained source bundle |

#### `ess generate schema normalize-check`

Check every source-pinned normalization branch and emit the canonical recipe

```text
ess generate schema normalize-check [OPTIONS] --recipe <RECIPE> <--bundle <BUNDLE>|--model <MODEL>>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--recipe` | `<RECIPE>` | yes |  | Authored ess-normalization/1 through /6 recipe; every branch is checked first |
| `--bundle` | `<BUNDLE>`… | no |  | Replay-checked source bundles referenced by canonical digest. Repeat as needed |
| `--model` | `<MODEL>`… | no |  | Compile a model specification and recheck its pinned selections. Repeat as needed |
| `--out` | `<OUT>` | no |  | Write the canonical checked recipe here; omit for JSON on stdout |

#### `ess generate schema normalize-run`

Execute one explicit normalization branch with checked input and output boundaries

```text
ess generate schema normalize-run [OPTIONS] --recipe <RECIPE> --branch <BRANCH> --input <INPUT> <--bundle <BUNDLE>|--model <MODEL>>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--recipe` | `<RECIPE>` | yes |  | Authored ess-normalization/1 through /6 recipe; every branch is checked first |
| `--bundle` | `<BUNDLE>`… | no |  | Replay-checked source bundles referenced by canonical digest. Repeat as needed |
| `--model` | `<MODEL>`… | no |  | Compile a model specification and recheck its pinned selections. Repeat as needed |
| `--branch` | `<BRANCH>` | yes |  | Exact external discriminator; never inferred from the input object |
| `--input` | `<INPUT>` | yes |  | Original JSON text; declared capture, numeric and positional input policies apply |
| `--out` | `<OUT>` | no |  | Write the complete result here; omit for JSON on stdout. Refusals write no result |

#### `ess generate schema normalize-generate`

Emit a source-pinned normalization library, or check its generated file bytes

```text
ess generate schema normalize-generate [OPTIONS] --recipe <RECIPE> --target <TARGET> --package <PACKAGE> --out <OUT> <--bundle <BUNDLE>|--model <MODEL>>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--recipe` | `<RECIPE>` | yes |  | Authored ess-normalization/1 through /6 recipe; every branch is checked first |
| `--bundle` | `<BUNDLE>`… | no |  | Replay-checked source bundles referenced by canonical digest. Repeat as needed |
| `--model` | `<MODEL>`… | no |  | Compile a model specification and recheck its pinned selections. Repeat as needed |
| `--target` | `<TARGET>` | yes |  | Executable normalization target, not structural type projection. One of `rust`, `go`, `typescript`. |
| `--package` | `<PACKAGE>` | yes |  | Native library package identity |
| `--module` | `<MODULE>` | no |  | Go module identity; required for Go and refused for other targets |
| `--out` | `<OUT>` | yes |  | Directory for generated library, source inputs and provenance report |
| `--check` |  | no |  | Check planned file bytes without writing; unrelated files are not inspected or removed |

#### `ess generate schema validate`

Validate JSON instances against schemas selected by their `schema` property

```text
ess generate schema validate [OPTIONS] --schemas <DIR> <PATHS>...
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `<PATHS>`… |  | yes |  | JSON files or directories to validate. Directories are searched recursively |
| `--schemas` | `<DIR>` | yes |  | Directory containing the authoritative `*.schema.json` registry |
| `--format` | `<FORMAT>` | no | `text` | How to render the validation report. One of `text`, `yaml`, `json`. |

#### `ess generate schema typescript`

Project a schema's structural types into a deterministic TypeScript module

```text
ess generate schema typescript [OPTIONS] --root <ROOT> --schemas <DIR> <SCHEMA_ID>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `<SCHEMA_ID>` |  | yes |  | The schema's exact `$id`, not its filename |
| `--root` | `<ROOT>` | yes |  | The exported root type name |
| `--schemas` | `<DIR>` | yes |  | Directory containing the authoritative `*.schema.json` registry |
| `--out` | `<OUT>` | no |  | Write the generated module here. Omit to print it |
| `--check` |  | no |  | Refuse when `--out` differs from the generated module, without writing it |

#### `ess generate build compile`

Validate and compile `ess-build/1` to canonical `ess-build-ir/1`

```text
ess generate build compile [OPTIONS] --path <PATH>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--out` | `<OUT>` | no |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess generate build graph`

Render the validated build DAG as deterministic Mermaid source

```text
ess generate build graph [OPTIONS] --path <PATH>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--out` | `<OUT>` | no |  |  |

#### `ess generate build execute`

Compile, project, and execute a build through Docker Buildx Bake

```text
ess generate build execute [OPTIONS] --path <PATH> --projection-out <PROJECTION_OUT>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  | Authored `ess-build/1` JSON or YAML |
| `--workdir` | `<WORKDIR>` | no | `.` | Repository root used as the `BuildKit` context |
| `--projection-out` | `<PROJECTION_OUT>` | yes |  | Directory receiving the reviewable `BuildKit` projection |
| `--target` | `<TARGETS>`… | no |  | Optional Bake target. Repeat to build a subset; omitted builds the default group |
| `--set` | `<SETTINGS>`… | no |  | Bake override such as `app.tags=registry.example/app:version`. Repeat as needed |
| `--push` |  | no |  | Push OCI outputs to their configured registries |
| `--load` |  | no |  | Load a single-platform OCI output into the local image store |

#### `ess generate component compile`

Validate and compile `ess-component/1` to canonical `ess-component-ir/1`

```text
ess generate component compile [OPTIONS] --path <PATH>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--out` | `<OUT>` | no |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess generate release verify`

Check release metadata consistency against exact build and runtime IR

```text
ess generate release verify [OPTIONS] --path <PATH> --build-ir <BUILD_IR> --runtime-ir <RUNTIME_IR>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--build-ir` | `<BUILD_IR>` | yes |  |  |
| `--runtime-ir` | `<RUNTIME_IR>` | yes |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess generate release bundle`

Combine a component and consistency-checked runtime/chart releases into an OCI payload

```text
ess generate release bundle [OPTIONS] --component-ir <COMPONENT_IR> --build-ir <BUILD_IR> --runtime-ir <RUNTIME_IR> --release <RELEASES>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--component-ir` | `<COMPONENT_IR>` | yes |  |  |
| `--build-ir` | `<BUILD_IR>` | yes |  |  |
| `--runtime-ir` | `<RUNTIME_IR>` | yes |  |  |
| `--release` | `<RELEASES>`… | yes |  | Executor-produced `ess-release/1` manifest. Supply once per release unit |
| `--out` | `<OUT>` | no |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess generate release verify-bundle`

Revalidate an `ess-release-bundle/1` received from an untrusted boundary

```text
ess generate release verify-bundle [OPTIONS] --path <PATH>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess generate release publish`

Publish a consistency-checked bundle, optionally qualifying a supplied local report

```text
ess generate release publish [OPTIONS] --path <PATH> --to <TO>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--to` | `<TO>` | yes |  | Tagged OCI destination. Consumers must use the digest printed by this command |
| `--spec` | `<SPEC>` | no |  | Explicit authored ESS model root, loaded once |
| `--report` | `<REPORT>` | no |  | Original standalone ess-conformance-report/2 JSON; never a generic check log |
| `--expected-suite` | `<EXPECTED_SUITE>` | no |  | Independently supplied original unfiltered coverage suite/5 or /7 JSON |
| `--expected-suite-input` | `<EXPECTED_SUITE_INPUT>` | no |  | Independently supplied original input/1 carrier, including every parent suite |
| `--report-sha256` | `<REPORT_SHA256>` | no |  | Optional SHA-256 pin of the raw report bytes, distinct from the OCI manifest digest |
| `--expected-input-sha256` | `<EXPECTED_INPUT_SHA256>` | no |  | Optional SHA-256 pin of the raw selected suite file or complete input carrier file |

#### `ess generate release fetch`

Check bundle content identity and consistency, then cache a digest-pinned OCI payload

```text
ess generate release fetch [OPTIONS] --from <FROM> --cache <CACHE>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--from` | `<FROM>` | yes |  | OCI source ending in `@sha256:<64 lowercase hex characters>` |
| `--cache` | `<CACHE>` | yes |  | Content-addressed cache root |
| `--out` | `<OUT>` | no |  | Optional copy of the verified canonical bundle |

#### `ess generate release check-conformance`

Qualify an original report/2 against an independent selection and explicit model, offline

```text
ess generate release check-conformance [OPTIONS] --component-ir <COMPONENT_IR> --build-ir <BUILD_IR> --runtime-ir <RUNTIME_IR> <--spec <SPEC>>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--spec` | `<SPEC>` | no |  | Explicit authored ESS model root, loaded once |
| `--report` | `<REPORT>` | no |  | Original standalone ess-conformance-report/2 JSON; never a generic check log |
| `--expected-suite` | `<EXPECTED_SUITE>` | no |  | Independently supplied original unfiltered coverage suite/5 or /7 JSON |
| `--expected-suite-input` | `<EXPECTED_SUITE_INPUT>` | no |  | Independently supplied original input/1 carrier, including every parent suite |
| `--report-sha256` | `<REPORT_SHA256>` | no |  | Optional SHA-256 pin of the raw report bytes, distinct from the OCI manifest digest |
| `--expected-input-sha256` | `<EXPECTED_INPUT_SHA256>` | no |  | Optional SHA-256 pin of the raw selected suite file or complete input carrier file |
| `--component-ir` | `<COMPONENT_IR>` | yes |  |  |
| `--build-ir` | `<BUILD_IR>` | yes |  |  |
| `--runtime-ir` | `<RUNTIME_IR>` | yes |  |  |

#### `ess generate release publish-conformance`

Qualify and upload the exact original report bytes in the same process through ORAS

```text
ess generate release publish-conformance [OPTIONS] --component-ir <COMPONENT_IR> --build-ir <BUILD_IR> --runtime-ir <RUNTIME_IR> --to <TO> <--spec <SPEC>>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--spec` | `<SPEC>` | no |  | Explicit authored ESS model root, loaded once |
| `--report` | `<REPORT>` | no |  | Original standalone ess-conformance-report/2 JSON; never a generic check log |
| `--expected-suite` | `<EXPECTED_SUITE>` | no |  | Independently supplied original unfiltered coverage suite/5 or /7 JSON |
| `--expected-suite-input` | `<EXPECTED_SUITE_INPUT>` | no |  | Independently supplied original input/1 carrier, including every parent suite |
| `--report-sha256` | `<REPORT_SHA256>` | no |  | Optional SHA-256 pin of the raw report bytes, distinct from the OCI manifest digest |
| `--expected-input-sha256` | `<EXPECTED_INPUT_SHA256>` | no |  | Optional SHA-256 pin of the raw selected suite file or complete input carrier file |
| `--component-ir` | `<COMPONENT_IR>` | yes |  |  |
| `--build-ir` | `<BUILD_IR>` | yes |  |  |
| `--runtime-ir` | `<RUNTIME_IR>` | yes |  |  |
| `--to` | `<TO>` | yes |  | Tagged OCI evidence destination; its returned manifest digest is not the report hash |

#### `ess generate stack resolve`

Resolve constraints to an exact `ess-stack-lock/1` using only the supplied catalogue

```text
ess generate stack resolve [OPTIONS] --path <PATH> --catalog <CATALOG>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--catalog` | `<CATALOG>` | yes |  |  |
| `--out` | `<OUT>` | no |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess generate stack validate`

Validate that a generic stack resolves completely

```text
ess generate stack validate [OPTIONS] --path <PATH> --catalog <CATALOG>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--catalog` | `<CATALOG>` | yes |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess generate deployment compile`

Bind an exact stack lock to an environment and emit `ess-deployment/1`

```text
ess generate deployment compile [OPTIONS] --path <PATH> --stack-lock <STACK_LOCK>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--stack-lock` | `<STACK_LOCK>` | yes |  |  |
| `--out` | `<OUT>` | no |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess generate deployment diff`

Report which independent releases differ between two deployment IR documents

```text
ess generate deployment diff [OPTIONS] --from <FROM> --to <TO>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--from` | `<FROM>` | yes |  |  |
| `--to` | `<TO>` | yes |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `json`. |

#### `ess generate deployment reconcile`

Reconcile changed independent Helm releases under an admitted recovery authority

```text
ess generate deployment reconcile [OPTIONS] --path <PATH> --cache <CACHE>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  | Desired canonical `ess-deployment/1` document |
| `--current` | `<CURRENT>` | no |  | Admitted baseline desired deployment. Omit for a first deployment |
| `--cache` | `<CACHE>` | yes |  | Cache root for digest-pinned chart artifacts |
| `--allow-removals` |  | no |  | Permit uninstalling releases absent from the desired deployment |
| `--dry-run` |  | no |  | Report a local unverified comparison preview and stop |
| `--timeout` | `<TIMEOUT>` | no | `5m` | Helm wait timeout |
| `--authority` | `<AUTHORITY>` | no |  | Select this authority from the protected recovery registry. Required for execution |
| `--retry-of` | `<RETRY_OF>` | no |  | Reference a predecessor invocation as `<store epoch>:<nonce>`; never a history filter |

### `ess verify`

Hold an implementation, or a later revision, to what a system says

#### `ess verify protocol run`

Simulate an unambiguous action sequence; this is model evidence, not target conformance

```text
ess verify protocol run [OPTIONS] --path <PATH> --actions <ACTIONS>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--actions` | `<ACTIONS>` | yes |  |  |
| `--out` | `<OUT>` | no |  |  |

#### `ess verify protocol replay`

Check an exact ess-prototrace/1 observation record against its model

```text
ess verify protocol replay [OPTIONS] --path <PATH> --trace <TRACE>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--trace` | `<TRACE>` | yes |  |  |

#### `ess verify protocol explore`

Explore bounded schedules with optional finite typed input witnesses

```text
ess verify protocol explore [OPTIONS] --path <PATH>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--inputs` | `<INPUTS>` | no |  |  |

#### `ess verify bindings`

Verify declared implementation bindings against scoped Kubernetes observations

```text
ess verify bindings [OPTIONS] --spec <SPEC> --realization <REALIZATION> --bindings <BINDINGS>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--spec` | `<SPEC>` | yes |  | Exact semantic ESS source directory or file |
| `--realization` | `<REALIZATION>` | yes |  | Source-owned ess-realization/1 or /2 document |
| `--bindings` | `<BINDINGS>` | yes |  | Environment-owned ess-observed-bindings/1 or /2 document |
| `--infra` | `<INFRA>` | no |  | Existing native infrastructure observation or IR; never implies a fresh live read |
| `--live` |  | no |  | Collect the document's exact context and namespace before checking |
| `--observation-out` | `<OBSERVATION_OUT>` | no |  | New observation file outside Git checkouts; its parent must already exist |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `json`. |
| `--markdown-out` | `<MARKDOWN_OUT>` | no |  | Optional deterministic generated reference; parent must exist and file must be new |

#### `ess verify conform synthesize`

Generate the suite the specification obliges.

`ir` writes the canonical suite document to the file `--out` names. `go` and `typescript` write a test package into the directory `--out` names — the runner, the evaluator and the suite — so an implementation in that language can be held to the specification by `go test` or `npm test` rather than by nothing, which is what a synthesized suite no runner can reach amounts to.

```text
ess verify conform synthesize [OPTIONS]
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--format` | `<FORMAT>` | no | `text` | Output rendering. One of `text`, `yaml`, `json`. |
| `--target` | `<TARGET>` | no | `ir` | What to write the suite as. One of `ir`, `go`, `typescript`. |
| `--out` | `<OUT>` | no |  | A file for `--target ir`, a directory for `--target go` or `--target typescript` |
| `--component` | `<COMPONENT>` | no |  | Hold one component to the specification rather than the whole system.<br /><br />Keeps only the scenarios whose every command, event and view this component accepts, publishes or owns, and lists the rest with what they need — they belong in the suite of the component that realises it. An implementation of one component answers `ErrUnsupported` to the other's scenarios, and a run with skips in it cannot say it passed. |
| `--scenarios` | `<SCENARIOS>` | no |  | The `ess-scenario/1` documents to compile beside the generated scenarios.<br /><br />One file or a directory. Immediate `ess-inputs.yaml` selects its exact scenarios list. Otherwise only immediate `.yaml`/`.yml` files are read; subdirectories are not searched. An empty selection is refused. When omitted, no authored scenarios are selected. |
| `--suite-format` | `<SUITE_FORMAT>` | no | `4` | Ordinary (4) or declared coverage (5); admitted features select newer required versions. One of `4`, `5`. |
| `--compact` |  | no |  | Write fresh IR as compact JSON with one trailing newline; requires --target ir |

#### `ess verify conform author`

Compile the scenarios an author wrote, and nothing the specification obliges.

The authoring surface on its own: every command, actor, outcome, event, error, view, entity, field, enum variant and lifecycle state a scenario names is resolved against the model, and a name it does not declare is refused here rather than at the first run that reaches it.

```text
ess verify conform author [OPTIONS]
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--format` | `<FORMAT>` | no | `text` | Output rendering. One of `text`, `yaml`, `json`. |
| `--scenarios` | `<SCENARIOS>` | no |  | One scenario file, or a directory using `ess-inputs.yaml` or shallow `.yaml`/`.yml` selection |
| `--out` | `<OUT>` | no |  | Where to write the compiled ordinary suite |
| `--suite-format` | `<SUITE_FORMAT>` | no | `4` | Ordinary (4) or declared coverage (5); retained features select newer required versions. One of `4`, `5`. |

#### `ess verify conform web`

Emit a browser conformance product for the scenarios.

Writes the original specification files, the admitted suite (`suite.json`) or coverage input (`input.json`), a Rust-derived `declarations.json` and a `browser.json` manifest that binds them all by digest. Serve the directory and open `index.html` to navigate every declaration; the page labels them admitted at emission and not executed.

Execution needs `rust/browser_host.rs` built for `wasm32-unknown-unknown` with your own target installation and copied beside `index.html` as `runner.wasm`; the emitted `README.md` gives the commands. The module re-admits the original bytes before any target call, and only its Rust runner produces reports.

```text
ess verify conform web [OPTIONS]
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--format` | `<FORMAT>` | no | `text` | Output rendering. One of `text`, `yaml`, `json`. |
| `--scenarios` | `<SCENARIOS>` | no |  | One scenario file, or a directory using `ess-inputs.yaml` or shallow `.yaml`/`.yml` selection |
| `--out` | `<OUT>` | no |  | Where to write the product |
| `--suite-format` | `<SUITE_FORMAT>` | no | `4` | Ordinary (4) or declared coverage (5); coverage emits the complete coverage input. One of `4`, `5`. |
| `--history` | `<HISTORY>` | no |  | An `ess-history/1` or `ess-history/2` document: draw it, checked against the specification, as one lane per client in a single self-contained `index.html`, printed when `--out` is absent. `--out` replaces the files `ess` owns in that directory, the player's included, so write history pages and the player to different directories |

#### `ess verify conform select`

Narrow a coverage suite/5, /7 or /9 by explicit IDs, retaining every exact original parent

```text
ess verify conform select [OPTIONS] --ids <IDS> --out <OUT>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--suite` | `<SUITE>` | no |  |  |
| `--suite-input` | `<SUITE_INPUT>` | no |  |  |
| `--ids` | `<IDS>` | yes |  | JSON array of sorted distinct IDs; an empty array is explicit |
| `--out` | `<OUT>` | yes |  | Destination for the complete ess-conformance-input/1 carrier |

#### `ess verify conform run`

Run a generated or committed suite against a built-in reference implementation

```text
ess verify conform run [OPTIONS] --target <TARGET>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no |  | The specification: synthesized when no suite is named, and executed by `interpreted`.<br /><br />Defaults to `.` for the other targets. `--target interpreted` requires it, because the interpreter executes this model and refuses one the suite was not synthesized from. |
| `--suite` | `<SUITE>` | no |  |  |
| `--suite-input` | `<SUITE_INPUT>` | no |  | Original coverage suite/5, /7 or /9 and its complete original parent chain |
| `--suite-format` | `<SUITE_FORMAT>` | no |  | Fresh ordinary (4) or declared coverage (5); accessors select 6 or 7. Loaded versions stay unchanged. One of `4`, `5`. |
| `--scenarios` | `<SCENARIOS>` | no |  | The `ess-scenario/1` documents to run beside the generated scenarios.<br /><br />Read only where the suite is synthesized rather than named by `--suite`: a committed suite already holds whatever was compiled into it. |
| `--target` | `<TARGET>` | yes |  | One of `billing`, `oracle-fixture`, `interpreted`. |
| `--report-out` | `<REPORT_OUT>` | no |  | Standalone JSON destination; suites/5-/7 require explicit report/2, even without this option |
| `--report-format` | `<REPORT_FORMAT>` | no | `1` | Report contract version; JSON/YAML detailed v2 is ess-conformance-run/2. One of `1`, `2`. |
| `--strict` |  | no |  | Require passed complete conformance (unavailable for legacy unknown coverage) |
| `--allow-incomplete` |  | no |  | Explicitly retain diagnostic execution exit behavior |
| `--known-failing` | `<KNOWN_FAILING>` | no |  | An `ess-known-failures/1` declaration: scenarios this build of the target is known to fail, bound to the exact suite bytes, specification, implementation and build (the running `ess` executable's SHA-256). Accounted in `--accounting-out`; the report, its verdict and the exit status are unchanged. Requires `--report-format 2` and `--report-out`. A declaration that does not bind this run, or names a scenario that did not fail, exits 2 and writes nothing |
| `--accounting-out` | `<ACCOUNTING_OUT>` | no |  | Where to write the `ess-known-failure-accounting/1` document: a new file, not an input and not `--report-out` |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess verify conform report`

Turn a runner's own per-scenario results into a report/2 for a suite ESS admits.

For a runner outside ESS, written in any language, that executed the suite itself. ESS admits the suite (an original suite document or an `ess-conformance-input/1` carrier) and takes coverage, suite reference and policy from that admission; the runner supplies only an `ess-conformance-results/1` document with one terminal status per scenario. The report's `producer_profile` is `external-scenario-status/1` (naming `--runner` when given), so it cannot be read as a run ESS executed.

Refused, writing nothing: a result for a scenario the suite does not contain, a scenario with no result, two results for one scenario, a status other than passed, failed, error or unsupported, and a `suite_digest` that is not the admitted suite's.

Exit 0: the report was written, whatever its verdict. Exit 2: an input was refused.

Known failures (`--known-failing`, an `ess-known-failures/1` declaration) are accounted in a separate `ess-known-failure-accounting/1` document and never change a report or its verdict. With `--results`, they also need `--implementation-build` (the SHA-256 of the immutable target build, known to the host before the run) and `--execution-context-out`, where the `ess-conformance-execution/1` context binding that build to the report is written. A runner that wrote report/2 and its own context itself — the generated Go and TypeScript runners, given `ESS_IMPLEMENTATION_BUILD` and `ESS_EXECUTION_CONTEXT_OUT` — is accounted with `--observed-report`, `--execution-context`, `--known-failing` and `--accounting-out`, which rewrite nothing and write only the accounting. Every output is a new file; refused inputs exit 2 before anything is written.

```text
ess verify conform report [OPTIONS] --suite <SUITE>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--suite` | `<SUITE>` | yes |  | The suite the runner executed, exactly the bytes it was given |
| `--results` | `<RESULTS>` | no |  | The runner's `ess-conformance-results/1` document |
| `--implementation` | `<IMPLEMENTATION>` | no |  | The implementation the runner held to the suite, as the report names it |
| `--report-out` | `<REPORT_OUT>` | no |  | Where to write the canonical `ess-conformance-report/2` |
| `--runner` | `<RUNNER>` | no |  | The runner that produced the results, as `<name>@<version>` |
| `--observed-report` | `<OBSERVED_REPORT>` | no |  | Account a report/2 a runner wrote itself, from its original bytes; writes only `--accounting-out` |
| `--execution-context` | `<EXECUTION_CONTEXT>` | no |  | The `ess-conformance-execution/1` context the runner's host wrote beside `--observed-report` |
| `--known-failing` | `<KNOWN_FAILING>` | no |  | An `ess-known-failures/1` declaration to account the report's failures against |
| `--accounting-out` | `<ACCOUNTING_OUT>` | no |  | Where to write the `ess-known-failure-accounting/1` document, as a new file |
| `--implementation-build` | `<IMPLEMENTATION_BUILD>` | no |  | With `--results` and `--known-failing`: the `sha256:` identity of the target build the results came from, which the host knew before the run |
| `--execution-context-out` | `<EXECUTION_CONTEXT_OUT>` | no |  | With `--results` and `--known-failing`: where to write the execution context, as a new file |

#### `ess verify conform mutate`

Audit the suite with specification mutants, each replayed against a reference target.

Derives mutants from the specification — one altering edit each — synthesizes a fresh ordinary suite for the specification and for every mutant, and runs each on a fresh target that implements the unchanged specification. A mutant is killed when its suite fails there; a survivor is a declared rule no synthesized scenario pins down. It is answered by declaring what makes the rule observable, or by filing a synthesis gap — not by authoring a scenario, which runs identically in every mutant's suite and so can never kill one. No authored scenario is run.

A baseline scenario the target reports unsupported or skipped did not execute: it is listed, not scored, and each mutant is scored on the scenarios the baseline executed. A mutant that no scored scenario killed is inconclusive when a scenario it changed was not scored; otherwise equivalent (ESS-MUTATE-005) when it left its outcome's guard satisfied by no input, decided only for equality, membership and truth tests of input fields against literals; otherwise unwitnessed (ESS-MUTATE-004) when its suite gained synthesis refusals the baseline does not have, when it is on an outcome whose scenario the baseline refused, or when it is a from-drop or transition-to mutant on a transition only such outcomes perform. It survives when every scored scenario passed and each scenario it left unscored is the baseline's own, unchanged.

Exit 0: no baseline scenario failed or ended error, at least one mutant ran and was not equivalent, every scored mutant was killed or equivalent, and none is inconclusive or unwitnessed. Exit 1: the specification did not load, or at least one mutant survived. Exit 3: a baseline scenario failed or ended error (ESS-MUTATE-001), the baseline executed nothing (nothing scored), the classes found no site (ESS-MUTATE-003), or no mutant survived and at least one was unwitnessed or inconclusive, or none ran that was not equivalent. Exit 2: the `--known-failing` declaration was refused. Known failures are listed first in the text and never count as a pass: the audit makes no conformance claim.

For an implementation of your own, split the audit in two. `--emit DIR` writes the baseline suite to `DIR/baseline/suite.json`, every mutant's suite to `DIR/<mutant-id>/suite.json` and a manifest, and runs nothing (exit 0, or 3 on ESS-MUTATE-003). Run your runner over each suite and write its conformance report to `report.json` beside it. `--collect DIR` scores those reports with the exit statuses above; a missing report makes its mutant inconclusive. `--emit` writes an ess-mutation-manifest/3, or /4 where it holds a sets-drop or precedence-swap mutant or names a component; `--collect` also reads the /2 and /1 manifests earlier releases wrote.

For a repository that implements one component, `--emit --component NAME` writes the component's suites, as `synthesize --component` writes them, and marks out of scope every mutant whose site belongs to another component (a command that component handles, a view it owns, a transition its commands perform): it has no suite, and `--collect` lists it in an ess-mutation-report/4 naming the component rather than scoring it. A survivor on the component's own site is scored and counted. `--collect --component NAME` refuses an emission made for another component or for none. `--target` takes no `--component`: the built-in targets implement whole systems.

```text
ess verify conform mutate [OPTIONS] <--target <TARGET>|--emit <EMIT>|--collect <COLLECT>>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` |  |
| `--target` | `<TARGET>` | no |  | The reference implementation every suite runs against. One of `billing`, `oracle-fixture`, `interpreted`. |
| `--class` | `<CLASS>`… | no |  | Only these classes; every class when absent. One of `from-drop`, `transition-to`, `guard-boundary`, `sets-retarget`, `guard-negate`, `guard-connective`, `error-swap`, `emit-drop`, `order-flip`, `sets-drop`, `precedence-swap`. |
| `--emit` | `<EMIT>` | no |  | Write the baseline's and every mutant's suite, and a manifest, into this new or empty directory; run nothing |
| `--collect` | `<COLLECT>` | no |  | Score the `report.json` a runner wrote beside each suite of an emitted directory |
| `--component` | `<COMPONENT>` | no |  | With `--emit`, scope every suite to this declared component; with `--collect`, require the emission to have been scoped to it |
| `--report-out` | `<REPORT_OUT>` | no |  | Where to write the `ess-mutation-report/3` document (`/4` for a component or a declaration) |
| `--known-failing` | `<KNOWN_FAILING>` | no |  | An `ess-known-failures/1` declaration of baseline scenarios the target is known to fail. They are excluded from scoring rather than refused, and each mutant is scored on the scenarios the baseline passed: a declared scenario, or one the baseline's suite does not hold, never kills. Every failure it does not name still refuses with ESS-MUTATE-001. With `--target` it binds the running `ess` executable's SHA-256 as the build; `--emit` copies it into the emission and binds it there (ess-mutation-manifest/4); `--collect` uses only the declaration the emission bound, and refuses any other. A refused declaration exits 2 |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess verify conform check-history`

Check a recorded concurrent history for linearizability against the specification's model.

Reads an `ess-history/1` or `ess-history/2` document recorded against the specification at `--path`, and searches for an order of its operations the interpreter accepts, answer for answer, reading an operation's recorded `decision_time` as the current time. The history records no inputs, so an operation is explained by any candidate input synthesis would submit for its command. Reads of views are not judged and are listed. A violation is reported with the longest partial linearization found and a shrunk history that is still a violation.

Exit 0: linearizable. Exit 1: violation. Exit 3: unknown — the search spent `--budget` before it finished, which is never a pass. Exit 2: the specification or the history could not be read, the specification did not load, or the history or one of its operations was refused.

```text
ess verify conform check-history [OPTIONS] --history <HISTORY>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--history` | `<HISTORY>` | yes |  | The `ess-history/1` or `ess-history/2` document |
| `--budget` | `<BUDGET>` | no | `1000000` | How many executions of the model the search may spend; the same history and budget always give the same verdict |
| `--settle` | `<READS>` | no | `4` | How many of a session's reads of an `eventual` view, invoked after the writes stop, may still be behind; every later read is judged converged. A count of reads, not of instants, so the clock a history was written on changes no verdict |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `json`. |

#### `ess verify conform import-history`

Convert a recorded command/response log into an `ess-history/1` document.

`--log` is JSON Lines, one call per line, in the log's own shape. `--adapter` is an `ess-history-adapter/1` document that names, for every operation field, the JSON pointer it sits at in a line, or declares it `absent`, and maps the log's completion words to `Returned` and `Indeterminate`. Nothing is guessed: a field an operation cannot be judged without is refused, named on every line that lacks it, and a field the history does without is reported on stderr as a coverage gap. The document is recorded against the specification at `--path`; judge it with `check-history`.

With `--output FILE`, the coverage gaps are also written as a JSON array to `FILE.gaps.json`, always (the document seed is never carried, so the list is never empty). A history imported with gaps carries seed 0 and generated operation identities by construction; the gaps file is the record of which.

Exit 0: written, to `--output` or standard output. Exit 2: the specification, the adapter or the log was refused, or `--output` (or its gaps file) is the `--log` or `--adapter` file (a hard link included) or a file of the `--path` specification, or a write failed; nothing was written. The two files are written to temporary siblings and renamed into place only when both writes succeeded.

```text
ess verify conform import-history [OPTIONS] --log <LOG> --adapter <ADAPTER>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no | `.` | One ESS file, or a directory with `ess-inputs.yaml` or `system.yaml` |
| `--log` | `<LOG>` | yes |  | The JSON Lines log |
| `--adapter` | `<ADAPTER>` | yes |  | The `ess-history-adapter/1` document, YAML or JSON |
| `--output` | `<OUTPUT>` | no |  | Where to write the `ess-history/1` document; standard output when absent |

#### `ess verify diff`

Compare two revisions semantically

```text
ess verify diff [OPTIONS] --from <FROM> --to <TO>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--from` | `<FROM>` | yes |  |  |
| `--to` | `<TO>` | yes |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `json`. |
| `--compatibility` |  | no |  | Classify each change as breaking, unknown or compatible for callers, readers and history; JSON output is then `ess-diff/14` |
| `--fail-on` | `<FAIL_ON>` | no |  | Exit 4 when an unacknowledged change is at or above this level, 0 otherwise; a refused input or acknowledgements file still exits 1. Implies `--compatibility`. One of `breaking`, `breaking-or-unknown`. |
| `--dimension` | `<DIMENSION>`… | no |  | The dimensions `--fail-on` considers; repeatable. Default: all three. One of `callers`, `readers`, `history`. |
| `--acknowledgements` | `<ACKNOWLEDGEMENTS>` | no |  | An `ess-diff-acknowledgements/1` JSON file naming change ids `--fail-on` lets pass, bound to the `before` and `after` digests of this comparison |

#### `ess verify impact`

Report conformance and generated artifacts invalidated by a semantic change

```text
ess verify impact [OPTIONS] --from <FROM> --to <TO>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--from` | `<FROM>` | yes |  |  |
| `--to` | `<TO>` | yes |  |  |
| `--suite` | `<SUITE>` | no |  |  |
| `--suite-input` | `<SUITE_INPUT>` | no |  | Exact coverage input and complete original parent chain |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `json`. |

### `ess infra`

Read an observed cluster, and import a concrete source through a declared adapter

#### `ess infra infra diagnose`

Diagnose an observation or IR

```text
ess infra infra diagnose [OPTIONS] --path <PATH>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess infra infra graph`

Render the typed infrastructure dependency graph

```text
ess infra infra graph [OPTIONS] --path <PATH>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--namespace` | `<NAMESPACE>` | no |  |  |
| `--format` | `<FORMAT>` | no | `mermaid` | One of `dot`, `mermaid`, `json`, `yaml`. |

#### `ess infra infra diff`

Compare two infrastructure snapshots

```text
ess infra infra diff [OPTIONS] --from <FROM> --to <TO>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--from` | `<FROM>` | yes |  |  |
| `--to` | `<TO>` | yes |  |  |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `json`. |

#### `ess infra import kubernetes`

Import a sanitized observation bundle, or scan one live cluster at the credential edge

```text
ess infra import kubernetes [OPTIONS]
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | no |  | Existing `infra-observation/1`, `/2` or `/3` bundle |
| `--context` | `<CONTEXT>` | no |  | Live kubeconfig context. Requires `--observation-out` |
| `--namespace` | `<NAMESPACE>` | no |  | Safe topology of one namespace and its referenced nodes; emits qualified version 2 |
| `--observation-out` | `<OBSERVATION_OUT>` | no |  | Where a live scan writes its sanitized source bundle |
| `--out` | `<OUT>` | no |  | Where to write the compiled `infra-ir` document |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

#### `ess infra import openapi`

Import supported `OpenAPI` service, operation, and interface-type semantics

```text
ess infra import openapi [OPTIONS] --path <PATH>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  |  |
| `--out` | `<OUT>` | no |  | Where to write the replay-checked `ess-openapi-import/1` envelope |
| `--format` | `<FORMAT>` | no | `text` | One of `text`, `yaml`, `json`. |

### `ess ui`

Load, document and run renderer-neutral UI documents (`ess-ui/1`)

#### `ess ui load`

Load an `ess-ui/1` document and print what it holds, or name the node that refuses it

```text
ess ui load [OPTIONS] --path <PATH>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  | The `ess-ui/1` document to load |
| `--model` | `<MODEL>` | no |  | The ESS specification the document's `model:` names (a directory, its `ess-inputs.yaml`, or one file): a choice's `options` naming one of its enums list that enum's variants |

#### `ess ui check`

Check an `ess-ui/1` document, and with `--model` its references into an ESS model; each finding is named by node path. Exits 1 when any finding is an error

```text
ess ui check [OPTIONS] --path <PATH>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--path` | `<PATH>` | yes |  | The `ess-ui/1` document to check |
| `--model` | `<MODEL>` | no |  | An ESS specification (one file, or a directory with `system.yaml`) the document's views, commands and events must exist in |
| `--format` | `<FORMAT>` | no | `text` | Output rendering. One of `text`, `json`. |
| `--lacks` | `<CAPABILITY>`… | no |  | A capability the target renderer lacks (repeatable), such as `no_file_upload` |

#### `ess ui docs`

Render the `ess-ui/1` reference from its schema, as HTML or Markdown

```text
ess ui docs [OPTIONS] --out <OUT>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--out` | `<OUT>` | yes |  | The file to write (with `--check`, the file to compare) |
| `--format` | `<FORMAT>` | no |  | The format; inferred from the extension of `--out` (`.md` is Markdown) when absent. One of `html`, `md`. |
| `--schema` | `<SCHEMA>` | no |  | The schema to render; the schema this build embeds when absent |
| `--check` |  | no |  | Write nothing; fail when `--out` differs from a fresh render |

#### `ess ui run`

Run an `ess-ui/1` document, answering reads from its fixtures, or with `--model` reading and commanding the HTTP surface the specification serves

```text
ess ui run [OPTIONS] --tui --path <PATH>
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `--tui` |  | yes |  | Run in the terminal; the only renderer this command offers so far, so it must be named |
| `--path` | `<PATH>` | yes |  | The `ess-ui/1` document to run |
| `--fixtures` | `<FIXTURES>` | no |  | A fixture directory replacing the one the document names |
| `--model` | `<MODEL>` | no |  | The ESS specification the document's `model:` names (a directory, its `ess-inputs.yaml`, or one file): reads and commands go to the HTTP surface its `reached_by: network` components serve instead of the fixtures, and no fixture channel plays. The `Authorization` header is read from `ESS_UI_AUTHORIZATION`, never from the command line |
| `--base-url` | `<URL>`… | no |  | Where a served component is reached, `http://` only: `<url>` when the document binds one component, else `<component>=<url>`, once per component |

#### `ess ui test`

Run `ess-ui-test/1` tests headless against the terminal renderer, or with `--playwright` write them as a Playwright spec for the generated React project. Exits 1 when a test fails

```text
ess ui test [OPTIONS] --path <PATH> <TESTS>...
```

| Argument | Value | Required | Default | Description |
|---|---|---|---|---|
| `<TESTS>`… |  | yes |  | `ess-ui-test/1` files |
| `--path` | `<PATH>` | yes |  | The `ess-ui/1` document under test; every test file must name it |
| `--format` | `<FORMAT>` | no | `text` | Report format. One of `text`, `json`. |
| `--playwright` | `<OUT>` | no |  | Write the tests as a Playwright spec for the generated React project to this file, instead of running them |
| `--model` | `<MODEL>` | no |  | The ESS specification the document's `model:` names (a directory, its `ess-inputs.yaml`, or one file): a choice's `options` naming one of its enums list that enum's variants. Reads still come from the fixtures |

[ess-cli-end]: #
