---
format: aep.planning-md/3
id: story:with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite
kind: story
status: draft
title: With more than one job the caller readings of a synthesis run on scoped threads, each uniform model's readings on one thread that keeps its memory, and the suite, refusals and notes are the bytes one job writes
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes
- depends_on: story:every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan
- depends_on: story:every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/caller.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/caller/readings.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/threads.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/a_synthesis_seed_is_accepted_for_a_caller_model.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/caller_arranged_rows.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/caller_readings_made_once_work.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/mutation_grants.rs
revision: 10
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
- The suite, refusals, notes and outside are byte for byte those of one job, for every job count,
  every repository model, every component, seeded runs, and every mutant of the mutation audit.
- A panic in a reading on another thread is resumed on the calling thread with its own payload,
  the same panic one job raises.
- The work counters of every reading are added to the calling thread, so every `*_work.rs` bound
  holds at any job count.
- No new dependency, no `unsafe`, no format change. The CLI flag is unit `p4`'s.

## Why

- Parent: `PARENT-THREADS`. This is the thread unit. Its decisions: D2 C (stage 2 on two threads,
  one per uniform model, each owning its model's memory) and D3 A (caller readings only).
- Consumer gap: one synthesis of the largest known downstream specification takes 279 s on one
  core of a many-core CI runner.
- Units `p1` (the plan, inline) and `p2` (context, counters, registry) are merged before this
  starts. This unit changes who runs the plan, and nothing else.

## Where

Read at `ff7f8e74fb`. `p1` and `p2` change the files below first, so **re-read them at the
wave's merged head before you start**, and find each site by its name. Record
`BASE=$(git rev-parse HEAD)`.

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

Also add `pub(super) fn add_off_thread(more: u64)`.

### `crates/verify/ess-conformance/src/synthesize/caller/readings.rs` (`p1`'s)

Replace the body of `read_all` (the signature stays):

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
  Above one job the lanes own that memory, and the readings run inline at merge time
  (`one_row_acted_on`, the rotation re-run) start from none.
- `mod tests`:
  - `counted` (3653) runs its synthesis inside `super::super::with_jobs(NonZeroUsize::MIN, ..)`,
    because `WHOLE` counts on the thread that runs a whole reading;
  - add the panic test below.

### `crates/verify/ess-conformance/src/witness_memo.rs`

- `counted` (421-428, `#[cfg(test)]`): run `run` inside `crate::synthesize::with_jobs(NonZeroUsize::MIN, ..)`.
  Its `WORK` recorder and `GENERATIONS` are per thread, and its key uses the model's address.

### `crates/verify/ess-conformance/src/synthesize/threads.rs` (`p2`'s)

Add to `STATE`:

- `synthesize.rs` `JOBS` and `OFF_THREAD` as `Caller`. They are only read and set on the
  calling thread.
- `synthesize/caller/readings.rs` `PANIC_IN` as `Test`.

`Work` is unchanged.

## Tests first

Time each new test in a debug build and name every one over 5 s with its time. No test may run
over 60 s. If one does, the unit says what to cut: N = 7 first, then the largest model of that
test, which you name. A test step stays within 120 s. No new test binary: every test goes into
an existing file. All new tests are red today with `error[E0432]: unresolved import
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

   If its time is over 30 s, lower `10` to the largest value under 30 s and write the expected
   count as `2 + 4 * (flags + 2)`.
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
   `synthesize.rs` 14696).

In `crates/verify/ess-conformance/tests/mutation_grants.rs`:

9. `a_grant_mutants_suites_are_the_same_at_four_jobs`. Build the text with `with_actors` (342),
   using the actors block of `the_added_grant_scenario_is_the_mutants_alone` (506-515): the
   `Librarian` with attributes holding `CatalogueBook`, and the `Reader` without them. Run
   `mutate::emit` with `[MutantClass::GrantDrop, MutantClass::GrantAdd]` at 1 and 4 jobs. Assert:
   - `files` are equal;
   - the manifest lists a `grant-drop/` and a `grant-add/` mutant.

   `with_dropped_grant` and `with_added_grant` (`mutate.rs` 2314-2318) are read inside readings
   by `grant::denied` (`synthesize.rs` 3947, `grant.rs` 81).

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

The mutation proof for 8, 9 and 11 is required, and it is reported:

1. Temporarily replace `context.enter(..)` in the worker by a direct call.
2. Run 8, 9 and 11.
3. Record which go red and the first differing line.
4. Restore.

8 and 9 must go red. If 11 stays green with the context removed, say so: the component's scope
then does not change which commands this fixture takes.

## Steps

1. Re-read `readings.rs`, `caller.rs` `synthesize` and `threads.rs` at the merged head.
2. Write tests 1-12 and record the red lines. Every target runs through the pool.
3. Add `with_jobs`, `jobs`, `readings_off_thread` and the two thread-locals. Classify them in
   `STATE`.
4. Write the threaded body of `read_all`, the panic seam and the `jobs() == 1` condition on
   `_kept`.
5. Run tests 1-12, then the mutation proof.
6. Run the corpus at four jobs. It is the byte proof. It is a command, not a new test:
   `ESS_SYNTH_JOBS=4` makes every synthesis of the two byte-table tests run on threads.
7. Run the gate.
8. One commit, message file via the Write tool, `git commit -F <file>`:
   `feat(conformance): caller readings run on scoped threads with the bytes one thread writes`.
   The body names the job counts tested and the corpus run.

No generated file changes. `with_jobs` and `readings_off_thread` are new public API, so
`task ci-lint` runs.

## Owned files

- `crates/verify/ess-conformance/src/synthesize/caller/readings.rs`: `read_all`'s body and the
  panic seam. `p1` created the file and is merged.
- `crates/verify/ess-conformance/src/synthesize.rs`: `JOBS`, `OFF_THREAD`, `with_jobs`, `jobs`,
  `readings_off_thread`, `add_off_thread` only.
- `crates/verify/ess-conformance/src/synthesize/caller.rs`: the `_kept` line, `counted`, test 12.
- `crates/verify/ess-conformance/src/witness_memo.rs`: `counted` only.
- `crates/verify/ess-conformance/src/synthesize/threads.rs`: three `STATE` rows.
- The four test files named under Tests first: the new tests and the `model_flags` refactor only.

## Not yours

- `crates/edge/ess-cli` and `website/` belong to `p4`; `crates/edge/ess-xtask`, `Taskfile.yml`
  and the design page's `## Measured` belong to `p5`.
- `tests/synthesis_suite_bytes_table.rs` and its tables belong to `t2` this wave. They are run,
  never edited.
- `tests/mutation_audit.rs` and `mutate.rs` belong to `m1` this wave.

## Gate

Each command is one pool job. `cargo` is the shim. Bounds:

- 60 s per test, 120 s per test step;
- a pool job ends after 10 minutes unless it names its time, 30 minutes at most;
- no `RUSTC_WRAPPER`, `SCCACHE_*` or `-j`.

```console
slot task fmt-check
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-conformance --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-cli --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --lib synthesize::caller synthesize::threads witness_memo
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test caller_readings_made_once_work --test mutation_grants --test caller_arranged_rows --test a_synthesis_seed_is_accepted_for_a_caller_model
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 ESS_SYNTH_JOBS=4 cargo test -p ess-conformance --locked --test caller_readings_made_once_work --test caller_arranged_rows --test a_seeded_scenario_notes_its_skipped_reversed_caller_run --test caller_values --test interpreted_caller --test one_time_caller --test mutation_grants --test mutation_sets_entries --test companion_rows_searched_once_work --test row_set_many_selectors_work
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table --test synthesis_suite_bytes_table_cli_read --test adversary_related_values_all_bytes --test external_beside_held_guard
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 ESS_SYNTH_JOBS=4 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table
SLOT_TIMEOUT=30m slot task ci-lint
```

- The `ESS_SYNTH_JOBS=4` run of `synthesis_suite_bytes_table` re-synthesizes every repository
  model and component at four jobs against the committed tables. If `t2` has moved that
  re-synthesis to the slow lane, run the slow lane's tests of the file with `ESS_SYNTH_JOBS=4`
  set. Report its time and the host's load average; its single tests run past 60 s by design.
- The `ess-cli` tests run in `p4`.

## Acceptance

1. Tests 1-12 pass. Their times are reported, and every one over 5 s is named.
2. The mutation proof turned 8 and 9 red with the context removed. The report has the lines.
3. `ESS_SYNTH_JOBS=4` over `synthesis_suite_bytes_table` passes with no table edited:
   `git diff $BASE -- crates/verify/ess-conformance/tests/fixtures` is empty.
4. The `ESS_SYNTH_JOBS=4` run of the caller-themed and work targets in `## Gate` passes.
5. `git diff $BASE -- Cargo.lock '**/Cargo.toml'` is empty: no new dependency.
6. `git grep -n 'unsafe' crates/verify/ess-conformance/src/synthesize/caller/readings.rs` prints
   nothing.
7. `every_static_of_the_crate_is_classified` (`p2`) passes with the three new rows.

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
- D6. The library reads `ESS_SYNTH_JOBS`, so every existing test and both byte tables run at four
  jobs unchanged. That is how identity is proven over the whole corpus. A value outside 1 to 256
  counts as 1 there; the CLI refuses it (`p4`).
- D7. No thread count above 256 is refused by the library (`with_jobs` takes any `NonZeroUsize`).
  The count is capped by the jobs to run anyway.

## Changelog

none here; `p4` writes the story's bullet, which names `with_jobs`.
