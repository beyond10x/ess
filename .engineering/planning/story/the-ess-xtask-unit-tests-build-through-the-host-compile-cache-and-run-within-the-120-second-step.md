---
format: aep.planning-md/3
id: story:the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step
kind: story
status: draft
title: The ess-xtask unit tests build through the host compile cache and the pool's jobserver and run their `docs::` tests within the 120 second step
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/main.rs
- confidence: cited
  path: crates/edge/ess-xtask/tests/ci_lanes.rs
revision: 6
---
## Outcome

`task test-xtask` no longer builds `ess-xtask` and its test targets outside the host's shared compile cache and job pool, and its run phase is separated from its compile phase in the log.

- The command has no `RUSTC_WRAPPER=`, `RUSTC_WORKSPACE_WRAPPER=`, `CARGO_BUILD_RUSTC_WRAPPER=`, `CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER=` and no `CARGO_BUILD_JOBS=2`. `consumer-check` has the same lines removed.
- `test-xtask` runs `cargo test --package ess-xtask --locked --no-run` and then `cargo test --package ess-xtask --locked`, so a log or a timing run shows the compile and the test time apart.
- The 28 `docs::` tests of the `ess-xtask` binary (about 217 s in addition A) run within 120 s, and the report says whether the 217 s were compile or run time.
- A test in `ci_lanes.rs` fails when a `test*` task or `consumer-check` empties the wrapper or sets build jobs.

## Why

Parent: `story:the-test-step-meets-the-test-budget`, step 4 and addition A (the `ess-xtask` unit binary, `docs::`, 216.73 s, "not read: `docs.rs` has 64 functions and no process spawn, so the time is in the tests that call into it or in the link").
Rules: "Tests stay fast" (120 s step; find the cost before touching anything); "One compile cache" (no build sets the wrapper to the empty string, no `-j` or `CARGO_BUILD_JOBS`).
Consumer gap: none.

## Where

Read at `bf128f9146` (unchanged in `ff7f8e74fb`). Re-read a line before editing it.

- `Taskfile.yml` lines 404-418, `test-xtask`: `desc`, a three-line comment (406-408, keep it), then one command (409-417) that starts `env -u CARGO_ENCODED_RUSTFLAGS RUSTUP_TOOLCHAIN=1.98.1 RUSTFLAGS='-C link-arg=-fuse-ld=lld'`, then `RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER=` (413), `CARGO_BUILD_RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER=` (414), `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0` (415), the `CARGO_PROFILE_*` flags (416-417), and `cargo test --package ess-xtask --locked` (418). `RUSTUP_TOOLCHAIN=1.98.1` equals `rust-toolchain.toml`; keep it.
- `Taskfile.yml` lines 597-611, `consumer-check` (parked lane, `CONSUMER_CHECKS=true`): its command at 602-611 carries the same lines (606-608: the wrapper assignments and `CARGO_BUILD_JOBS=2 CARGO_NET_OFFLINE=true CARGO_INCREMENTAL=0`). Line 599: "Command assignments override inherited variables, unlike task-level env defaults": so the shared prefix is a task `vars:` entry, not a task `env:`.
- `Taskfile.yml` `test` (77-99, the `test-xtask` call is a step: `ci_lanes.rs:309-322` pins that `test` calls `test-xtask` and does not restate it).
- `crates/edge/ess-xtask/src/docs.rs`: `#[cfg(test)] mod tests` 1207-1693 (28 `#[test]` functions, none spawns a process; they read the workspace through `crate::workspace_root()`: for example `a_catalogue_release_the_page_disagrees_with_is_refused` 1239, `every_family_a_published_page_names_is_tracked` 1394, `the_committed_readme_installs_the_newest_release` 1487, `the_committed_release_notes_do_not_trail_the_newest_release` 1640). `ess-xtask` has no library: its unit tests are in the binary `ess-xtask` (`main.rs` `mod docs;` line 7).
- `crates/edge/ess-xtask/tests/ci_lanes.rs`: `shell_commands` 236 (the commands of a task), `every_step_of_task_check_runs_in_some_pull_request_lane` 271, line 309-322 (the `test-xtask` pins). Append the new test at the end of the file.
- Findings, not yours (see `## Not yours`): `Taskfile.yml` 184, 208, 228, 255, 258, 278, 281, 307 (`env -u RUSTC_WRAPPER` in the harness tasks), `.github/workflows/ci.yml:836` (`CARGO_BUILD_JOBS: "2"`, macOS witness job).
- `CHANGELOG.md` `## [Unreleased]` line 3, `### Changed` line 5.

