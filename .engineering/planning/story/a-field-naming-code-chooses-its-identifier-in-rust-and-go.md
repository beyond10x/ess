---
format: aep.planning-md/3
id: story:a-field-naming-code-chooses-its-identifier-in-rust-and-go
kind: story
status: draft
title: A field's naming.code chooses its identifier in Rust and Go
summary: Code emitters read naming.code for declarations and enum variants but not for fields; split from beyond10x/ess#659.
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- depends_on: story:a-command-input-and-response-field-may-share-a-name-in-rust
revision: 1
---
## Need

Split from beyond10x/ess#659 (`story:a-command-input-and-response-field-may-share-a-name-in-rust`,
batch 2 unit U8). An author can write `naming: {code: …}` on a field, and before #659 the Rust
refusal for a field collision offered it as the remedy, but no code emitter reads it: field
identifiers come from `name::value_ident(&field.name)` (132 call sites across 20 files under
`crates/generate/ess-synth/src/`, against 0.76.0), and `naming.code` is collected only for
declarations (`crates/generate/ess-synth/src/alias.rs:19-32`) and enum variants (`:41-60`).

Today the working idiom for a collision inside one struct is a rename that keeps the wire
spelling (`{name: foo_bar2, naming: {wire: fooBar}}`), which changes the field's qualified name.
`naming.code` would let an author keep the name and choose only the identifier.

## Decisions

Draft. Not scheduled in batch 2.

- A field's `naming.code` chooses its identifier in Rust and in Go, read in one place beside
  `code_aliases` so the two targets cannot disagree (`alias.rs:5-7`).
- When it lands, the field/field collision remedy in
  `crates/generate/ess-synth/src/rust/feasibility.rs` offers `naming: { code: … }` again beside
  the wire-rename idiom.
- Needs its own fit review and scoping before a batch takes it.

## Acceptance

To be written at scoping. At least: a field with `naming: {code: other}` is emitted as `other` in
Rust and Go with its wire name and document schema unchanged, and a collision inside one struct
clears with `naming.code` on one field.
