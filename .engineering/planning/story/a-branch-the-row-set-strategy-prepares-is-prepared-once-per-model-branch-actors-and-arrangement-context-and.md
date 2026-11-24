---
format: aep.planning-md/3
id: story:a-branch-the-row-set-strategy-prepares-is-prepared-once-per-model-branch-actors-and-arrangement-context-and
kind: story
status: active
title: A branch the row-set strategy prepares is prepared once per model, branch, actors and arrangement context, and every later request reads the shared result.
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-row-set-branch-s-search-arranges-the-rows-of-a-selector-once-per-distinct-prefix-of-counts-not-once-per
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T01:51:04Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T01:51:06Z", actor: "human:timo", revision: 4}
---
# pf3-prepare-memo

Size: M (one memo module after `run_memo.rs`, its key, an argument struct for `prepare`, an
extension of `threads.rs` `STATE`, a work test, differential tests).

## Need

`row_set::prepare` (`row_set.rs` line 3233 on `integration/gaps-2026-11-24`) is called for the
same branch again and again: from `synthesize.rs` (line 5782, the outcome family), `existence.rs`
(line 2293) and `binding_effects/set_guard.rs` (line 192), and again per caller reading and per
driver arrangement. On the reporter's specification (tag 0.98.0, instrumented, pf1's reuse on):
487 `prepare` calls, of which 384 re-prepare a branch already prepared; those hold 470 of the
683 s spent in `prepare` (69 %). One branch of the dominant command was prepared 3 times at
37-57 s each. Every repeat redoes the whole search (`prepare_with` variants x count combinations
x rows).

## Outcome

`prepare`'s result (`Result<Prepared, RefusalCause>`) and the beside searches it cut are kept per
model, as `run_memo.rs` keeps branch runs, under a key built from every input `prepare` reads, and
a repeat request is answered from the kept entry: shared (`Rc`), not copied, the cut count added
again. Nothing a search decides changes. A switch like `run_memo::unkept` makes every request
fresh, for the differential tests.

The inputs `prepare` reads (from the code, tag 0.98.0 and the head; the implementor re-reads
before writing the key and adds any found):

- arguments: `prepare` reads only `models.arrangement` of `InvocationModels` (`caller.rs`
  line 180; `acting`, `credentials`, `seeds` unread, confirmed by the checker). Its four arguments
  travel in one struct, `Preparing { arrangement, command, outcome, actors }`: the three callers
  pass `&models.arrangement`, not the whole `InvocationModels`, so a later read of another model
  needs a new field. `outcome` is keyed by position in `command.outcomes`, as `run_memo` does;
- the thread context `synthesize/threads.rs` `Context::capture` (line 172) already lists for
  caller readings: `scope`, `dropped_write`, `dropped_grant`, `added_grant`, `phase_order`,
  `created_unkept`, `runs_unkept`, `first_fresh`, `attempts_fresh` (`row_set::attempts_fresh`,
  read by `prepare` itself), `companion_fresh`;
- the arrangement context `run_memo`'s key adds: `subject_fact::held_key` (rows held by
  `subject_fact::holding`), `earlier_runs::captured`, `witness::under_test_key`,
  `related_guard::open_key` (open beside and own searches);
- thread-locals of other crates that synthesis reads (`ess-domain`: 10 `thread_local!` blocks in
  `domain.rs`, `system.rs` (2), `types.rs` (2), `command.rs` (2), `command/precedence.rs`
  (`PHASE_ORDER`, used by `answers_before`; `Context` carries it), `command/row_set.rs`,
  `expression/attributes.rs`; `ess-primitives`: `facts.rs`, `predicate.rs`), each classified in
  acceptance 4;
- not inputs (they answer the same whatever their content, or only count): the memories of
  `made.rs`, `created_memo.rs`, `model_memo.rs`, `run_memo.rs`, `witness_memo.rs`, the overlap
  answers in `synthesize.rs`, and every work counter.

`run_memo`'s own key omits the `Context` fields; that is sound in its scope (value invariants) and
is not reused unchanged here.

## Acceptance

1. Red today: a new test in `row_set_many_selectors_work.rs` synthesizes, under `with_jobs(1)`
   (the memos are per thread, so `made` depends on `ESS_SYNTH_JOBS`), a generic model in which one
   row-set branch is prepared more than once from the same context: the `Order`/`Line`/`Approve`
   model of pf1 with `Approve`'s own existence scenario, so `existence.rs` (`on_rows`, lines
   2284-2293) prepares `Approve`'s creating branch beside the outcome family; the implementor
   confirms the repeat with the new counter. A new doc-hidden counter
   `row_set_preparations(run) -> (R, Preparations { asked, made, keys })` returns the requests,
   the searches made and the distinct keys. The test asserts `made < asked` and `made ==
   keys.len()`. Where one branch is made twice, the test's message names the key part that differs
   (the counter keeps the keys); the key is never narrowed to make it pass. Today `made == asked`.
2. Green: that test passes; entries are `Rc`-shared (no clone of `Prepared` on a hit, shown by
   the type of the stored entry).
