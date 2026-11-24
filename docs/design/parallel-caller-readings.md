# Caller readings on several threads

Status: implemented. The plan of readings, the thread state and the worker threads are in the tree:
with a job count above 1 the readings run on scoped threads (`synthesize/caller/readings.rs`), and
`ess verify conform synthesize` and `ess verify conform run` take `--jobs` and read
`ESS_SYNTH_JOBS`. The registry and the carriers are
`crates/verify/ess-conformance/src/synthesize/threads.rs`.

The job count: the CLI's `--jobs` (1 to 256), else `ESS_SYNTH_JOBS`, else 1. A command that
synthesizes refuses an `ESS_SYNTH_JOBS` that is not a whole number from 1 to 256 with exit 1; a
`conform run` given `--suite` or `--suite-input` synthesizes nothing and does not read it. A caller
of the library that sets no job count (`with_jobs`) gets the variable's value when it is a whole
number from 1 to 256 and 1 otherwise; only the CLI refuses an invalid value.

A synthesis of a model whose commands read the caller runs the whole synthesis again for each
assignment of callers, and again for each command and each focus. Those runs are the **caller
readings**. Each is one `synthesize_invocations` of one rewritten model under one focus. They
depend on each other only through the order their results are merged in, so they can run on
several threads. This page states what runs where, which state a reading reads from its thread,
and why the bytes of a suite do not change with the thread.

## What runs where

The plan of readings (`synthesize/caller/readings.rs`) lists, in order: two whole readings, then
two focused runs per mixed command, then two cross runs per mixed command. The arranged-callers
stage follows: first readings, adoptions, swapped readings, appends. The merge stays on the
calling thread, in today's order, over results indexed by their position in the plan. A reading
returns a value; the merge never reads a thread's state.

## Lanes

A reading whose arrangement is a uniform model runs on that model's one thread, in plan order, so
it reuses that model's kept memory as one thread does. That covers each whole reading, every cross
run, and every focused run of a command outside the rewritten set or of the only command in it.
The other readings go to any thread.

The memory is not shared between threads. The memories of `made` and `witness_memo` hold `Rc`
values that live on one thread's stack, so a lane owns its memory and no memory crosses a thread.

## Thread state

Every `static` declared in `crates/verify/ess-conformance/src` is one row of `STATE`, with how a
worker handles it. A unit test reads the sources and fails, naming the file and the static, when a
`static` is missing from `STATE` or `STATE` names one that no longer exists. A later change cannot
add thread state that a worker would silently miss.

The handlings:

- **Counter**: counted per thread. A worker's count is added to the calling thread's after the
  join (`Work`). A tuple counter is one row and gives one `Work` field per element.
- **Context**: set around a whole synthesis and read inside readings. Captured where the readings
  are planned and set on each worker (`Context`).
- **Scope**: opened and closed inside one reading, so empty wherever readings are planned.
  `Context::capture` checks it in debug builds.
- **Memory**: answers what the computation answers. A worker starts with it empty.
- **Lane**: a uniform model's memory kept across its readings, owned by the one thread that runs
  them.
- **Caller**: read or set on the calling thread, outside every reading (`GRANTS_KEPT`, read after
  the synthesis returned).
- **Test**: read only by the crate's own unit tests.
- **Shared**: a plain `static`, immutable, `Sync` by the compiler.
- **Unreached**: in a module no synthesis runs.

Three counters hold a flag beside their counts (`first_kept.rs` `WORK`, `related_guard.rs`
`COMPANION_WORK`, `row_set.rs` `ATTEMPTS`). Their rows say Counter, and `Context` carries the
flags. `related_guard.rs` `BESIDE_OPEN` has a `cut` field, a per-thread running count that is read
only as a difference inside one reading, so a worker starting it at 0 changes no answer.

