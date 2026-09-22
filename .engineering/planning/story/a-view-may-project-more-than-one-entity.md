---
format: aep.planning-md/1
id: story:a-view-may-project-more-than-one-entity
kind: story
status: archived
title: A view may project more than one entity
relations:
- decomposes: epic:specification-runs-as-a-fake-backend
revision: 3
---
## The gap

`pub source: QualifiedName` — one entity, and no join
(`crates/specify/ess-domain/src/view.rs:297`). A view can only publish columns of the table it
names.

## Where that stops being a modelling preference

The adopting project's live `GET /api/v2/agents` is a native-SQL projection with `LEFT JOIN`s. Its
documented response carries `presence`, `status`, `groups`, `tags`, `integration`, `webrtc` and
`state` — **33 of its properties are not columns of `acd_agent`**, measured by validating the
modelled response against the repository's own OpenAPI document.

So the view that models it is not incomplete by omission. It is incomplete by construction: there is
no `source` that reaches those fields, and no relation traversal to reach them with, even though the
entity declares the relations.

## Acceptance

A view may project fields reached through a declared `relations:` entry on its source — at minimum
one hop, `one` and `many` — and the generated query obligation's row type carries them. Cardinality
comes from the relation, so a `many` hop projects a list and a `one` hop projects a value or an
`Optional`.

## Why the relations already carry it

`ess specify validate` already refuses a relation whose target does not exist, whose linking field is
missing or is of the wrong type, and a second entity claiming to own the same one. The graph a join
needs is declared and checked; nothing reads it on the projection side.

## Retirement under the revised ESS evolution scope

The operator explicitly excluded this epic and all twelve dedicated stories on 2026-09-15 in approved plan ess-evolution-20260915 revision 1. Retire this draft through AEP without deleting its original content, relations or journal history. Its acceptance is no longer a requirement of the current initiative. Generic future protocol/UI work is recorded separately in task:deferred-protocol-ui-bindings and does not reopen this artifact.
