---
format: aep.planning-md/3
id: story:generated-rust-types-pass-rustfmt-and-clippy
kind: story
status: active
title: Generated Rust types pass rustfmt and clippy
summary: 'spindle wave 12: ess generate types --target rust fails rustfmt and clippy derivable_impls (ess 0.88.0)'
tags:
- backlog
- defect
- from-consumer
- queue-0.91.0
- rank-4
- severity-low
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/generate/ess-gen/tests/rust_types_lint.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/rust.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/rust_support.rs.txt
- confidence: inferred
  path: crates/generate/schema-contract/tests/rust_realization.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:44:30Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:44:31Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Reported by a downstream consumer against ess 0.88.0, reproduced on 0.92.0. The Rust types that
`ess generate types --target rust` writes do not pass `rustfmt --check`, and fail
`clippy -D warnings` with `derivable_impls`: the hand-written `impl<T> Default for EssPresence<T>`
(line 13 of every generated `types.rs`) that `#[derive(Default)]` with `#[default]` would express.
An adopter that runs fmt and clippy over its tree must exempt the generated file or patch it.

```
$ ess generate types --path examples/billing --target rust --package billing-types \
    --all-types --all-events --all-commands --out rs
$ rustfmt --edition 2021 --check rs/types.rs
Diff in rs/types.rs:11:   fn default() -> Self { Self::Absent }  -> multi-line body
Diff in rs/types.rs:18:   pub fn is_absent(&self) -> bool { matches!(self, Self::Absent) }
Diff in rs/types.rs:34:   pub fn ess_required<'de, D: …, T: …>(deserializer: D) -> … (over width)
Diff in rs/types.rs:314:  if let Ok(decoded) = ::serde_json::from_value::<…Variant0>(value.clone()) {
(5 hunks, exit 1)
$ clippy-driver --edition 2021 --crate-type lib -D warnings --emit=metadata presence.rs
error: this `impl` can be derived  [clippy::derivable_impls]   (exit 1)
```

`presence.rs` is lines 6-19 of the generated file, the preamble from
`crates/generate/schema-contract/src/realize/rust_support.rs.txt`. Replacing the impl with
`#[derive(Default)]` and `#[default] Absent` passes clippy and adds no `T: Default` bound: a
`EssPresence<NotDefault>::default()` still compiles. The union decoder's long line is written by
`crates/generate/schema-contract/src/realize/rust.rs` (`::serde_json::from_value::<{ty}>`, around
line 657).

## Acceptance

- `cargo test -p ess-gen --locked --test rust_types_lint` passes a case that writes `types.rs` with
  `ess generate types --target rust` for `examples/billing`, for each fixture of
  `crates/generate/ess-gen/tests/model_types.rs`, and for a model declaring an enum and a union,
  and asserts `rustfmt --edition 2021 --check` exits 0 on each.
- The same test asserts `cargo clippy -- -D warnings` exits 0 in the generated crate of each of
  those models.
- The same test adds to one generated crate a type `NotDefault` without `Default` and asserts that
  `EssPresence::<NotDefault>::default()` compiles.
- `cargo test -p schema-contract --locked --test rust_realization` passes a case asserting that the
  generated preamble holds `#[derive(Default)]` and `#[default]` on `Absent`, and no
  `impl<T> Default for EssPresence<T>`.

## Scope

- cited: crates/generate/schema-contract/src/realize/rust_support.rs.txt
- cited: crates/generate/schema-contract/src/realize/rust.rs
- inferred: crates/generate/schema-contract/tests/rust_realization.rs
- inferred: crates/generate/ess-gen/tests/rust_types_lint.rs (new)

confidence: medium

Shared surfaces moved: re-pins every byte-pin row that holds a generated Rust `types.rs`, naming
this story as the cause.
