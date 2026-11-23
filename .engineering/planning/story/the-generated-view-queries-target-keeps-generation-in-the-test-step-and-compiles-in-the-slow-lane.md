---
format: aep.planning-md/3
id: story:the-generated-view-queries-target-keeps-generation-in-the-test-step-and-compiles-in-the-slow-lane
kind: story
status: active
title: The generated-view-queries target keeps generation in the test step and compiles in the slow lane
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
  path: crates/generate/ess-synth/tests/generated_view_queries.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:02:53Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:02:54Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

`generated_view_queries` (about 257 s as a target, addition A) fits the step.

- The nine tests that build generated Rust or Go with `cargo` or `go` and run the result carry `#[ignore = "slow lane: ..."]` and run in `task test-slow` and the `slow` job.
- The tests that only generate and read the plan stay in the step; any of them that the first-step measurement puts over 30 s moves to the lane too.
- The target ends within 30 s in the step by the runner's own figure, on a warm build at a 1-minute load of 60 or less, and no test of it runs over 60 s.

## Why

Parent: `story:the-test-step-meets-the-test-budget`, step 5 (`slow` lane) and addition A, target `generated_view_queries`.
Rule: "Tests stay fast", an expensive step runs inside each test: here a `cargo` build per test (line 279), `go` (1095) and a built binary (1170).
Consumer gap: none.

## Where

Read at `80794bbd698` (unchanged since `bf128f9146`). Re-read a line before editing it.

`crates/generate/ess-synth/tests/generated_view_queries.rs` (2060 lines). Helpers: `cargo` 278 (spawns `$CARGO ... --offline` with `RUSTFLAGS=-D warnings`), the Go probe 1095 and runner 1101, the server spawn 1170, `ess_conformance::synthesize` 414.

Tests that stay (fn line): `every_fully_declared_view_query_is_generated_over_a_listing_storage_port` 74, `a_view_construct_the_generated_query_does_not_reproduce_keeps_the_query_owed` 130, `a_paged_view_is_an_obligation_in_the_plan` 195, `a_view_with_a_default_row_for_an_unknown_identity_is_an_obligation_in_the_plan` 227, `a_measure_condition_the_generated_query_does_not_reproduce_keeps_the_query_owed` 648, `every_query_of_the_workbench_views_is_generated_for_rust_and_go` 927, `adversary_an_optional_parameter_any_comparison_reads_keeps_the_query_owed` 1913.

Tests that get the marker (fn line; its `#[test]` is the line before, with nothing between):
- 334 `the_generated_queries_build_with_warnings_denied_and_pass_their_own_suite`
- 349 `the_single_crate_layout_passes_the_same_suite_with_its_server_feature`
- 499 `the_go_queries_build_and_pass_their_own_suite`
- 611 `conditional_measure_all_six_functions_execute_in_the_generated_rust_queries`
- 632 `conditional_measure_all_six_functions_execute_in_the_generated_go_queries`
- 982 `the_rust_servers_of_the_workbench_views_answer_their_parameters_and_orders`
- 1031 `the_go_servers_of_the_workbench_views_answer_their_parameters_and_orders`
- 1981 `adversary_rust_typed_parameters_and_orders_answer_as_the_interpreted_target`
- 2018 `adversary_go_typed_parameters_and_orders_answer_as_the_interpreted_target`

`crates/edge/ess-xtask/tests/no_test_builds.rs`: `ALLOWED` entry at line 67, `"crates/generate/ess-synth/tests/generated_view_queries.rs"`. This file is run as a test, not edited, and so it is not in this unit's `.scope`.

`crates/edge/ess-xtask/tests/slow_lane.rs`: `MARKED` (t1a). `CHANGELOG.md`: the `### Changed` heading under `## [Unreleased]`.

## Tests first

Bounds: wall-time figures are the runner's own per-test figure (`target/nextest/ci/junit.xml`), on a warm build at a 1-minute load of 60 or less (`cut -d' ' -f1 /proc/loadavg`, before and after; above 60 the run decides nothing and is repeated); no test over 60 s, the test step within 120 s, a pool job ends after 10 minutes unless it names its time (`SLOT_TIMEOUT=30m`, 30 at most); no Python for anything that stays. Time every test; any over 5 s is named in the report.

0. Measure first. `SLOT_TIMEOUT=30m cargo nextest run --profile ci -p ess-synth --test generated_view_queries` on the unchanged tree, then read every test's time from `target/nextest/ci/junit.xml`. Decision rules:
   - a test of the nine over 5 s is marked (the default, and expected);
   - a test of the seven under 5 s stays and needs no mention;
   - a test of the seven over 5 s is named in the report with its time; over 30 s it is marked too, with the reason `slow lane: <what it spends the time on>`, and its row is added to `MARKED` (for example `every_query_of_the_workbench_views_is_generated_for_rust_and_go`, line 927, which calls `synthesize` for the workbench views);
   - a test of the nine under 5 s is marked anyway: it spawns `cargo` or `go`, which the lane owns.
   - Report the sum of the seven. If it is over 30 s after the rule, mark the slowest until it is not.
1. `crates/edge/ess-xtask/tests/slow_lane.rs`: add the nine rows (or ten or more, per step 0) to `MARKED` under `crates/generate/ess-synth/tests/generated_view_queries.rs`, names sorted. `every_marker_in_the_tree_is_a_row_of_the_table_and_every_row_is_marked` is red today with `in the table, not marked` for each row (the markers do not exist yet).

No new test in `generated_view_queries.rs`: nothing in the file changes except attribute lines.

## Steps

