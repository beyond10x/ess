---
format: aep.planning-md/3
id: story:generated-fields-keep-declaration-order-and-box-only-recursion
kind: story
status: draft
title: Generated fields keep declaration order and box only recursion
summary: 'Adopter report N7: Generated Rust boxes every reference and sorts fields (ess#661).'
refs:
- provider: github
  reference: beyond10x/ess#661
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

The schema-contract plan keys fields by wire name in a `BTreeMap` (`crates/generate/schema-contract/src/realize/rust.rs:253`), so generated fields come out alphabetically, and the Rust realizer boxes every reference (`rust.rs:409`). Adopter report N7, ess 0.68.0.

## Acceptance

- `fields_follow_declaration_order`: a struct declaring `z, a, m` generates fields `z, a, m` in Rust, Go and TypeScript.
- `only_recursive_references_are_boxed`: a reference to a non-recursive named type is held by value; a type reaching itself keeps `Box`.
- `wire_bytes_are_unchanged`: encoding a fixture value gives the same JSON as before.
