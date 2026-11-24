---
format: aep.planning-md/3
id: story:every-caller-reading-of-a-synthesis-is-computed-from-explicit-inputs-in-a-fixed-plan
kind: story
status: active
title: Every caller reading of a synthesis is computed from explicit inputs in a fixed plan
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv
- confidence: cited
  path: crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/caller.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/caller/arranged.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/caller/readings.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/made.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness_memo.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-component-suites-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
revision: 17
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:03:05Z", actor: "human:timo", revision: 16, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:03:07Z", actor: "human:timo", revision: 17, decided_on: {"recorded":{"review_outcome":4}}}
---
## Outcome

- `caller::synthesize` (`crates/verify/ess-conformance/src/synthesize/caller.rs`) no longer runs a
  reading in the middle of a merge. It first builds a **plan**: a `Vec<Reading>` in a fixed order.
  It computes every reading of the plan through one function, `readings::read_all`, which returns
  the results in plan order. Then it merges those results in exactly today's order.
- The arranged-callers stage (`arranged_by_callers`) is split the same way: all first readings,
  then all adoptions in command order, then all swapped readings, then all appends in command
  order. The notes come out in today's order.
- `read_all` runs the plan inline on the calling thread, in plan order. This unit adds **no
  thread**. The next unit of this story (`p3`) replaces only the body of `read_all`.
- `Assigned` holds no `RefCell` and no `Rc`, and its two uniform models are `Arc<EssIr>`. No
  `Send` or `Sync` bound and no compile-time guard is written here; `p3` adds them where the
  threads are (routing decision X3). The memory those two models keep across syntheses (`made::Keeping`,
  `witness_memo::Keeping`) is opened by a guard, `readings::Kept`. The guard is `!Send`, because
  dropping it pops the stack of the thread that opened it.
- Every byte a synthesis writes is unchanged. That covers suite, refusals, notes and outside, for
  every repository model, for every component, and with seeds. No format version changes and no
  diagnostic changes.

## Why

- Parent: `story:the-caller-readings-of-a-synthesis-run-on-several-threads-and-write-the-suite-a-single-thread-writes`, the caller readings of a synthesis run on several threads and write
  the suite a single thread writes. This is its first unit: it changes no behaviour, and it makes
  every reading a function of explicit inputs in a fixed order, so the thread unit only has to
  change who runs the plan.
- Today `about_command`, `cross_single_command` and `arranged_by_callers` interleave readings with
  the merge (`caller.rs` 695-729, 995-1059). A thread pool cannot run what the merge has not asked
  for yet.

## Where

