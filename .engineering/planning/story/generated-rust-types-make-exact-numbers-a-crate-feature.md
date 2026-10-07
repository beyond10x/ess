---
format: aep.planning-md/3
id: story:generated-rust-types-make-exact-numbers-a-crate-feature
kind: story
status: draft
title: Generated Rust types turn on serde_json arbitrary_precision only where exact numbers need it, and let a consumer turn it off
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#483
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/schema-contract/src/realize.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/rust.rs
- confidence: inferred
  path: crates/generate/schema-contract/tests/types_manifest_features.rs
revision: 4
---
## Outcome

`ess generate types --target rust` stops forcing `serde_json/arbitrary_precision` on every crate
that depends on the generated library
(https://github.com/beyond10x/ess/issues/483).

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`, run on ess 0.55.0.

1. **Need.** A library crate cannot depend on generated types: the generated manifest
   (`crates/generate/schema-contract/src/realize/rust.rs:83`) always enables
   `arbitrary_precision`, and Cargo unifies features across the consumer's graph, so every
   `serde_json` user there changes its number bytes (`1.50` stays `1.50` instead of `1.5`, `1e2`
   becomes `1e+2` instead of `100.0`; quoted from the adopter, not re-measured). Requester's ask,
   theirs: "no flag to turn the feature off".
2. **Class.** Defect. A two-event model with only `String` fields gets the feature although its
   `types.rs` names `serde_json` nowhere (`grep -c serde_json types.rs` → 0, issue #483).
3. **Already expressible?** No: `ess generate types --help` lists no option for it.
4. **Fit.** The feature is needed only where a realized shape is `serde_json::Number`
   (`Shape::Number | Shape::Integer`, `rust.rs:314`) or reads a raw JSON value
   (`::serde_json::Value`, `rust.rs:289`/`:308`). The generated crate can declare it itself: a
   default-on crate feature that enables `serde_json/arbitrary_precision`, which a consumer turns
   off with `default-features = false`. No CLI flag, no format change, no ESS model change.
5. **Second adopter.** Any service that hashes or signs `serde_json::to_vec` output and also
   depends on a generated event library.
6. **Cost.** Generated `Cargo.toml` bytes change for every Rust type library (`--check` reports
   drift until regenerated). No new keyword, no format version.
7. **Considered.** (a) change nothing; (b) omit the feature when no shape needs it, and keep it
   forced otherwise; (c) a CLI flag `--rust-json-numbers exact|binary64`; (d) (b) plus a default-on
   crate feature. (d) is chosen: it adds no CLI surface, keeps exact numbers the default, and
   leaves the choice where Cargo already makes it.

## Decisions

- **accept, redesigned**: the requester asked for "a flag"; the design is a generated crate
  feature (option d), not an `ess` flag.

## Acceptance

- A model whose selected types realize no `serde_json::Number` and no `serde_json::Value` gets a
  generated `Cargo.toml` whose `serde_json` dependency carries no `arbitrary_precision`.
- A model that realizes one gets a generated crate feature, default-on, that enables
  `serde_json/arbitrary_precision`; the `serde_json` dependency itself does not name it. The
  `raw_value` feature for binary64 plans is unchanged.
- The types report states, for that crate, that turning the feature off realizes numbers as
  binary64 and names the fields affected.
- A test builds the generated crate with and without default features and asserts
  `serde_json::to_vec` of `{"a":1.50,"b":1e2}` gives `{"a":1.5,"b":100.0}` without it.
- The CHANGELOG states the manifest change and that `--check` reports drift until regeneration.

## Spec first

Spec first: model the change in this repository's ESS specification, validate it with the newest
`ess`, regenerate, then implement against the generated code. If the specification cannot express
it, stop and report that; do not hand-write a parallel model.
