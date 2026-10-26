---
format: aep.planning-md/3
id: story:a-lifecycle-state-carries-a-wire-name
kind: story
status: active
title: A lifecycle state carries a wire name
summary: 'Adopter report G4: A lifecycle state cannot carry a wire spelling (ess#639).'
tags:
- ess-0.70.0
refs:
- provider: github
  reference: beyond10x/ess#639
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T09:18:55Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-10-26T09:18:56Z", actor: "human:timo", revision: 8}
---
## Problem

A lifecycle state is a bare name, so a service whose wire spelling differs (`indexing` for `Indexing`) needs a separate enum kept in step by hand. `states: [{name:, wire:}]` is refused as "invalid type: map, expected a string". Reported by two adopters (G4 and an earlier report), ess 0.67.0 and 0.68.0. The enum variant's wire name (`RawEnumVariant`, `crates/specify/ess-domain/src/types.rs`) is the pattern. Already planned under unit U19 of the 0.69.0 wave; not yet in the store.

## Acceptance

Each line names the failing test written first, as a case in an existing test binary.

- `crates/specify/ess-domain/tests/enum_variant_wire_names.rs`:
  `a_lifecycle_state_declares_its_wire_name_from_ess_32` (validates; the state's name is unchanged),
  `a_state_wire_name_below_ess_32_is_refused_naming_ess_32` (at the state's line),
  `two_states_with_one_wire_spelling_are_refused`,
  `two_enum_variants_with_one_wire_spelling_are_refused_from_ess_32` and
  `two_enum_variants_with_one_wire_spelling_still_validate_below_ess_32`.
- `crates/specify/ess-compiler/tests/state_change_ir.rs`:
  `a_bare_lifecycle_keeps_its_ir_bytes` and `a_state_wire_name_reaches_the_state_enum_variant`.
- `crates/generate/ess-gen/tests/enum_variant_names.rs`:
  `a_state_wire_name_is_the_state_enums_wire_value` (JSON Schema `enum`, OpenAPI and AsyncAPI list
  `indexing`; `x-ess-variants` names `Indexing`).
- `crates/generate/schema-contract/tests/enum_variant_names.rs`:
  `generated_types_spell_a_state_by_its_wire_name` (Rust, Go and TypeScript decode `"indexing"`,
  refuse `"Indexing"`).
- `crates/generate/ess-synth/tests/served_codec_names.rs`:
  `a_served_view_answers_a_state_by_its_wire_name`.
- `crates/verify/ess-diff/tests/enum_variant_wire.rs`:
  `a_changed_state_wire_name_is_reported`.
- `crates/verify/ess-conformance/tests/field_wire_names.rs`:
  `a_state_wire_name_leaves_the_suite_bytes_unchanged` (the suite carries `Indexing`).

## Fit review

Reviewed 2026-10-26 against main `a3bc2df3d` (0.69.2). Reproduction:
`fit/repro/g4-state-wire.yaml` (one entity, `states: [{name: Active, wire: active}]`), refused on
0.69.2 as `invalid type: map, expected a string` (whole file, no line, no code).

1. **Need.** A state the service spells differently on the wire (`indexing` for `Indexing`)
   cannot say so; the adopter keeps a second enum in step with the lifecycle by hand. Requester's
   syntax: `states: [{name: Indexing, wire: indexing}]` (issue G4; a second adopter reported the
   same refusal).
2. **Class.** Gap. Every other named thing that reaches the wire can declare its spelling: enum
   variants (`ess/5`, `docs/design/enum-variant-wire-names.md:60-66`), errors (`ess/4`), fields
   (`naming: {wire:}`); a state cannot (`StateMachine.states: BTreeSet<StateName>`,
   `crates/specify/ess-domain/src/entity.rs:344-346`, raw form `:1840-1851`).
3. **Already expressible?** No. The idiom (a separate enum field set on every transition) adds a
   second fact the suite checks, while `<Entity>.State` is still projected as `Indexing`; it does
   not spell the state, so it fails policy 1 (`docs/design/generic-language-features.md` § Policy).
4. **Fit.** Not a grid R/O cell (`docs/design/language-grids.md`): it closes the naming facet for
   the one named construct lacking it. Reuses the variant's object form and `Naming` (`wire:`).
   The state enum is already built as an enum (`EntitySpec::state_type`, `entity.rs:987-1001`)
   and resolved like a declared type (`crates/specify/ess-compiler/src/resolve.rs:4260-4276`), so
   giving its variants the state's `Naming` reaches every projection that honours a variant's wire
   (`crates/generate/ess-synth/src/rust/wire.rs:149-157`, `go/http.rs:582`). Guards, `moves:`,
   `when_subject_state:` and suites keep the declared name, as for enum variants: a suite
   addresses values by declared names and the adapter spells the wire
   (`crates/verify/ess-conformance/tests/field_wire_names.rs:4-7`; variants by `name()`,
   `crates/verify/ess-conformance/src/input.rs:488`).
5. **Second adopter.** A job queue whose API answers `queued`, `running`, `done` for states
   `Queued`, `Running`, `Done`.
6. **Cost.** `ess/32` (object form admitted; bare names keep their bytes); a stricter rule
   (duplicate wire spellings, below); IR gains an optional per-state naming, skipped when empty so
   existing IR keeps its bytes; `ess-diff` needs a state-wire change case (new case, possibly an
   `ess-diff` minimum-format move: I don't know whether `<Entity>.State` already appears in
   `types()` at `crates/verify/ess-diff/src/diff.rs:696` and would be reported as
   `VariantWireNameChanged`). No new diagnostic class.
7. **Alternatives.** (a) Change nothing: the shadow enum, rejected (point 3). (b) A case policy on
   the lifecycle (`wire_case: snake`): no such construct anywhere else, rejected. (c) The
   variant's `{name, wire}` per state, lowered onto `<Entity>.State`'s variants (chosen). The
   requester's syntax is taken unchanged; its last expectation (suites compare by wire) is changed.

## Decisions

- **Accept, redesigned.** The syntax is the requester's; the design differs in where the spelling
  is read: it lowers onto the variants of `<Entity>.State`, so the projections that honour an enum
  variant's wire spelling honour a state's with no code of their own. Suites, the interpreter and
  every guard keep the declared state name, as for enum variants and fields; the story's line
  "a target answering `Indexing` fails" is replaced by the generated adapters' wire checks.
- From `ess/32`. Below it the object form is refused as `unsupported_format_version` naming
  `ess/32`, at the state's line, replacing today's whole-file `invalid type: map`.
- Two states of one lifecycle with one wire spelling are `conflicting_declaration`. The same rule
  is added for enum variants (two variants spelled `indexing` validate today: `fit/repro/
  g4-enum-dup.yaml`, valid on 0.69.2) as an `ess/32` `stricter:` entry; below `ess/32` enums keep
  validating.
- The model keeps `states` as the set of declared names; the spelling is held beside it (a map from
  state to `Naming`, skipped when empty), so the many readers of `.states` do not move.
- Entity Runtime stores the declared name; whether its HTTP surface must spell the wire name: I
  don't know, settle at implementation (refuse by name if not handled).

## Scope

| unit | files | confidence |
|---|---|---|
| parse and model | `crates/specify/ess-domain/src/entity.rs` (`RawStateMachine` `:1840-1862`, `StateMachine` `:344-357`, `state_type` `:987-1001`) | cited |
| gate | `crates/specify/ess-domain/src/system.rs` (`SUPPORTED_FORMATS` `:53`, `FORMAT_HISTORY` `:78`, new `V32` beside `V31` `:471`), `primitive_admission.rs` (beside the variant-naming gate `:231`) | cited |
| duplicate wire rule | `crates/specify/ess-domain/src/types.rs` (enum shape check), `entity.rs` lifecycle validation | inferred |
| IR | `crates/specify/ess-compiler/src/resolve.rs` (`state_type` `:4266`, `lifecycle` `:4286`), `src/ir.rs` | inferred |
| projections | none expected beyond the IR's state enum (`ess-gen` `types.rs`, `ess-synth` `rust/wire.rs:149-157`, `go/http.rs:582`, `schema-contract` realizers read variant wire already) | inferred |
| diff | `crates/verify/ess-diff/src/diff.rs` (`lifecycle_changes` `:1439`), `src/change.rs` | inferred |
| Entity Runtime | `crates/generate/ess-entity-runtime` | inferred (I don't know whether it touches wire spelling) |
| schema and docs | `schemas/generated/ess.schema.json` (`cargo xtask schema`), `website/docs/reference/spec-versions.md` (`ess/32` row), `docs/design/enum-variant-wire-names.md` | inferred |

Format: new source format `ess/32` (added + one `stricter:` entry). Crates: ess-domain,
ess-compiler, ess-diff, ess-gen/ess-synth/schema-contract (tests only if the lowering holds),
ess-conformance (tests only). Size L. Shares `system.rs`'s `ess/32` row with every other `ess/32`
story of the wave (`story:a-struct-that-ignores-undeclared-fields-may-declare-none`); serialize the
`FORMAT_HISTORY` edit.

confidence: medium
