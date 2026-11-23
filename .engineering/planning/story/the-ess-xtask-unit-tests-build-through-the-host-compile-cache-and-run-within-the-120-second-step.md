---
format: aep.planning-md/3
id: story:the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step
kind: story
status: active
title: The ess-xtask unit tests build through the host compile cache and run within the 120 second step
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
- confidence: cited
  path: crates/edge/ess-xtask/tests/ci_lanes.rs
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:02:42Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:02:44Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"review_outcome":4}}}
---
## Outcome

`task test-xtask` builds `ess-xtask` and its test targets through the host's shared compile cache and job pool, its compile and run phases are separate commands in the log, and its run phase holds the `docs::` tests within the 120 second step.

- The `test-xtask` command has no `RUSTC_WRAPPER=`, `RUSTC_WORKSPACE_WRAPPER=`, `CARGO_BUILD_RUSTC_WRAPPER=`, `CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER=` and no `CARGO_BUILD_JOBS=2`.
- `test-xtask` runs `cargo test --package ess-xtask --locked --no-run` and then `cargo test --package ess-xtask --locked`, so a log or a timing run shows the compile and the test time apart.
- The 28 `docs::` tests of the `ess-xtask` binary (about 217 s in addition A) run, with the rest of `ess-xtask`'s tests, within 120 s in the second command, on a warm build at a 1-minute load of 60 or less; the report says whether the 217 s were compile or run time.
- A test in `ci_lanes.rs` fails when `test-xtask` empties the wrapper, sets build jobs, or does not build before it runs.

## Why

Parent: `story:the-test-step-meets-the-test-budget`, step 4 and addition A (the `ess-xtask` unit binary, `docs::`, 216.73 s, "not read: `docs.rs` has 64 functions and no process spawn, so the time is in the tests that call into it or in the link").
Rules: "Tests stay fast" (120 s step; find the cost before touching anything); "One compile cache" (no build sets the wrapper to the empty string, no `-j` or `CARGO_BUILD_JOBS`).
Consumer gap: none.

## Where

Read at `80794bbd698` (unchanged since `bf128f9146`). Re-read a line before editing it.

- `Taskfile.yml` lines 404-418, `test-xtask`: `desc`, a three-line comment (406-408, keep it), then one command (409-417) that starts `env -u CARGO_ENCODED_RUSTFLAGS RUSTUP_TOOLCHAIN=1.98.1 RUSTFLAGS='-C link-arg=-fuse-ld=lld'`, then `RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER=` (413), `CARGO_BUILD_RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER=` (414), `CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0` (415), the `CARGO_PROFILE_*` flags (416-417), and `cargo test --package ess-xtask --locked` (418). `RUSTUP_TOOLCHAIN=1.98.1` equals `rust-toolchain.toml`; keep it. Line 599 (in `consumer-check`, read only) says "Command assignments override inherited variables, unlike task-level env defaults": so the shared prefix is a task `vars:` entry, not a task `env:`.
- `Taskfile.yml` `test` (77-99): the `test-xtask` call is a step, and `ci_lanes.rs:309-322` pins that `test` calls `test-xtask` and does not restate it.
- `crates/edge/ess-xtask/src/docs.rs`: `#[cfg(test)] mod tests` from line 1211 (28 `#[test]` functions, none spawns a process; they read the workspace through `crate::workspace_root()`: for example `a_catalogue_release_the_page_disagrees_with_is_refused`, `every_family_a_published_page_names_is_tracked`, `the_committed_readme_installs_the_newest_release`, `the_committed_release_notes_do_not_trail_the_newest_release`). `ess-xtask` has no library: its unit tests are in the binary `ess-xtask` (`main.rs` `mod docs;`, read only, not edited).
- `crates/edge/ess-xtask/tests/ci_lanes.rs`: `shell_commands` (the commands of a task), `every_step_of_task_check_runs_in_some_pull_request_lane`, and the `test-xtask` pins at 309-322. Append the new tests at the end of the file.
- Findings, not yours (see `## Not yours`): `consumer-check` (`Taskfile.yml` 597-611, same emptied wrapper and `CARGO_BUILD_JOBS=2`), the harness tasks (184, 208, 228, 255, 258, 278, 281, 307: `env -u RUSTC_WRAPPER`), `.github/workflows/ci.yml:836` (`CARGO_BUILD_JOBS: "2"`, macOS witness job).
- `CHANGELOG.md`: the `### Changed` heading under `## [Unreleased]`.

## Tests first

