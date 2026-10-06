---
format: aep.planning-md/3
id: story:feature-request-283
kind: story
status: implemented
title: 'A command can guard on only one related row (a second exists: false branch is ESS-COMMAND-004)'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#283
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T13:32:47Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-04T13:32:47Z", actor: "human:timo", revision: 6}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:09Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Outcome

A command guards on more than one related row, each with its own `exists: false` answer.

## Acceptance

- A command with two `via` paths and one `exists: false` branch each validates.
- Synthesis witnesses each branch with the other row arranged to pass.

## Origin

beyond10x/ess#283, reported downstream on 0.48.0. The `{related:}` copy part is #270.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (coordinator, 2026-10-01).

- Need: one command guarding on two related rows. Class: gap. Existing idiom: none; ESS-COMMAND-004 keeps one `exists: false`. Fit: generalises `when_related` per `via` path; no new construct.

## Decisions

- **accept, redesigned:** one `exists: false` branch per distinct `via` path; across related rows, step 1 and the new related-predicate step of the precedence order apply in declaration order. From the next format version (ess/21: ess/20 ships alone in 0.49.0). Synthesis arranges the other related rows to pass when witnessing one row's branch. Scheduled after #282, which owns the precedence step.