3. Key completeness, by construction: one function builds the key and destructures `Preparing`
   and `threads::Context` without `..`, so a new argument or context field fails to compile until
   the key takes it or the function says why not.
4. Key completeness, by test: `threads.rs` `STATE` (line 53) gains a column naming, for every
   `Scope` and `Context` row, the part of the prepare key that carries it (or why it needs none);
   `every_static_of_the_crate_is_classified` (line 514), which already scans every `static` of the
   crate (plain, atomic, `OnceLock`, `thread_local!`), fails on such a row without one. Beside it
   a second list classifies the `ess-domain` and `ess-primitives` thread-locals named above, each
   with the key part that carries it or why synthesis does not change it during a run, and a test
   in the same module scans those two crates' `src/` for `thread_local!` and `static` declarations
   and fails naming any not listed. No separate source-test target.
5. Differential: one `#[test]` per committed fixture model the row-set targets use and per K of
   the generic model (K = 2, 3; K = 4 only if its pair of debug runs is under 60 s), each
   synthesizing with the memo on and off (the `unkept` switch) and comparing the suites byte for
   byte. Each test under 60 s; every one is timed in the report, one over 5 s named with its time.
6. The byte pins of `## Gate` pass unchanged; on a release build the generic model's suite is
   `cmp`-identical to the head's for K = 2..5.

## Scope

- Cited: `run_memo.rs` (pattern: per-model stack, key string, `Entry { run, cuts }`, `unkept`,
  `ASKED`/`MADE` counters with `taken_*`/`add_*` for `threads.rs`).
- Cited: `threads.rs` `Context` (line 172), `STATE` (line 53) and its two tests (lines 514 and
  after), its counter list (carry the new counters, each with a `Counter` row and a `Work` field).
- Cited: the three callers of `prepare` (signature change to `Preparing`).
- Inferred, check first: that `Prepared` (`Setup`, input map, gaps) holds no per-call identity a
  caller mutates in place after a hit (if one does, the caller clones its own copy there, not the
  memo); that none of the listed `ess-domain` thread-locals changes inside one synthesis except
  through what `Context` carries.

## Owned files

- `crates/verify/ess-conformance/src/synthesize/prepare_memo.rs` (new)
- `crates/verify/ess-conformance/src/synthesize/row_set.rs` (`prepare` only: `Preparing`, route
  through the memo)
- `crates/verify/ess-conformance/src/synthesize.rs` (`mod prepare_memo;`, the remember guard where
  `run_memo::remember` is taken, the doc-hidden counter, the call at line 5782)
- `crates/verify/ess-conformance/src/synthesize/existence.rs` (the call at line 2293 only)
- `crates/verify/ess-conformance/src/synthesize/binding_effects/set_guard.rs` (the call at line
  192 only)
- `crates/verify/ess-conformance/src/synthesize/threads.rs` (`STATE` key column, the second list,
  their tests, counter transfer)
- `crates/verify/ess-conformance/tests/row_set_many_selectors_work.rs`
- `CHANGELOG.md` (one bullet)

## Not yours

`Plan::arrange` (pf1), the enumeration in `prepare_with` (pf2), `run_memo.rs`, `made.rs`, every
file under `ess-domain` and `ess-primitives` (listed, not changed).

## Tests first

Acceptance 1, 4 and 5; run them red, keep the lines. Each test under 60 s (the runner kills it at
60 s); the test step under 120 s.

## Gate

As pf1's `## Gate`, plus `cargo test -p ess-conformance --locked --lib synthesize::threads`
(acceptance 4), `--test row_set_upsert`, `--test mutation_row_sets` (mutation audit synthesizes
under dropped writes and grants: the key must separate them), and acceptance 6's release builds
and `cmp`.

## Depends on

pf1 and pf2 (same file; this touches only `prepare` and its callers). Lands last. pf2 does not
make it unnecessary for the reported shape: there the attempts are few and expensive and the
repeats hold 69 % of the time; pf2's rule does not apply to most of its selectors (their entities
move).

## Changelog

`### Fixed`: "Synthesis prepares each branch a row-set guard decides once per arrangement
context, where it prepared it again for every scenario family and driver that asked."

## Fit review

The finding and the growth table are pf1's (a downstream application, 2026-11-24: validation and
synthesis time exponential in the row-set selectors of one command; generic model K = 1..6:
0.03, 0.10, 0.54, 3.1, 25.0, 146.6 s user on 0.98.0). Estimated effect on the reported
specification, after pf1: 714 s wall towards about 250 s (the repeats' 470 s, less what a
correct key separates); not measured.

## Decisions

- D1. Key form. A (default): a string key like `run_memo`'s, built by the destructuring function
  of acceptance 3. B: a typed key (`Ord` struct); more code, no string formatting per request.
- D2. Completeness check. A (default): the destructuring constructor and the extended `STATE`
  with its cross-crate list. B: the constructor only (misses a thread-local read).
- D3. Scope of the memory. A (default): the same per-model scope as `run_memo::remember`, taken
  in the main synthesis too. B: only within one caller reading (smaller gain, smaller risk).
