---
title: "0.10–0.11 — the claims a generated scenario could not make"
description: >
  A command outcome says which entity fields it determines and from what, so a generated scenario
  asserts the values a row holds rather than only that the row exists. On the billing example the
  negative-total fault went from caught by 6 of 29 scenarios to 13. A wrong-state branch can also
  accept, and a view can take parameters.
slug: the-claims-a-scenario-could-not-make
tags: [release, ess, conformance]
date: 2026-09-21T09:30:00+02:00
release_tag: "0.11.1"
release_commit: d2f64890bc6fd9a3120093cc11bd7d7d41448737
---

A conformance suite is worth what its assertions are worth. These releases are about three claims
the model had no way to state, each found by an implementation that passed while being wrong.

{/* truncate */}

## `sets:` — which fields the branch determines, and from what

`sets:` is the twin of `payload:`, pointed at the subject instead of an emitted event. Before it,
**nothing in the model related a command's input to the entity state it produced**. A generated
scenario could find the row it had just created and say nothing about what was in it, so an
implementation that stored somebody else's amount passed.

It is validated like `payload:`: the field must exist on the entity, the source must be an input the
command takes, and the types must be assignable or carry a declared conversion.

A generated `contains` then names the values the row holds rather than only which row it is — read
from `sets:` where the source is an input the scenario chose, where the types do not cross a
conversion, and where the view projects the field at the entity's own type. Left out rather than
guessed at otherwise.

Measured on the normative billing example, which declares `sets:` on `CreateInvoice`: the deliberate
negative-total fault was **caught by 6 of 29 scenarios before, and 13 after**.

## `refuses: false` — a branch that accepts

`wrong_state:` could say one thing: the command refuses here, and reports this error. A command that
is deliberately idempotent had no way to be written down. ACD's `EndCall` on a call that has already
ended answers and does nothing, on purpose and documented — and its specification claimed a refusal
the code never makes, which was filed as a defect against ACD before anybody read the code.

It is a key rather than "leave `error:` out", and the distinction matters. A branch missing its error
is a mistake the validator has caught since `wrong_state:` shipped. Making the omission mean
*accepts* would have turned every one of those mistakes into a silent claim.

The generated scenario's id says which claim it carries: `…/accepts/EndCall` beside
`…/refuses/CancelInvoice`.

## `params:` — a view that is not one answer

Most views are one answer the whole system shares. A position in a queue is not: it is per queue,
and one ranked list of every queued call is a different thing from what the implementation can be
asked.

A parameter is read in the filter as `param.<name>`, so the predicate grammar needs nothing new, and
`params:` must be exactly the set the filter reads. Synthesis binds each from what the arrangement
settled — the scenario asks for the lane it just put the order in — and **the runner refuses to send
a request with a parameter it could not resolve**, because a query with a made-up parameter reads a
different set of rows and still returns something.

## And the crates moved

0.11.1 moved the crates under `crates/{specify,generate,verify,infra}/`. On its own that is
housekeeping; it matters because the command surface follows it one release later, and where a thing
is implemented stops being a second fact from how it is spelled.
