---
format: aep.planning-md/3
id: story:openapi-import-admits-open-objects-as-json
kind: story
status: active
title: Openapi import admits open objects as json
summary: 'Adopter report R9: OpenAPI import refuses open objects and nullable anyOf (ess#647).'
tags:
- ess-0.70.0
refs:
- provider: github
  reference: beyond10x/ess#647
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-cli/tests/openapi_accounting.rs
- confidence: cited
  path: crates/generate/ess-openapi/src/accounting.rs
- confidence: cited
  path: crates/generate/ess-openapi/src/lib.rs
- confidence: cited
  path: crates/generate/ess-openapi/tests/adversary_pass1.rs
- confidence: inferred
  path: crates/generate/ess-openapi/tests/fixtures/interface_schema_0_76.rs
- confidence: inferred
  path: crates/generate/ess-openapi/tests/open_objects.rs
- confidence: cited
  path: docs/design/review-openapi-accounting.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T13:39:56Z", actor: "human:timo", revision: 4, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-29T05:17:20Z", actor: "human:timo", revision: 8, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
---
## Problem

`ess infra import openapi` refuses open objects (`additionalProperties: true` or absent), empty schemas (`{}`), properties without a `type`, and nullable `anyOf: [{$ref: X}, {type: 'null'}]`. A protocol whose openness is its forward-compatibility rule cannot be imported, and adopters write converters by hand. Adopter report R9, ess 0.68.0, plus a second adopter with the same refusal classes. The refusals were kept deliberately under the proposed decision record `ess-evolution-08-reviewable-source-candidates`; the 2026-10-23 decision below takes these constructs out of it.

## Fit review

Re-checked on 2026-10-29 against 0.76.0. Batch 1 did not change `crates/generate/ess-openapi`; every line cited below was read in the tree at 0.76.0. `crates/edge/ess-cli/src/main.rs` moved: the reader call the scoper cited at `:5918` is now `:5921`.

1. **Need.** A valid OpenAPI 3.1 document is refused whole, operations included, when any schema is open, untyped or nullable through `anyOf`/`oneOf`. Minimal reproduction (a repro outside the repository, one operation, schemas `Codec`, `PingRequest {properties: {data: {}}}`, `Meta {additionalProperties: true}`, `PingResponse` with `audio_codec: anyOf [$ref Codec, {type: null}]`): the 0.76.0 release binary's `ess infra import openapi` exits 1 with 6 refusals, at `Meta/additionalProperties`, `Meta/properties`, `PingRequest/additionalProperties`, `PingRequest/properties/data`, `PingResponse/properties/audio_codec` and `PingResponse/required` (identical to 0.74.0). The requester's proposed syntax: `Json`, a struct with `undeclared_fields: ignored`, and `Optional<X>`.
2. **Class.**
   - For the service-interface import it is a gap. The refusals are deliberate (`docs/design/review-openapi-accounting.md:106`, `:108`; `crates/generate/ess-openapi/src/lib.rs:1089-1100`, `:870-875`), so not a defect.
   - For realizing native types it is a convenience: see 3.
3. **Already expressible?**
   - Types: yes, through the bundle route (`ess generate schema import-bundle` then `types-bundle`), which exists since 0.19.0. On 0.74.0, `import-bundle --dialect draft-2020-12` exits 0 on the reproduction; `types-bundle` for Rust gives `data: EssPresence<serde_json::Value>`, `Meta { ess_extra }` and `audio_codec: Option<Box<Codec>>`; Go gives `EssNullable[*Codec]` and `EssExtra map[string]EssJSON`. The reply to the adopter points at this route for types.
   - Import: no. The envelope's only consumer is `ess generate project openapi --ir` (reader `ess_openapi::read_import`, `crates/edge/ess-cli/src/main.rs:5921`), and it yields no ESS source. That would be ADR `ess-evolution-08`, which stays out of scope.
