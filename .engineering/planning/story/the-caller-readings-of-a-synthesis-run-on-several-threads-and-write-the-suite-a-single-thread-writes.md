---
format: aep.planning-md/3
id: story:the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes
kind: story
status: active
title: The caller readings of a synthesis run on several threads and write the suite a single thread writes
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/now_offset.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/caller.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/created_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/made.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/model_memo.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/run_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/seeds.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness_memo.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/main.rs
- confidence: inferred
  path: website/docs/reference/
revision: 18
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T22:55:13Z", actor: "human:timo", revision: 15}
- {from: "proposed", to: "active", at: "2026-11-22T22:55:14Z", actor: "human:timo", revision: 16}
---
## Outcome

`ess verify conform synthesize --jobs N` synthesizes a model whose actors carry attributes on up
to N threads and writes the suite, refusals, notes and counts that `--jobs 1` writes, byte for
byte, for every N. On the largest downstream specification known, four threads bring the
synthesis to about a third of its single-threaded time (inferred from the measured split, below),
within the memory the single thread needs plus a bounded amount per extra thread.

## Need

An adopter asked for "parallel synthesis across obligations": its CI synthesizes its largest
downstream specification on one thread (279 s wall on its runner with 0.97.4), and the runner has
more cores. Its proposal is evidence of a need (wall time on a many-core runner), not a design.

Measured on this host, integration head `df7eb37a65`, the downstream specification, command
`ess verify conform synthesize --path . --target ir --scenarios scenarios --out <file>` (
the profile notes, sections 1 and 3; the timeline, taken under gdb with breakpoints
on the reading, family and outcome entries, 439.2 CPU-s, suite byte-identical to the free run):

- One thread does all the work: 401.4 s user, 546.6 s wall at load 36-54, 1,385.3e9
  instructions:u, peak RSS 6.07 GiB.
- The model's actors carry attributes, so `caller::synthesize` reads it 222 times
  (`synthesize_invocations` runs): the whole model twice (all callers first: 56.6 CPU-s, 12.9 %;
  all callers second: 81.9, 18.6 %), each of the 43 commands whose two callers read it differently
  twice in `about_command` (84 runs, 132.0, 30.1 %, largest 14.8) and twice in
  `cross_single_command` (86 runs, 123.7, 28.2 %, largest 16.6), then 50 runs in
  `arranged_by_callers` (41.7, 9.5 %, largest 5.2). After the readings: 3.0 (0.7 %).
- Inside a reading, the outcome loop holds 71.3 % of the time; the most expensive obligation is
  4.2 % of the run summed over its 7 readings, the most expensive single call 1.0 %.

So the work is already cut into independent pieces by the code: 222 readings, each a full
synthesis of one rewritten model under one focus, whose results `caller::synthesize` merges in a
fixed order afterwards.

## Fit review

1. Need, apart from the proposal: wall time of one synthesis on a runner with idle cores. The
   adopter's words: "parallel synthesis across obligations".
2. Class: a gap in the tool (cost), not in the language; nothing authored changes.
3. Expressible today? Partly: `--component` synthesizes one component's operations, and a CI
   could run components in parallel jobs. It does not give the whole suite (a component's run
   refuses what other components realize, and cross-component scenarios are not in it), so it is
   no answer for a whole-system suite.
