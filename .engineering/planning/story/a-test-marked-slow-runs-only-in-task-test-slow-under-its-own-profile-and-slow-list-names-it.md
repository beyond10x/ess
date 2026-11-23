---
format: aep.planning-md/3
id: story:a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it
kind: story
status: active
title: A test marked slow runs only in `task test-slow`, under its own nextest profile, and `cargo xtask slow-list` names it
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
- supersedes: story:a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries
scope:
- confidence: cited
  path: .config/nextest.toml
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Taskfile.yml
- confidence: inferred
  path: crates/edge/ess-xtask/src/main.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/slow_lane.rs
- confidence: inferred
  path: crates/edge/ess-xtask/tests/slow_lane.rs
- confidence: cited
  path: crates/edge/ess-xtask/tests/test_budget.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/adversary_e_u2_bytes.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/adversary_e_u6_bytes.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/adversary_244b_window_free_bytes.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/adversary_287_pass1.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/adversary_w3_266_267_dump.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/external_beside_held_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs
revision: 22
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:02:38Z", actor: "human:timo", revision: 21, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:02:40Z", actor: "human:timo", revision: 22, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

A test marked `#[ignore = "slow lane: <reason>"]` runs only in `task test-slow`, and nothing else in the repository needs to know where it is.

- `cargo xtask slow-list` prints every marked test (`<package>\t<target>\t<test>`), and `--packages`, `--package <P> --targets` and `--package <P> --filter` print the parts `task test-slow` needs.
- `task test-slow` (optional `PACKAGES`) builds only the test targets that hold a marked test, runs exactly those tests under the new nextest profile `slow`, prints the ten slowest of each package, and fails when any test fails or when no test is marked at all.
- The six tests `task test-slow-probes` ran by hand carry the marker (or, when measured fast, run in the test step: step 0), and `task test-slow-probes` no longer exists.
- This unit changes no workflow file. The CI job that runs the lane is `t1b`.

## Why

Parent: `story:the-test-step-meets-the-test-budget`, design step 5 (a separate lane named `slow`) and decisions D1 and D3. Rule: "Tests stay fast": full regeneration and generated-code compiles run in a separate lane named `slow`, not in the test step.
Consumer gap: none. Units t2 to t5 and p3 (its corpus test) put their tests into the lane through this marker and need only this unit, not the CI wiring (t1b).

## Where

