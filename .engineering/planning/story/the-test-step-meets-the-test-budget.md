---
format: aep.planning-md/3
id: story:the-test-step-meets-the-test-budget
kind: story
status: active
title: 'The test step of the gate meets the test budget: 2 minutes of wall time, no test over 60 s'
summary: The 20 test jobs sum 296-360 test-step minutes per run and a plan-only pull request runs them all; the runner kills at 60 s, a plan-only diff runs no test job, expensive products are made once, regeneration moves to a slow lane
tags:
- tooling
relations:
- serves: vision:O2
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
  path: Taskfile.yml
- confidence: inferred
  path: crates/edge/ess-cli/tests
- confidence: cited
  path: crates/edge/ess-cli/tests/tutorial_page.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/edge/ess-xtask/tests/ci_lanes.rs
- confidence: cited
  path: crates/edge/ess-xtask/tests/no_test_builds.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests
- confidence: cited
  path: crates/generate/ess-synth/tests/generated_view_queries.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests
revision: 27
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T21:10:59Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-11-21T21:11:00Z", actor: "human:timo", revision: 6}
---
## Outcome

ESS's gate meets the organization's rule `Tests stay fast`, checked on CI:

- **The test step ends within 2 minutes of wall time.** The test step is every step of one CI run
  that runs tests (today the `Workspace tests` and `Feature-off tests` steps of the 20 `Test <m>/20`
  jobs, plus `task test-xtask` and `task test-doc` in `Checks`). Its wall time is measured from the
  start of the first such step to the end of the last, and is recorded with the run id.
- **No test runs longer than 60 s.** The runner kills a test at 60 s and counts it as failed, and
  every test-running step prints its ten slowest tests.
- **An expensive product is made once per run or per test binary**, never per test.
- **Full regeneration and generated-code compiles run in a lane named `slow`**, only when their
  sources changed, and on the `main` push, the nightly run and the release gate.
- **The test step makes no network request.**
- **A pull request that changes only planning files starts no test job and builds no test
  archive**, and `Gate` still reports its result.

The budget is not met by raising a timeout, adding shards or waiting longer; the rule says so.

## Need

Measured on 2026-11-21 through the GitHub API (`run view <id> --json jobs`, step start and end
times; job logs were not read):

| | PR run 44936791749 (`fix/swapped-run-keeps-caller-bound-input`) | release run 44942406608 (tag 0.97.1) |
|---|---|---|
| wall time | 59.9 min (18:40:20–19:40:14 UTC) | 70.7 min (19:42:57–20:53:36 UTC) |
| runner-minutes, all jobs | 628 (19.5 of them macOS) | 672 (with packaging) |
| runner-minutes, the 20 test jobs | 561 | 558 |
| test job duration, min / avg / max | 16.1 / 28.1 / 41.9 min | 15.4 / 27.9 / 37.9 min |
| per test job, avg: artifact download | 10.3 min (max 28.6, shard 1/20) | 6.9 min (max 12.7) |
| per test job, avg: setup (checkout, disk, toolchains, npm, Go) | 2.8 min | 2.9 min |
| per test job, avg: `Workspace tests` step | 9.3 min (3.8–15.6) | 9.9 min (3.9–19.5) |
| per test job, avg: `Feature-off tests` step | 5.5 min (number semantics only) | 8.1 min (full) |
| test step, summed over the 20 jobs | 296 min | 360 min |
| `Build workspace test archive`: compile / upload | 12.5 / 1.7 min (job 17.5) | job 18.3 |
| `Build feature-off test archive`: compile | 8.5 min (job 11.7) | job 12.5 |
| `Checks`: `xtask` / `Smoke` (with `test-doc`) | 2.8 / 4.3 min | job 15.6 |

- **Extraction versus running tests.** The two test steps each extract their archive
  (`cargo nextest list --extract-to`, `Taskfile.yml` `test-shard` and `test-feature-off`) and then
  run; the step times above hold both. Without the logs the two are not separable. The shortest
  `Workspace tests` step (3.8 min, PR shard 1/20, group a) bounds the extraction of a group at
  3.8 min.
- **Archive sizes.** The uploaded artifacts are 12.78 GB (`nextest-archive-workspace`, both
  groups) and 8.14 GB (`nextest-archive-feature-off`) (`/actions/runs/44936791749/artifacts`).
  Every test job downloads both, so one run moves about 418 GB. Extracted, group a is 20.4 GB and
  group b 25.2 GB (`AGENTS.md`, Gate).
