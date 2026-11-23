---
format: aep.planning-md/3
id: story:a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries
kind: story
status: archived
title: 'A `slow` lane runs the tests marked `#[ignore = "slow lane: <reason>"]` in `task test-slow` and in a `slow` CI job that `Gate` carries'
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
scope:
- confidence: cited
  path: .config/nextest.toml
- confidence: cited
  path: .github/workflows/ci.yml
- confidence: cited
  path: .github/workflows/release.yml
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
- confidence: cited
  path: crates/edge/ess-xtask/tests/ci_lanes.rs
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
- confidence: inferred
  path: crates/verify/ess-conformance/tests/external_beside_held_guard.rs
revision: 20
transitions:
- {from: "draft", to: "archived", at: "2026-11-23T05:31:34Z", actor: "human:timo", revision: 20}
---
## Outcome

A lane named `slow` exists, locally and on CI, and it carries every test that is marked `#[ignore = "slow lane: <reason>"]`.

- `task test-slow` builds only the test targets that hold a marked test and runs exactly the marked tests under the nextest profile `slow`. `cargo xtask slow-list` names them.
- `.github/workflows/ci.yml` has a job `slow` (`Slow lane`). It runs on a `main` push, the nightly run, a `queue/**` head, a merge group, and a call from `release.yml`, always. On a pull request from an ordinary head or a fork it runs unless every changed path is in a list of paths no slow test reads.
- `Gate` carries `slow`. A run that skipped the lane never counts for a release: `prior-gate` accepts only runs that cannot skip it.
- The six tests of `task test-slow-probes` carry the marker, and that task no longer exists. The probes now run in CI for the first time.

## Why

Parent: `story:the-test-step-meets-the-test-budget`, design step 5 and decisions D1 and D3. Organization rule "Tests stay fast": full regeneration and generated-code compiles run in a separate lane named `slow`, only when the sources changed, and compare bytes with the committed file.
Consumer gap: none. This unit is the mechanism the other units of the wave (t2 to t5, t7) put their tests into; without it none of them can leave the test step.

## Where

Read at `bf128f9146` on `integration/wave-2026-11-22`. Re-read a line before editing it; another session commits in this tree.

