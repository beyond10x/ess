---
format: aep.planning-md/3
id: story:tests-that-run-over-60-s-on-main-without-a-branch-of-the-week-run-within-60-s-or-move-to-the-slow-lane
kind: story
status: draft
title: Tests over 60 s on main that no branch made slower, in six ess-conformance targets and ess-cli explore_concurrent, run within 60 s or move to the slow lane
summary: 'A test-budget defect: tests measured over 60 s on main''s own tree in the gate logs of the week; each runs within 60 s in the test step or moves to the slow lane with its reason'
tags:
- defect
- test-budget
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
- informed_by: story:three-adversary-targets-run-a-reduced-sweep-in-the-test-step-and-the-full-sweep-in-the-slow-lane
revision: 2
---
## Summary

Tests that run over the 60 s bound on `main` with no branch of the week having made them slower:
each was measured over the bound on a branch and again over the bound on `main`'s own tree (the
"here / main" pairs below, the branch's time first). A test that is slow on `main` is a defect of
the test step under `AGENTS.md` § Tests stay fast and is fixed on its own, not waited out. The
times follow the machine's load (a warm rerun of seven of them at low load ended under 48 s each),
so the story's first step is the measure on the build node, not these figures.

## Evidence (gate logs of the week, read; no measurement by the author of this draft)

`ess-conformance`, branch time / `main` time, in seconds:

- `adversary_u22_pass3`, `every_reachable_branch_of_seeded_chains_over_wide_shared_inputs_has_its_scenario`:
  244.4 / 271.9 and 346.4 / 225.5 in two runs.
- `adversary_u22_pass2`, `every_reachable_branch_of_seeded_chains_over_optional_integer_and_compared_enums_has_its_scenario`:
  161.7 / 167.3, 153.7 / 140.2 and 251.5 / 140.2.
- `adversary_u22_pass3`, the `guards_sharing_an_enum` case: 122.3 / 61.2 in one run, 99.3 / 120.2 in
  another; `sixteen_variants`: 75.6 / 79.8.
- `aggregate_group_selection`, `the_wasm_runner_agrees_with_the_native_runner`: 131.6 / 72.8 and 105.8 / 72.8.
- `adversary_287_pass1`, `wasm_executes_the_same_shared_row_and_isolation_fixtures`: 74.0 / 71.4 and 97.8 / 71.4.
- `nested_response_observations`, `actual_foreign_readers_refuse_malformed_authority_before_any_target_callback`:
  78.1 / 81.9 and 106.0 / 60.9.
- `adversary_map_witness_pass1`, `adv196`: 63.3 / 60.7.

`ess-cli`, target `explore_concurrent` (29 to 37 tests): the slowest test 126.1 s at load 0.65 on
`main`'s tree and 156.1 to 359.1 s elsewhere; a rerun at load 16 had 21 of 30 tests over 60 s, the
slowest 170.2 s.

## Acceptance

1. Each test named above, and each other test of those seven targets that the measure on the build
   node finds over 60 s, runs within 60 s in the test step, or moves to the `slow` lane
   (`#[ignore = "slow lane: <reason>"]`) with its reason written beside it.
2. The measure is recorded with the node's load: the times of all tests of the seven targets, run
   by `cargo nextest run --profile ci` one target at a time, before and after.
3. The runner kills a test at 60 s in the test step (`story:the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests`
   holds this) and the step prints its ten slowest tests; after this story no test of these targets
   is on that list over 60 s.
4. A test moved to the `slow` lane keeps its assertions: the slow lane runs the same test body.

## Fit review

1. **Class.** A test-budget defect, not a feature: the organization's rule is 60 s per test.
2. **Not covered.** No story in the store names these tests: `story:three-adversary-targets-run-a-reduced-sweep-in-the-test-step-and-the-full-sweep-in-the-slow-lane`
   names `adversary_u22_pass1` and two other targets; the slow-lane story names `adv_e_u6_byte_identity_probe`.
   `adversary_u22_pass2` and `adversary_u22_pass3` are not in any story.
3. **Borders.** The parent `story:the-test-step-meets-the-test-budget`; the two stories above.
   The same fix shape applies (a reduced sweep in the step, the full sweep in the slow lane).

## Decisions

- TB-D1, default A: for a test that is slow because of a seeded sweep, A runs a few seeds in the
  step and the full range in the slow lane with the same assertions (as the border story does); B
  moves the whole test to the slow lane. Recommended A. Not architecture and not a security
  boundary; taken on default if unanswered.

## Scope

- `crates/verify/ess-conformance/tests/` (`adversary_u22_pass2.rs`, `adversary_u22_pass3.rs`,
  `aggregate_group_selection.rs`, `adversary_287_pass1.rs`, `nested_response_observations.rs`,
  `adversary_map_witness_pass1.rs`) and `crates/edge/ess-cli/tests/explore_concurrent.rs`; the
  slow-lane list (`cargo xtask slow-list`) and its pin test.

## Added evidence (2026-11-25)

- The test step of `ess-ui-test` ran 121 s in the narrowed gate of the pull request 830 (head
  `f93dcf8514`), over the 120 s step budget; the time includes compiling. It is a step over the
  budget, not one test over 60 s: the step's own build and its tests both count.
- One slow byte lane of the gate of the N89 fix (branch
  `fix/n89-a-setup-run-rebuilds-its-creating-scenario-once`, the lane that compares suites at 1, 2
  and 7 jobs) ran 156 s in the first gate run and 119 s in the re-run, a single test over the 60 s
  bound in both. Its test name is not recorded in the gate log read for this entry; the lane's
  first step is to name it.
