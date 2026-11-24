---
format: aep.planning-md/3
id: story:three-adversary-targets-run-a-reduced-sweep-in-the-test-step-and-the-full-sweep-in-the-slow-lane
kind: story
status: active
title: Three adversary targets run a reduced sweep in the test step and the full sweep in the slow lane
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
- depends_on: story:a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-xtask/tests/slow_lane.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_ess0700a_identity_beside_row_set_2.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_u22_pass1.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/binding_trigger_row_predicate.rs
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:02:56Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:02:57Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

Three `ess-conformance` test targets (addition A: about 800 s, about 105 s and one test over 60 s) each end within 30 s in the step by the runner's own figure, on a warm build at a 1-minute load of 60 or less, with the full sweep kept in the slow lane. A target that is over 30 s after the planned changes gets its slowest remaining test marked slow, one at a time, until it is within; no target is left over 30 s.

- `adversary_u22_pass1`: the two seeded sweeps run a few seeds in the step; the full seed ranges, with the same assertions, run in the slow lane.
- `adversary_ess0700a_identity_beside_row_set_2` (the `ess-conformance` file; the same-named file in `ess-domain` is not this one): the order sweep tries the rule's own order and its adjacent transpositions in the step (n orders); all n! orders run in the slow lane.
- `binding_trigger_row_predicate`: its time is measured first, and the cause decides between three named changes (step 2); no test of it runs over 60 s.

## Why

Parent: `story:the-test-step-meets-the-test-budget`, step 5 (`slow` lane) and addition A (the three targets; `synthesize(` called in the test, lines 338, 223 and 497 of today's files, the cost paid once per test or once per seed or order).
Rule: "Tests stay fast": an expensive product made once, never per test; no test over 60 s; the step within 120 s.
Consumer gap: none.

## Where

Read at `80794bbd698` (unchanged since `bf128f9146`). Re-read a line before editing it.

`crates/verify/ess-conformance/tests/adversary_u22_pass1.rs` (727 lines): `model` 38, `live` 61, `directed_steps` (imported helper), `missing` 235, `seeded_chain` 542, `guard` 520, `plain_guard` 660.
- Sweep 1: `#[test]` 554, `every_reachable_branch_of_seeded_sixteen_refusal_chains_over_the_adopter_shape_has_its_scenario` 555-576: `for seed in 0..24u64`, `seeded_chain(seed, 16)`, `live`, `directed_steps(|| synthesize(&ir))` at 563, `missing`, a failure list, `eprintln!` of dropped, and the bound `dropped * 4 < 24 * 16` at 572.
- Sweep 2: `#[test]` 682, `every_reachable_branch_of_seeded_plain_sixteen_refusal_chains_has_its_scenario` 683-717: `for seed in 100..148u64`, chains of `plain_guard`, `synthesize` at 696, bound `dropped * 4 < 48 * 16` at 713.
- The other six tests stay unless measured over 30 s: 329, 349, 373 (`a_refusal_the_step_bound_cut_short_names_the_bound`), 580 (`seed_22_reachable_refuse_14_over_the_adopter_shape_has_its_scenario`), 606, 721 (`a_seeded_chain_synthesizes_the_same_suite_twice`).

`crates/verify/ess-conformance/tests/binding_trigger_row_predicate.rs` (824 lines): `FIXTURE` 19-150, `model` 151, `statuses` 202 (runs the interpreted target against the suite), `status_of` 213, `kept` 319. Tests that synthesize `FIXTURE`: 222 `one_row_deciding_trigger_and_invoked_command_gives_flow_delivery_and_the_refusal` (`synthesize` 223), 252 `the_tier_is_created_for_the_branch_the_scenario_claims` (253), 271 `the_interpreted_target_passes_every_scenario` (273, then `statuses` 274), 286 `a_target_that_never_invokes_issue_fails_the_flow` (287, `statuses` 300), 305 `a_target_that_retries_not_eligible_fails_the_refusal` (306, `statuses` 315; about 105 s in addition A), 589 `adversary_the_trigger_is_expected_on_the_branch_the_row_sends_it_down` (590). Tests that synthesize their own model stay: 334, 401, 482, 536, 609, 651, 797. `Synthesis` is not `Serialize`, so it cannot be cached on disk.

`crates/verify/ess-conformance/tests/adversary_ess0700a_identity_beside_row_set_2.rs` (1003 lines): `struct Variant` 432 (`label`, `text`, `rule`, `pairs`, `shape`), `run` 453, `permutations` 465, `equivalent` 482, `every_order_told_apart` 494-566 (the loop over `permutations(&variant.rule)` at 520). Seven callers, their `#[test]` fn lines: `adversary_a2_every_step_4_order_is_told_apart_across_declaration_orders` 575, `adversary_a2_count_row_set_orders_are_told_apart` 619, `adversary_a2_row_set_over_the_other_named_row_orders_are_told_apart` 655, `adversary_a2_row_set_over_a_missing_named_row_orders_are_told_apart` 685, `adversary_a2_separate_creator_orders_are_told_apart` 731, `adversary_a2_when_beside_a_named_refusal_orders_are_told_apart` 839, `adversary_a2_forall_beside_named_rows_witnesses_every_branch` 951 (calls `every_order_told_apart` at 952). Others, which do not call it: 785, 943, 965, 974.

`crates/edge/ess-xtask/tests/slow_lane.rs`: `MARKED` (t1a). `CHANGELOG.md`: the `### Changed` heading under `## [Unreleased]`.

## Tests first

Bounds: wall-time figures are the runner's own per-test figure (`target/nextest/ci/junit.xml`), on a warm build (a second run with no source change; the first run's compile is not the figure) at a 1-minute load of 60 or less (`cut -d' ' -f1 /proc/loadavg`, before and after, both in the report; above 60 the run decides nothing and is repeated); no test over 60 s, the test step within 120 s, a pool job ends after 10 minutes unless it names its time (`SLOT_TIMEOUT=30m`, 30 at most); no Python for anything that stays. Time every new test; name any over 5 s.

