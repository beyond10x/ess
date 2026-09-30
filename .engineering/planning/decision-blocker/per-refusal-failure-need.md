---
format: aep.planning-md/3
id: decision-blocker:per-refusal-failure-need
kind: decision-blocker
status: cleared
title: Do direct callers still need the benign refusal?
relations:
- blocks: story:feature-request-269
withholds: test_result
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-09-30T13:33:58Z", actor: "human:timo", revision: 3}
---
## Question

For the downstream binding that should escalate one refusal and drop another: must direct callers of the bound command still see a refusal in the benign case? If not, the existing idiom (an accepting branch with `when_subject_state: [..]` and `preserves:`) covers it and no failure policy change is needed.

## Who answers

The downstream specification's authors, asked through beyond10x/ess#269.

Answered by the downstream on beyond10x/ess#269 (2026-09-30): direct callers must still see the refusal (a control plane sending the bound command in the wrong state must get `wrong_state`); only the binding's own delivery should treat it as benign. So the accepting-no-op idiom does not cover it, and a per-refusal failure policy is needed.
