---
format: aep.planning-md/3
id: story:eight-ess-xtask-tests-run-68-102-s-on-the-build-node-over-the-60-s-bound-seven-support-tests-adversary-cases
kind: story
status: active
title: 'Eight ess-xtask tests run 68-102 s on the build node, over the 60 s bound: seven `support::tests::adversary_*` cases and `scratch_leaks::tests::the_representative_suites_leave_nothing_under_tmpdir` each compile ess-cli through `cargo run`, so the CLI is built once before the tests'
summary: A fresh target directory makes eight ess-xtask tests wait 68-102 s each, because each test process runs `cargo run -p ess-cli` and the first one compiles the CLI while the others wait on the build lock; warm they take 6-11 s
tags:
- issues-2026-11-24
- test-budget
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T04:26:36Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T04:26:36Z", actor: "human:timo", revision: 4}
---
## Outcome

`task test-xtask` meets the rule `Tests stay fast` in a fresh target directory: no `ess-xtask` test
runs longer than 60 s, the seven CLI-driven `support::tests::adversary_*` tests and the leak check
run under 30 s, and the test step ends within 120 s. The `ess` binary is built once in a step
before the tests, and the tests run that binary.

## Need

**Reading.** Measured on the remote build node with
`CARGO_INCREMENTAL=0 slot --timeout 30m cargo nextest run -p ess-xtask --locked --profile ci
--no-fail-fast` (555 tests, 3 skipped, tree at head `b2cdf564b`, 2026-11-24). Every other test of
the package finishes in 22 s or less in every run.

The pool's owner reported the first line: `support::tests::adversary_all_real_material_rows_refuse_cell_removal_duplicate_and_order_drift`
in `ess-xtask::bin/ess-xtask`, PASS 67.802 s, test 548 of 548, pool job 1286, ended 04:46:33 CET on
2026-11-24, node 1-minute load 20.8 to 25.0 on 64 CPUs (the node was not starved for CPU).

| test (in `ess-xtask::bin/ess-xtask`) | run 1, fresh `target/` | run 3, fresh `target/budget-cold` | run 2, warm `target/` | earlier logs of this session (also on the node) |
|---|---|---|---|---|
| `support::tests::adversary_all_real_material_rows_refuse_cell_removal_duplicate_and_order_drift` | 99.3 s | 71.6 s | 8.4 s | 76.6 s cold, 5.8 s warm |
| `scratch_leaks::tests::the_representative_suites_leave_nothing_under_tmpdir` | 102.0 s | 74.6 s | 11.3 s | 79.0 s cold, 7.1 s warm |
| `support::tests::adversary_actual_explicit_site_and_combined_maps_keep_distinct_roots` | 94.4 s | 69.2 s | under 5 s | none |
| `support::tests::adversary_actual_cli_refusal_is_not_a_successful_support_observation` | 93.8 s | 68.9 s | under 5 s | none |
| `support::tests::adversary_front_page_override_replaces_adjacent_readme_for_a_specification_file` | 93.8 s | 68.5 s | under 5 s | none |
| `support::tests::adversary_actual_help_missing_target_metadata_cannot_borrow_neighbor_values` | 93.2 s | 68.4 s | under 5 s | none |
| `support::tests::adversary_adjacent_readme_is_selected_without_authored_flags` | 93.4 s | 68.3 s | under 5 s | none |
| `support::tests::adversary_real_docs_ir_marker_is_nested_json_not_visible_marker_text` | 92.4 s | 67.7 s | under 5 s | none |

- Run 1 and run 2 ran in the tree's own `target/` on the node, run 1 first (presumably the node had no build of
  `ess-cli` for that tree yet); the whole nextest step took 104.3 s in run 1 and 23.6 s in run 2. Run 3
  used `CARGO_TARGET_DIR=target/budget-cold` and took 76.8 s. Pool job number printed for run 3: 2130
  (waited 12 s, ran 148 s including the test build). Runs 1 and 2 printed none.
