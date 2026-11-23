---
format: aep.planning-md/3
id: story:the-tutorial-walkthrough-runs-only-the-ess-commands-in-the-test-step-and-npm-and-go-in-the-slow-lane
kind: story
status: active
title: The tutorial walkthrough runs only the ess commands in the test step and npm and go in the slow lane
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
- depends_on: story:a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it
scope:
- confidence: cited
  path: .github/workflows/ci.yml
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-cli/tests/tutorial_page.rs
- confidence: cited
  path: crates/edge/ess-xtask/tests/ci_lanes.rs
- confidence: cited
  path: crates/edge/ess-xtask/tests/slow_lane.rs
revision: 17
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:02:49Z", actor: "human:timo", revision: 16, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:02:51Z", actor: "human:timo", revision: 17, decided_on: {"recorded":{"review_outcome":3}}}
---
## Outcome

`tutorial_page` no longer needs the npm registry or a `go` build in the test step.

- `the_walkthrough_runs_and_prints_what_the_pages_record` runs every `ess` command and every file block of the Start-here pages, and prints the `requires=node` and `requires=go` blocks as skipped with a reason that names the slow lane. It runs within 30 s.
- A new test `the_walkthrough_runs_the_npm_and_go_blocks_too` carries `#[ignore = "slow lane: runs npm install and go build in the walkthrough"]` and runs the same walkthrough with the npm and go blocks. It fails (does not skip) when `ESS_TUTORIAL_NETWORK` is not `1` or node, npm or go is missing.
- `ESS_TUTORIAL_NETWORK=1` is set by `task test-slow` and by no step of the `test` job.

## Why

Parent: `story:the-test-step-meets-the-test-budget`, step 5 (the `slow` lane) and Need's table (`tutorial_page`, 41 s listed, 363 s seen in addition A).
Rule: "Tests stay fast": no network request and no expensive per-test step in the test step; the 60 s bound per test.
Consumer gap: none.

## Where

Read at `80794bbd698` (unchanged since `bf128f9146`). Re-read a line before editing it.

`crates/edge/ess-cli/tests/tutorial_page.rs` (993 lines):
- Doc table rows 19-20 (`requires=node`, `requires=go`) and 51-56 (`ESS_TUTORIAL_NETWORK=1 is the one switch this test reads`).
- `PAGES` 75-83, `NETWORK` 92, `walkthrough_blocks` 214, `steps` 225, `on_path` 348, `node_skipped` 357, `go_skipped` 368, `go_env` 378, `release` 397 (builds the release tree the commands run against), `copy_tree` 433.
- `struct Walkthrough` 457-470 (fields `node_skipped`, `go_skipped`), `Walkthrough::new` 472-500 (calls `node_skipped()` at the `node_skipped: node_skipped()` field and `go_skipped()` before it), `fn skipped` 502.
- Tests: `the_walkthrough_runs_and_prints_what_the_pages_record` 762-805 (the `#[test]` at 762; builds `Walkthrough::new` at 773 and prints the skipped kinds at 786-804), `every_ess_command_the_pages_show_is_run_by_the_walkthrough` 808, `every_page_with_a_walkthrough_block_is_run` 824, the other five tests 857-993 (read no toolchain).

`.github/workflows/ci.yml`: step `Workspace tests ${{ matrix.group }} ${{ matrix.part }}`, lines 641-645, `env: ESS_TUTORIAL_NETWORK: "1"` at 643-644 (the only place the variable is set).

`Taskfile.yml`: `test-slow` (created by t1a) gets an `env:` entry.

`AGENTS.md` lines 144-146: "`ESS_TUTORIAL_NETWORK=1` (set on CI's workspace test shards) additionally runs the Start-here pages' `npm` steps in `tutorial_page.rs`, which need the npm registry; unset, they print a skip. Its `go` steps need no network and run wherever `go` is on `PATH`."

`crates/edge/ess-xtask/tests/ci_lanes.rs`: append a function at the end of the file.