4. **Fit with what exists.**
   - Reuse the bundle realizer's vocabulary: `Shape::Json` and `Shape::Object { additional }` (`crates/generate/schema-contract/src/realize.rs:475-505`; the scoper's `:389`, `:403-406` moved with batch 1's #686).
   - The model's `undeclared_fields: ignored` is a decoder policy, not a structural fact. The structural IR records `additional_properties`; dropping members stays a realizer choice (`--rust-drop-unknown-members`).
   - Null as a value is spelled `Optional<X>`, the name the ESS model already uses (`crates/specify/ess-compiler/src/ir.rs:293`).
   - Siblings: `{}` and `true` mean the same thing; a missing `items` means `items: {}`.
   - Siblings that do not agree, by decision: `type: [T, "null"]` and 3.0 `nullable: true` keep importing as `T` plus a `feature-unpreserved` gap (`lib.rs:905-938`, `:1134-1142`; design `docs/design/review-openapi-accounting.md:166`, `:170-173`). Changing them would change the replay of envelopes stored today: `read_import` reimports the retained source and refuses an envelope whose replay differs (`crates/generate/ess-openapi/src/accounting.rs:181-187`). This is the "a sibling cannot say the same thing" red flag, accepted here under D135; making the three spellings agree needs `ess-openapi-import/2` and is a follow-up, not this story.
   - Targets: re-projection (`schema_value`, `lib.rs:387-466`) and the replay reader (`read_import`, `accounting.rs:142-189`). There is no code-generation consumer.
5. **A second adopter?** One is already in the issue (117 of 176 refusals were "must be closed"). A brand-free example: a webhook event catalog whose payloads carry an open `metadata` map and a nullable `previous: {$ref}`.
6. **Cost.**
   - No `ess/N` and no envelope version: `ess-openapi-import/1`, `ess-openapi-service-subset/1` and `ess-service-interface/1` stay. The additions are additive, as the tuple variant and OpenAPI 3.0 support were (`CHANGELOG.md:3659`).
   - Two new IR kinds, `json` (a unit) and `optional` (`{kind: optional, schema}`), and one optional field, `Object.additional_properties`, omitted when absent so closed objects keep their bytes.
   - Every input accepted today imports byte-identically, so every stored envelope still replays. The new nodes arise only from inputs refused today.
   - An old reader refuses the new pieces by name: the scoper ran the 0.74.0 `ess generate project openapi --ir` on edited envelopes and both exit 1 (`unknown variant \`json\``, `unknown field \`additional_properties\``). `InterfaceSchema` is unchanged from 0.74.0 to 0.76.0, so the same holds for 0.76.0 (inferred from the unchanged source, not re-run); `kind: optional` is an unknown variant in the same way.
   - Some refusal texts change. `Refusal` is a string record (`lib.rs:247`), not a `ValidationCode`.
   - One amendment to `docs/design/review-openapi-accounting.md`.
7. **Alternatives.**
   - (a) Change nothing and point at the bundle route. Misses the operator's approval.
   - (b) The scoper's variant: a nullable `anyOf`/`oneOf` imports as `X` with the existing null gap, so all three spellings agree. Dropped by D135: the operator approved `Optional<X>` on 2026-10-23.
   - (c) An `Optional` for all three null spellings. Changes replay of stored envelopes, so it needs `/2` and a reimport path; deferred.
   - (d) Import every open object as bare `Json`. Loses the declared property types.
   - Chosen: the design in `## Decisions`.

## Decisions (2026-10-23)

- Decided on the operator's morning queue (row 33, report rows 44 and 45), option A: the import refusals become one unit in 0.70.0, after 0.69.0 ships. They are `additionalProperties: true` (or absent) and `{}` imported as `Json` or as a struct that ignores undeclared fields, and a nullable `oneOf`/`anyOf` of one schema and `{type: 'null'}` imported as `Optional<X>`. An untagged `oneOf` of objects stays refused by name.
- The acceptance above stands. `a_nullable_any_of_imports_as_optional` also covers the `oneOf` spelling.
- Out of 0.69.0. It does not wait on `ess-evolution-08-reviewable-source-candidates` for these constructs.

## Decisions

Decision: **accept, redesigned** (2026-10-29, batch 2 of the open-issue wave). Bound by `approval-record:openapi-import-open-objects-in-0700` and D135 (`approval-record:d135-a-breaking-unit-leaves-the-batch`). No `format: ess/N`; every envelope format stays at `/1`. The acceptance below replaces the 2026-10-23 list; the test names that changed are noted.

