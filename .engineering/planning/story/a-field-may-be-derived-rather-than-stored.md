---
format: aep.planning-md/2
id: story:a-field-may-be-derived-rather-than-stored
kind: story
status: archived
title: A field may be derived rather than stored
relations:
- decomposes: epic:specification-runs-as-a-fake-backend
revision: 3
---
## The gap

Every entity field is a stored value. There is no construct for a field computed from the ones
beside it, so anything derived leaves the model.

## Three the adopting project hit in one screen

| field | what it is | where it ended up |
|---|---|---|
| `is_default` on a presence | `['available','busy'].contains(name)` — the domain class declares it `transient` | hand-written JavaScript |
| `count` on a presence | a count of rows referencing it | hand-written, hard-coded 0 |
| `hash` on an availability | a truncated SHA-256 of three of its own fields | not reproduced at all |

Each is sent by the real endpoint and each is invisible to the model, so a consumer reading the
specification is told the response has fewer fields than it has.

## Acceptance

An entity or view field may declare a derivation over fields of the same row — at minimum a boolean
over an enumerated set, and a count over a declared `many` relation — and `ess specify validate`
type-checks it. The generated realization computes it; it is not an obligation.

## Deliberately narrow

Not an expression language. The three cases above are a set membership, a relation count, and a
hash — and the third is fairly excluded as a projection concern rather than a domain one. A
derivation that needs a clock, a random source or IO stays outside.

## Retirement under the revised ESS evolution scope

The operator explicitly excluded this epic and all twelve dedicated stories on 2026-09-15 in approved plan ess-evolution-20260915 revision 1. Retire this draft through AEP without deleting its original content, relations or journal history. Its acceptance is no longer a requirement of the current initiative. Generic future protocol/UI work is recorded separately in task:deferred-protocol-ui-bindings and does not reopen this artifact.
