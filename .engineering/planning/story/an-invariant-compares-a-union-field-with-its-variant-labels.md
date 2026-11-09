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
scope:
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/invariant.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/invariant.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/entity.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/types.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/union_variant_guards.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/facts.rs
revision: 3
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

## Acceptance

- At the format the parent story opens, the reproduction's invariant validates, and so do
  `migrated_by != Agent` and `migrated_by in [Human]` on an entity and in a struct's invariant over a
  union member. A label that is no variant is `type_mismatch` naming the variants, as in a guard.
- The interpreted target evaluates the invariant over the stored variant label. Synthesis's
  invariant observations (`<entity>/invariant/after/<command>/<outcome>`) read the label, and a
  target that stores an `Agent` actor through the command fails the scenario. Where the command
  declares no refusal for an `Agent` input, synthesis reports the hole as
  `story:synthesis-offers-an-input-that-breaks-an-entity-invariant` does for every invariant.
- Entity Runtime lowers the comparison with the guard form of the parent story, or refuses it by
  name at lowering. Generated Rust and Go invariant checks evaluate it, or refuse the type by name.
- Below that format the invariant keeps its refusal, its hint naming the format.

## Scope

- inferred: crates/specify/ess-domain/src/entity.rs
- inferred: crates/specify/ess-domain/src/types.rs
- inferred: crates/verify/ess-conformance/src/interpret/facts.rs
- inferred: crates/generate/ess-synth/src/rust/invariant.rs
- inferred: crates/generate/ess-synth/src/go/invariant.rs
- inferred: crates/generate/ess-entity-runtime/src/subset.rs
- inferred: crates/specify/ess-domain/tests/union_variant_guards.rs
confidence: low

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
6. **Cost.** No format of its own: it lands in the parent's format. No new key or diagnostic code.
7. **Alternatives.** (a) Leave invariants out: one cell of the opened column stays closed, which
   the closing rule refuses. (b) Include them: chosen.

## Decisions

accept as proposed. The requester's `actor == Agent`-style comparison is the parent story's chosen
spelling, extended to invariants as the closing rule requires.

## Source

github:beyond10x/ess#764
