---
format: aep.planning-md/3
id: story:a-struct-that-ignores-undeclared-fields-may-declare-none
kind: story
status: implemented
title: A struct that ignores undeclared fields may declare none
summary: 'Adopter report R11: A struct ignoring undeclared fields cannot declare none (ess#649).'
refs:
- provider: github
  reference: beyond10x/ess#649
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T09:18:55Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-26T09:18:55Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-10-26T14:51:52Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Problem

ESS-TYPE-007 refuses a struct with `fields: []` because "no value can be one" (`crates/specify/ess-domain/src/types.rs:1254`). That holds for a closed struct only: with `undeclared_fields: ignored` every JSON object is a value. Adopters model empty wire objects as a `newtype` of `Map<String, Json>` instead. Adopter report R11, ess 0.68.0.

## Acceptance

Each line names the failing test written first, as a case in an existing test binary.

- `crates/edge/ess-cli/tests/validate_undeclared_fields.rs`:
  `an_open_struct_with_no_fields_validates_from_ess_32`,
  `a_closed_struct_with_no_fields_validates_from_ess_32`,
  `a_struct_with_no_fields_below_ess_32_is_refused_naming_ess_32` (ESS-TYPE-007, message
  unchanged, no ESS-COMMAND-001 cascade at the input using it), and
  `an_enum_with_no_variants_is_still_refused_at_ess_32`.
- `crates/generate/ess-gen/tests/undeclared_fields.rs`:
  `an_empty_struct_projects_as_an_open_or_closed_object` (JSON Schema, OpenAPI, AsyncAPI).
- `crates/generate/schema-contract/tests/rust_realization.rs`, `go_realization.rs`,
  `typescript_bindings.rs`: `an_open_empty_struct_decodes_any_object` (`{}` and `{"x": 1}`) and
  `a_closed_empty_struct_decodes_only_the_empty_object`.
- `crates/verify/ess-conformance/tests/undeclared_fields.rs`:
  `an_empty_struct_input_synthesizes_and_passes_on_the_interpreted_target`.

## Fit review

Reviewed 2026-10-26 against main `a3bc2df3d` (0.69.2). Reproduction: `fit/repro/r11-open-empty.yaml`
(`kind: struct`, `undeclared_fields: ignored`, `fields: []`, used as a command input) is refused on
0.69.2 as ESS-TYPE-007 `` `fit.notes.Ack` declares no fields, so no value can be one ``, and the
refusal cascades into ESS-COMMAND-001 (`fit.notes.Ack` is not a declared type) at the input that
uses it. The same file with one optional field (`fit/repro/r11-control.yaml`) is `valid`; the closed
variant (`fit/repro/r11-closed-empty.yaml`) is refused the same way.

1. **Need.** A wire object with no fixed members (an acknowledgment, an extension bag) cannot be
   declared as a struct; adopters fall back to a `newtype` of `Map<String, Json>`. Requester's
   syntax: `fields: []` beside `undeclared_fields: ignored` (issue R11).
2. **Class.** Defect: the refusal's reason is false. With `undeclared_fields: ignored` every object
   is a value; with the default (closed) `{}` is one. Its rationale is generated code that does not
   compile (`crates/specify/ess-domain/src/types.rs:1235-1240`), which no longer holds: the Rust
   realizer emits a zero-field closed record (`#[serde(deny_unknown_fields)]`) and an open one
   holding only `ess_extra` (`crates/generate/schema-contract/src/realize/rust.rs:504-552`, the
   zero-field branch at `:540`), TypeScript handles both (`realize/ts.rs:99-105`).
3. **Already expressible?** Only by the `Map<String, Json>` newtype, which is a different value
   (a map, not a struct; no `undeclared_fields:`), so not an idiom for the declared shape.
4. **Fit.** Not a grid R/O cell (`docs/design/language-grids.md`); it relaxes one refusal of a
   type kind. Siblings already admit no fields: an event or an error with `fields: []` validates
   (the errors of `fit/repro/r11-open-empty.yaml`). Enums and unions with no variants keep the
   refusal: they have no value. A command with `undeclared_fields:` and no `response:` keeps
   `missing_declaration` (`crates/specify/ess-domain/src/undeclared_fields.rs:14-19`): no response
   object exists there to open.
5. **Second adopter.** A webhook receiver acknowledges with `{}` and must accept vendor members it
   does not know.
6. **Cost.** `ess/32` (an admission); below it the refusal stays, its hint naming `ess/32`. No new
   keyword, no IR shape (an empty field list is already representable), no diagnostic class. The
   check moves: `check_shape` runs in `TryFrom<RawNamedType>` (`types.rs:2340`) before the struct's
   `undeclared_fields:` is set (`types.rs:1924-1930`, set at `undeclared_fields.rs:223`) and without
   the document's format, so the struct case moves to `TypeRegistry::validate_invariants`
   (`types.rs:2602-2626`), which has both. Side effect: below `ess/32` the refused struct no longer
   drops out of the registry, so the ESS-COMMAND-001 cascade disappears.
7. **Alternatives.** (a) Change nothing: the newtype workaround. (b) Admit only the open struct
   (the request): leaves a closed `{}` refused for a reason that is false and unlike events and
   errors. (c) Admit `fields: []` on every struct from `ess/32` (chosen).

## Decisions

- **Accept, redesigned.** From `ess/32` a struct may declare `fields: []`, open or closed: open,
  every object is a value and it projects as `{type: object, additionalProperties: true}`; closed,
  `{}` is its only value. The request asked for the open case only; the closed case is admitted
  too because the refusal's reason is false for it and events and errors already take no fields.
- Below `ess/32` an empty struct keeps ESS-TYPE-007, message unchanged, help naming `ess/32`.
- Enums and unions with no variants keep ESS-TYPE-007 at every format.

## Scope

| unit | files | confidence |
|---|---|---|
| refusal moved and gated | `crates/specify/ess-domain/src/types.rs` (`check_shape` `:1240-1274`, `TryFrom<RawNamedType>` `:2328-2343`, `validate_invariants` `:2602-2626`) | cited |
| gate | `crates/specify/ess-domain/src/system.rs` (`SUPPORTED_FORMATS` `:53`, `FORMAT_HISTORY` `:78`, new `V32` beside `V31` `:471`) | cited |
| generated types | none expected: `crates/generate/schema-contract/src/realize/rust.rs:540`, `ts.rs:99-105` already handle zero fields; `go.rs:216-286` | cited (Rust, TS), inferred (Go) |
| projections | `crates/generate/ess-gen/src/types.rs:897-902` (struct schema, `Additional::of`) | inferred, no change expected |
| docs | `website/docs/reference/spec-versions.md` (`ess/32` row), `website/docs/reference/diagnostics.md` (ESS-TYPE-007 text) | inferred |

Format: new source format `ess/32` (added). Crates: ess-domain, tests in ess-cli, ess-gen,
schema-contract, ess-conformance. Size S. Shares `system.rs`'s `ess/32` row with
`story:a-lifecycle-state-carries-a-wire-name` and `types.rs` (enum shape check) with that story's
duplicate-wire rule; serialize the two.

confidence: high