Bounds: wall-time figures are measured on a warm build at a 1-minute load of 60 or less (`cut -d' ' -f1 /proc/loadavg`, before and after, both in the report; above 60 the run decides nothing and is repeated), by the runner's own figure (the `finished in` line of each test binary under `cargo test`, and the shell's `time` of the whole command for the step); no test over 60 s, the test step within 120 s, a pool job ends after 10 minutes unless it names its time (`SLOT_TIMEOUT=30m`, 30 at most); no Python for anything that stays. Time every test; any over 5 s is named in the report. A measurement script is a scratch file in the session scratch directory, not committed, in POSIX sh, and starts without the `slot` prefix (it measures and waits); the `cargo` calls inside take a slot through the shim.

0. Measure first, three numbers. Use `CARGO_TARGET_DIR=<scratch>/xtask-target` for the first two so that neither run reuses the other's artifacts, and time each with the shell's `time`:
   - (a) compile as today: `cargo test --package ess-xtask --locked --no-run` under today's `env` prefix from `Taskfile.yml` lines 410-417, including `RUSTC_WRAPPER=` and `CARGO_BUILD_JOBS=2`;
   - (b) compile with the prefix of step 2 below (no wrapper assignments, no jobs);
   - (c) run only, in the target of (b), warm: `cargo test --package ess-xtask --locked` wall time and the `finished in` lines per binary, and `cargo nextest run --package ess-xtask --bin ess-xtask docs::` for per-test times (nextest is only the measuring tool here; `task test-xtask` stays on `cargo test`).
   Decision rules: if (a) minus (b) is over 60 s, the wrapper and jobs lines were the cost: steps 1 to 3 are the fix and nothing else is needed for `docs::`. If (c) shows a `docs::` test over 5 s, it is named, and step 3 applies. If (c) is over 120 s with every test under 5 s, the number of tests is the cost: report it as a finding and stop for that part (no test is changed on a guess). Landing rule: when the measured time cannot meet acceptance 4 (the second command over 120 s on a warm build at load 60 or less after steps 2 and 3), the unit does not land; it reports the measurement, the cause it found and the smallest change that would meet it, for the coordinator.
1. `crates/edge/ess-xtask/tests/ci_lanes.rs`, new test `test_xtask_sets_no_wrapper_and_no_build_jobs`: reads `Taskfile.yml`; for the task `test-xtask` only, no command and no `vars` value contains a word starting `RUSTC_WRAPPER=`, `RUSTC_WORKSPACE_WRAPPER=`, `CARGO_BUILD_RUSTC_WRAPPER=` or `CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER=` (with or without a value), `CARGO_BUILD_JOBS`, `SCCACHE_`, or the words `-j` and `--jobs`. Red today: it names `test-xtask` line 413. The pin over every `test*` task and over `consumer-check` is not this unit's (see `## Not yours`).
2. New test `test_xtask_builds_before_it_runs`: `shell_commands(&taskfile, "test-xtask")` has two commands, the first ending `--no-run`, the second not containing it, and both carry `--package ess-xtask --locked`, `RUSTUP_TOOLCHAIN=1.98.1`, `-fuse-ld=lld`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0` and `CARGO_PROFILE_TEST_DEBUG=0` (after variable expansion of the task's `vars`: the check reads the task's `vars` and its commands together). Red today: one command only.
3. Run both red.

## Steps

1. Measure (step 0 above), keep the three numbers.
2. `Taskfile.yml` `test-xtask`: add `vars:` with `XTASK_ENV` holding the shared prefix, as a folded string: `env -u CARGO_ENCODED_RUSTFLAGS RUSTUP_TOOLCHAIN=1.98.1 RUSTFLAGS='-C link-arg=-fuse-ld=lld' CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_INCREMENTAL=false CARGO_PROFILE_TEST_INCREMENTAL=false`. `cmds`: `{{.XTASK_ENV}} cargo test --package ess-xtask --locked --no-run` then `{{.XTASK_ENV}} cargo test --package ess-xtask --locked`. Keep the existing comment lines above the commands. No new comment. The four wrapper assignments and `CARGO_BUILD_JOBS=2` are gone.
3. Only if step 0 named a `docs::` test over 5 s: find the repeated expensive call in it (for example a walk of `website/docs` or a read of `CHANGELOG.md` made by several tests) by timing it with a scratch `eprintln!` of `Instant::elapsed`; share the product in one `fn documents_of_the_tree() -> &'static ...` with a `std::sync::OnceLock` inside `#[cfg(test)] mod tests` of `docs.rs`. A `OnceLock` is valid here and only here: `cargo test` runs all the tests of the binary in one process (not under nextest). Re-time and report both numbers. If nothing is over 5 s, skip this step.
4. Run the two new tests green, then `SLOT_TIMEOUT=30m task test-xtask` twice, the second on a warm build, and report: the first command's output of the second run (it shows no `Compiling` line: that is how the warm state is shown), the compile time and the run time apart (the second command's wall time and its `finished in` lines), and the 1-minute load before and after.
5. Report as findings, not changes: `consumer-check` and the `ci_lanes` pin over every `test*` task (the existing draft `story:consumer-coverage-build-profile-admits-the-hosts-compile-wrapper-and-no-taskfile-task-empties-rustc-wrapper` owns both), the harness tasks `Taskfile.yml` 184-307 (`env -u RUSTC_WRAPPER`, a build of a fixture under a clean environment; the rule allows an emptied wrapper only for a test's fixture child process), `ci.yml:836` (`CARGO_BUILD_JOBS: "2"` in the macOS output-ownership witnesses; a hosted runner has no pool and no sccache, so the setting is the forge's, but the rule says no `-j` or `CARGO_BUILD_JOBS` in a gate). There is no slot-repository story naming these exceptions; propose one in the report.
6. Add the Changelog bullet below. No `AGENTS.md` change.

