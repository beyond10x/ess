---
format: aep.planning-md/3
id: story:entity-runtime-lowers-alphabet-and-text-count
kind: story
status: draft
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
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/lowerable_subset.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/lowering.rs
- confidence: inferred
  path: website/docs/reference/entity-runtime-lowering.md
revision: 8
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