- `crates/edge/ess-xtask/src/main.rs`: `mod` list lines 3-21 (add `mod slow_lane;`), `enum Command` lines 76-168 (add a variant `SlowList(slow_lane::Args)` after `WhatsChanged`, line 151-156), `fn run` lines 210-250 (add the arm; the other arms call `module::run(&root, ...).map_err(|error| format!("{error:#}"))`, `Command::Docs => docs::run(&root)` is the plain form).
- `crates/edge/ess-xtask/src/slow_lane.rs`: new. `syn` (a `[dependencies]` entry of `ess-xtask`, `Cargo.toml` line 21) and `std` only, no `crate::` path, so that `tests/slow_lane.rs` can include it with `#[path = "../src/slow_lane.rs"] mod slow_lane;` as `tests/host_paths_adversary.rs:21` includes `../src/scratch.rs`.
- `crates/edge/ess-xtask/tests/slow_lane.rs`: new.
- `.config/nextest.toml`: `[profile.ci]` lines 19-21 and `[profile.ci.junit]` lines 23-24. Add `[profile.slow]` and `[profile.slow.junit]` after them.
- `Taskfile.yml`: `test` 77-99, `test-packages` 104-137, `test-browser-product` 320-323, `test-slow-probes` 325-329 (delete), `test-shard` 335-358, `test-slowest` 382-397, `check` 638-657.
- `.github/workflows/ci.yml`: job `diff` 63-127 (outputs 69-70; `plan-only` step 81-127), job `test` 429-673 (its setup steps 461-588 are the model for `slow`), job `gate` 772-817, job `output-ownership-macos` 819.
- `.github/workflows/release.yml`: `prior-gate` step 131-251, `qualifies` 153-168. Logic unchanged. Only its comment 108-130 gains one sentence.
- `crates/edge/ess-xtask/tests/ci_lanes.rs`: `gated_jobs` 65, `lane_invocations` 134, `every_step_of_task_check_runs_in_some_pull_request_lane` 271, `the_gate_check_aggregates_every_lane_and_cannot_be_skipped` 1206 (`bound` is 30 for every job but `test`, line 1258), `the_archive_jobs_build_the_test_binaries_without_debug_information` 1799, `PLAN_ONLY_*` constants 4666-4682, `assert_plan_only_skips_exactly_the_test_jobs` 4689 (a job other than the three skipped ones must not carry "plan-only" in its `if`, line 4713), `plan_only_candidate` 4756, `ci_context` 4773, `no_run_that_must_test_everything_or_stands_in_for_a_release_can_skip_for_a_plan_only_diff` 4862, `run_gate` 4971, `the_gate_passes_skipped_test_jobs_only_for_a_plan_only_diff` 5013, `run_plan_only_step` 5113, `plan_only_checkout` 5169, `the_plan_only_decision_is_every_changed_path_under_the_plan_store` 5203.
- `crates/edge/ess-xtask/tests/test_budget.rs`: `NEXTEST_TASKS` 15, `the_ci_profile_writes_the_junit_report_test_slowest_reads` 158-169 (asserts `report=target/nextest/ci/junit.xml` at line 165), `every_nextest_run_in_the_taskfile_uses_the_ci_profile` 171-191, `every_nextest_run_prints_its_ten_slowest_tests_even_when_it_fails` 193-217.
- The six probes: `crates/generate/ess-synth/tests/adversary_e_u2_bytes.rs:109` (`adv_e_u2_byte_identity_probe`), `adversary_e_u6_bytes.rs:161` (`adv_e_u6_byte_identity_probe`), `crates/verify/ess-conformance/tests/adversary_244b_window_free_bytes.rs:828` (`adversary_244b_every_window_free_model_keeps_its_bytes_against_3b1684d1a`), `adversary_w3_266_267_dump.rs:83` (`adv_every_model_synthesizes_one_document_and_dumps_for_base_comparison`), `adversary_287_pass1.rs:1526` (`caller_upsert_wasm_matches_actual_native_outcomes`) and `:1678` (`caller_set_wasm_matches_actual_native_outcomes`). Each line today is `#[ignore = "slow probe: `task test-slow-probes`"]`.
- `crates/verify/ess-conformance/tests/external_beside_held_guard.rs:1090`: a doc comment naming `task test-slow-probes`.
- `AGENTS.md` lines 220-222 (the sentence that names `task test-slow-probes`).
- `CHANGELOG.md` `## [Unreleased]` line 3, `### Changed` line 5.

## Tests first

Time every new test; none may run over 5 s, and the whole of `tests/slow_lane.rs` stays within 10 s. The test step bounds are 60 s per test and 120 s per step.

1. `crates/edge/ess-xtask/tests/slow_lane.rs` (new binary; fixtures under `crate::scratch::Scratch`, included by `#[path = "../src/scratch.rs"]`):
   - `the_scanner_reads_a_marker_on_a_test_and_nothing_else`: a scratch tree `crates/area/demo/{Cargo.toml,tests/a.rs,tests/b/main.rs,tests/b/support.rs}` where `a.rs` has `#[test] #[ignore = "slow lane: builds a crate"] fn one()`, a bare `#[test] #[ignore] fn bare()`, a `#[ignore = "slow probe: x"]` test, and an unmarked test. Expects the list to be the rows `demo a one` and, for a marker in `b/support.rs`, `demo b <fn>`; nothing for the other three. Red today: `couldn't read ../src/slow_lane.rs` (the module does not exist).
   - `the_scanner_refuses_a_marker_it_cannot_run`: a marker on a function without `#[test]`, a marker with an empty reason (`"slow lane: "`), and two marked functions of one name in one package each fail with a message naming file and function.
   - `every_marker_in_the_tree_is_a_row_of_the_table_and_every_row_is_marked`: scans the real tree and compares with the constant `MARKED: &[(&str, &[&str])]` (file relative to the workspace root, then its marked test names, sorted by path). It holds the six probes. Red once the module exists and before the markers are renamed: the failure lists the six rows as "in the table, not marked".
