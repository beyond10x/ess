---
format: aep.planning-md/3
id: story:validation-reads-selection-plan
kind: story
status: implemented
title: Validation reads the selection plan's phases
relations:
- decomposes: epic:one-selection-plan
- serves: vision:O2
- depends_on: story:selection-plan-design-and-type
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_guard.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/subject_state.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/held_state_order.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-07T13:18:14Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-07T13:18:15Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-10-07T23:30:52Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
# Story: Validation reads the selection plan's phases

## Why

`epic:one-selection-plan`. Validation restates the precedence order three times:
`validate_held_state_order` (`crates/specify/ess-domain/src/command.rs`, #486) classifies
held-state against accepting and external branches, `validate_partition`
(`command/subject_state.rs`) takes input refusals first by hand, and `related_guard.rs` orders
present-related refusals. A plan phase that moves makes all three wrong at once.

## What it delivers

The three checks classify branches through the phase classification
`story:selection-plan-design-and-type` places where `ess-domain` can read it.

## Acceptance

- Every `ess-domain` and `ess-compiler` test passes unchanged, the #486 tests included
  (`tests/held_state_order.rs`).
- None of the three checks matches on `OutcomeCondition` to decide which branch answers first.
- A test inside `story:selection-plan-design-and-type`'s phase-order override, with the held-state
  and accepting/external phases exchanged, shows the #486 refusal follow it: the order it refused
  validates and the order it admitted is refused. The story adds no seam of its own.

## Scope

Derived 2026-10-07 by `aep:story-scoper`. Lines marked #487 are read from PR #487's head
(`63bc242020`, branch `fix/selection-precedence`), because that rule was not on `985f58cc3a`. Every
line is **cited** or **inferred**.

- **Primary surface:** `crates/specify/ess-domain/src/command` validation — cited
- **Files:** `command.rs` the #486 rule (#487 :3279-3335, called at #487 :2759-2761); `command/subject_state.rs:127-133, 291-297, 359-371`; `command/related_guard.rs:865-919, 2100-2120, 2122-2153` — cited
- **Places that decide which branch answers first:**
  - #486 (#487 :3295, :3303, :3325-3329): `reads_held_state() || reads_subject_fact()` is step 4; `When` without `error`, `External`, `ExternalWhen` are step 6; declaration order — cited
  - `subject_state::validate_partition`: `is_input_refusal` (`When` + `error`, no subject, no replays, :127-133) answers first; of two, the lowest index (`.min()`, :362-370); undecidable refusals are dropped from the proof (:291-297) — cited
  - `related_guard`: step 5 only where `orders_present_refusals` holds (:900-904: ess/22, `wrong_state`, every present branch refuses, or a stored `via`), else the command is refused (:905-917); then one selected `Related{Holds}` refusal answers (:2131-2151), or the first declared per row with several rows (:2112) — cited
  - partition membership filters drop branches answered in other phases by matching `OutcomeCondition`: `subject_state.rs:310-321`, `related_guard.rs:1530-1544, 1885-1897` — cited
  - `finite.rs:258-261` returns `selected` in declaration order, which those tie-breaks rely on — cited
- **Not ordering sites:** `subject_fact.rs:553-559` (two selected branches are refused); `outcome_shapes.rs` (at-most-once markers) — cited. `row_set.rs:401-409` orders refusals before accepting by `error.is_some()`, a fourth site the story does not name — cited
- **Tests:** #486 `tests/held_state_order.rs`, `tests/subject_state_adversary.rs` (both #487); input refusal first `tests/refusal_beside_state.rs:104,123,138`, `tests/adversary_refstate_pass1.rs:56,69`; present-related order `tests/related_guard.rs:548,568,588,643,661` (#282), `tests/related_guard_multiple_vias.rs:91,184,200` (#283); stored `via` `tests/related_guard.rs:769` (inferred); `row_set` order: none found — cited
- **Also likely:** the classification module `story:selection-plan-design-and-type` creates (path not chosen) — inferred; `crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json`, only under `CONSUMER_CHECKS=true` (parked, `Taskfile.yml:384-390`) — cited
- **Confidence:** medium. All three sites read; the classification's home is undecided.
- **Would collide with:** any unit touching `CommandSpec::validate` / `OutcomeCondition` impl (`command.rs:659-736`, ~2756-2762), the `subject_state` or `related_guard` partitions, the dependency story's module — inferred

### Design decisions for the implementor (inferred)

1. Start after #487 is on `main`.
2. Classify per `Outcome`, not per `OutcomeCondition`: input refusal reads `subject`, `replays` and `error`; "accepting" reads `error.is_none()`; the related order reads `is_refusal()`. If the dependency story ships a condition-only classifier, report it rather than adding a second match.
3. Step 5 is a property of the whole command: read whether the plan has a present-related refusal phase instead of recomputing `orders_present_refusals`; keep the refusal at :905-917.
4. Keep declaration order inside a phase (`.min()` `subject_state.rs:362-370`, first declared `related_guard.rs:2112`).
5. Acceptance 3 needs an injectable classifier: pass the classification into the #486 check, test the swap with a `#[cfg(test)]` unit in `ess-domain`.
6. Derive the "accepting"/"external" word from the phase with the exact current text; `held_state_order.rs` asserts the message byte for byte.
7. Route the membership filters through the phase too.
8. Leave `row_set.rs:401-409` alone and name it in the PR.

## Scope as landed

Written 2026-10-07 at the merge (`f32c29191`, unit commit `c7dea8ca5`) from the implementor's
confirmation table and the adversary passes; the scoper's Scope above stays as it was.

| scoper's line | as landed |
|---|---|
| #486 rule, `subject_state.rs`, `related_guard.rs` sites (cited) | confirmed; each reads phases from `precedence::place` and order from `phase_order` |
| `row_set.rs:401-409` refusal-first rule | untouched, as briefed |
| membership filters drop "branches answered in other phases" | **wrong**: they drop branches the partition cannot prove (markers, `external:`, defaults); kept, comments reworded |
| decision 3, read the plan's step 5 instead of `orders_present_refusals` | **not taken**: it would admit `wrong_state:` beside a present-related refusal and acceptance (ess/22, input row), which validation refuses today (R2); the epic changes no verdict, so the rule stays and an adversary test pins R2 |
| decision 5, a `#[cfg(test)]` classifier seam | not needed: wave 2's `with_phase_order` reaches the checks |
| decision 6, the accepting/external word from the phase | not possible (both are `Accepting`); taken from the condition shape, message bytes unchanged |
| per-pair `refusal_settles` (round 1) | **replaced** in round 2: the input refusal answers first only when its phase is read first among the selected branches; a present-related refusal stops answering first only when an accepting branch is read before it |

Acceptance bullet 2 ("none of the three checks matches on `OutcomeCondition` to decide order") is
met except `orders_present_refusals`, kept for R2 as above. Measured: 18,530 (model, check)
outputs identical to the base.
