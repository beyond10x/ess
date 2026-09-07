---
title: "ESS: executable system specifications"
sidebar_position: 5
description: The specification model, the compile pipeline, and the five things derived from one document — docs, contracts, tests, diffs and structural code.
---

# ESS: executable system specifications

An **Executable System Specification** describes a system semantically: `CreateInvoice` is a
command, and `POST /invoices/commands/create-invoice` is one way to expose it. That distinction is
the whole design. The same specification can compile to a modular monolith or to distributed
services without the domain model changing, and a generated test is a statement about the system,
not about its HTTP layer.

## The model

A specification — one YAML file or a directory — declares:

| Construct | What it is |
|---|---|
| **system** | the root: name, version, the domains it contains |
| **types** | newtypes, structs, enums, unions — with invariants (`amount >= 0`) that travel into every projection |
| **entities** | identity-bearing state with a lifecycle, invariants, and declared ownership/reference relations |
| **commands** | the only way state changes; each declares its input and its **outcomes** — including the refusal branches, which the model gives no way to omit |
| **events** | facts a command's outcome emits |
| **errors** | declared domain refusals |
| **views** | read models: what they show, filtered by what |
| **actors** | who may invoke which commands |
| **components** | units of ownership; `reached_by: network` states where callers are without naming a protocol |
| **bindings** | event → command reactions across contexts, including what happens when they fail |
| **topology** | each component's runtime requirements: replica bounds, statefulness and required resources |

Illegal lifecycle transitions are illegal **by absence**: there is no arrow, and no second rule
forbidding them, because two places for one truth eventually disagree. The generated documentation
lists the absent pairs explicitly, derived from the same transitions.

## Logical, interface and delivery owners

ESS uses several documents to connect logical ownership to physical implementations. The words
*component*, *service* and *workload* occur in more than one. The owning document tells you which
identity a name selects:

| Term | Owner | What it describes |
|---|---|---|
| **Logical component** | `ess-domain` component `ComponentSpec` | Domain ownership, accepted commands and published events. A component can become a module or a process without changing which domain it owns. |
| **Deliverable descriptor** | `ess-deployment` component `ComponentSpec` (`ess-component/1`) | Repository paths for specification, realization, build and runtime inputs, plus independent runtime and chart release units. Compiling this descriptor does not open its referenced files. |
| **Imported component alias** (composition service) | `ess-composition` `ServiceImportSpec` → `ResolvedService` | A local key selecting one component from an exact system, specification version and compiled-model digest. It does not select a deployment release. |
| **Stack member** (stack service) | `ess-deployment` stack `SystemRequirement` → `LockedSystem` | A stack-local service key whose runtime and chart release constraints are resolved against a release catalog. |
| **Runtime requirement** (semantic workload) | `ess-domain` topology `Topology` and `Workload` | One logical component's replica bounds, statefulness and required resources. These are correctness requirements, not process/container grouping or observed placement. |
| **Runtime workload mapping** | `ess-deployment` runtime `RuntimeSpec` and `Workload` | Desired mapping of logical components to processes, containers, replica counts and storage. Compilation checks input digests, component coverage, replica bounds and stateful storage; it does not establish resource provisioning or satisfaction of every semantic resource requirement. |
| **Implementation manifest** (realization) | `ess-realization` `RealizationSpec` (`ess-realization/1`) | Selected components and actors, immutable implementation artifacts and physical entrypoints. Each selected component has exactly one implementation assignment. |
| **Entrypoint prerequisite** | `ess-realization` `RuntimeRequirement` | Declared OS, architecture, filesystem, network, cgroup, environment-variable or credential-source requirements. Compilation does not verify their availability. |
| **Declared interface reach and CLI contract** | `ess-domain` `Reach` and `RawCommandLineSurface`, retained by `ess-compiler` | Where callers reach a logical component and its optional CLI binary/group layout. Command words and flags derive from model names and input fields. |
| **Invocation and attachment** | `ess-realization` `EntryPointSpec` and `Invocation` | One argv or HTTP(S) URL invocation per entrypoint, with attachment, interaction, availability and support metadata. |
| **Derived HTTP contract** | `ess-gen` HTTP `routes`, consumed by OpenAPI and HTTP synthesis | POST command routes and, for network-reached components, GET view routes. A projected contract alone establishes no listener, deployed URL or public availability. |

A deliverable can implement several logical components. Its descriptor identity, their component
names, a composition alias and a stack service key therefore need not coincide. See
[Component delivery](component-delivery.md) for the documents that connect release selection to
runtime mapping. Infrastructure observations remain in the separate infrastructure model.

### Reach, entrypoints and identity

`reached_by` has exactly three values: `in_process` (the default), `network` and `command_line`.
A `command_line` component must declare a `cli` block; a `cli` block with either other reach is
refused. These are interface contracts in the authored and compiled semantic model. Behaviour and
actor permissions remain in that model too. Physical invocation, attachment and support belong in
the separate [realization document](../guides/record-realization.md).

