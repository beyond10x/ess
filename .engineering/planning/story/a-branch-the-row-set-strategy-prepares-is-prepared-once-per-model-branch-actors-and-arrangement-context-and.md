---
format: aep.planning-md/3
id: story:a-branch-the-row-set-strategy-prepares-is-prepared-once-per-model-branch-actors-and-arrangement-context-and
kind: story
status: draft
title: A branch the row-set strategy prepares is prepared once per model, branch, actors and arrangement context, and every later request reads the shared result.
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-row-set-branch-s-search-arranges-the-rows-of-a-selector-once-per-distinct-prefix-of-counts-not-once-per
- depends_on: story:a-row-set-branch-s-search-does-not-arrange-a-combination-of-counts-that-provably-cannot-take-the-branch-and
revision: 1
---
# pf3-prepare-memo

Size: M (one memo module after `run_memo.rs`, its key, a key-completeness test, a differential
test).

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
fresh, for the differential test.

The inputs `prepare` reads (from the code, tag 0.98.0 and the head; the implementor re-reads
before writing the key and adds any found):

- arguments: `models: &InvocationModels` — `arrangement`, `acting`, `credentials` (private: model,
  callers, who), `seeds`; `command`; `outcome` (by position in `command.outcomes`, as `run_memo`
  does); `actors`;
- the thread context `synthesize/threads.rs` `Context::capture` already lists for caller
  readings: `scope`, `dropped_write`, `dropped_grant`, `added_grant`, `phase_order`,
  `created_unkept`, `runs_unkept`, `first_fresh`, `attempts_fresh` (`row_set::attempts_fresh`,
  read by `prepare` itself), `companion_fresh`;
- the arrangement context `run_memo`'s key adds: `subject_fact::held_key` (rows held by
  `subject_fact::holding`), `earlier_runs::captured`, `witness::under_test_key`,
  `related_guard::open_key` (open beside and own searches);
- not inputs (they answer the same whatever their content, or only count): the memories of
  `made.rs`, `created_memo.rs`, `model_memo.rs`, `run_memo.rs`, `witness_memo.rs`, the overlap
  answers in `synthesize.rs`, and every work counter.

`run_memo`'s own key omits the `Context` fields and the `InvocationModels` beyond `ir`; that is
sound in its scope (value invariants) and is not reused unchanged here.

## Acceptance

1. Red today: a new test in `row_set_many_selectors_work.rs` synthesizes a generic model in which
   one row-set branch is prepared more than once (the `Order`/`Line`/`Approve` model of pf1 plus a
   command whose existence scenario sends `Approve`, so `existence.rs` prepares `Approve/approved`
   again; the implementor confirms the repeat with the new counter) and asserts, through a new
   doc-hidden counter `row_set_preparations(run) -> (R, (asked, made))`, that `made` equals the
   number of distinct branches. Today `made == asked`.
2. Green: that test passes; entries are `Rc`-shared (no clone of `Prepared` on a hit, shown by
   the type of the stored entry).
3. Key completeness, by construction: the key is built by one function that destructures
   `InvocationModels` and `threads::Context` without `..`, so a new field fails to compile until
   the key takes it or the function says why not.
4. Key completeness, by test: a source test in `ess-conformance` lists every `thread_local!`
   static under `src/` (about 50 today: `synthesize.rs` 8, `made.rs` 5, `witness.rs` 5,
   `subject_fact.rs` 4, …) with one class each — keyed, in `Context`, memory that answers the
   same, work counter, outside synthesis — and fails naming any static not in the list. A new
   input therefore cannot be read without a reviewer classifying it.
5. Differential: synthesizing each committed fixture model the row-set targets use, and the
   generic model at K = 2..4, gives byte-identical suites with the memo on and off (the
   `unkept` switch).
6. The byte pins of `## Gate` pass unchanged; on a release build the generic model's suite is
   `cmp`-identical to the head's for K = 2..5.

## Scope

- Cited: `run_memo.rs` (pattern: per-model stack, key string, `Entry { run, cuts }`, `unkept`,
  `ASKED`/`MADE` counters with `taken_*`/`add_*` for `threads.rs`).
- Cited: `threads.rs` `Context` (line 172) and its counter list (carry the two new counters).
- Inferred, check first: that `prepare` reads no other thread-local than those listed (acceptance
  4 makes this a checked list); that `Prepared` (`Setup`, input map, gaps) holds no per-call
  identity a caller mutates in place after a hit (if one does, the caller clones its own copy
  there, not the memo).

## Owned files

- `crates/verify/ess-conformance/src/synthesize/prepare_memo.rs` (new)
- `crates/verify/ess-conformance/src/synthesize/row_set.rs` (`prepare` only: route through the
  memo)
- `crates/verify/ess-conformance/src/synthesize.rs` (`mod prepare_memo;`, the remember guard where
  `run_memo::remember` is taken, the doc-hidden counter)
- `crates/verify/ess-conformance/src/synthesize/threads.rs` (counter transfer)
- `crates/verify/ess-conformance/tests/row_set_many_selectors_work.rs`
- `crates/verify/ess-conformance/tests/thread_local_inputs.rs` (new source test, acceptance 4;
  or a module of an existing source-reading target if the crate has one: prefer that)
- `CHANGELOG.md` (one bullet)

## Not yours

`Plan::arrange` (pf1), the enumeration in `prepare_with` (pf2), `run_memo.rs`, `made.rs`.

## Tests first

Acceptance 1, 4 and 5; run them red, keep the lines.

## Gate

As pf1's `## Gate`, plus `cargo test -p ess-conformance --locked --test thread_local_inputs`
(or the target chosen), `--test row_set_upsert`, `--test mutation_row_sets` (mutation audit
synthesizes under dropped writes and grants: the key must separate them), and acceptance 6's
release builds and `cmp`.

## Depends on

pf1 and pf2 (same file; this touches only `prepare`). Lands last. pf2 does not make it
unnecessary for the reported shape: there the attempts are few and expensive and the repeats
hold 69 % of the time; pf2's rule does not apply to most of its selectors (their entities move).

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
- D2. Completeness check. A (default): both the destructuring constructor and the thread-local
  classification test. B: the constructor only (misses a thread-local read).
- D3. Scope of the memory. A (default): the same per-model scope as `run_memo::remember`, taken
  in the main synthesis too. B: only within one caller reading (smaller gain, smaller risk).
