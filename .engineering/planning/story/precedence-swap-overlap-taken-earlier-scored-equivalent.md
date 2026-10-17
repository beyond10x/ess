---
format: aep.planning-md/3
id: story:precedence-swap-overlap-taken-earlier-scored-equivalent
kind: story
status: draft
title: A precedence swap whose overlap earlier branches take is scored equivalent
revision: 1
---
## Outcome

`ess verify conform mutate` scores a `precedence-swap` of two branches as `equivalent` when every
input both branches select is already taken by input-guarded branches declared before both, and
names those branches in the reason, instead of scoring it `survived`.

Closes https://github.com/beyond10x/ess/issues/539.

## Evidence

- https://github.com/beyond10x/ess/issues/539 (ess 0.59.0): branches `done-paused-default`
  (`pause: true` and `code` absent or `""`), `done-paused` (`pause: true`) and `done-default`
  (`code` absent or `""`), in that order. Swapping `done-paused` and `done-default` changes
  nothing: their overlap is exactly what `done-paused-default` takes. It is scored survived.
- Still on `origin/main` (260a3d1e3, read 2026-10-13): `unsatisfiable_guard`
  (`crates/verify/ess-conformance/src/mutate.rs:2362-2373`) scores a swap equivalent only when
  `All([one, other])` is unsatisfiable; branches declared earlier are not subtracted.
  `identical_answer` (`mutate.rs:2444`) covers the other equivalent case
  (`story:identical-answer-precedence-swap-scored-equivalent`, implemented, https://github.com/beyond10x/ess/issues/517).
  `satisfiable` and `decide` (`mutate.rs:2650-2700`) already decide a conjunction over the
  command's input domain, so `All([one, other, Not(Any(earlier))])` is in reach.

## Acceptance

Each line names the failing test written first.

- `crates/verify/ess-conformance/tests/mutation_identical_answer.rs` (or a new
  `mutation_covered_overlap.rs`), `a_swap_whose_overlap_an_earlier_branch_takes_is_equivalent`:
  the issue's three branches; the `done-paused`/`done-default` swap is `equivalent` with a reason
  naming `done-paused-default`.
- Same file, `a_swap_whose_overlap_is_partly_uncovered_still_survives_or_is_killed`: drop
  `code == ""` from the earlier branch's guard; the swap keeps the verdict its run gives.
- Same file, `an_earlier_branch_guarded_by_stored_state_does_not_cover`: an earlier branch with a
  `when_subject:` or `when_related:` guard is not subtracted (only input guards are decided here,
  as `unsatisfiable_guard` already limits itself, `mutate.rs:2400-2404`).
- The manifest records the reason. If it reuses `unsatisfiable_guard` with a new meaning
  ("overlap left after earlier branches"), the manifest format moves and an old-reader test in
  `mutation_identical_answer.rs` says so; a new field is the alternative. The implementor picks one
  and states it in the PR and CHANGELOG.

## Scope

- cited: `crates/verify/ess-conformance/src/mutate.rs` (`unsatisfiable_guard`, `guard_of`,
  `satisfiable`); `crates/verify/ess-conformance/tests/mutation_identical_answer.rs`.
- inferred: the manifest and report writers in `mutate.rs` (`:3089-3110`, `:4509-4540`,
  `:4600-4710`) if a field is added; `docs/design/` page for mutation equivalence, if one names the
  rule; CHANGELOG `[Unreleased]` "Fixed".

## Plan

- depends_on: none. Shares no file with the explorer stories or with
  `story:read-only-command-answers-an-unknown-instance`.
- du estimate: 8G (ess-conformance one `--test` at a time).
- adversary: yes (an `equivalent` verdict that is wrong hides a real survivor).