0. Measure first, for all three files: `SLOT_TIMEOUT=30m cargo nextest run --profile ci -p ess-conformance --test adversary_u22_pass1`, then `--test binding_trigger_row_predicate`, then `--test adversary_ess0700a_identity_beside_row_set_2` (one target per run; each build of an `ess-conformance` test is large). Read each test's time from `target/nextest/ci/junit.xml`. Decision rules: a test under 10 s stays as it is and is not touched, as long as its target ends within 30 s (the closing rule of step 6 applies to every target). A test of `adversary_u22_pass1` or of the 0700a file over 10 s is split (steps 1 and 3). For `binding_trigger_row_predicate`, step 2's rule.
1. `adversary_u22_pass1.rs`, new test `the_dropped_bound_scales_with_the_seeds_swept` (instant): `dropped_allowed(seeds: usize, refusals: usize) -> usize` equals `seeds * refusals / 4` rounded so that `dropped < allowed` is today's `dropped * 4 < seeds * refusals`; assert it for (24, 16), (48, 16) and (4, 16). Red today: compile error, `dropped_allowed` is not found.
2. `binding_trigger_row_predicate.rs`. The first measurement is `synthesize(&model(FIXTURE))` timed alone against `statuses(...)` alone for test 305 (print both with `eprintln!` in a scratch copy of the test, not committed). Rule:
   Apply the first branch that leaves the target within 30 s by the runner's figure (re-measure after each):
   - synth at most 10 s and `statuses` at most 20 s, and the target is within 30 s: the 105 s was host load; no change to this file, report the times;
   - synth over 10 s (or the target over 30 s with synth over 10 s): merge the tests that run `synthesize(&model(FIXTURE))` without mutating it (222, 252, 589, and the synthesis part of 271, 286 and 305) into one test `the_fixture_suite_keeps_its_flow_tier_and_trigger_expectations`, whose body calls the three existing bodies as helper functions over one `Synthesis`, and the three mutation checks over it. If the target is then within 30 s, stop here. Otherwise mark the mutation-run tests (286, 305 and 271) as slow twins with `#[ignore = "slow lane: runs the interpreted target on the fixture suite"]` and keep the three non-mutating checks in the merged step test. Add the red test `the_fixture_is_synthesized_by_one_test_only`: counts the occurrences of the text `synthesize(&model(` + `FIXTURE))` in `include_str!("binding_trigger_row_predicate.rs")` (write the needle as two `concat!`ed pieces so the test does not count itself) and asserts at most 1 for tests plus the helper. Red today: the count is 6;
   - `statuses` over 20 s, or the target still over 30 s: mark 271, 286 and 305 `#[ignore = "slow lane: runs the interpreted target on the fixture suite"]`, keep the others; then, if the target is still over 30 s, mark its slowest remaining test, and the next, until it is within.
