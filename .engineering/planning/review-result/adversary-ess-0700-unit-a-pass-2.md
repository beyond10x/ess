---
format: aep.planning-md/3
id: review-result:adversary-ess-0700-unit-a-pass-2
kind: review-result
status: active
title: Adversary pass 2, unit A (identity guard beside a row set)
relations:
- reviews: story:an-identity-guard-beside-a-row-set-validates
revision: 1
---
## Adversary pass 2, unit A (ess-0700 wave)

Worktree ess-0700-a, base d19ce4314, implementor's correction round 1 uncommitted; adversary added
`tests/adversary_ess0700a_identity_beside_row_set_2.rs` under ess-conformance (11 cases, 7 red) and
ess-domain (1 case). Method: a target wrapping the interpreter answers `Connect` with the first
refusal that holds in a given order, run for every order of the step-4 refusals; a suite should
fail exactly the orders that answer some reachable request differently from the rule.

Held: `when_subject` named at every row-set format; a row set over the sink; a row set over a
missing named row in the reproduction's order; a separate creator of the selected rows;
determinism over 5 variants; suite bytes of the other models; no ordering check lost from the old
suite.

Coordinator disposition (two passes spent): findings 1 and 2 go to a final correction round the
coordinator verifies; findings 3 to 6 are deferred to
`story:row-set-beside-named-rows-suite-tells-every-order-apart`, and the docs claim is narrowed.

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize/related_guard.rs","line":1125,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"a row-set refusal declared above a named-row refusal is never witnessed beside it, so a target reading the row set last passes"},
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","line":200,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"count gte 2 overlap scenarios hold one selected row, so a target reading the row set before sink-not-input or no-sink passes"},
{"file":"crates/verify/ess-conformance/src/synthesize/related_guard.rs","line":688,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"a when: beside a named-row refusal makes synthesis refuse the row-set branch and leaves 4 reorderings uncaught"},
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","line":3580,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"a forall-guarded accepting branch beside named rows leaves its default refusal unwitnessed; the same forall at ess/31 is witnessed"},
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","line":3580,"category":"property","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"in the forall model beside named rows, a target refusing on any wire from the source passes the whole suite"},
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","category":"property","severity":"note","verdict":"CONFIRMED","origin":"undecided","message":"the accepting forall scenario arranges no row satisfying the forall at ess/31 either, so the any-wire-refuses target passes there too"}
]
```
