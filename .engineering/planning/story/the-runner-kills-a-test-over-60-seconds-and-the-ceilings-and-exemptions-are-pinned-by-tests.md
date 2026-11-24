---
format: aep.planning-md/3
id: story:the-runner-kills-a-test-over-60-seconds-and-the-ceilings-and-exemptions-are-pinned-by-tests
kind: story
status: active
title: The runner kills a test over 60 seconds and the ceilings and exemptions are pinned by tests
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
- depends_on: story:a-slow-ci-job-runs-task-test-slow-on-every-run-that-can-count-for-a-release-and-gate-carries-it
- depends_on: story:the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route
- depends_on: story:the-generated-rust-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route
scope:
- confidence: cited
  path: .config/nextest.toml
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-xtask/tests/ci_lanes.rs
- confidence: cited
  path: crates/edge/ess-xtask/tests/slow_lane.rs
- confidence: cited
  path: crates/edge/ess-xtask/tests/test_budget.rs
revision: 18
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:03:02Z", actor: "human:timo", revision: 17, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:03:04Z", actor: "human:timo", revision: 18, decided_on: {"recorded":{"review_outcome":5}}}
---
## Outcome

The test budget is enforced by the runner and held down by a ratchet.

- Nextest kills a test of the `ci` profile at 60 s and counts it failed (`terminate-after = 1`). The tests measured over 60 s that stay in the step are exempt by name at up to 300 s, each exemption listed in `AGENTS.md`.
- Each test task of the step (`test-shard`, `test-feature-off`, `test-xtask`, `test-doc`) runs under `timeout --kill-after=30 <ceiling>`, exits 124 with a message that names the task and the ceiling, and the ceiling is a number in `AGENTS.md` § Gate. A test in `ci_lanes.rs` runs the handler of every task on exit 124 and holds the Taskfile and `AGENTS.md` to one table, so raising a ceiling is a visible edit.
- `AGENTS.md` § Gate describes the ceilings, the exemptions and the slow lane's place in the package-gate table.
- The ceilings are a ratchet set at the longest time seen plus 20 %, and each later story lowers them; this wave does not reach the 120 s test step (follow-up (e) on the wave page).

## Why

Parent: `story:the-test-step-meets-the-test-budget`, steps 1 to 3 and 6 (the runner enforces the budget; the step fails when over its ceiling; the ceilings ratchet down) and the slow lane's description. Rule: "Tests stay fast", "The runner enforces the budget: a test over 60 s is killed and counts as failed ... and the step itself fails when it runs over 120 s".
The ceilings start at today's measured times (a shard is about 25 minutes today), so the step passes now and cannot get slower; lowering them to 120 s is the story's end state, one follow-up per crate (see the map).
Cut from this unit by the coordinator (X9), listed as follow-ups on the wave page: the JUnit upload steps of the shards, and the hotfix gate running `task test-slow` with its `setup-go` and TypeScript steps. Neither is in the parent.
Consumer gap: none.

## Where

Read at `80794bbd698` (unchanged since `bf128f9146`) before t1a to t6 landed; re-read every line, they have moved some. Everything below is after t1a, t1b, t2 to t6, h2 and h3 are merged.

- `.config/nextest.toml`: `[profile.ci]` 19-21 (`slow-timeout = { period = "60s" }`, `final-status-level = "slow"`), `[profile.ci.junit]` 23-24, `[profile.slow]` and `[profile.slow.junit]` (t1a, after them). The `[[profile.default.overrides]]` at 8-16 are the model for the override syntax.
- `Taskfile.yml`: `test-shard` (335-358; the nextest run is its last command, preceded by `defer: { task: test-slowest }`), `test-feature-off` 371-380, `test-doc` 366-369, `test-xtask` (404-418 before t6; two commands after it). h2 and h3 add `- task:` lines and `env:` entries to `test`, `test-packages` and `test-shard`, and land before this unit: take their lines as they are and wrap only the cargo commands. `test-packages` and `test-slow` are not edited by this unit.
- `.github/workflows/ci.yml`: job `checks` (129-247) runs `task test-xtask` at 243; job `test` runs `task test-shard` and `task test-feature-off` (read only; not edited).
- `crates/edge/ess-xtask/tests/test_budget.rs`: `NEXTEST_TASKS` 15, `table` 77-90, `the_ci_profile_reports_every_test_over_sixty_seconds` 141-157 (accepts both `{ period = "60s" }` and `{ period = "60s", terminate-after = 1 }` and requires exactly two lines in `[profile.ci]`), the other tests as in t1a's `## Where`.
- `crates/edge/ess-xtask/tests/ci_lanes.rs`: `shell_commands` 236, `gated_jobs` 65; append new tests at the end.
- `AGENTS.md`: § Gate (line 63-), the package-gate table 156-164, the byte-pin paragraph 206-218 (t2), the slow-lane sentence at 220-222 (t1a).
- `CHANGELOG.md`: the `### Changed` heading under `## [Unreleased]`.
- Candidates for the exemption list (step 0(b)): `crates/verify/ess-conformance/tests/adversary_process_target_pass2.rs` (and a same-named file in `crates/edge/ess-cli/tests/`), `aggregate_group_selection.rs`, `adversary_287_pass1.rs`, `nested_response_observations.rs`, `typescript_suite_versions.rs` (all in `crates/verify/ess-conformance/tests/`).

