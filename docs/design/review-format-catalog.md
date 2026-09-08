# ESS format identities and canonical bytes

Status: inventory for `story:review-format-catalog`; this document changes no format or reader.
Source baseline: `4b66aac7b608b1deee9de88942390d4a6c5ec745` (production through
`acb7859e3202ffdc1ca840dde67f7ca4da33c746`). Source citations identify implementations, not a
released or deployed consumer inventory. The Rust/Web checked APIs and target-failure entries
additionally use frozen producer `f9a7cf7fcca79448a34b2754adb12f1a411573bd`.

## Reading the catalog

A discriminator selects a document contract. It is independent of the specification's major-only
`vN`, a delivery release's SemVer, and the ESS package version. `Version` displays `vN` but
serializes as a numeric major in typed IR; `FormatVersion` parses the syntax `ess/N` before
validation checks the supported set `[1]`. Parsing a well-shaped version is not support for it.
See [specification formats](../../crates/specify/ess-domain/src/system.rs) and
[major versions](../../crates/specify/ess-domain/src/name.rs).

The tables distinguish an authored DTO from its validated result and its machine rendering.
“Closed” means unknown DTO fields are refused; it does not mean every cross-document claim was
verified. “No reader” means no corresponding maintained ESS document-admission entry point was
found, not that another program cannot parse the JSON. JSON/YAML presentation alone does not add a
version or a hash contract. The public reference is [Formats](../../website/docs/reference/formats.md).

Canonical layout notation:

| Code | Existing writer contract |
|---|---|
| P | Typed `serde_json::to_string_pretty` plus one LF; struct field order and ordered maps are significant. |
| K | First convert to `serde_json::Value`, then pretty JSON plus LF; object keys are sorted. |
| C | Compact JSON with no appended LF. Typed serialization and serialization through a sorted `Value` are distinguished below. |
| Y | The indicated YAML writer; do not substitute it for canonical JSON when hashing. |
| — | No document-level canonical writer/digest is promised by this surface. |

## Specification, composition and realization

| Surface / discriminator | Separate version or identity | Producer; reader and admission | Canonical bytes / digest |
|---|---|---|---|
| Directory acquisition configuration: `format: ess-inputs/1` | Immediate `ess-inputs.yaml`; no independent identity or digest | Author → [package-local input discovery](../../crates/edge/ess-cli/src/input_discovery.rs). Closed required format/specification/scenarios fields, duplicate-key and path admission; only the active list receives regular-file, symlink, containment and canonical-target checks. No ancestor search, implicit scenario selection or fallback after invalid configuration. [Contract](review-authored-discovery.md). | —; original selected source text and existing role identities are preserved; configuration is absent from IR/suite provenance. |
| Authored specification: `format: ess/1` | Specification `version: vN`, distinct from ESS release | Author → `RawSpecFile::parse` (including duplicate-key rejection) → `Specification::assemble`/validation → compiler resolution. Format syntax admits future positive majors; validation supports only 1. [Owners](../../crates/specify/ess-domain/src/spec.rs), [support](../../crates/specify/ess-domain/src/system.rs). | Authored YAML/JSON has no raw-source canonical hash. Compilation determines semantic identity. |
| Compiled `EssIr`: **unversioned**, no `ess-ir/1` marker | Numeric specification major inside typed IR | `ess_compiler::compile` → `EssIr`; Serialize-only, with no general persisted-IR reader. [Owner](../../crates/specify/ess-compiler/src/ir.rs). | P for `to_canonical_json`; `source_digest()` hashes **C typed IR**, not those pretty bytes and not YAML. Bare lowercase SHA-256. |
| Authored composition: `format: ess-composition/1`, `services` array | Composition/service keys and exact supplied specification identities | `CompositionSpec::{from_json,from_yaml}` is closed DTO parsing; `compile` separately checks the marker, duplicates, references and supplied service identity. [Owner](../../crates/specify/ess-composition/src/lib.rs). | P via `to_canonical_json`; imported `source_digest` values identify compiled services. No independent composition hash API. |
| Compiled composition: `format: ess-composition/1`, `services` map | Same key space, resolved service surfaces | `compile` → `EssCompositionIr`; Serialize-only. The authored reader is **not** its reader, despite the shared marker. [Owner](../../crates/specify/ess-composition/src/lib.rs). | P; exact imported source digests, no whole-document hash API. |
| Client plan: `format: ess-client-plan/1` | Composition key, provider bindings and exact service identities | `EssCompositionIr::client_plan` → `EssClientPlan`; Serialize-only; consumed directly by client generation. [Owner](../../crates/specify/ess-composition/src/lib.rs). | P; service source digests are references, not a digest of client-plan bytes. |
| Authored realization: **`type: ess-realization/1`** | Realization id, specification and synthesis identities | Closed `RealizationSpec` JSON/YAML DTO → `compile` against supplied `EssIr`, including synthesis authority checks. DTO parsing alone is insufficient. [Owner](../../crates/specify/ess-realization/src/lib.rs). | Authored layout is not canonical identity; artifact and conformance evidence digest fields are references. |
| Compiled realization: **`type: ess-realization-ir/1`** | Same referenced identities; `realization_digest` | `compile` → Serialize-only `RealizationIr`; downstream runtime compilation consumes the typed result. [Owner](../../crates/specify/ess-realization/src/lib.rs). | P output; realization digest is `sha256:` over C of the **specification/synthesis/implementations tuple**, not the entire output document. |

## Source-driven integration additions

