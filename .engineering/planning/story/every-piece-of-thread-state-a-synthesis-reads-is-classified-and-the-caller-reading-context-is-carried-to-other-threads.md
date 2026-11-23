---
format: aep.planning-md/3
id: story:every-piece-of-thread-state-a-synthesis-reads-is-classified-and-the-caller-reading-context-is-carried-to-other-threads
kind: story
status: active
title: Every piece of thread state a synthesis reads is classified and the caller reading context is carried to other threads
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes
- depends_on: story:a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose
- depends_on: story:every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/admission.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/authored/ties.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/fixtures.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/now_offset.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate/contrast.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/created_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/disclosure.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/earlier_runs.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/first_kept.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/grant.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/made.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/model_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/ordered_head.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/run_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/shorthand.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/threads.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: inferred
  path: docs/design/parallel-caller-readings.md
revision: 38
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:03:08Z", actor: "human:timo", revision: 37, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:03:10Z", actor: "human:timo", revision: 38, decided_on: {"recorded":{"review_outcome":4}}}
---
## Outcome

- A new module, `crates/verify/ess-conformance/src/synthesize/threads.rs`, holds:
  - `STATE`: one row for every `static` declared anywhere in `crates/verify/ess-conformance/src`,
    each with its handling;
  - `Context`: the values a caller reading reads from its thread, captured on one thread and set
    on another;
  - `Work`: every work counter of the crate, read on one thread and added to another;
  - compile-time assertions that `EssIr` and `AdmittedSeeds` are `Sync` and `Synthesis` is
    `Send`.
- A unit test fails, naming the file and the static, when a `static` of the crate is missing from
  `STATE` or `STATE` names one that no longer exists. So no later change can add thread state that
  a worker would silently miss.
- `docs/design/parallel-caller-readings.md` (new) states the plan of readings, the lanes, every
  piece of thread state with its handling, the determinism argument and the memory expectation.
- Nothing spawns a thread outside the tests, and no byte of any synthesis changes. This unit
  changes no behaviour.
- `Context`, `Work` and the accessors have no caller outside the tests until `p3`. They stay
  clean under `clippy -D warnings` through one named allow, which `p3` removes:
  `#![cfg_attr(not(test), allow(dead_code))]` at the top of `threads.rs`, with a comment beside
  it. See D7.

## Why

- Parent: `story:the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes`.
  Its first point: every piece of state a reading touches that is not passed as an argument must
  be named, with what a worker must see and how.
