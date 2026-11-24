---
format: aep.planning-md/3
id: story:with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite
kind: story
status: active
title: With more than one job the caller readings of a synthesis run on scoped threads and write the same suite
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes
- depends_on: story:every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan
- depends_on: story:every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads
- depends_on: story:a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it
scope:
- confidence: cited
  path: crates/edge/ess-xtask/tests/slow_lane.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/authored/ties.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/fixtures.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/input.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/now_offset.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/aggregate/contrast.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/caller.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/caller/readings.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/created_memo.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/disclosure.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/earlier_runs.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/first_kept.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/grant.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/made.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/model_memo.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/ordered_head.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/run_memo.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/shorthand.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/threads.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/a_synthesis_seed_is_accepted_for_a_caller_model.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/caller_arranged_rows.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/caller_readings_made_once_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-component-suites-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/mutation_grants.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs
revision: 21
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:03:12Z", actor: "human:timo", revision: 20, decided_on: {"recorded":{"review_outcome":7}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:03:13Z", actor: "human:timo", revision: 21, decided_on: {"recorded":{"review_outcome":7}}}
---
## Outcome

- `ess_conformance::synthesize::with_jobs(jobs, run)` sets how many threads the caller readings of
  every synthesis inside `run` may use on this thread. Without it, the count is `ESS_SYNTH_JOBS`
  when that holds a whole number from 1 to 256, else 1.
- At 1, nothing changes from unit `p1`: no thread is started.
- Above 1, `readings::read_all` runs the plan on scoped threads from the standard library:
  - each uniform model's readings run, in plan order, on one thread that keeps that model's memory
    (its **lane**);
  - every other reading goes to whichever thread is free;
  - results are merged by plan index on the calling thread.
- These are the bytes one job writes (suite, refusals, notes, outside), as tested:
  - at 1, 2, 4 and 7 jobs on five caller fixtures, in the test step;
  - at 1, 2 and 7 jobs on every repository model and every component the byte tables walk, in
    the slow lane;
  - at 4 jobs for a seeded run and a component run of a caller model;
  - at 4 jobs for the mutants of the sets-drop and grant classes.
- A panic in a reading on another thread is resumed on the calling thread with its own payload,
  the same panic one job raises.
- The work counted on a worker is added to the calling thread. The 14 `*_work.rs` targets pass at
  `ESS_SYNTH_JOBS=1` and at `ESS_SYNTH_JOBS=4`.
- On a generic caller model of 12 commands read differently by two callers, 50 = 4 x 12 + 2
  readings run on another thread at 4 jobs, and 0 at 1 job.
- No new dependency, no `unsafe`, no format change. The CLI flag is unit `p4`'s.

## Why