## Tests first

Bounds: wall-time figures are measured on a warm build at a 1-minute load of 60 or less (`cut -d' ' -f1 /proc/loadavg`, before and after, both in the report; above 60 the run decides nothing and is repeated), by the runner's own per-test figure (`target/nextest/ci/junit.xml`) or, for a CI step, the step's own duration in the run; no test over 60 s, the test step within its ceiling (end state 120 s), a pool job ends after 10 minutes unless it names its time (`SLOT_TIMEOUT=30m`, 30 at most). No Python for anything that stays: the probes are Rust or POSIX sh in the scratch directory and are not committed. A measuring script starts without the `slot` prefix.

0. Measure first.
   - (a) The ceilings. Read the last five completed `main` push or `queue/**` pull-request runs of `ci.yml`: `b10x-gates gh --repo . --repository beyond10x/ess -- run view <run-id> --json jobs` from an empty directory outside every checkout (see `AGENTS.md` § Release), and take the longest duration of each step `Workspace tests ...`, `Feature-off tests ...` and the steps of `checks` that run `test-xtask` and `ci-smoke`'s doc tests. The ceiling of a task is the longest seen, plus 20 %, rounded up to a multiple of 60 s. If the runs cannot be read, use the parent's Need figures: `test-shard` 1500, `test-feature-off` 900, `test-xtask` 300, `test-doc` 300. Write the four numbers and their source in the report.
   - (b) The exemptions. For each test over 60 s still in the step after t2 to t6, find it: the tests named in addition A and the parent's Need list are `adversary_process_target_pass2` (`the_single_pass_reader_reads_generated_answers_as_the_reader_it_replaced`, 329 s measured locally), `aggregate_group_selection`, `adversary_287_pass1`, `nested_response_observations` and `typescript_suite_versions`; run each file under `SLOT_TIMEOUT=30m cargo nextest run --profile ci -p <crate> --test <name>` and read the per-test times from `target/nextest/ci/junit.xml`. An exemption is a test measured over 60 s on a warm build at load 60 or less (measure twice; a time that was only slow under load is not an exemption). Its allowed time is `terminate-after = ceil(seconds * 1.25 / 60)`, at most 5. A test over 300 s is not exempted: it is marked `slow lane` through t1a's machinery and its row is added to `MARKED`, which makes this unit touch one attribute line of that file and one row of `slow_lane.rs`; name it in the report.
   - (c) The kill. A scratch probe, not committed: add the file `crates/edge/ess-xtask/tests/zz_probe.rs` with `#[test] fn sleeps() { std::thread::sleep(std::time::Duration::from_secs(65)); }`, run `SLOT_TIMEOUT=30m cargo nextest run --profile ci -p ess-xtask --test zz_probe`, and see the test `TIMEOUT` and fail near 60 s (the `junit.xml` holds a failure for it); delete the file; `git status` shows nothing of it.
1. `test_budget.rs`:
   - `the_ci_profile_kills_a_test_after_sixty_seconds`: replaces `the_ci_profile_reports_every_test_over_sixty_seconds`'s acceptance of `{ period = "60s" }` alone: `[profile.ci]` has `slow-timeout = { period = "60s", terminate-after = 1 }` and `final-status-level = "slow"` and nothing else. Red today: `slow-timeout = { period = "60s" }` is the line.
   - `every_ci_override_is_a_listed_exemption_of_at_most_five_minutes`: every `[[profile.ci.overrides]]` table (read with a function like `table` for the header `[[profile.ci.overrides]]`, one table at a time) has exactly `filter = "..."` and `slow-timeout = { period = "60s", terminate-after = N }` with `N` from 2 to 5; and the set of filters equals the set of test names listed in `AGENTS.md` under the line `The exempt tests of the \`ci\` profile:` (one `- \`<name>\`: <seconds> s` line per test, the filter being `test(=<name>)`). Red today: the AGENTS.md list does not exist.
