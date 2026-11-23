---
format: aep.planning-md/3
id: story:a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose
kind: story
status: active
title: A setup run of a creating branch with several related rows uses the field values another scenario chose
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:a-setup-run-of-a-creating-branch-that-reads-several-related-rows-creates-its-row-with-the-field-values-another-scenario-chose
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv
- confidence: cited
  path: crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/a-creator-chosen-values-beside-several-rows.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/a-creator-reading-several-related-rows.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-component-suites-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/own_scenario_nesting_bounded_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/related_guard_multiple_vias.rs
revision: 22
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:03:28Z", actor: "human:timo", revision: 21, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:03:29Z", actor: "human:timo", revision: 22, decided_on: {"recorded":{"review_outcome":5}}}
---
## Outcome

- A scenario that needs a row whose field values the scenario chose is witnessed where the row's creating branch reads two or more related rows through its input and the strict search finds no run. The run is arranged as the creating branch's own scenario (`as_own_scenario`) with the chosen values sent, where today it is refused. This is the one statement of the outcome; tests 1 and 3, step 3, D8 and acceptance 2 say the same: **0 refusals, with no leg left over.** On the new fixture `a-creator-chosen-values-beside-several-rows.yaml` (`CHOSEN`): `UseGrant/outcome/used` and every leg of `UseGrant/outcome/grant-refused` exist, 0 refusals under `shop.desk.`, every scenario passes against `Interpreted`. On the existing fixture (`DESK`): the three legs of `shop.desk.ReceiveTicket/outcome/grant-refused` exist, and `related_guard_multiple_vias.rs::driven` asserts 0 refusals where it pins 3 today. A unit that ends with a leg still refused does not land: it stops and reports (D8).
- Every run the strict search finds, and every refusal of a run that still fails, keeps its bytes. Only a creating-branch run that was refused with a chosen input changes.

## Why

- Parent: `story:a-setup-run-of-a-creating-branch-that-reads-several-related-rows-creates-its-row-with-the-field-values-another-scenario-chose` (serves O2).
- Unit S3 of this wave arranges such a run as the branch's own scenario only when the run carries no chosen input (`drive_several`: `input.is_none()`), so any scenario needing a non-default value (`name == mail`, a state with `mode == denied`) is refused.
- The consumer gap: about 26 of an adopter's remaining refusals have this root (see the parent story's Need). It lands before the draft story `a-setup-runs-refusal-names-the-real-cause-where-its-branch-has-a-scenario-of-its-own`, which is not in this wave.
- Order in the wave: c1 is in round 1 and lands before unit p2 (`story:every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads`), which depends on it, classifies c1's counter `OWN_OVERLAY_WORK` in its registry and adds it to its `Work`; and before t2 (`story:the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane`), which stamps the tree that holds c1's new fixture. c1 re-pins the three tables below with the commands they print today and touches no stamp. c1 and the first threads unit (`story:every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan`) are in one round: the second of the two to land reruns the three model-table targets on the merged tree before it lands; a row that differs is c1's re-pin, regenerated with the table's own command, never merged by hand.

## Where

Read in the wave worktree at commit `e3f502951b` (branch `integration/wave-2026-11-22`). Paths below are under `crates/verify/ess-conformance/`.

**Base: this unit is built on `e3f502951b` or a descendant of it on the wave branch.** That head holds the correction of the own-scenario fallback (`7ea3fb0196`, `0612918ae0`): the fallback is bounded by nesting, `const OWN_NESTING: usize = 1` (2858), and runs only while `own_depth() < OWN_NESTING`, that is while no own-scenario arrangement is open; at the bound the caller gets the strict search's refusal as before. There is no memo of the own-scenario arrangement (one keyed on what the arrangement reads had 0 hits: every arrangement carries its own distinction number). The counted work test is `tests/own_scenario_nesting_bounded_work.rs` (depths 1 to 4: arrangements at most 21 per level, strict searches at most 8 x depth squared, never more than 1 open; a depth-3 test pins 6 refusals verbatim in `REFUSED`). The remaining cost has a draft story of its own (`story:the-own-scenario-arrangement-of-a-setup-run-finds-an-equal-arrangement-again-instead-of-building-it-once-per-path`), not this unit. Step 1 re-reads the sites below by name at the commit you start from; if it is a later head, the numbers move and the names hold.

