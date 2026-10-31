---
format: aep.planning-md/3
id: story:a-transition-and-a-state-may-carry-a-summary
kind: story
status: active
title: A lifecycle transition and a lifecycle state may carry a summary
summary: 'Transitions and states take no summary in any form, and a flat summary: on an enum type is refused without pointing at naming: (harness, 0.77.0).'
tags:
- adopter-report
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: cited
  path: crates/specify/ess-domain/src/primitive_admission.rs
- confidence: cited
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
- confidence: cited
  path: crates/verify/ess-diff/src/change.rs
- confidence: cited
  path: crates/verify/ess-diff/src/delta.rs
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
- confidence: cited
  path: models/toolchain/domains/specify.yaml
- confidence: cited
  path: schemas/generated/ess.schema.json
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-10-31T11:37:42Z", actor: "human:timo", revision: 15}
- {from: "proposed", to: "active", at: "2026-10-31T11:37:42Z", actor: "human:timo", revision: 16}
---
## Need

Reported by the harness specification session (2026-10-30, ESS 0.77.0 f692aab), while modelling an
agent loop whose every state and transition cites the code that performs it: the citations can only
be YAML comments, which no tool reads.

- A lifecycle transition takes no summary in any form: `summary:` and `naming: {summary: …}` are
  both refused as unknown fields of `Transition` (`name`, `from`, `to` under
  `deny_unknown_fields`, `crates/specify/ess-domain/src/entity.rs:282-283`).
- A lifecycle state takes no summary in any form: its mapping form is `name` and `wire` only
  (`SpelledState`, `entity.rs:1998-2005`).
