---
format: aep.planning-md/3
id: story:an-authored-text-literal-outside-its-alphabet-is-refused
kind: story
status: implemented
title: ESS refuses an authored text literal that is outside its field's alphabet
relations:
- serves: vision:O2
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T06:46:08Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-11-01T06:46:09Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-11-01T06:46:09Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1}}}
---
## Finding

Measured by adversary pass 1 of wave 2026-10-07b
(`review-result:adversary-wave-20261007b-u1-pass-1`, F1), pre-existing on `main`: ESS checks an
authored literal against a String type's declared prefix but not against its `alphabet`
(`crates/specify/ess-domain/src/command.rs:4499`), so `sets: note: "kept!"` validates against an
alphabet without `!`. The Entity Runtime lowering now refuses such a literal by name; the
specification itself still admits it.

## Acceptance

`ess specify validate` refuses an authored text literal whose characters are not all in the
effective alphabet of the field or argument it is written into, naming the literal, the first
character outside the alphabet and its location, the way beyond10x/ess#146 refused prefixes.

## Wider case

The U1 implementor of wave 2026-10-07b found the same gap one step wider, pre-existing: ESS also
admits a `sets:` literal that breaks its type's `invariants:`, and the lowered Entity Runtime rule
then refuses that branch on every request. The acceptance covers both: a literal that breaks its
type's alphabet or its invariants is refused at validation.

## Downstream report 2026-10-19

Downstream case (2026-10-19, finding 16): entity invariants are not checked against literal
writes. `invariants: [capacity > 0]` on the entity beside `sets: {capacity: 0}` validates (exit 0)
and synthesizes 8 scenarios with 0 refusals on 0.60.0, 0.62.0 and the integration build, while the
same literal against a newtype invariant (`value > 0`) is refused `[type_mismatch] … its invariant
'value > 0' does not hold for it` (`.engineering/repro/invariant-broken-by-literal/entity-invariant.yaml`
and `newtype-invariant.yaml`). So the type half of the wider case above is already enforced; the
entity half is not.

Quick fix, one unit, separable from the alphabet half: in
`crates/specify/ess-domain/src/command.rs`, beside the `InvariantReadsUnsetField` pass (~4550),
evaluate each entity invariant whose every read field the branch writes with a literal (or its
arrival state) with `LiteralFacts`, as the type-literal check does (~5357-5370); then the same for
`affects:` `sets:` in `command/set_effects.rs` (inferred, not probed). Values derived from input stay
out of scope (deciding that a guard implies the invariant is not decidable in general). Test: a
validation test under `crates/specify/ess-domain/tests/` (a literal `sets:` breaking an entity
invariant is refused naming the branch; an input-derived value is accepted); `task test-xtask` if a
new `ValidationCode` is added.

## Triage (D338)

Adopted at the 2026-10-31 findings triage (orchestrator D338, operator approval relayed by host-monitor D53), narrowed to the entity-invariant half: a `sets:` literal that breaks an entity invariant (`capacity > 0` with `sets: {capacity: 0}`) passes `ess specify validate`, and the lowered Entity Runtime rule then refuses that branch on every request. The alphabet half looks held by `layered_literal` since 0.60.0 (inferred, `crates/specify/ess-domain/src/command.rs`, beyond10x/ess#533).

## Already held (D343)

Checked 2026-11-01 for wave D343 on 45a1beba7 (story-scoper reading, coordinator run).

- **Entity-invariant half (the adopted narrowing): already held.** `validate_literal_invariants`
  (`crates/specify/ess-domain/src/command.rs:5300-5402`, called at `:5136`; `affects:` rows through
  `literal_rows` `:5414-5460` and `command/set_effects.rs:465`) refuses a `sets:` literal that breaks
  an entity invariant as `conflicting_declaration` (`:5376`). It landed in f9fa4d12d ("three
  downstream findings of 2026-10-19"), first tagged 0.63.0, with
  `crates/specify/ess-domain/tests/literal_entity_invariants.rs`. Run: `ess specify validate --path
  .engineering/repro/invariant-broken-by-literal/entity-invariant.yaml` (ess 0.77.0) refuses
  `outcome drained of pool.slots.DrainPool writes 0 into capacity, which breaks the invariant
  capacity > 0 of pool.slots.Pool` (ESS-COMMAND-004, `entity-invariant.yaml:46:5`).
- **Alphabet half: held** by `layered_literal` (`command.rs:6766`, alphabet check in
  `example_layers` `:6842-6854`, reached from `sets_literal` `:5182`), pinned by
  `crates/specify/ess-domain/tests/view_paging/adversary_m5_pass1.rs:138`
  (`the_same_out_of_alphabet_literal_in_sets_is_refused`) — code read, not run.

So the story leaves the D343 selection with no unit and is closed as implemented by f9fa4d12d.