## Tests first

Bounds: no test over 60 s, the test step within 120 s, a pool job ends after 10 minutes unless it names its time (`SLOT_TIMEOUT=30m`, 30 at most). Time every test; any over 5 s is named in the report. A measurement script is a scratch file in the session scratch directory, not committed, in POSIX sh, and starts without the `slot` prefix (it measures and waits); the `cargo` calls inside take a slot through the shim.

0. Measure first, three numbers. Use `CARGO_TARGET_DIR=<scratch>/xtask-target` for the first two so that neither run reuses the other's artifacts, and time each with `/usr/bin/time -v` or the shell's `time`:
   - (a) compile as today: `cargo test --package ess-xtask --locked --no-run` under today's `env` prefix from `Taskfile.yml` lines 410-417, including `RUSTC_WRAPPER=` and `CARGO_BUILD_JOBS=2`;
   - (b) compile with the prefix of step 2 below (no wrapper assignments, no jobs);
   - (c) run only: in the target of (b), `cargo nextest run --package ess-xtask --bin ess-xtask docs::` for per-test times from `target/nextest/default/...` or the printed times (nextest is only the measuring tool here, `task test-xtask` stays on `cargo test`), and `cargo test --package ess-xtask --locked` wall time.
   Decision rules: if (a) minus (b) is over 60 s, the wrapper and jobs lines were the cost: steps 1 to 3 are the fix and nothing else is needed for `docs::`. If (c) shows a `docs::` test over 5 s, it is named, and step 4 applies. If (c) total (run, not compile) is over 120 s with every test under 5 s, the number of tests is the cost: report it as a finding and stop for that part (no test is changed on a guess).
1. `crates/edge/ess-xtask/tests/ci_lanes.rs`, new test `no_test_task_empties_the_compile_wrapper_or_sets_build_jobs`: reads `Taskfile.yml`; for every task whose name starts with `test` and for `consumer-check`, no command contains a word starting `RUSTC_WRAPPER=`, `RUSTC_WORKSPACE_WRAPPER=`, `CARGO_BUILD_RUSTC_WRAPPER=` or `CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER=` (with or without a value), `CARGO_BUILD_JOBS`, `SCCACHE_`, or the words `-j` and `--jobs`; the commands' task-level `vars` values are checked the same way. Red today: it names `test-xtask` line 413 and `consumer-check` line 606. The harness tasks with `env -u RUSTC_WRAPPER` (the word is `-u`, not an assignment) are not named by it.
2. New test `test_xtask_builds_before_it_runs`: `shell_commands(&taskfile, "test-xtask")` has two commands, the first ending `--no-run`, the second not containing it, and both carry `--package ess-xtask --locked`, `RUSTUP_TOOLCHAIN=1.98.1`, `-fuse-ld=lld`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0` (after variable expansion of the task's `vars`, the check reads the task's `vars` and its commands together). Red today: one command only.
3. Run both red.

## Steps

1. Measure (step 0 above), keep the three numbers.
2. `Taskfile.yml` `test-xtask`: add `vars:` with `XTASK_ENV` holding the shared prefix, as a folded string: `env -u CARGO_ENCODED_RUSTFLAGS RUSTUP_TOOLCHAIN=1.98.1 RUSTFLAGS='-C link-arg=-fuse-ld=lld' CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_INCREMENTAL=false CARGO_PROFILE_TEST_INCREMENTAL=false`. `cmds`: `{{.XTASK_ENV}} cargo test --package ess-xtask --locked --no-run` then `{{.XTASK_ENV}} cargo test --package ess-xtask --locked`. Keep the existing comment lines above the commands. No new comment. The four wrapper assignments and `CARGO_BUILD_JOBS=2` are gone.
3. `consumer-check`: delete the wrapper assignments and `CARGO_BUILD_JOBS=2` from its command (606-608), leave `CARGO_NET_OFFLINE=true CARGO_INCREMENTAL=0` and everything else.
4. Only if step 0 named a `docs::` test over 5 s: find the repeated expensive call in it (for example a walk of `website/docs` or a read of `CHANGELOG.md` made by several tests) by timing it with a scratch `eprintln!` of `Instant::elapsed`; share the product in one `fn documents_of_the_tree() -> &'static ...` with a `std::sync::OnceLock` inside `#[cfg(test)] mod tests` of `docs.rs`. A `OnceLock` is valid here and only here: `cargo test` runs all the tests of the binary in one process (not under nextest). Re-time and report both numbers. If nothing is over 5 s, skip this step.
5. Run the two new tests green, then `SLOT_TIMEOUT=30m task test-xtask` once and report the compile time and the run time from the two commands' output (the second command's `Finished` line and the `test result` lines).
6. Report as findings, not changes: `Taskfile.yml` 184-307 harness tasks (`env -u RUSTC_WRAPPER`, a build of a fixture under a clean environment; the rule allows an emptied wrapper only for a test's fixture child process), `ci.yml:836` (`CARGO_BUILD_JOBS: "2"` in the macOS output-ownership witnesses; a hosted runner has no pool and no sccache, so the setting is the forge's, but the rule says no `-j` or `CARGO_BUILD_JOBS` in a gate). There is no slot-repository story naming these exceptions; propose one in the report.
7. No `AGENTS.md` change.