One logical component currently has one reach value. Simultaneous semantic CLI and HTTP surfaces
for that component have no combined reach value or independent interface collection. Duplicating
its domain owner is refused and cannot add a second surface. Current Rust and Go HTTP server
generation selects `network` components; the Clap target consumes the declared CLI layout.

A realization can describe multiple entrypoints, with unique IDs and exactly one primary; several
records may name the same implementation. This describes physical entrypoints without establishing
simultaneous CLI/HTTP synthesis. Realization compilation resolves references and invocation syntax;
it does not compare attachment or invocation with semantic reach, establish actor authorization,
or prove that an entrypoint runs.

The identity distinction matters when changing these documents. `EssIr::source_digest` hashes the
serialized compiled model, including nondefault reach and present CLI layout. Explicit default
`in_process` and an absent reach declaration serialize alike. Realization's `realization_digest`
hashes specification identity, synthesis identity and implementations; it excludes entrypoints.
An entrypoint-only edit can therefore leave that digest unchanged. Neither digest proves execution,
deployment or authorization.

Supporting several semantic interfaces in future would require a separate additive design: retain
one logical domain owner, define interface identities and references, decide model-identity and CLI
compatibility, and account for every admission, projection and synthesis consumer. Those criteria
do not add a multiple-interface capability to the current model.

### Reading the examples

The repository examples show which document owns a choice and where the supported boundary ends:

| Example | Owner and supported reading |
|---|---|
| **CLI-only: desk-service** | The [Clap fixture][cli-example] declares `reached_by: command_line`, binary `desk` and group `visits`. Its tests inspect generated grammar bytes. The Clap emitter provides parsing, completion and handler seams receiving `ArgMatches`; command behaviour remains implementation work. |
| **HTTP-only: Gatepass** | [pass-service][http-component] declares `network`; the [server source][http-server] links the handwritten realization to the generated HTTP server. The native executable has no argument parser and is not a semantic CLI. This source describes the linkage, not a currently running deployment. |
| **Simultaneous CLI and HTTP** | The single `Reach` value and CLI validation above are the current limitation. Multiple realization entrypoint records do not establish combined semantic reach or combined synthesis for one component. |
| **Physical invocation: billing-local** | The [realization example][realization-example] assigns `invoice-service` to `billing-binary` and describes `local-tui` through argv plus an environment prerequisite. Its artifact locator and identity are fixture values; the document does not establish an available implementation or a declared CLI grammar. |
| **Composition: Todo/Usage** | The [composition fixture][composition-example] selects `todo-component` and `usage-component` from the same exact workbench model under different aliases. It selects logical surfaces, not two independently resolved releases. |
| **Delivery: Oracle** | The [delivery fixture][delivery-example] defines one deliverable with runtime/chart release units and maps `order-service` and `dispatch-service` into one runtime workload. Logical ownership remains separate. Its stack selects releases from a catalog; this is compiler/projection example data, not a live deployment. |

