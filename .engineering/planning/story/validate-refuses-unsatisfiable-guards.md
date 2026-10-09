---
format: aep.planning-md/3
id: story:validate-refuses-unsatisfiable-guards
kind: story
status: draft
title: Validation refuses an unsatisfiable guard, a contradictory when and an unanswered held state
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

`ess specify validate` refuses a guard that can never hold (`input.x != input.x`), a `when:` whose
conjuncts contradict each other, and a held state that no outcome of a command answers, each with
its own code naming the branch.

## Evidence

An adopter on ess 0.56.0: validation accepts all three. On `main`,
`crates/specify/ess-domain/src/command.rs:3277-3280` refuses only a refusal whose guard always
holds (https://github.com/beyond10x/ess/issues/489), and `command/absent_input.rs:137` only
`not defined` on a required input; nothing rejects an unsatisfiable guard.

## Acceptance

- One validation test per shape: each is refused with a code and names the branch; a guard the
  prover cannot decide is accepted, not refused.