4. Fit. What makes the readings independent today, by reading the code:
   - each reading is `synthesize_invocations(models, focus)` over its own rewritten `EssIr`
     (`Assigned::model`, `arranged::lifted_model`); its result is a `Synthesis` value;
   - every memory is a thread-local stack keyed by the model's address (`witness_memo::MEMO`,
     `made::MADE`, `created_memo`, `run_memo`, `model_memo`, `now_offset::ANALYSED`,
     `OVERLAP_ANSWERS`, `subject_fact::ROWS`/`HELD`): a reading started on another thread has
     its own, and every memory answers what the computation would answer (`witness_memo`'s debug
     guard checks this on a sample of hits; `AGENTS.md` and the synthesis-time story require a
     key to hold everything the computation reads);
   - context a reading reads from its thread and that a worker must be given: the component
     scope (`synthesize::SCOPE`), the bound seeds (`seeds::DIGESTED`, keyed by model address).
     `UNDER_TEST`, `earlier_runs::EARLIER`, `grant::ADDED`/`DROPPED`, `synthesize::DROPPED`,
     `related_guard::BESIDE_OPEN` and `witness::REFERENCE` are opened and closed inside one
     reading;
   - the work counters (`*_work.rs` tests' `STEPS`, `BUILT`, `PICKS`, ...) are per thread.
   What is not independent: the merge. `caller::synthesize` appends the swapped run to each
   scenario of the first reading and consumes `Identities` as it goes (`append`,
   `identities.take`), replaces scenarios and refusals per mixed command in name order, and
   `arranged_by_callers` decides which commands to read from the merged suite and runs the
   swapped reading only where the first adopted a scenario. Identity literals are numbered in
   that merge. So the merge stays on one thread, in today's order, over results computed
   anywhere: the suite cannot depend on the thread count. The two uniform models (all first,
   all second) are kept for the whole caller synthesis (`Assigned::witnessed`, `made`), and the
   86 `cross` runs arrange on them and reuse their parked answers.
   Rejected unit: one obligation (one outcome of one command). Its memories are shared with
   every other obligation of the same reading (a created row, a witness of a creating command),
   so a worker per obligation repeats them; and the families after the outcome loop read the
   suite the loop filled. The reading is the unit the code already isolates.
5. Second adopter: any specification with actor attributes and many commands per caller (each
   such command costs 4 focused readings); also any CI with a core count above one.
6. Cost: one flag and one environment variable, no format, no new diagnostic, no new dependency
   (`std::thread::scope` and a channel; `Cargo.lock` holds no thread-pool crate, neither
   `rayon` nor `crossbeam`). `Rc<EssIr>` in `Assigned::models` and `witness_memo::Keeping`
   becomes `Arc<EssIr>` (or each worker builds its reading's model itself).
7. Considered: change nothing (279 s stays); split CI by component (not the whole suite);
   parallel obligations (above); a work-stealing pool crate (no need for 222 coarse units).

Decision: accept, redesigned (the unit is a caller reading, not an obligation).

## Design

- Units and order. `caller::synthesize` computes its readings in three stages, each a set of
  independent `synthesize_invocations` calls handed to N workers (largest expected first, by the
  previous stage's order), and merges each stage's results on the calling thread in today's
  order before the next stage:
  1. the two whole readings and the 84 `about_command` runs (none reuses another's memory: the
     `about` models are not uniform, so they are not kept);
  2. the 86 `cross_single_command` runs; the parked memories of the two uniform models are
     handed to the workers read-only (a frozen layer each worker's own memory sits on, looked up
     first locally, then in the frozen layer), so they are reused as today without copies;
  3. `arranged_by_callers`: the first reading of every refused command in parallel, the
     adoption in order, then the swapped readings of the adopting commands in parallel, then
     their merge in order.
  A model without actor attributes is one reading and runs on one thread as today (see D3).
- Each worker is a scoped thread that opens the same scopes `synthesize_in` opens for its model
  (`memoise`, `made::remember`, `now_offset::remember`, `model_memo::keep`,
  `created_memo::remember`) and copies the caller's component scope and bound seeds in. Results
  come back with their stage index and are merged by index, never in arrival order.
- `--jobs N` on `ess verify conform synthesize` (and the same setting for `ess verify conform
  run` when it synthesizes), `ESS_SYNTH_JOBS` as its default, default 1 (D1). N is a bound, not
  a promise: a stage with fewer units uses fewer threads.
- Work counters: each worker adds its thread's counts to the caller's on join, so a `*_work.rs`
  bound holds at every N.
- Format consequence: none. The bytes are those of `--jobs 1` by construction (the merge is
  unchanged and the memories are pure), held by the tests below.
- Memory. Serially both uniform memories are kept for the whole caller synthesis anyway (RSS
  2.57 GB after the first whole reading, 4.81 GB after the second, 6.09 GB at the end,
  the timeline notes), so stage 1 running both at once adds nothing to what is kept.
  Each further worker adds the working memory of one focused reading (the focused runs raised the
  serial peak by 0.37 GB in total; a single reading's transient peak is not measured: inferred
  below 1 GB). Expected peak: about 4.8 GB + (N - 2) x the transient of one focused reading.
  The adopter's runner allows 7 Gi, and one thread already peaks at 6.07 GiB here, so memory is
  measured at N = 1, 2, 4 before the default is chosen (D1, D4).
- Expected gain on the downstream specification (Amdahl over the measured stages, CPU-s of the
  gdb run, scheduling largest first; inferred, not run):

  | N | stage 1 (270.5, largest 81.9) | stage 2 (123.7, 16.6) | stage 3 (41.7, 5.2) | serial 3.0 | total | speed-up |
  |---|---|---|---|---|---|---|
  | 1 | 270.5 | 123.7 | 41.7 | 3.0 | 438.9 | 1.0 |
  | 2 | 135.3 | 61.9 | 20.9 | 3.0 | 221.1 | 2.0 |
  | 4 | 81.9 | 30.9 | 10.4 | 3.0 | 126.2 | 3.5 |
  | 8 | 81.9 | 16.6 | 5.2 | 3.0 | 106.7 | 4.1 |

  The ceiling is the whole reading with all callers second (81.9, 18.6 %): above 4 threads it is
  the critical path. On the adopter's runner that would put 279 s near 80 s at N = 4 if it has
  four cores and the memory fits. Going further needs the outcome loop of a whole reading split
  by command (its largest command is 12.7 of the 81.9), which shares memories across commands and
  is a later story, measured first (D3).

## Acceptance

- `crates/verify/ess-conformance/tests/` gets one byte-identity test over the committed model
  corpus (`synthesis_suite_bytes_table`'s models plus the caller models of the work tests): each
  synthesized at 1, 2 and 7 threads, suite, refusals, notes and the printed count line compared
  with `cmp`. Under 60 s, in the existing integration binary; a model too slow for that is left out
  and named.
- A generic caller model with 12 commands read differently by two callers synthesizes at
  `--jobs 4` with the same bytes as `--jobs 1` and spends less wall time (a work count, not a
  clock: the number of readings run on another thread than the caller's is 4 x 12 + 2 minus the
  ones on the caller's thread, counted by a test counter).
- The `*_work.rs` bounds pass at `ESS_SYNTH_JOBS=1` and `=4` (their counts summed over workers).
- On the downstream specification, measured once each at N = 1, 2, 4: suite sha256 equal, wall
  time, user CPU, peak RSS reported in the story; N = 4 is at most 45 % of N = 1's wall time on an
  idle 4-core machine.
- `ess verify conform synthesize --help` and `website/docs/reference/` name `--jobs`,
  `ESS_SYNTH_JOBS`, the default, and that the bytes do not depend on it.

## Decisions

- D1. Default thread count. A: 1 (opt in; nothing changes for anyone who does not ask);
  B: the available parallelism, capped at 4; C: the available parallelism. Recommended: A for the
  first release, because one thread already peaks at 6.07 GiB on the largest known specification
  and the adopter's runner allows 7 Gi; move to B once the peaks at N = 2 and 4 are measured.
- D2. Reuse of the two kept memories in stage 2. A: a frozen read-only layer shared by all
  workers (memory as today, needs `Send + Sync` memo types); B: each worker starts empty (simple,
  repeats the uniform readings' searches in every `cross` run, CPU unmeasured); C: run stage 2 on
  two threads only, one per uniform model, each owning its model's memory. Recommended: C first
  (no shared state, ceiling for stage 2 is 123.7 / 2 = 61.9, total at N = 4 about 157 CPU-s,
  2.8x), A when C is measured.
- D3. Scope of the first story. A: caller readings only (this story); B: also the outcome loop of
  one reading split by command, for every model. Recommended: A; B needs a measurement of how much
  of a reading's memory one command reuses from another before it is designed.
- D4. Flag name. A: `--jobs` (as cargo and make); B: `--threads`. Recommended: A.

## Decisions taken

Taken for this wave on 2026-11-22, 22:47 CET, as part of decision D24 (option A, "synthesis speed,
parallel"): the story's own recommendations stand.

- D1 A: the default is 1 thread; more threads are opt in.
- D2 C: stage 2 runs on two threads, one per uniform model.
- D3 A: caller readings only.
- D4 A: the flag is `--jobs`.

A measurement of peak memory at 2 and 4 threads is part of the work before any default changes.

Decided on 2026-11-23, after round 1 of the plan critics:

- The measurement tool is a committed `cargo xtask synth-jobs` that times a generated caller
  model at 1, 2 and 4 jobs and refuses differing suites, because the largest known downstream
  specification is private and no unit can read it.
- The acceptance "4 jobs take at most 45 % of 1 job" is judged by the coordinator on that
  specification, with the wave's release build and the tool's `--path`, and recorded here. The
  measurement unit's own acceptance is decided by counts and by identical suites.