Read at commit `ff7f8e74fb`, and re-checked at `80794bbd698` (`plan/wave-2026-11-23`, the plan
tree's head). Between the two, `caller.rs`, `caller/arranged.rs`, `made.rs` and `witness_memo.rs`
did not change, so every line number below holds at both. Record your base first:
`BASE=$(git rev-parse HEAD)`. If a file moved, find each site by its name.

### `crates/verify/ess-conformance/src/synthesize/caller.rs` (3797 lines)

- Line 30, `mod arranged;`: add `mod readings;` below it.
- `Assigned` (353-361) and `impl Assigned` (363-414). Rewrite them as:
  ```rust
  struct Assigned<'a> {
      ir: &'a EssIr,
      callers: &'a Callers,
      /// The commands whose reading depends on who sends them, in name order.
      rewritten: Vec<QualifiedName>,
      /// The models in which every rewritten command is sent by one caller, by assignment key:
      /// two models, or one where `rewritten` is empty and both callers read the model the same.
      uniform: BTreeMap<Vec<Who>, std::sync::Arc<EssIr>>,
  }
  ```
  - `new` keeps the `rewritten` computation of lines 365-374 unchanged. Then, for `who` in
    `[Who::First, Who::Second]` in that order, it computes `key = rewritten.iter().map(|_| who)`
    and inserts `Arc::new(ir.with_commands_rewritten(..))` with `entry(key).or_insert_with`.
    That is the same rewrite closure as lines 397-403, sending every command as `who`. So an
    empty `rewritten` yields one model, made for `First`, exactly as today's cache gave both
    calls the model the first call made.
  - `fn key(&self, who: Assignment<'_>) -> Vec<Who>` is today's line 392.
  - `fn model(&self, who: Assignment<'_>) -> Arc<EssIr>` returns `Arc::clone` of `uniform[key]`
    where present. Otherwise it returns a fresh `Arc` of the rewritten model, not kept. This is
    the non-uniform branch of lines 404-406.
  - `fn uniform_models(&self) -> Vec<&Arc<EssIr>>` returns the values of `uniform` in key order.
  - The `RefCell` fields `models`, `made` and `witnessed` (358-360) are deleted. The memory those
    Keepings held is held by `readings::Kept` (below), opened in `synthesize`.
- `synthesize` (650-739). Restructure. Keep every statement that is not a reading as it is, in
  the same order relative to the merge.
  1. Lines 651-656 stay. After `let assigned = Assigned::new(ir, &callers);` add
     `let _kept = readings::Kept::open(&assigned.uniform_models());`. It lives until the function
     returns, as `Assigned`'s Keepings do today.
  2. Move the computation of `mixed_commands` (686-694) up, before the readings, as
     `fn mixed(ir: &EssIr, callers: &Callers) -> BTreeSet<QualifiedName>` with the same body. It
     is pure: it reads only `ir` and `callers`. Test 2 calls it.
  3. `let plan = readings::plan(&mixed_commands, unfocused);`. `unfocused` is
     `#[cfg(test)] tests::UNFOCUSED.get()` and `false` otherwise. Read it here, on the calling
     thread, as `about_command` reads it today (936-942).
  4. `let mut results = readings::read_all(&plan, |reading| assigned.lane(reading), |reading| readings::read(&assigned, seeds, reading)).into_iter();`.
     Then take, in plan order: `whole`, `swapped`, then one `(other, back)` pair per mixed
     command, then one `(mixed, reversed)` pair per mixed command.
  5. The loop over `whole.suite.scenarios` (663-683) is unchanged, `one_row_acted_on` included:
     it still runs its reading inline when the merge asks for it.
  6. The loop over `mixed_commands` (695-727) consumes the `(other, back)` pairs in order, in
     place of calling `about_command`. Before the loop body, apply the rotation check of
     `about_command` (946-958): where the pair's readings were focused (`!unfocused`) and
     `turned(command, &other, &back, &rotated)`, replace the pair with
     `readings::read(&assigned, seeds, &Reading::About { command, back: false, whole: true })`
     and the same with `back: true`, run inline. `turned` is lines 946-952 extracted as a
     function. The body 705-726 is unchanged.
  7. `cross_single_command(..)` (728) becomes `merge_cross(ir, callers, crosses, &mut whole)`.
     `crosses` is the `(command, mixed, reversed)` triples in name order. Its body is lines
     839-890, run per triple. The model building of 816-830 moves into `readings::read`.
  8. `arranged_by_callers` (729), `seeded_unswapped` (730) and 731-738 stay in place.
- `cross_single_command` (812-892): replaced by `merge_cross` as above. Nothing in its merge
  changes: notes retained and pushed in the same order, refusals retained and extended the same.
- `about_command` (930-960): deleted. Its UNFOCUSED branch becomes the `whole` flag of the plan,
  its two focused runs are plan entries, and its `turned` branch is step 6.
- `run` (964-966) and `run_seeded` (969-989): unchanged except that `assigned.model(who)` now
  returns `Arc<EssIr>`. `&written` still derefs to `&EssIr`, and the `WHOLE` count at 975-978
  stays where it is.
- `arranged_by_callers` (995-1059). Split it into four passes over
  `let commands: Vec<QualifiedName> = arranged::refused_commands(ir, whole).into_iter().collect();`.
  1. `firsts = readings::read_all(&commands, |_| None, |command| arranged::lifted_model(ir, callers, &|_| Who::First, command).map(|(model, lifted)| (synthesize_plain(&model, Focus::About(command)), lifted)))`.
  2. A serial loop over `commands.iter().zip(firsts)`. Skip `None`, as line 1006 `continue`s.
     Otherwise run lines 1009-1019, and wrap the `adopt` call so its notes are set aside:
     `let mark = whole.notes.len(); any |= arranged::adopt(..); let adopted_notes = whole.notes.split_off(mark);`.
     Keep `(command, adopted, adopted_notes)` for every command whose `adopted` is non-empty.
  3. `swapped = readings::read_all(&kept, |_| None, |(command, ..)| arranged::lifted_model(ir, callers, &|_| Who::Second, command).map(|(model, lifted)| { let mut again = synthesize_plain(&model, Focus::About(command)); arranged::finish(&mut again, &lifted, &|step| mark(ir, callers, &|_| Who::Second, step)); again }))`.
     This is lines 1023-1031 unchanged.
  4. A serial loop in command order. First `whole.notes.extend(adopted_notes)`, then lines
     1032-1047 unchanged.

  Then `if any { grant::denied_since(..) }` (1050-1058), unchanged. Why the order holds:
  `adopt` reads only `whole.refusals` and the keys of `whole.suite.scenarios`
  (`arranged.rs` 233-239), and only refusals and scenarios about its own command (`about`,
  `caller.rs` 1321-1335, compares the whole command name). `append` reads the scenario, `again`
  and `identities`, and writes only that scenario's steps and `whole.notes`. So running every
  `adopt` before every `append` changes nothing but the position of the notes, and step 4 puts
  them back.
- `one_row_acted_on` (1568-1606): unchanged. Its `run(..)` returns through the new `model`.

### `crates/verify/ess-conformance/src/synthesize/caller/readings.rs` (new)

- A `//!` doc: what a reading is, the plan order, and that `read_all` returns results in plan
  order whoever computes them.
- The plan entries:
  ```rust
  pub(super) enum Reading<'q> {
      /// The whole model read as `Who` sends every command; seeded for `First` only.
      Whole(Who),
      /// One command's run for the mixed-caller loop: sent as `Second` with every other command
      /// sent as `First` (`back: false`), or the reverse (`back: true`); a whole synthesis where
      /// `whole`, focused on the command otherwise.
      About { command: &'q QualifiedName, back: bool, whole: bool },
      /// `merge_cross`'s run of one command: arranged as `First` and acted as `Second`, seeded
      /// (`reversed: false`), or the reverse, seed-free.
      Cross { command: &'q QualifiedName, reversed: bool },
  }
  ```
- `pub(super) fn plan<'q>(mixed: &'q BTreeSet<QualifiedName>, unfocused: bool) -> Vec<Reading<'q>>`
  returns, in this order: `Whole(First)`, `Whole(Second)`; for each `c` in `mixed` (name order)
  `About { c, back: false, whole: unfocused }` then `About { c, back: true, whole: unfocused }`;
  then for each `c` in `mixed` `Cross { c, reversed: false }` then `Cross { c, reversed: true }`.
  This is the order in which today's code runs them (657, 658, 944-945 per command,
  837-838 per command).
