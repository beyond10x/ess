---
format: aep.planning-md/3
id: story:entity-runtime-lowers-alphabet-and-text-count
kind: story
status: implemented
title: Entity Runtime lowering covers a String alphabet and a text count (entity-core 0.27.0)
refs:
- provider: github
  reference: beyond10x/entity-runtime#54
relations:
- serves: vision:O2
- decomposes: story:entity-runtime-lowers-entity-core-constructs
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/adversary_guards_lowering.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/lowerable_subset.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/lowering.rs
- confidence: cited
  path: docs/design/ess-evolution/entity-runtime-lowering.md
- confidence: cited
  path: docs/design/string-alphabet-and-length.md
- confidence: inferred
  path: website/docs/reference/entity-runtime-lowering.md
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-10-07T01:43:16Z", actor: "human:timo", revision: 9}
- {from: "proposed", to: "active", at: "2026-10-07T01:43:16Z", actor: "human:timo", revision: 10}
- {from: "active", to: "implemented", at: "2026-10-08T09:55:11Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"test_result":1,"review_outcome":3,"verification":1}}}
---
## Outcome

`ess-entity-runtime` lowers two constructs it refuses today, against entity-core 0.27.0:

- a String type's declared `alphabet` becomes the `alphabet:` key on the lowered `service/1`
  string field, argument or nested property;
- `<text>.count` in a guard becomes entity-core's `<path>.count` on a declared `string`, counted in
  Unicode scalar values.

Both leave the generated lowerable-subset table, and `LoweringCode::AlphabetUnsupported` and
`LoweringCode::TextLengthUnsupported` are no longer raised for them.

## What entity-core 0.27.0 does not take (still refused by name)

From https://github.com/beyond10x/entity-runtime/issues/54 (comment of 2026-10-06):

- a text length on a quantifier element or as a projection key;
- an alphabet on a declared response field: accepted by entity-core but not enforced, so ESS keeps
  refusing it rather than lowering a check that does not run.

## Acceptance

- `tests/lowering.rs::an_alphabet_and_a_text_length_are_refused_by_name_by_the_lowering` is
  re-written: the alphabet newtype and `note.count > 3` lower, and the lowered definition carries
  `alphabet` and `count`. It fails on `main` today.
- The two cases above that 0.27.0 does not take still refuse with their codes.
- The `ALPHABET` and `TEXT_COUNT` entries in `crates/generate/ess-entity-runtime/src/subset.rs`
  move from refused to lowered, and the generated subset table is regenerated.
- The `entity-core` pin in `Cargo.toml` and `Cargo.lock` moves from `0.24.1` to the release that
  carries both features.
- An Entity Runtime conformance run executes one scenario per construct.

## Spec first

Spec first: model the change in this repository's ESS specification, validate it with the newest
`ess`, regenerate, then implement against the generated code. If the specification cannot express
it, stop and report that; do not hand-write a parallel model.

## Scope corrections (2026-10-07)

From the implementor's phase-1 report:

- `tests/adversary_guards_lowering.rs` is in scope: its case
  `adv_an_input_text_length_in_a_stored_field_predicate_is_refused_by_name` becomes a lowering case.
- Both inferred lines held: `tests/lowerable_subset.rs` carries the subset and page checks, and
  `website/docs/reference/entity-runtime-lowering.md` is generated from `src/subset.rs`
  (`ESS_LOWERING_REFERENCE=write cargo test -p ess-entity-runtime --test lowerable_subset`).
- `docs/design/string-alphabet-and-length.md` (§1, §6, §8) and
  `docs/design/ess-evolution/entity-runtime-lowering.md:75` describe the refusals and the old
  revision; they are updated with the change.
- No Entity Runtime conformance lane exists in this repository. The acceptance line that asked for
  one is met by runtime decisions inside the lowering tests: the lowered definition is registered
  with entity-core and a value outside the alphabet, and a text over the count, is refused by
  entity-core itself.
- Also still refused (added in phase 1): a text length read through a union payload, which
  entity-core's runtime does not type (0.28.0 `runtime.rs` `walk`, per the implementor).