- **What 2 minutes means at today's volume.** 296 test-step minutes in 2 minutes of wall time
  would need 148 parallel jobs. The work in the step has to shrink, not be spread.
- **The evening of 2026-11-21.** The `main` push run 44942253777 (19:41:13–20:23:55) and the
  release run (from 19:42:57) ran side by side for 41 minutes. The release's workspace archive was
  ready at 20:01:45, but its test jobs started between 20:02:02 and 20:14:36, up to 12.9 minutes
  late, while the `main` run's 20 test jobs held the hosted runners from 20:01:31 until they were
  cancelled at 20:13:37–20:13:56. That other repositories' jobs queued behind them was reported
  and is not measured here.
- **A plan-only pull request runs everything.** `ci.yml` has no `paths` filter and no `if:` on
  the build or test jobs (lines 20–35, 161, 236, 315). Pull request 803 changes one file under
  `.engineering/planning/`; its CI run 44948556850 started every job, and at 21:04 UTC both
  archive builds were running with the 20 test jobs behind them. A full pull-request run costs
  about 628 runner-minutes.
- **The runner enforces nothing.** Nextest 0.9.143 runs the shards (`ci.yml` lines 198–200), but
  `.config/nextest.toml` sets no `slow-timeout`: it only serialises the Firefox binaries and one
  timing test. `test-xtask` and `test-doc` run under `cargo test`. The test job's limit is
  `timeout-minutes: 285` (`ci.yml` line 319, raised in `a5824dd176`, 2026-11-03).
- **The test step uses the network.** The workspace test step sets `ESS_TUTORIAL_NETWORK: "1"`
  (`ci.yml` line 518), so `ess-cli`'s `tutorial_page` runs `npm` against the registry.
  `transport-network` pulls a container image and installs an npm package before its two tests
  (`ci.yml` lines 575–582).

### What the tests are

Counted from the tree at 0.97.1 (`tests/*.rs` files directly under a crate's `tests/`, each one a
test binary; lines holding `#[test]` or `#[tokio::test]`, so macro-generated tests are not
counted):

| crate | test binaries | tests in `tests/` | tests in `src/` | `tests/` files that spawn `node`, `tsc`, `go`, `cargo`, `rustc` or `npm` |
|---|---|---|---|---|
| ess-conformance | 768 | 5,555 | 223 | 100 (106 with support modules) |
| ess-domain | 195 | 1,507 | 379 | 0 |
| ess-cli | 169 | 1,255 | 59 | 20 (22) |
| ess-synth | 146 | 952 | 22 | 52 (58) |
| ess-diff | 73 | 434 | 11 | 0 |
| workspace | 2,005 | 13,626 | 1,363 | 172 |

- `ess-conformance`'s tests call `synthesize(` or a sibling 1,944 times. 595 of its test files
  synthesize and use no `OnceLock` or `LazyLock`; 23 do. Synthesis here is a library call, and its
  cost per call is not measured.
- `ess-cli`'s tests start the `ess` binary (`CARGO_BIN_EXE_ess`) from 166 files.
- Five generated servers and harnesses are already built once, by Taskfile tasks, and handed to
  their tests by path (`integer-spellings-harness`, `append-set-harness`, `json-body-harnesses`,
  `ui-http-servers`, `clap-shared-binary`; `crates/edge/ess-xtask/tests/no_test_builds.rs` refuses a
  test that builds with cargo). That is the accepted form; the cases that run `tsc`, `node` and `go`
  per test have no such form yet.
- 109 tests are `#[ignore]`d. `task test-browser-product` (1120 s on 2026-10-22) and
  `task test-slow-probes` run in no CI lane.

Slow targets from local `cargo test` logs of 2026-11-21 on the shared development host (debug
builds through the shared build pool, under load; target times, tests in parallel; files under the
session scratch directory). These are local numbers, not CI's:

