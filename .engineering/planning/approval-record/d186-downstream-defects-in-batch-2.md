---
format: aep.planning-md/3
id: approval-record:d186-downstream-defects-in-batch-2
kind: approval-record
status: approved
title: Two downstream defects join batch 2 as U9 and U10 (D186)
summary: Filed as ess#702 and ess#703; U10 falls back to a warning if the adopter replay finds one refused
relations:
- decides: story:precedence-swap-of-disjoint-guards-is-equivalent
- decides: story:validate-refuses-a-branch-implied-by-an-earlier-input-refusal
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-29T07:28:21Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
---
## Decision

The orchestrator approved on 2026-10-29, under the operator's grant, as D186: two defects reported
by a downstream consumer are filed as issues and delivered in batch 2 as units U9 and U10, so they
ship in the batch 2 release. Batch 2 then holds six units, over the four-unit cap; the orchestrator
takes that for speed.

- U9, beyond10x/ess#703: mutate precedence-swap scores disjoint input guards `unwitnessed` where
  an input is Optional or a conjunct is undecidable.
- U10, beyond10x/ess#702: validate accepts a `when:` branch whose guard implies an earlier
  input-guarded refusal's guard.
- U10 is a new refusal. The adopter replay must show no adopter refused; if one is, U10 ships as a
  warning in this release and the refusal waits until the adopters are fixed.
- The consumer's specification is private: issues, tests, commits and release notes use toy
  specifications and name no consumer.

Decider: operator
