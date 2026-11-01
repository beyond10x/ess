---
format: aep.planning-md/3
id: story:a-null-when-absent-binary64-field-realizes
kind: story
status: implemented
title: A null_when_absent Binary64 field realizes in Rust
summary: An Optional<Binary64> field with presence null_when_absent gets its binary64 location at the anyOf node, so Rust realization refuses it.
tags:
- ess-0.70.0
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/model_types.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/binary64_null_when_absent.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T13:05:43Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-11-01T13:05:43Z", actor: "human:timo", revision: 8}
- {from: "active", to: "implemented", at: "2026-11-01T15:20:28Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Finding

Adversary pass on G-U15 (ce0bd3b57, merged 53f7f7571) before 0.69.0, on integrate/ess-batch-0690 f1197bcd4; recorded for 0.70.0 under approval-record:ess-0690-cut-after-gu19.

- `crates/generate/ess-gen/src/model_types.rs:360`: `required()` points at the `anyOf`, not the number; refusal `model_binary64_layout: compiler-owned Binary64 location does not resolve to a numeric wire node`.
- Pre-existing for declared structs (`demo.lab.Reading`); G-U15 makes it reachable through response roots.
- Red tests: `a_null_when_absent_binary64_response_field_realizes`, `a_null_when_absent_binary64_struct_field_realizes` (schema-contract, adversary_gu15_body_roots.rs).

## Scope

Derived 2026-11-01 by `aep:story-scoper` for wave D350 on dc60f4532. Every line is **cited** (read from the story or the tree) or **inferred**. The finding's line numbers have drifted; current lines are given.

- **Primary surface:** `crates/generate/ess-gen` computes the Binary64 location; `crates/generate/schema-contract` (realize) refuses it — cited
- `fields_numeric_locations` `crates/generate/ess-gen/src/model_types.rs:513-521`, using `field.type_ref.required()` at `:516` (called from `:129`) — cited. `required()` strips `Optional` (`crates/specify/ess-compiler/src/ir.rs:268-273`), so the location is the property node, not `/anyOf/0` — cited
- The nullable projection: `types.rs:721-722` (Optional arm), `:767` `nullable_field`, `:821-832` — cited. The refusal: `binary64_codec` `crates/generate/schema-contract/src/realize.rs:1178-1200` (message `:1194`) — cited
- **Pins today:** none for a `null_when_absent` Binary64; `ess-gen/tests/binary64_bounds.rs:41-43` and `schema-contract/tests/binary64_structural.rs:46` show Optional Binary64 with no presence — cited
- **New tests:** `crates/generate/schema-contract/tests/binary64_null_when_absent.rs` (response root and declared struct) — inferred
- **Safety fact:** `plan.binary64` is consulted only for `Shape::Number` (`realize/normalize/check.rs:485`) — cited; `model_paths` (`check.rs:169`) may refuse next — inferred
- **Confidence:** high that `required()` is the defect; medium that the fix suffices (not run)
- **Would collide with:** `model_types.rs` `fields_numeric_locations`; consumers of `binary64_locations` (`realize.rs:723`), `realize/go.rs` — inferred

## Delivered

D350, 2026-11-01: unit c47ab4f64 (merged 5893be750), fix round 7c5fd30a0 with the adversary's cases (merged 0ec9fb366). `types::property_type` gives `nullable_field` and `fields_numeric_locations` the number member of `anyOf [number, null]`; `schema-contract/tests/binary64_null_when_absent.rs` compares whole re-encoded documents, reads fields as `Option<EssBinary64>`, checks the `-0` sign bit and refuses `1e400`. A present `"spare": null` (`omitted_or_null`) round-trips without a generator change. Adversary pass 1: F1 fixed; F3 (shorthand struct locations) became `story:a-shorthand-struct-binary64-location-resolves`. Residue: Go output is planned, not compiled, on this host (no go1.26.5).