Read at `80794bbd698` (the tree of the wave's plan; the numbers below hold there and at `bf128f9146`, where they were first read). Re-read a line before editing it; other units land beside this one.

- `crates/edge/ess-xtask/src/main.rs`: `mod` list lines 3-21 (add `mod slow_lane;`), `enum Command` 76-168 (add a variant `SlowList(slow_lane::Args)` after `WhatsChanged`, 151-156), `fn run` 210-250 (add the arm).
- `crates/edge/ess-xtask/src/slow_lane.rs`: new. Uses `syn` (a `[dependencies]` entry of `ess-xtask`, `Cargo.toml` line 21), `clap` and `std` only, no `crate::` path, so `tests/slow_lane.rs` can include it with `#[path = "../src/slow_lane.rs"] mod slow_lane;` as `tests/host_paths_adversary.rs:21` includes `../src/scratch.rs`.
- `crates/edge/ess-xtask/tests/slow_lane.rs`: new binary (one new integration binary in `ess-xtask`; it cannot be a module of an existing one because each existing binary includes `src/` files through its own `#[path]` and this one needs `scratch.rs` and `slow_lane.rs` together).
- `.config/nextest.toml`: `[profile.ci]` 19-21, `[profile.ci.junit]` 23-24. Add `[profile.slow]` and `[profile.slow.junit]` after them.
- `Taskfile.yml`: `test-browser-product` 320-323 (stays as it is), `test-slow-probes` 325-329 (delete), `test-slowest` 382-397 (the `report=` line 387 changes), `check` 638-657 (not changed here; t1b adds the step).
- `crates/edge/ess-xtask/tests/test_budget.rs`: `NEXTEST_TASKS` 15; `the_ci_profile_writes_the_junit_report_test_slowest_reads` (the `REPORT` assertion near line 165); `every_nextest_run_in_the_taskfile_uses_the_ci_profile` (about 171-191); `every_nextest_run_prints_its_ten_slowest_tests_even_when_it_fails` (about 193-217); `table` 77-90.
- The six probes (each `#[ignore = "slow probe: \`task test-slow-probes\`"]` today): `crates/generate/ess-synth/tests/adversary_e_u2_bytes.rs:109` (`adv_e_u2_byte_identity_probe`), `adversary_e_u6_bytes.rs:161` (`adv_e_u6_byte_identity_probe`), `crates/verify/ess-conformance/tests/adversary_244b_window_free_bytes.rs:828` (`adversary_244b_every_window_free_model_keeps_its_bytes_against_3b1684d1a`), `adversary_w3_266_267_dump.rs:83` (`adv_every_model_synthesizes_one_document_and_dumps_for_base_comparison`), `adversary_287_pass1.rs:1526` (`caller_upsert_wasm_matches_actual_native_outcomes`) and `:1678` (`caller_set_wasm_matches_actual_native_outcomes`).
- Two comments that name the deleted task: `crates/verify/ess-conformance/tests/external_beside_held_guard.rs:1090` and `crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs:71-73` (a one-line edit in a file t2 owns: this unit lands first and t2 takes the file as it finds it).
- `AGENTS.md` lines 220-222: the sentence "`task test-browser-product` and `task test-slow-probes` run in no CI lane ...".
- `CHANGELOG.md`: the `### Changed` heading under `## [Unreleased]` (line 9 at `80794bbd698`).
- Pin read, not edited: `crates/edge/ess-xtask/tests/ci_lanes.rs` (no pin requires a `test*` task to appear in a CI lane; `every_step_of_task_check_runs_in_some_pull_request_lane`, line 271, concerns `check` only, which this unit leaves alone).

## Tests first

Bounds, stated once. Wall-time figures in this story are the runner's own per-test or per-step figure (`target/nextest/<profile>/junit.xml`), not the command's wall time, taken on a warm build at a 1-minute load of 60 or less (`cut -d' ' -f1 /proc/loadavg`, read before and after; above 60 the run decides nothing and is repeated or the reading is named undecided). The unit's commands are bounded: no test over 60 s in the test step, the test step within 120 s, a pool job ends after 10 minutes unless it names its time (`SLOT_TIMEOUT=30m`, 30 at most). No Python for anything that stays in the repository: the scanner is Rust and the lane's driver is POSIX sh in a Taskfile command, as `test-slowest` is. A measuring script is a scratch file and starts without the `slot` prefix.

1. `crates/edge/ess-xtask/tests/slow_lane.rs` (fixtures through `scratch.rs`'s `Scratch`):
   - `the_scanner_reads_a_marker_on_a_test_and_nothing_else`: a scratch tree `crates/area/demo/{Cargo.toml,tests/a.rs,tests/b/main.rs,tests/b/support.rs}`; `a.rs` holds `#[test] #[ignore = "slow lane: builds a crate"] fn one()`, a bare `#[test] #[ignore] fn bare()`, a `#[ignore = "slow probe: x"]` test and an unmarked test; `b/support.rs` holds one marked test. Expected list: `demo a one` and `demo b <the support test>` and nothing else. Red today: the module does not exist (compile error `couldn't read ../src/slow_lane.rs`).
   - `the_scanner_refuses_a_marker_it_cannot_run`: a marker on a function without `#[test]`, a marker with an empty reason (`"slow lane: "`), and two marked functions of one name in one package each fail with a message naming file and function.
   - `every_marker_in_the_tree_is_a_row_of_the_table_and_every_row_is_marked`: scans the real tree and compares with `MARKED: &[(&str, &[&str])]` (workspace-relative file, its marked test names, rows sorted by path; the six probes). Red once the module exists and before the markers are renamed: it lists the six rows as `in the table, not marked`.
2. `crates/edge/ess-xtask/tests/test_budget.rs`:
   - new `the_slow_profile_kills_a_test_after_twenty_minutes_and_writes_junit`: `[profile.slow]` holds exactly `slow-timeout = { period = "120s", terminate-after = 10 }` and `final-status-level = "slow"`, and `[profile.slow.junit]` holds `path = "junit.xml"`. Red: `.config/nextest.toml has no [profile.slow] table`.
   - `every_nextest_run_in_the_taskfile_uses_the_ci_profile`: accepts `--profile ci`, or `--profile slow` together with `--run-ignored only` and `-E`; `NEXTEST_TASKS` gains `test-slow`. Red: `test-slow` no longer runs nextest, naming the task.
   - `every_nextest_run_prints_its_ten_slowest_tests_even_when_it_fails`: a task that runs nextest in a loop (`test-slow`) passes when its one shell command holds `|| status=1` after the nextest call and `task test-slowest` after that, in this order; the deferred form stays for the other three tasks.
   - `the_ci_profile_writes_the_junit_report_test_slowest_reads`: the assertion at the `REPORT` line becomes `report="${SLOWEST_REPORT:-target/nextest/ci/junit.xml}"`; a second assertion: `test-slow` calls `task test-slowest` with `SLOWEST_REPORT=target/nextest/slow/junit.xml` on the same line.
3. Time each new test; none may run over 5 s, the three of `slow_lane.rs` together stay within 10 s. Run each red once before the implementation.

## Steps

0. Measure the six probes first. Run each once with its marker still `ignore`, from a tree where the code builds, at a 1-minute load of 60 or less: `SLOT_TIMEOUT=30m cargo nextest run --profile ci -p ess-synth --test adversary_e_u2_bytes --test adversary_e_u6_bytes --run-ignored only` and `SLOT_TIMEOUT=30m cargo nextest run --profile ci -p ess-conformance --test adversary_244b_window_free_bytes --run-ignored only`, then the same for `adversary_w3_266_267_dump` and for `adversary_287_pass1` with `-E 'test(/wasm_matches_actual_native_outcomes/)'` (one `ess-conformance` target per run; their builds are large). Read each test's time from `target/nextest/ci/junit.xml`. Decision rule, one threshold: a probe that runs over 30 s is marked slow (step 6) and its time goes into its reason; a probe that runs at most 30 s loses its `#[ignore]` and stays in the test step (not marked, no `MARKED` row), and the report names it with its time; a probe whose measurement was at a load above 60 is marked slow and named `undecided`. If every probe is at most 30 s, `MARKED` still needs one row: keep the slowest probe marked, so the lane is never empty. Write the six times in the report.
1. Write the tests of `## Tests first`, run them red.
2. `src/slow_lane.rs`. Scan `crates/<area>/<crate>/tests/**/*.rs`, skipping any directory named `fixtures`, with `std::fs`; parse each file with `syn::parse_file`; visit every `ItemFn` at any module depth; a test is a function with an attribute whose path ends in `test`; a marker is an `ignore` attribute `= "slow lane: <reason>"` on a test. The package name is the first `name = "..."` after `[package]` in the crate's `Cargo.toml`; the target is the first path component below `tests/` (the file stem, or the directory name). `Args` (`#[derive(Debug, clap::Args)]`): no flag prints one `<package>\t<target>\t<test>` line per marked test, sorted; `--packages` prints the package names, one per line; `--package <P> --targets` prints that package's targets, one per line; `--package <P> --filter` prints one nextest filterset, `test(/(^|::)(<name1>|<name2>)$/)`, so that a test that moves into a module of a merged binary (`story:one-integration-test-binary-per-large-crate`) is still found. Refuse as `the_scanner_refuses_a_marker_it_cannot_run` says. An empty tree is not an error for the scanner; `test-slow` refuses an empty lane (step 5).
3. `main.rs`: the `mod`, the `SlowList` variant and its arm.
4. `.config/nextest.toml`, without comments, after line 24: `[profile.slow]` with `slow-timeout = { period = "120s", terminate-after = 10 }` and `final-status-level = "slow"`, then `[profile.slow.junit]` with `path = "junit.xml"`.
5. `Taskfile.yml`:
   - `test-slowest`: the report line becomes `report="${SLOWEST_REPORT:-target/nextest/ci/junit.xml}"`; the rest is unchanged.
   - new task `test-slow`, one `desc` sentence, one POSIX sh command: `set -eu`, `status=0`, `wanted="{{.PACKAGES}}"`, a guard that exits 1 with `test-slow: no test is marked #[ignore = "slow lane: ..."]` when `cargo xtask slow-list --packages` prints nothing, then `for package in $(cargo xtask slow-list --packages)`; inside, skip a package not in `$wanted` when `$wanted` is non-empty; build the `--test <target>` words from `cargo xtask slow-list --package "$package" --targets`; run `cargo nextest run --locked --profile slow --run-ignored only --no-fail-fast --package "$package" $targets -E "$(cargo xtask slow-list --package "$package" --filter)" || status=1`; then `SLOWEST_REPORT=target/nextest/slow/junit.xml task test-slowest`; after the loop `exit "$status"`. One nextest run per package, because cargo may refuse a `--test` name that exists in only one of several selected packages (not verified by reading; a loop does not depend on it).
   - delete `test-slow-probes` (325-329). `test-browser-product` and the harness tasks are not touched (see Decisions).
   - no comment in what you add.
6. Mark the probes that step 0 said to mark: replace the `#[ignore = "slow probe: ..."]` line with `#[ignore = "slow lane: <reason>"]`, the reason a few words naming what the probe does and its measured seconds (for example `byte identity of every model's generated Rust and Go, about <n> s`); a probe that step 0 put in the step loses the line. Add one `MARKED` row per file with its marked names, sorted by path. Edit the two comments of `## Where` to say `#[ignore = "slow lane: ..."]` in `task test-slow` where they said `task test-slow-probes`.
7. `AGENTS.md` lines 220-222: keep `task test-browser-product` and its sentence as they are; replace "and `task test-slow-probes` run in no CI lane (those Firefox tests and slow probes are ignored in the CI test shards)" so the paragraph says: `task test-browser-product` runs in no CI lane; the slow lane, `task test-slow`, runs every test marked `#[ignore = "slow lane: <reason>"]` (`cargo xtask slow-list` names them). Leave the rest of the paragraph. `t1b` appends the CI half of the sentence.
8. `CHANGELOG.md` bullet (below). `git grep -n "test-slow-probes" -- . ':!.engineering'` finds nothing.
9. Run `## Gate`. Then `SLOT_TIMEOUT=30m task test-slow` once and report its wall time and each marked test's time.

## Owned files

`crates/edge/ess-xtask/src/main.rs` (the three lines), `crates/edge/ess-xtask/src/slow_lane.rs`, `crates/edge/ess-xtask/tests/slow_lane.rs`, `.config/nextest.toml` (`[profile.slow]` and `[profile.slow.junit]`), `Taskfile.yml` (`test-slow`, `test-slowest`'s report line, the deleted `test-slow-probes`), `crates/edge/ess-xtask/tests/test_budget.rs` (the four tests above), the six probe files (marker lines only), `crates/verify/ess-conformance/tests/external_beside_held_guard.rs` (the comment at 1090), `crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs` (the comment at 71-73), `AGENTS.md` lines 220-222, `CHANGELOG.md` (one bullet).

## Not yours

- t1b: the `diff` step, the `slow` job, `Gate`, `release.yml`, the `check` step, the `ci_lanes.rs` tests, the CI half of the `AGENTS.md` sentence. t2: the rest of `synthesis_suite_bytes_table.rs`. t3 to t5: their files. t6, t7.
- `test-browser-product` and the five harness tasks (`integer-spellings-harness`, `append-set-harness`, `json-body-harnesses`, `ui-http-servers`, `clap-shared-binary`/`binary64-harness`) stay as they are: moving them to the lane is a follow-up under the parent (X8(b) on the wave page), not this unit.
- Shared with later units: `crates/edge/ess-xtask/tests/slow_lane.rs` (`MARKED`: t2, t3, t4, t5 and p3 (artifact `with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite`, its corpus test) add rows; this unit lands first; at merge keep every row, one per file, sorted by path), `Taskfile.yml` (t3 adds `env:` to `test-slow`; t1b adds the `check` step; different lines), `AGENTS.md` (t1b appends to the same sentence), `CHANGELOG.md` (keep every unit's bullet at merge).

## Gate

```console
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-xtask --locked --test slow_lane --test test_budget --test ci_lanes
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-xtask --all-targets --locked -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-conformance --locked --test adversary_244b_window_free_bytes --test adversary_w3_266_267_dump --test adversary_287_pass1 --test external_beside_held_guard --test synthesis_suite_bytes_table -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-synth --locked --test adversary_e_u2_bytes --test adversary_e_u6_bytes -- -D warnings
slot task fmt-check
SLOT_TIMEOUT=30m task test-slow
```

Named `--test` targets only; `ess-conformance` one target at a time. `task test-xtask` is not needed (no `ValidationCode`, diagnostic, format version, `website/` or `docs/` change) and `task ci-lint` is not needed (no workflow changed, `Taskfile.yml` only gains a task and loses one: run it once anyway because the table row "a workflow, `Taskfile.yml` or a public API" names the file: `slot task ci-lint`).

## Acceptance

1. `cargo test -p ess-xtask --test slow_lane` is green; `MARKED` and the markers in the tree agree, and the three tests were red before the implementation.
2. `cargo xtask slow-list` prints one row per marked probe (as many as step 0 left marked, at least one), and `--filter` prints a `test(/(^|::)(...)$/)` filterset naming each.
3. `SLOT_TIMEOUT=30m task test-slow` runs exactly those tests, prints the ten slowest tests per package from `target/nextest/slow/junit.xml`, and exits 0 when they pass; with every marker removed it exits 1 with `no test is marked`.
4. The report lists the six probes with their measured seconds, the load read before and after each measurement, and the decision (marked or left in the step) by the 30 s threshold of step 0.
5. `cargo test -p ess-xtask --test test_budget` is green with `test-slow` in `NEXTEST_TASKS` and the `slow` profile pinned.
6. `git grep -n "test-slow-probes" -- . ':!.engineering'` finds nothing.
7. No workflow file changed: `git diff --stat` shows none under `.github/`.

## Decisions

Names every other unit repeats (fixed here, not reopened):
- marker: `#[ignore = "slow lane: <reason>"]` on a `#[test]` in a `crates/<area>/<crate>/tests/**.rs` file;
- command `cargo xtask slow-list`; task `test-slow` with optional `PACKAGES`; nextest profile `slow`;
- table `MARKED` in `crates/edge/ess-xtask/tests/slow_lane.rs`; variable `SLOWEST_REPORT` for `test-slowest`;
- CI names (t1b): job id `slow`, display name `Slow lane`, `diff` job output `slow` (`true`, `false` or empty).

Reasons:
- A marker, not a tag file or a name list: it sits on the test, `syn` reads it, `MARKED` pins it, and nextest's own `--run-ignored only` runs it. One source of truth that moves with the function when `story:one-integration-test-binary-per-large-crate` merges binaries.
- The filter is package plus a test-name regex, not `binary_id`, so that story's merge needs no change here. That story lands after this wave.
- The slow profile kills a test at 20 minutes (`period` 120 s, `terminate-after` 10); the 60 s test bound does not apply to the lane. The CI job's 30 minutes is t1b's.
- One nextest run per package, in a shell loop, because the cargo behaviour for a `--test` held by one package of several was not verified; revisit only with a measurement.
- The probe threshold is 30 s at a 1-minute load of 60 or less (coordinator decision Y8, the same as t3, t4 and t5): over it, marked slow; at most it, back in the test step; undecided under load, marked slow.
- `test-browser-product` and the harness tasks stay as they are; moving them is a follow-up under the parent (X8(b)).
- No `RUSTC_WRAPPER`, `SCCACHE_*`, `-j` or `CARGO_BUILD_JOBS` in the new task.
- D3 (feature-off tests to the lane) is not in this unit or in the wave.

## Changelog

`### Changed`, one bullet (keep every unit's bullet at merge):

- Testing: a test marked `#[ignore = "slow lane: <reason>"]` runs only in `task test-slow`, under the nextest profile `slow`, and `cargo xtask slow-list` names it; the byte-identity and WebAssembly probes that `task test-slow-probes` ran by hand carry the marker, and that task is gone (story:the-test-step-meets-the-test-budget).