3. `adversary_ess0700a_identity_beside_row_set_2.rs`, new test `the_adjacent_sweep_holds_n_orders_and_the_exhaustive_sweep_n_factorial` (instant): for a rule of five names, `orders(&rule, Sweep::Adjacent)` has 5 entries, the first is the rule itself and each other differs from it by one adjacent swap; `orders(&rule, Sweep::Exhaustive)` has 120 entries, all distinct. Red today: compile error, `Sweep` and `orders` are not found.

## Steps

1. Measure (step 0 and the first measurement of step 2 in `## Tests first`); write the numbers in the report.
2. `adversary_u22_pass1.rs`:
   - extract the loop of the two sweep tests into `fn sweep(seeds: std::ops::Range<u64>, chain: impl Fn(u64) -> Chain, label: &str)`, which runs the loop, asserts `failures.is_empty()` as today (message keeps `seed, steps, missing:` for sweep 1 and `N failing seeds:` for sweep 2: pass a closure or a bool for the message form), prints `refusals dropped as unreachable: {dropped} of {seeds.len() * 16}` and asserts `dropped < dropped_allowed(seeds.len(), 16)`;
   - add `fn dropped_allowed(seeds: usize, refusals: usize) -> usize` per step 1 of Tests first;
   - the two existing tests keep their names and call `sweep` over `0..24` and `100..148`, with `#[ignore = "slow lane: sweeps 24 seeded sixteen-refusal chains"]` and `#[ignore = "slow lane: sweeps 48 seeded plain sixteen-refusal chains"]`;
   - two step twins: `the_first_seeds_of_the_seeded_sixteen_refusal_chains_have_their_scenario_in_the_step` over `0..N1` and `the_first_seeds_of_the_plain_sixteen_refusal_chains_have_their_scenario_in_the_step` over `100..100+N2`, where `N1` and `N2` are the largest counts, each between 2 and 6, whose sum of times stays at most 20 s on the measured host. Pin the choice as `const STEP_SEEDS_SEEDED: u64` and `const STEP_SEEDS_PLAIN: u64`, each with a doc comment of one line naming the measured seconds per seed;
   - other tests of the file: apply the 10 s rule of step 0.
3. `adversary_ess0700a_identity_beside_row_set_2.rs`:
   - add `#[derive(Clone, Copy)] enum Sweep { Adjacent, Exhaustive }` and `fn orders(rule: &[&'static str], sweep: Sweep) -> Vec<Vec<&'static str>>`: `Adjacent` returns the rule and, for each `i` in `0..n-1`, the rule with items `i` and `i + 1` swapped (n orders); `Exhaustive` returns `permutations(rule)` unchanged;
   - `every_order_told_apart(variants, sweep)`: the loop at line 520 iterates `orders(&variant.rule, sweep)`;
   - for each caller the measurement of step 0 put over 10 s: move the `Variant` list into `fn variants_<short name>() -> Vec<Variant>`; the existing test keeps its name, becomes `Sweep::Exhaustive` and gets `#[ignore = "slow lane: tries every order of the rule"]`; add a step twin named `<existing name>_in_adjacent_swaps` that calls `every_order_told_apart(&variants_<short name>(), Sweep::Adjacent)`. No macro: the scanner of t1a reads functions, not expansions. A caller under 10 s calls `Sweep::Exhaustive` as it does today and is not split;
   - the adjacent sweep must still find a regression the exhaustive one finds in the common case: kill-check once by hand (flip one pair in `equivalent`) and record that an adjacent twin fails.
4. `binding_trigger_row_predicate.rs`: apply the branch step 2 of `## Tests first` chose; say which in the report with the two times.
5. `crates/edge/ess-xtask/tests/slow_lane.rs`: add one row per marked test under its file, sorted by path then name. Run `cargo test -p ess-xtask --locked --test slow_lane`.
6. Run each of the three targets in the step again (warm, load 60 or less); each ends within 30 s by the runner's figure. Closing rule: a target over 30 s gets its slowest remaining test marked slow (reason and `MARKED` row), one at a time, and is run again, until it is within 30 s. Then run `SLOT_TIMEOUT=30m task test-slow PACKAGES=ess-conformance` once; both pass; report each time and the lane's wall time.
7. Add the Changelog bullet below.
8. Do not edit `ci.yml`, `Taskfile.yml`, `AGENTS.md`.

