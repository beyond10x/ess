---
format: aep.planning-md/3
id: approval-record:adr-unblocked-units-in-0700
kind: approval-record
status: approved
title: Units unblocked by the 2026-10-23 ADR acceptance go to 0.70.0
summary: G-U3, G-U9, G-U22, G-U28, G-U36 and the N31 keep-versus-clear half are 0.70.0 work; 0.69.0 ships its current queue.
tags:
- ess-0.70.0
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-23T19:37:07Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
---
## Decision

Relayed by the orchestrator on 2026-10-23: "G-U3, G-U9, G-U22, G-U28, G-U36 and the N31 keep-vs-clear half go to 0.70.0, not the 0.69.0 queue. 0.69.0 ships with its current queue (G-U30, G-U19 and what was already in it). Reason: ess 0.69.0 gates loom 0.40.0 and the ADR 0098 row of the S9 closing audit."

These units were held back until the operator accepted the ADRs they rest on; all 15 proposed ADRs were accepted the same day ("I accept all of them"). Acceptance makes them buildable, and this record places them in 0.70.0.

## Effect

0.69.0 (PR #633) keeps its queue. The six items above are planned for 0.70.0.

Approved by session b10x-chef-orchestrator under its grant.
