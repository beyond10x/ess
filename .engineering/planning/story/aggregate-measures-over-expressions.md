---
format: aep.planning-md/3
id: story:aggregate-measures-over-expressions
kind: story
status: active
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
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T22:08:01Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-21T22:08:01Z", actor: "human:timo", revision: 3}
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
