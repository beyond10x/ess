---
format: aep.planning-md/3
id: approval-record:d242-713-in-batch-5
kind: approval-record
status: approved
title: '#713 joins batch 5 (D242)'
summary: ess generate ui --http (#713) joins batch 5 (0.80.0) as a sixth unit; an adopter is blocked on it
relations:
- decides: story:a-generated-ui-calls-the-routes-its-http-binding-serves
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-30T00:27:58Z", actor: "human:timo", revision: 3, executor: "agent:ess-issues-lead", correlation: "ess-issues-5"}
---
# #713 joins batch 5 (D242)

## Decision

On 2026-10-30 the orchestrator approved D242 under the operator's grant: file the missing
`ess generate ui --http` and place it in the earliest batch not yet gated after batch 4.

- The gap was filed as beyond10x/ess#713 through the bot, with a toy reproduction checked on
  `ess 0.77.0`.
- `story:a-generated-ui-calls-the-routes-its-http-binding-serves` joins batch 5 (`0.80.0`) as a
  sixth unit, beside the React renderer stories. A downstream adopter is blocked on it today.
- The option is opt-in, so the output without `--http` keeps its bytes (D135).

Decider: operator