- Depends on `p1` (merged before: the design page describes `p1`'s plan and lanes) and on `c1`
  (merged before: its counter `OWN_OVERLAY_WORK` is classified here). Routing decision X3.
- The mutation audit runs readings with `with_dropped_write`, `with_dropped_grant` and
  `with_added_grant` set around the whole synthesis (`mutate.rs` 5643, 2314-2318). A worker that
  did not see them would write a different mutant suite.
- `*_work.rs` bounds read thread-local counters on the calling thread. A worker's counts must
  reach it.

## Where

Read at `ff7f8e74fb`, and re-checked at `80794bbd698` (`plan/wave-2026-11-23`, the plan tree's
head). Between the two, only `synthesize.rs` (9 lines added after 3508) and
`synthesize/related_guard.rs` (the correction's `OWN_WORK` and its functions, 2838-2920, which
moved everything below by 62 lines) changed in `ess-conformance/src`. The line numbers below are
those of `80794bbd698`. `p1` and `c1` land before this unit and move lines in `caller.rs`,
`made.rs`, `witness_memo.rs`, `synthesize.rs` and `related_guard.rs`, so find each site by its
name. Record `BASE=$(git rev-parse HEAD)` first.

### Verified by reading: what is and is not `Send`/`Sync`

- `EssIr` (`crates/specify/ess-compiler/src/ir.rs` 3325-3388) holds `BTreeMap`s, `Vec`s and a
  `binary64::Scan` of two `OnceLock`s (`binary64.rs` 23-26). It holds no `Rc`, `RefCell` or
  `Cell`, so it is `Send + Sync`.
- `git grep -n 'Rc<\|RefCell\|Cell<'` over `ess-compiler/src`, `ess-domain/src` and
  `ess-primitives/src` finds only:
  - `resolve.rs` 746, a `Locator` used while compiling;
  - thread-locals;
  - `ess-domain` `system.rs` 1202, an `Rc` inside a thread-local.
- `AdmittedSeeds` is already a `static` (`seeds.rs` 46), so the compiler holds it `Sync` today.
- The only `Rc<EssIr>` were `caller.rs` 358 and 391, `made.rs` 323 and 334, and
  `witness_memo.rs` 124 and 135. Unit `p1`, merged before, replaced them with `Arc`.
- Every other `Rc` in the crate is a memo value on one thread's stack: `made.rs` 54-59,
  `subject_fact.rs`, `now_offset.rs` 124, `model_memo.rs` 31 and three `synthesize.rs` memos.
  None of them crosses.
- So the boundary is sound without `unsafe`. The workspace forbids it anyway (`Cargo.toml` 114,
  `unsafe_code = "forbid"`).

### `crates/verify/ess-conformance/src/synthesize.rs` (23068 lines)

- Module list (180-231): add `mod threads;` between `mod system_issued;` (228) and
  `mod value_invariant;` (229).
- `SCOPE` (3776) and `DROPPED` (14676) are private to this file. `threads.rs` is a child module
  and reads them directly. Add two accessors:
  - beside `in_scope` (3784): `fn scope() -> Option<BTreeSet<QualifiedName>>` returning
    `SCOPE.with_borrow(Clone::clone)`;
  - beside `dropped_key` (14705): `fn dropped_write() -> Option<DroppedWrite>` returning
    `DROPPED.with(|it| it.borrow().clone())`.
- Counters of this file (`Work` reads them directly, they are visible to the child): `SYNTHESES`
  3499, `COMMANDS_TAKEN` 3780, `DERIVED_EDGES` 9691, `BUILT_VIEW_INDEXES` 9693,
  `OVERLAP_SEARCHES` 20033, `PRECEDENCE_REPLAYS` 20326, `BINDING_TRIGGERS_TRIED` 21716. No
  accessor is needed for them.

### Accessors in the other modules

Each accessor is two lines. Add it directly below the counter's existing reader function, and give
each a one-line doc comment. Names are `taken_<static in lower case>()` returning the thread's
current value and `add_<static in lower case>(more)` adding to it. Visibility:

- `pub(super)` for modules under `synthesize/`;
- `pub(crate)` for the crate-level modules (`witness.rs`, `now_offset.rs`, `scenario.rs`,
  `input.rs`, `fixtures.rs`, `authored.rs`, `authored/ties.rs`, `admission.rs`). For those,
  `threads.rs` reaches them as `crate::<module>`.

| file | static (line) | today's reader | add |
|---|---|---|---|
| `synthesize/created_memo.rs` | `ASKED` 39, `MADE` 40 | `counting` 80 | `taken_asked/add_asked`, `taken_made/add_made` |
| `synthesize/created_memo.rs` | `UNKEPT` 42 (context) | `unkept` 92 | `fn is_unkept() -> bool` |
| `synthesize/run_memo.rs` | `ASKED` 35, `MADE` 36 | `counting` 65 | as above |
| `synthesize/run_memo.rs` | `UNKEPT` 38 (context) | `unkept` 77 | `fn is_unkept() -> bool` |
| `synthesize/made.rs` | `MAKES` 71, `PICKS` 84, `RELATED_PICKS` 103 | `counting` 76, `picks` 89, `related_picks` 108 | three pairs |
| `synthesize/model_memo.rs` | `ABSENT_BUILT` 22 | `absent_built` 73 | one pair |
| `synthesize/disclosure.rs` | `ORIGINS_BUILT` 14 | `followup_origins_built` 21 | one pair |
| `synthesize/ordered_head.rs` | `STEERING_WORK` 211 `(u64, u64)` | `steering_work` 216 | one pair over the tuple |
| `synthesize/aggregate.rs` | `LITERALS` 60 | `literals_collected` 66 | one pair |
| `synthesize/aggregate/contrast.rs` | `DECIDED` 736 | `deciding` 741 (re-exported as `aggregate::conditions_decided` 54) | one pair; `mod contrast` (`aggregate.rs` 48) is private, so also `pub(super) use contrast::{taken_decided, add_decided};` in `aggregate.rs` |
| `synthesize/shorthand.rs` | `TRAVERSALS` 131 | `traversals` 136 | one pair |
| `synthesize/first_kept.rs` | `WORK` 25 `(asked, made, fresh)` | `first_rows_made` 31 | `taken_work() -> (u64, u64)`, `add_work((u64, u64))`, `fn fresh() -> bool` (context) |
| `synthesize/row_set.rs` | `ATTEMPTS` 3338 `(count, fresh)`, `TRIED` 5022 | `row_set_attempts` 3345, `counting` 5027 | `taken_attempts/add_attempts` over the count, `fn attempts_fresh() -> bool` (context), `taken_tried/add_tried` |
| `synthesize/subject_fact.rs` | `FIELDS_BOUND` 626, `INPUTS_MADE` 640 `(u64, u64)` | `fields_bound` 631, `inputs_made` 645 | two pairs |
| `synthesize/related_guard.rs` | `COMPANION_WORK` 4977 `(asked, searched, fresh)` | `companion_work` 4985 | `taken_companion_work() -> (u64, u64)`, `add_companion_work`, `fn companion_fresh() -> bool` (context) |
| `synthesize/related_guard.rs` | `OWN_WORK` 2842 `(arranged, strict, deepest)`, the correction's | `own_scenario_work` 2868 | `taken_own_work() -> (u64, u64, usize)`, `add_own_work`: the first two fields are summed, the third is a maximum (`deepest.max(more.2)`), as `own_worked` (2891) keeps it |
| `synthesize/related_guard.rs` | `OWN_OVERLAY_WORK` (`c1` D4: one `thread_local!` `Cell<(u64, u64)>` in the block of `OWN_OPEN` and `OWN_WORK`, the pair `(built, asked_at_the_bound)`) | `own_overlay_work` (`c1`: `pub(super)` in `related_guard.rs`, re-exported `#[doc(hidden)] pub` in `synthesize.rs` beside `own_scenario_work` 3516, returning `(R, (u64, u64))`) | `taken_own_overlay_work() -> (u64, u64)`, `add_own_overlay_work((u64, u64))`: both fields summed |
| `synthesize/grant.rs` | `ADDED` 313, `DROPPED` 365 (context) | `with_added_grant` 323, `with_dropped_grant` 375 | `fn added_grant() -> Option<AddedGrant>`, `fn dropped_grant() -> Option<DroppedGrant>` |
| `witness.rs` | `STEPS` 2923, `BUILT` 2928, `CLONED` 2934 | `directed_steps` 2971, `inputs_built` 2954, `builders_cloned` 2942 | three pairs (`usize`) |
| `now_offset.rs` | `ANALYSES` 116 | `orderings_analysed` 165 | one pair |
| `scenario.rs` | `FEATURE_SCANS` 164, `SERIALIZED` 178 | `format_feature_scans` 171, `suites_serialized` 184 | two pairs |
| `input.rs` | `SEGMENTS_COPIED` 1687 | `path_segments_copied` 1694 | one pair |
| `fixtures.rs` | `FIXTURE_STEP_CLONES` 20, `PRECONDITION_STEP_CLONES` 21 | 27, 36 | two pairs |
| `authored.rs` | `DOCUMENTS_READ` 2966 | `documents_read` 2972 | one pair |
| `authored/ties.rs` | `REPLAYS` 29 | `replays_run` 35 | one pair; `mod ties` (`authored.rs` 270) is private, so also `pub(crate) use ties::{taken_replays, add_replays};` in `authored.rs` |
| `admission.rs` | `SUITES_PARSED` 109 | `suites_parsed` 115 | one pair |

`related_guard.rs` is also edited this wave by unit `c1` (`drive_several`, `as_own_scenario`,
`own_opening`, `arranged`, about 2830-3000 and 3770-4380), which is merged before this unit. Put
all of this unit's functions in that file (the `COMPANION_WORK`, `OWN_WORK` and
`OWN_OVERLAY_WORK` accessors and `beside_or_own_open`) directly below `companion_work`
(4985-4994), away from `c1`'s functions.