- **Empty and annotation-only schemas.** `{}`, `true`, and a schema with no `type` whose keys are only annotations or `$schema` import as `InterfaceSchema::Json` (`{kind: json}`). Annotations are recorded as normalizations, as today.
  - A schema with no `type` that carries any other assertion keeps today's refusal.
  - `false` stays refused.
  - A missing `items` imports as an array of `Json`.
  - `json` is a unit kind: `check_unit_schema_fields` (`accounting.rs:195-222`) admits it with no other field, so an envelope cannot smuggle fields onto it.
- **Objects.** `Object` gains `additional_properties: Option<Box<InterfaceSchema>>`:
  - `false` → `None` (closed, unchanged bytes)
  - absent, `true` or `{}` → `Json`
  - a schema → that schema (a typed map)
  - A missing `properties` means `{}`. So `{type: object, additionalProperties: true}` becomes an object with no properties and `Json` members, not bare `Json`; this keeps the "must be an object" constraint. Changed from the request, which asked for `Json`.
  - Not `undeclared_fields: ignored`: that is a decoder policy of the model, and the structural IR records what the schema admits. Changed from the request.
- **Nullable unions, as approved.** An `anyOf` or `oneOf` of exactly one schema S and `{type: 'null'}`, in either order, imports as `InterfaceSchema::Optional { schema: S }` (`{kind: optional, schema: …}`), and records no gap: the null member is preserved.
  - Re-projection writes `anyOf: [<S>, {type: 'null'}]`. For a pair of S and null the two keywords and both orders mean the same thing, so the spelling is normalized without an accounting entry; reimport of the projection gives an equal IR.
  - Checked projection admits it (no gap at that site).
  - If S itself admits null (a `type: [T, "null"]`, a 3.0 `nullable`, or another `optional`), it is refused at `…/anyOf` or `…/oneOf`: the pair would be ambiguous under `oneOf`.
  - A `required` property whose schema is `optional` means "present, possibly null"; absence stays expressed by `required`.
  - The other two null spellings are unchanged: `type: [T, "null"]` and 3.0 `nullable: true` still import as `T` with the `feature-unpreserved` gap. The scoper's null-gap variant for `anyOf`/`oneOf` is dropped (D135). Making the three agree is a follow-up for `ess-openapi-import/2`.
- **Other unions.** Any other `anyOf`, `oneOf`, `allOf` or `not`, including an untagged `oneOf` of objects, is refused at `…/<keyword>`. The refusal names the union instead of the generic "must declare `$ref`".
- **Required names not in `properties`.**
  - On an open object it is admitted as written: it means "required member of the extra-member schema".
  - On a closed object it stays refused, because no value can validate.
  - A name whose property schema was itself refused is no longer reported a second time as "not declared". This replaces the 2026-10-23 test `a_required_property_not_declared_is_reported_not_refused`: a refusal on a closed object is kept because no value validates, so it is not a coverage gap.
- **Unchanged.** `crates/edge/ess-cli/src/main.rs` is untouched; its reader is `ess_openapi::read_import` (`main.rs:5921`). OpenAPI 3.0's `{type: 'null'}` stays refused (3.0 has no null type); the implementor confirms it in `openapi30_schema` (`lib.rs:1255-1320`, comment at `:1296-1297`) with `an_openapi30_null_branch_stays_refused`.
- **Renamed tests from the 2026-10-23 list.** `an_open_object_imports_as_a_struct_that_ignores_undeclared_fields` → `an_open_object_imports_with_any_other_member`; `a_nullable_any_of_imports_as_optional` keeps its name and covers `oneOf`; `a_required_property_not_declared_is_reported_not_refused` → the two `a_required_member_outside_properties_*` tests.

## Acceptance

