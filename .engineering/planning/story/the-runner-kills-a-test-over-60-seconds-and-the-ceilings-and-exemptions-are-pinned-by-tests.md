---
format: aep.planning-md/3
id: story:the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests
kind: story
status: draft
title: The runner kills a test over 60 seconds and a test task over its listed step ceiling, and the ceilings, the exemptions and the lanes are written in AGENTS.md and pinned by tests
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
- depends_on: story:the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane
- depends_on: story:the-tutorial-walkthrough-runs-only-the-ess-commands-in-the-test-step-and-npm-and-go-in-the-slow-lane
- depends_on: story:the-generated-view-queries-target-keeps-generation-in-the-test-step-and-compiles-in-the-slow-lane
- depends_on: story:three-adversary-targets-run-a-reduced-sweep-in-the-test-step-and-the-full-sweep-in-the-slow-lane
- depends_on: story:the-ess-xtask-unit-tests-build-through-the-host-compile-cache-and-run-within-the-120-second-step
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
- confidence: cited
  path: crates/edge/ess-xtask/tests/ci_lanes.rs
- confidence: cited
  path: crates/edge/ess-xtask/tests/test_budget.rs
revision: 9
---
## Outcome

The test budget is enforced by the runner and held down by a ratchet.

- Nextest kills a test of the `ci` profile at 60 s and counts it failed (`terminate-after = 1`). The tests measured over 60 s that stay in the step are exempt by name at up to 300 s, each exemption listed in `AGENTS.md`.
- Each test task of the step (`test-shard`, `test-feature-off`, `test-xtask`, `test-doc`) runs under `timeout --kill-after=30 <ceiling>`, exits 124 with a message that names the task and the ceiling, and the ceiling is a number in `AGENTS.md` § Gate. A test in `ci_lanes.rs` holds the Taskfile and `AGENTS.md` to one table, so raising a ceiling is a visible edit.
- Every shard uploads its JUnit report as an artifact, so per-test times of a CI run are readable.
- A hotfix release's targeted gate runs the slow tests of the selected crates.
- `AGENTS.md` § Gate describes the slow lane, the ceilings and the exemptions.

## Why

Parent: `story:the-test-step-meets-the-test-budget`, steps 1 to 3 and 6 (the runner enforces the budget; the step fails when over its ceiling; the ceilings ratchet down) and the slow lane's description. Rule: "Tests stay fast", "The runner enforces the budget: a test over 60 s is killed and counts as failed ... and the step itself fails when it runs over 120 s".
The ceilings start at today's measured times (a shard is about 25 minutes today), so the step passes now and cannot get slower; lowering them to 120 s is the story's end state, one follow-up per crate (see the map).
Consumer gap: none.

## Where

Read at `bf128f9146` (unchanged in `ff7f8e74fb`) before t1 to t6 landed; re-read every line, t1 to t6 have moved some. Everything below is after t1 to t6 are merged.

- `.config/nextest.toml`: `[profile.ci]` 19-21 (`slow-timeout = { period = "60s" }`, `final-status-level = "slow"`), `[profile.ci.junit]` 23-24, `[profile.slow]` and `[profile.slow.junit]` (t1, after them). The `[[profile.default.overrides]]` at 8-16 are the model for the override syntax.
- `Taskfile.yml`: `test-shard` (335-358; the nextest run is its last command, preceded by `defer: { task: test-slowest }`), `test-feature-off` 371-380, `test-doc` 366-369, `test-xtask` (404-418 before t6; two commands after it), `test-packages` 104-137 (defer at 124; the nextest command and the `ess-xtask` case at 125-137), `test-slow` and `test-slowest` (t1).
- `.github/workflows/ci.yml`: job `test` (429-673): `Workspace tests ${{ matrix.group }} ${{ matrix.part }}` step 641-645 (`task test-shard GROUP=... PART=...`), `Disk after the workspace tests` 647-, `Feature-off tests ${{ matrix.shard }}` 664-666, `Disk after the feature-off tests` 668-. `actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1` is used at 328 and 421 (copy the `uses:` line). Job `checks` (129-247) runs `task test-xtask` at 243.
- `.github/workflows/release.yml`: job `hotfix-tests` 388-457 (steps: checkout, pinned rustc, rust-toolchain with the wasm target, go-task install, setup-node, nextest, rust-cache key `test-workspace`, fetch, `Test the selected crates` running `task test-packages PACKAGES="$PACKAGES"` at 457). It has no `actions/setup-go` and no TypeScript toolchain step.
- `crates/edge/ess-xtask/tests/test_budget.rs`: `NEXTEST_TASKS` 15, `table` 77-90, `the_ci_profile_reports_every_test_over_sixty_seconds` 141-157 (accepts both `{ period = "60s" }` and `{ period = "60s", terminate-after = 1 }` and requires exactly two lines in `[profile.ci]`), the other tests as in t1's `## Where`.
- `crates/edge/ess-xtask/tests/ci_lanes.rs`: `shell_commands` 236, `gated_jobs` 65; append new tests at the end.
- `AGENTS.md`: § Gate (line 63-), the package-gate table 156-164, the byte-pin paragraph 206-218 (t2), the slow-lane sentence at 220-222 (t1).
- `CHANGELOG.md` `## [Unreleased]` line 3, `### Changed` line 5.

