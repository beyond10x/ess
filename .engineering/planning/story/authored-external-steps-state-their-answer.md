---
format: aep.planning-md/3
id: story:authored-external-steps-state-their-answer
kind: story
status: active
title: An authored step expecting an external branch compiles without configure_external_outcome
refs:
- provider: github
  reference: beyond10x/ess#243
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T08:12:55Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T08:12:56Z", actor: "human:timo", revision: 3}
---
## Outcome

An authored step that expects an `external:` branch is satisfiable deterministically: the authored format states the external answer for the step and it compiles into a `configure_external_outcome`, or `ess author` refuses the step naming the missing answer (beyond10x/ess#243).

## Acceptance

- an authored step expecting an external refusal compiles with a `configure_external_outcome` for that call, and a target following it answers the external branch;
- a step expecting an external branch without a stated answer is refused by `ess author` with the branch named;
- steps expecting non-external branches compile as before (bytes unchanged).
