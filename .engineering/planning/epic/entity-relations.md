---
format: aep.planning-md/1
id: epic:entity-relations
kind: epic
status: implemented
title: An entity declares its relations, and the model checks them
summary: 'A relations: list on an entity, validated and projected, so an ownership relation is a declared fact rather than a typed id field plus an invariant somebody remembers.'
owner: ess
tags:
- domain
- relations
revision: 5
---
# Epic: An entity declares its relations, and the model checks them

## Outcome

A modeller writes *an Account owns many CommercialClients* as one declared relation on the entity, `ess validate` refuses it when the target, the carrying field or the cardinality do not agree, and every projection (JSON Schema, OpenAPI, Rust) carries the relation instead of each reader re-deriving it from an id field.

## Why Now

The second adopter of the planning plugins lost a plan on exactly this: the relationship between an account and a commercial-client object was inconsistent across passes. The review of that failure found that ESS is the only typed place in either stack to write the relation down, and that ESS today has no construct for it: `grep -rn 'cardinality\|association\|foreign' crates/ess-domain/src` returns nothing, `EntitySpec` holds `name`, `identity`, `fields`, `states`, `invariants`, `naming` and nothing relational (`crates/ess-domain/src/entity.rs:595-617`), and the only cross-member checks are member-to-domain ownership and command-to-lifecycle causes (`crates/ess-domain/src/spec.rs:302-309`, `:385`). A relation is therefore a typed id field on the child plus an invariant somebody remembers to write. The planning guardrail that now routes a new noun to ESS (`agentplugins` 0.4.0) routes it to a place that cannot hold the one fact that was missing.

## Scope

A `relations:` list on an entity; its vocabulary decided on a design page first; validation in `ess-domain`; the generated schema; the three projections; the billing example; and the `aep reverse openapi` mapping (filed in `aep`, `story:reverse-openapi-emits-relations`) and the plugin rule that cites it (filed in `agentplugins`, `story:ess-schema-cites-relations`).

## Out of Scope

- Deletion and cascade semantics at runtime. ESS describes; the entity runtime decides. A relation may *name* an `on_delete` policy only if the design page decides it belongs in the specification at all.
- A graph query language over relations. `ess` answers `validate`, `compile`, `diff`; a traversal verb is earned by a reader that needs it.
- Migrating existing adopters' models. There are no `relations:` entries to migrate; existing id fields keep validating exactly as today.

## Risks

- The vocabulary is decided by writing code instead of on the design page, and the first adopter's model then has to change. Mitigation: `story:relations-design-page` lands first, and the decision-blocker on the vocabulary is cleared by the operator, not by the implementor.
- Projections disagree on how a relation renders (OpenAPI has no native ownership construct). Mitigation: one extension key, named on the design page, used by all three.

## Ambiguities

- `requires-stakeholder-input` — the relation vocabulary: `owns` vs `references`, whether cardinality is `one|many` or `min/max`, and whether `on_delete` is in scope. Decides: the ESS owner. Raised as `decision-blocker:relation-vocabulary`.
- `inferable` — the identity type a relation is carried by is the target's `identity.type` (`crates/ess-domain/src/entity.rs:604`).
- `inferable` — every declared reference must resolve, and the assembler already refuses one that does not (`crates/ess-domain/src/spec.rs:215`, `:265` at the reviewed revision).

## Done When

`ess validate` refuses a relation whose target does not exist, whose carrying field has the wrong type, or whose cardinality contradicts the field's shape; `ess compile` emits the relation in all three projections; the billing example carries one ownership relation and its projections are checked in `task check`.
