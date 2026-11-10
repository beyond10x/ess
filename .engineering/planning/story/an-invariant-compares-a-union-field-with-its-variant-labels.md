---
format: aep.planning-md/3
id: story:an-invariant-compares-a-union-field-with-its-variant-labels
kind: story
status: draft
title: An invariant compares a union field with its variant labels
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#764
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key
- depends_on: story:synthesis-offers-an-input-that-breaks-an-entity-invariant
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
- depends_on: story:synthesis-and-the-interpreter-decide-a-guard-on-a-union-variant
- depends_on: story:generated-targets-decide-a-guard-on-a-union-variant
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/union_label_invariant.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/invariant.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/invariant.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/invariant_check.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/entity.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/types.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/union_variant_invariants.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/facts.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/union_variant_invariants.rs
revision: 7
---
## Need

Once a guard may compare a union value with its variant labels
(`story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key`, which admits `when:`,
`when_subject:`, a `when_related:` predicate, a row-set `where:` and a view `filter:`), an entity or
type invariant must be able to say the same of a stored union field: "a store is migrated only by a
person" is `migrated_by == Human`. That story's acceptance does not name invariants. Grid R's
invariant row reads the entity's own fields (`docs/design/language-grids.md:28`), and the refusal
is the type's, not the position's, so the union column opened there must reach this row too.

Minimal reproduction (0.92.0, `ess/36`): union `example.store.Actor` (tag `kind`, variants `Human`,
`Agent`), entity `example.store.Store` with `migrated_by: example.store.Actor` and the invariant
`migrated_by == Human` (`repro/run.txt`, `invariant`):

```text
error[ESS-ENTITY-002]: `migrated_by == Human`: operator `==` does not admit `example.store.Actor` (aggregate) and `Text literal `Human`` (Text)
```

For the synthesis and target bullets below, the reproduction gains a command
`example.store.Migrate` whose branch `migrated` creates the `Store` with
`sets: {migrated_by: input.by}` and holds no guard on `by`.

## Acceptance

- At `ess/37`, `ess specify validate` exits 0 on the reproduction's invariant, on
  `migrated_by != Agent` and on `migrated_by in [Human]`, each on the entity and in a struct's
  invariant over a union member (`cargo test -p ess-domain --locked --test union_variant_invariants`).
- In the same test, the invariant `migrated_by == Robot` is refused `type_mismatch` with a message
  naming the variants `Human` and `Agent`, as in a guard.
- At `ess/36`, the reproduction's invariant is still refused `ESS-ENTITY-002`, and the diagnostic's
  help names `ess/37` (same test).
- `ess verify conform synthesize` on the reproduction with `Migrate` reports one refusal with the
  code `story:synthesis-offers-an-input-that-breaks-an-entity-invariant` adds, at
  `example.store.Store/invariant/after/example.store.Migrate/migrated`, naming the invariant
  `migrated_by == Human` and the input `by` sent with variant `Agent`
  (`cargo test -p ess-conformance --locked --test union_variant_invariants`).
- With a refusal branch `when: by != Human` added before `migrated`, the same command reports
  `0 refusal(s)`, and `ess verify conform run --target interpreted` passes every scenario; the same
  suite with the `migrated` step edited to send `by` as `Agent` fails
  `example.store.Store/invariant/after/example.store.Migrate/migrated` with `ESS-CF-TARGET` naming
  the invariant (same test).
- The generated Rust and Go invariant checks evaluate the comparison: a new union fixture in
  `cargo test -p ess-synth --locked --test invariant_check` has `broken_invariant` name
  `migrated_by == Human` for a `Store` value holding `Agent`, and name nothing for one holding
  `Human`, in the Rust harness and in the Go harness.
- Entity Runtime lowering of the reproduction refuses the invariant by name: its diagnostics hold
  `LoweringCode::UnionVariantGuardUnsupported` naming `example.store.Store`'s invariant, under a
  row of `crates/generate/ess-entity-runtime/src/subset.rs` for an invariant
  (`cargo test -p ess-entity-runtime --locked --test union_label_invariant`).

## Scope

- inferred: crates/specify/ess-domain/src/entity.rs
- inferred: crates/specify/ess-domain/src/types.rs
- inferred: crates/specify/ess-domain/tests/union_variant_invariants.rs (new)
- inferred: crates/verify/ess-conformance/src/interpret/facts.rs
- inferred: crates/verify/ess-conformance/tests/union_variant_invariants.rs (new)
- cited: crates/generate/ess-synth/src/rust/invariant.rs (`preflight` refuses an invariant it
  cannot spell; the comparison is spelled here)
- cited: crates/generate/ess-synth/src/go/invariant.rs (the same, for Go)
- inferred: crates/generate/ess-synth/tests/invariant_check.rs (the Rust and Go harnesses)
- cited: crates/generate/ess-entity-runtime/src/subset.rs (the refused-construct rows beside
  `TEXT_COUNT_UNTYPED`, :430)
- inferred: crates/generate/ess-entity-runtime/tests/union_label_invariant.rs (new)

confidence: low

Shared surfaces: proposes one `CHANGELOG.md` sentence and its `ess/37` row text; regenerates
nothing of its own.

## Fit review

1. **Need.** An invariant over which variant a stored union holds. Requester's ask (theirs): "the
   same in `when_related` and invariants".
2. **Class.** Gap: the same refusal as in a guard (`run.txt`, `invariant`).
3. **Already expressible?** Only with the enum-beside-union workaround, which the parent story
   refuses as an idiom under policy 1.
4. **Fit.** The parent opens grid O's union column for comparison and membership "in every guard
   position". The closing rule (`docs/design/generic-language-features.md:211-216`) opens a column
   whole, so the invariant row, which reads own fields, gets the same cells and is not left as one
   unopened cell.
5. **Second adopter.** A payment entity whose invariant says a refund is only ever `method ==
   card`; a shipment whose `carrier` union is never `Pickup` once `state == Shipped`.
6. **Cost.** No format of its own: it lands in `ess/37`, the parent's format. No new key or
   diagnostic code.
7. **Alternatives.** (a) Leave invariants out: one cell of the opened column stays closed, which
   the closing rule refuses. (b) Include them: chosen.

## Decisions

accept as proposed. The requester's `actor == Agent`-style comparison is the parent story's chosen
spelling, extended to invariants as the closing rule requires.

- Generated Rust and Go evaluate the comparison rather than refuse it, because their invariant
  evaluators already compare a stored member with a literal and the union's tag is a stored member.
- Entity Runtime refuses the invariant by name rather than lowering it, because
  `story:generated-targets-decide-a-guard-on-a-union-variant` refuses the same comparison in a
  guard (`LoweringCode::UnionVariantGuardUnsupported`) and an invariant should not lower what a
  guard cannot.
- The tests go in new `union_variant_invariants.rs` targets, not in the parent's
  `union_variant_guards.rs`, so the two stories do not edit one test file.

## Source

github:beyond10x/ess#764
