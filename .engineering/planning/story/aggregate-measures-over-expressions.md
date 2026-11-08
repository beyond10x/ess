---
format: aep.planning-md/3
id: story:aggregate-measures-over-expressions
kind: story
status: implemented
title: An aggregate measure takes an expression, and min/max admit an ordered enum
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#561
- provider: github
  reference: beyond10x/ess#566
relations:
- serves: vision:O2
- decomposes: epic:generic-aggregates
- depends_on: story:expression-arithmetic-and-time
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/generate/ess-synth/src/go/behaviour/query.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/behaviour/query.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/view_query.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/types.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/aggregate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/views.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/change.rs
- confidence: inferred
  path: schemas/generated/ess.schema.json
revision: 20
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T22:08:01Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-21T22:08:01Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-08T22:10:14Z", actor: "human:timo", revision: 20, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

An aggregate measure's operand is an expression of the source row (a difference of two fields, a
product with a navigated attribute), and `min`/`max` admit every ordered type, including an enum
whose declaration states an order.

Binding design: `docs/design/generic-language-features.md` (being drafted), feature 3, measures.
Parent: `epic:generic-aggregates`.

## Fit review

Findings covered: https://github.com/beyond10x/ess/issues/561 and
https://github.com/beyond10x/ess/issues/566 (a downstream incident-ledger specification, ess 0.59.0,
`ess/24`).

1. **Need.** MTTR (`avg` of `ended_at - started_at`), total downtime, weighted downtime (`sum` of a
   duration times an attribute of an enum), and an incident's headline level as the worst level of
   its windows. Refusals: `ESS-VIEW-001 "… has no field ended_at - started_at …"`
   (`crates/specify/ess-domain/src/view.rs:1339`), `ESS-VIEW-009 "… a path into a field; an aggregate
   reads one top-level field"` (`view.rs:1322`), `ESS-VIEW-002 "… max does not admit level of type
   ledger.ImpactLevel"` (`view.rs:1379`). Requester's syntax (theirs): `aggregate: {avg: ended_at -
   started_at}`, `{sum: duration_s * level.weight}`, `order: [...]` on the enum.
2. **Class.** Gap.
3. **Already expressible?** Only by storing the computed value at write time (copy a difference into
   a field, store a rank integer beside the enum), which needs arithmetic in `sets:` that the
   language refuses too (https://github.com/beyond10x/ess/issues/560), and duplicates the enum's
   meaning.
4. **Fit.** The measure's operand is an expression of the expression language; ordered types come
   from one ordering rule (an enum's declared order is the same order guards compare with). Weight
   on the enum is an open question in the reporter's model; the binding design decides whether an
   enum carries attributes or the weight comes from a navigated row. Targets: validation,
   interpreter, synthesis (rows on either side of the extreme), Rust and Go generated view queries,
   diff.
5. **Second adopter.** Average fulfilment time per warehouse; the highest severity per ticket queue.
6. **Cost.** A source format version; an enum `order:` key (or the binding design's equivalent);
   diff classes for a changed order.
7. **Alternatives.** (a) Change nothing. (b) Per-measure keys. (c) Expression operands and one
   ordering rule (chosen).

## Decisions

Accept, redesigned into the generic aggregate feature.

## Acceptance

- `avg_over_a_field_difference`: MTTR per cause, witnessed with two rows of different durations.
- `sum_over_a_product_with_an_attribute`: weighted downtime witnessed with two levels.
- `max_over_an_ordered_enum`: the worst level per incident, witnessed where declaration order and
  alphabetical order disagree.

## State

2026-11-02 (store hygiene H0 of wave W1, reconcile roadmap section 2d; decision D9 option A, kept active with a dated note): Not started (ESS-VIEW-009); core of beyond10x/ess#561; scheduled W9.

## Scope

Derived 2026-11-07 by `story-scoper`. **cited** lines were read from the story or the tree; **inferred** lines are readings that could be wrong.

- **Not delivered**: `Aggregate.input` is still `Option<String>` (`ess-domain/src/view.rs`), ESS-VIEW-009 still refuses a path, and enums have no `ordered` flag. Cited.
- **Primary surface:** `crates/specify/ess-domain/src/view.rs` (`Aggregate`, `RawAggregate`, `validate_aggregate`, `result_type`): cited.
- **Also:** `crates/specify/ess-domain/src/types.rs` (enum `ordered: true`) and `view/measure.rs`: inferred.
- **Also:** `crates/specify/ess-compiler/src/ir.rs` (`ResolvedAggregate`) and `resolve.rs`: cited by design G3.
- **Also:** `crates/verify/ess-conformance/src/aggregate.rs` (`evaluate`), `synthesize/aggregate.rs`, `synthesize/aggregate/contrast.rs`, `interpret/views.rs` and `mutate.rs`: cited by design G3, `mutate.rs` inferred.
- **Also:** `crates/generate/ess-synth/src/view_query.rs`, `rust/behaviour/query.rs` and `go/behaviour/query.rs`: inferred.
- **Also:** `crates/verify/ess-diff/src/change.rs`, `crates/specify/ess-domain/src/system.rs` (the next source format after `ess/35`) and `schemas/generated/ess.schema.json`: inferred.
- **Documents:** `docs/design/generic-language-features.md` (G3), `website/docs/guides/specify/aggregate-views.md`, `website/docs/reference/spec-versions.md` and `CHANGELOG.md`: inferred.
- **Confidence:** high for `view.rs`, medium downstream.
- **Would collide with:** the other two aggregate stories, which share the whole pipeline and run after this one, and story:navigation-reaches-any-depth-in-view-filters-and-guards (`view.rs`).
- **Safety:** a field-input measure must keep its IR bytes and digests unchanged below the new format.
