---
format: aep.planning-md/2
id: story:field-wire-names-and-presence-policy
kind: story
status: implemented
title: A field keeps its wire name, and an Optional says whether it is sent as null
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-20260927
- supersedes: story:field-names-wire-and-value-types
revision: 4
---
## Scope

- #142: a field's wire name on command inputs and event fields. Finding (scoper): `Field` already
  flattens `Naming`, so `wire:` written on the field is accepted today
  (`crates/specify/ess-domain/src/types.rs:311-322`); only the nested `naming: {wire: …}` spelling
  is refused. Remaining: accept or name the nested spelling, and check OpenAPI, AsyncAPI, schema and
  conformance honour a field's `wire:`.
- #139: a field-level presence policy for `Optional<T>`: always sent as `null`, or omitted when
  unset.

Split from `story:field-names-wire-and-value-types` (archived).

## Acceptance

- The #142 repro validates and every projection spells the field by its wire name.
- A struct with one `null_when_absent` and one `omitted_when_absent` field projects both policies
  into schema and conformance, and a suite fails an implementation that swaps them.
