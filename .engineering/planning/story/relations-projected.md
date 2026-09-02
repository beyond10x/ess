---
format: aep.planning-md/1
id: story:relations-projected
kind: story
status: draft
title: A relation survives into every projection
summary: One extension key carries target and cardinality into JSON Schema, OpenAPI and Rust; golden tests per projection.
owner: ess
tags:
- projections
- relations
relations:
- decomposes: epic:entity-relations
- depends_on: story:relations-in-the-domain-model
revision: 1
---
# Story: A relation survives into every projection

## Outcome

A reader of the generated OpenAPI document, the JSON Schema or the Rust types sees the relation the model declared, under one extension key, rather than an id field they have to interpret.

## Context

Today a relation is a typed id field and readers re-derive it. The design page names the extension key; this story applies it to the three projections `ess compile` produces.

## Acceptance

- JSON Schema: the carrying property carries the extension key with target and cardinality.
- OpenAPI: the same key on the property, and the target schema is referenced by `$ref`.
- Rust: a doc attribute or a typed newtype reference that names the target; the generated crate compiles.
- A golden test per projection over the billing example diffs byte-exact.

## Out of Scope

Generating navigation code (fetch the owner, list the children). Projections describe; runtimes navigate.

## Ambiguities

- `inferable` — the extension key name comes from the design page; if the page has not landed this story cannot start (`depends_on`).

## Open Questions

None.