`crates/edge/ess-xtask/tests/slow_lane.rs`: the `MARKED` table (t1a). `CHANGELOG.md`: the `### Changed` heading under `## [Unreleased]`.

## Tests first

Bounds: wall-time figures are the runner's own per-test figure (`target/nextest/ci/junit.xml`), on a warm build at a 1-minute load of 60 or less (`cut -d' ' -f1 /proc/loadavg`, before and after; above 60 the run decides nothing and is repeated); no test over 60 s, the test step within 120 s, a pool job ends after 10 minutes unless it names its time (`SLOT_TIMEOUT=30m`, 30 at most); no Python for anything that stays. Time every new test and name any over 5 s.

0. Measure first. Run `SLOT_TIMEOUT=30m cargo nextest run --profile ci -p ess-cli --test tutorial_page` on the tree with the npm and go blocks forced off (`ESS_TUTORIAL_NETWORK` unset and `PATH` without `go`, for example `env -u ESS_TUTORIAL_NETWORK PATH=<path without go>`), and read the time of `the_walkthrough_runs_and_prints_what_the_pages_record` from `target/nextest/ci/junit.xml`. One threshold, 30 s: at most 30 s, go on with step 3 as written. Over 30 s, the step test runs only the longest prefix of `PAGES` whose cumulative time is at most 30 s (the pages are sequential: a later page reads files an earlier one wrote, so a prefix, never a subset), the full list runs in the slow twin, and the report names the two times. Do not split the pages into several tests.
1. `tutorial_page.rs`:
   - `the_toolchain_blocks_are_skipped_in_the_step_and_named_for_the_slow_lane` (new, under 5 s): `Walkthrough::new(scratch, Toolchains::Skipped)` sets `node_skipped` and `go_skipped` to reasons that both contain `slow lane`, whatever the environment holds, and does not call `go_env` (no process spawned). Red today: compile error, `Toolchains` is not found.
   - `the_walkthrough_runs_the_npm_and_go_blocks_too` (new, slow): see step 4. It cannot be run red in the step; its red is the compile error today and, once present, the failure `ESS_TUTORIAL_NETWORK=1 is required` when run with the variable unset (check once by hand: `SLOT_TIMEOUT=30m cargo test -p ess-cli --locked --test tutorial_page -- --ignored the_walkthrough_runs_the_npm_and_go_blocks_too`).
2. `crates/edge/ess-xtask/tests/ci_lanes.rs`, `the_tutorial_network_is_set_by_the_slow_lane_and_by_no_test_step`: parses `ci.yml` and `Taskfile.yml`; no step of the `test` job or of any job that runs `task test-shard` has `ESS_TUTORIAL_NETWORK` in its `env`; the `test-slow` task's `env` has `ESS_TUTORIAL_NETWORK: "1"`. Red today: the step at `ci.yml:643-644` sets it, and `test-slow` has no `env`.

## Steps

1. Measurement (step 0 above).
2. In `tutorial_page.rs` add `#[derive(Clone, Copy, PartialEq, Eq)] enum Toolchains { Skipped, Present }`. Change `Walkthrough::new(scratch: &Path)` to `new(scratch: &Path, toolchains: Toolchains)`. With `Skipped`: `node_skipped` is `Some("skipped in the test step: the slow lane runs the npm blocks")`, `go_skipped` is `Some("skipped in the test step: the slow lane runs the go blocks")`, and `go_cache` and `go_mod_cache` are `None` (no `go env` process). With `Present`: today's behaviour, calling `node_skipped()` and `go_skipped()` and `go_env`.
3. `the_walkthrough_runs_and_prints_what_the_pages_record` builds `Walkthrough::new(scratch.path(), Toolchains::Skipped)`. Its printed line keeps the form `<n> page(s): ... the requires=node and requires=go blocks skipped`. Apply the step-0 rule if the measurement says so. Keep every assertion of the test (every page has a `MARK` block, a file block and a command block exist).
4. Extract the body of that test after the `PAGES` assertions into `fn run_walkthrough(toolchains: Toolchains) -> (usize, usize, Vec<&'static str>)` (blocks run, commands run, kinds skipped) so the two tests share it; no macro (the slow-lane scanner reads `#[test]` functions, not macro expansions). Add
   ```rust
   #[test]
   #[ignore = "slow lane: runs npm install and go build in the walkthrough"]
   fn the_walkthrough_runs_the_npm_and_go_blocks_too() { ... }
   ```
   It asserts `std::env::var_os(NETWORK) == Some("1")` with the message `ESS_TUTORIAL_NETWORK=1 is required: the slow lane sets it`, asserts `on_path("node", "--version") && on_path("npm", "--version") && on_path("go", "version")` with a message naming the missing program, runs `run_walkthrough(Toolchains::Present)`, and asserts the skipped list is empty. A skip in the slow lane would hide a missing toolchain: it must fail.
