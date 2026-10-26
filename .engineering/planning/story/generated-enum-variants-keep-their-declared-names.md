---
format: aep.planning-md/3
id: story:generated-enum-variants-keep-their-declared-names
kind: story
status: implemented
title: Generated enum variants keep their declared names
summary: 'Adopter report N3: Declared enum variant names are lost in generated Rust (ess#658).'
tags:
- ess-0.69.0
refs:
- provider: github
  reference: beyond10x/ess#658
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T20:20:12Z", actor: "human:timo", revision: 4, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T20:20:12Z", actor: "human:timo", revision: 5, correlation: "ess-0690"}
- {from: "active", to: "implemented", at: "2026-10-26T09:27:33Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
## Problem

The Rust realizer names enum variants `V0..Vn` with a serde rename to the wire value (`crates/generate/schema-contract/src/realize/rust.rs:163`), because it reads the projected schema's value list and the declared names are gone by then. Adopter report N3, ess 0.68.0.

## Acceptance

- `variants_keep_declared_names`: `variants: [pending, in_review]` generates `Pending` and `InReview` with renames to the wire values.
- `a_wire_spelling_differs_from_the_name`: `{name: done, wire: DONE}` generates `Done` renamed to `DONE`.
- `a_name_with_no_identifier_is_refused`: a declared name that has no identifier after case conversion is refused by name with the `{name, wire}` remedy.
- `go_and_typescript_agree`: Go constants and TypeScript members carry the same declared names.

## Decision: fall back, do not refuse (2026-10-23)

A model that generates today keeps generating. When one declared variant name of an enum has no identifier (empty, a leading digit, `Self`), or two names give the same identifier, that enum keeps positional `V<n>` names in Rust and Go and gets no TypeScript constant. The acceptance test `a_name_with_no_identifier_is_refused` is replaced by `a_name_with_no_identifier_keeps_positional_variants` and `two_names_with_one_identifier_keep_positional_variants`.

An `x-ess-variants` annotation that is not a map from an enum value to a string is still refused (`model_variants`). ess-gen never writes one, so no model that generates today reaches that refusal.
