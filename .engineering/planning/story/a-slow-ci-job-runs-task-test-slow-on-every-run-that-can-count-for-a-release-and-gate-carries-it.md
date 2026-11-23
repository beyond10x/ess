---
format: aep.planning-md/3
id: story:a-slow-ci-job-runs-task-test-slow-on-every-run-that-can-count-for-a-release-and-gate-carries-it
kind: story
status: active
title: A slow CI job runs `task test-slow` on every run that can count for a release and `Gate` carries it
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
- depends_on: story:a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it
- supersedes: story:a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries
scope:
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
- confidence: cited
  path: crates/edge/ess-xtask/tests/ci_lanes.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:02:45Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-23T06:02:47Z", actor: "human:timo", revision: 9}
---
## Outcome

The slow lane (`task test-slow`, built by t1a) runs in CI and counts.

- `ci.yml` has a job `slow` (`Slow lane`). It runs on a `main` push, the nightly run, a `queue/**` head, a merge group and a call from `release.yml`, always. On a pull request from an ordinary head or a fork it runs unless every changed path is in a list of paths no slow test reads.
- `Gate` carries `slow`. A skipped `slow` passes only when the `diff` job decided that nothing a slow test reads changed.
- A run that skipped the slow lane never counts for a release: `prior-gate` accepts only runs that cannot skip it. A test pins this; no logic of `release.yml` changes.
- `task check` runs `test-slow` after `test`, so the local gate equals the forge.

## Why

