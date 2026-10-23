---
format: aep.planning-md/3
id: story:generated-cargo-manifests-take-compatible-version-ranges
kind: story
status: draft
title: Generated cargo manifests take compatible version ranges
summary: 'Adopter report N5: Generated Cargo.toml pins exact versions (ess#660).'
refs:
- provider: github
  reference: beyond10x/ess#660
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

The generated `Cargo.toml` pins exact `=` versions (`crates/generate/schema-contract/src/realize/rust.rs:100`, `:111`), downgrading crates in a consumer workspace (6 in an adopter's). The default `exact-numbers` feature (`rust.rs:109`) turns on `serde_json/arbitrary_precision` for the whole workspace. Adopter report N5 (and the feature part of N6), ess 0.68.0; #483 covered the feature, not the pins.

## Acceptance

- `the_manifest_uses_caret_ranges`: every dependency is `^<tested minimum>`, no `=`.
- `a_consumer_lock_with_newer_patches_resolves`: a fixture workspace locking `time` at a newer patch builds the generated crate without a downgrade.
- `the_number_feature_default_is_decided`: the default of `exact-numbers` matches the decision recorded on the issue, and the changelog names it.
