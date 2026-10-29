---
format: aep.planning-md/3
id: approval-record:d237-712-moves-to-batch-5
kind: approval-record
status: approved
title: '#712 moves to batch 5 (D237)'
summary: The grant/denied synthesize defect (#712) moves from batch 7 to batch 5 (0.80.0), the earliest batch not yet gated with room
relations:
- decides: story:grant-denied-is-synthesized-beside-a-ranked-view
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-29T23:26:05Z", actor: "human:timo", revision: 3, executor: "agent:ess-issues-lead", correlation: "ess-issues-5"}
---
# #712 moves to batch 5 (D237)

## Decision

On 2026-10-30 the orchestrator approved D237 under the operator's grant.
`story:grant-denied-is-synthesized-beside-a-ranked-view` (#712) is a synthesize defect: it drops
`grant/denied` in every suite, not only in the mutation audit. A defect takes the earliest batch not
yet gated. Batch 4 is full at six units, so the story moves from batch 7 to batch 5 (`0.80.0`).
#708 to #711 stay in batch 7.

Decider: operator
