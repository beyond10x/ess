---
format: aep.planning-md/3
id: approval-record:ess-0750-local-replay
kind: approval-record
status: approved
title: Tag 0.75.0 after a local replay while the adopter session is unreachable
summary: 'D140: local replay of every neutral reproduction against 0.74.0 stands in for the adopter replay; a later regression ships as 0.75.1.'
tags:
- ess-0.75.0
- non-interactive
relations:
- decides: approval-record:ess-0750-wave-1
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T11:38:07Z", actor: "human:timo", revision: 2, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Decision

Relayed by the orchestrator on 2026-10-28 (decision D140): the adopter session is unreachable, so
0.75.0 tags at green CI on 6bed257d or its green successor after a local replay that stands in for
the adopter one. The base is the 0.74.0 binary. It runs on every neutral adopter reproduction and
compares scenario counts and refusal codes. Any count or code that moves beyond the listed
rewordings is a stop. The full adopter replay runs against 0.75.0 when the adopter session is
back; a regression it finds ships as 0.75.1.

## Result

23 runs, no count, code or interpreted result moved. The only text change is the seeded gap R
refusal, and that suite equals the base once refusal messages are removed.
