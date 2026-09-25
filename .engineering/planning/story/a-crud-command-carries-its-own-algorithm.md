---
format: aep.planning-md/2
id: story:a-crud-command-carries-its-own-algorithm
kind: story
status: archived
title: A CRUD command carries its own algorithm
relations:
- decomposes: epic:specification-runs-as-a-fake-backend
revision: 3
---
## The gap

Every command and every view is an implementation obligation. `ess-synth` emits a trait whose doc
comment reads *"the contract is declared; the algorithm is not"*, and a stub that answers
`UnmetObligation` until a human writes the body.

```console
$ ess generate synthesize --path docs --target web
381 capabilities: 335 generated, 34 obligation(s), 12 refused
```

The 335 generated are structs, enums and ports. The 34 are the whole of the behaviour.

## Why that is more than ESS needs to decline

For a command whose outcome is `creates:`, `updates:` or `moves:` on a declared entity, and a view
that projects that entity, **the specification already determines what happens.** Insert the row,
apply the field assignments, take the transition, publish the declared events, project the result.
There is nothing to guess. `ess-domain` already refuses an outcome that changes nothing
(`ESS-COMMAND-007`), which is the same knowledge read the other way.

The obligations written by hand in the adopting project were, in full: insert a row, refuse on a
unique key, replace fields, delete a row, and return a sorted list. Five shapes, fourteen times.

## Acceptance

For a domain whose commands only create, update, move or delete declared entities, and whose views
project them, `ess generate synthesize` emits a realization that runs, with zero obligations — and a
generated in-memory store behind it. An outcome ESS cannot derive stays an obligation, and the plan
says which and why.

## What must stay an obligation

`external:` outcomes, anything reading a clock or an identity source, and anything whose effect is
not expressible as a declared change. The value is in the boundary being drawn by the emitter rather
than by the reader.

## Retirement under the revised ESS evolution scope

The operator explicitly excluded this epic and all twelve dedicated stories on 2026-09-15 in approved plan ess-evolution-20260915 revision 1. Retire this draft through AEP without deleting its original content, relations or journal history. Its acceptance is no longer a requirement of the current initiative. Generic future protocol/UI work is recorded separately in task:deferred-protocol-ui-bindings and does not reopen this artifact.
