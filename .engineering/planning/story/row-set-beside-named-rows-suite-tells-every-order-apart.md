---
format: aep.planning-md/3
id: story:row-set-beside-named-rows-suite-tells-every-order-apart
kind: story
status: draft
title: 'A row set beside named rows: the suite tells every refusal order apart'
summary: 'With when: beside a named refusal or a forall accepting branch, the synthesized suite misses reorderings or leaves a branch unwitnessed.'
tags:
- ess-0.71.0
relations:
- derived_from: review-result:adversary-ess-0700-unit-a-pass-2
revision: 1
---
## Finding

Found 2026-10-26 by adversary pass 2 on the 0.70.0 wave's unit A
(`review-result:adversary-ess-0700-unit-a-pass-2`). From `ess/32` a command may hold an
identity-addressed `when_related:` beside a row set. For three shapes, the synthesized suite does
not tell apart every reordering of the step-4 refusals, or it leaves a branch unwitnessed:

- a `when:` beside a named-row refusal: synthesis refuses the row-set branch, and 4 reorderings
  pass (`related_guard.rs` about :688);
- an accepting branch guarded by `forall` beside named rows: the default refusal is refused by
  synthesis ("no arrangement left a related row"), and a target that refuses on any selected row
  passes the whole suite (`row_set.rs` about :3580);
- the same `forall` mutant passes at `ess/31` without named rows; the origin is undecided because
  the base was not run.

## Acceptance

- For each shape, the suite fails every target that answers some reachable request differently
  from the one precedence rule, and passes the interpreter. Pass 2's method: wrap the interpreter,
  answer with the first refusal that holds in a given order, and run every order.
- The pass-2 cases `adversary_a2_when_beside_a_named_refusal_orders_are_told_apart`,
  `adversary_a2_forall_beside_named_rows_kills_forall_mutants`,
  `adversary_a2_forall_beside_named_rows_witnesses_every_branch` and
  `adversary_a2_forall_without_named_rows_kills_forall_mutants_at_ess_31` are restored and green.