- `src/synthesize/related_guard.rs`, today:
  - `drive_several` (2939). The fallback arm (2960-2972) has the condition `instance.is_none() && input.is_none() && known.is_empty() && own_depth() < OWN_NESTING && own_scenario_applies(driver, bound, arranging)`, then `as_own_scenario(ir, driver, actors, distinction, bound).map_err(|_| cause)`. Change: drop `input.is_none()` and pass `input` on; and add one arm before `other => other` that fires when the strict search failed, `instance.is_none()`, `input.is_some()`, `known.is_empty()`, `own_scenario_applies(..)` holds and `own_depth() >= OWN_NESTING` (the bound stops a chosen-values run that the fallback would otherwise take): it counts one `overlay_worked(0, 1)` (D4) and returns the strict search's `Err(cause)` unchanged. The strict search stays first and its refusal stays the answer where the fallback fails, so runs the strict search finds are untouched.
  - `own_scenario_applies` (2979): a creating branch, every row it reads named by an input the caller has not bound, of an entity no arrangement around the run is arranging. Unchanged.
  - `as_own_scenario` (2996): opens `own_opening` (2913, `OWN_OPEN` at 2837), counts `own_worked(1, 0)`, builds `InvocationModels::plain(ir)`, calls `arranged(&models, driver.command, driver.outcome, actors, distinction, None)`, merges `setup.bound` into `bound`, then `super::invoke_with(ir, driver, None, actors, &bound, &input)` and puts `setup.steps` first. Change: take `chosen: Option<&BTreeMap<String, Node>>` and call `arranged_choosing` (below) with it; where `chosen` is `Some` and non-empty, also count one overlay arrangement, `overlay_worked(1, 0)` (D4).
  - The counters and the bound, unchanged by this unit: `OWN_OPEN` (2837), `OWN_WORK` `(u64, u64, usize)` (2842: arrangements, strict searches, deepest open), `OWN_NESTING` (2858), `own_depth` (2861), `own_scenario_work` (2868, re-exported `pub` in `synthesize.rs` 3516), `own_worked` (2891), `own_opening` (2913). The fallback arm calls `own_worked(1, 0)` in `as_own_scenario` and the strict search counts `own_worked(0, 1)`.
  - `Around` (1944-1952, `Clone, Copy`, fields `command`, `outcome`, `rows`, `relaxed`) and its methods `hints` (1957), `admits` (1972; calls `leaves_to` (1265) per row beside). Change: add `chosen: Option<&'a BTreeMap<String, Node>>`.
  - `arranged` (3773): the entry for the several-rows case; builds `others` with `beside`, calls `arranged_beside` (3838), on `Err` calls `steered_beside` (3895), then `overlaps` where `goal` is none. Change: add `arranged_choosing(models, command, outcome, actors, distinction, goal, chosen)` holding today's body, make `arranged` call it with `None` (its callers `prepare_at_in` 3756, `unaccompanied` and the `as_own_scenario` of today keep the old name). The `!several(command)` early return ignores `chosen`.
  - `arranged_beside` (3838): builds `Around { command, outcome, rows: others, relaxed }` (3850), calls `arranged_at`, then, where `relaxed` is not set, `passes(ir, (command, outcome), other, &input)?` for every row beside (the refusal with `tried: 1`). Change: the parameter `(others, relaxed)` becomes `(others, relaxed, chosen)`; put `chosen` into `Around`. Its three call sites are in `arranged` and `steered_beside` (twice).
  - `steered_beside` (3895): re-arranges for the input the first try chose. Change: carries `chosen` to its two `arranged_beside` calls.
  - `arranged_at` (4264): `let fixed = subject_identity(ir, outcome, &setup);` (4361) and the call `with_row(..., around, (&pinned, &setup.steps, &fixed))` (4379). `pinned = pins(...)` is at 4353. Change: extend `fixed` with the overlay (rule in Decisions D1) before the call.
  - `with_row` (5435) and `with_row_where` (5471): unchanged. They already extend every candidate input with `fixed` (`input.extend(fixed.clone())`, in `searched_inputs`), steer the focus row toward `fixed` (`subject_fact::grounded_on_input(ir, entity, fixed, &predicates)`), test every candidate with `around.admits` and `takes`. This is why the chosen values go in as `fixed`, not on the finished input.
  - `selects` (2079) is not called by this unit; see D2.
  - `vias(command)` (1034, `pub(super)`) lists the input fields that name a row, `(field, entity)`.