## Owned files

The three test files named above (`crates/verify/ess-conformance/tests/adversary_u22_pass1.rs`, `crates/verify/ess-conformance/tests/binding_trigger_row_predicate.rs`, `crates/verify/ess-conformance/tests/adversary_ess0700a_identity_beside_row_set_2.rs`), the rows you add to `MARKED` in `crates/edge/ess-xtask/tests/slow_lane.rs`, and one bullet in `CHANGELOG.md`.

## Not yours

- t1a: the machinery. t1b: the CI job. t2: `synthesis_suite_bytes_table*.rs`, `enum_presence_guard.rs`. t3: `tutorial_page.rs`. t4: `generated_view_queries.rs`. t6, t7. Every other file of `ess-conformance/tests/`, including the same-named 0700a file in `ess-domain`.
- Shared: `slow_lane.rs` `MARKED` (t1a first; rows sorted, keep all at merge; t2, t3 and t4 also add rows to it), `CHANGELOG.md` (one bullet each).

## Gate

```console
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test adversary_u22_pass1
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test binding_trigger_row_predicate
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test adversary_ess0700a_identity_beside_row_set_2
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-conformance --locked --test adversary_u22_pass1 --test binding_trigger_row_predicate --test adversary_ess0700a_identity_beside_row_set_2 -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-xtask --locked --test slow_lane
SLOT_TIMEOUT=30m task test-slow PACKAGES=ess-conformance
slot task fmt-check
```

One `--test` per `ess-conformance` run: its full test build is the largest in the workspace. No synthesis code is touched, so `ess-cli` is not gated and none of the four byte-pin targets moves.

## Acceptance

1. Each of the three targets ends within 30 s under `cargo nextest run --profile ci -p ess-conformance --test <target>` (the `time` of the `testsuite` element in `target/nextest/ci/junit.xml`, second run on a warm build, 1-minute load of 60 or less, both loads in the report), and none of their tests runs over 60 s (times in the report). A target over 30 s after the planned changes is not accepted: step 6's closing rule marks its slowest remaining test until it is within.
2. `the_dropped_bound_scales_with_the_seeds_swept` and `the_adjacent_sweep_holds_n_orders_and_the_exhaustive_sweep_n_factorial` are green; the fixture-count test is green if step 2 chose the merge branch.
3. `cargo xtask slow-list` lists every marked test of these files; `task test-slow PACKAGES=ess-conformance` runs them and they pass with the original full ranges and orders.
4. The hand kill-check of step 3 is recorded: a changed `equivalent` fails the adjacent twin.
5. `cargo test -p ess-xtask --test slow_lane` is green with the new rows.

## Decisions

- Names (t1a, repeated): marker `#[ignore = "slow lane: <reason>"]`; `cargo xtask slow-list`; `task test-slow` with `PACKAGES`; profile `slow`; job `slow` (t1b); `MARKED`.
- The full-range tests keep their names and move to the lane; the step twins get new names. The mutation and byte-pin documents that name the old tests stay true.
- Explicit twin functions, never a macro: the scanner reads functions.
- `Adjacent` is the rule plus its n-1 adjacent transpositions: it exercises every pair of neighbours once, which the exhaustive sweep implies, at n runs instead of n!.
- The step seeds are chosen by measurement (2 to 6 seeds, at most 20 s), because the host load makes any fixed guess wrong.
- `Synthesis` is not `Serialize`, so no disk cache; for `binding_trigger_row_predicate` the "made once" form is one synthesis per process: merge the tests that read the same synthesis into one test (under nextest each test is its own process, so an `OnceLock` would not help).
- The 105 s of `a_target_that_retries_not_eligible_fails_the_refusal` may have been load: the first measurement decides, and the unit changes nothing when it was.
- No new dependency; no generated file; no pin moves.

## Changelog

`### Changed`:

- CI: the seeded and order sweeps of `adversary_u22_pass1` and `adversary_ess0700a_identity_beside_row_set_2` run a reduced set in the test step and the full set in the `slow` lane.