2. `ci_lanes.rs`:
   - `every_test_step_task_runs_under_its_ceiling_and_the_ceilings_are_one_table`: a constant `CEILINGS: [(&str, u32); 4]` with the four numbers of step 0(a); for each task: it has a `vars` entry `STEP_SECONDS` equal to the number; each command that runs `cargo nextest run` or `cargo test` starts with `timeout --kill-after=30 {{.STEP_SECONDS}} ` (for `test-xtask`, both of its commands); the command ends in a handler `|| { code=$?; if [ "$code" -eq 124 ]; then echo "<task>: over its step budget of {{.STEP_SECONDS}} s" >&2; fi; exit "$code"; }`; and `AGENTS.md` has the line `` - `<task>`: <n> s `` under the line `The step ceilings of the test tasks:`. Red today: no `STEP_SECONDS`.
   - `the_step_handler_exits_124_with_a_message_naming_the_task_and_passes_every_other_status_on`: for each task of `CEILINGS`, the test takes the handler text from the task's Taskfile command (everything after the first `|| `), expands `{{.STEP_SECONDS}}` to the ceiling, and RUNS it with `sh -c`: `( exit 124 ) || <handler>` must exit 124 with a stderr line containing the task name and `over its step budget`; `( exit 3 ) || <handler>` must exit 3 with empty stderr; `timeout 1 sleep 5 || <handler>` must exit 124 with the same message (it shows `timeout` itself reaches the handler). It spawns `sh` and `sleep` only, no build. Red today: the handler does not exist.
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
3. `Taskfile.yml`, one task at a time: `test-shard`, `test-feature-off`, `test-xtask` (both commands), `test-doc`: add `vars: STEP_SECONDS: '<ceiling>'` (merge into an existing `vars:` of the task; `test-shard` has one), and prefix the cargo command with `timeout --kill-after=30 {{.STEP_SECONDS}} `; follow with `|| { code=$?; if [ "$code" -eq 124 ]; then echo "<task>: over its step budget of {{.STEP_SECONDS}} s" >&2; fi; exit "$code"; }` (a command that was a single line becomes a block scalar `|` or a folded `>-` command; keep `defer: { task: test-slowest }` where it is). `test-packages` is not wrapped (a hotfix gate; its job has 60 minutes). `test-slow` is not wrapped (the profile kills a test at 20 minutes and the job is bound to 30). Locally, a task whose ceiling is over 10 minutes needs `slot --timeout` or `SLOT_TIMEOUT=30m` to run through the pool; say so in `AGENTS.md`, step 4.
4. `AGENTS.md` § Gate:
   - a paragraph after the one that ends `Every push also runs \`task fmt-check\`...` (line 166-167): "**The test step has a budget.**" saying: nextest kills a test of the `ci` profile at 60 s; each test task runs under `timeout` at its ceiling and fails over it; the ceilings only go down and are raised only by a story; the sum is the end state of 120 s. Then the line `The step ceilings of the test tasks:` followed by one line per task, `` - `test-shard`: <n> s `` and so on for the four tasks. Then the line `The exempt tests of the \`ci\` profile:` followed by one line per exempt test, `` - `<name>`: <seconds> s ``; each exemption is a defect and a follow-up story; the list shrinks.
   - the package-gate table (156-164): add two rows. "a test marked `#[ignore = \"slow lane: ...\"]` or the file that holds it | that crate's slow tests: `task test-slow PACKAGES=<crate>` (`cargo xtask slow-list` names them)"; and "`src/` of `ess-conformance`, `ess-gen`, `ess-transport`, `ess-http`, `ess-compiler`, `ess-domain`, `ess-primitives` or `ess-yaml`, or any model file | also `synthesis_suite_bytes_table`, re-stamping `tests/fixtures/synthesis-sources.digest` with the command it prints". Do not edit other rows.
5. `CHANGELOG.md`: the bullet below.
6. Run `## Gate`. Then `SLOT_TIMEOUT=30m task test-xtask` once.

## Owned files

`.config/nextest.toml` (`[profile.ci]` and its overrides), the `STEP_SECONDS` vars and `timeout` prefixes of the four test tasks in `Taskfile.yml`, the new tests of `crates/edge/ess-xtask/tests/test_budget.rs` and `crates/edge/ess-xtask/tests/ci_lanes.rs`, `AGENTS.md` § Gate (the new paragraph and two table rows), `CHANGELOG.md` (one bullet). Only if step 0(b) finds a test over 300 s: its one attribute line and its row in `crates/edge/ess-xtask/tests/slow_lane.rs`.

## Not yours