- Load: the node's own load was not readable from this workstation; only the pool's owner's figure
  above is the node's. This workstation's 1-minute load at the start of the runs was 16.8, 10.1 and
  12.9 (not the node's).
- The last column is from `wave-1124/s0/xt.log` (warm) and `wave-1124/s0/gate2.log` (cold) of
  the 2026-11-24 session; both ran on the node ("synced to build-1d"), so they are earlier node runs of the same
  shape; no log of this session ran on this workstation.
- Nextest's `ci` profile (`.config/nextest.toml`) has `slow-timeout = { period = "60s" }` and no
  `terminate-after`, and no override or exemption list: these tests are reported SLOW at 60 s and
  are not killed today. The `default` profile's overrides name only browser binaries and one ratio
  test.

**Cause.** The seven `adversary_*` tests call `crate::cli_output`, `crate::projection_artifacts` or
`render`, and `cli_output` (`crates/edge/ess-xtask/src/main.rs:811`) runs
`cargo run --quiet --locked --offline --package ess-cli --bin ess -- <args>`. `task test-xtask`
builds only the `ess-xtask` test binaries (`cargo test --no-run`), so `ess-cli` and its dependency
graph are not built when the tests start. Nextest runs each test in its own process, and every
process starts that same `cargo run`: the first compiles `ess-cli` (about 70-90 s of the wait on the
node) and the other seven block on cargo's build-directory lock for the same time, then each runs
its own CLI calls (`render` makes about 25). `scratch_leaks::tests::the_representative_suites_leave_nothing_under_tmpdir`
runs the unit tests `support::tests::adversary_actual_help_` again in a child process, plus five
integration binaries one after another, so it waits for the same build and then adds the sum of
those suites (6-10 s warm). This is the organization's known mistake: an expensive step (a crate
graph build) runs inside each test instead of once per run. `no_test_builds.rs` already refuses a
test file under `tests/` that spawns cargo ("Build what the test needs once in a Taskfile step
before the test binary runs, and hand the test the artifact's path"); the unit tests in `src/` are
outside its scan.

## Acceptance

- In a fresh target directory on the node, `cargo nextest run -p ess-xtask --locked --profile ci
  --no-fail-fast` ends with no test over 60 s; the seven `adversary_*` tests and the leak check each
  finish under 30 s; the `Summary` line reads under 120 s including the build step.
- `task test-xtask` builds `ess` once (`cargo build -p ess-cli --bin ess --locked`) in its build step,
  before `nextest`, with the same `XTASK_ENV`.
- Under test, `cli_output` runs the built binary found beside the test executable (`../ess` of
  `std::env::current_exe()`'s directory), not `cargo run`. It reads no environment variable of its
  own, because the node does not receive one. When the binary is missing the test fails naming
  `task test-xtask` as the step that builds it. The production path of `cargo xtask` commands
  (`main.rs:782`) keeps its behaviour.
- A Rust test in `crates/edge/ess-xtask/tests/no_test_builds.rs` fails when a `#[cfg(test)]` module
  under `crates/edge/ess-xtask/src` reaches a function that spawns `cargo`; putting the old
  `cargo run` back into the test path turns it red.
- `scratch_leaks` still lists and runs the same unit and integration suites; the leak check's
  `UNIT_SUITES` and `INTEGRATION_SUITES` are unchanged.
- The report of the change gives the nextest times of the eight tests on the node, cold and warm.

## Decisions

- **The CLI is built in a Taskfile step, not by a fixture in the test.** Cargo's lock serialises
  builds in one target directory, and a per-process `OnceLock` is not once per run under nextest;
  the build step is the form the organization's rule names ("a setup step before the tests").
  A fixture on disk keyed by the digest of its sources under `target/` is the alternative and is
  not needed while the build step runs on every `task test-xtask`.
- **The binary path comes from the test executable's location**, not from an environment variable,
  because a variable set by the caller does not reach a test that runs on the node.
- **Not decided here:** `terminate-after` for the `ci` profile, which makes the runner kill a test at
  60 s. That belongs to `story:the-test-step-meets-the-test-budget`; this story removes the eight
  tests that kill would fail, and must land first.

## Out of scope

- `ess-xtask::no_test_builds no_test_spawns_cargo_outside_the_shrinking_allowlist` (12-22 s on the
  node, 13.3 s in the earlier warm log): under the bound, reads the tree once per run.
- Every other crate's tests and the `ci` profile's `terminate-after`.

## Scope

- inferred: `crates/edge/ess-xtask/src/main.rs` (`cli_output`)
- inferred: `crates/edge/ess-xtask/src/support.rs`, `crates/edge/ess-xtask/src/scratch_leaks.rs`
  (only if a call site needs the binary path)
- inferred: `crates/edge/ess-xtask/tests/no_test_builds.rs` (the new scan of `src/` test modules)
- inferred: `Taskfile.yml` (`test-xtask`: the build step)
