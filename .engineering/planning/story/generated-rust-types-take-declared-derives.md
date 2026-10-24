---
format: aep.planning-md/3
id: story:generated-rust-types-take-declared-derives
kind: story
status: active
title: Generated rust types take declared derives
summary: 'Adopter report N2: No derive hook on generated Rust types (ess#657).'
refs:
- provider: github
  reference: beyond10x/ess#657
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T12:51:24Z", actor: "human:timo", revision: 2, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T12:51:24Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
---
## Problem

The Rust realizer emits one fixed derive list (`crates/generate/schema-contract/src/realize/rust.rs:14`) and takes no generator options. Adopters need `Eq`, `Hash`, `Default` and schema derives. Adopter report N2, ess 0.68.0.

## Acceptance

- `a_global_derive_is_added_to_every_type`: a generator option adding `Hash` puts it on every generated type, and the crate compiles.
- `a_per_type_derive_is_added_to_that_type_only`.
- `an_unsatisfiable_derive_is_refused_by_name`: `Eq` on a type holding a float field is refused at generation, naming the type and field.
- `no_option_keeps_the_bytes`: generation without options is byte for byte as before.
