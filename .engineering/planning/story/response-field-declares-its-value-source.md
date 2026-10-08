---
format: aep.planning-md/3
id: story:response-field-declares-its-value-source
kind: story
status: draft
title: 'A response field declares its value source: an input or a stored field'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#506
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A response field can state its value source, an input field or a stored field of the subject, and
conformance checks the echo on every accepting outcome.

## Evidence

GitHub https://github.com/beyond10x/ess/issues/506 (ess 0.56.0): a response field declares only its type, so `state`
returned equal to the request's `state` (RFC 6749 §4.1.2) and `iss` equal to the stored issuer
(RFC 9207 §2) cannot be stated. The converse, a generated value stored and returned, is
`story:stored-field-equals-returned-response-value` (https://github.com/beyond10x/ess/issues/498).

## Acceptance

- A response field may carry a value source (`input.<field>` or a stored field); a source whose
  type differs from the field's is refused with a named code.
- Every synthesized accepting scenario checks the returned value against the source, and a target
  returning another value fails.
- It adds an authored key, so it ships with the next source format, not in a 0.5x release before it.

## Design note

The requester proposes `{name: state, type: Optional<State>, value: input.state}`; that syntax is
evidence of the need, not the design. Decide it together with the `{response: <field>}` form of the
converse story so the two directions read as one construct.
