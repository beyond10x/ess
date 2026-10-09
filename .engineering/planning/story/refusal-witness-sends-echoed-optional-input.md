---
format: aep.planning-md/3
id: story:refusal-witness-sends-echoed-optional-input
kind: story
status: draft
title: A refusal's witness sends the Optional input its error payload echoes
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 2
---
## Outcome

When a refusal's error payload echoes an Optional input (`payload: {Err: {state: input.state}}`),
synthesis sends that input in the refusal's witness and `expect_error` asserts the echoed field,
so an implementation that drops it fails.

## Evidence

An adopter on ess 0.56.0: the synthesized refusal scenario omits the Optional `state` input, so
`expect_error` carries no fields and an implementation that drops the echo passes. An `example:`
on the input does not change it. Workaround in use: a hand-written
`error: {fields: {state: ...}}`.

## Acceptance

- A synthesis test over that shape writes the refusal scenario with `state` sent and
  `expect_error.fields.state` equal to it.
- A mutant implementation that omits the echoed field fails that scenario on the interpreted
  target.

## Evidence (second adopter report)

A second adopter on ess 0.56.0: synthesis never sends the Optional input `Authenticate.state`,
even with an `example:`, so no echo of it is checked. On `main`,
`crates/verify/ess-conformance/src/witness.rs:4480-4492` fills an Optional in the base input and
`:771-779` tries omissions last, so the path that leaves it out is still to be found from a
reproducer (possibly `synthesize.rs:2799`).
