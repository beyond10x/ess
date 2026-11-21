---
format: aep.planning-md/3
id: story:cargo-xtask-pr-watch-exits-non-zero-when-a-pull-requests-checks-end-red
kind: story
status: active
title: cargo xtask pr-watch waits for a pull request's check runs and exits non-zero, naming the red job and its failing task, when one ends red
tags:
- wave-2026-11-22
relations:
- serves: vision:O2
- derived_from: story:the-local-gate-catches-what-ci-catches
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/main.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/pr_watch.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T23:42:44Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-21T23:42:45Z", actor: "human:timo", revision: 5}
---
## Outcome

- `cargo xtask pr-watch <number> [--interval <s>] [--timeout <s>]` reads the head commit of a pull
  request, then polls that commit's check runs through `gh` until every one has completed.
- It exits with:
  - **0** when every check run ended `success`, `neutral` or `skipped`;
  - **1** when any other check run ended any other way. For each such run it prints the run's
    name, its conclusion, its URL and the failing task named in its annotations;
  - **2** on a timeout, a `gh` failure or unreadable JSON.
- It pushes nothing and writes nothing to GitHub.
- The decision is a pure function, tested on canned JSON with no network.
- No format version changes.

## Why (context only)

A session pushes and then has to read the CI lanes by hand. No task or script waits for a pull
request's checks and exits non-zero when one is red. The organization rule asks for one:
"a forge run that ends red makes the push or watch script exit non-zero". Decision O5 of
`story:the-local-gate-catches-what-ci-catches` says to read GitHub through `gh`, read-only, the
way `cargo xtask release status` already does (`crates/edge/ess-xtask/src/main.rs`,
`published_releases`, line 579).

## What to change

