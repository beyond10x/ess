---
format: aep.planning-md/3
id: review-result:adversary-a-arrangement-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.41 unit arrangement
relations:
- reviews: story:arrangement-searches-every-creating-command
- reviews: story:input-refusals-arrange-the-record-they-need
- reviews: story:arrangement-threads-distinction-through-reach-state
revision: 1
---
unit: arrangement (#198, #209, #199)
verdict: red
cases: executed 1417→1431, red 5
origin: introduced 2, pre-existing 2, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-n2/arrangement/adv1/
needs-coordinator: no

Adversary pass 1 (`aep:adversary`), 2026-09-28, head da217e68d + `tests/adversary_arrangement_pass1.rs`. Held: four mutant targets caught by the arranged half; three creators, guarded creator, owner recursion; byte-identical synthesis twice; 38 inputs byte-identical head vs base except the unit fixtures.

```findings
[{"file":"crates/verify/ess-conformance/src/synthesize/existence.rs","line":289,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"with no identity view the input-refusal scenario is withdrawn because only preservation failed, although the arrangement reaches the record, losing the plain send base filed"},{"file":"crates/verify/ess-conformance/src/synthesize/subject_fact.rs","line":1110,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"search tries creations in command-name order (BTreeMap), not the declaration order the decision and its doc comment state"},{"file":"crates/verify/ess-conformance/src/synthesize.rs","line":2812,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"arrange route tie goes to the first creation by name while its comment says first declared"},{"file":"crates/verify/ess-conformance/src/synthesize.rs","line":3652,"category":"property","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"a refusal witness beside a default also satisfies an overlapping sibling refusal, so the suite demands an outcome the model leaves open, and the arranged half sends it again"}]
```

Coordinator routing: #1 fix (keep the plain send; file the arranged half without the preservation assertion when no view observes the row); #2/#3 decision revised to name order (the IR keeps commands by name; the doc comments and the decision say name order; no bytes change); #4 fix in-unit (a refusal witness refutes every overlapping sibling input refusal).
