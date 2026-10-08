---
format: aep.planning-md/3
id: story:authored-success-step-asserts-no-error
kind: story
status: draft
title: An authored step naming a success outcome also asserts no error came back
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

An authored scenario step `outcome: <name>` that names a non-refusing outcome also asserts that no
error came back, so an implementation whose success outcome now refuses under the same name fails
the authored step.

## Evidence

An adopter on ess 0.56.0: a mutant whose success outcome refuses (same outcome name) passes the
authored step; only the synthesized scenario fails it. The authored step checks the outcome name
alone.

## Acceptance

- A conformance test runs an authored step expecting a success outcome against a target that
  answers that outcome with an error; the step fails, naming the unexpected error.
- An authored step expecting a refusal outcome is unchanged.