- t1a owns the lane machinery, `test-slow` and `test-slowest`, `[profile.slow]`, `AGENTS.md` 220-222, and the `test_budget.rs` tests it names. t1b owns the `slow` job, its `Gate` edits, `ci.yml` and `release.yml`. t2 to t6 own their test files, `AGENTS.md` 144-146 (t3) and 206-218 (t2), `test-xtask`'s two-command form (t6). h2 and h3 own their lines in `test`, `test-packages` and `test-shard` and h3 the Go set-up steps. The planning-only path filter (`PLAN_ONLY_*`, the `plan-only` step) and the ruleset are unchanged.
- No workflow file is edited by this unit. Follow-ups on the wave page: the JUnit upload steps of the shards; the hotfix gate running `task test-slow` with `setup-go` and the TypeScript step. The per-crate follow-ups (test files that spawn node, tsc, go, cargo, rustc or npm: `ess-conformance` 100, `ess-synth` 52, `ess-cli` 20) and D3 (feature-off tests to the slow lane): reported, not done.
- Shared: this unit lands last, after t1a, t1b, t2 to t6, h2 and h3. At merge: `Taskfile.yml` (different lines of the same tasks as h2 and h3: take theirs and add the wrapper; take `test-xtask` as t6 left it), `ci_lanes.rs` and `test_budget.rs` (append; keep every test of the earlier units), `AGENTS.md` (different paragraphs), `CHANGELOG.md` (keep all bullets).

## Gate

```console
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-xtask --locked --test test_budget --test ci_lanes --test slow_lane
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-xtask --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m task test-xtask
slot task ci-lint
slot task fmt-check
```

`Taskfile.yml` and `AGENTS.md` changed: `task ci-lint` and `task test-xtask` run (table rows "a workflow, `Taskfile.yml` or a public API" and "`CHANGELOG.md`, `website/` or `docs/`"). The exempt tests are measured in step 0, not run again as a gate. The whole `task check` is not run locally.

## Acceptance

1. `cargo test -p ess-xtask --test test_budget --test ci_lanes` is green with the new tests, and `the_ci_profile_kills_a_test_after_sixty_seconds` and the handler test were red before the edit.
2. The scratch probe of step 0(c) is killed at about 60 s and reported failed by nextest (a failure entry in `junit.xml`); the probe file is not in `git status`.
3. `task test-shard`, `test-feature-off`, `test-xtask` and `test-doc` each begin their cargo command with `timeout --kill-after=30 <ceiling>`; a ceiling in `Taskfile.yml`, in `CEILINGS` and in `AGENTS.md` is one number (`every_test_step_task_runs_under_its_ceiling_and_the_ceilings_are_one_table`); the handler of each task, run by `the_step_handler_exits_124_with_a_message_naming_the_task_and_passes_every_other_status_on`, exits 124 with the message on 124 and passes status 3 on silently.
4. Every exemption of `.config/nextest.toml` is in the `AGENTS.md` list with its measured time, and each listed test exists (`cargo nextest list -p <crate> --test <name>` names it).
5. `git diff --stat` against the unit's base shows no file under `.github/workflows/`.
6. The report names: the four ceilings and their source, and each exemption with its seconds and the load at which it was measured.

## Decisions

- Names (t1a/t1b, repeated): marker `#[ignore = "slow lane: <reason>"]`; `cargo xtask slow-list`; `task test-slow` with `PACKAGES`; profile `slow`; job `slow`, `Slow lane` (t1b); table `MARKED`. This unit adds: `STEP_SECONDS`, `CEILINGS`, the AGENTS.md lines `The step ceilings of the test tasks:` and `The exempt tests of the \`ci\` profile:`.
- A skipped slow lane never counts for a release: `prior-gate` qualifies only runs that cannot skip it (t1b's pin); this unit adds nothing to it.
- Ceilings start at today's measured step times, not at 120 s, because the step is 25 minutes today; a ratchet that fails a green tree would be reverted. The end state is 120 s and each per-crate story lowers a number. Raising one is an edit of `CEILINGS`, the Taskfile and `AGENTS.md` together, which a reviewer sees.
- The kill wrapper is coreutils `timeout --kill-after=30`: no new tool, no Python; exit 124 is a failure of the step, with a message naming the task. The handler is run by a test, not only matched as text.
- `terminate-after = 1` in the profile and exemptions by name at up to 5 periods (300 s): a test over 300 s is moved to the slow lane, not exempted.
- `test-xtask` is wrapped per command (compile, then run), each at the task's ceiling: a looser bound than one for the sum, accepted for simplicity.
- `test-xtask` and `test-doc` run `cargo test`, which kills no single test and writes no JUnit report, so in this wave they get the step ceiling only. The per-test 60 s kill and the ten-slowest print for them are not built here: they are a follow-up (run both tasks under nextest), listed on the wave page, in `t-followups.md` and in the parent (coordinator, Y6).
- `test-packages` and `test-slow` are not wrapped: a hotfix gate with a 60-minute job, and a lane whose own profile and job bound it.
- No new dependency, no comments in `nextest.toml` or `Taskfile.yml` lines you add.

## Changelog

`### Changed`:

- CI: nextest kills a test over 60 s and each test task fails over its listed step ceiling.
