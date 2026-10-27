---
format: aep.planning-md/3
id: story:an-affects-row-is-arranged-past-a-creators-emitting-not-found-branch
kind: story
status: implemented
title: An affects row is arranged past a creator's emitting not-found branch
summary: 'Adopter batch 2d N: affects over Ticket is refused when the creator''s exists false branch emits an event instead of naming an error'
tags:
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T21:52:54Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-26T21:52:55Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-27T11:23:16Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:b10x-chef-orchestrator-ess-0710"}
---
## Problem

Adopter review batch 2d, item N (repro `repro-r3b`, ess/25; reproduced on 0.69.2 and on the 0.70.0 integration head). `ResetMember/reset` deletes `Ticket` rows with `affects: [{entity: Ticket, where: member_id == input.member_id}]`. The only `Ticket` creator is `OpenTicket/opened`; before it a not-found branch guarded by `when_related: {via: input.member_id, exists: false}` emits an event (`demo.w.Missed`) instead of naming an error. Synthesis refuses `reset` as ESS-SYNTH-001: "`reset.affects[0]` is `demo.w.Ticket` and created only by `demo.w.OpenTicket/opened`, which no arrangement takes for a row the filter selects" (3/1). With `error: demo.w.MemberNotFound` instead of `emits:` it gives 4/0 (`repro-r3b-fixed`); deleting the branch also clears it. In the same run synthesis arranges `OpenTicket/opened` (creates a Member, opens a Ticket), so the row is arrangeable. The adopter keeps the emitting branch on purpose: every caller gets an accepted answer.

## Acceptance

- `repro-r3b` synthesizes `ResetMember/reset` with no refusal, and the suite passes the interpreter.
- Whether an `affects:` row is arranged does not depend on whether the creator's `exists: false` branch emits or names an error; `repro-r3b-fixed` keeps its suite bytes.
