---
format: aep.planning-md/1
id: story:review-openapi-semantic-accounting
kind: story
status: active
title: Account for every unpreserved OpenAPI constraint
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: inferred
  path: Cargo.lock
- confidence: cited
  path: README.md
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/generate/ess-openapi
- confidence: cited
  path: docs/design/review-format-catalog.md
- confidence: inferred
  path: docs/design/review-openapi-accounting.md
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 16
---
## Finding and source

F04 (P1) from `docs/reviews/2026-09-05-architecture-review.md:245`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/generate/ess-openapi/src/lib.rs:721`, `crates/generate/ess-openapi/src/lib.rs:859`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Each reviewed OpenAPI constraint that is not preserved produces a durable gap or refusal instead of a zero-gap success.

## Implementation boundary

Track consumed meaning by schema variant and OpenAPI dialect, including integer/number enums, arrays without items, local reference siblings and unresolved targets. Keep import diagnostics and provenance alongside persisted interface output through a designed compatible wrapper or versioned format; do not guess lifecycle entities.

## Validation

Use counterexamples for 3.0 and 3.1 reference semantics, integer enums, arrays without items and dangling refs; assert gaps/refusals both immediately and after writing/reloading the import result. Supported unchanged examples still round-trip.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Adding broad new OpenAPI features is optional; accurate refusal suffices for unsupported cases. Do not silently add fields to an old strict persisted envelope.

## Scope

Derived 2026-09-05 by independent aep-drive:story-scoper from story revision4 and coordinator0d267e25739ca495ad1a229393181ad1b75182f3; relevant sources unchanged by subsequent delivery merge. No files, scratch, store, Git state or builds changed during scoping.

- crates/generate/ess-openapi — cited; owns ServiceInterface/validate, InterfaceSchema, ImportReport/import/read_interface/project/schema_value, Importer::document/schema/object_schema/schema_gaps and four inline tests (lib.rs:12,17,40,189,233,261,275,290,383,468,723,809,858).
- crates/edge/ess-cli — cited; main.rs:2862 persists only imported.interface.to_canonical_json, then builds separate terminal report; :3174 reloads ServiceInterface and :3184 projects it. Import/project help (:554,:603) names old format. Durable accounting requires these real paths, not only library diagnostics.
- docs/design/review-openapi-accounting.md — inferred; settle dialect/subset accounting, gap versus refusal, provenance, new envelope/version, legacy reads, output defaults and incomplete projection before Rust APIs.
- ImportReport holds gaps/unresolved but has no Serialize or discriminator. ServiceInterface holds source_openapi but loses diagnostics and source identity. AdapterReport main.rs:2734 is Serialize-only terminal output without a versioned reader — cited.
- Existing ess-service-interface/1 and nested structures deny unknown fields. read_interface validates format and limited service/operation invariants, not historical completeness or reference resolution. Adding fields under /1 breaks old readers — cited.
- Importer currently accepts version.starts_with("3.1.") and refuses all3.0. Converter gets no dialect context; root jsonSchemaDialect/schema $schema are merely unpreserved keywords. No existing3.0 branch to preserve — cited.
- Explicit string schemas preserve string enum/const, but empty enum becomes emptyVec then disappears on projection. Integer/number/bool/array/object enum/const are unpreserved despite global keyword allowlist suppressing diagnostics. Missing/non-string type plus enum/const can infer string, conflating absent type and unsupported type-array — cited.
- Local schema $ref returns before sibling accounting. Non-string/external/non-component refs refuse. Missing component targets exist only in transient ImportReport. Path-item refs (:586) and request/response refs (:675) already refuse. Exercise escaping, component lookup, ref siblings and dangling refs at nested sites — cited.
- Missing array items returnsNone without diagnostic; callers may drop component/property or message schema. Required dropped property may incidentally refuse. Boolean/array items already refuse object-only decode; typed items recurse and other constraints rely on generic gap classifier — cited.
- Minimal direction: retain structural subset and accurately refuse unsupported shapes. Account consumed meaning per variant/dialect; every failed conversion emits a diagnostic before caller discards it. Preserve genuinely supported bare local refs/string constraints and distinguish annotations from constraints — inferred.
- Durable direction: new versioned import-result envelope containing unchanged interface plus deterministic accounting/provenance, or explicit new interface version. Reader returns accounting with interface; projection cannot silently promote partial or legacy-unqualified input to fully accounted — inferred.
- Existing tests cover semantic roundtrip, deterministic projection, immediate dangling refs and external refusal. CLI command_surface.rs:164 and Taskfile.yml:63 import generated billing OpenAPI but never prove persisted accounting reload — cited.
- Red-first every enum/ref/array branch in component/message/property positions and CLI write→read→project. Assert durable exact gaps/refusals, prior valid /1 bytes, actual old strict-reader rejection of new envelope, new-reader legacy handling and unsupported version. Package tests/fmt/Clippy then coordinator full/site gates — inferred.
- Actual local consumer is ESS CLI. Native ess-gen emits raw OpenAPI, not persisted ServiceInterface. Billing/CLI smoke and entity-relations design are compatibility evidence without requiring ess-gen source edits for accurate refusal/accounting — cited.
- Bounded locally available sibling search found no service-interface parser or byte verifier. Atlas catalogue records and agentplugins native ESS project invocation are not such consumers. This is not proof about published crates, installed binaries or external persisted files — cited.
- Keep3.0 refusal unless explicitly implementing additional support. Official3.0 Reference Object ignores siblings and array Schema Object requires items: https://spec.openapis.org/oas/v3.0.3.html#reference-object and #schema-object — cited primary spec, externally verified by scoper.
- In3.1 Schema Object $ref allows siblings, unlike nonschema Reference Object. Recognize or explicitly refuse document dialect/schema override before interpreting constraints: https://spec.openapis.org/oas/v3.1.0.html#reference-object and #schema-object ; https://json-schema.org/draft/2020-12/json-schema-core#section-8.2.3.1 — cited primary specs.
- Omitted3.1 items leaves elements unrestricted; refusal should say unsupported adapter shape, not malformed OAS. Enum/const apply beyond strings and emptyenum cannot silently become unconstrained: https://json-schema.org/draft/2020-12/json-schema-core#section-10.3.1.2 ; https://json-schema.org/draft/2020-12/json-schema-validation#section-6.1.2 — cited primary specs.
- Binding decisions remain: distinct wrapper versus new interface version; reader-first defaults; legacy accounting unavailable (never proven zero) and explicit reimport/acknowledgement/qualification-preserving projection; exact source-byte/normalization/importer identity and dependency/lockfile consequence. Do not reinterpret ESS source_digest. Add README/public documentation scopes before dispatch if defaults change — inferred.
- Clean Atlas7b00adf3b1004e0cdd8dd12aa4fa8cc8435a0432 equaled remote during scoping. Cross-repo ADR rule applies when another repo verifies changed bytes; none established for interface here, so revisit actual consumers instead of inventing an unconditional ADR requirement. Any public documentation still requires its normal downstream publication gates — cited/inferred coordinator boundary.
- Confidence high for converter, strict envelope and actual persisted writer/reader. No new contract decision or compatibility execution claimed. Collides on exact ess-openapi, ess-cli and proposed design tokens; shared planning/integration root-owned — cited/inferred.


## Bound Wave7 implementation

The coordinator accepts docs/design/review-openapi-accounting.md before source edits under the user's standing approval for all remediation waves. The complete original Scope above is retained. The following independent refresh remains attributed proposal text; its concrete design/scope/acceptance choices are now accepted for implementation. Planned cases are not executed evidence. Source pin e113a65 is contained in published combined fadbc674, whose complete ten-lane gate passed1768 Rust cases at2026-09-06T00:53:01Z. That gate is the opening baseline, not a result for this implementation.

# F04 scope refresh and acceptance delta

Proposal only, 2026-09-06. Bind against incoming ESS
`e113a65a0bac63e77cd17f43fa280a5bf56c93f9`; the initial refresh pin was
`1b1c4a3a424e3304e503747422a0c3d10d86928a`. Retain the existing detailed Scope and
finding history from story revision 6. The concise section below adds the intended documentation
and lockfile reservations once the accompanying binding is accepted. Source citations use incoming
e113. Neither acceptance nor any test result is claimed by this proposal.

## Scope

- Primary surface: `crates/generate/ess-openapi` — cited; owns strict ServiceInterface/1, transient ImportReport, import/read/project, variant conversion and inline tests (`src/lib.rs:12–80,189–318,723–970,975–1057`). Actual edits stay in this crate, including its manifest and focused fixtures/tests.
- CLI boundary: `crates/edge/ess-cli` — cited; actual intended edits are OpenAPI help and import/project routing in `src/main.rs:560,610,2825–2900,3165–3205` and focused CLI tests; the existing package reservation is retained.
- Binding document: `docs/design/review-openapi-accounting.md` — inferred; record the accepted envelope/profile, raw-source identity, checked admission, legacy/default/projection behavior and bounded dialect accounting before implementation.
- Lockfile: `Cargo.lock` — inferred; adding the already-resolved `sha2 = "0.11"` to ess-openapi changes its dependency list (current package at line 425; sha2 at line 1208). Root Cargo.toml has no workspace sha2 entry and needs no edit.
- User overview: `README.md` — cited; lines 81–86 currently claim import produces ServiceInterface/1 and describe its accounting, requiring correspondence with the new persisted default.
- Internal catalog: `docs/design/review-format-catalog.md` — cited; lines 143 and 149 identify the current interface reader and imported projection, requiring a new import-envelope row and qualifications on the retained interface/projection rows.
- Public CLI reference: `website/docs/reference/cli.md` — cited; lines 126 and 128 describe OpenAPI import/projection, requiring the new output marker and checked --ir refusal/reimport behavior.
- Public format reference: `website/docs/reference/formats.md` — cited; lines 152 and 155 describe current interface and imported projection, requiring the envelope/profile/source-hash contract and the retained low-level API's limits.
- Existing compatibility evidence: current inline supported 3.1.0 roundtrip, reference and projection tests, plus `crates/edge/ess-cli/tests/command_surface.rs:165–182` and Taskfile import smoke — cited; they do not establish durable reload, legacy/default migration or tamper rejection.
- Intended verification: measured variant/site counterexamples, exact durable gap/refusal checks, checked write/read/project controls, frozen old-reader rejection, retained old /1 and supported projection bytes, and before/after package checks — inferred; coordinator owns integration/site and publication gates.
- Shared incoming work: `crates/edge/ess-cli`, `website/docs/reference/cli.md` and `website/docs/reference/formats.md` — cited; e113 adds normalize-generate in src/normalize.rs and src/schema.rs, sets main.rs:3519 leaf count to 51, adds cli.md:104 and updates formats.md:88 for Plan::go. Preserve these incoming changes; no normalization source edits are needed for F04.
- Would collide with: edits using the exact existing ess-openapi or ess-cli package tokens, the binding document, Cargo.lock, README.md, or either catalog/public reference file — inferred; normalization work overlaps the ess-cli reservation and the two public reference files even though its implementation symbols differ. Coordinate the reservation before concurrent dispatch.
- Confidence: high for current ownership, loss boundary and documentation drift — cited; the exact incoming objects and existing story identify them. This scope is an implementation story, not a documents-only change.
- Exclusions: schema-contract, ess-gen, normalization.rs/schema.rs implementation, broad OpenAPI support, new normalization targets, new public navigation or manifest surfaces, and an unestablished cross-repository verifier/ADR — inferred; none is needed to implement the proposed F04 contract. Coordinator-owned release/planning records remain outside the implementor unit.

## Matching typed scope commands

These commands are a complete desired path set, not an instruction to replay already-recorded
entries. The first three are already present at story revision 6; root should add only the five new
paths after binding. No command below has been run.

```sh
aep plan artifact scope story:review-openapi-semantic-accounting --add crates/generate/ess-openapi
aep plan artifact scope story:review-openapi-semantic-accounting --add crates/edge/ess-cli
aep plan artifact scope story:review-openapi-semantic-accounting --add docs/design/review-openapi-accounting.md --inferred
aep plan artifact scope story:review-openapi-semantic-accounting --add Cargo.lock --inferred
aep plan artifact scope story:review-openapi-semantic-accounting --add README.md
aep plan artifact scope story:review-openapi-semantic-accounting --add docs/design/review-format-catalog.md
aep plan artifact scope story:review-openapi-semantic-accounting --add website/docs/reference/cli.md
aep plan artifact scope story:review-openapi-semantic-accounting --add website/docs/reference/formats.md
```

## Acceptance delta proposed after binding

Keep the original acceptance sentence. Add these concrete observable boundaries:

1. Successful persisted imports use the accepted distinct closed/versioned envelope containing the unchanged ServiceInterface/1, exact retained UTF-8 source plus SHA-256, fixed normalization identity, admitted dialect, and deterministic normalization/gap/reference-site accounting. Immediate and reloaded accounting agree; no count-only replacement or omitted accounting is accepted.
2. Checked reload reimports the retained source and compares the complete derived result. Unknown/missing/duplicate wire fields or maps, changed source/digest/profile/dialect/interface, removed gaps and changed unresolved sites refuse; a self-consistent new source is admitted only as that new source. Repeat canonical writes are byte-identical.
3. Enum/const consumption depends on the admitted variant; missing items and all reviewed unsupported shapes always emit explicit gap/refusal before any component/property/message can disappear. Schema-reference siblings are accounted under admitted 3.1 semantics. Unsupported versions/dialects/resource interpretation refuse, while existing supported bare local references and string constraints remain supported. Use the binding's explicit cases and primary-spec distinctions.
4. The CLI switches its flat and area --out writers together. The old strict ServiceInterface/1 reader and canonical bytes remain unchanged and reject the new envelope. The new import-document boundary reports legacy accounting unavailable with an actionable source-reimport refusal; it never upgrades absent history into complete accounting. --format remains terminal presentation only.
5. Checked CLI --ir projection refuses partial, unresolved or legacy-unqualified imports before output mutation. Complete admitted imports, including allowed annotation-only normalizations, retain the supported projected YAML. The low-level structural ServiceInterface projection and native --path producer remain explicitly distinct.
6. Help, README and both catalogs accurately state the implemented reader/default/profile/hash/projection limits. Preserve the incoming normalize-generate command, 51-leaf count, recipe /1 and /2 rows, and Plan::rust/Plan::go target-report wording. Proposed tests and catalog prose are not substitutes for executed package/integration evidence.

## What is not established

- Root has not yet bound the proposed marker/profile names, retained-source policy, replay admission or strict projection/default migration; the separate binding document makes these choices concrete for root to settle.
- No external persisted ServiceInterface consumer or cross-repo byte verifier was found by the bounded local search; this does not enumerate published crates, installed binaries or users' saved files.
- No new tests, builds, compatibility execution, publication or source edits were performed. Existing tests were inspected only.
- Incoming source may advance again before dispatch; root must preserve the new exact source when rebasing reservations and briefs. This report does not establish remote main after the supplied e113 object.

## First independent review and correction route

Independent first review of frozen 633e1d9839dc0e8dae071e955f8e8af540c8a685 is preserved verbatim in review-result:review-boundaries-7-openapi-adversary-pass-1 (report SHA-256 4ac12af6456773e2698fd4a5b76e97ecd4acb68e87477c5df2957d17ef6cdda3). It executed 163 package cases: 159 passed, four failed, zero ignored. One introduced defect at crates/generate/ess-openapi/src/accounting.rs:152 accepts and erases unknown fields on integer, number and boolean schema nodes before replay equality. Actual checked library and CLI callers reach it. A fresh implementor receives the frozen diff, retained tests and target/review-boundaries-7/correction-pass-1.md because the original session ended; the existing legacy DTO/reader stays compatible. No green review or implemented status is claimed.
