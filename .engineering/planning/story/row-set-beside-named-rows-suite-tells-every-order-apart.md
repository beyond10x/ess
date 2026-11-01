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
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_ess0700a_identity_beside_row_set_2.rs
revision: 5
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

## Scope

Derived 2026-11-01 by `aep:story-scoper` for wave D350 on dc60f4532. Every line is **cited** (read from the story or the tree) or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize/related_guard.rs` and `synthesize/row_set.rs` — cited (the story and its review-result name both sites)
- `related_guard.rs` `steering_for` :665-710 (:688 is the early `return Vec::new()` on an unsettled row); `row_set.rs` `prepare_with` :3376-3679, search closure :3540, `consistent` :3581, eager `later_sets_hold` :3585 — cited. Linking :688 or :3581 to the four reorderings or the forall refusal — inferred
- **Symbols:** `steering_for`, `read` :712, `rows` :725, `several` :748, `several_present` :535, `present_beside` :640, `beside_row` :1121, `later_sets_hold` :3245, `arrange_rows` :1690 (forall negation :1792-1796) — cited
- **Also likely:** `synthesize/precedence.rs` — inferred
- **Tests pinning today:** `tests/adversary_ess0700a_identity_beside_row_set_2.rs` (:572, :616, :652, :682, :728, forall control :782) — cited. None of the four story-named tests exists; commit d740b1f9e deferred them — cited
- **Confidence:** medium (sites and absent tests read; mechanism not run)
- **Would collide with:** `related_guard.rs` (`steering_for`, `read`, `rows`, `beside_row`), `row_set.rs` (`prepare_with`, `arrange_rows`, forall selection :2523-2700), `precedence.rs`, the test file above