Composition's current client emitter produces Rust with byte-buffer transport. Its plan preserves
selected operation names and model identity, without complete payload definitions or codecs; named
type traversal does not cover view parameters. The language-neutral plan does not promise a client
for every synthesis target. See [Composition clients](../reference/cli.md#composition-clients-selected-operations-and-byte-transport)
for the application-owned payload and transport boundary.

Stack resolution takes a stack and a release catalog. It carries the supplied `composition_digest`
without reading a compiled composition, so it does not prove correspondence with imported aliases
or complete typed operation-payload compatibility. Structural synthesis also leaves semantic
topology as `TopologyDeferred`; deployment runtime compilation is the separate owner of the
desired workload mapping.

[cli-example]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/tests/clap.rs
[http-component]: https://github.com/beyond10x/ess/blob/main/examples/gatepass/components.yaml
[http-server]: https://github.com/beyond10x/ess/blob/main/examples/gatepass-realization/src/bin/gatepass-server.rs
[realization-example]: https://github.com/beyond10x/ess/blob/main/examples/realizations/billing-local.yaml
[composition-example]: https://github.com/beyond10x/ess/blob/main/crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml
[delivery-example]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-deployment/tests/deployment.rs

## The pipeline

```text
source ──validate──► consistent?  ──compile──► normalized IR ──┬─► generate   docs, HTML site, JSON Schema, OpenAPI, AsyncAPI
                                                               ├─► conform    scenario suite → runner → evidence
                                                               ├─► diff       semantic delta between two revisions
                                                               ├─► impact     what the delta invalidates
                                                               └─► synthesize language-neutral plan → Rust / Go / browser / Clap code
```

`validate` answers "is this document consistent" and reports every problem in one run. `compile`
resolves every name to what it points at — in the IR, an unresolved reference is unrepresentable.
Everything downstream consumes the IR, and compiling the same source twice is byte-identical.

## What gets derived

### Projections (`ess generate`)

| Kind | Output | Why it exists |
|---|---|---|
| `docs` | Markdown with Mermaid diagrams | the cheapest completeness check: a construct with no rendering is a hole in a page a person reads |
| `site` | HTML pages, navigation, a local stylesheet and Mermaid assets | browse model-derived pages and explicitly selected authored pages/downloads; hosting remains external |
| `schema` | JSON Schema for command inputs, messages, named types, and entities | the type system, projected without losing its distinctions — newtypes stay separate definitions |
| `openapi` | one OpenAPI 3.1 document per component | the specification *is* the HTTP contract |
| `asyncapi` | one AsyncAPI 3.0 document per component | the same for messaging, including what happens when a binding fails |

Every artifact carries provenance: specification version, a digest of the resolved model, and the
digest of the model *slice* it derives from (`contract_digest`). Committed output is drift-checked
in CI. See [the worked example](../examples/specification-to-contracts.md) for real input and
output side by side.

The arrow is one-way: the typed ESS YAML is the specification, and the documentation is one
projection of it. ESS does not infer model semantics from an existing Markdown document. The
`site` projection renders HTML. Explicit `--kind site --out DIR` writes `index.html` and local
assets at that output root; combined generation puts them under `DIR/site/`. Authored pages and
downloads require explicit selection, and hosting remains external. The five default projections
above exclude the opt-in `--kind docs-ir`, which writes `docs-ir/document.json` carrying `ess-docs/1`.
See the [current-source support matrix](../status/where-this-stands.md#support-boundaries) for output
checks and the separate dated release observation.

### The conformance suite (`ess verify conform`)

The specification acts as an oracle: `synthesize` derives one scenario per obligation the
specification states — each declared outcome, each lifecycle move, each move that must *not* be
honoured, each invariant, each binding claim. `run` executes them against an implementation;
the standalone report carries the result across a repository boundary without importing workflow
or planning semantics.

A construct the specification does not say enough about to test is **refused, not omitted** — the
refusal prints beside the scenarios, because a suite quietly holding fewer checks than the
specification requires is the one failure a passing run cannot show. See
[Verify an implementation](../guides/verify-conformance.md).

### The semantic delta (`ess verify diff`, `ess verify impact`)

`diff` compares two compiled revisions: moving declarations between files, renaming files,
reordering blocks and rewriting comments report **nothing**; removing a currency variant reports one
narrowing. `impact` computes what stood on what moved — which conformance scenarios are owed again
and which generated artifacts are owed regeneration, each with the hop-by-hop dependency path that
explains it. It narrows what a change owes; it never claims a result still holds. See
[Track specification change](../guides/track-change.md).

### Structural synthesis (`ess generate synthesize`)

A language-neutral **synthesis plan** gives every capability of the specification exactly one
disposition: *generated*, *obligation* (a named piece of work a human must implement — every
algorithm is one), or *refused* (with the reason). Four targets render the plan: a Rust workspace,
a Go module, a WebAssembly browser bridge, and a Clap command grammar with completion support and
handler seams. The language-neutral plan travels with each tree; target-specific weakenings and
refusals are recorded separately. Clap handlers receive `clap::ArgMatches`, and its generated crate
depends on `clap` and `clap_complete` 4. Full synthesis refuses modeled Binary64 across all four
targets; the separate structural data libraries have their own support boundary.

Behaviour is **never** generated. The generated billing workspace, linked with the hand-written
realization of its eight obligations, passes the committed 29-scenario suite unchanged — and a
deliberately corrupted linkage fails exactly the scenario that exists to catch it. See
[Synthesize code from a specification](../guides/synthesize.md).

## The same pattern, pointed somewhere else

The pipeline shape — observe, normalize into a content-addressed IR, declare a desired state, judge
three-valued — is reused twice more, which is the strongest evidence available that it is a shape
and not a special case.

**Infrastructure.** The `infra-*` crates read an observation bundle from an external scanner and
compile it to a content-addressed IR; a typed graph and twenty coded diagnosis rules read it; a
declared desired state (`infra-spec/1`, twelve expectation kinds) evaluates against a snapshot; and
a gap projects back as a reviewable patch tree in which every value either came from the gap or is a
named obligation for a human. Nothing reaches a cluster. See
[Check infrastructure](../guides/check-infrastructure.md).

---

**Sources.** `crates/specify/ess-domain/` through `crates/generate/ess-synth/`;
`crates/verify/ess-diff/src/lib.rs` (the construct families); `crates/infra/infra-spec/src/spec.rs`;
`examples/billing/` (the normative
specification); `suites/generated/*/suite.json` (the scenario counts);
`generated/rust/billing/PLAN.md`; `CHANGELOG.md` §§ *0.4.0*, *0.5.0*.
