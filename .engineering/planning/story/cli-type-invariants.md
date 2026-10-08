---
format: aep.planning-md/3
id: story:cli-type-invariants
kind: story
status: draft
title: A CLI type carries an invariant, or the refusal names where the rule belongs
tags:
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A CLI type can carry an invariant, or the specification names one other place where a
refusal-stage rule over a CLI failure is stated and checked.

## Evidence

`ess specify cli` (0.56.0) refuses any invariant on a CLI type with "CLI type `<type>` has
unsupported invariants or union semantics" (`crates/generate/ess-cli-project`,
`tests/consumer_cli.rs:272`). An adopter wants to state on its `Failure` type that a timeout
after a dispatch step is reported as stage `dispatch` or `outcome_unknown`, never `admission`, and
carries the rule as a comment beside the type until ESS can express it.

## Acceptance

- Either: an invariant over the fields of a CLI type validates and compiles, and the CLI
  projection either enforces it or names it as an obligation in its coverage report; union
  semantics stay refused with their own message, separated from the invariant refusal.
- Or: a design page names the construct where a refusal-stage rule belongs, and the refusal message
  points at it.
- A conformance scenario fails a target that reports the forbidden stage.

## Spec first

Decide the construct in `docs/design/` before code; the refusal message is split first so an
adopter can tell the two causes apart.
