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
- serves: vision:O2
- decomposes: epic:adopter-gaps-wire-and-answers
revision: 4
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

## a2a wave 1

Relayed by the orchestrator on 2026-10-22 from the adopter session specifying A2A in
beyond10x/a2a (ESS 0.67.0, a2a main 3a18050):

- **M7** (a2a wave-1 gaps list, line 41): the found branch of a get command cannot bind its
  response body to the matched row. That is this story's stored-field value source, applied to
  every field of the response (the whole row), not one field.

## Fake backend, 2026-11-07

An adopter reported this on 2026-11-07 against ESS 0.87.0, while specifying an existing backend for
a testing fake. The gap is still present in 0.90.0.

- 24 commands bound with `body: response` answer fields that nothing in the model sources:
  - `success: true`, a literal;
  - `item.id`, equal to `input.id`, an input echo inside a nested response struct;
  - a fixed `message` text, a literal.
- The adopter's feature docs record each value, with a citation, but the model leaves them
  unmapped. A runtime built from the specification therefore mints its own values, and conformance
  cannot hold a target to the documented ones.
- Their framing: "ESS has no form that sources a command response field (as `payload:` sources
  event fields)."

This widens the outcome beyond an input or a stored field.

- A response value source should also take a literal, and a path into a nested response field.
  It should be per outcome, the same way `payload:` maps an event's fields.
- This is the same construct as the A2A M7 row binding above. One design should cover the input
  echo, the stored field, the whole matched row and the literal.
- The design decision about the converse `{response: <field>}` form is unchanged.

## Related (2026-11-25)

A second adopter asks (twentieth gap report, shape 4) whether `ess-http` can answer the stored row of a
command's subject without a declared response, for a PUT or POST that answers the saved resource.
Read at tag 0.99.1:

- It cannot. A command route answers `body: response` only for a `returns: true` outcome without an
  error, naming the command's declared response object; without `returns: true` the form is refused
  `ESS-HTTP-024` ("`response` without `returns: true`", `CHANGELOG.md`, 0.60.0, the `outcomes` entry;
  `docs/design/http-presentation-binding.md`, "Response body per outcome", the row `response`). The
  default `envelope` answer carries a `response` member only when one is declared.
- A declared response is a modelling task, and its values have no source in the model today: the
  members of a response are typed, and only a value an outcome generated and stores is tied to the
  response (`story:stored-field-equals-returned-response-value`, `{response: <field>}`). This story is
  the other direction, which the report needs: a response member whose value is the stored field of
  the subject, or the whole stored row (the M7 case of this story).

Decision HB-D5 (shape 4): where "answer the saved row" lives.

- A. In the model: this story's value source for a response member (an input, a stored field, the
  whole matched row, a literal), after which `body: response` of the existing binding answers it. One
  statement for every target; the suite checks the echoed row.
- B. In the binding: a body word `subject` that answers the stored row of the command's subject. Cost:
  the answer depends on a stored fact the specification of the command does not state, so the suite
  cannot check it, and each protocol binding would grow the same word.
- C. Decline: the consumer declares a response, a view answers the row, and the client issues the read.
- Recommended A: the row a command answers is semantics, which the binding only presents. This story
  already designs it; the report is a third adopter.
