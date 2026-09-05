---
title: Formats and digests
description: Identify ESS documents, their readers, and the bytes each digest names.
---

# Formats and digests

Choose a reader by the document's producer and shape as well as its marker. Some ESS outputs are
unversioned, and two different shapes can share a marker. A successful JSON or YAML parse does not
necessarily validate the document's claims.

A format version such as `ess/1`, a specification version such as `v3`, and a software release
such as `0.18.0` identify different things. Specification versions are major-only; typed IR serializes
that major as a number. Delivery release versions and constraints use SemVer independently.
[Version owners][versions], [specification format support][system].

## Which digest is this?

| Identity | Hash input and spelling |
|---|---|
| **Compiled model**: `source_digest` or suite `spec_digest` | SHA-256 of compact typed `EssIr` JSON, with no appended newline. Bare 64 lowercase hexadecimal characters. This is not a hash of raw YAML or pretty `ess compile` output. [Source][ir] |
| **Whole contract**: whole-model `contract_digest` | SHA-256 of a compact, key-sorted construct payload. Bare 64 lowercase hex; the payload differs from the compiled-model hash input. [Source][provenance] |
| **Sliced contract**: Constructs `contract_digest` | `slice-sha256/2:<64 lowercase hex>`, from the selected constructs and their dependency closure. The profile prefix is part of identity; a bare legacy slice digest requires regeneration. [Source][provenance] |
| **Delivery document**: `Digest` | `sha256:<64 lowercase hex>`. Canonical delivery `digest()` methods hash pretty JSON including its final LF. Artifact/OCI digests can use the same spelling for different bytes. [Source][delivery-identity] |
| **Realization**: `realization_digest` | Prefixed SHA-256 of the compact specification/synthesis/implementations tuple, not the entire realization document. [Source][realization] |
| **Infrastructure model**: InfraIR `digest` | Bare SHA-256 of compact, key-sorted `model` JSON, excluding the envelope and observation provenance. [Source][infra-ir] |
| **Infrastructure intent**: `InfraSpec::digest()`, projection `provenance.specification_digest` | Bare 64 lowercase SHA-256 of compact, key-sorted serialized typed `InfraSpec`: `format`, `name` and `expectations`, with no appended newline. Array order, including declared expectation order, remains significant. This differs from authored-file, InfraIR-model and whole-projection identity. [Digest][infra-spec-digest], [consumer][infra-project] |

A digest field identifies only the bytes its producer defines. Matching syntax does not establish
that two digest domains are interchangeable, that a report covers its exact suite, or that a remote
artifact has been fetched and verified.

## Specifications and implementation plans

“Closed DTO” below means unknown fields are refused. The named compile or validation step still
checks semantics. “Pretty JSON” means the writer's deterministic rendering with a final LF unless
the row says otherwise; it does not imply that those bytes are hashed.

| Document and discriminator | Version/identity carried separately | Reader and byte contract |
|---|---|---|
| Authored specification: `format: ess/1` | Specification `vN` | `RawSpecFile::parse`, assembly/validation and compilation. Format parsing checks syntax; semantic support is currently major 1. No canonical raw-source hash. [Source][spec] |
| Compiled `EssIr`: **unversioned** | Numeric specification major | Compiler-minted, Serialize-only; no general persisted-IR reader. Pretty JSON output; **compiled-model** digest uses compact bytes instead. There is no current `ess-ir/1` marker. [Source][ir] |
| Authored composition: `format: ess-composition/1` with a **services array** | Composition/service keys and exact service source digests | Closed JSON/YAML DTO, then `compile` against supplied services. Pretty canonical JSON; no whole-composition digest. [Source][composition] |
| Compiled composition: `format: ess-composition/1` with a **services map** | Resolved service identities | Serialize-only compiler output. The authored reader does not read this shape. Pretty JSON; service source digests remain references. [Source][composition] |
| Client plan: `format: ess-client-plan/1` | Composition key and exact services | Generated from compiled composition; Serialize-only. Pretty JSON; no client-plan byte digest. [Source][composition] |
| Authored realization: **`type: ess-realization/1`** | Realization id and specification/synthesis identities | Closed JSON/YAML DTO, then compilation against supplied ESS authority. No raw-document digest contract. [Source][realization] |
| Compiled realization: **`type: ess-realization-ir/1`** | Same identities plus realization digest | Serialize-only compiled output. Pretty JSON; **realization** tuple digest. [Source][realization] |
| `plan.json`: **unversioned** `SynthesisPlan` | Specification provenance | Neutral generated plan, consumed as a typed value by emitters. Pretty JSON and `PLAN.md`; **compiled-model/whole-contract** references, no plan-file hash. [Source][plan] |
| `target.json`: **unversioned** `TargetReport` | Target name and specification provenance | Successful Go/Web/Clap synthesis includes this refusal/weakening report; successful Rust has `target: None` and no target metadata. No persisted admission reader. Unchanged pretty JSON and `TARGET.md`; provenance references, no report-file hash. [Source][synthesis] |
| Complete failure: `format: ess-target-failure/1` | Target `rust` or `web`; unchanged neutral plan and its provenance | Serialize-only `TargetFailure` has `format`, `target`, `plan`, nonempty `causes`; private construction, read-only accessors, no Deserialize/admission reader. Typed pretty JSON+LF or CLI YAML; no failure-file digest or artifacts. [Source][target-failure] |