- `src/synthesize.rs`: `created_owned_made` (8778) calls `related_guard::drive(.., input, &chain)` (8801); `drive` (related_guard 2612) calls `drive_several` (2630). Not changed. Add the counter accessor `own_overlay_work`, beside `own_scenario_work` (3516, D4). The counter itself, `OWN_OVERLAY_WORK`, is a `thread_local!` `Cell` in the same `thread_local!` block as its neighbours `OWN_OPEN` and `OWN_WORK` (`related_guard.rs` 2834-2844), so unit p2's census of the crate's statics finds it by the same pattern as them.
- `src/synthesize/subject_fact.rs`: `make_creations` (5692) calls `created(.., Some(&input))` (about 5791) once per candidate input and drops a failed one without a cause. Not changed, read only.
- `tests/own_scenario_nesting_bounded_work.rs` (the correction's): `work(depth)` (172) reads `own_scenario_work(|| synthesize(&model))` on a generic chain; `ARRANGED_PER_LEVEL = 21` (188), `STRICT_PER_LEVEL = 8` (192), `the_own_scenario_fallback_nests_one_level_at_every_depth` (197), `REFUSED` (215, 6 refusals) and `past_the_nesting_bound_a_branch_is_refused_as_before_the_fallback` (252). Read only: this unit keeps it green with its bounds and its pinned refusals unchanged.
- `tests/related_guard_multiple_vias.rs` (754 lines): `DESK` (670), `OWN`, `DRIVEN`, `LEGS` (698), `every_status` (700), `driven` (715-745) with the pin `assert_eq!(result.suite.scenarios.len(), 17, ...)` and `assert_eq!(refusals.len(), 3, ...)` plus its comment "wait for the follow-up ... when it lands this count becomes 0". Helpers `replaced` (41), `ir` (57), `refusals_for` (179), `scenario` (224).
- `tests/fixtures/a-creator-reading-several-related-rows.yaml` (198 lines): the three legs are `any: [state == Revoked, {name: {none_of: [chat]}}, mode == denied]` of `ReceiveTicket/grant-refused` at line 168-175.
- Pin tables, each walks every ESS fixture file by `read_dir` and prints its own re-pin command on failure: `tests/fixtures/synthesis-suite-bytes-table.tsv` (row of the existing fixture at line 100, `suite=17 ... refusals=3 ...`), `../../specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv` and `../../generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv` (rows of the existing fixture at 236-240 and 127).
- Measured with the S3 unit's binary (not at this head; no build was run to write this story): `m-g21` (the new fixture) 13 scenarios, 3 refusals; the existing `DESK` 17 scenarios, 3 refusals; `m-g22` (`DESK` with `none_of: [mail]` in place of `none_of: [chat]`) 15 scenarios, 4 refusals (the legs plus `ReceiveTicket/outcome/received` and `ignored`).

## Tests first

Commands run from the repository root. Each new test is timed by the implementor in a debug build; none may run over 60 s (expected: under 3 s, a 144-line model); the step stays within 120 s.

1. `tests/related_guard_multiple_vias.rs::a_chosen_value_that_is_not_the_first_variant_is_arranged_beside_several_rows` (new). Model: the new fixture, `include_str!("fixtures/a-creator-chosen-values-beside-several-rows.yaml")` as `CHOSEN`. Asserts: `refusals_for(&result, "shop.desk.")` is empty; `scenario(&result, "shop.desk.UseGrant/outcome/used")` exists; `every_status(&result, model)` is `Passed` for every scenario. Red today:
   `assertion `left == right` failed: refusals: [...]` with `left: 3`, `right: 0` (two legs of `UseGrant/outcome/grant-refused` and `UseGrant/outcome/used`, code `ESS-SYNTH-003`).
2. Same file, `the_chosen_values_are_the_ones_sent` (new). Same model; in `UseGrant/outcome/used` find the `ScenarioStep::ExecuteCommand` of `shop.desk.IssueGrant` and assert its input `name` reads `mail` (compare `format!("{value:?}").contains("mail")`) and `mode` does not read `denied`; assert the `site_id` and `desk_id` inputs are `ScenarioValue::Instance` (the arrangement's rows, not the chosen ids). Red today: `scenario(..)` panics `shop.desk.UseGrant/outcome/used exists; refusals: [...]`.
3. Same file, `driven` edited: `assert_eq!(refusals.len(), 0, ...)`, the loop over refusals removed, the comment about the follow-up removed, the scenario count asserted as it measures (D5). Red today: `left: 3`, `right: 0` at `refusals`. Both existing tests that call it (`a_creator_whose_owned_rows_link_is_compared_with_the_input_is_driven`, `a_creator_whose_row_beside_reads_the_input_is_driven`) go red the same way and become green together.
4. Same file, `a_row_beside_the_focus_is_created_with_a_chosen_value` (new): `replaced(DESK, "              - {name: {none_of: [chat]}}\n", "              - {name: {none_of: [mail]}}\n")`; asserts `ReceiveTicket/outcome/received` and `ReceiveTicket/outcome/ignored` exist, `refusals_for(.., "shop.desk.")` is empty, every status `Passed`. Red today: 4 refusals, `ReceiveTicket/received` among them (`no candidate of the 1 tried satisfies a row of shop.desk.Grant for input.grant_id ... on which no branch answering before received is selected`).
5. Same file, `a_chosen_value_the_branchs_own_guard_refuses_is_not_sent` (new): the `CHOSEN` model with, inside `IssueGrant`'s outcomes just before `      - name: issued`, the branch `      - name: not-asked\n        when: mode == asked\n        error: shop.desk.GrantRefused\n` (use `replaced`). Asserts 0 refusals under `shop.desk.` and every scenario `Passed` (no setup run sends a `mode` that its own `not-asked` branch answers). Red today: the same 3 refusals as test 1. This is the control that the overlay is checked by the branch's selection (D1), not just set.
6. Same file `tests/related_guard_multiple_vias.rs` (a test beside the others: the counters are thread-local and read through a closure, so nothing needs a process of its own, and no new test binary is added, X7): `the_chosen_values_are_arranged_in_counted_work`. Reads `own_overlay_work(|| synthesize(&model))` (D4, a pair `(built, asked_at_the_bound)`) on `CHOSEN`, and on `CHOSEN` with the enum `GrantName` extended from `[chat, mail, push]` to six variants (`replaced` on the `variants:` line). Asserts `built >= 1` for the first (the overlay is used at all; the stub accessor of step 2 counts nothing, so this is the red line: `left: 0`, `right: 1`, or a failed compile naming `own_overlay_work` before step 2), `built(second) <= 2 * built(first) + 2` (linear in the candidates, not multiplied), and, from `own_scenario_work` read on the same runs, `open <= 1` (the nesting bound) and `arranged >= built` (every overlay arrangement is one of the counted own-scenario arrangements). It also pins the first run's `own_scenario_work` triple exactly as measured after this unit. It adds a count; `tests/own_scenario_nesting_bounded_work.rs` is not edited and stays green with its bounds unchanged.
6b. Same file, `a_chosen_values_run_nested_in_an_own_scenario_is_refused_as_before` (new). The nesting bound is 1, so a chosen-values setup run that is asked while an own-scenario arrangement is open (depth 2) is not taken and gets the strict search's refusal, as before this unit. The test asserts, in this order: (a) `asked_at_the_bound >= 1`, the second figure of `own_overlay_work` (D4), which counts exactly the chosen-values runs that reached the fallback with the strict search failed and were stopped by `own_depth() >= OWN_NESTING`: a chosen-values run was asked at depth 2; (b) `open <= 1`; (c) only then the refusals (below). A model that never asks a chosen-values run at depth 2 fails (a); it cannot pass on the refusals alone. The model is `CHOSEN` plus a second creator that needs the fallback inside the first one's arrangement: add an entity `Zone` (created by an `OpenZone` with no input) and make `OpenDesk` read `input.zone_id` (`exists: false` refusal `no-zone`) beside `input.site_id`, with an `elsewhere`-style branch `when_related: {via: input.zone_id, predicate: site_id != input.site_id}` and `Zone` carrying a `site_id` field set from `input.site_id`, mirroring `IssueGrant`, so that `Desk`, which `IssueGrant`'s own-scenario arrangement creates, is itself created through the fallback. Assertion (c): the refusals under `shop.desk.` that belong to the runs counted in (a) are exactly a pinned list, each with its strict cause verbatim, as `REFUSED` does in the correction's test. The implementor reads that list once from today's code (step 1, before any edit: the same runs' refusals there) and the list after the change must equal it row for row; that equality is the proof of "refused as before". Chosen-values runs at depth 1 are taken by the change and are not in the list. If (a) reads 0 on the model, change the model (add a third creator on the chain, as the correction's test does for depth 3) until a chosen-values run is asked at depth 2; if none of three models asks one, stop and report: then the interaction cannot arise on a model and the arm of `drive_several` that counts it is dead code, which the coordinator decides. Do not weaken the test to the depth assertion alone and do not invent a model that proves nothing. Red today: (a) is red until step 5 adds the count (the stub reads 0); (c) reads today's code and stays green through the change; it fails only where the change lets a nested run through.
7. Pin tables, re-pinned in step 7, each printing red before: `synthesis_suite_bytes_table` (red because `a-creator-chosen-values-beside-several-rows.yaml` has no row and the `a-creator-reading-several-related-rows.yaml` row's `suite=17 ... refusals=3` moves), `selection_precedence_table` and `lowered_definitions_table` (red because the new file has no row).

Existing tests that must stay green unchanged: `own_scenario_nesting_bounded_work` (the correction's, bounds and pinned refusals unchanged), `once_per_model_work`, `stored_rows_decided_once_work`, `created_rows_kept_work`, `related_guard_multiple_vias` (its other tests), `related_guard_owner_link`, `related_guard_stored_reference`, `related_guard`.

## Steps

1. Start from `e3f502951b` (or a descendant on the wave branch; `git rev-parse --short HEAD` in your tree, named in the report). Re-read by function name: `drive_several`, `own_scenario_applies`, `as_own_scenario`, `own_opening`, `own_depth`, `OWN_NESTING`, `own_scenario_work`, `own_worked`, and `tests/own_scenario_nesting_bounded_work.rs`. Write the line numbers you read into your report. Run `own_scenario_nesting_bounded_work` once before any edit and keep its printed counts (it prints arrangements, strict searches and open per depth): they are the base your change is held to.
2. Write tests 1-6b first (the fixture below as a new file, `include_str!`; all in `tests/related_guard_multiple_vias.rs`), add `own_overlay_work` as a stub (`OWN_OVERLAY_WORK`, a `thread_local!` `Cell<(u64, u64)>` in the block of `OWN_OPEN` and `OWN_WORK`, counting nothing; `#[doc(hidden)] pub fn own_overlay_work` in `synthesize.rs` re-exporting a `pub(super)` one in `related_guard.rs`, beside `own_scenario_work`), run the target and record the red lines. Before any edit of step 4, read once from today's code the refusals of the runs that test 6b's model will ask at depth 2 (pin list of 6b (c)). Time each new test.
3. Outcome is 0 refusals (see Outcome): any stop below leaves the unit unlanded. Print once, not committed, what `drive_several` receives for the `IssueGrant`/`issued` creation of `CHOSEN` with `Some(input)`: `instance`, `known.len()`, `bound` keys, `arranging`. The fallback is admitted for `Some(input)` only where `instance.is_none()`, `known.is_empty()` and `own_scenario_applies` hold, as for `None`; if `bound` or `known` is not empty there for the creation of a row filed under an owner, stop and report which and why under "what you could not verify" rather than widening the fallback condition (D6).
4. Add `chosen` to `Around`, thread it `arranged_choosing` to `arranged_beside` to `steered_beside` (the parameter groups named under Where), and extend `fixed` in `arranged_at` per D1.
4a. Check that option B is exercised, before step 5 goes on. Print once, not committed, the `fixed` map that `arranged_at` hands to `with_row` for `IssueGrant/issued` while `CHOSEN` is synthesized with the overlay `{name: mail, mode: ...}`: it must hold `name` and `mode` and neither `site_id` nor `desk_id` (the `vias` fields the arrangement owns), and the focus row the search picks must carry them. Run test 1. Stop condition: if `fixed` never holds the chosen fields (the filter of D1 drops them, or `with_row` does not steer by `fixed` for them), or it holds them and test 1 still reads the same 3 refusals, then B does not exercise `fixed` on this model: edit nothing more, leave the unit unlanded and report the printed `fixed`, the refusals and the line where the values were lost. The fallback is the story's own recommendation, option A (the overlay on the finished input, re-checked by hand through `projection`, `decided_beside`, `passes` and `leaves_to`); it is a change of D1 that the operator decides, and is not started inside this unit.
5. `as_own_scenario` takes `chosen`; `drive_several` passes `input`, drops `input.is_none()` (the `own_depth() < OWN_NESTING` condition stays, D11) and gains the counting arm for a chosen-values run stopped by the bound; no memo is added (D3); `overlay_worked(1, 0)` per own-scenario arrangement that carries a non-empty overlay and `overlay_worked(0, 1)` per run stopped by the bound (D4).
6. Run tests 1-6b, then the existing-green list. Run `own_scenario_nesting_bounded_work` and compare its printed counts with step 1's: its bounds (21 per level, 8 x depth squared, 1 open) and its 6 pinned refusals must hold unchanged. If the chain's counts or its `REFUSED` list move, stop and report the numbers: the fix is to the overlay, never to the correction's bounds or pins. Where another test goes red for a reason the design above does not explain, report it; do not weaken an assertion.
7. Re-pin: run, one at a time, the command each table test prints. From the repository root, with `$BASE` the commit before your work (`git rev-parse HEAD` before editing):
   - `SYNTHESIS_SUITE_BYTES_TABLE_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv" slot cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table`
   - `SELECTION_PRECEDENCE_TABLE_WRITE="$PWD/crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv" slot cargo test -p ess-compiler --locked --test selection_precedence_table`
   - `LOWERED_DEFINITIONS_TABLE_WRITE="$PWD/crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv" slot cargo test -p ess-entity-runtime --locked --test lowered_definitions_table`
   - Then for each table `T`: `comm -13 <(git show "$BASE:$T" | sort) <(sort "$T")` must print the new header and only lines beginning with `crates/verify/ess-conformance/tests/fixtures/a-creator-chosen-values-beside-several-rows.yaml` plus, in the synthesis table only, the one moved row of `a-creator-reading-several-related-rows.yaml`; `comm -23` the same for the removed lines. Any other moved row is a regression: stop and report it. The component table (`synthesis-component-suites-table.tsv`) is written by the same test run (`SYNTHESIS_COMPONENT_SUITES_TABLE_WRITE`); it must not change.
   - Time the synthesis of the new fixture and of `DESK` before and after the change (the write mode prints each model's synthesis time); a jump is fixed in this change.
8. `CHANGELOG.md` (below). `task fmt-check` through `slot`.

## Owned files

- `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (the functions named under Where only)
- `crates/verify/ess-conformance/src/synthesize.rs` (the `own_overlay_work` accessor only)
- `crates/verify/ess-conformance/tests/related_guard_multiple_vias.rs`
- `crates/verify/ess-conformance/tests/fixtures/a-creator-chosen-values-beside-several-rows.yaml` (new)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv`, `crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv`, `crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv` (re-pinned)
- `CHANGELOG.md` (one bullet under `### Fixed`)

## Not yours

- `tests/own_scenario_nesting_bounded_work.rs`, `OWN_NESTING` (stays 1), `OWN_WORK`'s tuple type, `own_scenario_work`, `own_depth`, `own_opening`: the correction's; use them, do not change them, raise no bound.
- The own-scenario arrangement's remaining cost (equal arrangements found again, once per path): the draft `story:the-own-scenario-arrangement-of-a-setup-run-finds-an-equal-arrangement-again-instead-of-building-it-once-per-path`; this unit adds no memo for it.
- `tests/fixtures/a-creator-reading-several-related-rows.yaml`: its bytes do not change; only its table row moves.
- `beside_refused` and the `tried:` constants in `related_guard.rs`, `created_owned_made`'s refusal mapping in `synthesize.rs`: the draft refusal-cause story edits them later.
- `selects`, `answering_refusal`, `input_guard`, `row_set.rs`: the draft row-set story edits them.
- `drive_toward` (2748): its `several` refusal stays (D7).
- `crates/edge/ess-xtask/` and every file under `website/` and `docs/`; this unit changes no format, no diagnostic code and no CLI text.
- The byte-pin stamp file and its digest test (unit t2's): c1 writes no stamp and runs none; t2 lands after c1 and stamps the tree that holds c1's fixture.
- The classification of `OWN_OVERLAY_WORK` in the thread-state registry, its entry in `Work` and `crates/verify/ess-conformance/src/synthesize/threads.rs`: unit p2's, which lands after c1.
- The other units of the wave (`CHANGELOG.md` is the only shared file: keep both bullets at merge).

## Gate

Per `AGENTS.md` § Gate: a change under `ess-conformance` synthesis also gates `ess-cli`; `ess-conformance` runs one `--test` at a time. Every build and test through `slot`; `CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0` in the environment; a test over 60 s is killed, the test step ends within 120 s, a pool job ends after 10 minutes unless it names its time (`SLOT_TIMEOUT=30m` for the first `ess-conformance` build of a fresh worktree and for `ess-cli`). No Python; nothing in `/tmp`; no `RUSTC_WRAPPER` or `SCCACHE_*`.

```console
slot task fmt-check
SLOT_TIMEOUT=30m cargo clippy -p ess-conformance --all-targets --locked -- -D warnings
cargo test -p ess-conformance --locked --test related_guard_multiple_vias
cargo test -p ess-conformance --locked --test own_scenario_nesting_bounded_work
cargo test -p ess-conformance --locked --test once_per_model_work
cargo test -p ess-conformance --locked --test stored_rows_decided_once_work
cargo test -p ess-conformance --locked --test created_rows_kept_work
cargo test -p ess-conformance --locked --test related_guard_owner_link --test related_guard_stored_reference --test related_guard
cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table
cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table_cli_read
cargo test -p ess-conformance --locked --test adversary_related_values_all_bytes
cargo test -p ess-conformance --locked --test external_beside_held_guard
cargo test -p ess-compiler --locked --test selection_precedence_table --test string_alphabet_and_length
cargo test -p ess-entity-runtime --locked --test lowered_definitions_table
cargo test -p ess-synth --locked --test served_codec_names
cargo test -p ess-openapi --locked --test adversary_ess_issues_2_u6
SLOT_TIMEOUT=30m cargo test -p ess-cli --locked --test tutorial_page
```

The lines from `synthesis_suite_bytes_table` down are the repository-wide byte pins `AGENTS.md` names for a change to synthesis; none of them is re-pinned unless this unit claims the row (acceptance 4 and 5), and they run as they are today: this unit lands before the stamp unit t2. Do not start `task check` or a whole-workspace run. `ess-synth`'s `go` target and the harness-based tests need the harness tasks of `AGENTS.md` § Gate; run `go` only if the repository state allows, else name it in the report as not run.

## Acceptance

1. `cargo test -p ess-conformance --locked --test related_guard_multiple_vias`: tests 1-6b above pass (6 and 6b included, X7: one target, no new binary); `driven` asserts 0 refusals and every scenario `Passed`; no test in the file over 60 s.
2. `UseGrant/outcome/used`, and every leg scenario of `UseGrant/outcome/grant-refused` and of `ReceiveTicket/outcome/grant-refused`, exist in the synthesized suites of `CHOSEN` and `DESK`, and each passes against `Interpreted` (tests 1 and 3), with 0 refusals in both suites. A leg that stays refused is a stop, not a pinned count (D8).
3. Test 6 (`the_chosen_values_are_arranged_in_counted_work`, in `related_guard_multiple_vias`) passes, and `own_scenario_nesting_bounded_work` (bounds and the 6 pinned refusals unchanged), `once_per_model_work` and `stored_rows_decided_once_work` pass with their bounds unchanged.
4. The three pin tables pass after re-pinning, and the `comm` proof of step 7 shows that only the new fixture's lines and the one moved row of `a-creator-reading-several-related-rows.yaml` changed; the component table is byte-identical.
5. The re-pins of acceptance 4 are the only ones: the other byte-pin targets of the Gate section (`synthesis_suite_bytes_table_cli_read`, which reads the re-pinned synthesis table and passes with it, `adversary_related_values_all_bytes`, `external_beside_held_guard`, `string_alphabet_and_length`, `served_codec_names`, `adversary_ess_issues_2_u6`, `tutorial_page`) pass with no edit to any file they read other than those three tables (they hold the runs the strict search finds).
6. Test 6b (`a_chosen_values_run_nested_in_an_own_scenario_is_refused_as_before`) passes: a chosen-values run asked while an own-scenario arrangement is open is refused by the nesting bound, `open <= 1` holds, and the correction's depth-3 `REFUSED` pin is unchanged (D11).
7. The existing-green list passes unchanged.
8. Timings are in the report: each new test, and the synthesis of `CHOSEN` and `DESK` before and after; no test over 60 s, the test step within 120 s.
9. `git diff --stat` shows only the owned files; `cargo xtask schema`, `cargo xtask cli-reference` and every projection need no change (nothing under `RawSpecFile`, no code, no format).

## Decisions

- D1 (the story's open D1; decided B, against the story's recommendation A). The chosen values enter the arrangement as `fixed` input fields (extended onto every candidate input of the focus row's search, and steering it), not as an overlay on the finished input. Reason: with A the finished input must then be re-checked against every row beside and the focus row, and `selects` (2079) takes one row, so A needs `projection`, `decided_beside`, `passes` and `leaves_to` called again by hand; B puts the values in before `Around::admits`, `takes` and `passes` run, so the existing checks decide it, once. The rule in `arranged_at`, after `let fixed = subject_identity(..)` and `pinned = pins(..)`: for each `(field, value)` of `around.chosen`, insert into `fixed` unless `fixed` already has the field, `setup.bound` or `pinned` names it, or it is one of `vias(around.command)` (a field naming a row: the arrangement owns those). `around.command` is the whole command; `arranged_at`'s own `command` is its projection.
- D2 (the dispatch question: `selects` or `projection`/`decided_beside`/`passes`). Neither is called new. `selects` stays one-row, called only inside `Around::admits`/`takes` as today. The several-row check is the existing `arranged_beside` check (`Around::admits` while searching, `passes` for each row beside afterwards), and runs on the input that holds the overlay. No new multi-row check is written.
- D3 (departure from the parent's D3, listed for the critics beside the one from D1). The parent's D3 option A, its recommendation, says to measure whether the added CPU comes from the new scenarios or from failed fallbacks and to bound the failed fallbacks with a memo. This unit takes neither: no memo, no CPU split, and the repeated arrangement is left to `story:the-own-scenario-arrangement-of-a-setup-run-finds-an-equal-arrangement-again-instead-of-building-it-once-per-path` (the wave page puts D3 to the critics as a veto choice, as it does D1). The only measurement kept is step 7's synthesis time of `CHOSEN` and `DESK` before and after. No memo, and no local variable shared between candidates. The correction ended without a memo of the own-scenario arrangement (0 hits: every arrangement has a distinction of its own), and under D1 B the arrangement itself depends on the chosen overlay (the overlay is in `fixed` before `Around::admits`, `takes` and `passes` run), so "build once, overlay per candidate" would be option A, which D1 rejects. Nor is there a loop in `drive_several` to hold a local: it is called once per candidate input, from `make_creations` (`subject_fact.rs` 5692, `created(.., Some(&input))` at about 5791). Equal candidates are already answered once by `created_memo` (keyed on the input), so one arrangement is built per distinct candidate input where the strict search fails. The cost is bounded by the nesting bound (the fallback never runs inside an open own-scenario arrangement, `own_depth() < OWN_NESTING`) and counted by this unit's own test (test 6): overlay arrangements `built(second) <= 2 * built(first) + 2` when the candidates' enum doubles. If the measured synthesis time of `CHOSEN` or `DESK` jumps after the change, that is reported with the two times and left to the draft story on the repeated arrangement; it is not fixed here with a memo.
- D4. Shape in one line: `OWN_OVERLAY_WORK` is one `thread_local!` `Cell<(u64, u64)>`, the pair `(built, asked_at_the_bound)`, read through `own_overlay_work`. The unit's own count is a separate counter, `OWN_OVERLAY_WORK`: a `thread_local!` `Cell<(u64, u64)>` in the block of `OWN_OPEN` and `OWN_WORK` (a counter, not a cache, not a context), read by `#[doc(hidden)] pub fn own_overlay_work<R>(run: impl FnOnce() -> R) -> (R, (u64, u64))` in `synthesize.rs` like `own_scenario_work` (the delta over `run`); unit p2 classifies it. `OWN_WORK`'s `(u64, u64, usize)` is not changed, so `own_scenario_nesting_bounded_work` cannot break through the counter. The pair is `(built, asked_at_the_bound)`: `built` counts the own-scenario arrangements that carried a non-empty overlay (`overlay_worked(1, 0)` in `as_own_scenario` beside `own_worked(1, 0)`); `asked_at_the_bound` counts the chosen-values runs that reached the fallback after the strict search failed and were stopped by `own_depth() >= OWN_NESTING` (`overlay_worked(0, 1)` in the extra arm of `drive_several`), which is what test 6b needs to know that its model asks. What it adds to `own_scenario_work`: that count is separate from the arrangements made without an input, so a test can bound the overlay's own work and see it is used at all; the correction's `arranged` figure still counts every one (an overlay arrangement is one of them).
- D11. The nesting bound is untouched and decides the interaction. `drive_several`'s condition keeps `own_depth() < OWN_NESTING` (1): a chosen-values setup run is taken only while no own-scenario arrangement is open. A chosen-values run asked inside an own-scenario arrangement (depth 2) is not taken and gets the strict search's refusal, exactly as a run without an input does at that depth. Test 6b pins that; the depth-3 refusals of the correction's test stay as they are.
- D5. The scenario count pinned by `driven` and `CHOSEN` is the one measured after the change; the legs may become scenarios (17 may grow). State the measured count and each id that is new in the report. Not guessed here.
- D6. The condition of the fallback keeps `instance.is_none()`, `known.is_empty()` and `own_scenario_applies`; only `input.is_none()` goes. A moving run and a run after steps stay as S3 left them (the story's Design). The later lifecycle state is reached by the fact search's moves after the creation.
- D7. `drive_toward`'s `several` refusal stays (the story's D2, option A, no measured case).
- D8. The `any:` legs and `used` are the acceptance (the story's Outcome, three kinds of scenario). The outcome is 0 refusals and no leg is left over. If a leg stays refused after the change, do not widen the design and do not pin a nonzero count: the unit does not land, and the report names which leg, the refusal text, and which of `fixed`, `Around::admits` or `passes` refused it, for the coordinator to decide. The `later state` kind is covered through `Grant/state/Revoked/refuses/...` in `DRIVEN`; no new code for it.
- D9. The new fixture is generic (`shop.desk`) and is added exactly as written below.
- D10. `CHOSEN` is a test-local name for `include_str!("fixtures/a-creator-chosen-values-beside-several-rows.yaml")`.

The fixture, `crates/verify/ess-conformance/tests/fixtures/a-creator-chosen-values-beside-several-rows.yaml`, byte for byte:

```yaml
format: ess/37
system: shop
version: v1
domain: shop.desk
summary: >-
  A grant is issued only to a desk of the site the caller names; a grant is used only when it is
  not revoked, carries the mail name and is not denied.
types:
  - {name: shop.desk.SiteId, kind: newtype, of: Uuid}
  - {name: shop.desk.DeskId, kind: newtype, of: Uuid}
  - {name: shop.desk.GrantId, kind: newtype, of: Uuid}
  - {name: shop.desk.GrantName, kind: enum, variants: [chat, mail, push]}
  - {name: shop.desk.Mode, kind: enum, variants: [free, asked, denied]}
entities:
  - name: shop.desk.Site
    identity: {name: site_id, type: shop.desk.SiteId}
    relations:
      - {name: desks, kind: owns, target: shop.desk.Desk, cardinality: many, via: site_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: shop.desk.Desk
    identity: {name: desk_id, type: shop.desk.DeskId}
    fields:
      - {name: site_id, type: shop.desk.SiteId}
    relations:
      - {name: grants, kind: owns, target: shop.desk.Grant, cardinality: many, via: desk_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: shop.desk.Grant
    identity: {name: grant_id, type: shop.desk.GrantId}
    fields:
      - {name: desk_id, type: shop.desk.DeskId}
      - {name: site_id, type: shop.desk.SiteId}
      - {name: name, type: shop.desk.GrantName}
      - {name: mode, type: shop.desk.Mode}
    lifecycle:
      initial: Granted
      states: [Granted, Revoked]
      terminal: [Revoked]
      transitions:
        - {name: revoke, from: [Granted], to: Revoked}
errors:
  - {name: shop.desk.SiteNotFound, summary: No site carries this identity., fields: []}
  - {name: shop.desk.DeskNotFound, summary: No desk carries this identity., fields: []}
  - {name: shop.desk.DeskElsewhere, summary: The desk belongs to another site., fields: []}
  - {name: shop.desk.GrantNotFound, summary: No grant carries this identity., fields: []}
  - {name: shop.desk.GrantRefused, summary: The grant is revoked or does not allow this., fields: []}
events:
  - name: shop.desk.SiteOpened
    fields: [{name: site_id, type: shop.desk.SiteId}]
  - name: shop.desk.DeskOpened
    fields: [{name: desk_id, type: shop.desk.DeskId}]
  - name: shop.desk.GrantIssued
    fields: [{name: grant_id, type: shop.desk.GrantId}]
  - name: shop.desk.GrantRevoked
    fields: [{name: grant_id, type: shop.desk.GrantId}]
  - name: shop.desk.GrantUsed
    fields: [{name: grant_id, type: shop.desk.GrantId}]
actors:
  - name: shop.desk.Keeper
    may: [shop.desk.OpenSite, shop.desk.OpenDesk, shop.desk.IssueGrant, shop.desk.RevokeGrant, shop.desk.UseGrant]
commands:
  - name: shop.desk.OpenSite
    input: []
    outcomes:
      - name: opened
        creates: shop.desk.Site
        instance: site_id
        emits: [shop.desk.SiteOpened]
        payload: {shop.desk.SiteOpened: {site_id: {generated: true}}}
  - name: shop.desk.OpenDesk
    input:
      - {name: site_id, type: shop.desk.SiteId}
    outcomes:
      - name: no-site
        when_related: {via: input.site_id, exists: false}
        error: shop.desk.SiteNotFound
      - name: opened
        creates: shop.desk.Desk
        instance: desk_id
        emits: [shop.desk.DeskOpened]
        payload: {shop.desk.DeskOpened: {desk_id: {generated: true}}}
        sets: {site_id: input.site_id}
  - name: shop.desk.IssueGrant
    input:
      - {name: site_id, type: shop.desk.SiteId}
      - {name: desk_id, type: shop.desk.DeskId}
      - {name: name, type: shop.desk.GrantName}
      - {name: mode, type: shop.desk.Mode}
    outcomes:
      - name: no-site
        when_related: {via: input.site_id, exists: false}
        error: shop.desk.SiteNotFound
      - name: no-desk
        when_related: {via: input.desk_id, exists: false}
        error: shop.desk.DeskNotFound
      - name: elsewhere
        when_related: {via: input.desk_id, predicate: site_id != input.site_id}
        error: shop.desk.DeskElsewhere
      - name: issued
        creates: shop.desk.Grant
        instance: grant_id
        emits: [shop.desk.GrantIssued]
        payload: {shop.desk.GrantIssued: {grant_id: {generated: true}}}
        sets: {desk_id: input.desk_id, site_id: input.site_id, name: input.name, mode: input.mode}
  - name: shop.desk.RevokeGrant
    input:
      - {name: grant_id, type: shop.desk.GrantId}
    outcomes:
      - name: no-grant
        unknown_instance: true
        error: shop.desk.GrantNotFound
      - name: revoked
        moves: shop.desk.Grant.revoke
        instance: grant_id
        emits: [shop.desk.GrantRevoked]
        payload: {shop.desk.GrantRevoked: {grant_id: input.grant_id}}
      - name: wrong-state
        wrong_state: true
        error: shop.desk.GrantRefused
  - name: shop.desk.UseGrant
    input:
      - {name: grant_id, type: shop.desk.GrantId}
    outcomes:
      - name: no-grant
        when_related: {via: input.grant_id, exists: false}
        error: shop.desk.GrantNotFound
      - name: grant-refused
        when_related:
          via: input.grant_id
          predicate:
            any:
              - state == Revoked
              - {name: {none_of: [mail]}}
              - mode == denied
        error: shop.desk.GrantRefused
      - name: used
        emits: [shop.desk.GrantUsed]
        payload: {shop.desk.GrantUsed: {grant_id: input.grant_id}}
views:
  - name: shop.desk.Grants
    source: shop.desk.Grant
    consistency: read_your_writes
    fields:
      - {name: grant_id, type: shop.desk.GrantId}
      - {name: mode, type: shop.desk.Mode}
```

`UseGrant` reads one row, so the strict search drives it; its `used` branch needs a grant with `name == mail`, which is not the enum's first variant, so the fact search asks `IssueGrant/issued` (two rows read, a link of the desk compared with an input) for exactly that value.

## Open for the operator

None. D1 is a design choice the story left to this unit and it is made above with its reason; it is reversible in one file.

## Changelog

Under `## [Unreleased]`, `### Fixed`, appended after the last bullet of that heading:

- Synthesis witnesses a scenario that needs a related row with chosen field values, a grant whose `name` is `mail` or whose `mode` is `denied` for instance, where the row's creating branch reads several related rows through its input: the setup run is arranged as that branch's own scenario with the chosen values sent, where the scenario was refused (`ESS-SYNTH-003`). The `any:` legs of a refusal over such a row and the scenarios that need it in a later state are witnessed. Suites where the strict search already found the run keep their bytes.