- `pub(super) fn read(assigned: &Assigned<'_>, seeds: &AdmittedSeeds, reading: &Reading<'_>) -> Synthesis`:
  - `Whole(First)` is `run_seeded(assigned, &|_| Who::First, Focus::Whole, seeds)`.
  - `Whole(Second)` is `run(assigned, &|_| Who::Second, Focus::Whole)`.
  - `About { c, back: false, whole }` is `run_seeded(assigned, sent, focus, seeds)`, where
    `sent` is `Second` for `c` and `First` otherwise (703), and `focus` is `Focus::Whole` where
    `whole`, else `Focus::About(c)`.
  - `back: true` is `run(assigned, back, focus)`, where `back` is `sent(name).other()`.
  - `Cross { c, reversed }` builds `forward` or `backward` exactly as lines 817-830 do, from
    `assigned.model(&|_| Who::First)` and `assigned.model(&|_| Who::Second)`. Then it calls
    `super::super::synthesize_invocations(&models, Focus::About(c))`.
- `impl Assigned { pub(super) fn lane(&self, reading: &Reading<'_>) -> Option<&Arc<EssIr>> }`.
  It returns the uniform model the reading's arrangement is, or `None`:
  - `Whole(w)`: `uniform[key(all w)]`.
  - `About`: `self.uniform.get(&self.key(assignment))`. The assignment is the one `read` uses.
  - `Cross { reversed: false }`: the all-`First` model. `reversed: true`: the all-`Second`
    model.

  Put it here and not in `caller.rs`, because only the executor reads it. `p3` uses it to keep
  each uniform model's memory on one thread.
