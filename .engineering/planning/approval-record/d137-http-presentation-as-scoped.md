---
format: aep.planning-md/3
id: approval-record:d137-http-presentation-as-scoped
kind: approval-record
status: approved
title: Operation ids, servers and the unreached-component note as scoped (D137)
summary: '#653 additive in ess-http/1; #651 one description sentence, routes stay'
refs:
- provider: github
  reference: beyond10x/ess#651
- provider: github
  reference: beyond10x/ess#653
relations:
- decides: story:an-http-binding-names-operation-ids-and-servers
- decides: story:a-component-not-reached-over-the-network-has-no-http-route
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T10:43:10Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Decision

D137, taken by the orchestrator under the operator's grant on 2026-10-28, on the U4 scoping: "#653:
accept, additive per-route operation_id + servers keyed by network component (ESS-HTTP-032/033 in
ess-http/1); U5 takes 034+. #651: accept as one sentence in OpenAPI info.description for a
non-network-reached component; routes stay (removing them breaks the tutorial and the ESS-HTTP-013
pins; removal would be an operator decision-blocker, not needed). D137: take both as scoped."

## Effect

U4 (http-presentation) implements both as scoped, additively within `ess-http/1`; U5 numbers its
codes from ESS-HTTP-034.

Decider: operator