## Tests first

Bounds: no test over 60 s, the test step within its ceiling (end state 120 s), a pool job ends after 10 minutes unless it names its time (`SLOT_TIMEOUT=30m`, 30 at most). No Python for anything that stays: the probes are Rust or POSIX sh in the scratch directory and are not committed. A measuring script starts without the `slot` prefix.

0. Measure first.
   - (a) The ceilings. Read the last five completed `main` push or `queue/**` pull-request runs of `ci.yml`: `b10x-gates gh --repo . --repository beyond10x/ess -- run view <run-id> --json jobs` from an empty directory outside every checkout (see `AGENTS.md` § Release), and take the longest duration of each step `Workspace tests ...`, `Feature-off tests ...` and the steps of `checks` that run `test-xtask` and `ci-smoke`'s doc tests. The ceiling of a task is the longest seen, plus 20 %, rounded up to a multiple of 60 s. If the runs cannot be read, use the parent's Need figures: `test-shard` 1500, `test-feature-off` 900, `test-xtask` 300, `test-doc` 300. Write the four numbers and their source in the report.
   - (b) The exemptions. For each test over 60 s still in the step after t2 to t6, find it: the tests named in addition A and the parent's Need list are `adversary_process_target_pass2` (`the_single_pass_reader_reads_generated_answers_as_the_reader_it_replaced`, 329 s measured locally), `aggregate_group_selection`, `adversary_287_pass1`, `nested_response_observations` and `typescript_suite_versions`; find each file with `ls crates/*/*/tests/<name>.rs`, run it under `SLOT_TIMEOUT=30m cargo nextest run --profile ci -p <crate> --test <name>` and read the per-test times from `target/nextest/ci/junit.xml`. An exemption is a test measured over 60 s (not one that was only slow under load: measure twice). Its allowed time is `terminate-after = ceil(seconds * 1.25 / 60)`, at most 5. A test over 300 s is not exempted: it is marked `slow lane` through t1's machinery and its row is added to `MARKED`, which makes this unit touch one attribute line of that file; name it in the report.
   - (c) The kill. A scratch probe, not committed: add the file `crates/edge/ess-xtask/tests/zz_probe.rs` with `#[test] fn sleeps() { std::thread::sleep(std::time::Duration::from_secs(65)); }`, run `SLOT_TIMEOUT=30m cargo nextest run --profile ci -p ess-xtask --test zz_probe`, and see the test `TIMEOUT` and fail near 60 s; delete the file; `git status` shows nothing of it.1. `test_budget.rs`:
   - `the_ci_profile_kills_a_test_after_sixty_seconds`: replaces `the_ci_profile_reports_every_test_over_sixty_seconds`'s acceptance of `{ period = "60s" }` alone: `[profile.ci]` has `slow-timeout = { period = "60s", terminate-after = 1 }` and `final-status-level = "slow"` and nothing else. Red today: `slow-timeout = { period = "60s" }` is the line.
   - `every_ci_override_is_a_listed_exemption_of_at_most_five_minutes`: every `[[profile.ci.overrides]]` table (read with a function like `table` for the header `[[profile.ci.overrides]]`, one table at a time) has exactly `filter = "..."` and `slow-timeout = { period = "60s", terminate-after = N }` with `N` from 2 to 5; and the set of filters equals the set of test names listed in `AGENTS.md` under the line `The exempt tests of the \`ci\` profile:` (one `- \`<name>\`: <seconds> s` line per test, the filter being `test(=<name>)`). Red today: the AGENTS.md list does not exist.
