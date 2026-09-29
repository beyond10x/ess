---
format: aep.planning-md/3
id: review-result:adversary-a-arrangement-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.41 unit arrangement
relations:
- reviews: story:arrangement-searches-every-creating-command
- reviews: story:input-refusals-arrange-the-record-they-need
- reviews: story:arrangement-threads-distinction-through-reach-state
revision: 1
---
unit: arrangement (#198, #209, #199)
verdict: red
cases: executed 1431→1444, red 1
origin: introduced 2, pre-existing 0, undecided 1
wrote-outside-worktree: ~/.cache/ess-wave-n2/arrangement/adv2/
needs-coordinator: yes

Adversary pass 2 (`aep:adversary`), 2026-09-28, head 38544d6b2 + `tests/adversary_arrangement_pass2.rs` (12 passed, 1 red). Held: 46 models, no refusal witness triggers the wrong refusal, no lost shape; generate --check; 41 inputs compared, only unit fixtures and explore-external-exits differ (both correct).

```findings
[{"file":"crates/verify/ess-conformance/src/synthesize/existence.rs","line":336,"category":"acceptance","severity":"note","verdict":"INFEASIBLE","origin":"undecided","message":"with no identity view the arranged half requires only error and no event, so a target silently moving the record on refusal passes although the command own wrong-state branch would reveal it"},{"file":"docs/design/input-guard-overlap-precedence.md","line":35,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"the table says a refusal witness refutes sibling refusals only without a default, which the correction made false"},{"file":"docs/design/outcome-shapes.md","line":280,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"the arranged half is said to require the row unchanged unconditionally, but only where an identity view shows the row"}]
```

Trend: pass 1 → 4, pass 2 → 3 notes, carried 0. Coordinator routing: the no-view gap is filed as story:a-no-view-arranged-half-probes-the-row-through-the-command (the red case is rewritten to assert today with a message naming it, outcome no-op); the two doc lines fixed in correction 2 (outcome fixed).
