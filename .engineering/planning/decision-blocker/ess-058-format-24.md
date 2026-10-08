---
format: aep.planning-md/3
id: decision-blocker:ess-058-format-24
kind: decision-blocker
status: cleared
title: 'Does 0.58.0 ship source format ess/24 for #498 and #500?'
relations:
- blocks: story:stored-field-equals-returned-response-value
- blocks: story:response-and-struct-admit-undeclared-fields-when-declared-ignored
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-08T02:07:06Z", actor: "human:timo", revision: 3}
---
## Question

#498 and #500 add authored keys and need source format `ess/24`, whose scope is `release-plan:ess-24-one-language` (unscheduled). Does 0.58.0 ship `ess/24` with them, or do both move to `release-plan:ess-24-one-language`?

| option | does | costs |
|---|---|---|
| A | 0.58.0 ships `ess/24` with #498 and #500 | a format bump in 0.58.0; the rest of the ess/24 scope must be settled or split first |
| B | #498 and #500 move to `release-plan:ess-24-one-language`; 0.58.0 ships #496 #499 #501 and the #497 answer | two issues stay open until ess/24 is scheduled |

## Decided

B: #498 and #500 ship with source format `ess/24` and move to `release-plan:ess-24-one-language`; 0.58.0 ships #496, #499, #501 and the #497 answer.
