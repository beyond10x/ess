---
format: aep.planning-md/3
id: story:a-row-set-branch-s-search-does-not-arrange-a-combination-of-counts-that-provably-cannot-take-the-branch-and
kind: story
status: draft
title: A row-set branch's search does not arrange a combination of counts that provably cannot take the branch, and still writes the suite the full search writes.
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-row-set-branch-s-search-arranges-the-rows-of-a-selector-once-per-distinct-prefix-of-counts-not-once-per
revision: 3
---
# pf2-skip-impossible-counts

Size: L (a design page, a rule with eight static preconditions, an exact fallback that restarts
the whole search, a work counter test and a fallback test).

## Need

`row_set::prepare_with` (`row_set.rs`, the `search` closure after `let unstandable = …`, line 3797
on `integration/gaps-2026-11-24`) tries the combinations of `COUNTS = [1, 2, 0, 3]` per selector,
first selector slowest, until one stands, cut at `ARRANGEMENTS = 4096`. Only a failure to arrange a
selector's rows prunes (`agreed`); a combination `consistent` rejects moves to the next count of
the LAST selector. So for a branch preceded by K' row-set refusals `exists: true`, every
completion of every prefix where one of those selectors has count 1 or 2 is arranged and rejected:
about 2·(4^(K'-1) + … + 4 + 1) attempts before the first combination where all of them are 0.
This is the 4^K law of the fit review; pf1 only cheapens each attempt.

## Outcome

Rule R1 (established by reading the code; the design page carries its soundness argument): in the
search for branch `b` of command `c`, let `r` be a branch of `c` that
`precedence::answers_before(format, c, b)` (`synthesize/precedence.rs` line 32) lists, guarded
`RelatedSet { selection: s, test: exists: true, input: None }`, and let `i` be `s`'s selector
index in the `Plan`. If P1-P8 all hold, no combination with `counts[i] > 0` can stand:

- `arrange_rows` takes a selected row only where `truth(row, goal) == Truth::True`, `goal =
  All[filter_i, foralls_i]`, and only where `follows` accepts it;
- by P1 and P7 the walk (`Reading::truth`, lines 930-988, over the values `apply` recomputes,
  lines 1583-1595) computes the same values the arrangement settled, so at the end of the steps
  the row is still a member of `s`; `ExistsTrue.decide` (`Reading::takes`, line 1018) is `True` as
  soon as one member is `True`, whatever the other rows;
- `consistent` (line 1156) then returns `Err("`r` is not decidedly passed over: its row set is
  True")` (`before.contains(r) && taken != False`); `Unknown` from opaque later steps fails the
  same way.

Preconditions, each checked statically per `Plan` before the search (else no skip in that
`prepare_with` call):

- P1. No declared command has an outcome that acts on an existing row of `s.entity` (moves,
  updates, `instances:` set effect) and no binding writes it (`bound_writes`).
- P2. `s.reads` is empty and `filter_i` holds no related-row test (`related_rows::tests` empty).
- P3. `filter_i` reads no `subject.` path whose entity fails P1.
- P4. `plan.heads` is `None` for EVERY selector of the plan (not only `i`): `crowded` (set at
  line 2173 inside an attempt of any selector reading a head) decides the reason after the search
  (lines 3906-3913), so a skipped attempt could change it. `plan.owners[i]` is `None` and
  `foralls_i` is empty. `plan.links[i]` is `None`, or the link conjunct compares a field of `s`
  with the input the send binds to the linked row: `arrange_row` then takes only rows naming that
  row (`linked.names(row) && accept(row)`, line 2402; `Linked::names`, line 4128), so the conjunct
  holds in the walk too (see "What still qualifies").
- P5. Not `pins_identity(ir, s.entity, filter_i)` with `identity_selectors(ir)` (`one_record`: a
  decoy may carry the selected row's identity).
- P6. `r`'s guard has `input: None`.
- P7. Every field `filter_i` reads (after the link conjunct is taken out) is set by every creating
  outcome of `s.entity` from its input or a literal: no `copied:` value, `related:` read, row-set
  read, head read or published value (checker F2: otherwise the arranged truth and the walked
  truth can differ, and the full search can stand at `counts[i] = 1`).
- P8. No attempt can cut a search for a row beside or an own search: `related_guard::open_key()`
  is empty when `prepare_with` starts, and the graph command -> entity of a row it reads beside ->
  that entity's creators is acyclic from the creators of every selected entity (checked
  statically; argued on the design page). Attempts add to `related_guard::cuts`, which decides
  output (`related_guard.rs` line 3123), and a skipped attempt adds nothing.

The skip: when the enumeration reaches a combination with `counts[i] > 0` for such an `i`, it
does not arrange it and moves on as if every combination agreeing with `counts[0..=i]` had been
tried and rejected (the next count of `i`, every later selector's first).

Byte identity:

- First failure. `first_failure` keeps only the first failure (`get_or_insert`), and a skipped
  combination's failure could have been a `walk` error or another branch's. So no combination is
  skipped until `first_failure` is set; until then every combination is arranged as today.
- Cap. The full search may prune inside a skipped subtree (`agreed`), so its attempts there are
  not countable without arranging them. The search keeps an upper bound `U`: `U` starts at 0 with
  `tried` and carries across both `search` calls (`search(&HEAD_COUNTS)` and `search(&[1])`), as
  `tried` does; it adds every attempt made and, per skipped subtree, the product of the later
  selectors' count lists times the variants. Where the skipping search ends with `U <
  ARRANGEMENTS`, the full search cannot have been cut before that point and ends in the same
  combination, or the same exhaustion with the same first failure. Where `U` reaches
  `ARRANGEMENTS`, the fallback reruns the WHOLE search of `prepare_with` with skipping off, from
  `tried = 0`, `first_failure = None` and a new `Plan` (so `crowded` and pf1's levels start empty):
  today's search, today's bytes.

## What still qualifies, and the expected gain

- Generic model (`Order`/`Line`/`Approve`, K refusals `exists: true` over `Line` where
  `order_id == input.order_id` and a flag): `Line` has no move, update or binding (P1); its filter
  reads `order_id` and a Boolean flag, both set by `AddLine` from its input (P7); no heads, no
  owner, no input guard, identity not pinned; `AddLine` reads `Order` beside, `Order`'s creator
  reads nothing beside (P8 acyclic). The attempts that stand are the linked ones (instrumented on
  tag 0.98.0: `link=true` stands, `link=false` fails), where `links[i]` is `Some` with the conjunct
  `order_id == input.order_id`: they qualify only under P4's link clause above. So: all K
  selectors qualify with that clause; with a strict "`links[i]` is `None`", the rule never fires
  in a standing attempt of the generic model and there is no gain at K = 6.
- With the link clause: a branch `line-fj` needs on the order of 2·K + 4 arranged attempts per
  `prepare_with` call instead of about 2·4^(K-1); `U` for the last refusal at K = 6 is about
  2·(4^5 + 4^4 + 4^3 + 4^2 + 4) + 4 = 2,732 < 4,096, so no fallback. Target K = 6 under 5 s user:
  supported by the reading (with pf1 about 1.6 ms per attempt at K = 5 on the tag), not measured.
  K >= 7: `U` about 10,924, the fallback runs, today's time.
- Reported shape (in generic terms): 37 refusals `exists: true` over a row set; only 6 of them read
  an entity whose rows never change (P1 alone; bindings not counted, so at most 6), none of them on
  the command that holds 94 % of the time (both its selectors read entities with moves). This rule
  does not reach the reported time; pf3 does (repeated preparation) and pf1 (work per attempt).

## Acceptance

1. A design page `docs/design/row-set-count-pruning.md`: R1, P1-P8 with the code each relies on,
   the link clause of P4 and its argument, the first-failure and cap arguments, the fallback that
   restarts the whole search, and what it does not cover (`exists: false` refusals answered before:
   count 0 does not make their set empty; `forall` guards; heads; moving entities).
2. Red today: a new test in `row_set_many_selectors_work.rs` synthesizes the generic model at
   K = 5 (D1) under `row_set_arrangements_tried` (it counts `Plan::arrange` attempts, so a skipped
   combination is not counted) and asserts at most 1,000 attempts for the whole synthesis (D2). On
   tag 0.98.0 the K = 5 model made 6,793; the implementor records the head's figure in the red run.
3. Green: that test passes; the generic model's suite and `validate --format json` are
   `cmp`-identical to the unchanged head's for K = 2, 3, 4, 5, 6 (release builds; K = 6 unchanged
   takes about 150 s user, run once in the background, without `slot`).
4. Fallback, on a cheap model where `U` reaches 4,096 at the first skip while the full search
   stays small: the R1 selector first, then six selectors over an entity no command creates (as
   the checkout of the twelve-selector model in the same file), so the full search is pruned at
   once. The test asserts that the fallback ran (a doc-hidden counter of fallbacks) and that the
   suite and refusal text equal the head's. Under 60 s.
5. No skip where a precondition fails: the generic model at K = 3 (308 attempts on the tag) with
   `Line` given an `updates:` branch (P1) makes as many attempts as the unchanged head.
6. The byte pins of `## Gate` pass unchanged.

## Scope

- Cited: the `search` closure and its two calls in `prepare_with` (lines 3797-3913), `consistent`
  (1156), `Reading::takes` (1018), `Reading::truth` (930-988), `apply` (1583-1595), `arrange_rows`,
  `arrange_row` (2402), `Linked::names` (4128), `unstandable` (5004), `related_guard.rs` 3123.
- Inferred, check first: that `supply` binds the link's input to the linked row's instance in
  every linked attempt (the link clause of P4 rests on it); that `walk`'s `apply` changes an
  existing row only through the sent command's subject effect, a set effect or a binding (P1).

## Owned files

- `docs/design/row-set-count-pruning.md` (new)
- `crates/verify/ess-conformance/src/synthesize/row_set.rs` (the `search` closure, the restart in
  `prepare_with`, a new check function beside it; not `Plan::arrange`)
- `crates/verify/ess-conformance/src/synthesize.rs` (the doc-hidden fallback counter)
- `crates/verify/ess-conformance/src/synthesize/threads.rs` (its `STATE` row and `Work` field)
- `crates/verify/ess-conformance/tests/row_set_many_selectors_work.rs`
- `CHANGELOG.md` (one bullet)

## Not yours

`Plan::arrange` and its levels (pf1), `prepare` (pf3), `consistent` (read only), `precedence.rs`.

## Tests first

Acceptance 2, 4 and 5 as tests in `row_set_many_selectors_work.rs`; run them red and keep the
lines. Each under 60 s; one over 5 s is named with its time and load.

## Gate

As pf1's `## Gate`, plus `cargo test -p ess-conformance --locked --test row_set_upsert`,
`--test coalesce_selectors`, `--test nested_ordered_heads` (heads, P4), and acceptance 3's release
builds and `cmp`.

## Depends on

pf1 (same file: pf1 changes `Plan::arrange`, this the enumeration that calls it and the restart
that makes a new `Plan`). pf1 lands first; this rebases on it. The design page can be written and
reviewed in parallel.

## Changelog

`### Fixed`: "Synthesis of a command refusing on several row sets no longer arranges the row
counts that cannot take the branch it witnesses, so its time no longer grows fourfold with each
row-set refusal answered before that branch."

## Fit review

The finding and the growth table are pf1's (a downstream application, 2026-11-24: validation and
synthesis time exponential in the row-set selectors of one command; generic model K = 1..6:
0.03, 0.10, 0.54, 3.1, 25.0, 146.6 s user on 0.98.0). This story is the only one of the three that
changes the growth law, and only where R1's preconditions hold (generic model: all selectors under
P4's link clause; reported shape: at most 6 of 37 refusals).

## Decisions

- D1. Model size of the work test. A (default): K = 5 (6,793 attempts on the tag). B: K = 4 if the
  debug run of the unchanged code exceeds 60 s.
- D2. The bound. A (default): at most 1,000 attempts over the whole K = 5 synthesis (reading: about
  2·K + 4 per `prepare_with` call, 7 branches, a handful of calls each). B: a per-branch bound,
  which needs a counter scoped to one branch.
- D3. The cap. A (default): upper bound `U` across both searches with a restart of the whole search
  (byte-identical everywhere). B: count only arranged attempts against the cap; faster for K >= 7,
  but a branch the full search cut may get a scenario or another refusal (bytes move; `### Changed`
  bullet and the owner's word).
- D4. Linked selectors. A (default): P4's link clause (without it the rule never fires on the
  generic model's standing attempts). B: strict `links[i] == None`; simpler proof, no gain on the
  reported growth shape; the story then only documents the rule.