The output-ownership wave introduces the private `ess-output-state/1` reader/writer at the CLI
edge. Its selected [binding](review-output-ownership.md) defines canonical sorted-key JSON plus
LF, native `UnixBytes1` components, fixed owner inventory, root binding, transaction decisions and
checksum coverage. Strict admission rejects malformed/noncanonical/unknown state and preserves
the evidence. This format is independent of all generated artifact formats and their digests;
an older ESS generator does not participate in its ownership or recovery contract. The public
[format reference](../../website/docs/reference/formats.md#generated-output-state) records the same
boundary. Runtime and native platform validation remain owed at wave opening.

This section reconciles the catalogue with the schema/data work and the frozen Rust feasibility
implementation. It describes the combined integration preview, not an ESS release or a migrated
external consumer. The public [format reference](../../website/docs/reference/formats.md) carries
the same additional identities.

Normalization `/2` below is additionally pinned to source
`f0cbf56a1e3985a08effffc88a3e7f5b17893a9b`, whose public format reference marks it unreleased.

| Surface | Admission and canonical identity |
|---|---|
| `ess-schema-bundle/1` | Component selection; `Bundle::read` reimports exact retained source and compares the full typed envelope. P bytes. |
| `ess-schema-bundle/2` | Explicit document-root identity plus local definition closure; the replay reader rejects /1 carrying a document root. P bytes; existing /1 bytes are unchanged. |
| `ess-types-report/3` | Serialize-only data-target accounting, distinct bundle/model input variants and explicit target configuration. P bytes; no report-byte hash or persisted reader. |
| `ess-normalization/1` | Closed recipe DTO plus `Plan::read`/`check` against supplied checked bundles. Branches, root pins, stage composition and expressions require semantic checking. P bytes. |
| `ess-normalization/2` (unreleased at this source) | Same closed recipe DTO and `Plan::read`/`check` boundary; adds ordered text/list construction, original item indices, filtered mapping, first-match selection and explicit binary64 input/conversion declarations. P bytes. |
| `ess-normalization-target/1` | Serialize-only standalone Rust adapter report from either checked recipe version; its /1 is independent of the recipe version and application synthesis. P bytes; no persisted report reader. |
| `ess-target-failure/1` | Serialize-only complete Rust/Web synthesis refusal: unchanged neutral plan, ordered typed causes, no artifacts. P bytes; no independent digest. Unversioned `target.json` still identifies the separate optional weakening report. |

Owners: [bundle](../../crates/generate/schema-contract/src/bundle.rs),
[data realization](../../crates/generate/schema-contract/src/realize.rs),
[normalization](../../crates/generate/schema-contract/src/realize/normalize.rs),
[normalization target](../../crates/generate/schema-contract/src/realize/normalize/target.rs),
[synthesis refusal](../../crates/generate/ess-synth/src/failure.rs).

The `/1` checker refuses `/2` operations and any `binary64_inputs` declaration, including `{}`;
an explicit `null` fails DTO parsing. When that field is omitted, typed serialization retains
the existing `/1` field order without emitting the new field, and raw-JSON input follows the
existing numeric-admission path.
[Recipe](../../crates/generate/schema-contract/src/realize/normalize/recipe.rs),
[version checks](../../crates/generate/schema-contract/src/realize/normalize/check.rs),
[input decoding](../../crates/generate/schema-contract/src/realize/normalize/input.rs).

All new byte hashes are bare lowercase SHA-256. They are not interchangeable:

- Source identity hashes exact retained UTF-8 source, without normalization.
- Bundle identity hashes complete P bytes, including source, selected roots and dialect.
- Model data input retains the compiler/provenance-owned source and contract digests; its
  projection digest separately hashes `ModelTypes::to_json()` K bytes, including provenance.
- Recipe identity hashes checked `Plan::to_json()` P bytes.
- Target schema and file digests hash exact emitted bytes. The target report excludes itself
  from its file map; no circular whole-output digest is claimed.

Canonical bytes include the writer's final LF where P or K specifies it. Digest syntax alone
does not attest runtime equivalence, external artifact authenticity or a deployed reader upgrade.

## Independent component delivery

All discriminators in this table use `format`. Authored DTO readers are closed but defer semantic
compilation. In contrast, the eight persisted types `BuildIr`, `RuntimeIr`, `ComponentIr`,
`ReleaseManifest`, `ReleaseBundle`, `ReleaseCatalog`, `StackLock` and `DeploymentIr` use
[checked generic Deserialize](../../crates/generate/ess-deployment/src/validation.rs), including
nested admission. Their public `validate` methods recheck mutable values; decoding alone does not
establish absent ESS/realization authority, registry contents, signatures or external trust.

| Surface / discriminator | Separate version or identity | Producer; reader and admission | Canonical bytes / digest |
|---|---|---|---|
| Build input: `ess-build/1` | Build/system identities and referenced semantic content | `BuildSpec` JSON/YAML → `compile_build`; closed DTO first, DAG/output semantics at compile. [Owner](../../crates/generate/ess-deployment/src/build.rs). | Authored bytes: —; no input-file digest contract. |
| Build IR: `ess-build-ir/1` | Compiled build and semantic identity | `compile_build`; checked `BuildIr` Deserialize/from_json/validate; executor and runtime compilation consume it. [Owner](../../crates/generate/ess-deployment/src/build.rs). | P; `digest()` = `sha256:` + SHA-256(P bytes). |
| Runtime input: `ess-runtime/1` | Exact ESS, realization and build references | `RuntimeSpec` JSON/YAML → `compile_runtime` with authority inputs; closed DTO first. [Owner](../../crates/generate/ess-deployment/src/runtime.rs). | Authored bytes: —. |
| Runtime IR: `ess-runtime-ir/1` | Compiled runtime and referenced semantic/build identities | `compile_runtime`; checked `RuntimeIr` Deserialize/from_json/validate; `validate_against_build` adds supplied-build checks. [Owner](../../crates/generate/ess-deployment/src/runtime.rs). | P; prefixed SHA-256(P). |
| Component input: `ess-component/1` | `semantic_version` and independently named release units | `ComponentSpec` JSON/YAML → `compile_component`; closed DTO, then semantics. [Owner](../../crates/generate/ess-deployment/src/component.rs). | Authored bytes: —. |
| Component IR: `ess-component-ir/1` | Component and semantic version; release-unit names are not release versions | `compile_component`; checked `ComponentIr` Deserialize/from_json/validate. [Owner](../../crates/generate/ess-deployment/src/component.rs). | P; prefixed SHA-256(P). |
| Release manifest: `ess-release/1` | **Release SemVer**, source commit, exact semantic/build/runtime/artifact identities | Executor-produced record; checked `ReleaseManifest` JSON/YAML/Deserialize, local `validate`, and `verify_release` against supplied build/runtime. [Owner](../../crates/generate/ess-deployment/src/release.rs). | P; prefixed SHA-256(P). Attachment digests do not prove attachment authenticity. |
| Release bundle: `ess-release-bundle/1` | Component and independently versioned runtime/chart release identities | `bundle_release`; checked JSON/YAML/Deserialize and `verify_release_bundle`. OCI publish/fetch uses this payload, with a separate registry manifest digest. [Owner](../../crates/generate/ess-deployment/src/component.rs). | P; prefixed SHA-256(P), distinct from the OCI manifest digest. |
| Release catalog: `ess-release-catalog/1` | Candidate semantic versions and nested release SemVers | Offline populated DTO → checked `ReleaseCatalog` JSON/YAML/Deserialize/validate → stack resolver. [Owner](../../crates/generate/ess-deployment/src/stack.rs). | No dedicated canonical writer or whole-catalog digest API; nested releases/runtime carry their identities. |
| Stack input: `ess-stack/1` | Semantic-major constraints plus runtime/chart **SemVer requirements** | Closed `StackSpec` JSON/YAML → `resolve_stack` with a release catalog. [Owner](../../crates/generate/ess-deployment/src/stack.rs). | Authored bytes: —. |
| Stack lock: `ess-stack-lock/1` | Exact selected release versions and digests | `resolve_stack`; checked `StackLock` Deserialize/from_json/validate; deployment compiler consumes it. [Owner](../../crates/generate/ess-deployment/src/stack.rs). | P; prefixed SHA-256(P). |
| Environment input: `ess-environment/1` | Environment id and exact stack-lock digest | Closed `EnvironmentSpec` JSON/YAML → `compile_deployment` with supplied lock. Private coordinates and Secret references stay environment-owned. [Owner](../../crates/generate/ess-deployment/src/environment.rs). | Authored bytes: —; no implicit whole-input digest. |
| Deployment IR: `ess-deployment/1` | Exact stack identity and independent release artifacts | `compile_deployment`; checked `DeploymentIr` Deserialize/from_json/validate; explicit reconcile consumes admitted documents. [Owner](../../crates/generate/ess-deployment/src/environment.rs). | P; prefixed SHA-256(P). |
| Deployment diff: `ess-deployment-diff/1` | `from`/`to` deployment digests, no own release version | CLI compares admitted deployments and writes a JSON object with added/changed/removed service sets; no report reader. [Owner](../../crates/edge/ess-cli/src/main.rs). | K stdout; referenced deployment digests, no diff digest. |

`Digest` admits exactly `sha256:` plus 64 lowercase hexadecimal characters. The
[canonical delivery helper](../../crates/generate/ess-deployment/src/identity.rs) hashes its caller's
bytes; only the individual `digest()` methods establish which bytes those are. OCI references,
payload identity and evidence attachment identity must remain separate.

## Change and conformance

| Surface / discriminator | Separate version or identity | Producer; reader and admission | Canonical bytes / digest |
|---|---|---|---|
| Legacy delta: `format: ess-diff/1` | Before/after system major and compiled-model digests | Explicit legacy serialization retains frozen vocabulary/order/bytes and refuses v2-only changes. `RawEssDelta` is closed; conversion to `EssDelta` validates support, same-system identity, ids, relations, ordering and uniqueness. [Delta](../../crates/verify/ess-diff/src/delta.rs), [reader](../../crates/verify/ess-diff/src/raw.rs). | P; endpoint source digests, no delta hash API. |
| Current delta: `format: ess-diff/2` | Same endpoint identities | Default `diff` output; supported majors exactly `[1,2]`. Serialization also checks the selected version, so mutable content cannot be relabeled as /1. Same raw-to-validated reader boundary. [Owner](../../crates/verify/ess-diff/src/delta.rs). | P with v2 vocabulary and residual `unclassified-changed`; no new endpoint digest domain. |
| Impact: `format: ess-impact/3` | Embedded /2 delta, optional suite provenance, artifact identities | `ess_diff::impact` returns `EssImpact`; no report reader. Graph vocabulary has 26 relations, including `row-shape` and actual CLI/network domain view exposure. CLI passes no `GeneratedTree`. [Owner](../../crates/verify/ess-diff/src/impact.rs), [graph](../../crates/specify/ess-compiler/src/graph.rs), [CLI](../../crates/edge/ess-cli/src/main.rs). | P; references source/contract digests. A report does not hash itself or attest surviving results. |
| Authored scenario: **`type: ess-scenario/1`** | Domain, scenario name and purpose; no release version | Closed authored `Document` parsing followed by authored compilation against IR; exact marker and semantic checks belong to that compilation. [Owner](../../crates/verify/ess-conformance/src/authored.rs). | Authored YAML/JSON: —; generated suite records its result. |
| Suite: **`provenance.suite_version: ess-conformance/4`** | Specification `vN`, `spec_digest`, whole-model `contract_digest`; no exact-suite digest | Synthesizers/authored compiler → `ConformanceSuite`; ordinary derived Deserialize/from_json. The historical raw DTO parser remains unadmitted. Rust `AdmittedSuite` and generated Go validate original version-specific structure before execution; Rust typed entry points serialize once and admit that value before identity/callbacks. [Suite](../../crates/verify/ess-conformance/src/scenario.rs), [runner](../../crates/verify/ess-conformance/src/runner.rs), [Go](../../crates/verify/ess-conformance/src/go/runtime.go). | P; both provenance digests remain bare hashes. The unchanged DTO has no embedded complete-suite digest or durable complete coverage. Separate report/2 binds exact admitted bytes; raw DTO parsing alone makes no original-byte admission claim. |
| Rust standalone report: `format: ess-conformance-report/1` | Specification/model digest, implementation, `suite_version` | `ConformanceReport::standalone`; checked closed generic Deserialize/from_json validates marker, supported suite major, counts/list and status. It does not verify exact-suite coverage or uniqueness of opaque failure ids. [Owner](../../crates/verify/ess-conformance/src/evidence.rs). | P; `completed_at` is `Timestamp(u64)`. `spec_digest` is the compiled-model reference; no report hash API. |
| Go standalone report: `format: ess-conformance-report/1` | Same fields; producer-specific outcome vocabulary | Generated Go runtime writes the report; ESS Rust report reader admits the documented non-pass vocabulary. Go counts failed/skipped; Rust also has error/unsupported. [Writer](../../crates/verify/ess-conformance/src/go/runtime.go), [reader](../../crates/verify/ess-conformance/src/evidence.rs). | Go `json.MarshalIndent` + LF; **`completed_at` is int64**, not the Rust u64 domain. Same marker does not establish byte-for-byte producer equivalence. |
| Detailed run: **unversioned** `ConformanceReport` | Suite provenance, implementation, run/scenario identities and timestamps | Rust runner → detailed CLI JSON/YAML; distinct from the standalone JSON written by `--report-out`. Serialize-only legacy detailed result; the default remains unversioned. [Report](../../crates/verify/ess-conformance/src/report.rs), [CLI](../../crates/edge/ess-cli/src/main.rs). | P canonical method / CLI JSON, Y presentation; referenced suite/model identities, no exact-suite/report digest. |

## Generated documents, plans and external dialects

| Surface / discriminator | Separate version or identity | Producer; reader and admission | Canonical bytes / digest |
|---|---|---|---|
| Documentation IR: `format: ess-docs/1` | System/specification version, **per-page** provenance | `ess_gen::docs::document`; ordinary `Document`/page Deserialize, with separate `validate_page_ids` and renderer destination checks. No uniform marker/digest-profile validation is implied by decoding its String fields. [Document](../../crates/generate/ess-gen/src/document.rs), [producer](../../crates/generate/ess-gen/src/docs.rs). | P in CLI docs-ir output; pages carry whole/sliced contract digests. No single document stamp; `read_digests` deliberately refuses to choose a page. |
| Browser catalog: `format: ess-browser-catalog/1` | System/specification version and plan provenance | Web synthesis writes the catalog; generated browser code consumes its data. No general ESS catalog-admission API. [Writer](../../crates/generate/ess-synth/src/web/catalog.rs), [browser](../../crates/generate/ess-synth/src/web/page.rs). | K; whole-model provenance, no catalog-content hash. |
| Imported interface: `format: ess-service-interface/1` | `source_openapi` version and independent `service.version` | Retained `ServiceInterface` DTO, then explicit `validate` for format and interface invariants. Its legacy integer, number and boolean unit variants discard unknown fields during deserialization. Structural data alone has no durable import-accounting claim; legacy files require original-source reimport for checked CLI projection. [Owner](../../crates/generate/ess-openapi/src/lib.rs). | Unchanged P; no interface digest API. |
| OpenAPI import: `format: ess-openapi-import/1` | Fixed `ess-openapi-service-subset/1` profile, admitted schema dialect and retained source version | Private closed DTO → `read_import`: duplicate-key preflight, retained-source reimport and comparison of the complete interface/accounting/identity. Sealed `ImportReport`, read-only accessors; old interface readers reject this wrapper. [Owner](../../crates/generate/ess-openapi/src/accounting.rs). | P; `source.sha256` is bare lowercase SHA-256 of exact retained UTF-8 source, not compiled-model identity. No redundant whole-envelope digest field. |
| Synthesis `plan.json`: **unversioned** `SynthesisPlan` | Specification provenance; no target-specific semantic version | `SynthesisPlan::of` / synthesis → Serialize-only neutral plan; target emitters consume the typed plan. [Owner](../../crates/generate/ess-synth/src/plan.rs). | P and `PLAN.md`; source/whole provenance, not a hash of the plan file. |
| Synthesis `target.json`: **unversioned** `TargetReport` | Specification provenance plus target name | Successful Go/Web/Clap synthesis includes this Serialize-only report of refusals/weakenings; successful Rust has `target: None` and no target metadata. No persisted report reader. [Owner](../../crates/generate/ess-synth/src/lib.rs). | Unchanged P and `TARGET.md`; source/whole provenance, no target-report digest. Partial reports remain successful values. |
| Complete target failure: `format: ess-target-failure/1` | `target: rust` or `web`; unchanged neutral `plan` with specification provenance | Checked synthesis/Rust/Web APIs return private-constructed, Serialize-only `TargetFailure`: `format`, `target`, `plan`, nonempty `causes`. No Deserialize or persisted admission reader. [Envelope](../../crates/generate/ess-synth/src/failure.rs), [facade](../../crates/generate/ess-synth/src/lib.rs). | P via `to_canonical_json`; CLI also presents Y/text. Causes and their nonempty source identities are sorted/deduplicated. Plan provenance is referenced; no failure-document digest or artifacts. |
| ESS contract JSON Schema: `$schema: https://json-schema.org/draft/2020-12/schema` | ESS specification version/provenance; no generated root `$id` | `ess-gen` schema/types projection; external JSON Schema consumers, not a new ESS envelope. [Schema](../../crates/generate/ess-gen/src/schema.rs), [writer](../../crates/generate/ess-gen/src/types.rs). | Deterministic pretty JSON+LF; source/sliced provenance. An adopter-owned copy may add a resource ID, independently of those digests. |
| ESS-generated OpenAPI: `openapi: 3.1.0` | `info.version` is specification `vN`; `x-ess-provenance` is separate | `ess-gen::openapi`, also embedded by HTTP synthesis; consumers are OpenAPI tooling. [Owner](../../crates/generate/ess-gen/src/openapi.rs). | Deterministic YAML with comment plus structured provenance, or pretty JSON+LF for HTTP output; source/sliced digests. |
| Imported-interface OpenAPI projection: `openapi: 3.1.0` | Interface `service.version`, retained source version separate | CLI `--ir` uses `read_import` and `project_import`; semantic gaps, unresolved sites or legacy accounting-unavailable input refuse before output. Known annotation omissions may pass. The retained `ess_openapi::project` API is an unqualified structural projection; native `ess-gen` is separate. [Owner](../../crates/generate/ess-openapi/src/lib.rs), [checked admission](../../crates/generate/ess-openapi/src/accounting.rs). | Unchanged Y from ordered JSON Value; no ESS semantic digest is invented for imported input. |
| AsyncAPI: `asyncapi: 3.0.0` | `info.version` is specification version | `ess-gen::asyncapi`; external AsyncAPI consumers. [Owner](../../crates/generate/ess-gen/src/asyncapi.rs). | Deterministic Y with comment and structured provenance; source/sliced digests. |
| ESS source schema: `$schema: http://json-schema.org/draft-07/schema#` | Schema describes `RawSpecFile`, including format/version syntax | `cargo xtask schema` derives Schemars schema; editors/schema validators consume it. It does not run whole-system validation or resolution. [Owner](../../crates/edge/ess-xtask/src/main.rs), [fixture](../../schemas/generated/ess.schema.json). | P; committed schema is compared as complete bytes; no embedded semantic digest. |
| Generated Rust HTTP startup records: `log: ess/1` | Specification/contract and runtime address/port/language | Rust HTTP emitter builds facts; generated server completes and prints JSON lines. This is **not authored specification input**. [Owner](../../crates/generate/ess-synth/src/rust/http.rs). | Compact runtime JSON lines; dynamic runtime facts, no whole-record canonical hash. |
| Generated Go HTTP startup records: `log: ess/1` | Same shared startup facts, Go runtime fields | Go emitter reuses Rust `startup_facts`; generated Go server prints records. No ESS log-reader contract. [Owner](../../crates/generate/ess-synth/src/go/http.rs). | Compact runtime JSON lines; no whole-record hash or cross-language byte-equivalence promise. |

The contract and source-syntax generators do not assign registry IDs. The accepted
[resource-identity binding](review-schema-resource-identity.md) uses separate adopter-owned copies
with only a root `$id` added, plus strict application selector envelopes referencing those copies.
The copies retain their respective dialects, root-local references, constraints and provenance;
the original generated bytes remain unchanged. A successful registry report identifies the selected
envelope, not every referenced payload. The registry resolves supplied resources offline and checks
exact ID collisions within that invocation. It does not establish an organization namespace,
historical immutability or a public endpoint. The executable
[public workflow](../../website/docs/guides/generate-artifacts.md#generated-schemas-in-a-local-registry)
also distinguishes syntax acceptance, semantic assembly and restricted TypeScript projection.

### Checked synthesis APIs and failure limits

| Public call | Current result |
|---|---|
| `ess_synth::synthesize(ir)` and `synthesize_for(ir, target)` | `Result<Synthesis, TargetFailure>`; `synthesize` selects Rust. [Facade](../../crates/generate/ess-synth/src/lib.rs). |
| `ess_synth::rust::workspace(ir, plan)` | `Result<Vec<Artifact>, TargetFailure>`; allocation/representation checks precede rendering. [Rust](../../crates/generate/ess-synth/src/rust/mod.rs). |
| `ess_synth::web::workspace(ir, plan)` | `Result<web::Emission, TargetFailure>`; checks the Rust prerequisite and actual Web codec allocation. [Web](../../crates/generate/ess-synth/src/web/mod.rs). |

Each cause carries `code`, nonempty sorted unique `sources`, and nonempty human-readable `detail`.
The current closed kebab-case codes are `invalid-identifier`, `symbol-collision`, `path-collision`,
`recursive-layout`, `binding-assignment`, `missing-type-owner`, `wire-collision` and
`missing-representation`. A failure can carry a plan with zero capabilities; causes are independent
of capability accounting. The error implements `Display` and `std::error::Error` with read-only
accessors. [Error type](../../crates/generate/ess-synth/src/failure.rs).

`Err` returns no `Synthesis`, code/manifest vector or diagnostic artifact tree. It withholds the
whole requested workspace, including individually feasible modules. Successful `Synthesis` retains
its typed `plan`, `artifacts` and optional `target` fields; that container has no own Serialize
envelope. Neutral plan bytes and successful/partial `TargetReport` bytes retain their contracts.
The direct APIs expect the plan for the supplied IR; a fabricated mismatched plan is outside the
compiler-admitted-input guarantee. Internal coverage assertions remain, so the finite checks are
not a universal compiler proof or a promise that programming defects cannot panic.
[Facade](../../crates/generate/ess-synth/src/lib.rs), [decision](review-rust-target-feasibility.md).

On target failure the CLI writes text or the JSON/YAML envelope to stdout, then exits 1 before
`write_artifacts`: it creates no output directory and leaves an existing destination untouched.
Successful text output still counts the neutral plan; partial target notes remain in
`TARGET.md`/`target.json`. This early refusal does not add rollback for later I/O failures.
[CLI](../../crates/edge/ess-cli/src/main.rs).

`web::browser_catalog(ir, plan) -> BrowserCatalog` remains a separate semantic API with unchanged
`ess-browser-catalog/1` bytes. It does not run the workspace's fatal feasibility gate. Its equality
with a generated catalog applies when the Web workspace is emitted; catalog availability does not
establish code feasibility. Previously valid Rust output is preserved. Shared HTTP/Web codec
changes borrow map-key diagnostic paths in the previously compiler-invalid Integer, Boolean and
Bytes decoder branches. Other Web code changes are limited to the previously broken zero-delivery
replay branch, empty-event log branch and missing catalog-only export buffers. Existing
report/catalog contracts remain unchanged.
[Catalog API](../../crates/generate/ess-synth/src/web/mod.rs), [decision](review-rust-target-feasibility.md).

## Infrastructure

These are a separate bounded context from `EssIr`. The checked ownership transformation added no
wire version and does not establish collection completeness.

| Surface / discriminator | Separate version or identity | Producer; reader and admission | Canonical bytes / digest |
|---|---|---|---|
| Observation: `format: infra-observation/1` | Context, scan timestamp, `scout_version` (producer release) | `ess-kubernetes::scan` sanitizes before writing. Permissive `RawBundle` → `Observation::try_from` validates marker/kinds/values; unknown collected fields are tolerated. Current scope/retry qualifications remain limited. [Writer](../../crates/infra/ess-kubernetes/src/lib.rs), [reader](../../crates/infra/infra-domain/src/observation.rs), [raw](../../crates/infra/infra-domain/src/raw.rs). | Pretty JSON Value **without appended LF**; scanner prints SHA-256 of those full output bytes. That scan-file digest is not InfraIR model identity. |
| Persisted IR: `format: infra-ir/1` | Observation provenance plus model digest | `compile`/`InfraIr::document` → `read_document`: exact format, closed mirrors, model hash and resolved-reference membership. Private model + `model()`/`try_transform` protects ownership; admission does not rederive every unresolved fact, domain value or collection-completeness claim. [Writer/API](../../crates/infra/infra-compiler/src/ir.rs), [reader](../../crates/infra/infra-compiler/src/read.rs). | CLI P envelope. Bare SHA-256 of **C key-sorted model only**; excludes provenance and envelope/digest fields. |
| Intent: `format: infra-spec/1` | Human-readable `name` plus typed-intent digest, not release/version identity | `infra_spec::read_spec`: JSON/YAML → raw closed shapes → `TryFrom` checks marker, scope, predicates and remedies. Typed `InfraSpec` is Serialize-only; no CLI canonical source writer. [Reader](../../crates/infra/infra-spec/src/raw.rs), [type/digest](../../crates/infra/infra-spec/src/spec.rs). | `InfraSpec::digest()` hashes C key-sorted serialized typed intent (`format`, `name`, `expectations`), preserving array order, with no LF. Bare 64 lowercase SHA-256; not raw authored-file identity. |
| Drift: `format: infra-drift/1` | Before/after context and InfraIR model digests | `infra_spec::drift` compares typed IRs and checks context; Serialize-only report. It does not prove comparable collection completeness. [Owner](../../crates/infra/infra-spec/src/drift.rs). | K; referenced model digests, no report hash. |
| Simulation: `format: infra-simulation/1` | Intent name and snapshot context/digest | `simulate` from validated intent and typed IR; Serialize-only report with true/false/unknown outcomes. [Owner](../../crates/infra/infra-spec/src/simulate.rs). | K; snapshot model digest, no simulation hash. |
| Graph: `format: infra-graph/1` | Context, optional namespace, `source_digest` | `GraphDocument::of` over typed InfraIR/graph; Serialize-only. Here `source_digest` means **InfraIR model digest**, not EssIr's digest. [Owner](../../crates/infra/infra-analyze/src/graph.rs). | P; referenced InfraIR digest, no graph hash. |
| Projection JSON: `format: infra-projection/1`, **`artifacts`** | Intent name; `provenance.snapshot_digest` names InfraIR model, `provenance.specification_digest` names typed InfraSpec | `project` returns checked `Result` and mints both input digests; `Projection::document`/`to_json` emits `ProjectionDocument`, including emitted artifact contents; no reader. [Owner](../../crates/infra/infra-project/src/project.rs). | K; separate referenced model/intent identities, no whole-projection digest. |
| Projection YAML / direct typed serialization: `format: infra-projection/1`, **`patches`/`objects`** | Same intent name, snapshot digest and typed-intent `specification_digest` | CLI YAML serializes `Projection` directly, not `ProjectionDocument`, retaining both provenance digests; no reader. The marker is shared with the distinct JSON output shape. [Type](../../crates/infra/infra-project/src/project.rs), [CLI](../../crates/edge/ess-cli/src/main.rs). | Y in CLI; no additional digest. Artifact JSON files use the separate patch writer. |

## Other machine output and artifact containers

### Qualified namespace topology migration

`infra-observation/2` adds mandatory closed `coverage` (`namespace_topology`, exact namespace).
All seventeen declared kind lists are required. Namespace identity and referenced-node membership
are checked. It deliberately omits configuration/Secret payloads, literals and probes. The old
observation/1 writer retains successful bytes; a failed read no longer changes scope through retry.

`infra-ir/2` adds this coverage to the canonical model and digest. The IR/1 reader forbids the
field; legacy documents keep their canonical bytes. The new reader checks qualification against
membership and omitted content, and checked transformations cannot change it. Independent frozen
old-envelope tests reject version 2. This is a concrete format migration within the existing
infrastructure bounded context, not a shared envelope with EssIr.

`infra-graph/2` and `infra-drift/2` retain coverage. Drift refuses unequal coverage (including
legacy/qualified pairs), and does not interpret referenced-node membership as cluster membership.
`infra-simulation/2` retains coverage in `CollectionLimited` unknown outcomes. The unversioned
diagnosis presentation adds `INFRA-DIAG-021`; projections refuse qualified input before output.
No infrastructure projection format changes. No deployed reader is assumed upgraded by this change.

These outputs have no ESS version discriminator or persisted admission protocol. A JSON/YAML
presentation can be redirected to disk, but doing so does not make it a versioned input to another
ESS command. They have no own semantic/release version or whole-document digest unless a field
explicitly references one already cataloged above.

| Unversioned surface | Producer / reader boundary | Layout and identity |
|---|---|---|
| Generated artifact map (`path`, `contents`, `slice`) | `ess generate` / `synthesize` machine output; CLI and xtask use actual artifact contents. [Artifact](../../crates/generate/ess-gen/src/artifact.rs), [CLI](../../crates/edge/ess-cli/src/main.rs), [xtask](../../crates/edge/ess-xtask/src/main.rs). | CLI pretty JSON+LF or YAML; stamps belong to individual contents, not the map. |
| Interaction `SystemGraph` | `ess graph`; Serialize-only graph presentation. [Owner](../../crates/generate/ess-gen/src/graph.rs). | CLI JSON/YAML, DOT or Mermaid; no graph hash. |
| Inspected declaration map | `ess inspect` selects named families from serialized IR. [Owner](../../crates/edge/ess-cli/src/main.rs). | Sorted JSON Value / YAML; fragments are not a persisted EssIr reader format. |
| `ValidationSummary` | CLI successful validation; no report reader. [Owner](../../crates/edge/ess-cli/src/main.rs). | Pretty JSON+LF/YAML; displayed specification major, no report hash. |
| `RefusalReport` | CLI specification parse/assemble/resolve refusal; no report reader. [Owner](../../crates/edge/ess-cli/src/main.rs). | Pretty JSON+LF/YAML; problems and diagnostics, no independent version/hash. |
| `AdapterReport` | Kubernetes/OpenAPI import and projection coverage reporting; no report reader. [Owner](../../crates/edge/ess-cli/src/main.rs). | Pretty JSON+LF/YAML; coverage/refusals/unresolved counts, no independent version/hash. |
| Delivery diagnostic array | `ess-deployment::Diagnostics` through CLI; no document-admission reader. [Owner](../../crates/generate/ess-deployment/src/diagnostic.rs). | CLI JSON/YAML; code/stage/subject/detail, no digest. |
| Composition diagnostic array | Composition compiler refusal through CLI. [Owner](../../crates/specify/ess-composition/src/lib.rs). | CLI JSON/YAML; no digest. |
| Realization diagnostics | Realization compilation refusal through CLI. [Owner](../../crates/specify/ess-realization/src/lib.rs). | CLI JSON/YAML; no digest. |
| Infrastructure `Diagnosis` | `infra diagnose` renders typed findings; no persisted reader. [Owner](../../crates/infra/infra-analyze/src/diagnose.rs). | CLI JSON/YAML; sorted findings, no digest. |
| Schema-validation result object | `schema validate` constructs schema_count/valid/issues from offline registry checks; no report reader. [Owner](../../crates/edge/ess-cli/src/schema.rs). | CLI JSON/YAML; referenced schema resource identities, no report digest. |
| BuildKit Bake JSON and Containerfiles | `project_buildkit` writes executor inputs consumed by Docker Buildx; no ESS format marker. [Owner](../../crates/generate/ess-deployment/src/build.rs). | Deterministic external-tool inputs; not a new ESS IR/digest profile. |
| Helm/Kubernetes manifests and strategic patches | Delivery projection and infrastructure projection emit external API objects and patches. [Delivery](../../crates/generate/ess-deployment/src/environment.rs), [patches](../../crates/infra/infra-project/src/patch.rs). | External `apiVersion`/`kind` or patch semantics; patch JSON is key-sorted pretty+LF. Object API versions do not version `infra-projection/1`. |
| Generated Markdown/HTML, Rust/Go/Clap/TypeScript code, Cargo/Go manifests and browser assets | Generator/synthesis/schema-contract writers produce destination files, not a common persisted DTO. [Generation](../../crates/generate/ess-gen/src/lib.rs), [synthesis](../../crates/generate/ess-synth/src/lib.rs), [TypeScript](../../crates/generate/schema-contract/src/typescript.rs). | Writer-defined deterministic bytes; use any actual ESS provenance stamp rather than assigning a format marker from the file extension. Adopter-owned schema `$id` remains its authority. |

## Digest profiles and admission

| Identity | Bytes hashed and spelling | What it does not establish |
|---|---|---|
| EssIr `source_digest`, suite `spec_digest` | SHA-256 of compact **typed compiled EssIr**, bare 64 lowercase hex. [Owner](../../crates/specify/ess-compiler/src/ir.rs). | Not raw YAML, pretty IR output, suite bytes or release identity. |
| Whole-model `contract_digest` | Compact sorted construct-payload object, bare 64 lowercase hex. [Owner](../../crates/generate/ess-gen/src/provenance.rs). | Different payload from source_digest; no exact-suite identity. |
| `ModelSlice::Constructs` contract digest | Same payload family restricted by the current dependency closure; **`slice-sha256/2:<64 lowercase hex>`**. [Owner](../../crates/generate/ess-gen/src/provenance.rs), [graph](../../crates/specify/ess-compiler/src/graph.rs). | Equal suffixes across profiles are not equal identities. Legacy bare slice stamps owe regeneration. |
| Delivery `Digest` | `sha256:<64 lowercase hex>`; canonical delivery methods hash P bytes. [Owner](../../crates/generate/ess-deployment/src/identity.rs). | The prefix alone cannot tell payload, registry-manifest, semantic, build or evidence identity apart. |
| Realization identity | Prefixed SHA-256 of compact specification/synthesis/implementations tuple. [Owner](../../crates/specify/ess-realization/src/lib.rs). | Not a hash of the whole realization document or its prose entrypoints. |
| InfraIR identity | Bare SHA-256 of compact sorted model JSON. [Owner](../../crates/infra/infra-compiler/src/ir.rs). | Excludes provenance; valid handles and digest do not prove observation completeness. |
| InfraSpec identity; projection `provenance.specification_digest` | Bare 64 lowercase SHA-256 of compact key-sorted serialized typed `InfraSpec` (`format`, `name`, `expectations`), no LF. Array order, including declared expectation order, remains significant. [Digest](../../crates/infra/infra-spec/src/spec.rs), [consumer](../../crates/infra/infra-project/src/project.rs). | Not raw authored YAML, the InfraIR model digest or a whole-projection hash. |
| Exact-suite bytes, **explicit opt-in** | `SuiteReference.digest_profile: sha256-json-bytes/1` in report/2, run/2 and replay/1. [Owner](../../crates/verify/ess-conformance/src/counts.rs). | SHA-256 of every admitted original suite/1–5 UTF-8 byte, including final newline. The carrier and reduced execution view are not hash inputs. Legacy coverage remains unknown; suite/4 and report/1 bytes stay frozen. |

`Provenance::read_digests` now delegates to [stamp admission](../../crates/generate/ess-gen/src/stamp.rs):
complete authoritative comment/JSON/YAML envelopes, unique keys/locations, exclusive aliases and
accepted source/contract hash spellings. Paired OpenAPI/AsyncAPI comment and structured copies must
agree. Synthesis Cargo manifest frames carry only the comment; the stamp reader recognizes the
writer frame and does **not** parse or validate TOML. Generic `Provenance`/`SlicedProvenance`
Deserialize has String fields and supplies no equivalent admission. `ess-docs/1` has nested page
stamps, not one artifact stamp. Historical substring readers may find unrelated model text; do not
claim that every legacy reader rejects a new profile.

The committed-tree check is the library `ess_diff::impact` path accepting a `GeneratedTree`, which
uses `verify_committed`. The current CLI passes `None` and exposes no `--generated` option. Its
artifact answer comes from regeneration of the compared models, not inspection of an on-disk tree.
See [impact](../../crates/verify/ess-diff/src/impact.rs) and [CLI](../../crates/edge/ess-cli/src/main.rs).

## Representative existing bytes

- [Committed InfraIR](../../examples/k3d-dev-cluster/cluster.ir.json) is a pretty envelope, but its
  `digest` is recomputed from sorted compact `model` bytes only. The scanner's
  [historical observation](../../examples/k3d-dev-cluster/observation.json) and its scan provenance
  do not become that hash input.
- [Committed projection summary](../../examples/k3d-dev-cluster/projection/SUMMARY.md) carries
  separate specification and snapshot digests. The [projection writer](../../crates/infra/infra-project/src/project.rs)
  derives them from `InfraSpec::digest()` and `InfraIr::digest()` respectively; the summary's
  displayed specification digest is not a hash of the authored file or of SUMMARY.md.
- [Billing suite](../../suites/generated/billing/suite.json) and
  [neutral plan](../../generated/rust/billing/plan.json) carry the same source/whole identities;
  their different file bytes are not their own identity hashes.
- [Generated schema](../../generated/schema/events/billing.invoice.InvoiceCreated.schema.json)
  shows source identity beside the profiled Constructs contract identity.
- Delivery's P-byte hashing is exercised by the
  [canonical/digest methods](../../crates/generate/ess-deployment/src/identity.rs) and
  [delivery tests](../../crates/generate/ess-deployment/tests); adding/removing the final LF changes
  those byte hashes. Source-digest hashing does not include that rendering LF in the first place.

These are layout/hash-input references. A source-reading catalog is not an executed compatibility
test; the implementation report records any actual fixture calculations separately.

## Successors and relying parties

Default writers remain suite **/4**, standalone report **/1**, and unversioned detailed runs.
Explicit report **/2** and detailed **ess-conformance-run/2** retain original
suite/1–4 admission, exact-byte pairing and unknown coverage, and now accept admitted suite/5
coverage. Their JSON objects use sorted UTF-8
keys, two-space layout and a final LF; counts/timestamps are exact unsigned decimal u64. Rust u64
and Go int64 timestamps remain separate frozen legacy domains. The
[binding conformance design](review-conformance-coverage.md) supplies suite **/5** and durable
declared coverage/selection. The [transport binding](review-conformance-coverage-transport.md)
adds complete original parent input and paired browser admission. No default movement, publisher
authentication, independent inventory honesty or deployed adopter readiness follows from these types.

| New opt-in document | Closed fields and checked meaning |
|---|---|
| `provenance.suite_version: ess-conformance/5` | Exactly provenance/scenarios/coverage; declared scope, origins, filter, knowledge, selected/outside IDs, every refusal occurrence and every requested authored source. Immutable original-byte admission; the legacy DTO cannot issue known coverage by relabeling. [Owner](../../crates/verify/ess-conformance/src/coverage.rs). |
| `ess-conformance-input/1` | Exactly format/suite_json/parent_suites; original selected text and complete nearest-first original parents. Admission compares full surviving definitions/dependencies, inventory and provenance; no ambient lookup or lineage-depth cap. [Owner](../../crates/verify/ess-conformance/src/coverage.rs). |
| `ess-conformance-replay/1` | Exactly format/model/suite/input; explicit closed typed model projection, exact selected reference and admitted input. The browser validates before replay state, displays all refusals and produces no execution report. Existing projection omissions remain. [Owner](../../crates/verify/ess-conformance/src/web_replay.rs). |
| `ess-conformance-report/2`, `ess-conformance-run/2` | Actual immutable execution capability binds selected original bytes. Complete nonempty all-pass inventory with no in-scope refusal qualifies only for its declared selection. The standalone and detailed envelopes remain separate. [Owner](../../crates/verify/ess-conformance/src/counts.rs). |

Fresh builders check model Binary64 before inventory/output. Go admits the full original u64 wire
and lineage before adapting selected fields to its host int width; unrepresentable selected execution
fields refuse before callbacks, while omitted parent fields remain admitted. Exact timestamp domains
are unchanged. Suite/5 always requires explicit report/2, even with allow-incomplete or no destination.
Impact's admitted-input path requires complete inventory and keeps selection as separate context;
persisted output remains ess-impact/3 with unchanged fields and meanings.

Before moving a default, implement admission and preserved legacy fixtures, migrate actual readers,
regenerate/test runtimes and record the coordinated Atlas decision and shipped versions. The design
records prior pinned AEP adapter and independent report-reader seams; it does not establish that
current external mains or deployed readers migrated. Realization conformance digests are references,
not demonstrated byte-verifying report readers. Existing F01 coordination records SDK comparison of
complete generated files and Atlas ADR 0036, not an external delta/impact parser. ESS has no AEP
dependency, and this catalog creates none.

The frozen Rust/Web producer introduces checked Result APIs and `ess-target-failure/1` while
retaining the successful plan/report/catalog contracts above. External callers must handle the
checked result when upgrading. Coordinator-owned reader integration and delivery remain separate;
this inventory establishes no SDK/AgentIDE upgrade, release or deployed-consumer compatibility.

No schema resource identity redesign, mass format rename, `ess-ir/2`, new universal registry or
normalization of historical bytes is part of this documentation change.
