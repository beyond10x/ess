---
format: aep.planning-md/3
id: decision-blocker:ess-24-schedule
kind: decision-blocker
status: cleared
title: 'When does source format ess/24 ship with #498 and #500?'
relations:
- blocks: release-plan:ess-24-one-language
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-08T08:49:45Z", actor: "human:timo", revision: 3}
---
## Question

#498 and #500 need source format `ess/24` (`decision-blocker:ess-058-format-24`), which had no release. When do they ship?

| option | does | costs |
|---|---|---|
| A | `ess/24` with #498 and #500 ships as 0.59.0, after 0.58.0 | a source format bump; ess-http/1 stage 2 moves to 0.60.0 |
| B | keep them unscheduled | both issues stay open |
| C | close both, naming the stories; reopen at `ess/24` | adopters follow the store, not the issues |

## Decided

A: `ess/24` with #498 and #500 ships as 0.59.0, after 0.58.0. ess-http/1 stage 2 moves to 0.60.0, and #493 stays open until then. The `ess/24` plan is settled or split during the 0.58.0 work so 0.59.0 starts right after it.
