---
format: aep.planning-md/3
id: story:generated-server-manifests-take-compatible-version-ranges
kind: story
status: implemented
title: Generated server manifests take compatible version ranges
summary: 'Follow-up to N5 (ess#660): the generated server, single-binary and normalization manifests still pin exact versions.'
refs:
- provider: github
  reference: beyond10x/ess#660
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T11:50:59Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-23T11:51:00Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-26T09:27:33Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
## Problem

After the types realizer moved to `^` ranges (ess#660), three generators still write exact `=` pins into a consumer's manifest: the generated HTTP server (`crates/generate/ess-synth/src/rust/http.rs:294`: clap, uuid, time and more), the generated single-binary crate (`crates/generate/ess-synth/src/rust/single.rs:264`) and the normalization target (`crates/generate/schema-contract/src/realize/normalize/target.rs:66`, `:79` base64). A consumer workspace that includes any of them is downgraded the same way. Same class as adopter report N5.

## Acceptance

- `generated_server_manifest_uses_caret_ranges`: the `http.rs` manifest has no `=` requirement; each is `^<tested minimum>`.
- `generated_single_manifest_uses_caret_ranges`: the same for `single.rs`, optional dependencies included.
- `normalization_manifest_uses_caret_ranges`: the same for `normalize/target.rs`, unless a format document pins those manifest bytes for an existing normalization format; then the pinned formats keep their bytes, the reason is recorded here, and only new output moves.
- `no_generator_writes_an_exact_pin`: a source scan of `crates/generate/*/src` finds no `"=<digit>` requirement in a generated manifest.
- Byte-pinned outputs move with the manifests, each change listed in the unit evidence.

## Decisions

- The normalization target keeps its exact pins. `docs/design/raw-json-normalization.md:325-326` and `docs/design/positional-array-normalization.md:445-447` require the legacy manifest bytes to stay identical, and `schema-contract/tests/fixtures/normalization_legacy_maps.json` pins their `Cargo.toml` digests for v1-v5. Only a new normalization format may move them. The source scan `no_generator_writes_an_exact_pin` exempts exactly those 4 pins in `normalize/target.rs`.
- The generated manifests declare no `rust-version`. The tested minimums build on Rust 1.85; a fresh resolve may pick newer releases that need a newer Rust (uuid 1.28.0 needs 1.89, time 0.3.55 needs 1.88). Declaring `rust-version` is a separate byte change, not part of this story.
