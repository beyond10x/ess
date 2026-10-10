---
format: aep.planning-md/3
id: story:interpreted-absent-optional-comparison-is-false
kind: story
status: draft
title: An interpreted when_subject comparison with an absent Optional field is false
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

On the interpreted target, a `when_subject` comparison that meets an absent Optional stored field
is false, so the scenario runs and the command takes its other branch instead of being reported
unsupported.

## Evidence

An adopter on ess 0.56.0: entity `Code {method: Optional<String>}`, an outcome guarded
`when_subject: {predicate: method == "plain"}`. A scenario exchanging a code created without a
method, expecting the default success outcome, is reported unsupported by `--target interpreted`.
Workaround in use: `{method: {defined: true}}` beside every comparison. Not yet reproduced on
`main`.

## Acceptance

- A conformance test runs that scenario on `--target interpreted` and it passes with the default
  success outcome.
- The same scenario agrees on the Rust, Go and TypeScript targets.
- The predicates reference states what a comparison with an absent stored field evaluates to.