- Parent: `story:the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes`.
  This is the thread unit. Its decisions: D2 C (stage 2 on two threads, one per uniform model,
  each owning its model's memory) and D3 A (caller readings only).
- Consumer gap: one synthesis of the largest known downstream specification takes 279 s on one
  core of a many-core CI runner.
- Units `p1` (the plan, inline) and `p2` (context, counters, registry) are merged before this
  starts. This unit changes who runs the plan, and nothing else.
- The parent's acceptance asks for a byte-identity test over the committed model corpus at 1, 2
  and 7 threads. Over every model that is several minutes of synthesis, so it is a slow-lane test,
  using the mechanism of unit `t1a`. The five fixtures run in the test step.

## Where

Read at `80794bbd698` (`plan/wave-2026-11-23`, the plan tree's head). `p1` and `p2` change the
files below first, so **re-read them at the wave's merged head before you start**, and find each
site by its name. Record `BASE=$(git rev-parse HEAD)`.

### `crates/verify/ess-conformance/src/synthesize.rs`

Beside `syntheses_started` (3498-3509), add:

```rust
thread_local! {
    /// The job count [`with_jobs`] set on this thread; `None` outside one.
    static JOBS: std::cell::Cell<Option<std::num::NonZeroUsize>> = const { std::cell::Cell::new(None) };
    /// How many caller readings ran on a thread other than the one that planned them.
    static OFF_THREAD: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Runs `run` with the caller readings of every synthesis in it on at most `jobs` threads.
/// The bytes a synthesis writes do not depend on `jobs`.
pub fn with_jobs<R>(jobs: std::num::NonZeroUsize, run: impl FnOnce() -> R) -> R { /* replace, restore on drop, also on a panic */ }

/// The job count: [`with_jobs`]'s, else `ESS_SYNTH_JOBS` where it is a whole number from 1 to
/// 256, else 1.
pub(super) fn jobs() -> std::num::NonZeroUsize { /* as stated */ }

/// What `run` returns, and how many caller readings ran on another thread while it ran. Not
/// part of the suite.
#[doc(hidden)]
pub fn readings_off_thread<R>(run: impl FnOnce() -> R) -> (R, u64) { /* before/after of OFF_THREAD */ }
```

Also add:

- `pub(super) fn add_off_thread(more: u64)`;
- `#[doc(hidden)] pub fn current_jobs() -> std::num::NonZeroUsize { jobs() }`. Unit `p4`'s test
  reads it to show that the flag reaches the count.

### `crates/verify/ess-conformance/src/synthesize/caller/readings.rs` (`p1`'s)

Add the thread bounds that `p1` left out (routing decision X3) to the signature:

```rust
pub(super) fn read_all<'m, J: Sync, R: Send>(
    jobs: &[J],
    lane: impl Fn(&J) -> Option<&'m Arc<EssIr>> + Sync,
    read: impl Fn(&J) -> R + Sync,
) -> Vec<R>
```

Add the compile-time guard at the end of the file:

```rust
const _: () = {
    const fn sync<T: Sync>() {}
    sync::<super::Assigned<'static>>();
    sync::<super::Callers>();
};
```

If a bound fails to compile, the error names the type that holds an `Rc` or a `Cell`. Report it,
and replace it only where it is a plain data field and not a memo.

Replace the body of `read_all`:

1. Let `threads = super::super::jobs().get()`. If `threads == 1` or `jobs.len() < 2`, return
   `jobs.iter().map(read).collect()`. That is `p1`'s body.
2. Group the job indices:
   - `lanes: Vec<(&Arc<EssIr>, Vec<usize>)>`: one entry per distinct `lane(job)`, compared with
     `Arc::ptr_eq`, in order of first appearance. Each entry holds its jobs' indices in plan
     order.
   - `pool: Vec<usize>`: every job whose `lane` is `None`, in plan order.
3. `count = lanes.len().max(threads.min(lanes.len() + pool.len()))`. So every lane has a thread
   of its own, and the count never exceeds the jobs to run.
4. Capture `let context = super::super::threads::Context::capture();` on the calling thread.
   Under `#[cfg(test)]`, also capture the panic seam (below).
5. In `std::thread::scope`, spawn `count` threads with
   `std::thread::Builder::new().name(format!("ess-synth-{t}")).stack_size(8 << 20).spawn_scoped(scope, ..)`.
   Each thread runs inside `context.enter(..)`:
   1. If it has a lane (`t < lanes.len()`), it opens `Kept::open(&[model])`, runs that lane's
      jobs in order, and drops the `Kept`.
   2. Then it takes pool jobs from a shared `std::sync::atomic::AtomicUsize` cursor
      (`fetch_add(1, Ordering::Relaxed)`) until the cursor passes `pool.len()`.
   3. Every job runs as `std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| read(&jobs[i])))`.
      The thread keeps `(i, result)` and returns them with `threads::Work::taken()`.
6. Join every handle in thread order. If a `join` itself is `Err`, the panic escaped a job (for
   example from `Kept::open`): `std::panic::resume_unwind` it. Otherwise:
   - `work.add()` on the calling thread;
   - place each result at its index;
   - count the jobs run;
   - `super::super::add_off_thread(count)`.
7. If any result is `Err(payload)`, `resume_unwind` the payload with the lowest job index.
   Otherwise return the results in index order.

At every count, every job runs to its end, also after another job has panicked. So the panic
surfaced is always the lowest-indexed one, as on one thread.

Under `#[cfg(test)]`, add the panic seam:

```rust
#[cfg(test)]
thread_local! {
    /// The command whose first `About` reading panics, for the panic test only.
    pub(super) static PANIC_IN: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}
```

- `read` checks it first: for `Reading::About { command, back: false, .. }` naming it, it calls
  `panic!("a test panic in the reading of `{command}`")`.
- The executor captures its value and sets it in each worker, next to `context.enter`.

### `crates/verify/ess-conformance/src/synthesize/caller.rs` (`p1`'s `synthesize`)

- The `let _kept = readings::Kept::open(..)` that `p1` added becomes
  `let _kept = (super::jobs().get() == 1).then(|| readings::Kept::open(&assigned.uniform_models()));`.
  Above one job the lanes own that memory, and the readings that run inline at merge time
  (`one_row_acted_on`, the rotation re-run) start from none.
- `mod tests`:
  - `counted` (3653) runs its synthesis inside `super::super::with_jobs(NonZeroUsize::MIN, ..)`,
    because `WHOLE` counts on the thread that runs a whole reading;
  - add the panic test below.

### `crates/verify/ess-conformance/src/witness_memo.rs`

- `counted` (421-428, `#[cfg(test)]`): run `run` inside `crate::synthesize::with_jobs(NonZeroUsize::MIN, ..)`.
  Its `WORK` recorder and `GENERATIONS` are per thread, and its key uses the model's address.

### `crates/verify/ess-conformance/src/synthesize/threads.rs` (`p2`'s)

- Add to `STATE`:
  - `synthesize.rs` `JOBS` and `OFF_THREAD` as `Caller`. They are only read and set on the
    calling thread.
  - `synthesize/caller/readings.rs` `PANIC_IN` as `Test`.
- Remove the `#![cfg_attr(not(test), allow(dead_code))]` that `p2` put at the top of the file,
  with its comment naming this unit. `read_all` now uses `Context` and `Work`, and through them
  every accessor.
- Remove every other line that carries the comment `p3 removes this`, in whichever file of
  `crates/verify/ess-conformance/src/` `p2` left it (`git grep -n 'p3 removes'
  crates/verify/ess-conformance/src`; `p2`'s report lists those files). Change nothing else in
  those files (routing decision Y2).

### `crates/edge/ess-xtask/tests/slow_lane.rs` (`t1a`'s)

- Add one row to `MARKED: &[(&str, &[&str])]`: `("crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs", &["every_repository_model_and_component_writes_its_bytes_at_one_two_and_seven_jobs"])`.
  The path is workspace-relative and the rows are sorted by path, as `t1a` sets the table.
- If `t2` already put a row for the same file (its own slow tests), add the name to that row's
  list rather than adding a second row: one row per file. `t1a` names this unit among the units
  that add rows; keep every row at merge (routing decision Y1).
- `Work` is unchanged.

### `crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs`

- The corpus walker and the line makers are reused unchanged:
  - `models(&root())` (135): every ESS source file, and every directory with a `system.yaml`;
  - `model_lines` (205): stage, or suite count and digest, refusals, notes, outside;
  - `component_lines` (230): one line per component;
  - `UNAFFORDABLE`.
- Unit `t2` edits this file after this unit (routing decision X1: `t2` lands after `p3`). This
  unit only appends one test function at the end of the file (380), and `t2` keeps it.

## Tests first

- Time each new test in a debug build and name every one over 5 s with its time.
- Wall-time figures are judged at a 1-minute load of 60 or less (`cut -d' ' -f1 /proc/loadavg`),
  on a warm build, as test time (the runner's figure, not the command's wall time).
- No test of the test step may run over 60 s. If one does, cut N = 7 first, then the largest
  model of that test, which you name. A test step stays within 120 s.
- No new test binary: every test goes into an existing file.
- All new tests are red today with `error[E0432]: unresolved import
  ess_conformance::synthesize::with_jobs` (or `readings_off_thread`), unless stated otherwise.

In `crates/verify/ess-conformance/tests/caller_readings_made_once_work.rs`:

- Refactor `fn model(caller: bool)` (36) into `fn model_flags(flags: usize, caller: bool)` with
  `FLAGS` replaced by `flags`, and `fn model(caller: bool) -> String { model_flags(FLAGS, caller) }`.
  The existing test is unchanged.
- Helper `fn rendered(synthesis: &Synthesis) -> String` returns
  `suite.to_canonical_json().unwrap()`, then the refusals, the notes and the outside, each joined
  with `"\n"` as they print (`Display`), each block under a `--- <name>` line.
- Helper `fn same_at_every_job_count(label: &str, text: &str)`. For `n` in `[1, 2, 4, 7]` it
  runs `with_jobs(NonZeroUsize::new(n).unwrap(), || synthesize(&ir(text)))` and asserts
  `rendered` equals that of `n = 1`. On failure, the message names the label and `n`, and
  prints the first differing line.

1. `a_two_caller_swap_with_a_related_tie_writes_one_suite_at_every_job_count`:
   `include_str!("fixtures/caller-swap-related-tie.yaml")`. 10 scenarios, swapped runs.
2. `a_caller_addressed_identity_writes_one_suite_at_every_job_count`:
   `fixtures/caller-addressed-identity.yaml`. 11 scenarios.
3. `a_guard_beside_creating_rows_writes_one_suite_at_every_job_count`:
   `fixtures/list-identity-guard-beside-creates.yaml`. 11 scenarios.
4. `a_one_time_follow_up_guarded_by_its_caller_writes_one_suite_at_every_job_count`:
   `fixtures/one-time-followup/caller-guard.yaml`. 13 scenarios; it reads `Readings` for
   follow-up cells.
5. `the_card_model_writes_one_suite_at_every_job_count`: `model(true)`. Six mixed commands, two
   of them in `rewritten`, so it has both lane readings and pool readings.

   These five are taken from the repository's caller fixtures with the bytes table's own counts
   (`synthesis-suite-bytes-table.tsv`). They are small enough for a debug build. Each exercises a
   different part of `caller.rs`.
6. `twelve_commands_read_by_two_callers_run_their_readings_off_the_calling_thread`:
   `model_flags(10, true)` has 12 commands, all granted to the attributed actor.
   - `readings_off_thread(|| with_jobs(4, || synthesize(..)))` gives `50 == 2 + 4 * 12`: two
     whole readings, two focused and two cross runs per command, no rotation, no one-row re-run;
   - `with_jobs(1)` gives `0`;
   - the two `rendered` are equal.

   The parent's acceptance names this count. If the test runs over 30 s, lower `10` to the
   largest value under 30 s and write the expected count as `2 + 4 * (flags + 2)`.
7. `the_creating_rows_counted_at_four_jobs_are_those_counted_at_one`: `creations_made` of
   `model(true)` is equal at `with_jobs(1)` and `with_jobs(4)`, and both are
   `<= TIMES * plain`. The lanes give each uniform model the sequence of readings one thread
   gives it, so the count is not a bound but an equality.
8. `a_sets_drop_mutants_suites_are_the_same_at_four_jobs`. The documents of `model(true)`
   (`vec![(Source::new("board.yaml"), RawSpecFile::parse(&text)?)]`) and a `SourceMap` holding
   the text. Run `ess_conformance::mutate::emit(&documents, &texts, &[MutantClass::SetsDrop])`
   at `with_jobs(1)` and `with_jobs(4)`. Assert:
   - `emission.files` are equal;
   - at least one key of `files` names a `sets-drop/` mutant's suite. `Mark<i>`'s
     `flag_<i>: input.on` is a site.

   `with_dropped_write` is set around each mutant's synthesis (`mutate.rs` 5643, and in `emit`
   6890), and readings read it in `unread_apart` and in five memo keys (`dropped_key`,
   `synthesize.rs` 14705).

In `crates/verify/ess-conformance/tests/mutation_grants.rs`:

9. `a_grant_mutants_suites_are_the_same_at_four_jobs`. Build the text with `with_actors` (342),
   using the actors block of `the_added_grant_scenario_is_the_mutants_alone` (506-515): the
   `Librarian` with attributes holding `CatalogueBook`, and the `Reader` without them. Run
   `mutate::emit` with `[MutantClass::GrantDrop, MutantClass::GrantAdd]` at 1 and 4 jobs. Assert:
   - `files` are equal;
   - the manifest lists a `grant-drop/` and a `grant-add/` mutant.

   `with_dropped_grant` and `with_added_grant` (`mutate.rs` 2314-2318) are read inside readings
   by `grant::denied` (`synthesize.rs` 3956, `grant.rs` 81).

In `crates/verify/ess-conformance/tests/caller_arranged_rows.rs`:

10. `the_arranged_callers_stage_writes_the_same_bytes_at_four_jobs`: `TOKENS`, `affecting()`
    and `related()` (`ir` 181) at 1 and 4 jobs, `rendered` equal. Also assert that the suite
    at 4 jobs holds the scenario `a_set_effect_over_a_caller_written_field_is_witnessed` (252)
    asserts on: read its id there. That scenario is adopted by `arranged_by_callers`, so the
    arranged stage ran.

In `crates/verify/ess-conformance/tests/a_synthesis_seed_is_accepted_for_a_caller_model.rs`:

11. `a_seeded_and_a_component_synthesis_write_the_same_bytes_at_four_jobs`, on `model(ATTR)`:
    - `synthesize_with_seeds(&ir, &seeds(&ir, &["flagged"]))` at 1 and 4 jobs: suite JSON,
      refusals and notes equal;
    - `ess_conformance::synthesize::synthesize_for(&ir, "svc")` at 1 and 4: the same.

    The component scope (`SCOPE`) and the seeds cross to the workers.

In `crates/verify/ess-conformance/src/synthesize/caller.rs` `mod tests`:

12. `a_panic_in_a_reading_on_another_thread_is_the_panic_one_thread_raises`. Set
    `readings::PANIC_IN` to `Some("demo.notes.CloseNote")`. Run
    `std::panic::catch_unwind(|| with_jobs(1, || super::super::synthesize(&compiled(NOTES))))`
    and the same at 4 jobs, then reset `PANIC_IN` to `None`. Assert:
    - both are `Err`;
    - both payloads `downcast_ref::<String>()` to
      ``a test panic in the reading of `demo.notes.CloseNote` ``.

    Red today: `PANIC_IN` and `with_jobs` do not exist.

In `crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs`, a slow-lane test. It
carries `t1a`'s marker, `#[test] #[ignore = "slow lane: <reason>"]`, as
`#[ignore = "slow lane: every model and component synthesized at 1, 2 and 7 jobs, about <n> s"]`
with the measured seconds. It runs in `task test-slow`, and its `MARKED` row is this unit's:

13. `every_repository_model_and_component_writes_its_bytes_at_one_two_and_seven_jobs`. For `n`
    in `[1, 2, 7]`, inside `with_jobs(NonZeroUsize::new(n).unwrap(), ..)`, build:
    - `models(&root())` mapped through `model_lines`;
    - `models(&root())` mapped through `component_lines`.

    Call `models` once per count, so no `Model` needs to be `Clone`. Assert that the lines at 2
    and at 7 equal the lines at 1, model by model. On failure, print the label, the count and
    both lines.

    - `UNAFFORDABLE` models stay excluded as the tables exclude them.
    - A line holds the suite's count and SHA-256 digest and the full refusals, notes and outside
      as they print. Equal digests stand for `cmp` of the suite bytes (D8).
    - Red today: the same compile error.
    - It runs in the slow lane only: never in the test step, never under the 60 s bound of the
      test step. `t1a`'s slow profile kills a test at 20 minutes. Report its time.
    - The parent asks for the corpus plus the caller models of the work tests. Those models are
      generated in code, and tests 5-7 cover them in the test step.

### The mutation proof (required, and reported)

1. Temporarily replace `context.enter(..)` in the worker by a direct call. Run tests 8, 9 and 11,
   record which go red and the first differing line, and restore.
2. Then, one at a time, remove one setter from `Context::enter`, run its test, and restore:
   - `with_dropped_write` for test 8;
   - `with_dropped_grant` and `with_added_grant` for test 9;
   - `scoped_to` for test 11.

What a green mutant means:

- Tests 8 and 9 must go red under step 1, and under their setters in step 2. If one stays green,
  its model does not make a worker read that context. Change the model until it does, and say
  how. Examples: a write that a focused reading of a rewritten command drops; a grant of a command
  that only the attributed actor holds.
- If no model in reach turns it red, stop and report the unit as not accepted: a mutation test
  that cannot fail proves nothing.
- Test 11 under `scoped_to` may stay green, if the component `svc` takes every command of the
  model. Then say so, and add a second component to `model(ATTR)` that takes only one command, so
  that the scope changes what a worker synthesizes. If test 11 still stays green, report it.

## Steps

1. Re-read `readings.rs`, `caller.rs` `synthesize`, `threads.rs`,
   `synthesis_suite_bytes_table.rs` and `MARKED` in `crates/edge/ess-xtask/tests/slow_lane.rs` at
   the merged head.
2. Write tests 1-13 and record the red lines. Every target runs through the pool. With test 13's
   marker, add its `MARKED` row, so that `cargo test -p ess-xtask --test slow_lane` stays green.
3. Add `with_jobs`, `jobs`, `current_jobs`, `readings_off_thread` and the two thread-locals.
   Classify them in `STATE`. Remove every line with the comment `p3 removes this`
   (`git grep -n 'p3 removes' crates/verify/ess-conformance/src`).
4. Add the bounds and the guard to `read_all`. Then write the threaded body, the panic seam and
   the `jobs() == 1` condition on `_kept`.
5. Run tests 1-12, then the mutation proof, then test 13 in the slow lane.
6. Run the gate.
7. One commit, message file via the Write tool, `git commit -F <file>`:
   `feat(conformance): caller readings run on scoped threads with the bytes one thread writes`.
   The body names the job counts tested, test 13's time and the mutation proof's red lines.

No generated file changes. `with_jobs`, `current_jobs` and `readings_off_thread` are new public
API, so `task ci-lint` runs.

## Owned files

- `crates/verify/ess-conformance/src/synthesize/caller/readings.rs`: `read_all`'s bounds and body,
  the guard and the panic seam. `p1` created the file and is merged.
- `crates/verify/ess-conformance/src/synthesize.rs`: `JOBS`, `OFF_THREAD`, `with_jobs`, `jobs`,
  `current_jobs`, `readings_off_thread`, `add_off_thread` only.
- `crates/verify/ess-conformance/src/synthesize/caller.rs`: the `_kept` line, `counted`, test 12.
- `crates/verify/ess-conformance/src/witness_memo.rs`: `counted` only.
- `crates/verify/ess-conformance/src/synthesize/threads.rs`: three `STATE` rows, and the removal
  of `p2`'s module-level dead-code allow.
- Every line carrying the comment `p3 removes this`, in whichever file of
  `crates/verify/ess-conformance/src/` `p2` left it (`git grep -n 'p3 removes'`); nothing else in
  those files.
- The four test files named under Tests first: the new tests and the `model_flags` refactor only.
- `crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs`: test 13 only, appended.
- `crates/edge/ess-xtask/tests/slow_lane.rs`: the `MARKED` row (or name) for test 13 only.

## Not yours

- `crates/edge/ess-cli` and `website/` belong to `p4`; `crates/edge/ess-xtask` (except test 13's `MARKED` row), `Taskfile.yml`
  and the design page's `## Measured` belong to `p5`.
- The byte tables (`tests/fixtures/*-table.tsv`) and every other line of
  `synthesis_suite_bytes_table.rs` belong to `t2`, which lands after this unit. They are read and
  run here, never re-pinned.
- The slow-lane marker, its scanner, its task and every other row of `MARKED` belong to `t1a`
  and to the `t` units that add rows.
- `mutate.rs`: nothing here edits it.

## Shared files

- `synthesis_suite_bytes_table.rs` with `t2`. This unit lands first and appends one function;
  `t2` keeps it.
- `crates/edge/ess-xtask/tests/slow_lane.rs` with `t1a` (merged before) and `t2` to `t5`, which
  add rows too: keep every row at merge, one per file, sorted by path.
- `threads.rs` with `p2` (merged before).

## Gate

- Each command is one pool job; `cargo` is the shim.
- Bounds: 60 s per test, 120 s per test step. A pool job ends after 10 minutes unless it names
  its time, 30 minutes at most.
- No Python, no files in `/tmp`, no `RUSTC_WRAPPER`, `SCCACHE_*` or `-j`.

```console
slot task fmt-check
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-conformance --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-cli --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --lib -- synthesize::caller synthesize::threads witness_memo
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test caller_readings_made_once_work --test mutation_grants --test caller_arranged_rows --test a_synthesis_seed_is_accepted_for_a_caller_model
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test caller_readings_made_once_work --test companion_rows_searched_once_work --test created_rows_kept_work --test nested_paths_extended_in_place_work --test once_per_model_work --test own_scenario_nesting_bounded_work --test ranked_and_shorthand_work --test related_guard_ranked_candidates_work --test row_set_many_selectors_work --test rows_moved_by_many_commands_work --test step_clones_work --test stored_row_bound_once_work --test stored_rows_decided_once_work --test witness_unread_choices_work
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 ESS_SYNTH_JOBS=4 cargo test -p ess-conformance --locked --test caller_readings_made_once_work --test companion_rows_searched_once_work --test created_rows_kept_work --test nested_paths_extended_in_place_work --test once_per_model_work --test own_scenario_nesting_bounded_work --test ranked_and_shorthand_work --test related_guard_ranked_candidates_work --test row_set_many_selectors_work --test rows_moved_by_many_commands_work --test step_clones_work --test stored_row_bound_once_work --test stored_rows_decided_once_work --test witness_unread_choices_work
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 ESS_SYNTH_JOBS=4 cargo test -p ess-conformance --locked --test caller_arranged_rows --test a_seeded_scenario_notes_its_skipped_reversed_caller_run --test caller_values --test interpreted_caller --test one_time_caller --test mutation_grants --test mutation_sets_entries
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table --test synthesis_suite_bytes_table_cli_read --test adversary_related_values_all_bytes --test external_beside_held_guard
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-xtask --locked --test slow_lane
SLOT_TIMEOUT=30m slot task ci-lint
SLOT_TIMEOUT=30m task test-slow PACKAGES=ess-conformance
```

- `task test-slow` runs every test marked in `ess-conformance`, test 13 among them. Report test
  13's time from its run and the 1-minute load at its start.

- The byte-pin targets run as they are today (routing decision X2). `t2` lands later.
- The `ess-cli` tests run in `p4`.

## Acceptance

Each line checks one claim of the Outcome:

1. Tests 1-5 pass: five fixtures at 1, 2, 4 and 7 jobs. Times are reported, and every one over
   5 s is named.
2. Test 13 passes in the slow lane: every repository model and component at 1, 2 and 7 jobs.
   Its time is reported.
3. Test 11 passes: a seeded run and a component run at 4 jobs.
4. Tests 8 and 9 pass, and the mutation proof turned them red with the context removed and with
   each of their setters removed. The report has the lines.
5. Test 12 passes: the same panic payload at 1 and 4 jobs.
6. The 14 `*_work.rs` targets pass at the default and at `ESS_SYNTH_JOBS=4`. Test 7 passes: an
   equal creation count at 1 and 4.
7. Test 6 passes: 50 readings off the calling thread at 4 jobs, 0 at 1.
8. The byte-pin targets pass with no table edited:
   `git diff $BASE -- crates/verify/ess-conformance/tests/fixtures` is empty.
9. `git diff $BASE -- Cargo.lock '**/Cargo.toml'` is empty: no new dependency.
10. `git grep -n 'unsafe' crates/verify/ess-conformance/src/synthesize/caller/readings.rs`
    prints nothing.
11. `every_static_of_the_crate_is_classified` (`p2`) passes with the three new rows, and
    `git grep -n 'p3 removes' crates/verify/ess-conformance/src` prints nothing.
12. `cargo test -p ess-xtask --test slow_lane` passes: test 13's marker and its `MARKED` row
    agree, and `cargo xtask slow-list` prints
    `ess-conformance\tsynthesis_suite_bytes_table\tevery_repository_model_and_component_writes_its_bytes_at_one_two_and_seven_jobs`.

## Decisions

- D1. Threads are `std::thread::scope` with `Builder::spawn_scoped`. The cursor is an
  `AtomicUsize`. There is no channel and no new crate: `Cargo.lock` holds neither `rayon` nor
  `crossbeam`, and 222 coarse jobs need no work stealing.
- D2. Worker stack is 8 MiB, the main thread's size on Linux. A reading that fits the CLI's main
  thread fits a worker; the standard library's default for spawned threads is 2 MiB.
- D3. Lanes carry the parent's D2 C to every uniform-model reading (see `p1` D3). A pool reading
  on a uniform model would start from empty memory. The card model's creating rows would rise
  from the bound's 18 toward the 34 that `caller_readings_made_once_work.rs` (25-29) records for
  fresh readings, and test 7 would fail.
- D4. The calling thread plans, waits and merges, and runs no reading of the plan itself, so
  `N` is the number of threads running readings. Above one job, it keeps no uniform model's
  memory, because the lanes keep it.
- D5. Every job runs even after one panicked, and the lowest-indexed panic is resumed. That is
  deterministic, and a panic is a defect anyway, so the cost of finishing is accepted.
- D6. The library reads `ESS_SYNTH_JOBS`, so every existing test runs at four jobs unchanged
  (the `## Gate` runs the work targets that way). A value outside 1 to 256 counts as 1 there; the
  CLI refuses it (`p4`).
- D7. No thread count above 256 is refused by the library (`with_jobs` takes any `NonZeroUsize`).
  The count is capped by the jobs to run anyway.
- D8. Test 13 compares the byte tables' own lines (count, SHA-256 of the canonical suite, full
  refusals, notes and outside). Holding three full suites of every model at once to `cmp` them
  would cost memory and add nothing that a SHA-256 does not detect.

## Open for the operator

- **The expected speed-up.** Lanes put every reading on a uniform model on one thread: each whole
  reading, every cross run, and every focused run of a command outside `rewritten`. That is about
  2.2x at 2 or more jobs on the profile (inferred), against the parent's 3.5x.
  - A (recommended, and what this unit builds): lanes. Bytes and work bounds hold as they are.
  - B: those readings go to the pool with fresh memory. More parallel, but the work bound of
    `caller_readings_made_once_work.rs` must be loosened, and CPU rises.
  - C: a frozen, read-only memory layer shared by all workers. A later story: the memo values are
    `Rc` and would have to become `Send + Sync`.

  `p5`'s table, and the coordinator's run on the largest known downstream specification, show
  what A gives.

## Changelog

None here. `p4` writes the story's bullet, which names `with_jobs`.