- `pub(super) fn read_all<'m, J, R>(jobs: &[J], lane: impl Fn(&J) -> Option<&'m Arc<EssIr>>, read: impl Fn(&J) -> R) -> Vec<R>`.
  The named `'m` lets the lane borrow from `Assigned` rather than from the job. Its body in this
  unit is `jobs.iter().map(read).collect()`, and `lane` is unused (`let _ = &lane;`). The
  signature has no `Send` or `Sync` bound: `p3` adds `J: Sync`, `R: Send` and `+ Sync` on the two
  closures when it runs them on threads. So this unit compiles whatever those traits say of
  `Synthesis`, `EssIr` and `AdmittedSeeds`.
- `pub(super) struct Kept { _made: made::Keeping, _witnessed: crate::witness_memo::Keeping }`
  with `pub(super) fn open(models: &[&Arc<EssIr>]) -> Self`. `open` calls `Keeping::open()` on
  both and then `keep(model)` for each model, made first, as `Assigned::new` and `model` do
  today (380-381, 407-408).
- No compile-time `Sync` guard in this unit; `p3` adds it.

### `crates/verify/ess-conformance/src/witness_memo.rs` (588 lines)

- `Keeping` (119-152): `models: Vec<std::rc::Rc<EssIr>>` (124) becomes
  `Vec<std::sync::Arc<EssIr>>`. `keep(&mut self, ir: &std::rc::Rc<EssIr>)` (135) becomes
  `keep(&mut self, ir: &std::sync::Arc<EssIr>)`, and line 141 becomes `Arc::clone`.
- Add the field `_thread: std::marker::PhantomData<*const ()>`, set in `open`, documented as:
  "A Keeping pops the stack of the thread that opened it, so it is not sent to another thread".
  The raw pointer makes the type `!Send` and `!Sync` without `unsafe`.

### `crates/verify/ess-conformance/src/synthesize/made.rs` (380 lines)

- `Keeping` (322-351): the same two changes. `models: Vec<Rc<EssIr>>` (323) becomes
  `Vec<std::sync::Arc<EssIr>>`, `keep(&mut self, ir: &Rc<EssIr>)` (334) takes
  `&std::sync::Arc<EssIr>`, and `Rc::clone` (340) becomes `Arc::clone`. Add the same
  `_thread` field. `use std::rc::Rc;` (35) stays: `Made` still uses it.

Nothing else in either file changes. In particular the memo values (`Rc<Selections>`,
`Rc<RowPicks>`, `Rc<dyn Any>`) stay `Rc`: they live on one thread's stack and never cross.

## Tests first

All tests are unit tests in the library's own test binary: no new test binary. Time each new test
in a debug build and name its time in your report (they are expected well under 1 s). No test may
run over 60 s, and a test step stays within 120 s.

In `caller.rs`'s `mod tests` (3522), next to the existing tests. Use its `compiled` (3645) and
`NOTES` (3541).

