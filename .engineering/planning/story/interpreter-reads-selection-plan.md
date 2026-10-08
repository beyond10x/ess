---
format: aep.planning-md/3
id: story:interpreter-reads-selection-plan
kind: story
status: draft
title: The model interpreter selects through the selection plan
relations:
- decomposes: epic:one-selection-plan
- serves: vision:O2
- depends_on: story:selection-plan-design-and-type
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute/existence.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute/related.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/interpreter_selection_plan.rs
revision: 5
---
# Story: The model interpreter selects through the selection plan

## Why

`epic:one-selection-plan`. `select` and `responding_core`
(`crates/verify/ess-conformance/src/interpret/execute.rs`) decide the precedence order by matching
condition kinds in one routine, with `orders_present_related_refusal` and
`subject_refusals_before_present_related` as special cases. beyond10x/ess#486 was this routine
answering in declaration order where the design did not.

## What it delivers

The interpreter takes its branch order from `story:selection-plan-design-and-type`'s plan: phase by
phase, and within a phase in the plan's order. What each condition evaluates to (held rows, row
sets, related rows, provider choices, Unknown deferral) stays where it is.

## Acceptance

- Every existing `ess-conformance` test passes unchanged (CI's 20 test shards green).
- A new test, `tests/interpreter_selection_plan.rs`, builds a plan through
  `story:selection-plan-design-and-type`'s seam with the held-state phase (step 4) and the
  accepting/external phase (step 6) exchanged. On `tests/fixtures/external-beside-held-guard.yaml`
  with `rushed: when: rush == true` declared after `stale` (the model #487's
  `tests/selection_precedence.rs` builds), a pick holding revision 1 sent `revision: 2, rush: true`
  is answered `stale` under the real plan and `rushed` under the exchanged one. The test adds no
  seam of its own.
- `select` no longer matches on `ResolvedCondition` to decide order; the matches that remain decide
  what a branch's guard evaluates to.

## Out of scope

Synthesis (`story:synthesis-reads-selection-plan`), which shares the crate but not these files.

## Scope

Derived 2026-10-07 by `aep:story-scoper` on `985f58cc3a`. Every line is **cited** (read from the
story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance/src/interpret/execute.rs` — cited (story Why)
- **Files, ORDER (to read the plan):** — cited, read in tree
  - `execute.rs:488` `responding_core`: fixed sequence 502-511 (existence before related), 514-519 `related_absent`, 520 `refused_by_input`, 523 `existence::existing`, 556-597 held rows with early `unknown_instance` (589), 602-606 `addressed_row`, 613-633 present-related preflight, 634 `select`
  - `execute.rs:677` `orders_present_related_refusal`, `:696` `subject_refusals_before_present_related`, `:948` `is_present_related_refusal`
  - `execute.rs:744` `select`: loops 821-830 (input refusals), 831-864 (present-related), 865-936 (accepting/external, declaration order), 937-943 (`Otherwise`)
  - `execute.rs:1238` `stored_reference` (steps 3-5, calls `select` at 1274), `:1330` `refused_by_input`, `:1139` `related_absent`, `:1933` `unknown_instance` (chain: `UnknownInstance`, then `synthesize::declared_not_found` at 1945, then `wrong_state`)
  - `execute/related.rs:38` `reads_in_order`, `:72` `several`; `execute/existence.rs:106` calls `select(.., true, false)`
- **Files, EVALUATION (stay):** `execute/subject.rs:27` `Held::selects`; `execute/row_set.rs:214` `decisions` and `Reader`; the `select` closures `row_set_takes`/`holds`/`eligible_external`/`forced` (`execute.rs:759-818`); `related.rs:99` `reference`, `:121` `held`; `execute.rs:1086` `selected_subject_refusal` — cited
- **Callers of `select`, each with its own flag pair:** `execute.rs:634`, `:709`, `:1274`, `existence.rs:106` — cited
- **Tests per ordering path:** — cited
  - missing related row before input refusal: `tests/precedence_order.rs:97,117`
  - input refusal, first declared: `tests/refusal_beside_state.rs`, `tests/overlap_precedence_correction1.rs`
  - accepting/external in declaration order, `Open`: `tests/external_beside_held_guard.rs:497-734`, `tests/mixed_guard_overlap.rs`, `tests/accepting_guard_overlap.rs`, `tests/mixed_guard_wrong_state.rs`
  - ess#282 composition: `tests/interpreted_command_execution.rs`, `tests/related_guard_moves.rs`
  - several rows (#283): `tests/related_guard_multiple_vias.rs`, `tests/adversary_283_pass1.rs`
  - stored reference (#304): `tests/related_guard_stored_reference.rs`, `tests/adversary_related_via_stored_pass1.rs`
  - row sets and `addressed_row`: `tests/row_sets.rs` (through `tests/support_row_sets/mod.rs`), `tests/adversary_row_sets_precedence.rs`, `tests/row_set_upsert.rs` (#462)
  - existence: `tests/interpreted_existence.rs`; history path (`history.rs:194` → `responding_core`): `tests/row_sets_history.rs`, `tests/generated_history_values.rs`
- **Outside `interpret/`, not touched:** `linearize.rs:383` `reads_other_rows` reaches the order only through `history::execute`; `decision.rs:29`, `now_offset.rs:136,440,565`, `sessions.rs:266` and `interpret.rs:456` read guards, not order — cited
- **A seventh copy, in Go:** `src/go/explore.go:1543,1722` is the explorer's Go reference model; it reads `ir.json` and copies `interpret::execute::select` by name, so it cannot read a derived plan. Not listed in the epic — cited
- **Also likely:** `crates/verify/ess-conformance/tests/interpreter_selection_plan.rs` (new) — inferred
- **Documents:** the module table `execute.rs:13` and the `select` doc comment `:724-738` describe the order in prose — cited
- **Confidence:** high — the story names the routines, and every caller was traced by `git grep`
- **Would collide with:** any unit editing `interpret/execute.rs`, `execute/related.rs` or `execute/existence.rs`; PR #487 edits `tests/external_beside_held_guard.rs` and `tests/mixed_guard_overlap.rs`
- **Synthesis overlap:** `synthesize/related_guard.rs:853` has its own `orders_present_related_refusal`, and `execute.rs:1945` calls `synthesize.rs:11523` `declared_not_found`; both stay with `story:synthesis-reads-selection-plan` — cited
- **Safety fact:** every consumer reaches the order through `responding` (`execute.rs:453`) or `responding_core` (`:488`, `history.rs:194`), so keeping those signatures changes no caller outside `execute.rs` — inferred
- **Design decisions for the implementor:** — inferred
  1. Replace the two `bool`s on `select` (`include_present_related_refusals`, `prioritize_present_related_refusals`) with a plan phase range, so the four callers state which phases they read.
  2. Run each phase through a `match` on the plan's phase enum with no wildcard arm; any `ResolvedCondition` match that remains (`execute.rs:869`, `:907-934`) only picks the evaluator.
  3. Stopping rules differ by phase: input refusals stop at the first that holds; `Externals::Open` collects without stopping (`:927-931`). Keep this in the interpreter unless the plan models it.
  4. Delete `orders_present_related_refusal`, `is_present_related_refusal` and `related::several` once nothing uses them; leave the synthesis copy.
  5. A `#[cfg(test)]` constructor cannot be reached from an integration test: the swap test needs a `#[doc(hidden)] pub` constructor or a feature on the plan's crate.
  6. Build the plan once per `responding_core` call; never store it across decisions.

**Could not establish:** the plan type's name (`ess_domain::selection::SelectionPlan`,
`ess-domain/src/selection.rs:233`, and `ess_compiler::ir::ResolvedSelectionPlan`, `ir.rs:2392`, are
both taken); whether exchanging two phases changes the answer for `external-beside-held-guard.yaml`
(today `stale` and `unlisted` share one loop, `execute.rs:865`); a test for the multi-address branch
of `existence.rs:106`.
