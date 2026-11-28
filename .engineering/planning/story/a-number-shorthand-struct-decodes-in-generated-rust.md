---
format: aep.planning-md/3
id: story:a-number-shorthand-struct-decodes-in-generated-rust
kind: story
status: draft
title: A number shorthand struct decodes in generated Rust
summary: Generated Rust refuses a struct whose short form is a number as rust_shorthand_number; the shorthand Binary64 story merged with this deviation (backlog)
tags:
- backlog
- deviation
- from-finding
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 2
---
## Need

`story:a-shorthand-struct-binary64-location-resolves` asks that a shorthand struct written as a
number alone (example `Gauge`) realizes on every target. On `--target rust` it is still refused as
`/$defs/tracker.ops.Gauge: rust_shorthand_number`. That refusal is deliberate
(`crates/generate/schema-contract/src/realize/rust.rs:502-504`): with `arbitrary_precision`,
serde_json hands a number to a visitor as a map, which cannot be told apart from the record, so the
short form cannot be read soundly. The story merged with this deviation; its test asserts the
named refusal, not `model_binary64_layout`.

Amendment A4 (thirteenth gap report, 2026-11-24): this story is `ess generate types --target rust` only. Synthesized Rust, Go and web servers refuse a shorthand struct for another reason, owned by story:the-generated-rust-go-and-web-targets-read-a-struct-written-in-its-short-form-instead-of-refusing-the-model; the serde reason given here does not apply to them.

## Acceptance

- A shorthand struct whose short form is a number decodes in generated Rust from both its number
  and its object form. Decimal and Binary64 values keep their exact text, and `-0` keeps its sign.
- `ess-cli` `shorthand_binary64` asserts exit 0 for Gauge on `--target rust`, replacing the
  `rust_shorthand_number` assertion.
