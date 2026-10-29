---
format: aep.planning-md/3
id: approval-record:d229-u20-in-batch-4-and-705-in-batch-7
kind: approval-record
status: approved
title: '#706 is U20 in batch 4; #705 goes to batch 7 (D229)'
summary: Batch 4 (0.79.0) takes the trigger-branch row predicate as a fifth unit U20; the subject-payload set guard goes to batch 7
relations:
- decides: story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding
- decides: story:a-set-guard-fed-by-a-subject-payload-is-arranged
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-29T21:58:40Z", actor: "human:timo", revision: 3, executor: "agent:ess-issues-lead", correlation: "ess-issues-4"}
---
# #706 is U20 in batch 4; #705 goes to batch 7 (D229)

## Decision

On 2026-10-29 the orchestrator approved D229 under the operator's grant. It accepts the placement
proposed for the two conformance-synthesis issues filed that day.

- beyond10x/ess#706 (a defect): `story:a-row-predicate-choosing-the-trigger-branch-is-arranged-for-the-binding`
  is unit U20 of batch 4 (`0.79.0`). Batch 4 holds five units, under the D186 allowance; U19 stays.
- beyond10x/ess#705: `story:a-set-guard-fed-by-a-subject-payload-is-arranged` goes into batch 7.

The orchestrator is told the tag when `0.79.0` ships U20 and when batch 7 ships the #705 story.

Decider: operator
