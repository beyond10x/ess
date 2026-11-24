---
format: aep.planning-md/3
id: story:ess-verify-conform-synthesize-and-run-take-jobs-and-the-bytes-are-the-same-at-every-job-count
kind: story
status: active
title: ess verify conform synthesize and run take --jobs and the bytes are the same at every job count
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes
- depends_on: story:with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite
- depends_on: story:the-documents-and-help-describe-the-writer-true-conformance-entry-and-ess-composition-6
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/generated_docs_synthesize_help.rs
- confidence: cited
  path: website/docs/reference/cli.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:03:15Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:03:16Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

- `ess verify conform synthesize` and `ess verify conform run` take `--jobs <JOBS>`, a whole
  number from 1 to 256. It sets how many threads the caller readings of the synthesis may use.
- Without the flag, the count is `ESS_SYNTH_JOBS` when it is set, else 1.
  - An `ESS_SYNTH_JOBS` that is not a whole number from 1 to 256 is refused with exit 1 and one
    line that names the value.
  - `--jobs 0` or `--jobs 257` is refused by clap with exit 2.
- The flag and the variable reach the synthesis: at 2 and 4 jobs, caller readings run on other
  threads; at 1 job, none do (counted by `p3`'s `readings_off_thread`).
- At every count the suite file, standard output and the exit status are the same bytes.
- The help of both verbs says that, and `website/docs/reference/cli.md` is regenerated from it.
- `CHANGELOG.md` gets the story's one bullet.

## Why

- Parent: `story:the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes`.
  Decisions D1 A (default 1, opt in) and D4 A (the flag is `--jobs`).
- Unit `p3` built the threads behind `ess_conformance::synthesize::with_jobs`. This unit hands
  that to the adopter who runs the CLI.
- The default stays 1 until `p5`'s measurement of peak memory and CPU at 2 and 4 jobs has been
  read by the operator. Raising the default is not this wave's.

## Where

Read at `80794bbd698` (`plan/wave-2026-11-23`, the plan tree's head). Unit `w3` lands before
this one and edits two doc comments of `main.rs` (179 and 185) and two rows of `cli.md`, so
re-read every line at the merged head of `p3` and `w3`, and find each site by its name.

### `crates/edge/ess-cli/src/main.rs`

- `ConformCommand::Synthesize` (623-689): add after `synthesis_seed` (684-687):

  ```rust
  /// Run the caller readings of the synthesis on at most JOBS threads.
  ///
  /// The suite, the refusals and the notes are the same bytes at every count; only the time and
  /// the memory change. Without `--jobs`, `ESS_SYNTH_JOBS` sets the count when it is set, and
  /// otherwise it is 1.
  #[arg(long, value_name = "JOBS", value_parser = clap::value_parser!(u16).range(1..=256))]
  jobs: Option<u16>,
  ```

- `ConformCommand::Run` (774-826): the same field, with the first line reading "Run the caller
  readings of the synthesis, where the suite is synthesized rather than named by `--suite`, on at
  most JOBS threads."
- New, beside `conform_run`:
  - `fn job_count(flag: Option<u16>, variable: Option<std::ffi::OsString>) -> Result<NonZeroUsize>`:
    - the flag when given;
    - else the variable: a value that is not a whole number from 1 to 256 is
      ``bail!("ESS_SYNTH_JOBS is `{value}`; it takes a whole number from 1 to 256")``, written
      lossily;
    - else 1.

    The variable is a parameter, so that the unit tests need no `std::env::set_var`.
  - `fn with_job_count<R>(flag: Option<u16>, run: impl FnOnce() -> R) -> Result<R>`: calls
    `job_count(flag, std::env::var_os("ESS_SYNTH_JOBS"))`, then
    `ess_conformance::synthesize::with_jobs(count, run)`.
- `conform()` (3793):
  - the `Synthesize` arm (3795-3813) binds `jobs` and returns
    `with_job_count(jobs, || synthesize_suite(..))?`;
  - the `Run` arm (3840) becomes
    `command @ ConformCommand::Run { jobs, .. } => with_job_count(jobs, || conform_run(command))?`.
    `jobs` is `Copy`, so a binding after `@` compiles on 1.85.

  Both arms go through `with_job_count`, and nothing else sets the count.
- `conform_run` (4300): the destructure (4301-4316) adds `jobs: _`.
- `main` (1693) turns the error into exit 1, as for every other `bail!`.
- The library (`p3`'s `jobs()`) reads the same variable and counts an invalid value as 1. The
  CLI refuses it first, so on these two verbs the library never sees an invalid value.
- `mod tests` (6419): the two unit tests below.

### `website/docs/reference/cli.md`

- Generated by `cargo xtask cli-reference`, which writes through `std::env::temp_dir()`, so set
  `TMPDIR` to your scratch directory. Checked by `projection-check` (`Taskfile.yml` 547). The
  sections are `ess verify conform synthesize` (1492) and `ess verify conform run` (1585). Never
  edit them by hand.

### `CHANGELOG.md`

- `## [Unreleased]` / `### Added` (5). One bullet, at the end of the list. Keep both bullets at
  merge: `w3` and other units of the wave add their own.

## Tests first

- Bounds: no test over 60 s, a test step within 120 s. Wall-time figures are judged at a 1-minute
  load of 60 or less, as test time. Time each new test and name any over 5 s.
- No new test binary:
  - the integration tests go into `crates/edge/ess-cli/tests/generated_docs_synthesize_help.rs`,
    which already runs `CARGO_BIN_EXE_ess` from the workspace root (`workspace_root`, `help`);
  - the two unit tests go into `main.rs`'s `mod tests`.
- Output goes under `env!("CARGO_TARGET_TMPDIR")`, never `/tmp`.
- The model is `crates/verify/ess-conformance/tests/fixtures/caller-swap-related-tie.yaml`
  (10 scenarios, two callers, swapped runs). The integration tests read it by path from the
  workspace root. The unit tests read it with
  `include_str!("../../../verify/ess-conformance/tests/fixtures/caller-swap-related-tie.yaml")`
  and compile it as `caller_readings_made_once_work.rs` 148-153 does.

In `main.rs` `mod tests`. They show that the count reaches the threads, which byte equality
cannot: equal bytes would pass with the flag ignored.

1. `the_flag_reaches_the_threads_of_a_synthesis`. For `n` in `[1, 2, 4]`:
   `with_job_count(Some(n), || ess_conformance::synthesize::readings_off_thread(|| synthesize(&ir)).1)`.
   Assert:
   - the count at 1 is `0`;
   - the counts at 2 and 4 are equal and greater than 0. Every reading runs off the calling
     thread at any count above 1 (`p3` D4).
   - `with_job_count(Some(4), ess_conformance::synthesize::current_jobs)` is `4`.

   Red today: `with_job_count` does not exist.
2. `the_variable_reaches_the_job_count_and_the_flag_wins`. With `job_count` called directly:
   - `(None, Some("4"))` gives 4;
   - `(Some(2), Some("nonsense"))` gives 2;
   - `(None, None)` gives 1;
   - `(None, Some("0"))`, `(None, Some("257"))` and `(None, Some("two"))` are `Err`, the last
     with the message above.

   Red today: the same.

In `crates/edge/ess-cli/tests/generated_docs_synthesize_help.rs`:

3. `generated_docs_synthesize_help_names_jobs_and_its_variable`: the flattened help of
   `synthesize --help` contains `--jobs <JOBS>`, `ESS_SYNTH_JOBS` and
   `the same bytes at every count`. Red today: the help names none.
4. `generated_docs_run_help_names_jobs`: the same three strings in `run --help`. Red today.
5. `four_jobs_write_the_suite_one_job_writes`. Run `synthesize --path <fixture> --target ir
   --out <dir>/one.json --jobs 1`, then the same with `--out <dir>/four.json --jobs 4`, with
   `ESS_SYNTH_JOBS` removed from the child's environment. Assert:
   - both exit 0;
   - the two files are byte-equal;
   - both standard outputs are equal once the `--out` path is replaced. Read the first output
     before writing the comparison: if it does not name the path, compare it unchanged.

   Red today: exit 2, `unexpected argument '--jobs'`.
6. `the_variable_sets_the_job_count_and_the_flag_wins`. Run with `ESS_SYNTH_JOBS=4` and no flag,
   and with `ESS_SYNTH_JOBS=nonsense` and `--jobs 2`. Both exit 0 and write the bytes of test 5's
   `one.json`. Red today: the second exits 2.
7. `a_job_count_outside_one_to_256_is_refused`:
   - `--jobs 0` and `--jobs 257` exit 2, and standard error names `--jobs`;
   - `ESS_SYNTH_JOBS=0`, `=257` and `=two` (no flag) exit 1, and standard error contains
     ``ESS_SYNTH_JOBS is `two`; it takes a whole number from 1 to 256`` for the last;
   - nothing is written at `--out`.

   Red today: the flag cases exit 2 for the wrong reason ("unexpected argument"), so assert the
   word `--jobs` with `value` or `range` in the message; the variable cases exit 0.
8. `run_synthesizes_with_jobs`: `run --path <fixture> --target interpreted --jobs 4 --format
   json`. It has the exit status and standard output of the same command with `--jobs 1`. If the
   interpreted target refuses this model at `--jobs 1` today, use `--path examples/billing
   --target billing` instead, and name the one used. Red today: exit 2.

## Steps

1. Re-read the sites at the merged head. Record `BASE=$(git rev-parse HEAD)`.
2. Write tests 1-8 and record the red lines.
3. Add the two fields, `job_count`, `with_job_count`, the two arms and the `jobs: _`.
4. Run `TMPDIR=<scratch> cargo xtask cli-reference` (through the pool) and read the diff of
   `cli.md`: only the two `--jobs` rows and their text change. If `w3` has not landed, stop:
   it lands first (routing decision X4).
5. Write the changelog bullet (below).
6. Run the gate.
7. One commit with the message file written by the Write tool, `git commit -F <file>`:
   `feat(cli): conform synthesize and run take --jobs and ESS_SYNTH_JOBS`.

The `### Added` bullet:

> `ess verify conform synthesize` and `ess verify conform run` take `--jobs <JOBS>` (1 to 256,
> default 1, or `ESS_SYNTH_JOBS` when it is set), which runs the caller readings of a synthesis on
> that many threads: each model a caller reading reuses runs on one thread, and the suite, the
> refusals and the notes are the bytes one job writes. The library offers the same as
> `ess_conformance::synthesize::with_jobs`. An `ESS_SYNTH_JOBS` that is not a whole number from 1
> to 256 is refused with exit 1. No format version changes
> (`docs/design/parallel-caller-readings.md`).

## Owned files

- `crates/edge/ess-cli/src/main.rs`: the two `jobs` fields, `job_count`, `with_job_count`, the
  two `conform()` arms, the `jobs: _` in `conform_run`, unit tests 1-2.
- `crates/edge/ess-cli/tests/generated_docs_synthesize_help.rs`: tests 3-8.
- `website/docs/reference/cli.md`: regenerated only.
- `CHANGELOG.md`: one bullet.

## Not yours

- `crates/verify/ess-conformance/**` is `p3`'s and merged; change nothing there. If a test here
  shows a byte difference between job counts, stop and report it as a `p3` defect with the
  command that shows it.
- `Mutate` gets no `--jobs`; the library still reads `ESS_SYNTH_JOBS` there.
- `ess check validate --format json`, which also synthesizes (`cli.md` 526), gets no flag either.
- `crates/edge/ess-xtask` and `Taskfile.yml` are `p5`'s.
- `main.rs` 179 and 185 and their `cli.md` rows are `w3`'s.

## Shared files

- `w3` lands before this unit (routing decision X4) and shares three files:
  - `crates/edge/ess-cli/src/main.rs` (different lines: `w3` two doc comments of `Compose`, this
    unit the jobs flag);
  - `website/docs/reference/cli.md`: this unit, landing second, regenerates it with
    `cargo xtask cli-reference` and never merges it by hand;
  - `CHANGELOG.md`: keep both bullets at merge.
- `CHANGELOG.md` is also shared with every other unit of the wave that writes a bullet: keep both
  bullets at merge.

## Gate

- Each command is one pool job; `cargo` is the shim.
- Bounds: 60 s per test, 120 s per test step. A pool job ends after 10 minutes unless it names
  its time, 30 minutes at most.
- No `RUSTC_WRAPPER`, `SCCACHE_*` or `-j`. No Python. No files in `/tmp`: `TMPDIR` points at
  your scratch directory.

```console
slot task fmt-check
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-cli --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --bin ess -- tests::the_flag_reaches_the_threads_of_a_synthesis tests::the_variable_reaches_the_job_count_and_the_flag_wins tests::cli_reference_block
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test generated_docs_synthesize_help --test synthesize_layout --test synthesis_seeds_cli
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test tutorial_page
SLOT_TIMEOUT=30m slot task projection-check
SLOT_TIMEOUT=30m slot task test-xtask
SLOT_TIMEOUT=30m slot task ci-lint
SLOT_TIMEOUT=30m slot task site-build
```

- `test-xtask` covers the changelog and website pins.
- `site-build` is required because `website/` changes. It fetches the docs system with `npm ci`;
  if the network refuses, say so in the report rather than skipping it silently.

## Acceptance

1. Tests 1 and 2 pass: the flag and the variable reach the threads (readings off the calling
   thread above 0 at 2 and 4 jobs, 0 at 1) and the count (`current_jobs`).
2. Tests 3-8 pass, with their times reported.
3. `cargo xtask cli-reference --check` passes, and the `cli.md` diff against `w3`'s head holds
   only the two `--jobs` rows and their text.
4. `git diff $BASE -- crates/verify Cargo.lock '**/Cargo.toml'` is empty.
5. `CHANGELOG.md` has exactly one new bullet from this unit, under `### Added`, and every bullet
   that was there at `$BASE`.
6. `--jobs` absent and `ESS_SYNTH_JOBS` unset: the command runs exactly as before. Test 5's
   `one.json` equals the suite of the same command at `$BASE`. Compare once by hand with the
   `$BASE` binary and name the result.

## Decisions

- D1. Clap has no `env` feature in this workspace (`Cargo.toml`). Reading the variable in
  `with_job_count` avoids a new feature and keeps one rule for the CLI and the library.
- D2. An invalid variable is refused (exit 1) rather than ignored. An adopter who sets it
  wrongly in CI would otherwise run on one thread unawares. The flag's range error stays clap's
  (exit 2), like every other flag.
- D3. `u16` with a clap range, not `NonZeroUsize`. The help shows the bound, and 256 is far above
  any runner's core count while bounding thread stacks at 2 GiB.
- D4. Only these two verbs get the flag (D3 A of the parent: caller readings only, where the
  profile measured the cost).
- D5. The wiring is observed in unit tests of `with_job_count`, the one function both arms call,
  through `p3`'s counter. A process test cannot see a thread count after the process ends, and
  printing one would change standard output.

## Changelog

The bullet under Steps, under `## [Unreleased]` / `### Added`. It is the parent story's only
changelog entry. Keep both bullets at merge.