1. **New file `crates/edge/ess-xtask/src/pr_watch.rs`.** Put a `//!` doc at the top: what the
   command does, its three exit statuses, and that it reads through `gh` and writes nothing. Then
   these items, with exactly these names and signatures:
   ```rust
   use std::path::Path;
   use std::process::{Command, ExitCode};
   use std::time::Duration;

   /// Arguments of `cargo xtask pr-watch`.
   #[derive(Debug, clap::Args)]
   pub(crate) struct Args {
       /// The pull request number.
       number: u64,
       /// Seconds between two reads of the check runs.
       #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..))]
       interval: u64,
       /// Seconds to wait in all before giving up with exit status 2.
       #[arg(long, default_value_t = 5400)]
       timeout: u64,
   }

   /// One check run of the watched commit, as `GET /repos/{owner}/{repo}/commits/{sha}/check-runs` reports it.
   #[derive(Debug, Clone, PartialEq, Eq)]
   pub(crate) struct CheckRun {
       pub(crate) id: u64,
       pub(crate) name: String,
       pub(crate) status: String,
       pub(crate) conclusion: Option<String>,
       pub(crate) url: String,
   }

   /// What one read of the check runs says.
   #[derive(Debug, PartialEq, Eq)]
   pub(crate) enum Verdict {
       /// No run yet, or at least one run not `completed`.
       Pending { completed: usize, total: usize },
       /// Every run completed with a passing conclusion.
       Passed { total: usize },
       /// Every run completed and these did not pass, sorted by name.
       Failed(Vec<CheckRun>),
   }

   /// A run that did not pass, with the tasks its annotations name.
   #[derive(Debug, PartialEq, Eq)]
   pub(crate) struct FailedRun {
       pub(crate) run: CheckRun,
       pub(crate) tasks: Result<Vec<String>, String>,
   }

   /// How the watch ended.
   #[derive(Debug, PartialEq, Eq)]
   pub(crate) enum Outcome {
       Passed { total: usize },
       Failed(Vec<FailedRun>),
       TimedOut { completed: usize, total: usize },
       Error(String),
   }

   /// The conclusions that pass. Any other conclusion, or none, fails.
   const PASSING: &[&str] = &["success", "neutral", "skipped"];

   pub(crate) fn parse_check_runs(json: &str) -> Result<Vec<CheckRun>, String>;
   pub(crate) fn decide(runs: &[CheckRun]) -> Verdict;
   pub(crate) fn failing_tasks(json: &str) -> Result<Vec<String>, String>;
   pub(crate) fn watch(
       fetch_runs: &mut dyn FnMut() -> Result<String, String>,
       fetch_annotations: &mut dyn FnMut(u64) -> Result<String, String>,
       polls: u64,
       progress: &mut dyn FnMut(usize, usize),
       wait: &mut dyn FnMut(),
   ) -> Outcome;
   pub(crate) fn exit_status(outcome: &Outcome) -> u8;
   pub(crate) fn report(outcome: &Outcome) -> String;
   pub(crate) fn main(args: &Args) -> ExitCode;
   fn gh(root: &Path, args: &[&str]) -> Result<String, String>;
   ```
   What each function does:
   - **`parse_check_runs`**:
     - parse with `serde_json::from_str::<serde_json::Value>`;
     - require an object with `total_count` (u64) and `check_runs` (array);
     - from each element read `id` (u64), `name` (string), `status` (string), `conclusion`
       (string or `null` → `None`) and `html_url` (string; missing → `""`);
     - a missing or mistyped field returns `Err` naming the field, as in
       `"a check run has no numeric \`id\`"`;
     - if `total_count` is greater than the array's length, return
       `Err(format!("the commit has {total} check runs and one page holds {len}; pr-watch reads one page of 100"))`.
   - **`decide`**:
     - an empty slice is `Pending { completed: 0, total: 0 }`, because check runs appear some
       seconds after a push;
     - if any run's `status` is not `"completed"`, the result is
       `Pending { completed: <runs whose status is "completed">, total: runs.len() }`. It waits
       for every run even when one has already failed, so one red run names every failing job,
       as CI's `fail-fast: false` does;
     - otherwise collect the runs whose `conclusion` is `None` or not in `PASSING`;
     - none: `Passed { total }`; some: `Failed(..)`, sorted by `name` then `id`.
   - **`failing_tasks`**:
     - parse a JSON array of annotations; anything else returns `Err("the annotations are not a JSON array")`;
     - for each element's `message` string, for each line, find `Failed to run task "` and take
       the text up to the next `"`;
     - return the names in first-seen order, without duplicates.
     The `Gate` annotations read `task: Failed to run task "<name>": exit status N`, as described
     in `AGENTS.md` § "Reading a failed `Gate` without the log".
   - **`watch`** runs this loop for `poll` in `1..=polls`:
     - call `fetch_runs()`. On `Err(e)` return `Outcome::Error(e)`;
     - `parse_check_runs`. On `Err(e)` return `Outcome::Error(e)`;
     - `decide`:
       - `Passed` returns `Outcome::Passed`;
       - `Failed(runs)`: for each run call `fetch_annotations(run.id)`, then `failing_tasks` on
         an `Ok`. The result goes into `FailedRun.tasks`; an `Err` from either is kept as `Err`
         and does not change the outcome. Return `Outcome::Failed`;
       - `Pending { completed, total }`: call `progress(completed, total)`, then `wait()` only if
         `poll < polls`.
     - When the loop ends, return `Outcome::TimedOut` with the last pending counts.
   - **`exit_status`**: `Passed` → 0, `Failed` → 1, `TimedOut` and `Error` → 2.
   - **`report`**, exact text, each line ending in `\n`:
     - `Passed { total }`: `pr-watch: all {total} check runs passed`
     - `Failed(runs)`: per run, first `red: {name} ({conclusion}) {url}`, where `{conclusion}`
       is `no conclusion` for `None`. Then per task `  failing task: {task}`. When `tasks` is
       `Ok(vec![])`, one line `  failing task: not named in the annotations`. When it is
       `Err(e)`, one line `  failing task: annotations unreadable: {e}`.
     - `TimedOut { completed, total }`: `pr-watch: timed out with {completed} of {total} check runs completed`
     - `Error(e)`: `pr-watch: {e}`
   - **`gh`**: runs `Command::new("gh").args(args).current_dir(root).output()`.
     - A spawn error returns
       `Err(format!("running gh: {error}. pr-watch reads GitHub through gh and is not part of the offline gate"))`.
     - A non-zero exit returns `Err(format!("\`gh {}\` failed: {}", args.join(" "), stderr.trim()))`.
     - Success returns stdout as a UTF-8 string.
   - **`main`**:
     - get `root` from `crate::workspace_root()` (`main.rs` line 350; a child module may call
       the parent's private function). On `Err`, print `pr-watch: {e}` to stderr and return
       `ExitCode::from(2)`;
     - read the head with `gh(root, &["pr", "view", &number, "--json", "headRefOid"])`. Parse
       `{"headRefOid": "<sha>"}` and require a non-empty string. A failure is `Outcome::Error`
       and exit 2;
     - print `pr-watch: watching pull request {number} at {sha}` to stderr;
     - `polls = args.timeout.div_ceil(args.interval).max(1)`;
     - `fetch_runs` calls `gh(root, &["api", &format!("repos/{{owner}}/{{repo}}/commits/{sha}/check-runs?per_page=100&filter=latest")])`.
       The doubled braces produce the literal placeholders `{owner}` and `{repo}`, which `gh api`
       fills from the checkout's remote. `filter=latest` keeps only the newest attempt of a
       re-run job;
     - `fetch_annotations(id)` calls `gh(root, &["api", &format!("repos/{{owner}}/{{repo}}/check-runs/{id}/annotations?per_page=100")])`;
     - `progress` prints `pr-watch: {completed} of {total} check runs completed` to stderr, only
       when the pair differs from the last one printed;
     - `wait` is `std::thread::sleep(Duration::from_secs(args.interval))`;
     - print `report(&outcome)`: to stdout for `Passed` and `Failed`, to stderr for `TimedOut`
       and `Error`. Return `ExitCode::from(exit_status(&outcome))`.
   - The head commit is read once. A later push to the pull request is not followed (see Out of
     scope).
2. **`crates/edge/ess-xtask/src/main.rs`.** This unit is the only one that edits this file.
   - **Module.** Add `mod pr_watch;` to the module list, lines 3-20, before `mod presentation;`
     (line 12). If `cargo fmt` moves it, keep rustfmt's placement.
   - **Variant.** In `enum Command` (line 75), after the `Release { … }` variant (lines 158-162)
     and before the closing `}` at line 163, add:
     ```rust
         /// Wait for a pull request's check runs to end; exit 1 naming each red run and its
         /// failing task, 2 on a timeout or a `gh` failure. Reads GitHub through `gh`, writes
         /// nothing.
         PrWatch(pr_watch::Args),
     ```
     clap derives the subcommand name `pr-watch`.
   - **`main`** (lines 186-199). Replace its first line, `match run(Cli::parse()) {`, with:
     ```rust
         let cli = Cli::parse();
         if let Command::PrWatch(args) = &cli.command {
             return pr_watch::main(args);
         }
         match run(cli) {
     ```
     The rest of `main` stays.
   - **`run`.** Add this arm to the `match cli.command` (lines 203-280), after the last
     `Command::Release { command: ReleaseCommand::Status }` arm. It keeps the match exhaustive
     without a panic:
     ```rust
             Command::PrWatch(_) => Err("pr-watch is dispatched by main, not by run".to_owned()),
     ```
3. **Constraints on `pr_watch.rs`.**
   - No `unsafe`. No new dependency: `serde_json` and `clap` are already in
     `crates/edge/ess-xtask/Cargo.toml`.
   - Clippy pedantic is on. Avoid `as` casts between integer types, and give items doc comments.
   - The file must not contain the text `temp_dir()` or `crate::scratch::Scratch`. If it did,
     `src/scratch_leaks.rs` (`MAKERS`, line 42) would require it in its suite lists.
   - Keep it under 400 lines, tests included.

## Tests first

A `#[cfg(test)] mod tests { use super::*; … }` block at the end of `pr_watch.rs`. No network, no
`gh`, no sleeping: `watch` gets closures. Every test runs in milliseconds. All are red before the
change because the module does not exist.

Canned JSON. Build these with small helper functions in the test module:
- `run(id, name, status, conclusion)` returns
  `{"id":<id>,"name":"<name>","status":"<status>","conclusion":<"conclusion" or null>,"html_url":"https://github.com/o/r/runs/<id>"}`.
- `runs_json(&[..])` returns `{"total_count":<len>,"check_runs":[…]}`.
- `TASK_ANNOTATION` = `[{"path":".github","start_line":1,"annotation_level":"failure","message":"task: Failed to run task \"test-shard\": exit status 100"}]`.

1. `every_run_successful_neutral_or_skipped_passes`: runs `(1,"Gate","completed","success")`,
   `(2,"Docs","completed","neutral")` and `(3,"Test 1/20","completed","skipped")`. `decide`
   returns `Passed { total: 3 }`.
2. `a_failed_run_fails_once_every_run_completed`: `(1,"Gate","completed","success")` and
   `(2,"Test 11/20","completed","failure")`. Result: `Failed` holding run 2 only.
3. `cancelled_timed_out_and_missing_conclusions_fail`: `(1,"A","completed","cancelled")`,
   `(2,"B","completed","timed_out")` and `(3,"C","completed",null)`. Result: `Failed` with
   names `["A","B","C"]`.
4. `an_unfinished_run_keeps_the_watch_pending_beside_a_failure`:
   `(1,"Test 2/20","completed","failure")` and `(2,"Test 3/20","in_progress",null)`. Result:
   `Pending { completed: 1, total: 2 }`.
5. `no_check_run_yet_is_pending`: `{"total_count":0,"check_runs":[]}` gives
   `Pending { completed: 0, total: 0 }`.
6. `more_runs_than_one_page_is_an_error`: `{"total_count":150,"check_runs":[<one run>]}` makes
   `parse_check_runs` return `Err` containing `"150 check runs"`.
7. `a_run_without_an_id_is_an_error`: `{"total_count":1,"check_runs":[{"name":"x","status":"completed","conclusion":"success"}]}`
   gives `Err` containing `` `id` ``.
8. `the_failing_task_is_read_from_the_annotations`: `failing_tasks(TASK_ANNOTATION)` returns
   `Ok(vec!["test-shard"])`. Two annotations naming `test-shard` and one naming `ci-lint` return
   `["test-shard","ci-lint"]`.
9. `annotations_without_a_task_name_none_and_a_non_array_is_an_error`:
   `[{"message":"Process completed with exit code 1."}]` gives `Ok(vec![])`. `{"message":"x"}`
   gives `Err`.
10. `watch_names_the_red_run_and_its_task_and_exits_1`:
    - `fetch_runs` returns, in turn, a pending JSON (`(5,"Test 11/20","in_progress",null)`) and
      then a completed one (`(5,"Test 11/20","completed","failure")`, `(6,"Gate","completed","success")`);
    - `fetch_annotations` asserts its id is `5` and returns `TASK_ANNOTATION`;
    - `polls = 5`, and `wait` counts its calls.

    Expect:
    - an `Outcome::Failed` with one run, id 5, tasks `Ok(["test-shard"])`;
    - `wait` called exactly once and `exit_status` = 1;
    - `report` equal to
      `"red: Test 11/20 (failure) https://github.com/o/r/runs/5\n  failing task: test-shard\n"`.
11. `watch_passes_and_exits_0`: one completed all-success JSON. Expect `Outcome::Passed { total: 2 }`,
    `exit_status` 0, report `"pr-watch: all 2 check runs passed\n"`, and `wait` never called.
12. `watch_times_out_while_pending_and_exits_2`: `fetch_runs` always returns
    `(1,"A","completed","success")` and `(2,"B","queued",null)`, with `polls = 3`. Expect
    `TimedOut { completed: 1, total: 2 }`, `wait` called 2 times, `progress` called 3 times with
    `(1, 2)`, `exit_status` 2, and report
    `"pr-watch: timed out with 1 of 2 check runs completed\n"`.
13. `a_gh_failure_exits_2`: `fetch_runs` returns `Err("`gh api …` failed: HTTP 404")`. Expect an
    `Outcome::Error` holding that text and `exit_status` 2.
14. `an_annotation_failure_keeps_the_red_outcome`: one completed failure, and `fetch_annotations`
    returns `Err("HTTP 403")`. Expect `Failed`, `exit_status` 1, and the report containing
    `"  failing task: annotations unreadable: HTTP 403\n"`.
15. `the_command_line_parses`: `Cli::try_parse_from(["ess-xtask", "pr-watch", "806", "--interval", "10", "--timeout", "60"])`
    succeeds, with `Command::PrWatch` holding 806/10/60. `["ess-xtask","pr-watch","806"]` gets
    the defaults 30 and 5400. `--interval 0` is refused. Use `super::super::{Cli, Command}`; this
    test is why `Args`' fields are readable from the test module (same module, so no `pub` is
    needed).

