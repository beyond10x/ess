---
format: aep.planning-md/3
id: approval-record:d136-synthesis-defects-and-store-drift
kind: approval-record
status: approved
title: 'The #554 synthesis defects become stories; batch 1 fixes the store drift (D136)'
summary: No GitHub issues for the two defects
relations:
- decides: story:a-subject-guard-scenario-arranges-the-rows-its-setup-reads
- decides: story:an-unknown-identity-beside-a-related-guard-is-synthesized
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T10:43:09Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Decision

D136, taken by the orchestrator under the operator's grant on 2026-10-28: "file the two #554
synthesis defects as AEP stories in the store (not GitHub issues) and put them in a batch; fix the
store drift items in batch 1."

## Effect

- `story:a-subject-guard-scenario-arranges-the-rows-its-setup-reads` (SealIncident `already-sealed`
  fails against the reference implementation) and
  `story:an-unknown-identity-beside-a-related-guard-is-synthesized` (LinkAlert `unknown-incident`
  refused with ESS-SYNTH-008) decompose `epic:generic-guard-precedence`. No GitHub issue is filed;
  #554 stays the visible umbrella.
- Batch 1 also fixes the store drift the triage found: stories still gated on `ess/31`, ADR bodies
  that say "proposed" under status `accepted`, and #671 without a story of its own.

Decider: operator
