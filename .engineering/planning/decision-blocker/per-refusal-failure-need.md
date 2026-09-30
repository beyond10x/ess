---
format: aep.planning-md/3
id: decision-blocker:per-refusal-failure-need
kind: decision-blocker
status: open
title: Do direct callers still need the benign refusal?
relations:
- blocks: story:feature-request-269
withholds: test_result
revision: 1
---
## Question

For the downstream binding that should escalate one refusal and drop another: must direct callers of the bound command still see a refusal in the benign case? If not, the existing idiom (an accepting branch with `when_subject_state: [..]` and `preserves:`) covers it and no failure policy change is needed.

## Who answers

The downstream specification's authors, asked through beyond10x/ess#269.