## Commands

Prefix `cargo` with the compile-cache variables of `AGENTS.md` § Local compile cache:
`RUSTC_WRAPPER=sccache SCCACHE_DIR=~/.cache/sccache/ess SCCACHE_SERVER_PORT=4303 SCCACHE_CACHE_SIZE=2G`.

1. `cargo test -p ess-xtask --locked --bin ess-xtask pr_watch::`: the 15 tests pass, each in
   milliseconds.
2. `cargo test -p ess-xtask --locked --bin ess-xtask scratch_leaks::tests::every_source_that_makes_tmpdir_scratch_is_run_by_the_leak_check`:
   passes.
3. `cargo run -q -p ess-xtask --locked -- pr-watch --help`: prints the three arguments.
4. `cargo clippy -p ess-xtask --all-targets --locked -- -D warnings`: clean.
5. `cargo fmt --package ess-xtask`, then `slot task fmt-check`: clean.
6. Optional, network: `cargo run -q -p ess-xtask --locked -- pr-watch <an open or merged pull request> --timeout 60`
   from a checkout whose `gh` is logged in. It prints the watched head, then exits 0, 1 or 2 as
   the checks say. If `gh` is not authenticated, report that and skip this step.

Byte pins: none move. `ess-xtask` is not part of any generated output.