2. `ci_lanes.rs`:
   - `every_test_step_task_runs_under_its_ceiling_and_the_ceilings_are_one_table`: a constant `CEILINGS: [(&str, u32); 4]` with the four numbers of step 0(a); for each task: it has a `vars` entry `STEP_SECONDS` equal to the number; each command that runs `cargo nextest run` or `cargo test` starts with `timeout --kill-after=30 {{.STEP_SECONDS}} ` (for `test-xtask`, both of its commands); the command ends in a handler that prints a line containing the task name and `over its step budget` and exits with the status when the status is 124; and `AGENTS.md` has the line `` - `<task>`: <n> s `` under the line `The step ceilings of the test tasks:`. Red today: no `STEP_SECONDS`.
   - `the_shards_upload_their_junit_reports`: in job `test`, a step with `uses` `actions/upload-artifact@...` (the same SHA as the build jobs' upload, compared with the one at `ci.yml:328`), `if: ${{ always() }}`, `path: target/nextest/ci/junit.xml`, comes directly after `Workspace tests ...` and another directly after `Feature-off tests ...`, with `name` `junit-workspace-${{ matrix.group }}-${{ matrix.part }}` and `junit-feature-off-${{ matrix.shard }}`. Red today: none.
   - `the_hotfix_tests_job_runs_the_slow_tests_with_the_toolchains_they_need`: `release.yml`'s `hotfix-tests` job has an `actions/setup-go` step with `go-version: 1.25.10` (the same `uses` line and version as in job `slow` of `ci.yml`, t1) and the same TypeScript toolchain step as job `slow`, and `Taskfile.yml`'s `test-packages` calls `task: test-slow` with `vars: { PACKAGES: '{{.PACKAGES}}' }` after its nextest command. Red today: none of them.
3. Run each red once, then implement.

## Steps

1. Measure (step 0 of `## Tests first`), keep the numbers.
2. `.config/nextest.toml`, no comments in what you add: `[profile.ci]` line 1 becomes `slow-timeout = { period = "60s", terminate-after = 1 }`. After `[profile.ci.junit]`, one table per exemption:
   ```
   [[profile.ci.overrides]]
   filter = "test(=<name>)"
   slow-timeout = { period = "60s", terminate-after = <N> }
   ```
   (`terminate-after` means the test is killed after N periods, that is N times 60 s.)
3. `Taskfile.yml`, one task at a time: `test-shard`, `test-feature-off`, `test-xtask` (both commands), `test-doc`: add `vars: STEP_SECONDS: '<ceiling>'` (merge into an existing `vars:` of the task; `test-shard` has one), and prefix the cargo command with `timeout --kill-after=30 {{.STEP_SECONDS}} `; follow with `|| { code=$?; if [ "$code" -eq 124 ]; then echo "<task>: over its step budget of {{.STEP_SECONDS}} s" >&2; fi; exit "$code"; }` (a command that was a single line becomes a block scalar `|` or a folded `>-` command; keep `defer: { task: test-slowest }` where it is). `test-packages` is not wrapped (a hotfix gate; its job has 60 minutes). `test-slow` is not wrapped (the profile kills a test at 20 minutes and the job is bound to 30). Locally, a task whose ceiling is over 10 minutes needs `slot --timeout` or `SLOT_TIMEOUT=30m` to run through the pool; say so in `AGENTS.md`, step 6.
4. `test-packages`: after the nextest command (line 125-130) and before the `ess-xtask` case, add `- task: test-slow` with `vars: { PACKAGES: '{{.PACKAGES}}' }`. The `defer` of `test-slowest` stays above: `test-slow` prints its own slowest tests through `SLOWEST_REPORT`.
5. `ci.yml` job `test`: after `Workspace tests ...` add the upload step; after `Feature-off tests ...` add the second. Both with `if: ${{ always() }}`, `retention-days: 7`, `if-no-files-found: ignore`, `name` as in step 2 of `## Tests first`. Order matters: the feature-off run overwrites the same report path, so the first upload must come before it. `ci_lanes.rs` pins that steps of the job keep their order if it reads it (`the_archive_jobs...`); run the whole `ci_lanes` binary.
6. `release.yml` job `hotfix-tests`: after the `actions/setup-node` step add the `actions/setup-go` step (the exact `uses` line, `with: go-version: "1.25.10"`, as job `slow` and job `test` have it) and the TypeScript toolchain step (copy byte for byte from job `slow`, which t1 copied from `test`). No other change to `release.yml`: `prior-gate` and its comment stay as t1 left them.
7. `AGENTS.md` § Gate:
   - a paragraph after the one that ends `Every push also runs \`task fmt-check\`...` (line 166-167): "**The test step has a budget.**" saying: nextest kills a test of the `ci` profile at 60 s; each test task runs under `timeout` at its ceiling and fails over it; the ceilings only go down and are raised only by a story; the sum is the end state of 120 s; JUnit reports of every shard are uploaded as artifacts (`junit-workspace-<group>-<part>`, `junit-feature-off-<shard>`, 7 days). Then the line `The step ceilings of the test tasks:` followed by one line per task, `` - `test-shard`: <n> s `` and so on for the four tasks. Then the line `The exempt tests of the \`ci\` profile:` followed by one line per exempt test, `` - `<name>`: <seconds> s ``; each exemption is a defect and a follow-up story; the list shrinks.
   - a paragraph "**The slow lane**" (t1 wrote one sentence at 220-222; do not duplicate it, point to it) only if t1's sentence leaves out the hotfix gate: add "The hotfix gate's `task test-packages` runs `task test-slow` for the selected crates."
   - the package-gate table (156-164): add two rows. "a test marked `#[ignore = \"slow lane: ...\"]` or the file that holds it | that crate's slow tests: `task test-slow PACKAGES=<crate>` (`cargo xtask slow-list` names them)"; and "`src/` of `ess-conformance`, `ess-gen`, `ess-transport`, `ess-http`, `ess-compiler`, `ess-domain`, `ess-primitives` or `ess-yaml`, or any model file | also `synthesis_suite_bytes_table`, re-stamping `tests/fixtures/synthesis-sources.digest` with the command it prints". Do not edit other rows.
8. `CHANGELOG.md`: the bullet below.
9. Run `## Gate`. Then `SLOT_TIMEOUT=30m task test-xtask`, and `SLOT_TIMEOUT=30m task test-packages PACKAGES="ess-xtask"` once (the second proves the `test-slow` call with an `ess-xtask` selection finds no marked test of that package and exits 0).
10. Measure and report the wall time of `task test-slow` (all packages) and name the longest package; if it is over 20 minutes, report it as a finding for a matrix split of the `slow` job (a follow-up, not made now).

## Owned files

`.config/nextest.toml` (`[profile.ci]` and its overrides), the `STEP_SECONDS` vars and `timeout` prefixes of the four test tasks and the `test-slow` call of `test-packages` in `Taskfile.yml`, the two upload steps in job `test` of `ci.yml`, the two setup steps in `hotfix-tests` of `release.yml`, the new tests of `test_budget.rs` and `ci_lanes.rs`, `AGENTS.md` § Gate (the new paragraphs and two table rows).

## Not yours

- t1 owns the lane machinery, the `slow` job and its `Gate` edits, `test-slow` and `test-slowest`, `[profile.slow]`, `AGENTS.md` 220-222, `release.yml`'s comment, and the `test_budget.rs` and `ci_lanes.rs` tests it names. t2 to t6 own their test files, `AGENTS.md` 144-146 (t3) and 206-218 (t2), `test-xtask`'s two-command form (t6). The planning-only path filter (`PLAN_ONLY_*`, the `plan-only` step) and the ruleset are unchanged.
- The per-crate follow-ups (test files that spawn node, tsc, go, cargo, rustc or npm: `ess-conformance` 100, `ess-synth` 52, `ess-cli` 20) and D3 (feature-off tests to the slow lane): reported, not done.
- Shared: this unit lands last, after t1 to t6. At merge: `Taskfile.yml` (different tasks; take `test-xtask` as t6 left it and add the wrapper to both commands), `ci.yml` (different jobs), `ci_lanes.rs` and `test_budget.rs` (append; keep every test of the earlier units), `AGENTS.md` (different paragraphs), `CHANGELOG.md` (keep all bullets).

## Gate

```console
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-xtask --locked --test test_budget --test ci_lanes --test slow_lane
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-xtask --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m task test-xtask
slot task ci-lint
slot task fmt-check
```

`Taskfile.yml`, workflows and `AGENTS.md` changed: `task ci-lint` and `task test-xtask` run (table rows "a workflow, `Taskfile.yml` or a public API" and "`CHANGELOG.md`, `website/` or `docs/`"). The exempt tests are measured in step 0, not run again as a gate. The whole `task check` is not run locally.

## Acceptance

1. `cargo test -p ess-xtask --test test_budget --test ci_lanes` is green with the four new tests and `the_ci_profile_kills_a_test_after_sixty_seconds` red before the edit.
2. The scratch probe of step 0(c) is killed at about 60 s and reported failed by nextest; the probe file is not in `git status`.
3. `task test-shard`, `test-feature-off`, `test-xtask` and `test-doc` each begin their cargo command with `timeout --kill-after=30 <ceiling>`; a ceiling in `Taskfile.yml`, in `CEILINGS` and in `AGENTS.md` is one number (`every_test_step_task_runs_under_its_ceiling_and_the_ceilings_are_one_table`). Check the exit path once by hand with a 5 s ceiling on a scratch copy of `test-doc` or by `timeout 2 sleep 5; echo $?` printing 124 through the handler form, and name the result.
4. Every exemption of `.config/nextest.toml` is in the `AGENTS.md` list with its measured time, and each listed test exists (`cargo nextest list -p <crate> --test <name>` names it).
5. Every shard job has the two upload steps (`the_shards_upload_their_junit_reports`).
6. `the_hotfix_tests_job_runs_the_slow_tests_with_the_toolchains_they_need` is green; `task test-packages PACKAGES=ess-xtask` ends 0.
7. The report names: the four ceilings and their source, each exemption with its seconds, and the wall time of `task test-slow`.

## Decisions

- Names (t1, repeated): marker `#[ignore = "slow lane: <reason>"]`; `cargo xtask slow-list`; `task test-slow` with `PACKAGES`; profile `slow`; job `slow`, `Slow lane`; diff-step output `slow`; table `MARKED`; `SLOWEST_REPORT`. This unit adds: `STEP_SECONDS`, `CEILINGS`, the AGENTS.md lines `The step ceilings of the test tasks:` and `The exempt tests of the \`ci\` profile:`, the artifacts `junit-workspace-<group>-<part>` and `junit-feature-off-<shard>`.
- A skipped slow lane never counts for a release: `prior-gate` qualifies only runs that cannot skip it (t1's `no_run_that_stands_in_for_a_release_can_skip_the_slow_lane`); this unit adds nothing to it. The full regeneration is in every run that can count.
- Ceilings start at today's measured step times, not at 120 s, because the step is 25 minutes today; a ratchet that fails a green tree would be reverted. The end state is 120 s and each per-crate story lowers a number. Raising one is an edit of `CEILINGS`, the Taskfile and `AGENTS.md` together, which a reviewer sees.
- The kill wrapper is coreutils `timeout --kill-after=30`: no new tool, no Python; exit 124 is a failure of the step, with a message naming the task.
- `terminate-after = 1` in the profile and exemptions by name at up to 5 periods (300 s): a test over 300 s is moved to the slow lane, not exempted.
- `test-xtask` is wrapped per command (compile, then run), each at the task's ceiling: a looser bound than one for the sum, accepted for simplicity.
- `test-packages` and `test-slow` are not wrapped: a hotfix gate with a 60-minute job, and a lane whose own profile and job bound it.
- The hotfix gate runs the selected crates' slow tests, and the job gets `setup-go` and the TypeScript toolchain so they do not fail for a missing tool; this lengthens a hotfix gate by the slow tests of the touched crates only.
- The JUnit upload is `if: always()` with `if-no-files-found: ignore`: a killed shard still leaves its report, and a shard that never ran nextest does not fail on the upload.
- The matrix split of the `slow` job is not made now: measure first (step 10).
- No new dependency, no comments in `nextest.toml` or `Taskfile.yml` lines you add.

## Changelog

`### Changed`:

- CI: nextest kills a test over 60 s, each test task fails over its listed step ceiling, every shard uploads its JUnit report, and the hotfix gate runs the slow tests of the selected crates.
