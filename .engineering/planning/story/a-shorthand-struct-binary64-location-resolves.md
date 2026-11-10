---
format: aep.planning-md/3
id: story:a-shorthand-struct-binary64-location-resolves
kind: story
status: draft
title: A shorthand struct's Binary64 locations resolve in the published schema
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/model_types.rs
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/adversary_d350_u1_pass1.rs
- confidence: inferred
  path: crates/generate/ess-gen/tests/model_types.rs
revision: 6
---
## Problem

Found by adversary pass 1 of wave D350 U1 (`review-result:adversary-d350-u1-pass-1`, F3), confirmed
on 0.92.0. A struct with a shorthand (`ess/27`) is published as `oneOf [short, object]`
(`crates/generate/ess-gen/src/types.rs:993-1003`), but `body_numeric_locations`
(`crates/generate/ess-gen/src/model_types.rs:546-550`) still points its Binary64 fields at
`/$defs/X/properties/...`, which does not resolve in the published schema.

```yaml
format: ess/27
system: tracker
version: v1
domain: tracker.ops
types:
  - name: tracker.ops.Reading
    kind: struct
    shorthand: label
    fields:
      - {name: label, type: String}
      - {name: value, type: Optional<Binary64>}
  - name: tracker.ops.Plain
    kind: struct
    fields:
      - {name: label, type: String}
      - {name: value, type: Optional<Binary64>}
```

```
$ ess generate types --path spec.yaml --target rust --package demo --root tracker.ops.Reading --out rust
error: /$defs/tracker.ops.Reading/properties/value: model_binary64_layout: compiler-owned Binary64 location does not resolve to a numeric wire node
exit 1
$ ess generate types --path spec.yaml --target go --package demo --module example.com/demo --root tracker.ops.Reading --out go
error: (the same) — exit 1
$ ess generate types --path spec.yaml --target typescript --root tracker.ops.Reading --out ts
exit 0; types-report.json names `model_binary64` at `/$defs/tracker.ops.Reading/properties/value`,
and `json_number_precision` at `/$defs/tracker.ops.Reading/oneOf/1/properties/value`
```

`tracker.ops.Plain` realizes on all three targets.

## Acceptance

Every bullet is a case in `cargo test -p ess-gen --locked --test model_types`, over the model above:

- `ess generate types --target rust` and `--target go --root tracker.ops.Reading` exit 0; the
  generated type decodes both `"x"` and `{"label": "x", "value": 1.5}`.
- The TypeScript types-report names the `model_binary64` obligation at
  `/$defs/tracker.ops.Reading/oneOf/1/properties/value`, and every Binary64 location the compiler
  owns resolves, as a JSON pointer, to a numeric node of the published schema.
- A struct `tracker.ops.Gauge` whose shorthand field `level` is itself `Binary64` names its
  locations at `/$defs/tracker.ops.Gauge/oneOf/0` and `/$defs/tracker.ops.Gauge/oneOf/1/properties/level`,
  and `--target rust` and `--target go` over it exit 0.
- `tracker.ops.Plain`'s generated Rust, Go and TypeScript files and its types-report are byte for
  byte those of 0.92.0, and `cargo test -p ess-synth --locked --test served_codec_names --test go`
  passes unchanged.

## Decisions

- A shorthand field that is itself `Binary64` realizes rather than being refused, because
  `oneOf/0` is the field's own `type_ref` (`types.rs:999`), a numeric node, so its location
  resolves like any other.
- The cases go in `tests/model_types.rs`, beside the existing model-type selection tests, not in
  `tests/binary64_bounds.rs`.

## Scope

- cited: crates/generate/ess-gen/src/model_types.rs
- cited: crates/generate/ess-gen/src/types.rs
- inferred: crates/generate/ess-gen/tests/model_types.rs
- inferred: crates/generate/ess-gen/tests/adversary_d350_u1_pass1.rs

confidence: high

Shared surfaces: proposes one `CHANGELOG.md` sentence; moves no byte pin.
