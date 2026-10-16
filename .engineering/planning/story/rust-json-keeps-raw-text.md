---
format: aep.planning-md/3
id: story:rust-json-keeps-raw-text
kind: story
status: implemented
title: The Rust types target can keep a Json value as its raw JSON text
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-14T14:33:22Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-14T14:33:22Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-15T08:45:07Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

`ess generate types --target rust` can map a `Json` value to its raw JSON text
(`Box<serde_json::value::RawValue>`), chosen per type or per field, so the bytes of that value
round-trip unchanged whether or not the build turns on the `exact-numbers` feature.

Closes https://github.com/beyond10x/ess/issues/577.

## Evidence

https://github.com/beyond10x/ess/issues/577 (ess 0.59.0): a `Json` or `Optional<Json>` field is
emitted as `::serde_json::Value` (`crates/generate/schema-contract/src/realize/rust.rs`, `Emitter::ty`,
`Shape::Json`), which puts it under `exact-numbers`. With that feature off, serde_json parses floats
best-effort: `1.0715660391465826e-75` reads back as `...825e-75`, then `...823e-75`. A consumer that
stores a record's exact JSON text cannot use the generated type for it.

## Design

Tooling only; no format change. A repeatable flag on `generate types`,
`--rust-raw-json <QualifiedType>` or `<QualifiedType>.<field>`, resolves to schema pointers in a
`Plan.raw_json` set (beside `Plan.binary64`, `crates/generate/schema-contract/src/realize.rs`). The
Rust emitter writes `Box<::serde_json::value::RawValue>` for a `Json` shape at such a pointer
(the `raw_value` serde_json feature, as the binary64 union path already does), and does not count
it toward `exact-numbers`. A name that resolves to no type or field, or to a field that is not
`Json`, is refused naming the flag value. Other targets refuse the flag.

## Acceptance

- `raw_json_field_round_trips_its_bytes`: with the flag on a field, a generated crate built with
  `exact-numbers` off parses `{"v":1.0715660391465826e-75}` and serializes the same bytes.
- `raw_json_on_a_type_covers_every_json_field_of_it`.
- `raw_json_names_an_unknown_or_non_json_location_and_is_refused`.
- Without the flag, `generate types --target rust` output is byte-identical (existing pins hold).

Tests go into existing test binaries of the crates touched; no new `tests/*.rs` file.
