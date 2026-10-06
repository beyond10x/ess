---
format: aep.planning-md/3
id: story:outcome-groups
kind: story
status: implemented
title: One outcome can be declared for a group of commands
refs:
- provider: github
  reference: beyond10x/ess#105
relations:
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-26T02:40:29Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-26T02:41:00Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:37Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

One outcome can be declared for a group of commands.

## Why

GitHub issue beyond10x/ess#105; its Observed and Expected sections are the contract and are not restated here.

## Acceptance

- Every expectation in beyond10x/ess#105 holds, each with a red-first test.
- Where the issue offers alternatives, the design page or unit report names the one taken.

## Delivery reconciliation 2026-10-02

Source audit identifies delivered behavior in 05e995c06 under ess/12, included in released 0.51.0. Domain expansion covers explicit members, actor/domain selection and collisions; external refusals remain declared errors only. Domain tests, compiler IR equivalence and conformance exact-suite equivalence provide current regression coverage. N expanded command members still produce N command scenarios by design; this does not promise a smaller execution baseline. Historical red-first evidence has not been recovered; docs/design/outcome-groups.md still labels its design proposed. Keep these evidence/documentation discrepancies explicit rather than reimplementing delivered behavior or claiming acceptance complete.