1. `the_plan_lists_each_reading_once_in_the_order_the_merge_reads_them`. On `compiled(NOTES)`,
   `mixed` holds the commands whose two callers differ (`CreateNote` and `CloseNote`, as
   `a_run_written_for_one_command_gives_the_suite_the_whole_runs_gave` counts 2 + 2·2).
   `readings::plan(&mixed, false)` rendered as text is exactly `Whole(First)`, `Whole(Second)`,
   `About(CloseNote,first)`, `About(CloseNote,back)`, `About(CreateNote,first)`,
   `About(CreateNote,back)`, `Cross(CloseNote,forward)`, `Cross(CloseNote,reversed)`,
   `Cross(CreateNote,forward)`, `Cross(CreateNote,reversed)`. Write a small local render
   function; do not derive `Debug` on `Reading` only for this. Red today: `error[E0433]`, no
   module `readings`.
2. `a_reading_on_a_uniform_model_names_that_model_as_its_lane`. On `compiled(NOTES)`,
   `Assigned::new` gives `rewritten == [CloseNote, CreateNote]`, both reading the caller.
   Assert:
   - `lane(Whole(First))` and `lane(Cross{reversed:false})` are the same `Arc` (`Arc::ptr_eq`),
     and `lane(Whole(Second))` and `lane(Cross{reversed:true})` are the other one;
   - the two are distinct;
   - `lane(About{CreateNote, back:false, ..})` is `None`, because its key `[First, Second]` is
     mixed.

   Then the case of a command outside `rewritten`. In `NOTES` (`caller.rs` 3541-3634), `AddTag`
   is held only by `Clerk` and `Auditor` (3570-3573), which declare no attributes. So
   `Callers::for_command` gives it the empty map for both callers, and it is not in
   `mixed_commands` (686-694). NOTES has no mixed command outside `rewritten`, so the test makes
   one. Compile a copy of `NOTES` in which the attributed actor `AccountUser` (3565-3569) also
   holds it: replace `may: [demo.notes.CreateNote, demo.notes.CloseNote]` with
   `may: [demo.notes.CreateNote, demo.notes.CloseNote, demo.notes.AddTag]`. Then:
   - `AddTag`'s two callers carry `AccountUser`'s two different value sets (`Callers::of`
     266-267), so it is mixed;
   - `AddTag` reads no caller, so `written` leaves it unchanged, and it stays outside
     `rewritten` (365-374).

   The test asserts both premises first: `mixed(&ir, &callers)` of the copy contains
   `demo.notes.AddTag`, and `Assigned::new(..).rewritten` does not. Then it asserts that
   `lane(About{AddTag, back:false, ..})` is the all-`First` model and
   `lane(About{AddTag, back:true, ..})` the all-`Second` one: a command outside `rewritten` reads
   the uniform model.

   Red today: `error[E0433]`, no module `readings`, because `lane` and `readings::Reading` do not
   exist yet. Nothing else in the test is new: both premises hold on today's code.
3. The existing tests of this module stay green unchanged. They are the order proof at the level
   of whole runs:
   - `a_caller_reading_command_costs_no_whole_synthesis_of_its_own` (3682): `whole == 2`;
   - `a_command_sent_in_turn_reads_its_scenarios_from_the_whole_runs` (3702): `whole == 4`, so
     the rotation re-run still happens;
   - `a_run_written_for_one_command_gives_the_suite_the_whole_runs_gave` (3716):
     `2 + 2 * 2`, with byte-equal suites.

The byte proof is the repository's own pins, run unchanged (`## Gate`). There is no new byte
test in this unit: at one thread the bytes are today's by construction, and the pins hold every
caller fixture in `crates/verify/ess-conformance/tests/fixtures`.

## Steps