`ess_synth::synthesize` and `synthesize_for` return `Result<Synthesis, TargetFailure>`.
The direct `rust::workspace` and `web::workspace` APIs return `Result<Vec<Artifact>, TargetFailure>`
and `Result<web::Emission, TargetFailure>` respectively. Rust checks allocation and representation
before rendering; Web checks its Rust prerequisite and Web codec allocation. `Err` withholds the
whole requested workspace, even if some modules could be emitted. `Ok` retains the existing
`Synthesis` fields and neutral plan bytes; partial Go/Web/Clap reports remain successful values.
`Synthesis` itself has no serialized envelope.
[Facade][synthesis], [Rust][rust-workspace], [Web][web-workspace].

Each failure cause has a `code`, nonempty sorted unique `sources`, and nonempty `detail`; causes
are also sorted and deduplicated. Current codes are `invalid-identifier`, `symbol-collision`,
`path-collision`, `recursive-layout`, `binding-assignment`, `missing-type-owner`, `wire-collision`
and `missing-representation`. A plan with zero capabilities can still fail. The typed error
implements `Display` and `std::error::Error`; it carries no independent digest.
These checks do not constitute a universal compiler proof; internal coverage assertions remain,
and direct workspace calls require the plan for the supplied IR.
[Error][target-failure], [API limits][synthesis].

For a complete target failure, `ess synthesize` prints the text error or JSON/YAML envelope to
stdout and exits 1 before writing artifacts. It creates no output directory and leaves an existing
destination untouched. Successful text output still counts the neutral plan; partial target notes
are in `TARGET.md`/`target.json`. Later I/O failures have no new rollback guarantee.
[CLI][cli].

## Schema imports and data realization

These surfaces are separate from full application synthesis. Structural data targets do not
implement a model's lifecycle, and normalization executes only an explicitly checked recipe.

| Document | Reader and byte contract |
|---|---|
| `format: ess-schema-bundle/1` | Qualified component import. `Bundle::read` reimports retained source and compares the complete typed result, including selection and dialect. Pretty typed JSON plus LF; `source_digest` hashes exact retained UTF-8 source bytes. [Source][schema-bundle] |
| `format: ess-schema-bundle/2` | Document-root import with explicit `document_root` identity and local definition closure. The same replay reader checks this shape; /1 cannot carry document-root identity and its existing bytes remain unchanged. [Source][schema-bundle] |
| `format: ess-types-report/3` | Serialize-only structural target accounting: roots, declarations, configuration, annotations and obligations. Bundle input carries raw-source and complete-bundle identities; model input carries compiled-model, contract and model-schema projection identities. No report-file digest or persisted admission reader. Earlier /1 and /2 reports are not current writers. [Source][data-realization] |
| `format: ess-normalization/1` | Closed recipe DTO, then `normalize::Plan::read`/`check` with supplied checked bundles. All branches and ordered stage boundaries check before execution. Canonical typed pretty JSON plus LF; parsing alone is not admission. [Source][normalization] |
| `format: ess-normalization-target/1` | Serialize-only standalone Rust library report from `Plan::rust`, with recipe, root/schema and emitted-file identities. Pretty typed JSON plus LF; no persisted report-admission API. It is not a full-synthesis target report. [Source][normalization-target] |

