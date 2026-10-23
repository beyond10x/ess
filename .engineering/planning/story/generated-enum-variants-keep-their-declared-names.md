---
format: aep.planning-md/3
id: story:generated-enum-variants-keep-their-declared-names
kind: story
status: draft
title: Generated enum variants keep their declared names
summary: 'Adopter report N3: Declared enum variant names are lost in generated Rust (ess#658).'
refs:
- provider: github
  reference: beyond10x/ess#658
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

The Rust realizer names enum variants `V0..Vn` with a serde rename to the wire value (`crates/generate/schema-contract/src/realize/rust.rs:163`), because it reads the projected schema's value list and the declared names are gone by then. Adopter report N3, ess 0.68.0.

## Acceptance

- `variants_keep_declared_names`: `variants: [pending, in_review]` generates `Pending` and `InReview` with renames to the wire values.
- `a_wire_spelling_differs_from_the_name`: `{name: done, wire: DONE}` generates `Done` renamed to `DONE`.
- `a_name_with_no_identifier_is_refused`: a declared name that has no identifier after case conversion is refused by name with the `{name, wire}` remedy.
- `go_and_typescript_agree`: Go constants and TypeScript members carry the same declared names.