- Previously accepted documents keep their exact bytes. The existing `checked_envelope_preserves_exact_source_and_legacy_bytes` stays green unchanged: `cargo test -p ess-openapi --locked --test accounting checked_envelope_preserves_exact_source_and_legacy_bytes`.
- New file `crates/generate/ess-openapi/tests/open_objects.rs`; run each with `cargo test -p ess-openapi --locked --test open_objects <fn>`:
  - `an_empty_schema_imports_as_json`: `{}`, `true` and `{description}`, at all 5 sites.
  - `an_open_object_imports_with_any_other_member`: absent and `true`; projection then reimport gives an equal IR.
  - `a_typed_map_imports_its_member_schema`
  - `an_object_without_properties_imports`: open, and closed-empty.
  - `an_array_without_items_imports_as_an_array_of_json`
  - `a_nullable_any_of_imports_as_optional`: `anyOf` and `oneOf`, both orders, with `$ref` and with an inline schema; no gap is recorded, checked projection succeeds, and projection then reimport gives an equal IR.
  - `the_other_null_spellings_keep_their_gap`: `type: [T, "null"]` and 3.0 `nullable: true` import exactly as on 0.76.0 (`T` and the `feature-unpreserved` gap).
  - `an_untagged_object_one_of_stays_refused`: also `allOf`, `not`, and a 3-branch `anyOf`; the refusal names the keyword.
  - `a_one_of_whose_other_branch_admits_null_stays_refused`: S as `type: [T, "null"]`, as a nested nullable `anyOf`, and in a 3.0 document as `nullable: true`.
  - `a_required_member_outside_properties_imports_on_an_open_object`
  - `a_required_member_outside_properties_stays_refused_on_a_closed_object`
  - `a_refused_property_is_not_also_reported_as_undeclared`
  - `a_typeless_schema_with_assertions_stays_refused`
  - `an_openapi30_null_branch_stays_refused`
  - `the_reported_document_imports`: the minimal reproduction shape of #647 imports with exit 0; its only gap is the integer `format` at `PingResponse/properties/count/format`.
  - `a_0_76_reader_refuses_the_new_nodes_by_name`: a frozen copy of the 0.76.0 `InterfaceSchema` in `tests/fixtures/interface_schema_0_76.rs` refuses `kind: json`, `kind: optional` and `additional_properties` by name.
  - `closed_import_rejects_unknown_json_variant_fields`: `{kind: json, extra: 1}` in an envelope is refused by `read_import`.
- Updated in place, run with `cargo test -p ess-openapi --locked`:
  - `adversary_pass1.rs::unsupported_items_cannot_disappear_at_nested_or_message_sites` (`tests/adversary_pass1.rs:190-209`) keeps only `false` and `["string"]`.
  - `lib.rs` test `review_missing_array_items_refuses_at_every_site` (`lib.rs:1619-1621`) is replaced by the `open_objects.rs` items test.
- CLI: `an_open_document_imports_and_checked_projection_names_only_the_format_gap`: `cargo test -p ess-cli --locked --test openapi_accounting an_open_document_imports_and_checked_projection_names_only_the_format_gap`.

## Scope

Derived 2026-10-28 by `story-scoper`, re-checked against 0.76.0 on 2026-10-29. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/generate/ess-openapi` — cited
- **Files:** `crates/generate/ess-openapi/src/lib.rs` — `InterfaceSchema` :195-243 (gains `Json`, `Optional`, `Object.additional_properties`), `schema_value` :387-466, `schema` :813-903, `null_admitting` :905-938 (unchanged behaviour), `object_schema` :1084-1131, `null_unpreserved` :1134-1142, test :1619-1621 — cited
- **Files:** `crates/generate/ess-openapi/src/accounting.rs` — `check_unit_schema_fields` :195-222 gains `json` — cited
- **Files:** `crates/generate/ess-openapi/tests/adversary_pass1.rs` :190-209 — cited
- **Files:** `crates/generate/ess-openapi/tests/open_objects.rs` and `tests/fixtures/interface_schema_0_76.rs` (new) — inferred
- **Also likely:** `crates/edge/ess-cli/tests/openapi_accounting.rs` — inferred, one end-to-end test; `crates/edge/ess-cli/src/main.rs` unchanged — cited (:5921)
- **Documents:** `docs/design/review-openapi-accounting.md` rows :106, :108 and an amendment beside :166-173 — cited; `CHANGELOG.md` — inferred
- **Confidence:** high — every refusal site and the reader were read; the old-reader behaviour was run on 0.74.0
- **Would collide with:** any unit touching `ess-openapi`; `CHANGELOG.md` (every unit). Disjoint from `schema-contract`, so it does not collide with the second story of U6 or with U7.
- **Safety fact:** every stored `/1` envelope replays because closed objects serialize unchanged, the other null spellings import unchanged, and the new nodes arise only from inputs refused today; the 0.74.0 reader refuses the new nodes by name — level 4 (ran the release binary). Open risk at level 2, unproven: a new unit kind `json` silently drops unknown fields unless it is added to `check_unit_schema_fields` (`accounting.rs:195-222`); `closed_import_rejects_unknown_json_variant_fields` covers it.
