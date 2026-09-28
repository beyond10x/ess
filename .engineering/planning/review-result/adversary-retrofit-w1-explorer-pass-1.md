---
format: aep.planning-md/3
id: review-result:adversary-retrofit-w1-explorer-pass-1
kind: review-result
status: active
title: 'Adversary pass 1: an external branch from a state the ordinary branch does not act from is dropped'
relations:
- reviews: story:explorer-takes-external-branches
revision: 1
---
Adversary pass 1 against story:explorer-takes-external-branches (beyond10x/ess#156), aep:adversary, 2026-09-27.

verdict: red
cases: executed 649→651, red 1
origin: introduced 1, pre-existing 0, undecided 0

Case `an_external_branch_from_a_state_the_ordinary_branch_does_not_act_from_is_reached`
(`crates/verify/ess-conformance/tests/explore_external_adversary.rs`): `Close` has an ordinary
branch moving from `Open` and an `external:` branch moving from `Held`. Both languages report
`Close/overridden` unreached; the TypeScript result shows
`"ambiguous":["exploreadv.desk.Close: closed, wrong-state"]`.

Finding: `crates/verify/ess-conformance/src/go/explore.go:963-972` and `src/ts/explore.ts` ~702-709 —
`exploreDecide` returns `ambiguous` when the ordinary branch cannot move from the subject's state
and discards the eligible external branch it computed, so the branch is reported unreached and
`AssertExplored` fails a correct target even with `AllowExcluded`. Suggested fix: return `take`
with no ordinary outcome and only the externals, as the no-`otherwise` path does.

Held: removing the "arrangements may stay in force" rule makes the `arranges` double disagree in
both languages (mutant caught); outputs keep their bytes without external branches; Go and
TypeScript pick at the same point; unarrangeable branches stay out of `unreached`; replay
re-arranges external steps.

```findings
[{"file": "crates/verify/ess-conformance/src/go/explore.go", "line": 963, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "When the ordinary branch cannot move from the subject's state, exploreDecide (and decide in explore.ts) returns ambiguous and drops an eligible external branch, so a correct target fails the explorer's assertion."}]
```