- A flat `summary:` on an enum type is refused as an unknown field ("expected `variants` or
  `attributes`") without pointing at `naming: {summary: …}`, which the enum type does take. A
  struct field and an enum variant take a flat `summary:`, so the flat form is easy to write by
  mistake.

## Triage on main (f692aab, 0.77.0)

Re-run 2026-10-31 with the 0.77.0 binary over the reporter's eight one-line variants: `control`,
`enum-type-naming` and `enum-variant` valid; `transition`, `transition-naming`, `state`,
`state-naming` and `enum-type` refused with the messages above. Not fixed.

## Requester's proposal (theirs)

None beyond the observation; the variants tried were a flat `summary:` and `naming: {summary: …}`
on a transition and on a state.

## Where it lives

Lines on `main` f692aab (0.77.0), read by the scoper on 2026-10-31.

- `crates/specify/ess-domain/src/entity.rs:283` `Transition` (`name`, `from`, `to`,
  `deny_unknown_fields`); `:2000` `SpelledState` (`{name, wire}`, `wire` required), `RawState::Spelled`
  at `:1974`; `:368` `StateMachine.naming`, filled from `wire` at `:1940`; `:1037` `state_type()`
  copies each state's naming into its variant of `<Entity>.State`; `:1916` `conflicting_spellings`.
- `crates/specify/ess-domain/src/primitive_admission.rs:664-678`: the `ess/32` gate fires on any
  `naming` entry, not only a wire; `crates/generate/ess-entity-runtime/src/lib.rs:1058` likewise.
- `crates/specify/ess-domain/src/types.rs:2239-2247` `RawNamedType` holds `naming` and flattens its
  body; `:1795` `RawTypeBody` refuses unknown keys per kind and never names `naming`.
- `crates/verify/ess-diff/src/diff.rs:1439` compares only a transition's route; `:2824,2831` the
  residual leaves any other transition key unclassified.

## Fit review

1. **Need.** A lifecycle's states and moves cannot carry a one-line description a tool reads.
   Minimal reproduction: `states: [{name: Closed, summary: …}, Open]`, `transitions: [{name: open,
   from: [Closed], to: Open, summary: …}]`. The requester tried flat `summary:` and
   `naming: {summary: …}` on both.
2. **Class.** Transition and state: gap. Enum type: a misleading refusal for what is already
   expressible (`naming: {summary: …}` validates).
3. **Already expressible?** Not for transitions or states (the 0.77.0 runs in the triage above). An
   outcome's flat `summary:` describes the outcome, not the move, and a state has no host.
4. **Fit.** A member of a declaration writes `summary:` flat: a field (`types.rs:577`), an enum
   variant (`types.rs:889`), an outcome (`command.rs:2634`); a state's mapping is a variant's flat form
   with one more key. Guards, `moves:` and suites keep naming states and moves by name. No projection
   reads a variant's summary today (only `ess-diff`, `diff.rs:518`), so generated docs skip these as
   they skip variant summaries.
5. **Second adopter.** A payments service documents `Held` and its `release` move.
6. **Cost.** `ess/33` (shared with the wave), `ess-diff/22` for one new diff kind, no new
   `ValidationCode`, schema regeneration, no migration. `Transition` gains a public field.
7. **Considered.** (a) change nothing: a state has no host; (b) `naming: {summary}` on both: a
   transition has no wire, display or code; (c) both spellings: two spellings for one idea; (d) flat:
   chosen. For the type, a flat synonym is refused; the parse-time message names `naming:`.

## Decisions

**accept, redesigned** (wave D325). Not ADR-level: the constructs take the spelling their siblings
use. Format: `ess/33`, `ess-diff/22`.

- Transition: `summary` optional, skipped when absent.
- State: the mapping form is `{name, wire?, summary?}` with at least one of the two; `{name}` alone
  stays refused with "write the state as the name". The summary is stored in
  `StateMachine.naming[state].summary`, so it reaches the variant of `<Entity>.State`; two
  declarations of one state with different summaries are `conflicting_declaration`.
- `primitive_admission.rs` and `ess-entity-runtime` move from "has naming" to "has a wire", so a
  summary-only state is not refused as a wire spelling.
- Below `ess/33` either is refused as `unsupported_format_version` naming `ess/33`, at the state or
  transition.
- Diff: a moved state summary is `<Entity>.State`'s existing `variant-summary-changed`; a moved
  transition summary is a new `transition-summary-changed`, compatible, minimum `ess-diff/22`.
- Type: a flat `wire`/`display`/`summary`/`code` on a named type is refused with the spelling that
  reads it, `naming: {summary: …}` (a parse problem, no `ValidationError`).
- Out of scope: rendering member summaries in generated docs and the web catalog; summaries on
  relations and invariants.
- CHANGELOG `[Unreleased]`: Added (lifecycle summaries, `ess/33`, `transition-summary-changed`) and
  Changed (the flat-naming refusal on a type; `Transition` and `RawState::Spelled` gain fields).

## Acceptance

- `crates/specify/ess-domain/tests/enum_variant_wire_names/state_wire.rs`:
  `a_lifecycle_state_and_transition_carry_a_summary_from_ess_33`;
  `a_lifecycle_summary_below_ess_33_is_refused_naming_ess_33`.
- `crates/specify/ess-compiler/tests/state_change_ir/state_wire.rs`:
  `a_state_and_transition_summary_reach_the_ir`; `a_bare_lifecycle_keeps_its_ir_bytes` stays green.
- `crates/verify/ess-diff/tests/canonical.rs`:
  `a_transition_summary_edit_is_compatible_documentation` (before `unclassified-changed`; after
  `transition-summary-changed`, `ess-diff/22`, compatible); a moved state summary gives
  `variant-summary-changed` (`tests/enum_variant_wire.rs`).
- `crates/generate/ess-entity-runtime/tests/state_wire_name.rs`:
  `a_summary_only_state_is_not_refused_as_a_wire_spelling`.
- `crates/edge/ess-cli/tests/validate_parse_refusals.rs`: `a_flat_summary_on_a_type_names_naming`;
  end to end `the_lifecycle_summary_reproduction_validates_from_ess_33` (valid, exit 0, under
  `ess/33`; refused naming `ess/33` under `ess/32`).