5. Update the doc rows 19-20 and the paragraph 51-56 of the file: the step skips both; the slow lane runs both and requires the variable.
6. `ci.yml`: delete the two lines `env:` and `ESS_TUTORIAL_NETWORK: "1"` of the `Workspace tests` step (641-645); keep the `if` and `run`. Do not touch any other line of that step.
7. `Taskfile.yml`: add an `env:` block with `ESS_TUTORIAL_NETWORK: "1"` to the `test-slow` task t1a created (t1a lands first; at merge keep t1a's `cmds` and add only the `env:` key). No comment in the file.
8. Add the `slow_lane.rs` `MARKED` row `crates/edge/ess-cli/tests/tutorial_page.rs` with `the_walkthrough_runs_the_npm_and_go_blocks_too`, in path order.
9. Add the `ci_lanes.rs` test of `## Tests first`, appended at the end of the file.
10. `AGENTS.md` lines 144-146: replace with "`ESS_TUTORIAL_NETWORK=1`, set by `task test-slow`, makes the slow lane's `the_walkthrough_runs_the_npm_and_go_blocks_too` run the Start-here pages' `npm` and `go` steps, which need the npm registry and `go`; the test step's walkthrough runs every `ess` command and skips those blocks." Do not touch any other line.
11. Time the tests: `the_walkthrough_runs_and_prints_what_the_pages_record` and the new unit test, and `task test-slow PACKAGES=ess-cli` once (`SLOT_TIMEOUT=30m`); report the three times. The slow twin must finish well within the profile's 20 minutes.

## Owned files

`crates/edge/ess-cli/tests/tutorial_page.rs`, `AGENTS.md` lines 144-146, the `ESS_TUTORIAL_NETWORK` lines of `ci.yml` (643-644), the `env:` key of `test-slow` in `Taskfile.yml`, the new function appended to `crates/edge/ess-xtask/tests/ci_lanes.rs`, the row you add to `crates/edge/ess-xtask/tests/slow_lane.rs`, and one bullet in `CHANGELOG.md`.

## Not yours

- t1a: `test-slow`'s commands, `slow_lane.rs`'s machinery, `nextest.toml`. t1b: the `slow` job and `Gate`. t2, t4, t5, t6, t7: their files. `ess-cli`'s other tests, including `generate_ui_http` and its servers.
- Shared, all by "keep both sides at merge": `Taskfile.yml` (the `test-slow` task: t1a lands first; you add `env:` only), `ci.yml` (different jobs: t1b's `slow` job and `Gate` edits and h3's `setup-go` step in `build-tests`; your deletion is in the `test` job), `ci_lanes.rs` (appended tests: t1b, t6, h3 and you; t1b lands first and changes `run_gate`'s signature, yours does not call it), `slow_lane.rs` `MARKED` (t2, t4, t5 add rows beside yours), `CHANGELOG.md` (one bullet each), `AGENTS.md` (different lines).
- The pin of this unit covers `ESS_TUTORIAL_NETWORK` only. The parent's wider promise (the test step runs no network command, pinned by `ci_lanes.rs`, and `transport-network` named a network lane) is a follow-up under the parent (X8(d) on the wave page), not this unit.

## Gate

```console
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-cli --locked --test tutorial_page
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-cli --locked --test tutorial_page -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-xtask --locked --test ci_lanes --test slow_lane
SLOT_TIMEOUT=30m task test-slow PACKAGES=ess-cli
slot task fmt-check
slot task ci-lint
```

`ess-cli` is the crate touched; `tutorial_page` is one of the named byte-pin targets (AGENTS.md § Gate list), so it is run. The `ess-cli` integration tests are not run as a whole. `ESS_TYPES_NODE` and the JavaScript tools of § Gate are needed only for the slow twin (`PATH` with `node`, `npm`, `go`); the step test needs none.

## Acceptance

1. `cargo test -p ess-cli --test tutorial_page -- --nocapture` with `ESS_TUTORIAL_NETWORK` unset and no `go` on `PATH` is green and its output holds the skip line, whose text is exactly `skipped in the test step: the slow lane runs the npm blocks` for the node blocks and `skipped in the test step: the slow lane runs the go blocks` for the go blocks, followed by the summary line ending `the requires=node and requires=go blocks skipped`.
2. The same command with `ESS_TUTORIAL_NETWORK=1` and `go` on `PATH` prints the same two skip lines (the step skips whatever the environment holds): `the_toolchain_blocks_are_skipped_in_the_step_and_named_for_the_slow_lane` is green, and it asserts that `Walkthrough::new(.., Toolchains::Skipped)` holds no `go_cache` and no `go_mod_cache` (so no `go env` process is started).
3. `the_walkthrough_runs_and_prints_what_the_pages_record` ends within 30 s by the runner's figure, on a warm build at a 1-minute load of 60 or less (if step 0 applied the prefix rule, the prefix is named and its time is the figure); the measurement before and the figure after are in the report.
4. `cargo xtask slow-list` lists `ess-cli tutorial_page the_walkthrough_runs_the_npm_and_go_blocks_too`; `task test-slow PACKAGES=ess-cli` is green (the task sets `ESS_TUTORIAL_NETWORK=1` itself), and the ignored test run directly with the variable unset, `env -u ESS_TUTORIAL_NETWORK cargo nextest run -p ess-cli --test tutorial_page --run-ignored only`, fails with `ESS_TUTORIAL_NETWORK=1 is required`.
5. `the_tutorial_network_is_set_by_the_slow_lane_and_by_no_test_step` is green; `grep -n ESS_TUTORIAL_NETWORK .github/workflows/ci.yml` finds nothing.
6. `cargo test -p ess-xtask --test slow_lane` is green with the new row.

## Decisions

- Names (t1a, repeated): marker `#[ignore = "slow lane: <reason>"]`; `cargo xtask slow-list`; `task test-slow` with `PACKAGES`; profile `slow`; job `slow` (t1b); `MARKED`.
- D1/D3 not reopened. The slow lane needs `node`, `npm`, `go` and the TypeScript toolchain: t1b's `slow` job installs them (setup-go 1.25.10, setup-node 22, the TypeScript step copied from the `test` job). The twin passes in CI only once t1b has landed; locally `task test-slow PACKAGES=ess-cli` runs it with the host's tools.
- Both toolchains are skipped in the step, not only npm: the `go` blocks need no network but still spend a `go build` per run. The `ess` commands are the part the pages record and stay in the step.
- A slow twin that fails when a toolchain is missing, not one that skips: a skip in the lane that exists to run them would be green and prove nothing.
- No per-page split: the pages are sequential (a later page reads the files an earlier page wrote). A prefix of `PAGES` is the only reduction allowed, and only under the step-0 rule (30 s).
- No macro for the two tests: the scanner of t1a reads functions, not expansions.
- `ESS_TUTORIAL_NETWORK` moves to the `test-slow` task `env:` and not to the `slow` job's `env`, so that the local `task test-slow` and CI run the same thing.

## Changelog

`### Changed`:

- CI: the Start-here walkthrough test runs the `ess` commands of the pages in the test step and runs their `npm` and `go` blocks in the `slow` lane, where `ESS_TUTORIAL_NETWORK=1` is set; no test step sets it (story:the-test-step-meets-the-test-budget).
