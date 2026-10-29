---
format: aep.planning-md/3
id: approval-record:d133-role-input-drift-keeps-the-rule
kind: approval-record
status: approved
title: role_input_drift keeps its rule; the reporter is asked which side sends (D133)
summary: 'Option A now for #666; B or C follows the reporter answer'
refs:
- provider: github
  reference: beyond10x/ess#666
relations:
- decides: story:role-input-drift-tells-a-caller-from-an-implementation
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T10:43:05Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Decision

D133, taken by the orchestrator under the operator's grant on 2026-10-28: "D133 #666: option A now
(keep the rule, document it, pin with a fixture); post the reporter question in the issue thread
through the App (neutral wording, no customer or company names); B or C follows the reporter's
answer."

Option A of `docs/plan/wave-ess-issues-1.md` § Decisions for the operator, #666: `role_input_drift` on a
`stands: implements` entry whose local command lacks contract inputs stays. The design page
`composition-contract-roles.md` says that a caller of a contract command is not a role entry (use
`gates` or no entry), and a fixture pins
`an_implementation_missing_a_contract_input_drifts`, including a missing `Optional` contract input.

## What follows

The question to the reporter (which side sends the subset, and whether the fields are `Optional` in
the contract) is posted in the issue thread. Its answer picks the follow-up:

- B: a `stands: calls` role, at the next `ess-composition/N` (after #664 and #671 take `/5`).
- C: `implements` tolerates a missing `Optional` contract input behind `reader: true`.

Until then the story keeps option A's scope only.

Decider: operator
