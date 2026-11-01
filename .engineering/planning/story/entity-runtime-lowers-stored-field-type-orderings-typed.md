---
format: aep.planning-md/3
id: story:entity-runtime-lowers-stored-field-type-orderings-typed
kind: story
status: draft
title: Entity Runtime lowers a stored field type's ordering invariant typed, or refuses it by name
relations:
- serves: vision:O2
revision: 1
---
## Finding

Measured by adversary pass 1 of wave D343 (`review-result:adversary-d343-u2-pass-1`, F4, out of
scope there, inferred from code): the Entity Runtime lowering of a stored field type's ordering
invariant goes through the untyped `lower_predicate` path (`crates/generate/ess-entity-runtime/src/lib.rs`,
`lower_nominal_invariants`), the path D343 U2 replaced for input types. An ordering over a
`Timestamp` or text becomes a string `compare`, which entity-core 0.28.0 answers `Unknown` when no
scale is declared. U2's implementor also noted that `==` on a `Timestamp` is still a string
comparison, in guards as well.

## Acceptance

- A stored field type's ordering invariant lowers through the typed path (Timestamp to
  `before`/`after`, number to `compare`), or is refused by name with a lowering code; none lowers to
  a rule entity-core answers `Unknown`.
- A red case first shows today's behaviour on a Timestamp-ordered stored field.