## Owned files

`Taskfile.yml` task `test-xtask` (lines 404-418 only), `crates/edge/ess-xtask/src/docs.rs` `mod tests` (step 3 only, if it applies), the two new functions appended to `crates/edge/ess-xtask/tests/ci_lanes.rs`, and one bullet in `CHANGELOG.md`.

## Not yours

- t1a: `test-slow`, `test-slowest`, `check` and `test-slow-probes` lines of `Taskfile.yml`, `nextest.toml`, `slow_lane.rs`, `test_budget.rs`. t1b: `ci.yml`, `release.yml`. t7: the step budget wrapper around `test-xtask` (a `timeout` around its commands, its ceiling in `AGENTS.md` § Gate) and the `ci_lanes.rs` ratchet. t2 to t5: test files.
- The `consumer-check` task (`Taskfile.yml` 597-611) and the `ci_lanes` pin over every `test*` task: the draft `story:consumer-coverage-build-profile-admits-the-hosts-compile-wrapper-and-no-taskfile-task-empties-rustc-wrapper`. `crates/edge/ess-xtask/src/main.rs`: not edited.
- The harness tasks 184-307, `fuzz-check`, `test-doc`, `test-feature-off*` and `ci.yml:836`: findings only.
- Shared, all by "keep both sides at merge": `Taskfile.yml` (different tasks; t1a, t7, h2 and h3 edit others; the merge is by task), `ci_lanes.rs` (appended at the end by t1b, t3, h3 and you), `docs.rs` (w1 adds one row outside `mod tests`), `CHANGELOG.md` (one bullet each). t7 lands after this unit and wraps the two commands you wrote: it takes them as they are.

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

1. `test_xtask_sets_no_wrapper_and_no_build_jobs` and `test_xtask_builds_before_it_runs` are green and were red before the edit.
2. `grep -nE "RUSTC_WRAPPER=|CARGO_BUILD_JOBS" Taskfile.yml` finds matches only in `consumer-check` (lines 597-611, the draft story's); none in `test-xtask`.
3. `SLOT_TIMEOUT=30m task test-xtask` is green; the report names the compile time and the run time apart, and the three numbers of step 0.
4. On a warm build (the second run of `task test-xtask`, whose first command prints no `Compiling` line) at a 1-minute load of 60 or less, the second command, which holds the `docs::` tests, ends within 120 s by the shell's `time`; the report states the time and both loads. There is no exception for a cold cache: the figure of a run that compiled is not the figure. If step 0 found the 217 s were compile, the report says so, with the before and after of the compile.
5. No test of `ess-xtask` runs over 60 s; any over 5 s is named with its time.

## Decisions

- Names from t1a are not used here. This unit has no marker and no slow lane: the `ess-xtask` tests run in the step.
- The shared prefix is a task `vars:` entry, not `env:`, because go-task task `env:` does not override an inherited variable (the `Taskfile.yml` comment at 599 says so), and the whole point of the prefix is to override.
- The two commands, not one: the compile and the run are then separate lines in the log and separate timings for t7's step ceilings. `cargo test --no-run` followed by `cargo test` shares one build.
- `RUSTUP_TOOLCHAIN=1.98.1` and the lld, debug and incremental flags stay: they carry the consumer-check native profile this task exists for (the comment in the task).
- `consumer-check` and the pin over every `test*` task are cut from this unit (coordinator, X9): the parent asks only that the `ess-xtask` unit tests run within the step. They are the draft story's.
- Findings for the harness tasks and the macOS witness are reported, not changed: they are fixtures of a test's own child process (allowed) or the forge's own runner (a decision for a story in the slot repository, which does not exist yet).
- No `docs::` test is changed on a guess: step 3 applies only to a test the measurement names.

## Changelog

`### Changed`:

- CI: `task test-xtask` builds through the host's compile cache and job pool instead of an emptied `RUSTC_WRAPPER` and `CARGO_BUILD_JOBS=2`, and compiles and runs in two commands.
