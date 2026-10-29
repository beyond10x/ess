---
format: aep.planning-md/3
id: approval-record:d138-schema-invariants-as-scoped
kind: approval-record
status: approved
title: Schema invariants unit as scoped (D138)
summary: '#685 #686 #687 #688; U1, U6, U7 sequenced on realize.rs'
refs:
- provider: github
  reference: beyond10x/ess#685
- provider: github
  reference: beyond10x/ess#686
- provider: github
  reference: beyond10x/ess#687
- provider: github
  reference: beyond10x/ess#688
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T10:43:11Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Decision

D138, taken by the orchestrator under the operator's grant on 2026-10-28, on the U1 scoping: "#688
accept as proposed (x-ess-invariants + integer bounds via types::message; moves committed generated/
entity and OpenAPI files). #687 accept redesigned (Binary64 bounds as numeric keywords only for a
literal that is the shortest spelling of its binary64 value). #685 accept redesigned (.count on
String -> minLength/maxLength; list counts deferred to their own story). #686 accept redesigned
(distinct: -> uniqueItems or one bounded contains per enum/Boolean variant, with type: object +
required; open key stays annotation; realize.rs change for model input only). U1 shares realize.rs
with U6 and U7: sequence them. D138: take U1 as scoped; apply D135 if moving the committed generated/
files changes what a consumer sees."

## Effect

U1 (schema-invariants) runs as scoped; U1, U6 and U7 merge in that order because they share
`realize.rs`. If the moved `generated/` files change what a consumer sees, D135 applies.

Decider: operator