2. `crates/edge/ess-xtask/tests/test_budget.rs`:
   - `the_slow_profile_kills_a_test_after_twenty_minutes_and_writes_junit` (new): `[profile.slow]` holds exactly `slow-timeout = { period = "120s", terminate-after = 10 }` and `final-status-level = "slow"`, and `[profile.slow.junit]` holds `path = "junit.xml"`. Red: `.config/nextest.toml has no `[profile.slow]` table`.
   - `every_nextest_run_in_the_taskfile_uses_the_ci_profile`: the loop at 183-190 accepts `--profile ci`, or `--profile slow` together with `--run-ignored only` and `-E`. `NEXTEST_TASKS` gains `test-slow`. Red: `` `test-slow` no longer runs nextest ``.
   - `every_nextest_run_prints_its_ten_slowest_tests_even_when_it_fails`: a task that runs nextest in a loop (`test-slow`) passes when its one shell command holds `|| status=1` after the nextest call and `task test-slowest` after that, in this order. The deferred form stays for the other three tasks.
   - `the_ci_profile_writes_the_junit_report_test_slowest_reads`: line 165 asserts the command contains `${SLOWEST_REPORT:-target/nextest/ci/junit.xml}`. A second assertion: `test-slow` calls `task test-slowest` with `SLOWEST_REPORT=target/nextest/slow/junit.xml` on the same line.
3. `crates/edge/ess-xtask/tests/ci_lanes.rs` (append each as a new function; do not restructure an existing one beyond what is named):
   - `the_slow_job_is_gated_bounded_and_runs_the_slow_task`: job `slow` exists, `needs` is exactly `[diff]`, `if` is exactly `${{ needs.diff.outputs.slow != 'false' }}`, `timeout-minutes` is 30, a step runs `task test-slow`, `env.CARGO_PROFILE_DEV_DEBUG` and `env.CARGO_PROFILE_TEST_DEBUG` are `"0"`, `Gate`'s `needs` and its script read `needs.slow.result`. Red: `ci.yml has no slow job`.
   - `the_slow_decision_may_skip_only_a_pull_request_of_an_ordinary_head`: the `diff` job has a step `slow` whose `if` equals `PLAN_ONLY_STEP_IF` (line 4675), reuses the `runs` table of test 4862 (the same eleven tuples and expected `may` flags) through `plan_only_candidate`, and asserts `outputs.slow` is `${{ steps.slow.outputs.slow }}`.
   - `no_run_that_stands_in_for_a_release_can_skip_the_slow_lane`: the `described` table of test 4862 (lines 4922-4931) run through `prior_gate_qualifies`; no run that qualifies is one the slow decision may skip.
   - `the_slow_decision_is_every_changed_path_in_the_skippable_list`: runs the `slow` step against real Git histories as test 5203 runs the `plan-only` step. Expected outputs are in step 3 of `## Steps`. Generalize `run_plan_only_step` (5113) and `plan_only_checkout` (5169) with a step-id parameter instead of copying them.
   - `the_gate_passes_a_skipped_slow_job_only_when_the_diff_said_so`: extend `run_gate` (4971) with a `slow` output parameter, passed for `${{ needs.diff.outputs.slow }}`. Skipped `slow` passes only when `diff` succeeded and the output is `false`; skipped with an empty output, a failed or a cancelled `slow` fails the gate, and the log says why. Existing callers of `run_gate` pass `""`.
   - `the_archive_jobs_build_the_test_binaries_without_debug_information`: the set of builders (line 1801) also holds every job whose lane is `test-slow`.

## Steps

