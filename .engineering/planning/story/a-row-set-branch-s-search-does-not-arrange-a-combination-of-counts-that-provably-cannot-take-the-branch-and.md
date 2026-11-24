---
format: aep.planning-md/3
id: story:a-row-set-branch-s-search-does-not-arrange-a-combination-of-counts-that-provably-cannot-take-the-branch-and
kind: story
status: draft
title: A row-set branch's search does not arrange a combination of counts that provably cannot take the branch, and still writes the suite the full search writes.
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-row-set-branch-s-search-arranges-the-rows-of-a-selector-once-per-distinct-prefix-of-counts-not-once-per
revision: 1
---
# pf2-skip-impossible-counts

Size: L (a design page, a rule with its static preconditions, an exact fallback, a work counter
and its test).

## Need

`row_set::prepare_with` (`row_set.rs`, the `search` closure after `let unstandable = …`, line 3797
on `integration/gaps-2026-11-24`) tries the combinations of `COUNTS = [1, 2, 0, 3]` per selector,
first selector slowest, until one stands, cut at `ARRANGEMENTS = 4096`. Only a failure to arrange a
selector's rows prunes (`agreed`); a combination whose rows `consistent` rejects moves to the next
count of the LAST selector. So for a branch preceded by K' row-set refusals `exists: true`, every
completion of every prefix where one of those selectors has count 1 or 2 is arranged and rejected:
about 2·(4^(K'-1) + … + 4 + 1) attempts before the first combination where they all are 0. This is
the 4^K law of the fit review; pf1 only cheapens each attempt.

## Outcome

Rule R1 (established by reading the code; its soundness argument is the design page's): in the
search for branch `b` of command `c`, let `r` be a branch of `c` that
`precedence::answers_before(format, c, b)` lists, guarded `RelatedSet { selection: s, test:
exists: true, input: None }`, and let selector index `i` be `s`'s. If every precondition P1-P6
holds, no combination with `counts[i] > 0` can stand, because:

- `arrange_rows` takes a selected row only where `truth(row, goal) == Truth::True`, `goal =
  All[filter_i, foralls_i]` (non-head path), and only where `follows` accepts it;
- the rows a later step leaves of `s`'s entity are those rows unchanged (P1), so at the end of the
  steps the row is still a member of `s` (`Reading::members` over the same `supplied`), and
  `ExistsTrue.decide` is `True` whenever one member is `True`, whatever the other rows;
- `consistent` then returns `Err("`r` is not decidedly passed over: its row set is True")`
  (`before.contains(r) && taken != False`), and `Unknown` from opaque later steps fails the same
  way.

Preconditions (each checked statically, per `Plan`, before the search):

- P1. No declared command has an outcome that acts on an existing row of `s.entity` (moves,
  updates, `instances:` set effect) and no binding writes it (`bound_writes`): its rows never
  change after creation.
- P2. `s.reads` is empty and `filter_i` holds no related-row test (`related_rows::tests` empty):
  the filter does not depend on rows arranged later.
- P3. `filter_i` reads no `subject.` path whose entity fails P1 (the subject cannot move under the
  filter).
- P4. `plan.heads[i]`, `plan.links[i]` and `plan.owners[i]` are `None`, and `foralls_i` is empty
  (an `exists` test has no predicate, so the `violate` variants never name `i`).
- P5. Not `pins_identity(ir, s.entity, filter_i)` with `identity_selectors(ir)`: there a decoy may
  carry the selected row's identity (`one_record`), so the selected row is not a separate record.
- P6. `r`'s guard has `input: None` (an input guard can make `r` False whatever the rows).

The skip: when the enumeration reaches a combination with `counts[i] > 0` for such an `i`, it
does not arrange it and moves on as if every combination agreeing with `counts[0..=i]` had been
tried and rejected (the next count of `i`, every later selector's first).

Byte identity (both are acceptance items):

- The first failure. `first_failure` keeps only the first failure (`get_or_insert`), and a skipped
  combination's failure in the full search could be a `walk` error or a `consistent` error of
  another branch, not R1's. So no combination is skipped until `first_failure` is set: until then
  every combination is arranged as today. Afterwards a skip cannot change it.
- The cap. The full search may prune inside a skipped subtree (`agreed`), so how many attempts it
  would have spent there is not known without arranging them; counting skipped tries exactly is not
  possible. Instead the search keeps an upper bound `U`: attempts made plus, per skipped subtree,
  the product of the later selectors' count lists times the variants (every combination, no
  pruning). Where the skipping search ends (found, or exhausted) with `U < ARRANGEMENTS`, the full
  search cannot have been cut before that point, and it reaches the same combination, or the same
  exhaustion with the same first failure: identical. Where `U` reaches `ARRANGEMENTS`, the branch is
  searched again by the full enumeration of today (fallback), so a branch the full search cut is
  still cut, with today's text.

Growth on the generic model by this reading (not measured): for branch `line-fj`, selectors 1 to
j-1 meet R1 (Line has no update, no move, no binding; filters read `input.order_id` and a flag);
each costs one arranged attempt before `first_failure` is set and then is skipped at counts 1 and 2,
so a branch needs on the order of 2·K + 4 arranged attempts instead of about 2·4^(K-1). `U` for
the last refusal at K = 6 is about 2·(4^5 + 4^4 + 4^3 + 4^2 + 4) + 4 = 2,732 < 4,096: no fallback.
At K = 7 it is about 10,924: the fallback runs and the time is today's (cut at 4,096 attempts).
Target K = 6 under 5 s user: supported by the reading (with pf1, about 1.6 ms per attempt on the
tag at K = 5 and a few hundred attempts in all), to be measured. K >= 7 is not helped by this rule.

For the reported specification the rule helps little: its dominant command makes few attempts
(402 over 24 preparations), most of its selectors read entities with moves (P1 fails), and its time
is per attempt and in repeated preparation (pf3).

## Acceptance

1. A design page `docs/design/row-set-count-pruning.md`: rule R1, P1-P6 with the code each relies
   on, the first-failure and cap arguments above, the fallback, and what it does not cover
   (`exists: false` refusals answered before: count 0 does not make their set empty, rows of another
   selector or of a creator chain may be selected, so no rule; `forall` guards; heads).
2. Red today: a new test in `row_set_many_selectors_work.rs` synthesizes the generic model at
   K = 5 (decision D1) under `row_set_arrangements_tried` (it counts `Plan::arrange` attempts,
   so a skipped combination is not counted) and asserts at most 1,000 attempts for the whole
   synthesis (decision D2). On tag 0.98.0 the K = 5 model made 6,793 (instrumented run; all
   branches, all `prepare_with` calls); the implementor records the head's figure in the red run.
3. Green: the same test passes; the generic model's suite and `validate --format json` are
   `cmp`-identical to the unchanged head's for K = 2, 3, 4, 5, 6 (release builds; K = 6 unchanged
   takes about 150 s user, run it once in the background, without `slot`).
4. A test of the fallback: a model whose search the full enumeration cuts at 4,096 (the generic
   model at K = 7, or `row_set_many_selectors_work.rs`'s existing twelve-selector model if its
   branches reach the cut) keeps its refusal text byte for byte; under 60 s, else name its time.
5. A test where a precondition fails (P1: give `Line` an `updates:` branch) shows no skip: the
   attempts equal the unchanged head's.
6. The byte pins of `## Gate` pass unchanged.

## Scope

- Cited: the `search` closure in `prepare_with`, `consistent` (line 1156), `Reading::takes`
  (line 1018), `arrange_rows` (selected rows: `accept = truth(row, &goal) == True && fresh`),
  `precedence::answers_before` (`synthesize/precedence.rs` line 32), `unstandable` (line 5004).
- Inferred, check first: that `takes` for `exists: true` is `True` as soon as one member is
  `True` (read: `test.decide(&members)`); that `walk`'s `apply` changes an existing row only through
  the sent command's subject effect, a set effect or a binding (P1 covers all three).

## Owned files

- `docs/design/row-set-count-pruning.md` (new)
- `crates/verify/ess-conformance/src/synthesize/row_set.rs` (the `search` closure and a new
  `fn impossible_counts` or similar beside it; not `Plan::arrange`)
- `crates/verify/ess-conformance/tests/row_set_many_selectors_work.rs`
- `CHANGELOG.md` (one bullet)

## Not yours

`Plan::arrange` and its levels (pf1), `prepare` (pf3), `consistent` (read only), `precedence.rs`.

## Tests first

Acceptance 2, 4 and 5 as tests in `row_set_many_selectors_work.rs` (the generic model of pf1,
`Order`/`Line`/`Approve`, K a parameter); run them red and keep the lines.

## Gate

As pf1's `## Gate`, plus `cargo test -p ess-conformance --locked --test row_set_upsert` and
`--test coalesce_selectors`, and acceptance 3's release builds and `cmp`.

## Depends on

pf1 (same file; pf1 changes `Plan::arrange`, this the enumeration in `prepare_with` that calls
it). pf1 lands first; this rebases on it. The design page can be written and reviewed in
parallel.

## Changelog

`### Fixed`: "Synthesis of a command refusing on several row sets no longer arranges the row
counts that cannot take the branch it witnesses, so its time no longer grows fourfold with each
row-set refusal answered before that branch."

## Fit review

The finding and the growth table are pf1's (a downstream application, 2026-11-24: validation and
synthesis time exponential in the row-set selectors of one command; generic model K = 1..6:
0.03, 0.10, 0.54, 3.1, 25.0, 146.6 s user on 0.98.0). This story removes the 4^K attempts where
R1 applies; it is the only one of the three that changes the growth law.

## Decisions

- D1. Model size of the work test. A (default): K = 5 (6,793 attempts today, seconds in release,
  pruned to tens). B: K = 4 if the debug run of the unchanged code exceeds 60 s.
- D2. The bound. A (default): at most 1,000 attempts over the whole K = 5 synthesis (reading:
  about 2·K + 4 per `prepare_with` call, 7 branches, a handful of calls each: several hundred).
  B: a per-branch bound, which needs a second counter scoped to one branch (more API).
- D3. The cap. A (default): upper bound `U` with exact fallback (byte-identical everywhere). B:
  count only arranged attempts against the cap; faster for K >= 7 but a branch the full search
  cut may now get a scenario or another refusal (bytes move; needs a `### Changed` bullet and the
  owner's word).
