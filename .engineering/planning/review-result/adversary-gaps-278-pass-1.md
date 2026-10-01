---
format: aep.planning-md/3
id: review-result:adversary-gaps-278-pass-1
kind: review-result
status: active
title: Adversary pass 1, downstream gaps unit feature-request-278
relations:
- reviews: story:feature-request-278
revision: 1
---
unit: story:feature-request-278, tree gaps-278
verdict: NEEDS-CHANGE, red 2 (introduced 1, pre-existing 1)

- warning (subject_fact.rs:2041): the twin filter compared guard text, so `result in [Healthy]` beside `result == Healthy` still rendered `c and none of: c'`.
- warning (subject_fact.rs:2057): with an equal twin declared earlier, the refusal blamed a missing row.
- note (subject_fact.rs:1941): two third-pass conditions were implied wherever it ran.
- held: every new scenario passes a hand-written first-declared target and fails last-declared and Unique mutants.

Correction 1: semantic guard comparison (admit_alike), the earlier twin named as the cause, dead conditions turned into debug_assert.
Tests: tests/adversary_278_pass1.rs (8).
