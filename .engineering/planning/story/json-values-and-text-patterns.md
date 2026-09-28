---
format: aep.planning-md/3
id: story:json-values-and-text-patterns
kind: story
status: implemented
title: Any JSON value is a type, and a text type can require a prefix or pattern
relations:
- decomposes: epic:retrofit-findings-20260927
- serves: vision:O2
- supersedes: story:field-names-wire-and-value-types
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T14:52:26Z", actor: "human:timo", revision: 2, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T14:53:02Z", actor: "human:timo", revision: 3, imported: true}
- {from: "active", to: "implemented", at: "2026-09-27T15:03:56Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}, imported: true}
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
