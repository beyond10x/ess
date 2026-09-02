---
format: aep.planning-md/1
id: story:relations-design-page
kind: story
status: draft
title: The relation vocabulary is decided on a design page before code
summary: 'docs/design/ess-entity-relations-design-v0.1.md: shape, refusals, projections, rejected alternatives.'
owner: ess
tags:
- design
- relations
relations:
- decomposes: epic:entity-relations
revision: 1
---
# Story: The relation vocabulary is decided on a design page before code

## Outcome

A reader of `docs/design/ess-entity-relations-design-v0.1.md` knows what a relation is in ESS, what it refuses, how it renders in each projection, and which alternatives were rejected — before any parser accepts one.

## Context

The repository's design documents live in `docs/design/` (six `*-design-v0.1.md` files at `2979e6e`). The vocabulary questions are the ones an implementor would otherwise settle by typing: ownership vs reference, cardinality form, direction, and whether a relation may name a deletion policy. `decision-blocker:relation-vocabulary` holds those questions and this page is where the answer is written.

## Acceptance

- The page defines the YAML shape with one example for each of `owns` and `references`, and one refused example for each validation rule.
- It names the projection of a relation in JSON Schema, OpenAPI and Rust, with one extension key used by all three.
- It lists the rejected alternatives (an invariant-only encoding; a separate `relations.yaml` document) with the reason each was rejected.
- It is linked from `README.md` and from the epic.

## Out of Scope

Code. This story lands as a document and a cleared decision-blocker.

## Ambiguities

- `requires-stakeholder-input` — the vocabulary itself; see `decision-blocker:relation-vocabulary`, which names a default for silence.

## Open Questions

None beyond the blocker.
