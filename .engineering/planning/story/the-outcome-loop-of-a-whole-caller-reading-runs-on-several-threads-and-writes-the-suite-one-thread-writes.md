---
format: aep.planning-md/3
id: story:the-outcome-loop-of-a-whole-caller-reading-runs-on-several-threads-and-writes-the-suite-one-thread-writes
kind: story
status: draft
title: The outcome loop of a whole caller reading runs on several threads and writes the suite one thread writes
summary: --jobs runs caller readings in about two long lanes, so the critical path is one whole reading and four jobs are slower than two on the one measurement (gap report, 2026-11-24)
tags:
- external-request
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes
revision: 2
---
## Summary

With `--jobs` above the number of long readings, the work inside one whole caller reading, which today is one thread, is spread over
the further threads, and the suite, refusals and notes are the bytes one job writes.

## Need

A downstream application reported on 2026-11-24 (eleventh gap report; reporter name kept out) that a system of five components and
about 800 scenarios took six to nine minutes to synthesize on one core of a 64-CPU node. `--jobs` (on main, not released,
`story:the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes`) runs caller readings on
several threads, and stops short of the node:

- The two whole readings (every command sent by the first caller, every command by the second) and every cross run read uniform
  models, and a uniform model's memory lives on one thread, so each runs in one of two lanes
  (`docs/design/parallel-caller-readings.md` § Lanes; `synthesize/caller/readings.rs` `read_all`). Threads beyond two serve only the
  focused runs.
- In the earlier timeline of a downstream specification of the same shape (the parent story's measurement: 438.9 CPU-s over 222
  readings), the whole reading with all callers second held 81.9 CPU-s, 18.6 %, the longest single unit; the two whole readings and
  the cross runs together were about 60 % of the CPU (12.9 + 18.6 + 28.2 %). The parent story's table says the ceiling above four threads is that reading.
- Measured on a generated model (`task synth-jobs-profile`, 20 cores at load 39 to 48): 1.86x at 2 jobs, 1.63x at 4, with peak memory
  1.45 times the single job's at 4 (`parallel-caller-readings.md` § Measured). Four jobs are slower than two.
- The parent story's decision D3 chose "caller readings only" and named the next step: split the outcome loop of one reading by command,
  "which shares memories across commands and is a later story, measured first". No such story is in the plan store.

## Findings covered

- `--jobs` cannot use more than about two lanes of a large caller model; the critical path is one whole reading.
- The measured scaling is below the parent story's estimate and degrades at four jobs.

## Fit review

1. Need: wall time of one synthesis of a large caller model on a runner with many idle cores.
2. Class: gap in the tool (cost). Nothing authored changes.
3. Expressible today: no. `--component` parts do not give the whole suite and, in the reported run, took as long as the whole
   (`CHANGELOG.md` 0.97.3 entry; the report).
4. Fit: the same merge-by-plan-index determinism the parent story states (results indexed by position, no unordered iteration, memories
   that answer what the computation answers, `parallel-caller-readings.md` § Determinism). The unit is new: one command's obligations
   inside one reading. The parent story rejected "one obligation" because its memories are shared with the rest of the reading and the
   families after the outcome loop read the suite the loop filled; this story's first outcome is the measurement that says whether a
   command is a unit that shares little enough.
5. Second adopter: any specification with actor attributes and several dozen commands (the parent story's adopter, 279 s on 0.97.4
   there).
6. Cost: no format, no diagnostic, no dependency (scoped threads exist). More memory per thread; the default stays 1.
7. Alternatives: change nothing (use `--jobs 2`, the measured best on the generated model); reduce single-thread work instead
   (`pf1` to `pf3`, a different cause: row-set arrangement), which composes with this; parts and a merge (ST-D5, a persisted
   document).

## Decisions

Taken on the default, 2026-11-24 16:27 CET (the coordinator, under the owner's delegation): ST-D4, wider parallelism than caller readings. Decided: A, measure first (per-command CPU share of a whole reading, how much memory one command's obligations read from another's), record it, then split the outcome loop of a whole reading by command if the share is small. B was to split by command now, without the measurement; C to decline and tell adopters to use `--jobs 2` and to cut single-thread work (`pf1` to `pf3`). Not architecture: suite bytes, format and determinism rules do not change. The parent story's own D3 said "measured first", and the one measurement on main (a generated model) shows four jobs slower than two.

## Acceptance

- the_per_command_share_of_a_whole_reading_is_measured: before any split, `cargo xtask synth-jobs` (or a sibling command) reports, for
  a generated caller model, the CPU time of each command's obligations inside the whole readings and how much of the memory one
  command's obligations read that another's wrote; the figures are recorded in the story or a design page. (A measurement outcome: if
  commands share most of their memory, the story ends with a decision to decline, recorded.)
- the_bytes_do_not_depend_on_the_thread: the suite, refusals, notes and count line of every corpus model that `the-caller-readings`
  tests cover are `cmp`-identical at 1, 2, 4 and 7 jobs after the change.
- more_threads_than_lanes_do_work: at `--jobs 4` on the generated model, more than one thread runs obligations of one whole reading (a
  counter of obligations run off the reading's own thread is above 0), and the wall time at 4 jobs is below the wall time at 2 jobs,
  where today it is above (41.6 s against 36.5 s, `parallel-caller-readings.md` § Measured).
- memory_is_stated: the peak at 1, 2, 4 and 8 jobs is printed by the measuring command and written in `parallel-caller-readings.md`;
  the default stays 1.
- every_thread_state_is_classified: a new `static` the change adds is a row of `STATE` (the existing unit test fails otherwise).

## Scope

- crates/verify/ess-conformance/src/synthesize.rs  cited — the outcome loop of one reading (`outcome_scenario_noting`), `synthesize_invocations`
- crates/verify/ess-conformance/src/synthesize/caller/readings.rs  cited — `read_all`, lanes
- crates/verify/ess-conformance/src/synthesize/threads.rs  cited — `STATE`, `Context`, `Work`
- crates/verify/ess-conformance/src/synthesize/made.rs, witness_memo.rs, run_memo.rs, created_memo.rs, model_memo.rs  cited — the memories a command's obligations share
- crates/edge/ess-xtask/src/synth_jobs.rs  cited — the measurement
- docs/design/parallel-caller-readings.md  cited

## Format

None. Suite bytes are those of one job.
