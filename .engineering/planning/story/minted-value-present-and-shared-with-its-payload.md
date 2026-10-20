---
format: aep.planning-md/3
id: story:minted-value-present-and-shared-with-its-payload
kind: story
status: implemented
title: A minted value is stored present and read from its own payload
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-19T21:15:04Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-19T21:15:04Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-19T21:15:05Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

An outcome can mint a value into an `Optional` stored field as present (`{generated: true,
present: true}`), and a `sets:` entry can read the value this outcome's own payload minted
(`{payload: <Event>.<field>}`), so one mint feeds both the event and the row.

Closes https://github.com/beyond10x/ess/issues/545.

## Fit review

1. **Need.** A sequence minted when an entry is selected is published and stored; an invariant
   requires it present from a later state on. Today `{generated: true}` into an `Optional` field may
   produce absence (the interpreted target does), so the model's own next transition breaks its
   invariant (17 synthesized scenarios of one adopter moved to error); and payload and `sets:` are
   two independent mints, so the model cannot say the stored value is the published one.
2. **Class: gap.**
3. **Already expressible?** Only by making the minted value a command input, which makes the
   scenario author choose a value the implementation chooses (#544's problem).
4. **Fit.** Two small source forms beside the existing `{generated: true}`: a `present` flag on the
   generated source for an `Optional` target, and a `payload` source admitted in `sets:` that names
   an event and field of the same outcome's payload whose source is `{generated: true}` (or any
   source; reading a mint is the need). Generated Rust and Go call the required generator for a
   present mint; the interpreter and synthesis treat it as present and use one minted value for
   both places.

## Decisions

- Keys: `{generated: true, present: true}` (only on an `Optional` target; refused on a required
  one, which is already present) and `sets: {<field>: {payload: <Event>.<field>}}` (the event must
  be in this outcome's `payload:`; the field types must agree; refused otherwise).
- Source format `ess/25` (open), with an old-reader test.
- Reversible: both are new optional forms; nothing existing changes meaning.

## Acceptance

Each line names the failing test written first.

- `crates/specify/ess-domain/tests/minted_value_sources.rs`
  `present_generated_into_an_optional_field_validates_at_ess_25`,
  `present_on_a_required_field_is_refused`, `a_payload_source_naming_another_outcomes_event_is_refused`,
  `a_payload_source_with_a_mismatched_type_is_refused`.
- `crates/verify/ess-conformance/tests/minted_value_sources.rs`
  `the_interpreter_keeps_a_present_mint_present_through_the_invariant` (the issue's
  `not (state == Bridged) or defined(connect_seq)` model passes every synthesized scenario) and
  `a_stored_value_read_from_the_payload_equals_the_published_one` (a target storing a different
  value fails a scenario).
- `crates/generate/ess-synth` tests (file inferred) `present_mint_calls_the_required_generator`
  for Rust and Go.

## Scope

- inferred: `crates/specify/ess-domain/src/command/` (sets sources, payload sources),
  `crates/specify/ess-domain/src/wire.rs` (raw keys), `crates/specify/ess-compiler/src/ir.rs`,
  `crates/verify/ess-conformance/src/{synthesize.rs,interpret/execute.rs}`,
  `crates/generate/ess-synth/src/{rust,go}/behaviour.rs`, `schemas/generated/ess.schema.json`
  (`cargo xtask schema`), `docs/design/` payload-sources page.

## Plan

- depends_on: the unit that opens `ess/25`.
- du estimate: 10G.
- adversary: yes (a new source form that feeds generated code and the interpreter).
- shared surfaces: `wire.rs`, `ir.rs`, `interpret/execute.rs` with other ess/25 units (sequence).