0. Measure first, once, and put the numbers in the report. Before any change: `SLOT_TIMEOUT=30m cargo nextest run --profile ci -p ess-synth --test adversary_e_u2_bytes --test adversary_e_u6_bytes --run-ignored only` and the same for `-p ess-conformance --test adversary_244b_window_free_bytes --test adversary_w3_266_267_dump --test adversary_287_pass1 --run-ignored only -E 'test(/wasm|window_free|dump/)'`. Read each test's time from `target/nextest/ci/junit.xml`. Decision rule: a probe over 600 s is marked all the same, and its time goes into the `<reason>`; the profile's 20-minute kill stays.
1. Write the tests above, run them red.
2. `src/slow_lane.rs`. Scan `crates/<area>/<crate>/tests/**/*.rs`, skipping any directory named `fixtures`, with `std::fs`; parse each file with `syn::parse_file`; visit every `ItemFn` at any module depth; a test is a function with an attribute whose path ends in `test`; a marker is an `ignore` attribute with `= "slow lane: <reason>"` on a test. The package is the first `name = "..."` after `[package]` in the crate's `Cargo.toml`. The target is the first path component below `tests/` (the file stem, or the directory name). `Args` (clap, `#[derive(Debug, clap::Args)]`): no flag prints one `<package>\t<target>\t<test>` line per marked test, sorted; `--packages` prints the package names, one per line; `--package <P> --targets` prints that package's targets, one per line; `--package <P> --filter` prints one nextest filterset, `test(/(^|::)(<name1>|<name2>)$/)`, so a test that moves into a module of a merged binary (`story:one-integration-test-binary-per-large-crate`) is still found. Refuse as `the_scanner_refuses_a_marker_it_cannot_run` says. An empty tree is not an error for the scanner; `test-slow` refuses an empty lane (step 5).
3. `.github/workflows/ci.yml`, job `diff`: add output `slow: ${{ steps.slow.outputs.slow }}` (line 69-70) and a step `slow` after the `plan-only` step (after line 127), with the same `if` as every step of the job (`${{ env.PLAN_ONLY_CANDIDATE == 'true' }}`), `shell: bash`, no `env`. Its script follows the `plan-only` step: two arrays at the top, `skippable` and `needed`, then `git diff --no-renames --name-only -z "HEAD^1" HEAD`. The step prints `slow=false` when every changed path matches a pattern of `skippable` and none of `needed`, and `slow=true` on any other path, on an empty list, and on any error (a failure to read the history decides `true`). Bash patterns, `*` crosses `/`:
   - `skippable`: `.engineering/?*`, `docs/*`, `website/*`, `*.md`, `release-notes/*`, `crates/ui/*`, `crates/infra/*`, `crates/edge/ess-xtask/*`.
   - `needed` (checked first): `.engineering/repro/*`, `website/docs/start/*`, `website/docs/getting-started.md`, `crates/edge/ess-xtask/src/slow_lane.rs`.
   - Anything else, `.github/*`, `Taskfile.yml`, `.config/*`, `Cargo.lock`, `crates/specify/*`, `crates/generate/*`, `crates/verify/*`, `crates/edge/ess-cli/*`, `examples/*`, `models/*`, `schemas/*`, runs the lane.
   `ci_lanes.rs` expectations for the history test: `docs/x.md` only gives `false`; `AGENTS.md` only gives `false`; `crates/ui/a.rs` plus `docs/x.md` gives `false`; `crates/verify/a.rs` gives `true`; `.engineering/repro/501/system.yaml` gives `true`; `website/docs/start/install.md` gives `true` and `website/docs/other.md` gives `false`; `.github/workflows/ci.yml` gives `true`; one skippable and one other path gives `true`; no changed path gives `true`; a depth-1 checkout (history unreadable) gives `true`.
