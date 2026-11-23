---
format: aep.planning-md/3
id: story:the-test-runner-reports-every-test-over-60-s-and-the-ten-slowest
kind: story
status: implemented
title: The test runner names every test over 60 s and each test step prints its ten slowest tests
tags:
- wave-2026-11-22
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
scope:
- confidence: cited
  path: .config/nextest.toml
- confidence: cited
  path: Taskfile.yml
- confidence: inferred
  path: crates/edge/ess-xtask/tests/test_budget.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T23:42:59Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-11-21T23:43:00Z", actor: "human:timo", revision: 6}
- {from: "active", to: "implemented", at: "2026-11-23T01:57:05Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

- `.config/nextest.toml` has a profile `ci`: nextest marks a test SLOW each 60 s it runs, lists
  every such test in the run's final summary, and writes `target/nextest/ci/junit.xml`.
- `test-shard`, `test-feature-off` and `test-packages` in `Taskfile.yml` run nextest with
  `--profile ci`, and each defers the new task `test-slowest`, which prints the ten slowest tests
  of that JUnit report, also when the run failed.
- The profile reports and does not kill: no test is terminated at 60 s in this unit.
- `crates/edge/ess-xtask/tests/test_budget.rs` holds all of this and runs in milliseconds.

## What to change

This is design step 1 of `story:the-test-step-meets-the-test-budget`. Facts verified on
2026-11-22 with cargo-nextest 0.9.143, the version `ci.yml` installs (`tool: cargo-nextest@0.9.143`)
and the one on the development host:

- nextest has no "print the N slowest tests" option (`cargo nextest run --help`: only
  `--status-level` and `--final-status-level`, neither sorts). So a short printer reads the JUnit
  report. It is not Rust because the test jobs compile nothing (`ci_lanes.rs`,
  `the_test_shards_run_the_archives_two_jobs_build_side_by_side_and_compile_nothing`) and the
  archives hold no `ess-xtask` binary. It prints and always exits 0, so it is a report, not a
  checker.
- The keys `slow-timeout`, `final-status-level` and `[profile.<name>.junit] path` are accepted
  (nextest warns about an unknown key, and it warned only about a deliberately wrong one).
- The JUnit report lands at `target/nextest/ci/junit.xml` relative to the workspace root, both for
  a normal run and for an archive run with `--workspace-remap . --extract-to .`.
- The `[[profile.default.overrides]]` already in `.config/nextest.toml` (the Firefox test group,
  the `threads-required` override) still apply under `--profile ci`
  (`cargo nextest show-config test-groups --profile ci` lists them as "override for default
  profile").
- In go-task, a `defer:` entry registers when reached and runs when the task ends, also after a
  failed command, and the task still fails with the original error. So the `defer` must come
  BEFORE the `cargo nextest run` command. Verified with go-task 3.54; CI installs 3.52.0, which
  has `defer`.
- `ci_lanes.rs`'s `shell_commands` keeps only string commands, so a `defer:` mapping does not change
  the four commands of `test-shard` or the one of `test-feature-off` it destructures. It requires
  the `test-feature-off` command to START with
  `cargo nextest run --archive-file {{.NEXTEST_ARCHIVES}}/feature-off.tar.zst`, so `--profile ci`
  goes later in that line. In all three tasks it goes immediately before `--no-fail-fast`.

**Report, not kill (decided).** The tests over 60 s on CI are not known: job logs cannot be read,
and only one is known from a local run (`ess-conformance` `adversary_process_target_pass2`, test
`the_single_pass_reader_reads_generated_answers_as_the_reader_it_replaced`, 329 s, 2026-11-21).
With `terminate-after = 1`, any untimed test over 60 s would turn the wave's one CI run red. The
kill is the next unit, written from the JUnit reports this unit's CI run uploads.

### Steps

1. **`.config/nextest.toml`** (19 lines at `origin/main`, ending with the
   `the_cost_of_one_document_stays_within_reach_of_linear_in_the_registry` override). Append at the
   end of the file, after one blank line, exactly this, with no comment lines:

   ```toml
   [profile.ci]
   slow-timeout = { period = "60s" }
   final-status-level = "slow"

   [profile.ci.junit]
   path = "junit.xml"
   ```

   Change nothing else in the file. Do not set `status-level`, `fail-fast`, `retries` or
   `terminate-after`.

2. **`Taskfile.yml`, task `test-packages`** (lines 101-131 at `origin/main`).
   - Insert after line 118 (`      - task: clap-shared-binary`) and before line 119 (`      - |`):
     ```yaml
           - defer: { task: test-slowest }
     ```
   - Line 126, today `          cargo nextest run --locked --no-fail-fast $args`, becomes
     `          cargo nextest run --locked --profile ci --no-fail-fast $args`.

3. **`Taskfile.yml`, task `test-shard`** (lines 305-326).
   - Insert after line 325 (`      - rm {{.NEXTEST_ARCHIVES}}/workspace-{{.GROUP}}.tar.zst`) and
     before line 326:
     ```yaml
           - defer: { task: test-slowest }
     ```
   - Line 326 becomes (one line; only `--profile ci ` is added before `--no-fail-fast`):
     `      - cargo nextest run --binaries-metadata target/nextest/binaries-metadata.json --cargo-metadata target/nextest/cargo-metadata.json --workspace-remap . --target-dir-remap target --profile ci --no-fail-fast --partition slice:{{.PART}}`

4. **`Taskfile.yml`, task `test-feature-off`** (lines 339-347).
   - Insert after line 346 (`    cmds:`) and before line 347:
     ```yaml
           - defer: { task: test-slowest }
     ```
   - Line 347 becomes (one line):
     `      - cargo nextest run --archive-file {{.NEXTEST_ARCHIVES}}/feature-off.tar.zst --workspace-remap . --extract-to . --extract-overwrite --profile ci --no-fail-fast --partition slice:{{.SHARD}}`

5. **`Taskfile.yml`, new task `test-slowest`**, inserted after the `test-feature-off` task and
   before `  test-feature-off-doc:` (line 349 at `origin/main`), with one blank line on each side:

   ```yaml
     test-slowest:
       desc: Print the ten slowest tests of the last nextest run under the `ci` profile, from its JUnit report; prints and never fails.
       cmds:
         - |
           export LC_ALL=C
           report=target/nextest/ci/junit.xml
           if [ ! -f "$report" ]; then
             echo "test-slowest: no JUnit report at $report"
             exit 0
           fi
           echo "the ten slowest tests of this run ($report):"
           grep -o '<testcase [^>]*>' "$report" \
             | sed -E 's/^.* name="([^"]*)" classname="([^"]*)".* time="([0-9.]+)".*$/\3 \2 \1/' \
             | sort -rn \
             | head -n 10 \
             | awk '{ printf "%10.3fs  %s %s\n", $1, $2, $3 }'
   ```

   This exact script was run against a 12-case report and printed the output in `## Tests first`,
   test 5. Do not change the task `test`, `test-xtask`, `test-doc` or any other task.

6. **New file `crates/edge/ess-xtask/tests/test_budget.rs`**, with the tests of `## Tests first`.
   Required skeleton (fill in the test bodies as specified there):

   ```rust
   //! The runner half of the test budget (the organization rule `Tests stay fast`): nextest names
   //! every test that runs over 60 s, and each task that runs nextest prints its ten slowest tests
   //! from the JUnit report of the `ci` profile, also when the run failed.

   use std::fs;
   use std::path::{Path, PathBuf};
   use std::process::Command;

   use ess_yaml::Value;

   /// Where nextest writes the `ci` profile's JUnit report, relative to the workspace root.
   const REPORT: &str = "target/nextest/ci/junit.xml";

   /// The tasks that run `cargo nextest run` today.
   const NEXTEST_TASKS: [&str; 3] = ["test-feature-off", "test-packages", "test-shard"];

   fn workspace_root() -> PathBuf {
       Path::new(env!("CARGO_MANIFEST_DIR"))
           .ancestors()
           .find(|directory| {
               fs::read_to_string(directory.join("Cargo.toml"))
                   .is_ok_and(|manifest| manifest.contains("[workspace]"))
           })
           .expect("a workspace manifest stands above this crate")
           .to_path_buf()
   }

   fn taskfile() -> Value {
       let text = fs::read_to_string(workspace_root().join("Taskfile.yml")).expect("Taskfile.yml");
       ess_yaml::from_str(&text).expect("Taskfile.yml parses")
   }

   fn nextest_config() -> String {
       fs::read_to_string(workspace_root().join(".config/nextest.toml"))
           .expect(".config/nextest.toml")
   }

   /// The non-blank, non-comment lines of the TOML table `header` (such as `[profile.ci]`), up to
   /// the next line that starts a table.
   fn table(config: &str, header: &str) -> Vec<String> {
       let mut lines = config.lines().map(str::trim);
       assert!(
           lines.by_ref().any(|line| line == header),
           ".config/nextest.toml has no `{header}` table"
       );
       lines
           .take_while(|line| !line.starts_with('['))
           .filter(|line| !line.is_empty() && !line.starts_with('#'))
           .map(str::to_owned)
           .collect()
   }

   /// The string commands of `task`, in order (a `task:` or `defer:` entry is a mapping and is left out).
   fn string_commands(taskfile: &Value, task: &str) -> Vec<String> {
       taskfile["tasks"][task]["cmds"]
           .as_sequence()
           .into_iter()
           .flatten()
           .filter_map(|command| command.as_str().map(str::to_owned))
           .collect()
   }

   /// Whether `command` passes `flag` followed by exactly `value` as the next word.
   fn passes(command: &str, flag: &str, value: &str) -> bool {
       let words: Vec<&str> = command.split_whitespace().collect();
       words.windows(2).any(|pair| pair[0] == flag && pair[1] == value)
   }

   /// Every task whose string commands run `cargo nextest run`, by name.
   fn nextest_tasks(taskfile: &Value) -> Vec<String> {
       taskfile["tasks"]
           .as_mapping()
           .expect("Taskfile.yml has tasks")
           .keys()
           .filter_map(|key| key.as_str())
           .filter(|name| {
               string_commands(taskfile, name)
                   .iter()
                   .any(|command| command.contains("cargo nextest run"))
           })
           .map(str::to_owned)
           .collect()
   }

   fn slowest_command(taskfile: &Value) -> String {
       let commands = string_commands(taskfile, "test-slowest");
       let [command] = commands.as_slice() else {
           panic!("Taskfile.yml defines no single `test-slowest` command: {commands:?}")
       };
       command.clone()
   }

   /// Runs `test-slowest`'s command with `sh` in a fresh directory under `CARGO_TARGET_TMPDIR`,
   /// holding `report` at `REPORT` when given. Returns whether it succeeded, and its stdout.
   fn run_slowest(directory: &str, report: Option<&str>) -> (bool, String) {
       fs::create_dir_all(env!("CARGO_TARGET_TMPDIR")).unwrap();
       let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
           .join("test_budget")
           .join(directory);
       let _ = fs::remove_dir_all(&root);
       fs::create_dir_all(&root).unwrap();
       if let Some(report) = report {
           let path = root.join(REPORT);
           fs::create_dir_all(path.parent().unwrap()).unwrap();
           fs::write(&path, report).unwrap();
       }
       let output = Command::new("sh")
           .arg("-c")
           .arg(slowest_command(&taskfile()))
           .current_dir(&root)
           .output()
           .expect("sh runs");
       (output.status.success(), String::from_utf8(output.stdout).unwrap())
   }
   ```

   Do not use `std::env::temp_dir()` in this file: `ess-xtask`'s scratch-leak check
   (`src/scratch_leaks.rs`, `MAKERS`) would then require the file to be listed in its suites. Keep
   `fs::create_dir_all(env!("CARGO_TARGET_TMPDIR"))` as the first line of `run_slowest`: a test
   shard's fresh `target/` has no `target/tmp`, and another unit's check refuses a write into it
   before it is created.

## Tests first

Write `crates/edge/ess-xtask/tests/test_budget.rs` with these six tests before steps 1-5. All six
are red before the change: `.config/nextest.toml` has no `[profile.ci]`, the three tasks pass no
`--profile ci` and defer nothing, and `Taskfile.yml` defines no `test-slowest`. Each reads two small
files or runs one `sh` over a 2 KB file: milliseconds, no build, no network.

1. `the_ci_profile_reports_every_test_over_sixty_seconds`
   - `table(&nextest_config(), "[profile.ci]")` has exactly two lines.
   - One is `final-status-level = "slow"`.
   - The other is `slow-timeout = { period = "60s" }` or
     `slow-timeout = { period = "60s", terminate-after = 1 }` (the second is the next unit's kill;
     accepting it now means that unit does not have to edit this test).
   - Failure message: `the nextest profile \`ci\` does not report every test over 60 s: {lines:?}`.
2. `the_ci_profile_writes_the_junit_report_test_slowest_reads`
   - `table(&nextest_config(), "[profile.ci.junit]")` equals `["path = \"junit.xml\""]`.
   - `slowest_command(&taskfile())` contains `report=target/nextest/ci/junit.xml`, which is
     `format!("report={REPORT}")`.
3. `every_nextest_run_in_the_taskfile_uses_the_ci_profile`
   - `nextest_tasks(&taskfile())` contains each of `NEXTEST_TASKS` (a superset is allowed: a later
     task that runs nextest is held by the next assertion too).
   - For every task it returns, every string command containing `cargo nextest run` satisfies
     `passes(command, "--profile", "ci")`.
   - Failure message: `` `{task}` runs nextest without `--profile ci`: {command} ``.
4. `every_nextest_run_prints_its_ten_slowest_tests_even_when_it_fails`
   - For every task `nextest_tasks` returns, take its raw `cmds` sequence
     (`taskfile["tasks"][task]["cmds"].as_sequence()`).
   - The position of the first entry with `entry["defer"]["task"].as_str() == Some("test-slowest")`
     exists and is smaller than the position of the first string entry containing
     `cargo nextest run`.
   - Failure messages: `` `{task}` does not defer `test-slowest` `` and
     `` `{task}` defers `test-slowest` only after nextest runs, so a failed run prints nothing ``.
5. `test_slowest_prints_the_ten_slowest_tests_slowest_first`
   - Call `run_slowest("ten-slowest", Some(SAMPLE))`, where `SAMPLE` is this report (a `const` in
     the test file; the indentation inside it does not matter):
     ```xml
     <?xml version="1.0" encoding="UTF-8"?>
     <testsuites name="nextest-run" tests="12" skipped="0" failures="1" errors="0" uuid="x" timestamp="2026-11-22T00:12:28.633+01:00" time="99.0">
         <testsuite name="crate-a::alpha" tests="12" skipped="0" errors="0" failures="1">
             <testcase name="t01" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="1.500"/>
             <testcase name="t02" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="0.009"/>
             <testcase name="t03" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="12.250"/>
             <testcase name="t04" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="3.000"/>
             <testcase name="t05" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="61.042">
                 <failure type="test failure">killed</failure>
             </testcase>
             <testcase name="t06" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="9.900"/>
             <testcase name="t07" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="0.100"/>
             <testcase name="t08" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="10.000"/>
             <testcase name="t09" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="2.000"/>
             <testcase name="t10" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="0.500"/>
             <testcase name="t11" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="4.000"/>
             <testcase name="t12" classname="crate-a::alpha" timestamp="2026-11-22T00:12:28.633+01:00" time="0.200"/>
         </testsuite>
     </testsuites>
     ```
   - Assert success, and stdout equals exactly (11 lines, each ending in `\n`; the times are
     right-aligned to 10 characters before the `s`):
     ```text
     the ten slowest tests of this run (target/nextest/ci/junit.xml):
         61.042s  crate-a::alpha t05
         12.250s  crate-a::alpha t03
         10.000s  crate-a::alpha t08
          9.900s  crate-a::alpha t06
          4.000s  crate-a::alpha t11
          3.000s  crate-a::alpha t04
          2.000s  crate-a::alpha t09
          1.500s  crate-a::alpha t01
          0.500s  crate-a::alpha t10
          0.200s  crate-a::alpha t12
     ```
   - t02 (0.009 s) and t07 (0.100 s), the two fastest, are not printed.
6. `test_slowest_reports_a_missing_junit_report_and_succeeds`
   - `run_slowest("no-report", None)` succeeds, and stdout equals exactly
     `"test-slowest: no JUnit report at target/nextest/ci/junit.xml\n"`.

## Commands

Run in the unit's worktree, in this order.

1. `cargo test -p ess-xtask --locked --test test_budget` after writing the tests and before steps
   1-5: 6 failed.
2. After steps 1-5: `cargo test -p ess-xtask --locked --test test_budget`: 6 passed, well under a
   second of test time.
3. `cargo test -p ess-xtask --locked --test ci_lanes`: all pass, unchanged (about 12 s locally on
   2026-11-21). This proves the `defer` entries and `--profile ci` keep the existing lane pins green.
4. `cargo nextest run --locked --profile ci -p ess-yaml`: passes; the output's first lines say
   `with nextest profile: ci`, and `target/nextest/ci/junit.xml` exists afterwards. (`ess-yaml` is a
   leaf crate with a few unit tests, so this builds little.)
5. `slot task test-slowest`: prints `the ten slowest tests of this run (target/nextest/ci/junit.xml):`
   and up to ten lines of `ess-yaml` tests; exit 0.
6. `cargo clippy -p ess-xtask --all-targets --locked -- -D warnings`: no warnings.
7. `slot task fmt-check`: passes.

No byte-pin target of `AGENTS.md` can move: no generator, synthesis or generated file changes. No
format version changes.

## Files this unit owns

- `.config/nextest.toml`
- `Taskfile.yml` (the only unit of the wave that edits it)
- `crates/edge/ess-xtask/tests/test_budget.rs` (new)

Must not touch:

- `crates/edge/ess-xtask/src/**`, `crates/edge/ess-xtask/Cargo.toml`, `Cargo.lock`: no new
  dependency (no TOML crate; `table` above reads the file line by line).
- `crates/edge/ess-xtask/tests/ci_lanes.rs` (the coordinator), `target_tmpdir.rs` and
  `toolchain_pin.rs` (other units), `no_test_builds.rs`.
- `rust-toolchain.toml`, `.github/workflows/*`, `AGENTS.md`, `CHANGELOG.md`.

## For the coordinator at integration

1. **`Taskfile.yml`, other units' lines.** This unit adds `- defer: { task: test-slowest }` to
   `test-packages` after `- task: clap-shared-binary`. The target-tmp unit inserts
   `- rm -rf target/tmp` at the same place: keep both, in either order, both before the `- |` block
   that runs `cargo nextest run`. `test_budget.rs` only requires the `defer` before that block. The
   toolchain unit's `test-xtask` and `consumer-check` lines do not touch the lines this unit edits.

2. **`.github/workflows/ci.yml`, job `test`** (so the next unit, the kill at 60 s, can read every
   test's time without the job logs, which the API serves only as a zip). Insert after the step
   `Workspace tests ${{ matrix.group }} ${{ matrix.part }}` and before
   `Disk after the workspace tests`:
   ```yaml
         - name: Keep the workspace tests' JUnit report
           if: ${{ !cancelled() }}
           uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1
           with:
             name: junit-workspace-${{ strategy.job-index }}
             path: target/nextest/ci/junit.xml
             if-no-files-found: warn
             retention-days: 7
   ```
   and after the step `Feature-off tests ${{ matrix.shard }}` and before
   `Disk after the feature-off tests`:
   ```yaml
         - name: Keep the feature-off tests' JUnit report
           if: ${{ !cancelled() }}
           uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1
           with:
             name: junit-feature-off-${{ strategy.job-index }}
             path: target/nextest/ci/junit.xml
             if-no-files-found: warn
             retention-days: 7
   ```
   The name uses `strategy.job-index`, not `matrix.shard`: an artifact name may not contain `/`.
   The feature-off run overwrites the same file, so the first upload has to come before it. The
   action pin is the one `ci.yml` already uses, which
   `nextest_and_every_action_are_pinned_by_commit` requires. The plan-only change's `diff` job does
   not affect these steps.

3. **`AGENTS.md` § Gate**, a new paragraph after the plan-only paragraph ("A pull request that
   changes only the plan store runs no test job."):
   ```markdown
   **The test runner names every test over 60 s and the ten slowest.** `test-shard`,
   `test-feature-off` and `test-packages` run nextest with `--profile ci` (`.config/nextest.toml`):
   `slow-timeout = { period = "60s" }` marks a test SLOW each 60 s it runs,
   `final-status-level = "slow"` lists every such test in the run's summary, and the run writes
   `target/nextest/ci/junit.xml`, which each test job uploads as `junit-workspace-<n>` and
   `junit-feature-off-<n>`. Each of the three tasks defers `test-slowest`, which prints the ten
   slowest tests of that report, also when the run failed. The profile reports and does not kill
   yet: which tests run over 60 s on CI was not known, and `terminate-after = 1` comes with the list
   those reports give. `test-xtask` and `test-doc` run under `cargo test` and have no per-test limit
   yet. `crates/edge/ess-xtask/tests/test_budget.rs` holds the profile and the three tasks to this.
   ```

4. **`CHANGELOG.md`**, under `## [Unreleased]` → `### Changed`, after the plan-only bullet:
   ```markdown
   - CI: the test jobs run nextest under a `ci` profile that lists every test running over 60 s in the run's summary and writes a JUnit report, and each test step prints its ten slowest tests, also when it fails
   ```

5. **`ci_lanes.rs`**: nothing is required; `test_budget.rs` holds the profile and the tasks. If the
   coordinator adds the two upload steps above, a pin that each test job uploads both reports
   belongs in `ci_lanes.rs`, written by the coordinator.

## Out of scope

- Killing a test at 60 s (`terminate-after = 1`). That is the next unit, written from the JUnit
  reports of this wave's CI run. Do not add it, and do not add a per-test override list.
- A per-test limit for `test-xtask` and `test-doc`. They run under `cargo test`, which has no
  stable per-test timeout. Nextest runs no doc-tests at all. Moving `test-xtask` to nextest needs
  nextest installed in the `Checks` job (`ci.yml`, not this unit's) and has never been tried with
  one process per test. Follow-up unit.
- The 120 s bound on a whole test-running step (the story's acceptance list). Follow-up unit.
- Any edit to `ci.yml`, `ci_lanes.rs`, `AGENTS.md` or `CHANGELOG.md` (the coordinator applies the
  lines above).
- Changing the nextest version, `status-level`, the partitions, the archives or the local `task test`
  (which stays `cargo test`).
- Making any test faster, moving tests, or writing the printer in Rust or another language.

## Acceptance

- `.config/nextest.toml` ends with the `[profile.ci]` and `[profile.ci.junit]` tables of step 1
  (tests 1 and 2).
- `test-shard`, `test-feature-off` and `test-packages` pass `--profile ci` to `cargo nextest run`
  (test 3) and defer `test-slowest` before it (test 4).
- `test-slowest` prints the ten slowest tests of a JUnit report, slowest first, and exits 0, also
  without a report (tests 5 and 6; command 5).
- `cargo nextest run --profile ci` writes `target/nextest/ci/junit.xml` (command 4).
- `ci_lanes.rs` passes unchanged (command 3). Clippy and `fmt-check` are clean (commands 6 and 7).

## Depends on

nothing. The coordinator's `ci.yml` upload steps (integration item 2) depend on this unit's
profile and report path.