| target | tests run | time | spawns |
|---|---|---|---|
| `ess-conformance` `adversary_process_target_pass2` | 1 | 329 s; `the_single_pass_reader_reads_generated_answers_as_the_reader_it_replaced` alone, over 60 s | |
| `ess-conformance` `aggregate_group_selection` | 29 | 136 s | `cargo`, `go`, `node`, `tsc` |
| `ess-conformance` `adversary_287_pass1` | 24 | 106 s | `tsc`, `node`, `go` |
| `ess-conformance` `nested_response_observations` | 22 | 99 s | `go` |
| `ess-cli` `tutorial_page` | 8 | 41 s | `ess`, `go`, `npm` |
| `ess-conformance` `typescript_suite_versions` | 36 | 38 s | `tsc`, `node` |
| `ess-cli` `authored_scenarios` | 48 | 36 s | `ess` |
| `ess-conformance` `one_row_instance_bound` | 5 | 32 s | |
| `ess-conformance` `synthesis_suite_bytes_table` | 2 | 30 s | byte pin |
| `ess-xtask` `no_test_builds` | 4 | 27 s | |

Which tests run over 60 s on CI is not known: nextest's per-test times are only in the logs. Step 1
below produces that list.

## Design

Steps land one after another; each is a pull request with its own `ci_lanes.rs` pins.

1. **The runner reports, then kills.** Add a `ci` profile to `.config/nextest.toml` with
   `slow-timeout = { period = "60s" }` and a JUnit report, use it in `test-shard`,
   `test-feature-off` and `test-packages`, and print the ten slowest tests of each step from the
   JUnit file. Run `test-xtask` and `test-doc` under the same 60 s limit. The first run lists every
   test over 60 s; that list is this story's worklist. Once step 4 has emptied it, set
   `terminate-after = 1`. Saving: none; it makes the budget enforced and the slow tests named.
2. **A plan-only pull request runs no test job.** A first job in `ci.yml` lists the changed paths
   of a `pull_request` run (`git diff --name-only` against the base). When every path is under
   `.engineering/`, the archive builds and the test jobs are skipped and `Gate` counts them as
   passed for that reason only. A `main` push, the nightly run, a `queue/**` head, a `merge_group`
   run and a `workflow_call` from `release.yml` never skip. `docs/` and `website/` are not in the
   filter: `test-xtask`, `tutorial_page` and `site_navigation` read them. Saving: about 590
   runner-minutes per plan-only pull request (561 for the test jobs, 29 for the two archive builds,
   run 44936791749); how many such pull requests a week is not measured.
3. **No network in the test step.** `ESS_TUTORIAL_NETWORK` leaves the test step; the `npm` steps
   of `tutorial_page` move to the `slow` lane (step 5), and `transport-network` is named a network
   lane outside the test step. Saving: not measured.
4. **Expensive products made once.** Each test binary that compiles or runs generated Go,
   TypeScript or Rust per test gets one shared fixture: the generated module or package is written
   and built once per binary (`OnceLock`), or once per run by a Taskfile task like the five harness
   tasks, and every test reads it. A case that needs its own compile moves to `slow`. First the
   targets named in the table above and those step 1 lists, then the rest of the 172 files.
   Saving: not measured per target; it is the bulk of the 296 test-step minutes only if the
   spawning tests are, which step 1 shows.