The following hashes are bare lowercase SHA-256, but have different input domains:

- Bundle `source_digest`: original source bytes, not parsed or canonicalized JSON.
- `bundle_digest`: complete `Bundle::to_json()` bytes, including retained source, dialect,
  root selection, format identity and final LF. A raw-source digest cannot replace it.
- Model `projection_digest`: complete `ModelTypes::to_json()` JSON Schema projection bytes,
  including provenance and final LF. Its model `source_digest` and `contract_digest` retain
  their separately defined semantic domains. [Model projection][model-data]
- Normalization `recipe_digest`: checked `Plan::to_json()` bytes, including final LF.
- Normalization `schema_digest` and `files` values: exact emitted UTF-8 file bytes.
  `normalization-report.json` is deliberately excluded from its own file-digest map.

## Component delivery

Every marker here uses the `format` key. Build/runtime/component inputs are closed DTOs followed
by compilation. Persisted IR, release, bundle, catalog and lock readers validate through generic
Serde as well as convenience readers; nested documents cross that boundary too. Revalidate mutable
values before use. These checks establish local consistency, with additional checks where authority
inputs are supplied; they do not prove remote artifact contents or authenticity.
[Validation owner][delivery-validation].

| Document | Independent version or identity | Reader / canonical digest |
|---|---|---|
| `ess-build/1` | Build/system and semantic references | `BuildSpec` DTO → `compile_build`; authored bytes have no canonical digest. [Source][build] |
| `ess-build-ir/1` | Compiled build identity | Checked `BuildIr`; pretty JSON, **delivery-document** digest. [Source][build] |
| `ess-runtime/1` | Exact ESS/realization/build references | `RuntimeSpec` DTO → `compile_runtime`; no raw-input digest. [Source][runtime] |
| `ess-runtime-ir/1` | Compiled runtime identity | Checked `RuntimeIr`; supplied-build validation is separate. Pretty JSON, **delivery-document** digest. [Source][runtime] |
| `ess-component/1` | Semantic major and release-unit names | `ComponentSpec` DTO → `compile_component`; no raw-input digest. [Source][component] |
| `ess-component-ir/1` | Component and semantic version | Checked `ComponentIr`; pretty JSON, **delivery-document** digest. [Source][component] |
| `ess-release/1` | Release SemVer, source commit and exact artifact/evidence identities | Checked `ReleaseManifest`; verify against supplied build/runtime where available. Pretty JSON, **delivery-document** digest. [Source][release] |
| `ess-release-bundle/1` | Independent runtime/chart releases | Checked `ReleaseBundle` and bundle verification. Pretty JSON, **delivery-document** digest, separate from the OCI registry manifest digest. [Source][component] |
| `ess-release-catalog/1` | Candidate semantic versions and release SemVers | Checked `ReleaseCatalog`; no whole-catalog canonical hash API. [Source][stack] |
| `ess-stack/1` | Semantic-major and release SemVer constraints | Closed `StackSpec` DTO → resolver with catalog; no raw-input digest. [Source][stack] |
| `ess-stack-lock/1` | Exact selected release versions/digests | Checked `StackLock`; pretty JSON, **delivery-document** digest. [Source][stack] |
| `ess-environment/1` | Environment id and exact stack digest | Closed `EnvironmentSpec` DTO → deployment compiler; no raw-input digest. [Source][environment] |
| `ess-deployment/1` | Exact stack and release identities | Checked `DeploymentIr`; pretty JSON, **delivery-document** digest. [Source][environment] |
| `ess-deployment-diff/1` | Before/after deployment digests | CLI-produced added/changed/removed sets; no reader. Key-sorted pretty JSON; no diff-file digest. [Source][cli] |

## Change and conformance records

