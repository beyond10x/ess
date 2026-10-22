---
format: aep.planning-md/3
id: story:a-guard-compares-fields-of-two-named-rows
kind: story
status: draft
title: A guard predicate compares fields of two rows the input names (deferred)
tags:
- adopter-report
- deferred
refs:
- provider: github
  reference: beyond10x/ess#564
relations:
- decomposes: epic:generic-row-queries
- serves: vision:O2
- informed_by: story:navigation-reaches-any-depth-in-view-filters-and-guards
revision: 1
---
## Status: deferred

Deferred: outside the binding design's cut, not built in the 0.68.0 wave, and not to be moved
until the design admits navigation in guard predicates. The design says so in
`docs/design/generic-language-features.md` lines 247-252: "a second hop in a view or a guard,
`E[id]`, and navigation in guard predicates are not in this cut and stay refused by the paths' own
checks. … a comparison across two rows (`E[input.a].direction == E[input.b].direction`) is one such
branch per value until navigation reaches guard predicates." Wave 4 item 2 (line 623) names `E[id]`
for a later format.

## Outcome

A guard predicate compares fields of two rows the input names, `E[input.a].x == E[input.b].x`, in
one branch.

Finding 11 of https://github.com/beyond10x/ess/issues/564 (downstream report, 2026-10-19): a wire
may join two ports only where their directions differ. On the integration build (e44d2b75c,
sizing pass 2, 2026-10-21) the comparison is refused ESS-COMMAND-001. Reproduction:
`.engineering/repro/two-rows-one-branch/all.yaml`.

## Idiom until then

One `when_related: {all: [{via, predicate}, …]}` branch per value, each row with its own
predicate (design lines 249-252; `ess/28`): for a two-valued direction, one branch refusing
`a.direction == output and b.direction == output` and one refusing `input`/`input`.

## Fit review

1. **Need.** A rule that relates two named rows to each other, not each to a constant.
2. **Class.** Gap, deliberately outside the first cut.
3. **Already expressible?** For a finite enum, yes, by the idiom above (one branch per value);
   not for an unbounded type.
4. **Fit.** `E[id]` navigation in predicates, G2 wave 4 item 2.
5. **Second adopter.** Transfer only between two accounts of one currency.
6. **Cost.** Navigation in guard predicates: interpreter, synthesis (two rows equal and unequal
   on the compared field), generated Rust and Go, Entity Runtime refusal by name.
7. **Alternatives.** (a) The idiom. (b) `E[id]` in predicates (the design's eventual answer).

## Decisions

- Deferred, per design lines 247-252. This draft is not moved; the coordinator moves it when a
  design revision admits navigation in guard predicates.

## Acceptance (when undeferred)

- `crates/specify/ess-domain/tests/row_queries_navigate.rs`:
  `a_guard_predicate_compares_fields_of_two_named_rows` (the reproduction validates at the
  admitting format; below it ESS-COMMAND-001 names that format).
- `crates/verify/ess-conformance/tests/related_guard.rs`:
  `a_two_row_comparison_is_witnessed_equal_and_unequal`.
