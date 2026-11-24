---
format: aep.planning-md/3
id: story:synthesis-witnesses-a-comparison-of-two-keyed-lists-so-a-target-that-matches-elements-by-position-drops-the
kind: story
status: draft
title: Synthesis witnesses a comparison of two keyed lists, so a target that matches elements by position, drops the strict half or swaps a quantifier fails
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-guard-compares-two-keyed-lists-element-by-element-and-the-reference-shows-a-dominance-refusal-that-the
- depends_on: story:a-nested-collection-quantifier-is-witnessed-apart
revision: 1
---
## Need

`g14-keyed-list-compare` documents and pins the keyed-list comparison on the interpreted target.
That target decides it, but the suite does not yet check it. A row-set guard whose selector holds a
nested quantifier over the candidate's list and the input's list has to be witnessed by arranged
rows. Each row's list has to make one half of the comparison hold and the other fail, with the
deciding element neither first nor last. Nested collection quantifiers are not yet witnessed
apart:

- `story:a-nested-collection-quantifier-is-witnessed-apart` (draft, backlog) names three shapes
  whose inner mutants survive. One of them is a nested quantifier in a `when_related` predicate
  over a stored list, and its test is ignored
  (`crates/verify/ess-conformance/tests/input_list_quantifiers.rs:1206-1213`,
  `#[ignore = "backlog: nested-quantifier-apart …"]`).
- A stored list is spread element by element, or the search falls back to a plain row
  (`story:collection-quantifiers-witnessed-everywhere`, implemented; `subject_fact::arranged_row`).
  Nothing spreads a candidate row's list against an input list under a key tie.

Without this story, a target that compares the two lists by position, reads `>=` for `>` in the
strict half, or evaluates `forall` for `exists` may pass every scenario.

## Outcome

The suite synthesized for the `g14-keyed-list-compare` fixture holds scenarios that tell the
comparison apart from its five usual mistakes. A shape synthesis cannot spread writes a `note:`
that names its scenario rather than passing silently.

## Acceptance

- Red first: `crates/verify/ess-conformance/tests/keyed_list_witness.rs` (or a module of the
  binary `g14-keyed-list-compare` added) synthesizes the fixture
  `crates/verify/ess-conformance/tests/fixtures/keyed-list-compare.yaml`. Its suite holds
  `demo.offers.Submit/outcome/beaten` and `demo.offers.Submit/outcome/submitted` and passes on the
  interpreted target. Each of five mutated targets fails at least one scenario:
  1. elements matched by position instead of by `criterion_id` (the arranged lists hold the same
     keys in different orders);
  2. `>=` for `>` in the strict half (a row equal on every criterion is arranged);
  3. the outer `forall` evaluated as `exists` (a row better on two criteria, worse on the middle
     one);
  4. the inner `exists` evaluated as `forall`;
  5. the sense ignored (a `lower_better` criterion whose better value is the numerically smaller
     one).
  Every arranged list has at least three elements, and its deciding element is neither first nor
  last. Today at least one mutant survives, or a scenario is missing; the report names which.
- Where a list cannot be spread (an element type with one value, say), `ess verify conform
  synthesize` prints a `note:` that names the scenario and the guard, using the existing `Note`
  vocabulary.
- The third shape of `story:a-nested-collection-quantifier-is-witnessed-apart` (a nested
  quantifier in a `when_related` predicate over a stored list) loses its `#[ignore]` here if this
  story's change makes it pass. Otherwise it stays with that story.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes. A re-pinned
  row exists only for a model with a nested list quantifier in a selector, and names "keyed-list
  comparison witnessed" beside it.
- The test step stays inside the budget: no new test runs over 60 s, and the report names the
  time of every new test over 5 s.

## Fit review

Tree `bcfc51555`. Report item 14, second half. See `g14-keyed-list-compare` § Fit review for
questions 1-5 and 7.

6. **Cost.** No format, keyword, diagnostic or IR change. Synthesis writes more scenarios for
   models with this shape, so their suite bytes move. No suite vocabulary moves: the scenarios
   use existing steps. Size L.

## Decisions

**Accept, redesigned** (`P2B-D1` (a)). Open question:

- **D1. Fold it into the existing backlog story or keep it apart?** (a) Keep it apart and land it
  after `story:a-nested-collection-quantifier-is-witnessed-apart`, reusing that story's inner
  spreading. (b) Absorb that story's third shape here and leave it the two input-list shapes.
  **Recommended: (a)** when that story is scheduled in the same wave, **(b)** otherwise.

## Scope

- inferred: `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (`arranged_row`,
  `spread`)
- inferred: `crates/verify/ess-conformance/src/witness.rs` (nested rebinding at `:3271`, list
  building)
- inferred: `crates/verify/ess-conformance/src/synthesize/row_set/` (candidate rows of a selector)
- inferred: `crates/verify/ess-conformance/tests/keyed_list_witness.rs` (new)
- cited: `crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs` pins
- Size: L (about 550 changed lines).

## Depends on

- `g14-keyed-list-compare`
- `story:a-nested-collection-quantifier-is-witnessed-apart` (D1)

## Source

A downstream application, 2026-11-24, report item 14.