| Document and discriminator | Separate identity | Reader and byte contract |
|---|---|---|
| `format: ess-diff/1` | Before/after compiled-model digests and specification majors | Legacy vocabulary/bytes retained; raw closed DTO → validated delta. Explicit legacy writing refuses new-only kinds. Pretty JSON; no delta-file hash. [Writer][delta], [reader][delta-reader] |
| **Default** `format: ess-diff/2` | Same endpoint identities | Supported delta majors are 1 and 2. Admission checks ids, relations, order, uniqueness and same-system identity; serialization checks the selected vocabulary. Pretty JSON. [Source][delta] |
| **Current** `format: ess-impact/3` | Embedded /2 delta, optional suite and artifact identities | `ess_diff::impact` returns `EssImpact` with 26 dependency relations; no persisted report reader. Pretty JSON; references input digests. [Source][impact] |
| Authored **`type: ess-scenario/1`** | Domain/scenario identity and purpose | Closed authored DTO, then compilation against IR. No raw-source canonical digest. [Source][authored] |
| Suite **`provenance.suite_version: ess-conformance/4`** | Specification `vN`, model and whole-contract digests | Derived Deserialize/from_json parses a suite. Declared support `[1,2,3,4]` is not uniform execution admission: syntax parsing alone can accept an unsupported major. Pretty JSON; no digest of exact suite bytes. [Source][suite] |
| Rust `format: ess-conformance-report/1` | Model digest, implementation and suite-version claim | Checked closed reader validates version/counts/list/status; it does not establish exact-suite coverage or unique opaque result ids. Pretty JSON; unsigned u64 `completed_at`. [Source][report] |
| Go `format: ess-conformance-report/1` | Same claims, Go failed/skipped vocabulary | Generated Go writer; current Rust admission accommodates its non-pass vocabulary. Indented JSON+LF, signed int64 `completed_at`; no cross-producer byte/range equivalence is implied. [Source][go-report] |
| Detailed `ConformanceReport`: **unversioned** | Suite provenance, implementation, run/scenario identities | Detailed CLI JSON/YAML is distinct from standalone `--report-out` JSON. Serialize-only; pretty canonical JSON, no report-file or exact-suite hash. [Source][detailed-report] |

`ess verify impact` computes generated-artifact obligations from the compared models. The CLI
has no `--generated` option and does not inspect a committed output tree. The library API can accept
a `GeneratedTree` for that additional check. See [Track specification change](../guides/track-change.md)
and the [impact implementation][impact].

## Generated documents and external formats

| Document | Independent version/identity | Reader and bytes |
|---|---|---|
| `format: ess-docs/1` | Specification version and **per-page** provenance | Derived document/page parsing; explicit page-id and renderer checks are separate. CLI pretty JSON; nested String digests are not automatically profile-validated. [Source][docs-ir] |
| `format: ess-browser-catalog/1` | Specification version and plan provenance | Web writer/generated browser consumer; no general catalog-admission API. Key-sorted pretty JSON, whole-contract provenance. [Source][browser-catalog] |
| `format: ess-service-interface/1` | `source_openapi` dialect and `service.version` | Closed DTO then explicit `validate`. Pretty JSON; no interface digest. [Source][interface] |
| JSON Schema draft 2020-12 | `$id`, specification version and ESS provenance | ESS contract projection for external schema consumers. Pretty JSON, source/sliced stamps. [Source][schema] |
| OpenAPI 3.1.0 from ESS | `info.version` and ESS provenance | ESS generation and HTTP synthesis; deterministic YAML or pretty JSON, source/sliced stamps. [Source][openapi] |
| OpenAPI 3.1.0 from imported interface | Interface service version | Separate `ess-openapi` projection; deterministic YAML, no invented compiled-ESS digest. [Source][interface] |
| AsyncAPI 3.0.0 | `info.version` and ESS provenance | ESS generation; deterministic YAML, source/sliced stamps. [Source][asyncapi] |
| ESS authoring schema, draft-07 | Describes source format/version syntax | `cargo xtask schema`; pretty JSON, complete-byte drift check. Schema validation does not resolve a whole specification. [Source][xtask] |
| Rust HTTP startup JSON lines: **`log: ess/1`** | Specification facts plus runtime language/address/port | Generated server output; compact lines, no whole-record hash. This shared marker value does not make the log a specification document. [Source][rust-http] |
| Go HTTP startup JSON lines: **`log: ess/1`** | Shared specification facts plus Go runtime fields | Separate generated writer using shared startup facts; no log-admission reader or cross-language byte promise. [Source][go-http] |

`web::browser_catalog(ir, plan) -> BrowserCatalog` remains a separate semantic catalog API with
unchanged `ess-browser-catalog/1` bytes. It does not run the workspace's fatal feasibility gate;
catalog availability does not establish that a Rust/Web workspace can be emitted. When a Web
workspace is emitted, its catalog uses the same bytes. [Catalog API][web-workspace].

