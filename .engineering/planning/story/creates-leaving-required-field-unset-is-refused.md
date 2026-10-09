---
format: aep.planning-md/3
id: story:creates-leaving-required-field-unset-is-refused
kind: story
status: draft
title: A creates effect that leaves a required field unset is refused
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A `creates:` effect that leaves a required (non-Optional, no default) field of the created entity
unset is refused at validation, whether or not an invariant reads the field.

## Evidence

An adopter on ess 0.56.0: such a `creates:` validates. On `main` it is refused only when an
invariant reads the field (ESS-COMMAND-018,
`crates/specify/ess-compiler/src/resolve.rs:145`).

## Acceptance

- A validation test: a `creates:` that sets every required field but one is refused naming the
  field; one that sets all of them, or leaves only Optional fields unset, validates.
- The refusal is checked against the repository's own specifications and examples first; any it
  would break is listed in the story before the change lands.
