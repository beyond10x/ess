---
format: aep.planning-md/1
id: story:relations-in-the-domain-model
kind: story
status: implemented
title: An entity's relations are typed, resolved and refused like its fields
summary: RelationSpec on EntitySpec, a validation pass beside validate_lifecycle_causes, four refusals each with a test that breaks it.
owner: ess
tags:
- domain
- relations
relations:
- decomposes: epic:entity-relations
- depends_on: story:relations-design-page
revision: 4
---
# Story: An entity's relations are typed, resolved and refused like its fields

## Outcome

A modeller who writes a wrong relation is told which rule it broke, by `ess validate`, with the same hint style the field rules use.

## Context

`EntitySpec` has `identity`, `fields`, `states`, `invariants`, `naming` (`crates/ess-domain/src/entity.rs:595-617`). This story adds `relations: Vec<RelationSpec>` and the validation pass that reads it, placed beside `validate_lifecycle_causes` (`crates/ess-domain/src/spec.rs:305`) because it is, like that one, a relation between two members that neither can see alone. Shape and vocabulary come from `story:relations-design-page`.

## Acceptance

- `RelationSpec` parses from the shape on the design page and is rejected with a positioned error otherwise.
- `ess validate` refuses: a target that is not a declared entity; a `via` field that does not exist on the source entity; a `via` field whose type is not the target's identity type (wrapped by `Optional<…>` or `List<…>` as the cardinality demands); an `owns` relation whose target is owned by more than one entity.
- Every refusal is a `ValidationCode` with a hint, and each has a test that breaks it on purpose.
- `schemas/generated/ess.schema.json` regenerates with the new key and `task check` is green.

## Out of Scope

Projections (`story:relations-projected`) and the example (`story:relations-in-the-billing-example`).

## Ambiguities

- `inferable` — the identity type to compare against: `EntitySpec.identity: Field` (`entity.rs:604`).
- `requires-stakeholder-input` — whether an unowned child of an `owns` relation is refused or merely reported. Decides: ESS owner, inside `decision-blocker:relation-vocabulary`. Default: refused.

## Open Questions

None.
