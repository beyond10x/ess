---
format: aep.planning-md/2
id: story:json-values-and-text-patterns
kind: story
status: implemented
title: Any JSON value is a type, and a text type can require a prefix or pattern
relations:
- decomposes: epic:retrofit-findings-20260927
- serves: vision:O2
- supersedes: story:field-names-wire-and-value-types
revision: 4
---
## Scope

- #138: a `Json` primitive: any JSON value, projected as an unconstrained schema, compared
  structurally.
- #146: `pattern:` or `prefix:` on a `String` newtype, used as constraint and as witness grammar.

Split from `story:field-names-wire-and-value-types` (archived). Reach (scoper, inferred): 13
non-test exhaustive `match` sites on `Primitive` for #138; the witness generator in
`crates/verify/ess-conformance/src/witness.rs` for #146.

## Acceptance

- `{kind: newtype, of: Json}` validates, projects to `{}` in JSON Schema, and a suite compares an
  object payload structurally.
- A `String` newtype with `prefix: "/"` synthesizes only witnesses that start with `/`, and a
  literal that does not is refused.