Parent: `story:the-test-step-meets-the-test-budget`, design step 5 and decision D1 (b)/D3 (b): a `slow` job exists, only when the sources changed on a pull request, and always where a run can stand in for a release. Rules: "Tests stay fast" and "The local gate equals the forge" (the lane's CI job and `task check` run the same task).
Consumer gap: none. Units t2 to t5 need only t1a; this unit is built beside them.

## Where

Read at `80794bbd698`; the `ci.yml`, `release.yml`, `Taskfile.yml` and `ci_lanes.rs` lines below are unchanged since `bf128f9146`. Re-read a line before editing it.

- `.github/workflows/ci.yml`: `on:` 21-36; job `diff` 63-127 (outputs 69-70, the `plan-only` step 81-127, whose `if` is `${{ env.PLAN_ONLY_CANDIDATE == 'true' }}`); `checks` 129-247; `build-tests` 249; `build-feature-off` 337; job `test` 429-673 (`timeout-minutes: 285` at 434; setup steps 461-588 are the model for `slow`; `Workspace tests` 641-645); `transport-network` 675; `msrv` 727; job `gate` 772-817 (`needs` 774, the comment 775-777, the `for job in` list near 799, the `skipped` branch 806-813); `output-ownership-macos` 819.
- `.github/workflows/release.yml`: `prior-gate` step 131-251 with `qualifies()` at 153-168 (a push or schedule run of `main`, or a pull-request run of a `queue/` head of this repository; never an ordinary head, a fork or a merge group); the comment above it 108-130 gains one sentence. No code change.
- `Taskfile.yml`: `check` 638-657 (`- task: test` at about 642).
- `crates/edge/ess-xtask/tests/ci_lanes.rs`: `gated_jobs` 65, `lane_invocations` 134, `every_step_of_task_check_runs_in_some_pull_request_lane` 271, `the_gate_check_aggregates_every_lane_and_cannot_be_skipped` 1206 (`bound` 30 for every job but `test`, line 1258), `the_archive_jobs_build_the_test_binaries_without_debug_information` 1799, `PLAN_ONLY_*` constants 4666-4682 (`PLAN_ONLY_STEP_IF` 4675), `assert_plan_only_skips_exactly_the_test_jobs` 4689 (no job other than the three skipped may carry "plan-only" in its `if`, about line 4713), `plan_only_candidate` 4756, `ci_context` 4773, `prior_gate_qualifies` 4792, the test at 4862 that no run standing in for a release can skip, `run_gate` 4971, `the_gate_passes_skipped_test_jobs_only_for_a_plan_only_diff` 5013, `run_plan_only_step` 5113, `plan_only_checkout` 5169, `the_plan_only_decision_is_every_changed_path_under_the_plan_store` 5203. New tests are appended at the end of the file.
- Pins the new job must satisfy: `gated_jobs` (65) and the 1206 test (`Gate` needs every gated job, every job but `test` has `timeout-minutes: 30`); the go-task install pins (about 1467 and 1661); the cache `save-if` pin (about 2643); the lockfile fetch before the lane (about 4148); `a_plan_only_pull_request_skips_exactly_...` (4726).
- `AGENTS.md` lines 220-222 (t1a rewrote the sentence); `CHANGELOG.md` `### Changed` heading under `## [Unreleased]`.

## Tests first

Bounds as in t1a: wall-time figures are the runner's own, on a warm build at a 1-minute load of 60 or less; no test over 60 s, the test step within 120 s, a pool job ends after 10 minutes unless it names its time (`SLOT_TIMEOUT=30m`, 30 at most); no Python for anything that stays. Time each new test; none over 5 s, name any over 5 s.

All in `crates/edge/ess-xtask/tests/ci_lanes.rs`, appended; a helper that exists is reused or generalized, never copied:
1. `the_slow_job_is_gated_bounded_and_runs_the_slow_task`: job `slow` exists; `needs` is exactly `[diff]`; `if` is exactly `${{ needs.diff.outputs.slow != 'false' }}` and does not contain "plan-only"; `timeout-minutes` is 30; a step runs `task test-slow`; `env.CARGO_PROFILE_DEV_DEBUG` and `env.CARGO_PROFILE_TEST_DEBUG` are `"0"`; `Gate`'s `needs` and its script read `needs.slow.result`. Red today: `ci.yml has no slow job`.
2. `the_slow_decision_may_skip_only_a_pull_request_of_an_ordinary_head`: the `diff` job has a step `slow` whose `if` equals `PLAN_ONLY_STEP_IF`; reuses the `runs` table of the 4862 test (the same tuples and expected `may` flags) through `plan_only_candidate`; `outputs.slow` is `${{ steps.slow.outputs.slow }}`.
3. `no_run_that_stands_in_for_a_release_can_skip_the_slow_lane`: the `described` table of the 4862 test run through `prior_gate_qualifies`; no run that qualifies is one the slow decision may skip.
4. `the_slow_decision_is_every_changed_path_in_the_skippable_list`: runs the `slow` step against real Git histories as 5203 runs `plan-only`; generalize `run_plan_only_step` and `plan_only_checkout` with a step-id parameter. Expected outputs are in step 3 below.
5. `the_gate_passes_a_skipped_slow_job_only_when_the_diff_said_so`: extend `run_gate` with a `slow` output parameter for `${{ needs.diff.outputs.slow }}` (existing callers pass `""`). A skipped `slow` passes only when `diff` succeeded and the output is `false`; skipped with an empty output, failed or cancelled fails the gate and the log says why.
6. `the_archive_jobs_build_the_test_binaries_without_debug_information`: its set of builders (line 1801) also holds every job whose lane is `test-slow`.
7. `the_check_task_runs_the_slow_lane_after_the_tests`: `check`'s steps hold `test-slow` directly after `test`, and `every_step_of_task_check_runs_in_some_pull_request_lane` stays green (the `slow` job runs `task test-slow`).
Run each red once before the implementation.

## Steps

1. Write the tests, run them red.
2. `ci.yml` job `diff`: add output `slow: ${{ steps.slow.outputs.slow }}` and a step `slow` after the `plan-only` step (after line 127), with the same `if` as every step of the job, `shell: bash`, no `env` (`ci_lanes.rs` pins every step of that job to `PLAN_ONLY_STEP_IF`). Its script follows the `plan-only` step: two arrays at the top, `skippable` and `needed`, then `git diff --no-renames --name-only -z "HEAD^1" HEAD`.
3. The step prints `slow=false` when every changed path matches a pattern of `skippable` and none of `needed`, and `slow=true` on any other path, on an empty list, and on any error (a failure to read the history decides `true`). Bash patterns, `*` crosses `/`:
   - `skippable`: `.engineering/?*`, `docs/*`, `website/*`, `*.md`, `release-notes/*`, `crates/ui/*`, `crates/infra/*`, `crates/edge/ess-xtask/*`.
   - `needed`, checked first: `.engineering/repro/*`, `website/docs/start/*`, `website/docs/getting-started.md`, `crates/edge/ess-xtask/src/slow_lane.rs`.
   - Everything else, including `.github/*`, `Taskfile.yml`, `.config/*`, `Cargo.lock`, `crates/specify/*`, `crates/generate/*`, `crates/verify/*`, `crates/edge/ess-cli/*`, `examples/*`, `models/*`, `schemas/*`, runs the lane.
   Expected outputs for test 4: `docs/x.md` only gives `false`; `AGENTS.md` only gives `false`; `crates/ui/a.rs` plus `docs/x.md` gives `false`; `crates/verify/a.rs` gives `true`; `.engineering/repro/501/system.yaml` gives `true`; `website/docs/start/install.md` gives `true` and `website/docs/other.md` gives `false`; `.github/workflows/ci.yml` gives `true`; one skippable and one other path gives `true`; no changed path gives `true`; a depth-1 checkout (unreadable history) gives `true`.
4. `ci.yml`, new job `slow` after `test` (before `transport-network`): `name: Slow lane`, `needs: [diff]`, `if: ${{ needs.diff.outputs.slow != 'false' }}`, `runs-on: ubuntu-latest`, `timeout-minutes: 30`, `env: { CARGO_PROFILE_DEV_DEBUG: "0", CARGO_PROFILE_TEST_DEBUG: "0" }`. Its steps are the `test` job's setup steps, copied byte for byte so the pins that read them stay true: checkout (461-465), the pinned rustc (517-526), `dtolnay/rust-toolchain` with the wasm target (528-532), `actions/setup-go` 1.25.10 (536-539), `Install go-task` (541-553), `actions/setup-node` 22 (555-560), the TypeScript toolchain (562-573), the nextest install (575-577), `Swatinem/rust-cache` with `key: slow` and the same `save-if` (580-584), `cargo fetch --locked` as its own step (587-588), then `- name: Slow lane` running `task test-slow`. No disk-watch steps and no archive downloads: the lane compiles its own targets.
5. `Gate` (772-817): add `slow` to `needs`; env entries `SLOW: ${{ needs.slow.result }}` and `SLOW_SKIPPABLE: ${{ needs.diff.outputs.slow }}`; `"slow=$SLOW"` to the `for job in` list; in the `skipped` branch a case `slow)` that continues only when `[ "$DIFF" = success ] && [ "$SLOW_SKIPPABLE" = false ]` and prints `the slow job was skipped because no changed path can reach a slow test`. Update the comment above `if:`.
6. `Taskfile.yml` `check`: `- task: test-slow` directly after `- task: test`.
7. `release.yml` comment (108-130): one sentence: "A run qualifies only if its `slow` job ran: `Gate` fails a skipped `slow` job except on a run that cannot qualify." No code change.
8. `AGENTS.md` lines 220-222: append to t1a's sentence: the CI job `slow` carries the lane; a pull request runs it unless every changed path is in the `skippable` list of the `diff` job's `slow` step; the `main` push, the nightly run, a `queue/**` head, a merge group and the release gate always run it; `Gate` carries it; a run that skipped it never counts for a release.
9. `CHANGELOG.md` bullet (below).
10. Run `## Gate`. The `slow` job's own wall time is read from its first CI run and reported (a matrix split is a follow-up if it exceeds 20 minutes).

## Owned files

`.github/workflows/ci.yml` (the `diff` job's `slow` step and output, the `slow` job, `Gate`; not the `test` job), `.github/workflows/release.yml` (the comment), `Taskfile.yml` (the `check` line), `crates/edge/ess-xtask/tests/ci_lanes.rs` (the seven tests and the two generalized helpers), `AGENTS.md` (the CI half of the sentence at 220-222), `CHANGELOG.md` (one bullet).

## Not yours

- t1a: the marker, the scanner, `test-slow`, `[profile.slow]`, `test_budget.rs`, the six probes. t3 deletes the `ESS_TUTORIAL_NETWORK` lines in the `test` job (641-645): a different job of `ci.yml`. t7 wraps four test tasks in `Taskfile.yml` and edits `AGENTS.md` § Gate paragraphs, not this one. The planning-only path filter (`PLAN_ONLY_*`, the `plan-only` step) and the ruleset are unchanged; the `slow` step is a second step beside it.
- Shared, all by "keep both sides at merge": `ci_lanes.rs` (appended tests; t3, t6 and the body-cap unit h3 also append there, h3 for its `setup-go` test; this unit's `run_gate` signature change is the only edit of an existing function, and it lands before t3 and h3), `ci.yml` (h3 adds a `setup-go` step to `build-tests`; t3 edits the `test` job; different jobs), `CHANGELOG.md` (every unit adds one bullet).

## Gate

```console
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-xtask --locked --test ci_lanes --test slow_lane --test test_budget
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-xtask --all-targets --locked -- -D warnings
slot task ci-lint
slot task fmt-check
```

A workflow and `Taskfile.yml` changed: `task ci-lint` runs. `task test-xtask` is not needed (no `ValidationCode`, diagnostic, format version, `website/` or `docs/` change). Record the time of `--test ci_lanes` before and after.

## Acceptance

1. `ci.yml` has a job `slow` that `Gate` needs and reads; `cargo test -p ess-xtask --test ci_lanes` fails when the job loses its `if`, its `timeout-minutes: 30`, its `task test-slow` step, or its place in `Gate`'s `needs` (test 1).
2. `ci_lanes.rs` fails when a `main` push, the nightly run, a `queue/**` head, a merge group or a `release.yml` call can skip `slow` (test 2), and when a run that skips it can qualify in `prior-gate` (test 3).
3. `Gate` passes a skipped `slow` only on the decision `false` of a successful `diff` (test 5).
4. The `slow` step's decision matches the table of step 3 on real Git histories (test 4).
5. `task check`'s steps hold `test-slow` after `test` and the lane pin stays green (test 7).
6. `git diff` of `ci.yml` shows the `plan-only` step, `PLAN_ONLY_IF` and the three plan-only skips byte-identical to before (only added lines in the `diff` job and in `Gate`, plus the new job).

## Decisions

- Names (from t1a, repeated): marker `#[ignore = "slow lane: <reason>"]`; `cargo xtask slow-list`; task `test-slow` with `PACKAGES`; profile `slow`; job id `slow`, name `Slow lane`; `diff` output `slow`; table `MARKED`; `SLOWEST_REPORT`.
- A skipped slow lane never counts for a release. It is structural: `prior-gate` accepts only a push or schedule run of `main` and a pull-request run of a `queue/` head (release.yml 153-168), and the `slow` step decides only for `PLAN_ONLY_CANDIDATE` runs, which are neither. A merge group never qualifies and runs the lane anyway. The full regeneration therefore runs in every run that can count for a release, and in the release's own gate.
- The skip list is fail-safe: an unknown path, an unreadable history or an empty diff runs the lane. A missed regression is worse than 20 runner-minutes.
- The trigger is inverted against the parent's wording ("only when the sources changed"): the lane runs unless every changed path is known to be outside what a slow test reads. Reason: a positive list of sources would miss a path nobody listed.
- The 30-minute job bound is the repository's pin for every job but `test`.
- `check` runs `test-slow` so that the local gate equals the forge; it is a long local step, and § Gate says to run the whole `task check` only to reproduce a lane.
- The hotfix targeted gate (`test-packages`, `Hotfix tests` job) does not run the slow lane in this wave: that is a follow-up (X9) on the wave page.
- D3 (the feature-off run moves to `slow`) is not in this unit or in the wave.

## Changelog

`### Changed`, one bullet (keep every unit's bullet at merge):

- CI: a `slow` job runs `task test-slow` beside the test shards and `Gate` carries it; a pull request skips it only when every changed path is a plan, documentation, UI, infrastructure or `ess-xtask` path, and every other run (`main`, nightly, queue, merge group, release) runs it, so a run that skipped it never stands in for a release (story:the-test-step-meets-the-test-budget).