5. **The `slow` lane.** One job runs the byte-pin targets `AGENTS.md` lists (`ess-conformance`
   `synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
   `adversary_related_values_all_bytes`, `external_beside_held_guard`; `ess-synth`
   `served_codec_names`, `go`, `feasibility`, `adversary_related_guard_behaviour_pass1`,
   `http_binding_rust`; `ess-openapi` `adversary_ess_issues_2_u6`; `ess-compiler`
   `string_alphabet_and_length`; `ess-cli` `tutorial_page`), the generated-server compiles of step 4,
   the five harness tasks and the cases of `test-slow-probes`. It runs on a pull request whose paths
   touch the generators, synthesis, `generated/`, the fixtures or the targets themselves, and always
   on the `main` push, the nightly run, a `queue/**` head and the release gate. `Gate` carries it
   when it ran. Saving: not measured.
6. **What remains in the 2-minute step.** In-process tests of the libraries and the CLI. After
   steps 4 and 5 the remaining test-step time is measured on a `main` push run; whether it fits 2
   minutes on the current job layout is decision D2 below.

The release gate keeps testing the tagged bytes: `release.yml`'s `prior-gate` qualifies a run only
when its test jobs and its `slow` lane ran, and a run that skipped them for a plan-only diff never
qualifies.

### Decisions D1 to D3: the options (the choices are under `## Decisions`)

- **D1. Generated-code compiles.** About 172 test files run `tsc`, `node`, `go`, `cargo` or `rustc`;
  the cost of one compile is not measured, but the slow targets in Need all spawn one, and in that
  number they do not fit a 2-minute step. Options: (a) all of them in `slow`, run when generator or synthesis
  sources change and on `main`, nightly and release; (b) commit the generated outputs with a digest
  of their sources, compare the digest in the test step, compile them once in `slow`; (c) keep a
  few representative compiles per target language in the test step behind a shared fixture and the
  rest in `slow`.
- **D2. Where the test step runs.** Each test job spends 6.9–10.3 min on average downloading 20.9 GB
  before any test, which alone exceeds the budget if counted. Options: (a) keep the archive and
  the 20 jobs and count only the test-running steps; (b) fewer jobs on larger runners that build and
  test in one place (a cost decision); (c) shrink the archives first through
  `story:one-integration-test-binary-per-large-crate`.
- **D3. The feature-off run.** 110–162 test-step minutes per run duplicate packages without
  `arbitrary_precision`. Options: (a) keep the pull-request subset in the test step and the full run
  in `slow`; (b) move all of it to `slow`; (c) keep it as it is and count it in the budget.

### Follow-up stories

Step 4 is too large for one change. It is split into one story per crate once step 1 has listed
the slow tests: `ess-conformance` (100 spawning files), `ess-synth` (52), `ess-cli` (20).

## Acceptance

- A pull request whose diff is only under `.engineering/` starts 0 `Test <m>/20` jobs and neither
  archive build, and `Gate` is green. `ci_lanes.rs` fails when a `main` push, a nightly run, a
  `queue/**` head, a `merge_group` run or a `workflow_call` can skip them.
- `.config/nextest.toml`'s profile used by CI sets `slow-timeout = { period = "60s",
  terminate-after = 1 }`, and `test-xtask` and `test-doc` run under the same limit. A test that
  sleeps 61 s fails its step. `ci_lanes.rs` fails when a test-running step uses another profile or
  no limit.
- Every test-running step prints its ten slowest tests with their times.
- The test step sets no `ESS_TUTORIAL_NETWORK` and runs no network command; `ci_lanes.rs` fails
  when it does.
- A `slow` job exists with the triggers of design step 5; `Gate` carries it; `ci_lanes.rs` pins its
  triggers and that every byte-pin target `AGENTS.md` lists runs in it.
- No test in the test step runs `tsc`, `node`, `go`, `npm`, `cargo` or `rustc` per test;
  `no_test_builds.rs` or `ci_lanes.rs` fails when one does outside a once-per-binary fixture.
- On a `main` push run after the last step, the test step's wall time (design, Outcome) is
  2 minutes or less, recorded here with the run id, or the decision D2 took names what is counted.
- `release.yml`'s `prior-gate` refuses a run whose test jobs or `slow` lane were skipped;
  `a_release_runs_its_full_gate_unless_a_full_gate_passed_the_tagged_tree` covers it.
- `AGENTS.md` § Gate describes the lanes as they are.
- A test-running step fails when it runs over 120 s, locally and in CI, and says which step and
  how long (the organization rule as amended on 2026-11-21). The bound is enforced by the task
  that runs the step, not by a job timeout, and `ci_lanes.rs` fails when a test-running step
  runs without it. Until the steps above have landed, a step known to be over the bound is
  listed by name with its measured time in `AGENTS.md` § Gate, and the list only shrinks.
- A pull request whose diff is only prose `*.md` files outside `website/` passes its required
  checks without a test job once `test-xtask`, `tutorial_page` and `site_navigation`, which read
  those files, run in one small job (the second change of design step 2).

## Fit review

The skill `.agents/skills/assessing-external-requests/SKILL.md` is written for changes to the
authored language; this request changes ESS's own gate, which no authored document reaches. Its
seven questions are answered for the gate.

1. **The need.** The organization's rule `Tests stay fast` (organization `AGENTS.md`, 2026-11-21)
   sets a budget ESS's gate exceeds: the test step sums 296–360 minutes over 20 jobs and spans 16–42
   minutes per job (Need), no runner limit exists below the job's 285 minutes, and a plan-only pull
   request runs it all (pull request 803). The rule came to this repository from another session,
   on the operator's rule.
2. **Class: defect.** The rule makes a step over budget a defect that gets a story and is fixed
   before the next feature.
3. **Can it already be expressed?** In part. Nextest is already the runner, so the 60 s kill is
   configuration (`.config/nextest.toml`). The five harness tasks and `no_test_builds.rs` are the
   made-once form for generated Rust servers. `test-packages` and `.github/scripts/hotfix-scope.sh`
   already select tests by changed crates for hotfix tags. None of it limits a test, filters a
   plan-only diff or keeps per-test `tsc`, `node` and `go` out of the step.
4. **Fit with what is there.** The lanes stay in `ci.yml` and `Taskfile.yml`, held by
   `ci_lanes.rs` (31 tests today, among them `every_step_of_task_check_runs_in_some_pull_request_lane`,
   `the_workspace_shards_are_one_complete_partition`, `the_gate_check_aggregates_every_lane_and_cannot_be_skipped`
   and `a_release_runs_its_full_gate_unless_a_full_gate_passed_the_tagged_tree`). The `slow` lane is
   a step of `task check` like any other, so `every_step_of_task_check_runs_in_some_pull_request_lane`
   must learn that a lane can run conditionally. A skipped required check reads as passing, which
   is why `Gate` runs `if: always()` (`ci.yml` lines 633–659); the plan-only skip goes through
   `Gate`, not around it.
5. **A second adopter.** Every repository under the rule; for ESS, every pull request and every
   release, and the other repositories that share the hosted runners.
6. **Cost.** No format, key, diagnostic or generated byte changes. Cost is CI and test code: a new
   job, a nextest profile, changes to `ci_lanes.rs`, `no_test_builds.rs`, `release.yml`'s
   `prior-gate`, and per-target fixture rewrites in three crates. A pull request touching the
   generators runs `slow` as well, so its wall time does not fall as far as one that does not.
7. **What else was considered.**
   - *Change nothing.* Refused: the rule binds and says a step over budget is fixed before the next
     feature.
   - *More shards or larger timeouts.* Refused by the rule; the history shows it does not hold
     (`story:test-shards-finish-inside-their-limit`, `story:test-shards-leave-room-for-the-download-retry`,
     the limit now 285 minutes).
   - *Select tests by changed crate on every pull request*, as hotfix tags do. Kept for later: it
     lowers the average but not the worst case, and the budget is a worst case.

## Decisions

- **Adopt.** The organization's rule binds; the budget is taken as the rule writes it (the skill's
  *accept as proposed*). Decided by the planning session on 2026-11-21.
- **Ordering.** Before the next feature story, as the rule says. Step 2 goes in first and alone,
  because it saves about 590 runner-minutes per plan-only pull request at once and depends on
  nothing. Step 1 follows and names the slow tests the rest works on.
- **Step 2 starts with `.engineering/` only.** A pull request that changes only documentation
  still runs the tests that read `docs/` and `website/` (`test-xtask`, `tutorial_page`,
  `site_navigation`). It stops starting the 20 test jobs once those three run in one small job
  of their own; that is the second change of step 2, not a new story.
- **D1, generated-code compiles: option (b).** The generated outputs are committed with a digest
  of their sources, the test step compares the digest and reads the files, and the outputs are
  compiled once per run in `slow`, never per test. A case whose output is not worth committing
  moves to `slow` whole (option (a) for that case). Reason: the repository already commits example
  trees under `generated/` and compares them byte for byte, so this extends a practice it has;
  and a digest check is the only form of these tests that fits a 2-minute step. Cost: more
  committed generated files, a regeneration commit with every generator change, and a compile
  error in generated code shows on a pull request only when the pull request touches generator,
  synthesis or fixture sources; otherwise on the `main`, nightly, queue or release run.
- **D2, where the test step runs: option (c).** The archives shrink first, through
  `story:one-integration-test-binary-per-large-crate`, which becomes part of this work and lands
  after step 1. No larger runners: option (b) spends money and is not taken without the operator.
  Option (a), counting only the test-running steps, is how the budget is measured until the
  archives have shrunk, and is not the end state. Reason: 2,005 test binaries are why each job
  downloads 20.9 GB; fewer binaries cut the download, the link time and the 45 GB of local test
  build together. Cost: about 1,900 test files move into a few binaries per crate, test names
  gain a module path, and every pin that names a test target (`AGENTS.md`, `ci_lanes.rs`,
  `Taskfile.yml` filters) is rewritten once.
- **D3, the feature-off run: option (b).** All of it moves to `slow`. It runs on a pull request
  whose paths touch a crate that names `arbitrary_precision`, and always on the `main` push, the
  nightly run, a `queue/**` head and the release gate. Reason: it repeats packages that the test
  step already ran, at 110 to 162 test-step minutes per run, which the rule calls a full
  regeneration. Cost: a change outside those paths that breaks the feature-off build is found on
  the queue or `main` run and not on its pull request.
- **Shared fixtures.** The conformance test files that call synthesis share one synthesized
  fixture per test binary or read a committed generated file; no test synthesizes for itself
  what another test of its binary synthesizes too.
- D1 to D3 were decided by the planning session on 2026-11-21, on the design authority's request
  to take the recommended option for each. The options are listed under Design.
- **Accepted by the operator on 2026-11-21** as decided above ("do it", passed on by the design
  authority), for this story and `story:one-integration-test-binary-per-large-crate`. The word
  does not cover larger runners; nothing is bought.
- **Correction, 2026-11-21: an in-process fixture shares nothing under cargo-nextest.** CI runs
  the tests with cargo-nextest, which starts one process per test, so a `OnceLock` or `LazyLock`
  is built once per test, not once per binary. Wherever this story says "once per binary
  (`OnceLock`)" or "one shared fixture per test binary", the form is a cache on disk under
  `target/`, keyed by the digest of its sources, or a setup step that runs before the tests.
  The 23 files that use `OnceLock` or `LazyLock` today share on a local `cargo test` and not on
  CI; step 1's ten-slowest report shows what that costs.
- **One integration-test binary per crate, and no full debug information for tests**, as the
  organization rule now says. The first is D2; the second holds already: `[profile.dev] debug`
  keeps line tables only (`AGENTS.md` § Gate).
- **Debug information of test binaries is measured as part of D2.** Test builds carry line tables
  today. On the development host a cargo configuration file sets the same and wins over the
  manifest, so a change in `Cargo.toml` alone has no effect there. Before the archives are
  reshaped, the share of `.debug_*` sections in one test binary is read with `readelf -S` on a
  CI-built binary; if it is a large part of the 20.9 GB, the archive build sets
  `CARGO_PROFILE_DEV_DEBUG=false` in its environment, proven by the same `readelf -S`, and a
  failing test then prints no file and line in a backtrace, which is the cost.
- **`.engineering/repro/` is not plan-only.** Tests read reproductions there (`ess-conformance`,
  `ess-domain` and `ess-synth` targets), so a pull request that changes a file under it runs
  every test job. Design step 2 is built with that exclusion, and a test fails when an
  archived test reads any other path under `.engineering/`.
- **Kept on a plan-only run:** `checks` (`test-xtask` scans the whole tree), `msrv`,
  `transport-network` and the macOS output-ownership witnesses, which are required checks by
  name. Skipped: the two archive builds and the 20 test jobs.

## Out of scope

- The `Checks` job's lint, Clippy, rustdoc and smoke steps, except `test-xtask` and `test-doc`.
- Compile time and disk of the test build (`story:one-integration-test-binary-per-large-crate`,
  `task:test-builds-take-less-disk`).
- The macOS output-ownership witnesses, packaging, `msrv`, Website and Atlas.
- The browser-product tests beyond moving them to `slow`.

## Overlap

- `story:one-integration-test-binary-per-large-crate` (draft): 1,294 test files on 2026-10-07,
  2,005 now; fewer binaries shrink the archives each test job downloads (D2 option c). It changes no
  test's own time.
- `task:test-builds-take-less-disk` (draft): disk of test builds.
- Implemented and not reopened: `story:ci-test-shards-each-extract-half-the-test-binaries`,
  `story:test-shards-finish-inside-their-limit`, `story:test-shards-leave-room-for-the-download-retry`,
  `story:ci-test-shards-leave-evidence-when-the-runner-runs-out-of-disk`.

No epic covers CI or the gate; like its siblings above, this story serves `vision:O2` directly.

## Scope

Derived 2026-11-22 by `story-scoper`. Every line is **cited** (read from the story or the tree) or
**inferred** (a reading that could be wrong).

- **Primary surface:** `.github/workflows/ci.yml`, `Taskfile.yml`, `.config/nextest.toml` (gate and runner configuration) — cited, the story's Design steps 1 to 6 name them
- **Files:** `crates/edge/ess-xtask/tests/ci_lanes.rs`, `crates/edge/ess-xtask/tests/no_test_builds.rs`, `.github/workflows/release.yml` (`prior-gate`), `AGENTS.md` § Gate — cited
- **Test targets, ess-conformance:** `tests/synthesis_suite_bytes_table.rs`, `tests/enum_presence_guard.rs`, `tests/adversary_u22_pass1.rs`, `tests/binding_trigger_row_predicate.rs`, `tests/adversary_ess0700a_identity_beside_row_set_2.rs`, `tests/fixtures/synthesis-*-table.tsv` — cited (files exist, read); the story names only the directory
- **Test targets, other crates:** `crates/generate/ess-synth/tests/generated_view_queries.rs`, `crates/edge/ess-cli/tests/tutorial_page.rs` — cited
- **Also likely:** `crates/edge/ess-xtask/src/docs.rs` (unit tests), `Taskfile.yml` `test-xtask` — inferred: the 216 s looks like compile, not test time
- **Symbols:** `[profile.ci]` `slow-timeout`, `test-slowest`, `test-xtask`, `test-doc`, `ESS_TUTORIAL_NETWORK`, `UNAFFORDABLE` — cited
- **Documents:** `AGENTS.md` § Gate — cited
- **Confidence:** high for the gate files and the named targets, medium for the unit cut: per-test times are not reproducible read-only
- **Would collide with:** any unit touching `ci.yml`, `Taskfile.yml`, `.config/nextest.toml` or `ci_lanes.rs`; any unit that re-pins or edits the synthesis suite tables, `enum_presence_guard.rs` or `tutorial_page.rs`
- **Safety fact:** moving a test to `slow` or `#[ignore]` changes no generated byte, because the tables stay committed — `synthesis_suite_bytes_table.rs:36-37` (`include_str!`), unproven

## Tests found over the bound after this story was written

Eight test targets measured over the 60 s bound, or over the 2-minute step, on the shared
development host (debug builds, one loaded machine, target or test times as the running session
printed them; the logs are not in the tree). Every name below exists at `16f2fc88c2`. The table in
Need lists two of them (`synthesis_suite_bytes_table` at 30 s and `tutorial_page` at 41 s) with
times that are now out of date; the other six are not listed anywhere in this story.

| target | test | seconds seen | in this story | how it spends the time (read in the source) |
|---|---|---|---|---|
| `ess-conformance` `synthesis_suite_bytes_table` | the target (2 tests) | 60 to 811 by host load | listed at 30 s | `synthesize(` at line 213, one suite per row of the table |
| `ess-conformance` `enum_presence_guard` | `no_corpus_suite_requires_a_branch_the_precedence_takes_elsewhere` (line 709) | about 716 | no | `synthesize(` at line 167, called by the test |
| `ess-conformance` `adversary_u22_pass1` | one of 8 tests | about 800 | no | `synthesize(` at line 338 |
| `ess-conformance` `binding_trigger_row_predicate` | `a_target_that_retries_not_eligible_fails_the_refusal` (line 305) | about 105 | no | `synthesize(` at line 223 |
| `ess-conformance` `adversary_ess0700a_identity_beside_row_set_2` | one of 11 tests, not named by the session | one test over 60 | no | `synthesize(` at line 497 |
| `ess-synth` `generated_view_queries` | the target, seen twice | about 257 | no | spawns `cargo` (line 279), `go` (line 1095) and a built binary (line 1170), and calls `synthesize(` (line 414) |
| `ess-cli` `tutorial_page` | 8 tests | 363 | listed at 41 s | `ess`, `go` and `npm` per test |
| `ess-xtask` unit binary | the tests filtered to `docs::` | 216.73 | no | not read: `docs.rs` has 64 functions and no process spawn, so the time is in the tests that call into it or in the link |

A file named `adversary_ess0700a_identity_beside_row_set_2.rs` exists in two crates,
`crates/verify/ess-conformance/tests/` (11 tests, calls `synthesize(`) and
`crates/specify/ess-domain/tests/`. The report did not say which crate; the first is the likely
one and has to be confirmed with the test's own time before the story names it.

Six of the eight are the mistake this story exists for: a suite synthesized inside the test, so
its cost is paid once per test. The step 1 list (ten slowest tests, per target) replaces the
numbers above with CI's, and the acceptance below holds each of them:

- No test named in this section runs over 60 s in the test step, and the `ess-xtask` unit binary
  runs `docs::` within the step's 120 s. A byte-pin target (`synthesis_suite_bytes_table`) that
  cannot is in the `slow` lane of design step 5, with its sources digest, and the test step reads
  the committed file instead.
- `generated_view_queries` builds with cargo once, in a Taskfile task, and the test reads its path
  (the accepted form named in Need); `no_test_builds.rs` covers the file it names.

CI, 2026-11-26 (runs of that day, times as the runner printed them): `explore_concurrent` 170 s; `external_request_binding::no_use_and_owed_only_models_compile_without_an_external_port` 160 s; `explore_concurrent_reads_adversary_2` 107 s; the cases of `coverage_producers`, `one_row_instance_bound`, `emit_swap_served` and the `go` cases of `http_binding_rust` each over 60 s. The slow-lane test `every_repository_model_and_component_writes_its_bytes_at_one_two_and_seven_jobs` ran 96.6 s and was not killed by the `slow` profile. Locally, 57 `ess-cli` tests ran over 60 s. None of these is classified yet: the first step for each is its cause (a product built or synthesized inside the test, a process per test, or a missing kill) against the bound, and for the slow-lane test the `slow` profile's `terminate-after` is the check.

## Decisions taken

Taken for the wave of 2026-11-23, after round 1 of the plan critics. The story stays active after
the wave: these parts of it are not in the wave's units and are follow-ups.

Acceptance lines not in the wave:

- (a) `ci_lanes.rs` pins that every byte-pin target `AGENTS.md` lists runs in the slow job. The
  wave moves 4 of the 13 listed targets; the other 9 (`adversary_related_values_all_bytes`,
  `external_beside_held_guard`, `served_codec_names`, `go`, `feasibility`,
  `adversary_related_guard_behaviour_pass1`, `http_binding_rust`, `adversary_ess_issues_2_u6`,
  `string_alphabet_and_length`) stay in the test step.
- (b) The browser-product tests and the five harness tasks move to the slow lane; the wave leaves
  them as they are.
- (c) A pull request whose diff is only prose `*.md` files outside `website/` passes its required
  checks without a test job.
- (d) The test step runs no network command, pinned by `ci_lanes.rs`, and `transport-network` is
  named a network lane; the wave pins `ESS_TUTORIAL_NETWORK` only.

Cut from units:

- The `ess-xtask` unit-test unit does not edit `consumer-check` and adds no `ci_lanes` pin over
  every `test*` task for wrapper and job settings; both belong to the draft story
  `story:consumer-coverage-build-profile-admits-the-hosts-compile-wrapper-and-no-taskfile-task-empties-rustc-wrapper`.
- The runner-kill unit adds no JUnit upload step to the shard jobs and does not make the hotfix
  gate run `task test-slow`.

Earlier follow-ups, still open: every feature-off test moves to the slow lane (decision D3 above);
one story per crate for the test files that spawn node, tsc, go, cargo, rustc or npm in the test
step (172 files: `ess-conformance` 100, `ess-synth` 52, `ess-cli` 20), each lowering a ceiling of
the runner-kill unit; a matrix split of the slow job if `task test-slow` takes over 20 minutes; a
story in the slot repository naming the exceptions found (the harness tasks with `env -u
RUSTC_WRAPPER`, the `CARGO_BUILD_JOBS` of the macOS witness job in `ci.yml`); each test the
runner-kill unit exempts from the 60 s kill is a defect with its own story.

Also decided: the slow lane is two units, the local mechanism and the CI wiring; the byte-pin
stamp is written once, by the byte-pin unit, after every unit that changes a source it hashes;
a unit that adds a test file to a crate that already has an integration binary says why it cannot
be a module of an existing one.

Added after round 2 of the plan critics, for the wave of 2026-11-23. Not reached by the wave:

- `test-xtask` and `test-doc` run `cargo test`, which kills no single test and writes no JUnit
  report; in the wave they get the step ceiling only. Running them under nextest, so that a test
  over 60 s is killed and the ten slowest are printed for them, is a follow-up.
- (e) The test step ends within 120 s on a `main` push, with the run id recorded here. Not reached:
  the runner-kill unit's ceilings are a ratchet set at the longest time seen plus 20 %, and each
  later story lowers them.
- (f) Decision D2, `story:one-integration-test-binary-per-large-crate`, lands after the wave.

What the wave delivers of this story: a slow lane exists, six kinds of expensive test leave the
test step, a test over 60 s is killed, and the step ceilings are pinned. The two-minute end state
is not claimed.