For the reading scopes (debug check in `Context::capture`), add one query each, directly below
the scope's opener:

- `witness.rs` below `under_test` (1542): `pub(crate) fn under_test_open() -> bool`
  (`UNDER_TEST` 1517 is `Some`);
- `synthesize/earlier_runs.rs` below `scenario` (39): `pub(super) fn open() -> bool`
  (`EARLIER` 22 is `Some`);
- `synthesize/related_guard.rs` below `companion_work`: `pub(super) fn beside_or_own_open() -> bool`
  (`BESIDE_OPEN.open` 1329 non-empty, or `OWN_OPEN` 2837 non-empty);
- `synthesize/subject_fact.rs` below `holding` (6530): `pub(super) fn holding_any() -> bool`
  (`HELD` 6508 non-empty).

### `crates/verify/ess-conformance/src/synthesize/threads.rs` (new)

1. `//!` doc: what crosses a thread when caller readings run elsewhere, and a pointer to
   `docs/design/parallel-caller-readings.md`. Below it:
   ```rust
   // p3 removes this: `readings::read_all` then uses `Context` and `Work`, and through them
   // every accessor; until then only this module's tests do.
   #![cfg_attr(not(test), allow(dead_code))]
   ```
2. The handling of each static:
   ```rust
   /// How a static of this crate is handled where caller readings run on another thread.
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub(super) enum Handling {
       /// Counted per thread; a worker's count is added to the calling thread's on join ([`Work`]).
       Counter,
       /// Set around a whole synthesis and read inside readings; captured where the readings are
       /// planned and set on each worker ([`Context`]).
       Context,
       /// Opened and closed inside one reading; empty wherever readings are planned (checked by
       /// [`Context::capture`] in debug builds).
       Scope,
       /// A memory that answers what the computation answers; a worker starts with it empty.
       Memory,
       /// A uniform model's memory kept across its readings; owned by the one thread that runs them.
       Lane,
       /// Read or set only on the calling thread, outside every reading.
       Caller,
       /// Read only by the crate's own unit tests, which run with one job.
       Test,
       /// A plain `static`: immutable, `Sync` by the compiler.
       Shared,
       /// In a module no synthesis runs (the process target, the web host).
       Unreached,
   }
   ```
