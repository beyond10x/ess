---
format: aep.planning-md/3
id: story:entity-runtime-lowering-enforces-input-type-invariants
kind: story
status: implemented
title: Entity Runtime lowering enforces, or refuses by name, the invariants of an unstored input's type
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/adversary_alphabet_count_pass1.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv
- confidence: cited
  path: website/docs/reference/entity-runtime-lowering.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T06:49:18Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-01T06:49:19Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-11-01T10:37:57Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Finding

Measured by adversary pass 1 of wave 2026-10-07b
(`review-result:adversary-wave-20261007b-u1-pass-1`, F3), pre-existing on `main`: the Entity Runtime
lowering visits the invariants of entity identity and stored field types only
(`crates/generate/ess-entity-runtime/src/lib.rs:936`). A command input whose type carries
`invariants:` and is not stored has them dropped, so entity-core accepts an input ESS rejects.
Example: a `Label` newtype with `value.count <= 3`; ESS rejects `"abcd"`, the lowered definition
takes it.

## Acceptance

Each invariant of an unstored input's type either lowers to a precondition entity-core enforces, or
is refused by name with a lowering code; none is dropped. The adversary case
`adv_an_input_only_text_length_invariant_…` in
`crates/generate/ess-entity-runtime/tests/adversary_alphabet_count_pass1.rs` flips from asserting
today's behaviour to asserting the new one.

## Scope

Derived 2026-11-01 by `aep:story-scoper` for wave D343 on 45a1beba7. Every line is cited or inferred.

- **Primary surface:** `crates/generate/ess-entity-runtime` — cited. The finding's `lib.rs:936` has drifted (it is in `check_options`); the invariant visit is `build_definitions` `lib.rs:992-1019`.
- **Files:** `src/lib.rs` — `lower_nominal_invariants` (:1361, newtype arm :1428-1500), `schema_for_fields` (:2182-2204, calls `lower_field` only, so input-type invariants never reach the nominal walker), `lower_command` input (:1969), `OperationDefinition.preconditions` (:2171, always empty), `LoweringCode` (:433), `PathRewrite::Input` (:4276), `optional_guard` (:4016) — cited.
- **Files:** `src/subset.rs` — invariant row (:190-193), `LoweringCode::ALL` (:673), code names (~:869) — cited.
- **entity-core 0.28.0:** `OperationDefinition.preconditions: Vec<RuleDefinition>` (`definition.rs:703`) reads `$args.*`; `CreateDefinition` (`definition.rs:660`) has no precondition slot, and `OutcomeDefinition.when` would change branch selection — cited.
- **Tests:** `tests/adversary_alphabet_count_pass1.rs:578` (flips), `tests/lowering.rs`, `tests/lowerable_subset.rs` — cited; `tests/lowered_definitions_table.rs` + `tests/fixtures/lowered-definitions-table.tsv` only if a pinned model has an input-type invariant — inferred.
- **Documents:** `website/docs/reference/entity-runtime-lowering.md`, generated from `subset.rs` and byte-pinned by `lowerable_subset` — cited.
- **Confidence:** high on the seam; the creation case needs the design rule in `## Decisions`.
- **Would collide with:** any unit editing `ess-entity-runtime` lowering, `subset.rs` rows or the lowered-definitions TSV.

## Decisions

D343, 2026-11-01 (coordinator, inside the orchestrator-approved item): an invariant of an unstored
input's type lowers to what entity-core enforces — an `OperationDefinition.preconditions` rule over
`$args.input.<field>` where the operation has that slot, or a schema constraint where the existing
field walker already maps that predicate (alphabet, text count) — and anything else is refused by
name with a lowering code listed in `subset.rs`. A creation has no precondition slot in entity-core
0.28.0 (`CreateDefinition`, `definition.rs:660`), and putting the invariant into an outcome's `when`
would change which branch answers, so an invariant a creation cannot enforce is refused by name.
Nothing is dropped. The generated reference page and the lowered-definitions table move with it.

## Delivered

D343, 2026-11-01: unit 453cdd71a and correction 1c8ea5dd7 (merged 67a6eb77b, 0c4cc25cc). Scope check:
`lib.rs`, `subset.rs`, the lowered-definitions TSV and the generated reference page confirmed. An
ordering is lowered typed (Timestamp `before`/`after`, number `compare`); a text ordering or one
whose fact type cannot be resolved is refused `InputInvariantUnsupported` (`INPUT_RULE_ORDERING`).
Adversary pass 1: F1, F2 fixed; F3 dismissed (entity-core answers a refusing branch before
preconditions; nothing is accepted); F4 became `story:entity-runtime-lowers-stored-field-type-orderings-typed`.
