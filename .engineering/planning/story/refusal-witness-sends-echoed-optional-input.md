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
revision: 3
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

## Reproducer (ess 0.56.0)

A minimal fixture to write for the test, in a neutral domain `demo.authorize`:

- Command `Authorize` with four Optional inputs: `prompt: Optional<Prompt>` (`example: login`),
  `nonce: Optional<String>`, `max_age: Optional<Int>` (no example), and
  `state: Optional<State>` (`example: af0ifjsldkj`).
- A success outcome `issued` whose event `CodeIssued` sets `state: input.state`, beside
  refusals on the other inputs. `ess specify validate` accepts it.

Observed on a model of this shape: `ess verify conform synthesize --target ir` wrote 34
scenarios (16 authored, 24 refusals). In the synthesized, non-authored scenarios the `Authorize`
steps sent `prompt` 15/15, `nonce` 15/15, `max_age` 15/15 and `state` 0/15, so no synthesized
scenario checks the echo of `state` in `CodeIssued`; only authored ones do. The shape is the same
for a success outcome's event as for a refusal's error payload.

## Acceptance (added)

- On the fixture above, at least the synthesized scenario that reaches `issued` sends `state`
  and expects `CodeIssued.state` equal to it.
- A mutant implementation that drops `state` from `CodeIssued` fails a synthesized scenario.