Generated artifact maps, validation/refusal/adapter summaries, inspected declarations, interaction
graphs, delivery/composition/realization diagnostics, infrastructure diagnoses and schema-validation
reports are **unversioned machine presentations**. Their JSON/YAML can be saved, but there is no
general ESS reader for those report files or an implicit whole-report digest.
[CLI owners][cli], [schema reports][schema-cli], [infrastructure diagnosis][diagnosis].

Markdown, HTML, generated source code and build manifests follow their individual writers.
Kubernetes `apiVersion`/`kind`, Helm artifacts and BuildKit inputs use their external contracts;
they do not introduce another ESS format marker. Adopter-owned JSON Schema `$id` identifies its
schema resource independently of any digest.

## Infrastructure records

| Document and discriminator | Independent identity | Reader and byte contract |
|---|---|---|
| `format: infra-observation/1` | Context, scan time, scanner release | Sanitized scanner output; permissive raw DTO → observation validation. Pretty JSON without an appended LF; scanner-reported hash covers those file bytes. It does not prove complete collection scope. [Writer][scanner], [reader][observation] |
| `format: infra-ir/1` | Observation provenance and model digest | `read_document` checks exact format, closed mirrors, hash and resolved-reference membership. CLI pretty envelope; **infrastructure-model** digest. Checked model transformations add no wire version or completeness proof. [API][infra-ir], [reader][infra-reader] |
| `format: infra-spec/1` | Human-readable intent name and typed-intent digest | JSON/YAML → raw shapes → validated `InfraSpec`. `digest()` hashes the compact sorted typed intent; no canonical authored-file digest. [Reader][infra-spec], [digest][infra-spec-digest] |
| `format: infra-drift/1` | Before/after context and model digests | Serialize-only typed comparison; key-sorted pretty JSON. Context agreement does not prove equal collection scope. [Source][infra-drift] |
| `format: infra-simulation/1` | Intent name and snapshot digest | Serialize-only simulation with unknown outcomes; key-sorted pretty JSON, no simulation hash. [Source][infra-simulation] |
| `format: infra-graph/1` | Context/namespace and `source_digest` | Serialize-only graph; pretty JSON. Its source digest names the **InfraIR model**, not EssIr. [Source][infra-graph] |
| `format: infra-projection/1` JSON, **artifacts list** | Intent name; `provenance.snapshot_digest` names InfraIR model, `provenance.specification_digest` names typed InfraSpec | `ProjectionDocument` contains emitted file contents and both input digests; key-sorted pretty JSON, no reader or whole-output hash. [Source][infra-project] |
| `format: infra-projection/1` YAML, **patches/objects lists** | Same intent name, snapshot digest and typed-intent `specification_digest` | CLI serializes `Projection` directly, retaining both input digests. This differs from the JSON document despite the shared marker; no persisted reader. [Type][infra-project], [CLI][cli] |

## Compatibility boundaries

Use the owning reader's validation API. The stamp reader accepts complete authoritative envelopes
and supported profiles, including matching comment/structured copies where the writer emits both.
Generic String deserialization does not perform that check. Cargo synthesis stamp recognition does
not validate TOML, and a docs document has per-page stamps rather than one artifact stamp.
[Stamp reader][stamp].

Suite **/5**, standalone report **/2**, detailed **ess-conformance-run/2**, and the exact-suite
`sha256-json-bytes/1` profile are successor designs, **not current writer/reader support**. They need
implemented admission, legacy fixtures, reader migration, regenerated runtimes and coordinated
default changes. Current suite/4 and report/1 do not establish durable complete coverage or exact
suite-byte identity. A format catalog alone does not establish an external consumer upgrade.