| path under `src/` | static | handling | what a worker sees |
|---|---|---|---|
| `admission.rs` | `SUITES_PARSED` | Counter | starts at 0; its count is added to the calling thread on join |
| `authored.rs` | `DOCUMENTS_READ` | Counter | starts at 0; its count is added to the calling thread on join |
| `authored/ties.rs` | `REPLAYS` | Counter | starts at 0; its count is added to the calling thread on join |
| `fixtures.rs` | `FIXTURE_STEP_CLONES` | Counter | starts at 0; its count is added to the calling thread on join |
| `fixtures.rs` | `PRECONDITION_STEP_CLONES` | Counter | starts at 0; its count is added to the calling thread on join |
| `input.rs` | `SEGMENTS_COPIED` | Counter | starts at 0; its count is added to the calling thread on join |
| `mutate.rs` | `GRANTS_KEPT` | Caller | not read on a worker |
| `now_offset.rs` | `ANALYSED` | Memory | starts empty and answers what the computation answers |
| `now_offset.rs` | `ANALYSES` | Counter | starts at 0; its count is added to the calling thread on join |
| `process/exact.rs` | `VISITED` | Unreached | no synthesis runs there |
| `process/tests.rs` | `SUITE` | Shared | immutable; shared |
| `scenario.rs` | `FEATURE_SCANS` | Counter | starts at 0; its count is added to the calling thread on join |
| `scenario.rs` | `SERIALIZED` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize.rs` | `BINDING_TRIGGERS_TRIED` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize.rs` | `BUILT_VIEW_INDEXES` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize.rs` | `COMMANDS_TAKEN` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize.rs` | `DERIVED_EDGES` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize.rs` | `DROPPED` | Context | set from the planning thread by `Context::enter` |
| `synthesize.rs` | `NOTHING` | Shared | immutable; shared |
| `synthesize.rs` | `OVERLAP_ANSWERS` | Memory | starts empty and answers what the computation answers |
| `synthesize.rs` | `OVERLAP_SEARCHES` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize.rs` | `PRECEDENCE_REPLAYS` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize.rs` | `SCOPE` | Context | set from the planning thread by `Context::enter` |
| `synthesize.rs` | `SYNTHESES` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/aggregate.rs` | `LITERALS` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/aggregate/contrast.rs` | `DECIDED` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/caller.rs` | `UNFOCUSED` | Test | read by the crate's own unit tests, which run with one job |
| `synthesize/caller.rs` | `WHOLE` | Test | read by the crate's own unit tests, which run with one job |
| `synthesize/created_memo.rs` | `ASKED` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/created_memo.rs` | `KEPT` | Memory | starts empty and answers what the computation answers |
| `synthesize/created_memo.rs` | `MADE` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/created_memo.rs` | `UNKEPT` | Context | set from the planning thread by `Context::enter` |
| `synthesize/disclosure.rs` | `ORIGINS_BUILT` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/earlier_runs.rs` | `EARLIER` | Scope | empty; opened and closed inside the reading |
| `synthesize/first_kept.rs` | `WORK` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/grant.rs` | `ADDED` | Context | set from the planning thread by `Context::enter` |
| `synthesize/grant.rs` | `DROPPED` | Context | set from the planning thread by `Context::enter` |
| `synthesize/made.rs` | `KEPT` | Lane | owned by the one thread of its lane |
| `synthesize/made.rs` | `MADE` | Memory | starts empty and answers what the computation answers |
| `synthesize/made.rs` | `MAKES` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/made.rs` | `PICKS` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/made.rs` | `RELATED_PICKS` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/model_memo.rs` | `ABSENT_BUILT` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/model_memo.rs` | `KEPT` | Memory | starts empty and answers what the computation answers |
| `synthesize/ordered_head.rs` | `STEERING_WORK` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/related_guard.rs` | `BESIDE_OPEN` | Scope | empty; opened and closed inside the reading |
| `synthesize/related_guard.rs` | `COMPANION_WORK` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/related_guard.rs` | `OWN_OPEN` | Scope | empty; opened and closed inside the reading |
| `synthesize/related_guard.rs` | `OWN_OVERLAY_WORK` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/related_guard.rs` | `OWN_WORK` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/row_set.rs` | `ATTEMPTS` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/row_set.rs` | `TRIED` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/run_memo.rs` | `ASKED` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/run_memo.rs` | `KEPT` | Memory | starts empty and answers what the computation answers |
| `synthesize/run_memo.rs` | `MADE` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/run_memo.rs` | `UNKEPT` | Context | set from the planning thread by `Context::enter` |
| `synthesize/seeds.rs` | `DIGESTED` | Memory | starts empty and answers what the computation answers |
| `synthesize/seeds.rs` | `EMPTY` | Shared | immutable; shared |
| `synthesize/shorthand.rs` | `TRAVERSALS` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/subject_fact.rs` | `FIELDS_BOUND` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/subject_fact.rs` | `HELD` | Scope | empty; opened and closed inside the reading |
| `synthesize/subject_fact.rs` | `INPUTS_MADE` | Counter | starts at 0; its count is added to the calling thread on join |
| `synthesize/subject_fact.rs` | `ROWS` | Memory | starts empty and answers what the computation answers |
| `web_execution/host.rs` | `HOST` | Unreached | no synthesis runs there |
| `witness.rs` | `BUILT` | Counter | starts at 0; its count is added to the calling thread on join |
| `witness.rs` | `CLONED` | Counter | starts at 0; its count is added to the calling thread on join |
| `witness.rs` | `REFERENCE` | Scope | empty; opened and closed inside the reading |
| `witness.rs` | `STEPS` | Counter | starts at 0; its count is added to the calling thread on join |
| `witness.rs` | `UNDER_TEST` | Scope | empty; opened and closed inside the reading |
| `witness_memo.rs` | `GENERATIONS` | Test | read by the crate's own unit tests, which run with one job |
| `witness_memo.rs` | `KEPT` | Lane | owned by the one thread of its lane |
| `witness_memo.rs` | `MEMO` | Memory | starts empty and answers what the computation answers |
| `witness_memo.rs` | `SHARED` | Memory | starts empty and answers what the computation answers |
| `witness_memo.rs` | `WORK` | Test | read by the crate's own unit tests, which run with one job |

