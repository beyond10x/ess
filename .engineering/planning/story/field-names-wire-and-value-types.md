---
format: aep.planning-md/2
id: story:field-names-wire-and-value-types
kind: story
status: draft
title: Underscore names, newtype map keys, field wire names, presence policy, Json, text patterns
relations:
- decomposes: epic:retrofit-findings-20260927
revision: 2
---
## Scope

- #141: a field name may begin with `_` (or carry a wire name).
- #143: a map key may be a newtype of an admitted key primitive.
- #142: `naming: {wire: …}` on command input and event fields, carried into OpenAPI, AsyncAPI,
  schema and conformance.
- #139: a field-level presence policy (`null_when_absent` / `omitted_when_absent`).
- #138: a `Json` primitive, projected as an unconstrained schema and compared structurally.
- #146: `pattern:` or `prefix:` on a `String` newtype, used as constraint and witness grammar.

## Acceptance

Each repro in the issue validates, generates its projections and synthesizes a suite whose
witnesses the declared constraint admits; generated Rust, Go and TypeScript code compiles for
every new shape.