[versions]: https://github.com/beyond10x/ess/blob/main/crates/specify/ess-domain/src/name.rs
[schema-bundle]: https://github.com/beyond10x/ess/blob/main/crates/generate/schema-contract/src/bundle.rs
[data-realization]: https://github.com/beyond10x/ess/blob/main/crates/generate/schema-contract/src/realize.rs
[model-data]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/model_types.rs
[normalization]: https://github.com/beyond10x/ess/blob/main/crates/generate/schema-contract/src/realize/normalize.rs
[normalization-target]: https://github.com/beyond10x/ess/blob/main/crates/generate/schema-contract/src/realize/normalize/target.rs
[system]: https://github.com/beyond10x/ess/blob/main/crates/specify/ess-domain/src/system.rs
[spec]: https://github.com/beyond10x/ess/blob/main/crates/specify/ess-domain/src/spec.rs
[ir]: https://github.com/beyond10x/ess/blob/main/crates/specify/ess-compiler/src/ir.rs
[composition]: https://github.com/beyond10x/ess/blob/main/crates/specify/ess-composition/src/lib.rs
[realization]: https://github.com/beyond10x/ess/blob/main/crates/specify/ess-realization/src/lib.rs
[provenance]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/provenance.rs
[stamp]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/stamp.rs
[delivery-identity]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-deployment/src/identity.rs
[delivery-validation]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-deployment/src/validation.rs
[build]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-deployment/src/build.rs
[runtime]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-deployment/src/runtime.rs
[component]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-deployment/src/component.rs
[release]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-deployment/src/release.rs
[stack]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-deployment/src/stack.rs
[environment]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-deployment/src/environment.rs
[cli]: https://github.com/beyond10x/ess/blob/main/crates/edge/ess-cli/src/main.rs
[delta]: https://github.com/beyond10x/ess/blob/main/crates/verify/ess-diff/src/delta.rs
[delta-reader]: https://github.com/beyond10x/ess/blob/main/crates/verify/ess-diff/src/raw.rs
[impact]: https://github.com/beyond10x/ess/blob/main/crates/verify/ess-diff/src/impact.rs
[authored]: https://github.com/beyond10x/ess/blob/main/crates/verify/ess-conformance/src/authored.rs
[suite]: https://github.com/beyond10x/ess/blob/main/crates/verify/ess-conformance/src/scenario.rs
[report]: https://github.com/beyond10x/ess/blob/main/crates/verify/ess-conformance/src/evidence.rs
[go-report]: https://github.com/beyond10x/ess/blob/main/crates/verify/ess-conformance/src/go/runtime.go
[detailed-report]: https://github.com/beyond10x/ess/blob/main/crates/verify/ess-conformance/src/report.rs
[plan]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/src/plan.rs
[synthesis]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/src/lib.rs
[target-failure]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/src/failure.rs
[rust-workspace]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/src/rust/mod.rs
[web-workspace]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/src/web/mod.rs
[docs-ir]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/document.rs
[browser-catalog]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/src/web/catalog.rs
[interface]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-openapi/src/lib.rs
[schema]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/schema.rs
[openapi]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/openapi.rs
[asyncapi]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-gen/src/asyncapi.rs
[xtask]: https://github.com/beyond10x/ess/blob/main/crates/edge/ess-xtask/src/main.rs
[rust-http]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/src/rust/http.rs
[go-http]: https://github.com/beyond10x/ess/blob/main/crates/generate/ess-synth/src/go/http.rs
[schema-cli]: https://github.com/beyond10x/ess/blob/main/crates/edge/ess-cli/src/schema.rs
[diagnosis]: https://github.com/beyond10x/ess/blob/main/crates/infra/infra-analyze/src/diagnose.rs
[scanner]: https://github.com/beyond10x/ess/blob/main/crates/infra/ess-kubernetes/src/lib.rs
[observation]: https://github.com/beyond10x/ess/blob/main/crates/infra/infra-domain/src/observation.rs
[infra-ir]: https://github.com/beyond10x/ess/blob/main/crates/infra/infra-compiler/src/ir.rs
[infra-reader]: https://github.com/beyond10x/ess/blob/main/crates/infra/infra-compiler/src/read.rs
[infra-spec]: https://github.com/beyond10x/ess/blob/main/crates/infra/infra-spec/src/raw.rs
[infra-spec-digest]: https://github.com/beyond10x/ess/blob/main/crates/infra/infra-spec/src/spec.rs
[infra-drift]: https://github.com/beyond10x/ess/blob/main/crates/infra/infra-spec/src/drift.rs
[infra-simulation]: https://github.com/beyond10x/ess/blob/main/crates/infra/infra-spec/src/simulate.rs
[infra-graph]: https://github.com/beyond10x/ess/blob/main/crates/infra/infra-analyze/src/graph.rs
[infra-project]: https://github.com/beyond10x/ess/blob/main/crates/infra/infra-project/src/project.rs