1. Measure (step 0).
2. Write the `MARKED` rows and run `cargo test -p ess-xtask --locked --test slow_lane`: red.
3. Insert `#[ignore = "slow lane: <reason>"]` between each `#[test]` and its `fn` for the marked tests. Reasons, one short clause each: `builds the generated Rust queries with cargo and runs their suite` (334), `builds the single-crate layout with cargo and runs its suite` (349), `builds the generated Go queries and runs their suite` (499), `builds the generated Rust queries and runs the six measure functions` (611), `builds the generated Go queries and runs the six measure functions` (632), `builds and runs the Rust workbench servers` (982), `builds and runs the Go workbench servers` (1031), `builds the generated Rust queries for typed parameters and compares with the interpreted target` (1981), `builds the generated Go queries for typed parameters and compares with the interpreted target` (2018). The line numbers shift by one per insertion: apply from the bottom of the file upward.
4. Leave `no_test_builds.rs` as it is: the file stays in `ALLOWED` because the cargo and go spawns remain, now only inside slow tests. Run `cargo test -p ess-xtask --locked --test no_test_builds` to prove it.
5. `cargo test -p ess-xtask --locked --test slow_lane` is green. Run `SLOT_TIMEOUT=30m task test-slow PACKAGES=ess-synth` once; every marked test must pass in the lane (they did before: only their lane moved). Report the time of each and the lane's wall time for `ess-synth`.
6. Time the step: `SLOT_TIMEOUT=30m cargo nextest run --profile ci -p ess-synth --test generated_view_queries` ends within 30 s; report it and the time of the slowest remaining test.
7. Add the Changelog bullet below.
8. No change to `ci.yml`, `Taskfile.yml`, `nextest.toml`, `AGENTS.md` (t1a, t1b and t7 own them).

## Owned files

`crates/generate/ess-synth/tests/generated_view_queries.rs` (attribute lines only), the nine or more rows you add to `MARKED` in `crates/edge/ess-xtask/tests/slow_lane.rs`, and one bullet in `CHANGELOG.md`.

## Not yours

- t1a: `slow_lane.rs` machinery, `Taskfile.yml`, `nextest.toml`. t1b: `ci.yml`, the `slow` job. t2, t3, t5, t6, t7: their files. `no_test_builds.rs`: unchanged. The other 51 `ess-synth` test files that spawn node, tsc, go, cargo, rustc or npm (a follow-up per crate, named in the map comments).
- Shared: `crates/edge/ess-xtask/tests/slow_lane.rs` (`MARKED`: t1a lands first; t2, t3, t5 add rows beside yours; keep every row at merge, rows sorted by path then name), `CHANGELOG.md` (keep every bullet).

## Gate

```console
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo nextest run --profile ci -p ess-synth --test generated_view_queries
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-synth --locked --test generated_view_queries -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-xtask --locked --test slow_lane --test no_test_builds
SLOT_TIMEOUT=30m task test-slow PACKAGES=ess-synth
slot task fmt-check
```

A generator output moves nothing here, so the "also `ess-synth`'s whole test set" row does not apply (no committed tree under `generated/` changes). `ess-synth` tests run as the named target only.

## Acceptance

1. `cargo xtask slow-list` lists the nine tests under `ess-synth generated_view_queries` (plus any the step-0 rule added).
2. `cargo nextest run --profile ci -p ess-synth --test generated_view_queries` ends within 30 s by the runner's figure (the `time` attribute of the `testsuite` element in `target/nextest/ci/junit.xml`), on a warm build (a second run with no source change; the first run's compile is not the figure) at a 1-minute load of 60 or less (both loads in the report). Its remaining tests all pass, the nine print as ignored, and each remaining test over 5 s is named in the report with its time. If the second run is over 30 s, the slowest remaining test is marked (step 0 rule) until it is not; the target is never left over 30 s.
3. `task test-slow PACKAGES=ess-synth` runs the nine and they pass.
4. `cargo test -p ess-xtask --test slow_lane` and `--test no_test_builds` are green; `git diff --stat` for the repository shows `generated_view_queries.rs` changed only by inserted attribute lines.

## Decisions

- Names (t1a, repeated): marker `#[ignore = "slow lane: <reason>"]`; `cargo xtask slow-list`; `task test-slow` with `PACKAGES`; profile `slow`; job `slow` (t1b); `MARKED`.
- D1 (a)/(b) not reopened. For this file the "made once" rule is met by moving the whole compile to the slow lane: the lane builds each generated crate a few times at most and takes no part of the 120 s step. The cargo spawns stay inside the tests, so the file stays in `no_test_builds.rs` `ALLOWED`.
- Marker only; no helper, no refactor of the file: a refactor would move lines under other units' feet and is not needed to meet the budget.
- Per-test marking, not whole-file `#![cfg(...)]`: the seven generation tests are the fast coverage that the step keeps.

## Open for the operator

Addition A (acceptance) asks that `generated_view_queries` "builds with cargo once, in a Taskfile task, and the test reads its path". The nine tests build different generated crates (the Rust layout, the single-crate layout, Go, the workbench servers, typed-parameter variants), so one Taskfile build does not serve them all; a port is a harness per variant, as `task integer-spellings-harness` does for one.
- A (recommended): mark the nine slow and keep the in-test cargo builds. The step meets the budget, the 60 s bound holds in the lane (each test under the profile's 20 minutes), and the file keeps its `ALLOWED` entry. It is the form D1 (b) gives for a case not worth committing.
- B: port the nine to Taskfile harness tasks, as `task json-body-harnesses` does, with `no_test_builds.rs` covering the file. A larger follow-up (several harness tasks, new environment variables), not in this wave.

This unit implements A. Choosing B later changes only the lane's content, not t1a's mechanism.

## Changelog

`### Changed`:

- CI: the nine tests of `generated_view_queries` that build generated Rust or Go with `cargo` or `go` run in the `slow` lane; the generation and plan tests of the file stay in the test step.