4. `ci.yml`, new job `slow` placed after `test` (before `transport-network`): `name: Slow lane`, `needs: [diff]`, `if: ${{ needs.diff.outputs.slow != 'false' }}`, `runs-on: ubuntu-latest`, `timeout-minutes: 30`, `env: { CARGO_PROFILE_DEV_DEBUG: "0", CARGO_PROFILE_TEST_DEBUG: "0" }`. Its steps are the `test` job's setup steps, copied byte for byte so the pins that read them stay true: checkout (461-465), pinned rustc (517-526), `dtolnay/rust-toolchain` with the wasm target (528-532), `actions/setup-go` 1.25.10 (536-539), `Install go-task` (541-553), `actions/setup-node` 22 (555-560), the TypeScript toolchain (562-573), `taiki-e/install-action` nextest (575-577), `Swatinem/rust-cache` with `key: slow`, `cache-targets: "false"` omitted, and the same `save-if` (580-584 shows the form; `compile_caches_are_saved_only_where_a_later_run_restores_them` pins `save-if`), `cargo fetch --locked` as its own line (587-588), then `- name: Slow lane` running `task test-slow`. No disk-watch steps and no archive downloads: the lane compiles its own targets.
5. `Gate` (772-817): add `slow` to `needs` (line 774), an env entry `SLOW: ${{ needs.slow.result }}` and `SLOW_SKIPPABLE: ${{ needs.diff.outputs.slow }}`, `"slow=$SLOW"` to the `for job in` list (line 799), and in the `skipped` branch (806-813) a case `slow)` that continues only when `[ "$DIFF" = success ] && [ "$SLOW_SKIPPABLE" = false ]` and prints `the slow job was skipped because no changed path can reach a slow test`. Update the comment above `if:` (775-777).
6. `.config/nextest.toml`, after line 24, without comments: `[profile.slow]` with `slow-timeout = { period = "120s", terminate-after = 10 }` and `final-status-level = "slow"`, then `[profile.slow.junit]` with `path = "junit.xml"`.
7. `Taskfile.yml`:
   - `test-slowest`: the report line becomes `report="${SLOWEST_REPORT:-target/nextest/ci/junit.xml}"`; the rest is unchanged.
   - New task `test-slow`, `desc` one sentence, one shell command (POSIX sh):
     `set -eu`, `status=0`, `wanted="{{.PACKAGES}}"`, a guard that exits 1 with `test-slow: no test is marked #[ignore = "slow lane: ..."]` when `cargo xtask slow-list --packages` prints nothing, then `for package in $(cargo xtask slow-list --packages)`; inside, skip a package not in `$wanted` when `$wanted` is non-empty; build `targets` from `cargo xtask slow-list --package "$package" --targets` as `--test <t>` words; run `cargo nextest run --locked --profile slow --run-ignored only --no-fail-fast --package "$package" $targets -E "$(cargo xtask slow-list --package "$package" --filter)" || status=1`; then `SLOWEST_REPORT=target/nextest/slow/junit.xml task test-slowest`; after the loop `exit "$status"`. One nextest run per package, because `cargo` may refuse a `--test` name that exists in only one of several selected packages (not verified, see Decisions).
   - `check` (638-657): `- task: test-slow` after `- task: test` (line 642).
   - Delete `test-slow-probes` (325-329).
   - New lines in this file carry no comments.
8. Rename the six markers (the lines in `## Where`) to `#[ignore = "slow lane: <reason>"]`, the reason in a few words naming what the probe does (for example `byte identity of every model's generated Rust and Go`); keep `#[test]` and the function. Add the six rows to `MARKED`. Rewrite the doc comment at `external_beside_held_guard.rs:1090` to say `#[ignore = "slow lane: ..."]` in `task test-slow` instead of `task test-slow-probes`.
9. `release.yml` comment at 108-130: add after line 116 "A run qualifies only if its `slow` job ran: `Gate` fails a skipped `slow` job except on a run that cannot qualify." No code change.
10. `AGENTS.md` lines 220-222: `task test-browser-product` stays as it is; replace `` `task test-slow-probes` run in no CI lane (those Firefox tests and slow probes are ignored in the CI test shards) `` so the paragraph says: `task test-browser-product` runs in no CI lane; the slow lane, `task test-slow` and the CI job `slow`, runs every test marked `#[ignore = "slow lane: <reason>"]` (`cargo xtask slow-list` names them); a pull request runs it unless every changed path is in the `skippable` list of the `diff` job's `slow` step; the `main` push, the nightly run, a `queue/**` head, a merge group and the release gate always run it, and `Gate` carries it. Leave the rest of the paragraph and every other paragraph to t7.
11. `CHANGELOG.md`: the bullet under `## [Unreleased]` `### Changed`, placed directly under the existing first bullet.
12. Run `## Gate`. Then `SLOT_TIMEOUT=30m task test-slow` once, locally, and report its wall time and each probe's time.

