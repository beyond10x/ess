---
format: aep.planning-md/3
id: story:response-field-presence-conditioned-on-another-field
kind: story
status: draft
title: A response field's presence is conditioned on another field's value
tags:
- adopter-report
- design-first
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 2
---
## Outcome

A response field's presence can be conditioned on another field's value: a field is required when
a list field of the same response contains a given value.

## Evidence

An adopter on ess 0.56.0 needs a response field REQUIRED when another list field contains a
given value and cannot state it. Related to https://github.com/beyond10x/ess/issues/499
(response constraints), which has no cross-field condition.

## Acceptance

- The specification states the condition on a response type; validation refuses a condition that
  names an absent field.
- Synthesis writes a scenario where the condition holds and the field is asserted present, and one
  where it does not hold.
- It adds an authored key, so it ships with a source format; the design is a page under
  `docs/design/` before code.