1. Read `caller.rs` 643-1059 and 1568-1606, `caller/arranged.rs` 222-287, and the two
   `Keeping`s. Write tests 1-2 and record their red lines:
   `slot cargo test -p ess-conformance --lib --locked synthesize::caller::tests` (through the
   shim, `cargo` takes its slot itself; use `SLOT_TIMEOUT=30m` for the first build).
2. Change `witness_memo.rs` and `made.rs` (Arc, `_thread`). Build.
3. Create `caller/readings.rs` and rewrite `Assigned`. Build.
4. Restructure `synthesize`, add `merge_cross`, delete `cross_single_command` and
   `about_command`. Build.
5. Split `arranged_by_callers` into its four passes. Build.
6. Run the gate. If a byte pin moves, the restructure changed an order: find it and fix it. No
   pin is re-pinned in this unit, and no table row moves.
7. One commit, message file written with the Write tool and committed with
   `git commit -F <file>`:
   `refactor(conformance): caller readings are planned, computed in plan order and then merged`,
   with a body naming the three interleavings removed and the byte pins that held.

No generated file changes. No `cargo xtask` command applies.

## Owned files

- `crates/verify/ess-conformance/src/synthesize/caller.rs`: everything listed under Where. Not
  its `mod tests`, except the two new tests.
- `crates/verify/ess-conformance/src/synthesize/caller/readings.rs` (new). `p3` edits it after
  this unit has merged.
- `crates/verify/ess-conformance/src/witness_memo.rs`: `Keeping` only.
- `crates/verify/ess-conformance/src/synthesize/made.rs`: `Keeping` only.

## Not yours

- `src/synthesize/threads.rs`, every counter accessor and the design page belong to `p2`, which
  lands after this unit (`p2` depends on `p1`). `p2` adds functions to `made.rs` (beside `MAKES`,
  `PICKS` and `RELATED_PICKS`, lines 69-117), never in `Keeping`.
- `synthesize.rs` (`p2`, `p3`), `caller/arranged.rs` (no unit edits it), every file under
  `tests/` (`p3`), `crates/edge/ess-cli` (`p4`), `crates/edge/ess-xtask` (`p5`).
- `related_guard.rs` and the three model tables: unit `c1`
  (`a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose`),
  in round 1 beside this unit. This unit reads the two synthesis tables
  (`synthesis-suite-bytes-table.tsv`, `synthesis-component-suites-table.tsv`) as pins and
  re-pins none. Only `c1` re-pins: its three tables are those two files' targets plus
  `selection-precedence-table.tsv` and `lowered-definitions-table.tsv`.
- Landing beside `c1` (routing decision Y4): the second of this unit and `c1` to land reruns
  the three model-table targets on the merged tree before it lands. A row that differs is `c1`'s
  re-pin, regenerated with the table's own command, never merged by hand.

## Gate

Run these in order, each through the pool. Every command builds one named target, and `cargo` is
the slot shim. Bounds:

- 60 s per test, 120 s per test step;
- a pool job ends after 10 minutes unless it names its time, and `SLOT_TIMEOUT=30m` is the most;
- no `RUSTC_WRAPPER`, `SCCACHE_*` or `-j`.

```console
slot task fmt-check
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-conformance --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo clippy -p ess-cli --all-targets --locked -- -D warnings
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --lib -- synthesize::caller witness_memo
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test caller_readings_made_once_work --test caller_arranged_rows --test a_synthesis_seed_is_accepted_for_a_caller_model --test a_seeded_scenario_notes_its_skipped_reversed_caller_run --test caller_values --test interpreted_caller --test one_time_caller --test caller_swap_related_tie_written_from_input --test mutation_grants
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table --test synthesis_suite_bytes_table_cli_read --test adversary_related_values_all_bytes --test external_beside_held_guard
SLOT_TIMEOUT=30m CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-cli --locked --test tutorial_page
```

- `synthesis_suite_bytes_table` is the byte proof. It runs as it is today (routing decision X2);
  `t2`, which changes it, lands after every `p` unit. Report its time.
