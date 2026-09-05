---
format: aep.planning-md/1
id: story:review-openapi-semantic-accounting
kind: story
status: draft
title: Account for every unpreserved OpenAPI constraint
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/generate/ess-openapi
- confidence: inferred
  path: docs/design/review-openapi-accounting.md
revision: 6
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