## Owned files

`crates/edge/ess-xtask/src/slow_lane.rs`, `crates/edge/ess-xtask/tests/slow_lane.rs`, `.config/nextest.toml` (`[profile.slow]`), the `test-slow` and `test-slowest` tasks and the `check` and `test-slow-probes` lines of `Taskfile.yml`, the `diff` job's `slow` step and the `slow` job and `Gate` of `ci.yml`, the six probe files (marker lines only), `external_beside_held_guard.rs:1090`, and the `test_budget.rs` functions named above.

## Not yours

- `crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs`, `synthesis_suite_bytes_table_cli_read.rs`, `enum_presence_guard.rs` (t2), `crates/edge/ess-cli/tests/tutorial_page.rs` and the `ESS_TUTORIAL_NETWORK` lines of `ci.yml` (644) and of `AGENTS.md` 144-146 (t3), `generated_view_queries.rs` (t4), the three adversary files (t5), `test-xtask` (t6), `terminate-after`, the `timeout` wrapper, `test-packages`, the `AGENTS.md` § Gate lists and `release.yml` code (t7).
- `ci.yml`'s `plan-only` step and the ruleset: unchanged. The `slow` step is a second step beside it.
- Shared with later units: `Taskfile.yml`, `ci.yml`, `ci_lanes.rs`, `tests/slow_lane.rs` (`MARKED`), `CHANGELOG.md`, `AGENTS.md`. This unit lands first. A later unit that edits the same file keeps both sides at merge: rows of `MARKED` are one per line, sorted by path; the `CHANGELOG.md` bullets sit one under the other; `ci_lanes.rs` additions are new functions at the end of the file.

## Gate

Bounds: no test over 60 s, the test step within 120 s, a pool job ends after 10 minutes unless it names its time (`SLOT_TIMEOUT=30m`, 30 at most). No Python for anything that stays in the repository: the scanner is Rust and the lane's driver is POSIX sh inside a Taskfile command, as `test-slowest` is.

```console
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-xtask --locked --test slow_lane --test test_budget --test ci_lanes
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-xtask --all-targets --locked -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-conformance -p ess-synth --locked --test adversary_244b_window_free_bytes --test adversary_w3_266_267_dump --test adversary_287_pass1 --test external_beside_held_guard --test adversary_e_u2_bytes --test adversary_e_u6_bytes -- -D warnings
slot task fmt-check
slot task ci-lint
SLOT_TIMEOUT=30m task test-slow
```

The `ess-xtask` tests run as named `--test` targets; `task test-xtask` is not needed for this unit (no `ValidationCode`, diagnostic, format version, `website/` or `docs/` change). Record the time of `--test ci_lanes` before and after.

## Acceptance

1. `cargo xtask slow-list` prints exactly the six probe rows, and `cargo test -p ess-xtask --test slow_lane` is green; the marker scan and `MARKED` agree.
2. `task test-slow` runs the six probes (two in `ess-synth`, four in `ess-conformance` over `adversary_244b_window_free_bytes`, `adversary_w3_266_267_dump` and `adversary_287_pass1`), prints the ten slowest tests of each package from `target/nextest/slow/junit.xml`, and exits non-zero when any fails. With the six markers removed it exits 1 with `no test is marked`.
3. `task test-slow-probes` does not exist; `grep -rn "test-slow-probes" Taskfile.yml AGENTS.md crates` finds nothing.
4. `ci.yml` has a job `slow` that `Gate` needs and reads; `cargo test -p ess-xtask --test ci_lanes` fails when the job loses its `if`, its `timeout-minutes: 30`, its `task test-slow` step, or its place in `Gate`'s `needs`.
5. `ci_lanes.rs` fails when a `main` push, the nightly run, a `queue/**` head, a merge group or a `release.yml` call can skip `slow`, and when a run that skips it can qualify in `prior-gate` (`no_run_that_stands_in_for_a_release_can_skip_the_slow_lane`).
6. `Gate` passes a skipped `slow` only on the decision `false` of a successful `diff` (`the_gate_passes_a_skipped_slow_job_only_when_the_diff_said_so`).
7. The `slow` step's decision matches the table in step 3 on real Git histories (`the_slow_decision_is_every_changed_path_in_the_skippable_list`).
8. `cargo test -p ess-xtask --test test_budget` is green with `test-slow` in the nextest tasks and the `slow` profile pinned.
9. `ci.yml`'s `plan-only` step, `PLAN_ONLY_IF` and the three plan-only skips are byte-identical to before (`git diff` shows only added lines in the `diff` job and in `Gate`).