- If `c1` landed before this unit, rerun the three model-table targets on the merged tree before
  landing (routing decision Y4): `synthesis_suite_bytes_table` above, plus
  `SLOT_TIMEOUT=30m cargo test -p ess-compiler --locked --test selection_precedence_table` and
  `SLOT_TIMEOUT=30m cargo test -p ess-entity-runtime --locked --test lowered_definitions_table`.
  A row that differs is `c1`'s re-pin: regenerate it with the command the failing table prints,
  never merge it by hand.
- `ess-synth`'s and `ess-openapi`'s byte pins compare generated trees, not synthesis. This unit
  does not touch them.

## Acceptance

1. `the_plan_lists_each_reading_once_in_the_order_the_merge_reads_them` and
   `a_reading_on_a_uniform_model_names_that_model_as_its_lane` pass.
2. The three existing `caller::tests` counts (2, 4, 2 + 2·2) pass unchanged.
3. `synthesis_suite_bytes_table` (both tests), `synthesis_suite_bytes_table_cli_read`,
   `adversary_related_values_all_bytes` and `external_beside_held_guard` pass with no table row
   changed: `git diff $BASE -- crates/verify/ess-conformance/tests/fixtures` is empty.
4. Every caller-themed target in `## Gate` passes unchanged.
5. `git grep -n 'std::rc::Rc<EssIr>\|Rc<EssIr>' crates/verify/ess-conformance/src` prints
   nothing.
6. This unit starts no thread. The check names both the file and the directory, because a
   pathspec `synthesize/caller` alone leaves out `synthesize/caller.rs`:
   `git grep -nE 'thread::|\bspawn' -- crates/verify/ess-conformance/src/synthesize/caller.rs crates/verify/ess-conformance/src/synthesize/caller/`
   prints nothing. It prints nothing at `80794bbd698` too. `thread_local!` (the test counters at
   `caller.rs` 3532) does not match `thread::`, and `\b` keeps out a `respawn`. The `Rc<EssIr>`
   sites of `caller.rs` (358, 391-412) are held by acceptance 5, whose path is all of `src/`.
7. Clippy is clean on `ess-conformance` and `ess-cli`, and `task fmt-check` passes.

## Decisions

- D1. The cut is the whole reading (`synthesize_invocations` of one rewritten model under one
  focus), never an obligation. That is the parent's fit review, point 4.
- D2. `one_row_acted_on` and the rotation re-run (`turned`) stay inline at merge time. Both
  depend on a merge result. They are rare: neither runs on the largest known specification's
  profile (84 `about_command` runs for 43 commands, none turned).
- D3. A reading on a uniform model has that model as its **lane**: `Whole`, every `Cross`, and
  every `About` of a command outside `rewritten`, or of the only command in it. That is the
  parent's D2 C extended to stage 1. The parent's design says the `about` models "are not
  uniform, so they are not kept", but `Assigned::model` (391-413) gives a command outside
  `rewritten` the uniform model, and its focused runs reuse the whole reading's memory. That
  reuse is what `caller_readings_made_once_work.rs` bounds (`TIMES = 4`).
- D4. Results are held until merged, in place of being dropped one command at a time. The extra
  memory is the focused results (a few scenarios each) and is accepted. `p5` measures peak RSS.
- D5. `Kept` is `!Send` through `PhantomData<*const ()>`, not through a comment. A Keeping moved
  to another thread would pop the wrong stack. The workspace forbids `unsafe_code`
  (`Cargo.toml` `[workspace.lints.rust]`), and no `unsafe` is needed.
- D6. `Arc`, not borrowed `&EssIr` with a lifetime, for the uniform models. `Keeping` must hold
  the model so its address cannot be reused while memory about it is parked (`witness_memo.rs`
  119-121). An `Arc` does that across threads.
- D7. No design page in this unit. `p2` writes `docs/design/parallel-caller-readings.md`,
  including this plan.

## Changelog

none (no behaviour changes; `p4` writes the story's one bullet).