## Owned files

`Taskfile.yml` tasks `test-xtask` and `consumer-check` (lines 404-418 and 597-611 only), `crates/edge/ess-xtask/src/docs.rs` `mod tests` (step 4 only, if it applies), and the two new functions appended to `ci_lanes.rs`.

## Not yours

- t1: `test-slow`, `test-slowest`, `check` and `test-slow-probes` lines of `Taskfile.yml`, `ci.yml`, `nextest.toml`, `slow_lane.rs`, `test_budget.rs`. t7: the step budget wrapper around `test-xtask` (a `timeout` around its commands, its ceiling in `AGENTS.md` § Gate) and the `ci_lanes.rs` ratchet. t2 to t5: test files.
- The harness tasks 184-307, `fuzz-check`, `test-doc`, `test-feature-off*` and `ci.yml:836`: findings only.
- Shared: `Taskfile.yml` (different tasks; t1 and t7 edit others; the merge is by task), `ci_lanes.rs` (append at the end; keep both sides at merge), `CHANGELOG.md` (one bullet each). t7 lands after this unit and wraps the two commands you wrote: it takes them as they are.

## Gate

```console
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-xtask --locked --test ci_lanes
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-xtask --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m task test-xtask
slot task fmt-check
slot task ci-lint
```

`task test-xtask` is the gate for a change under `crates/edge/ess-xtask/` and to `docs.rs`; it runs the whole `ess-xtask` test set once (`ess-xtask` has no `--test` subset worth naming here because the step under change is that task). `task ci-lint` runs because `Taskfile.yml` changed.

## Acceptance

1. `no_test_task_empties_the_compile_wrapper_or_sets_build_jobs` and `test_xtask_builds_before_it_runs` are green and were red before the edit.
2. `grep -nE "RUSTC_WRAPPER=|CARGO_BUILD_JOBS" Taskfile.yml` finds nothing (the `env -u RUSTC_WRAPPER` lines of the harness tasks do not match the first pattern).
3. `SLOT_TIMEOUT=30m task test-xtask` is green; the report names the compile time and the run time apart, and the three numbers of step 0.
4. The `docs::` tests run within 120 s (run phase, compile excluded); if step 0 found the time was compile, the report says so, with the before and after of the compile.
5. No test of `ess-xtask` runs over 60 s; any over 5 s is named with its time.

## Decisions

- Names from t1 are not used here. This unit has no marker and no slow lane: the `ess-xtask` tests run in the step.
- The shared prefix is a task `vars:` entry, not `env:`, because go-task task `env:` does not override an inherited variable (the `Taskfile.yml` comment at 599 says so), and the whole point of the prefix is to override.
- The two commands, not one: the compile and the run are then separate lines in the log and separate timings for t7's step ceilings. `cargo test --no-run` followed by `cargo test` shares one build.
- `RUSTUP_TOOLCHAIN=1.98.1` and the lld, debug and incremental flags stay: they carry the consumer-check native profile this task exists for (the comment in the task).
- `consumer-check` is fixed in the same change because it is the same lines and the same rule; it stays opt-in (`CONSUMER_CHECKS=true`).
- Findings for the harness tasks and the macOS witness are reported, not changed: they are fixtures of a test's own child process (allowed) or the forge's own runner (a decision for a story in the slot repository, which does not exist yet).
- No `docs::` test is changed on a guess: step 4 applies only to a test the measurement names.

## Changelog

`### Changed`:

- CI: `task test-xtask` and `task consumer-check` build through the host's compile cache and job pool instead of an emptied `RUSTC_WRAPPER` and `CARGO_BUILD_JOBS=2`, and `test-xtask` compiles and runs in two commands.