## Files this unit owns

- `crates/edge/ess-xtask/src/pr_watch.rs` (new)
- `crates/edge/ess-xtask/src/main.rs`: the four edits above only

Must not touch:
- Every other file under `crates/edge/ess-xtask/src/` and `crates/edge/ess-xtask/tests/`
  (`target_tmpdir.rs` and `toolchain_pin.rs` are other units' new files; `ci_lanes.rs` is the
  coordinator's).
- `crates/edge/ess-xtask/Cargo.toml`.
- `Taskfile.yml`, `AGENTS.md`, `CHANGELOG.md`, `.github/workflows/*`.

## For the coordinator at integration

1. **`Taskfile.yml`**: a new task, placed after the `release-status` task (lines 472-476 at `origin/main`):
   ```yaml
     pr-watch:
       desc: Wait for a pull request's check runs, e.g. `task pr-watch PR=806`; exits 1 naming each red run and its failing task, 2 on a timeout or a gh failure. Reads GitHub through gh; outside `check`.
       requires:
         vars: [PR]
       cmds:
         - cargo xtask pr-watch {{.PR}}
   ```
2. **`AGENTS.md` § Gate**, the package-gate paragraph (lines 142-146 at `origin/main`). Replace
   "— then push and read the lanes." with:
   "— then push and run `task pr-watch PR=<number>` in the background: it exits 0 when every
   check run passed, 1 naming each red run and its failing task, and 2 on a timeout (90 minutes
   by default) or a `gh` failure."
   In § Agent plugin, at the end of the paragraph "**Reading a failed `Gate` without the log.**"
   (line 391), add:
   "`cargo xtask pr-watch` reads the same annotations through `gh` and prints the failing task
   for each red run."
3. **`CHANGELOG.md`, `## [Unreleased]`, under `### Changed`**:
   `- Repository: `cargo xtask pr-watch <number>` (`task pr-watch PR=<number>`) waits for a pull request's check runs and exits 1 when one ends red, naming the run and the failing task from its annotations, and 2 on a timeout or a `gh` failure; it reads GitHub through `gh` and writes nothing.`

## Out of scope

- Pushing, re-running, cancelling or commenting. No GitHub write of any kind.
- Following a new head commit pushed during the watch, legacy commit statuses (only check runs),
  and more than 100 check runs (an error, exit 2).
- Running in the offline gate or in CI. It needs the network.
- Reading job logs. The log endpoint redirects to a zip, and the annotations are the route.
- `Taskfile.yml` and `AGENTS.md` (coordinator).

## Acceptance

- `cargo test -p ess-xtask --locked --bin ess-xtask pr_watch::` passes all 15 tests (Commands 1).
- `cargo xtask pr-watch --help` lists `<NUMBER>`, `--interval` and `--timeout` (Commands 3).
- `exit_status` maps `Passed`/`Failed`/`TimedOut`/`Error` to 0/1/2/2, held by tests 10-14.
- Clippy, fmt and the scratch-leak lister are clean (Commands 2, 4, 5).

## Depends on

nothing