## Decisions

- Names every other unit repeats (fixed here, not reopened): marker `#[ignore = "slow lane: <reason>"]` on a `#[test]` in a `crates/<area>/<crate>/tests/**.rs` file; command `cargo xtask slow-list`; task `test-slow` with the optional variable `PACKAGES`; nextest profile `slow`; CI job id `slow`, name `Slow lane`; `diff` job output `slow` (`true`, `false` or empty); table `MARKED` in `crates/edge/ess-xtask/tests/slow_lane.rs`; `SLOWEST_REPORT` for `test-slowest`.
- A marker, not a tag file or a name list: the marker sits on the test, `syn` reads it, `MARKED` pins it, and nextest's own `--run-ignored only` runs it. Reason: one source of truth that moves with the function when `story:one-integration-test-binary-per-large-crate` merges binaries.
- The filter is `package` plus a test-name regex, not `binary_id`, so that story's merge of binaries needs no change here. That story lands after this wave (Decision in the map comments).
- A skipped slow lane never counts for a release. It is structural: `prior-gate` accepts only a push or schedule run of `main` and a pull-request run of a `queue/` head (release.yml 153-168), and the `slow` step decides only for `PLAN_ONLY_CANDIDATE` runs, which are neither. A merge group never qualifies and runs the lane anyway. The full regeneration therefore runs in every run that can count for a release, and in the release's own gate.
- The skip list is fail-safe: an unknown path, an unreadable history or an empty diff runs the lane. Reason: a missed regression is worse than 20 runner-minutes.
- The slow profile kills a test at 20 minutes (`period` 120 s, `terminate-after` 10) and the job at 30; the test step's 60 s bound does not apply to the lane. The `timeout-minutes: 30` is held by `ci_lanes.rs` line 1258 for every job but `test`.
- `check` runs `test-slow` so that the local gate equals the forge. It is a long local step; the `AGENTS.md` § Gate rule is to run the whole `task check` only to reproduce a CI lane.
- One nextest run per package, in a shell loop. Reason: whether `cargo nextest run -p A -p B --test x` accepts a target that only one package holds was not verified by reading; a loop does not depend on it. Revisit only with a measurement.
- Hotfix tags: unchanged here. `release.yml`'s `Hotfix tests` job runs `task test-packages`; t7 makes that task run `test-slow` for the selected packages.
- D3 (the feature-off run moves to `slow`) is not in this unit or in the wave's seven units; the report names it as a follow-up story.
- No `RUSTC_WRAPPER`, `SCCACHE_*` or `-j` appears in the new task or job.

## Changelog

`### Changed`, one bullet:

- CI: a test marked `#[ignore = "slow lane: <reason>"]` runs in `task test-slow` and in a `slow` job beside the test shards, which `Gate` carries; a pull request skips the job only when every changed path is a plan, documentation, UI, infrastructure or `ess-xtask` path, and every other run (`main`, nightly, queue, merge group, release) runs it. The six byte-identity and WebAssembly probes that `task test-slow-probes` ran only by hand now run there, and that task is gone.

Split on 2026-11-23 after round 1 of the plan critics (design finding: one unit held the local mechanism and the CI wiring): the mechanism is `story:a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it`, the CI wiring `story:a-slow-ci-job-runs-task-test-slow-on-every-run-that-can-count-for-a-release-and-gate-carries-it`.
