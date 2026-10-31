---
format: aep.planning-md/3
id: approval-record:d232-707-is-u21-in-batch-4
kind: approval-record
status: approved
title: '#707 is U21 in batch 4 (D232)'
summary: The generated Rust server's loose request decoding (#707) joins batch 4 (0.79.0) as a sixth unit U21
relations:
- decides: story:the-generated-rust-server-refuses-what-its-openapi-refuses
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-29T23:01:29Z", actor: "human:timo", revision: 3, executor: "agent:ess-issues-lead", correlation: "ess-issues-4"}
---
# #707 is U21 in batch 4 (D232)

## Decision

On 2026-10-29 the orchestrator approved D232 under the operator's grant: file the loose request
decoding of the generated Rust server and place it the way #705 and #706 were placed.

- Filed as beyond10x/ess#707 through the bot, with a toy reproduction checked on `ess 0.77.0`.
- `story:the-generated-rust-server-refuses-what-its-openapi-refuses` is a defect, in which the server
  is looser than the contract it publishes. It goes to the earliest batch not yet gated: batch 4
  (`0.79.0`), as unit U21, its sixth unit, over the cap as D229 allowed for U20.

Decider: operator
