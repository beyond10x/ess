---
format: aep.planning-md/3
id: story:generated-server-manifests-take-compatible-version-ranges
kind: story
status: draft
title: Generated server manifests take compatible version ranges
summary: 'Follow-up to N5 (ess#660): the generated server, single-binary and normalization manifests still pin exact versions.'
refs:
- provider: github
  reference: beyond10x/ess#660
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

After the types realizer moved to `^` ranges (ess#660), three generators still write exact `=` pins into a consumer's manifest: the generated HTTP server (`crates/generate/ess-synth/src/rust/http.rs:294`: clap, uuid, time and more), the generated single-binary crate (`crates/generate/ess-synth/src/rust/single.rs:264`) and the normalization target (`crates/generate/schema-contract/src/realize/normalize/target.rs:66`, `:79` base64). A consumer workspace that includes any of them is downgraded the same way. Same class as adopter report N5.

## Acceptance

- `generated_server_manifest_uses_caret_ranges`: the `http.rs` manifest has no `=` requirement; each is `^<tested minimum>`.
- `generated_single_manifest_uses_caret_ranges`: the same for `single.rs`, optional dependencies included.
- `normalization_manifest_uses_caret_ranges`: the same for `normalize/target.rs`, unless a format document pins those manifest bytes for an existing normalization format; then the pinned formats keep their bytes, the reason is recorded here, and only new output moves.
- `no_generator_writes_an_exact_pin`: a source scan of `crates/generate/*/src` finds no `"=<digit>` requirement in a generated manifest.
- Byte-pinned outputs move with the manifests, each change listed in the unit evidence.
