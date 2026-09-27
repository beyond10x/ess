---
format: aep.planning-md/2
id: story:field-names-wire-and-value-types
kind: story
status: archived
title: Underscore names, newtype map keys, field wire names, presence policy, Json, text patterns
relations:
- decomposes: epic:retrofit-findings-20260927
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/name.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/primitive_admission.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: schemas/generated/ess.schema.json
revision: 6
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

## Derived scope

Derived 2026-09-27 by `aep:story-scoper`. Every line is **cited** or **inferred**.

- **Finding:** #142 is mostly built: command inputs and event fields are `Field`, which flattens `Naming` and accepts `wire:` on the field (`crates/specify/ess-domain/src/types.rs:311-322`); only the nested `naming: {wire: …}` spelling is refused — cited
- **Primary surface:** `crates/specify/ess-domain` — cited
- **Files:** `crates/specify/ess-domain/src/types.rs` — cited: `field_name` :548-570 and `Field::PATTERN` :329/:314 (#141), `TypeRef::Map` :136 and map-key refusal :194 (#143), `Primitive` :44 (#138), newtype `alphabet` :602/:971 (#146), "is not a declared type" :1385/:1411 (#138)
- **Files:** `crates/specify/ess-domain/src/name.rs:201` (`Naming`) — cited, #142
- **Files:** `crates/specify/ess-domain/src/system.rs:67-98` — inferred, a new format version
- **Files:** `crates/specify/ess-domain/src/primitive_admission.rs` — inferred
- **Files:** `crates/specify/ess-domain/src/command.rs:3622-3660` — cited as the alphabet/example check; inferred as the #146 site
- **Files:** `crates/specify/ess-compiler/src/ir.rs:314`, `ResolvedField` — cited; #139 presence policy there — inferred
- **Also likely:** every `match` on `TypeRef::Map` (~30 files) — inferred for #143 if the key type widens
- **Also likely:** every exhaustive `match` on `Primitive` (13 non-test files) — inferred for #138
- **Also likely:** `crates/generate/ess-gen/src/{openapi,asyncapi,types}.rs`, `ess-conformance/src/{witness,synthesize}.rs`, `ess-synth/src/{rust,go,web}/`, `generated/schema`, `schemas/generated` — inferred
- **Documents:** a new design note, CHANGELOG — inferred
- **Confidence:** medium — `ess-domain` sites cited; #143 and #138 fan-out counted from match arms
- **Would collide with:** any unit touching `types.rs` or `system.rs`, any unit matching on `TypeRef` or `Primitive` in `ess-synth`, `ess-gen` or `ess-conformance`, and any schema regeneration
- **Not established:** #142's remaining projection work; where #138 compares structurally; where #146's witness grammar goes; where #139's policy lives; #143's reach depends on design (widen key vs resolve newtype at parse); Go identifiers for `_`-leading names.
