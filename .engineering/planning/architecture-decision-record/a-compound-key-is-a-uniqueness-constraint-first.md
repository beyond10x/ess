---
format: aep.planning-md/3
id: architecture-decision-record:a-compound-key-is-a-uniqueness-constraint-first
kind: architecture-decision-record
status: accepted
title: A compound key is first a uniqueness constraint beside a one-field identity
summary: 'Proposes unique: [fields] refused on create like a duplicate identity, with a record-typed identity later if compound joins are needed (row 21 group J).'
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "proposed", to: "accepted", at: "2026-10-23T19:36:14Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"approval":1}}, correlation: "ess-0690"}
---
Status: accepted 2026-10-23 by the operator, after reviewing all 15 proposed ADRs in full: "I accept all of them" (see this record's transitions).

## Context

An entity's identity is one field (`EntitySpec::identity` is a single `Field`), and a relation's
`via:` names one field. Composite keys were left out of scope when duplicate-identity refusal was
designed: "Range overlap, composite keys and the concurrency argument stay out of scope"
(`docs/design/cross-record-and-stored-field-guards.md:596-600`).

Adopter repro (beyond10x/connectors, 4 `ESS-LIMIT` markers of the 126 triaged in tracker row 21,
group J):

- `ess/domains/feed.yaml:19`: a feed item's identity is (container, id); the specification makes
  `item_id` the identity and keeps `container_id` beside it, and fixtures keep item ids unique across
  containers.
- `ess/domains/feed.yaml:186`: a container id is unique only within its connection.
- `ess/domains/mutations.yaml:194`: an operation reference is (instance, operation id); a relation
  cannot join on both.
- `ess/domains/artifact_provenance.yaml:106`: no uniqueness over (url, revision).

In each case the specification is accepted, but it states a stronger uniqueness than the system has,
so a suite cannot produce the case the system must handle: two rows with the same inner id under
different parents.

## Options

| | what it is | cost |
|---|---|---|
| A | **Composite identity.** `identity:` may list several fields; the identity type is the tuple; relations' `via:` may list the same number of fields. | New format version; every place that handles one identity field changes: compiler handles, view keys, wire paths for `GET /<entity>/{id}`, generated storage keys, synthesis of distinct rows, `ess verify diff`. The largest change; matches the adopters' domains directly. |
| B | **A record-typed identity.** The identity stays one field, whose type may be a declared record (`FeedItemKey { container_id, item_id }`); relations point at it with one `via:`. | New format version; reuses one-field plumbing, but every consumer of an identity value must handle a structured key (wire encoding of a record in a path, ordering, map keys in generated code). |
| C | **A uniqueness constraint beside a one-field identity.** Identity stays a synthetic single field; an entity may declare `unique: [container_id, item_id]`, refused on create like a duplicate identity; relations keep one `via:`. | Smaller format change; synthesis witnesses the duplicate refusal with the existing duplicate-identity shape; adopters with a natural compound key still carry a synthetic id, and a relation still cannot join on two fields (`mutations.yaml:194` stays a marker). |

## Recommendation

C first, then B if relations on compound keys are still needed. C covers three of the four markers
with the duplicate-refusal machinery that exists, and it does not change what an identity is. B
keeps one identity field everywhere, so it costs less than A when the joins are needed.

## Effect

- **Format:** next open `ess/N` for `unique:` (C); B is a later format of its own.
- **Synthesis:** a duplicate tuple on create is witnessed as a refusal with no event and the first
  row unchanged, the shape `cross-record-and-stored-field-guards.md` already uses for duplicate
  identities; two rows that differ in one member are both created.
- **Generated code:** servers and the interpreter check the tuple on create and on any update that
  writes a member; the storage port gains no method.

## Acceptance evidence

None yet. Acceptance is the operator's move of this record to `accepted`.
