---
format: aep.planning-md/3
id: approval-record:g-u34-diff-line-in-0700
kind: approval-record
status: approved
title: G-U34 lands without its diff acceptance line
summary: The diff classification of an added when_subject guard moves to story:diff-decides-an-added-subject-guard for 0.70.0.
relations:
- decides: story:guard-tests-list-membership
- decides: story:diff-decides-an-added-subject-guard
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-23T15:43:39Z", actor: "human:timo", revision: 2}
---
## Decision

Relayed by the orchestrator on 2026-10-23: "approved: drop \"diff calls the added guard breaking\" from G-U34; story diff-decides-an-added-subject-guard stays draft for 0.70.0. Reason: keeps 0.69.0 (ess#633, on loom 0.40.0's path) out of ess-diff scope."

## Effect

`story:guard-tests-list-membership` lands in 0.69.0 without its diff acceptance line. `story:diff-decides-an-added-subject-guard` stays draft, tagged ess-0.70.0.

Approved by session b10x-chef-orchestrator under its grant.
