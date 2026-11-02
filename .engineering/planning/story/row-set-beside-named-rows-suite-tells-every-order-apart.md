---
format: aep.planning-md/3
id: story:row-set-beside-named-rows-suite-tells-every-order-apart
kind: story
status: implemented
title: 'A row set beside named rows: the suite tells every refusal order apart'
summary: 'With when: beside a named refusal or a forall accepting branch, the synthesized suite misses reorderings or leaves a branch unwitnessed.'
tags:
- ess-0.71.0
relations:
- derived_from: review-result:adversary-ess-0700-unit-a-pass-2
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/precedence.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_ess0700a_identity_beside_row_set_2.rs
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-02T07:12:07Z", actor: "human:timo", revision: 9}
- {from: "proposed", to: "active", at: "2026-11-02T07:12:07Z", actor: "human:timo", revision: 10}
- {from: "active", to: "implemented", at: "2026-11-02T14:24:24Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"test_result":3,"review_outcome":2}}}
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

Derived 2026-11-02 by `aep:story-scoper` for wave D368 on 6437975cb, replacing the D350 scope of 2026-11-01. Every line is **cited** (read from the story, the tree, or a probe run of `ess` 0.82.0 on models outside the repository) or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize/row_set.rs` (unchanged since dc60f4532) and `synthesize/related_guard.rs` (changed by D357, new `related_guard/rewritten.rs`) — cited
- **Still reproduces at 0.82.0 (when:):** a `when: input.from_port != input.to_port` on `sink-not-input` in `identity-beside-row-set.yaml` gives 6 scenarios and 1 refusal (`Connect/source-taken`, `ROWS_GUARDED_CREATOR`); the unmodified fixture gives 7 and no refusal — cited (run)
- **Still reproduces at 0.82.0 (forall):** a `forall: to_port != input.to_port` on `connected` beside the four named refusals refuses `Connect/duplicate`; the `connected` scenario arranges no earlier wire from the source, so a target refusing on any wire from the source passes — cited (run)
- **Finding 6 (ess/31 without named rows):** undecided — inferred
- **Refusal path:** `prepare_with` :3376-3679 (search closure :3540, `consistent` :3581, eager `later_sets_hold` :3585) returns `ROWS_UNARRANGED`; `guarded_creator` :3277 renames it `ROWS_GUARDED_CREATOR` (:248, :3362); help text `synthesize.rs:1386` — cited
- **`when:` site:** `named_refusal` (`related_guard.rs:764`, `input: None` at :769/:774) is false for a refusal carrying `when:`, so `beside_named` (`row_set.rs:183`, :195) is false; `steering_for` :666-711 filters `input: None` at :700 (early return :690) — sites cited, cause inferred
- **forall site:** `arrange_rows` :1690 (forall negation :1792-1796), the selection plan's foralls :2523-2700, `rows_tried` :4055, `steered_inputs` :4151, `left_refusing` :4190 — cited; fix site inferred
- **Other symbols:** `several_present` :536, `present_beside` :641, `read` :713, `rows` :726, `several` :749, `named_rows` :1221, `beside_row` :1122 (`related_guard.rs`) — cited. Also likely `synthesize/precedence.rs` — inferred
- **The four story-named cases exist nowhere:** `git log --all -S` on each name finds only d740b1f9e (the story file); a unit writes them — cited. Scaffolding on main in `tests/adversary_ess0700a_identity_beside_row_set_2.rs`: `Shape::sink_apart` :229/:262, `REV`/`DUP` :59-60, `FORALL` :772, `forall_without_named_rows_at_31` :800, harness `every_order_told_apart` :491 — cited
- **Tests pinning today:** that file's order tests :572, :616, :652, :682, :728, the ess/31 control :782, determinism pin `33908` :833/:860; `adversary_ess0700a_identity_beside_row_set.rs`, `row_sets*.rs` — cited
- **Pinned bytes that could move:** `synthesis-suite-bytes-table.tsv` rows for forall fixtures (`filtered-related-reads.yaml` :204, `latest-correlated-record.yaml` :212, `precedence-row-set-updating-branch.yaml` :44, `fold.yaml` :14) and `synthesis-component-suites-table.tsv`, only if the forall fix changes accepting scenarios outside the named-rows path — inferred. `suites/generated/*`: no forall, not expected — cited
- **Size:** medium (~350 changed lines) — inferred. **Confidence:** medium (defects reproduced, fix site not proven)
- **Would collide with:** `row_set.rs` (`prepare_with`, `guarded_creator`, `arrange_rows`, selection plan, `beside_named`), `related_guard.rs` (`named_refusal`, `steering_for`, `named_rows`, `beside_row`), `precedence.rs`, the pass-2 test file, the synthesis bytes tables
- **Safety fact:** `named_refusal` is called only from `row_set.rs:195`, so admitting a `when:` refusal changes only the `beside_named` path — level 2, unproven

## Delivered

D368, 2026-11-02: a command guarding on a row set beside rows its input names (`ess/32`) is witnessed on every refusal order. Unit 3e665f021, merged 7e0d2e880, with a 12-line `synthesize.rs` change approved by the wave lead after U1 merged. Fix round 1 (7510ae01f, merged 21ef8a7fa) answered both review passes: a named-row refusal with a `when:` now gets a send that holds it, the accepting `forall` fallback tries `SEND_ROWS`, the creator may be sent the linked row twice, and the `named_refusal` doc is corrected. No suite bytes moved. Out-of-scope rows became `story:a-creator-under-a-row-set-and-when-is-arranged`, `story:a-row-set-suite-fails-a-target-refusing-less` and `story:a-not-equal-or-any-selector-beside-named-rows-is-witnessed`.