3. `pub(super) const STATE: &[(&str, &str, Handling)]`: `(path under src/, static name,
   handling)`, sorted by path, then name. Exactly these 74 rows. Count them by kind: Counter 40,
   Context 6, Scope 6, Memory 10, Lane 2, Caller 1, Test 4, Shared 3, Unreached 2. Where they
   come from:
   - `git grep -nE '\bstatic [A-Z][A-Z0-9_]*\s*:' crates/verify/ess-conformance/src` prints 72
     lines at `ff7f8e74fb` and 73 at `80794bbd698` (the correction's `OWN_WORK`);
   - `c1` adds `OWN_OVERLAY_WORK`, the 74th. It is one static, so it is one row (Counter). Its
     `Cell<(u64, u64)>` gives `Work` two summed fields, `own_overlay_built` and
     `own_overlay_asked_at_the_bound`.

   The count is one row per `static` line the census pattern finds: 73 at `80794bbd698`, plus
   one for `c1`. `Work` has more fields than there are Counter rows, because a tuple counter
   gives one field per element. If the merged head shows another static, classify it too and
   name it in your report.
   - **Counter:**
     - `admission.rs` `SUITES_PARSED`;
     - `authored.rs` `DOCUMENTS_READ`;
     - `authored/ties.rs` `REPLAYS`;
     - `fixtures.rs` `FIXTURE_STEP_CLONES`, `PRECONDITION_STEP_CLONES`;
     - `input.rs` `SEGMENTS_COPIED`;
     - `now_offset.rs` `ANALYSES`;
     - `scenario.rs` `FEATURE_SCANS`, `SERIALIZED`;
     - `synthesize.rs` `SYNTHESES`, `COMMANDS_TAKEN`, `DERIVED_EDGES`, `BUILT_VIEW_INDEXES`,
       `OVERLAP_SEARCHES`, `PRECEDENCE_REPLAYS`, `BINDING_TRIGGERS_TRIED`;
     - `synthesize/aggregate.rs` `LITERALS`;
     - `synthesize/aggregate/contrast.rs` `DECIDED`;
     - `synthesize/created_memo.rs` `ASKED`, `MADE`;
     - `synthesize/disclosure.rs` `ORIGINS_BUILT`;
     - `synthesize/first_kept.rs` `WORK` (its two counts; its flag is context, see below);
     - `synthesize/made.rs` `MAKES`, `PICKS`, `RELATED_PICKS`;
     - `synthesize/model_memo.rs` `ABSENT_BUILT`;
     - `synthesize/ordered_head.rs` `STEERING_WORK`;
     - `synthesize/related_guard.rs` `COMPANION_WORK` (two counts; flag context), `OWN_WORK`
       (two sums and a maximum), `OWN_OVERLAY_WORK` (`c1`; the pair `(built, asked_at_the_bound)`, two sums);
     - `synthesize/row_set.rs` `ATTEMPTS` (count; flag context), `TRIED`;
     - `synthesize/run_memo.rs` `ASKED`, `MADE`;
     - `synthesize/shorthand.rs` `TRAVERSALS`;
     - `synthesize/subject_fact.rs` `FIELDS_BOUND`, `INPUTS_MADE`;
     - `witness.rs` `STEPS`, `BUILT`, `CLONED`.
   - **Context:**
     - `synthesize.rs` `SCOPE`, `DROPPED`;
     - `synthesize/grant.rs` `ADDED`, `DROPPED`;
     - `synthesize/created_memo.rs` `UNKEPT`;
     - `synthesize/run_memo.rs` `UNKEPT`.

     The three fresh flags inside `first_kept.rs` `WORK`, `related_guard.rs` `COMPANION_WORK` and
     `row_set.rs` `ATTEMPTS` are context too. The table has one row per static, so those three
     rows say `Counter`, and `Context` carries their flags; say so in a comment above each of
     those rows.
   - **Scope:**
     - `synthesize/earlier_runs.rs` `EARLIER`;
     - `synthesize/related_guard.rs` `BESIDE_OPEN` (its `cut` field is a per-thread running
       count read only as differences inside one reading, `cuts()` call sites in
       `created_memo.rs` 184-187, `made.rs` 289-291, `run_memo.rs` 130-133, `subject_fact.rs`
       2613-2632 and 6611-6635, `row_set.rs` 3257-3260, `related_guard.rs` 3067-3080 and
       4952-4968, so a worker's starting at 0 changes no answer), `OWN_OPEN`;
     - `synthesize/subject_fact.rs` `HELD`;
     - `witness.rs` `UNDER_TEST`, `REFERENCE` (debug only).
   - **Memory:**
     - `now_offset.rs` `ANALYSED`;
     - `synthesize.rs` `OVERLAP_ANSWERS`;
     - `synthesize/created_memo.rs` `KEPT`;
     - `synthesize/made.rs` `MADE`;
     - `synthesize/model_memo.rs` `KEPT`;
     - `synthesize/run_memo.rs` `KEPT`;
     - `synthesize/seeds.rs` `DIGESTED`;
     - `synthesize/subject_fact.rs` `ROWS`;
     - `witness_memo.rs` `MEMO`, `SHARED` (the debug guard's sampling count: which answers
       from memory are searched again; it changes no answer).
   - **Lane:** `synthesize/made.rs` `KEPT`; `witness_memo.rs` `KEPT`.
   - **Caller:** `mutate.rs` `GRANTS_KEPT` (read in `synthesized_from`, 6734, after the
     synthesis returned).
   - **Test:**
     - `synthesize/caller.rs` `WHOLE`, `UNFOCUSED` (`UNFOCUSED` is read where the plan is built,
       on the calling thread, unit `p1`);
     - `witness_memo.rs` `WORK`, `GENERATIONS`.
   - **Shared:** `process/tests.rs` `SUITE`; `synthesize.rs` `NOTHING`; `synthesize/seeds.rs`
     `EMPTY`.
   - **Unreached:** `process/exact.rs` `VISITED`; `web_execution/host.rs` `HOST`.

   Two names appear in two files: `KEPT`, `MADE`, `DROPPED` and `ASKED`. So a row is keyed by
   path and name together.
4. `Context`:
   ```rust
   /// What a caller reading reads from the thread it runs on, as the planning thread holds it.
   #[derive(Debug, Clone)]
   pub(super) struct Context {
       scope: Option<BTreeSet<QualifiedName>>,
       dropped_write: Option<super::DroppedWrite>,
       dropped_grant: Option<super::DroppedGrant>,
       added_grant: Option<super::AddedGrant>,
       phase_order: [ess_domain::command::precedence::Phase; 8],
       created_unkept: bool,
       runs_unkept: bool,
       first_fresh: bool,
       attempts_fresh: bool,
       companion_fresh: bool,
   }
   ```
   - `pub(super) fn capture() -> Self` reads every field through the accessors above.
     `phase_order` comes from `ess_domain::command::precedence::phase_order()`, which is public
     (`precedence.rs` 581). It also carries `debug_assert!(!crate::witness::under_test_open() && !super::earlier_runs::open() && !super::related_guard::beside_or_own_open() && !super::subject_fact::holding_any(), "caller readings are planned outside every reading's scope")`.
   - `pub(super) fn enter<R>(&self, run: impl FnOnce() -> R) -> R` sets every field on the
     current thread, runs `run` and restores what was there, also on a panic. It uses only the
     existing setters, nested, outermost first:
     1. `with_phase_order(self.phase_order, ..)` (`precedence.rs` 596, `#[doc(hidden)] pub`);
     2. `super::with_dropped_write(self.dropped_write.clone(), ..)`;
     3. `super::with_dropped_grant(..)`;
     4. `super::with_added_grant(..)`;
     5. where `scope` is `Some`, `let _scope = super::scoped_to(scope.clone());`;
     6. where `created_unkept`, `super::created_memo::unkept(..)`; where `runs_unkept`,
        `super::run_memo::unkept(..)`;
     7. `super::first_kept::first_rows_made(self.first_fresh, ..).0`,
        `super::row_set_attempts(self.attempts_fresh, ..).0`,
        `super::companion_work(self.companion_fresh, ..).0`. These three set the flag and
        restore it. The counts they return are dropped, because the counters keep counting on
        the thread, and `Work` reads them there.
   - Not in `Context`, each with its reason in the design page:
     - `GRANTS_KEPT` (read on the calling thread after the synthesis);
     - the reading scopes (empty at capture, asserted);
     - the memories (they start empty and answer the same);
     - `ess-domain`'s assembly flags `CONVERTING_ATTRIBUTES` (`attributes.rs` 382),
       `ADMITS_DELETIONS` (`set_effects.rs` 74) and the filtered-read flag (`row_set.rs` 617),
       set only while a specification is assembled, never around a synthesis;
     - `MAP_KEY_NEWTYPES` (`types.rs` 282), set only by `MapKeyNewtypes::scope` (447), which no
       synthesis opens.
5. `Work`:
   ```rust
   /// Every work counter of this crate on one thread, in [`STATE`] order.
   #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
   pub(super) struct Work { /* one field per counter of the Counter rows; tuples as two fields */ }
   ```
   - `pub(super) fn taken() -> Self` reads the current thread's values.
   - `pub(super) fn add(&self)` adds them to the current thread.
   - `pub(super) fn since(&self, earlier: &Self) -> Self` is the field-wise difference, used by
     the test.
   - `pub(super) const NAMES: &[&str]` lists the counters as `"<path>:<STATIC>"`.
   - A worker's `Work::taken()` at its end is its own count, because thread-locals start at 0 on
     a new thread. The planning thread adds it after joining.
   - Not summed, each with its reason in the design page: `ess-compiler`'s `binary64::SCANS`
     (`binary64.rs` 42-45) and `ess-domain`'s `DECLARATIONS_EXAMINED`, `CONTAINMENT_CHECKS`
     (`domain.rs` 175-180), `TYPE_CHECK_WORK` (`system.rs` 1168) and the `MapKeyNewtypes` copy
     count. They belong to other crates and have no adder. The only test reading one around a
     synthesis, `once_per_model_work.rs` 72-92, uses models without actor attributes, which never
     run a reading on another thread.
6. Compile-time assertions:
   ```rust
   const _: () = {
       const fn send<T: Send>() {}
       const fn sync<T: Sync>() {}
       sync::<EssIr>();
       send::<EssIr>();
       sync::<super::AdmittedSeeds>();
       send::<super::Synthesis>();
       sync::<Context>();
   };
   ```
   If one fails to compile, the error names the field that holds an `Rc` or a `Cell`. Replace it
   with its `Arc` or plain equivalent only where it is a plain data field that is not a memo.
   Name it in your report.

### `docs/design/parallel-caller-readings.md` (new)

Sections, in this order. Brand-free: name no downstream project; say "the largest known
downstream specification".

1. **What runs where.** A caller reading is one `synthesize_invocations` of one rewritten model
   under one focus. The plan, in order, is `p1`'s `readings::plan`: two whole readings, then two
   focused runs per mixed command, then two cross runs per mixed command. The arranged-callers
   stage follows: first readings, adoptions, swapped readings, appends. The merge stays on the
   calling thread, in today's order, over results indexed by plan position.
2. **Lanes.** A reading whose arrangement is a uniform model runs on that model's one thread, in
   plan order, so it reuses that model's kept memory as one thread does. That covers each whole
   reading, every cross run, and every focused run of a command outside `rewritten` or of the
   only command in it. The other readings go to any thread. This is the parent story's
   decision D2 C. Say why the memory is not shared: `made` and `witness_memo` memories hold `Rc`
   values on one thread's stack.
3. **Thread state.** A table of the 74 statics (77 once `p3` adds its three): path, name, handling, and one line on what a
   worker sees. Then the cross-crate items of points 4 and 5 above.
4. **Determinism.** Results are merged by plan index, never in arrival order. No unordered
   iteration is added. A memory answers what the computation answers: the debug guard of
   `witness_memo.rs` 337-350 checks a sample, and the byte tables check the whole corpus. So the
   bytes do not depend on which thread ran a reading, nor on the memory it started with.
5. **Memory.** One thread keeps both uniform models' memories for the whole caller synthesis
   today: after the first whole reading 2.57 GB, after the second 4.81 GB, at the end 6.09 GB on
   the largest known downstream specification. Two lanes keep the same two memories. Each further
   thread adds the working memory of one reading. The default is one thread. Leave a
   `## Measured` heading with the line "Filled by the measurement unit." (`p5` writes it.)

## Tests first

Unit tests in `threads.rs`'s own `#[cfg(test)] mod tests`, in the library test binary: no new
test binary. Time each. They should take well under 5 s; one synthesizes a 1.4 KB model. No test
over 60 s, the step within 120 s.

1. `every_static_of_the_crate_is_classified`. Walk `concat!(env!("CARGO_MANIFEST_DIR"), "/src")`
   recursively in byte order of path, using `std::fs` only. On every line, find each occurrence
   of `static ` followed by an upper-case identifier and `:`. Skip lines whose trimmed start is
   `//`, and skip `'static`. Collect `(path relative to src/ with '/', name)`. Assert the set
   equals `STATE`'s `(path, name)` set. The message lists the missing pairs and the extra ones,
   each on its own line, and ends with: "classify it in `STATE` (`src/synthesize/threads.rs`) and,
   for a Counter, add it to `Work`". Also assert that `STATE` is sorted and has no duplicate.
   Red today: `error[E0583]` or `E0432`, no module `threads`.
2. `work_names_every_counter_row`. `Work::NAMES` equals the `Counter` rows of `STATE`, as
   `"<path>:<name>"`, in order. Red: the same compile error.
3. `a_worker_sees_the_context_of_the_planning_thread`. On the test thread, nest:
   - `with_dropped_write(Some(DroppedWrite { command: "shop.order.Place".into(), outcome: "placed".into(), target: "note".into(), source: "note".into() }), ..)`;
   - `with_dropped_grant(Some(DroppedGrant { actor: "shop.order.Clerk".into(), command: "shop.order.Place".into() }), ..)`;
   - `with_added_grant(Some(AddedGrant { .. same .. }), ..)`;
   - `let _scope = scoped_to(BTreeSet::from([QualifiedName::new("shop.order.Place").unwrap()]));`;
   - `created_memo::unkept(..)`;
   - `first_rows_made(true, ..)`;
   - `with_phase_order(<the PRECEDENCE order with its first two phases exchanged>, ..)`.

   Inside, `let context = Context::capture();`, then
   `std::thread::scope(|s| s.spawn(|| (context.enter(Context::capture), Context::capture())).join().unwrap())`.
   Assert:
   - the first of the pair equals `context`, field by field (derive `PartialEq` on `Context`);
   - the second (on the worker, outside `enter`) equals a fresh `Context` with every field at
     its default.

   Red: the same compile error.
4. `a_workers_counts_are_added_to_the_planning_thread`. In a spawned scoped thread, synthesize
   `include_str!("../../tests/fixtures/caller-struct-identity.yaml")` through
   `crate::synthesize::synthesize` (compile it as `caller.rs`'s tests do, `caller.rs` 3645-3650)
   and return `Work::taken()`. On the test thread: `let before = Work::taken(); worker.add();`.
   Assert:
   - `Work::taken().since(&before) == worker`;
   - `worker != Work::default()`: the synthesis counted something;
   - `creations_made(|| worker.add()).1 == worker.<makes field>`: the public reader sees the
     added count.

   Red: the same compile error.
5. `capturing_inside_a_reading_scope_is_refused_in_debug`.
   `#[cfg(debug_assertions)] #[should_panic(expected = "caller readings are planned outside every reading's scope")]`.
   Call `let _open = earlier_runs::scenario(); Context::capture();`. Red: the same compile error.

The existing suites must stay green unchanged. This unit changes no behaviour.

## Steps

1. Create `threads.rs` with `Handling`, `STATE` and test 1. Run it and keep its output: it is
   your check that the 74 rows are complete.
2. Add the accessors (table above), then `Context`, `Work` and the assertions, then tests 2-5.
3. Write the design page.
4. Run the gate.
5. The statics of other units, by order:
   - the correction's `OWN_WORK` is on the plan tree's head;
   - `c1`'s `OWN_OVERLAY_WORK` is merged before this unit.

   Both are classified here as `Counter`, one row each, with two accessors each and one `Work` field per tuple element (`OWN_WORK` three, `OWN_OVERLAY_WORK` two). `p3` lands
   after this unit and classifies its own (`JOBS`, `OFF_THREAD`, `PANIC_IN`). No other unit of
   the wave adds a static to `ess-conformance/src`. Test 1 names any static left out.
6. One commit, message file via the Write tool, `git commit -F <file>`:
   `refactor(conformance): every static a synthesis reads is classified for caller readings on other threads`.

No generated file changes. `docs/` changed, so `task test-xtask` runs (AGENTS.md § Gate table).

## Owned files

- `crates/verify/ess-conformance/src/synthesize/threads.rs` (new)
- `docs/design/parallel-caller-readings.md` (new; `p5` appends `## Measured` later)
- In the files of the accessor table: only the new accessor functions, the four scope queries
  and, in `synthesize.rs`, `mod threads;`, `scope()` and `dropped_write()`.

## Not yours

- `synthesize/caller.rs`, `synthesize/caller/readings.rs`, and the `Keeping`s of `made.rs` and
  `witness_memo.rs`. These are `p1`'s, merged before. In `made.rs` this unit adds functions only
  beside the counters (69-117), and `p1` edited only `Keeping` (308-351).
- `drive_several`, `as_own_scenario`, `own_opening`, `arranged` and `OWN_OVERLAY_WORK`'s
  declaration in `related_guard.rs`: those are `c1`'s, merged before.
- `mutate.rs`: nothing here edits it. `GRANTS_KEPT` is only classified.
- Every file under `tests/`.

## Gate

Each command is one pool job. `cargo` is the shim. Bounds:

- 60 s per test, 120 s per step;
- a job ends at 10 minutes unless it names its time, 30 minutes at most;
- no `RUSTC_WRAPPER`, `SCCACHE_*` or `-j`.

```console
slot task fmt-check
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-conformance --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-cli --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --lib synthesize::threads
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test caller_readings_made_once_work --test companion_rows_searched_once_work --test row_set_many_selectors_work --test stored_rows_decided_once_work --test once_per_model_work --test own_scenario_nesting_bounded_work --test mutation_grants --test mutation_sets_entries
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table
SLOT_TIMEOUT=30m slot task test-xtask
```

- The byte-pin target runs as it is today (routing decision X2); `t2` lands after every `p` unit.
- The accessors only add functions, so `ess-cli` is gated for Clippy only.
- Clippy with `-D warnings` is clean through the one allow of D7. If it still names an accessor
  as dead, the fallback is an item-level allow on that accessor only, always with the comment
  `p3 removes this`: `#[cfg_attr(not(test), allow(dead_code))] // p3 removes this`. Your report
  lists every file that got one, with the accessor (routing decision Y2). `p3` removes every
  such line.

## Acceptance

1. `every_static_of_the_crate_is_classified` passes with 74 rows, one per `static`: the 73 of
   `80794bbd698` plus `c1`'s `OWN_OVERLAY_WORK`, one Counter row whose pair gives `Work` two
   summed fields. Any further static of the merged head is named in the report, with its row.
2. `work_names_every_counter_row`, `a_worker_sees_the_context_of_the_planning_thread`,
   `a_workers_counts_are_added_to_the_planning_thread` and
   `capturing_inside_a_reading_scope_is_refused_in_debug` pass.
3. The library compiles with the `Send`/`Sync` assertions and no `unsafe`:
   `git grep -n unsafe crates/verify/ess-conformance/src/synthesize/threads.rs` prints nothing.
4. The work and mutation targets in `## Gate` and `synthesis_suite_bytes_table` pass unchanged.
5. `task test-xtask` passes with the new design page.
6. `git grep -nwE 'spawn|spawn_scoped' crates/verify/ess-conformance/src` lists only lines of
   `threads.rs`'s tests. `-w` keeps `respawn` (`process.rs`, `process/tests.rs`) out; at
   `80794bbd698` the command prints nothing.
7. Clippy is clean with `-D warnings`. `git grep -n 'p3 removes' crates/verify/ess-conformance/src`
   lists the allow of D7 and the item-level allows the report lists (file and accessor), and
   nothing else.

## Decisions

- D1. Every work counter of the crate is summed, not only those the `*_work.rs` tests read. One
  rule is easier to keep than a judgement per counter, and an unread counter costs two lines.
  `VISITED` and `HOST` are the exceptions, because no synthesis runs their modules.
- D2. Context is re-established per worker from the caller's value, through the existing
  `with_*` setters. No reading refuses to go parallel because of one: every value is cheap to
  clone.
- D3. The reading scopes are asserted empty where the plan is built, not carried. No code opens
  one around `caller::synthesize`: it is reached only from `synthesize_in` (`synthesize.rs` 3621),
  whose callers are the public entry points and `mutate.rs` (6689, 6723).
- D4. The registry is a test that reads the sources, inside the library (anything executable is
  Rust; no shell checker; no new test binary).
- D5. Other crates' counters are not summed (no adder exists, and adding one is a change to
  `ess-compiler` and `ess-domain` that no test needs). The design page names them.
- D6. The design page is written here because its core is this unit's table. `p1` (merged
  before) built the plan and the lanes it describes; `p3` builds the threads.
- D7. Dead code until `p3` (routing decision X3), handled by a named allow rather than by moving
  the items into `p3`. `Context` and `Work` are what this unit's tests prove, so they belong with
  the registry. The allow is `#![cfg_attr(not(test), allow(dead_code))]` at the top of
  `threads.rs`, with the comment "p3 removes this". `rustc` treats an item under `allow(dead_code)`
  as used, and so also the accessors it calls. `p3`'s steps remove the allow, and `p3`'s
  acceptance 11 checks that it is gone.

## Changelog

none.
