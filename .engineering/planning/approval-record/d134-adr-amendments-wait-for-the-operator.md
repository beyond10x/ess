---
format: aep.planning-md/3
id: approval-record:d134-adr-amendments-wait-for-the-operator
kind: approval-record
status: approved
title: 'ADR amendments for #650 and #674 wait for the operator (D134)'
summary: Two decision-blockers; batches proceed without them
relations:
- informed_by: decision-blocker:websocket-upgrade-binding-form
- informed_by: decision-blocker:a-view-without-a-source
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T10:43:06Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Decision

D134, taken by the orchestrator under the operator's grant on 2026-10-28: "#650 (WebSocket half) and
#674: both need an ADR amendment, which is the operator's. File a decision-blocker for each in the ESS store
with the options table from the plan, the recommendation (#650 A, #674 A staged), and a body line
`Decider: operator`. Batches proceed without them; those units wait."

## Effect

- `decision-blocker:websocket-upgrade-binding-form` blocks the WebSocket half of
  `story:http-binding-carries-transport-framing` (recommendation A, a sibling `ess-websocket/1`).
- `decision-blocker:a-view-without-a-source` blocks `story:a-view-may-answer-values-no-row-holds`
  (recommendation A, staged).
- No batch waits for either.

Decider: operator