The table has 74 rows. The change that spawned the workers added its own statics (a job count and
the two flags of a worker) and classified them in `STATE` too.

### What `Context` carries

- the scope of a component-scoped synthesis (`SCOPE`);
- the dropped write of a `sets-drop` mutant (`synthesize.rs` `DROPPED`);
- the dropped and the added grant of the `grant-drop` and `grant-add` mutants (`grant.rs`);
- the two `UNKEPT` switches that make every creation or run afresh;
- the three fresh flags named above;
- the phase order, which a test may exchange (`ess-domain`, `with_phase_order`).

The mutation audit sets the dropped write and the grants around a whole synthesis. A worker that
did not see them would write a different mutant suite, so each worker sets them from the planning
thread's values through the existing setters, nested, and restores them also on a panic. No
reading refuses to go parallel because of one: every value is cheap to clone.

### What `Context` leaves out

- `GRANTS_KEPT`, read on the calling thread after the synthesis returned;
- the reading scopes, which are empty where the readings are planned. No code opens one around the
  caller synthesis, which is reached only from the entry points and the mutation audit;
- the memories, which start empty and answer the same;
- the assembly flags of `ess-domain` (`CONVERTING_ATTRIBUTES`, `ADMITS_DELETIONS` and the
  filtered-read flag), which are set only while a specification is assembled, never around a
  synthesis;
- `MAP_KEY_NEWTYPES`, set only by a scope no synthesis opens.

### What `Work` leaves out

The counters of other crates have no adder: `binary64::SCANS` of `ess-compiler`, and
`DECLARATIONS_EXAMINED`, `CONTAINMENT_CHECKS`, `TYPE_CHECK_WORK` and the map-key copy count of
`ess-domain`. The only test that reads one around a synthesis uses models without actor
attributes, which never run a reading on another thread.

### What crosses the thread boundary

`EssIr` and `AdmittedSeeds` are `Sync`, and `Synthesis` is `Send`; a compile-time assertion in
`threads.rs` holds that. `EssIr` holds maps, vectors and a scan of two `OnceLock`s, and no `Rc`,
`RefCell` or `Cell`. The workspace forbids `unsafe`, and the boundary needs none. Every other `Rc`
in the crate is a memo value on one thread's stack and never crosses.

## Determinism

- Results are merged by plan index, never in arrival order.
- No unordered iteration is added.
- A memory answers what the computation answers. The debug guard of `witness_memo.rs` checks a
  sample of the answers it gives, and the byte tables check the whole corpus.
- So the bytes do not depend on which thread ran a reading, nor on the memory it started with.

## Memory

One thread keeps both uniform models' memories for the whole caller synthesis today. On the
largest known downstream specification the process held 2.57 GB after the first whole reading,
4.81 GB after the second and 6.09 GB at the end. Two lanes keep the same two memories. Each
further thread adds the working memory of one reading. The default is one thread.

## Measured

One synthesis (`ess verify conform synthesize --target ir --compact`) of a generated model of 48
flags and 50 commands, 24 of them reading the caller, at `ESS_SYNTH_JOBS` 1, 2 and 4. Release
build, 2026-11-23, on the tree of commit `34bc86011` plus the command, 20 cores, load average
39.05 at the start (below 60) and 47.77 at the end. Command: `task synth-jobs-profile`. Peak
memory is polled every 20 ms, so it is a lower bound by at most the last 20 ms.

| jobs | wall s | user s | sys s | peak MiB | speed-up | scenarios | suite sha256 |
|---|---|---|---|---|---|---|---|
| 1 | 67.9 | 61.5 | 0.6 | 1327 | 1.00x | 270 | 684c1d82ccd0d234654cd34dce073df00c03d5bacf5148d21f5e8657d9888f52 |
| 2 | 36.5 | 66.0 | 0.9 | 1526 | 1.86x | 270 | 684c1d82ccd0d234654cd34dce073df00c03d5bacf5148d21f5e8657d9888f52 |
| 4 | 41.6 | 84.8 | 1.8 | 1930 | 1.63x | 270 | 684c1d82ccd0d234654cd34dce073df00c03d5bacf5148d21f5e8657d9888f52 |

The speed-up is 1.86x at 2 jobs and 1.63x at 4 against the parent's estimate of 3.5x: on this
model four jobs are slower than two, so the estimate does not hold here. Peak memory at 4 jobs is
1930 MiB against 1327 MiB at one job, 1.45 times as much. The default stays 1.
