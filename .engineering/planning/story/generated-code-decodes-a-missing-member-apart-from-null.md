---
format: aep.planning-md/3
id: story:generated-code-decodes-a-missing-member-apart-from-null
kind: story
status: active
title: Generated code decodes a missing member apart from null
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#678
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:an-input-keeps-missing-and-null-apart
- depends_on: story:conformance-holds-missing-keeps-and-null-clears
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/generate/ess-gen/src/types.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/items.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/layout.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: inferred
  path: crates/generate/schema-contract/src/realize/ts.rs
- confidence: cited
  path: website/docs/reference/entity-runtime-lowering.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:44:17Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:44:18Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}}
---
## Need

Generated Rust, Go and TypeScript decode an `omitted_or_null` input member into one absent value
today, so generated behaviour cannot keep on missing and clear on `null`
(`story:an-input-keeps-missing-and-null-apart`; ADR `a-missing-input-keeps-and-null-clears`,
Effect, "Interpretation and generated code"). Entity Runtime must refuse the constructs by name.

## Acceptance

- For an input declaring `omitted_or_null` that a `missing: unchanged` copy or a `sent()` guard
  reads, the generated Rust, Go and TypeScript input types decode `{}`, `{"summary": null}` and
  `{"summary": "x"}` to three distinct values; inputs no such construct reads keep their generated
  bytes (`cargo test -p ess-synth --locked --test go` and `--test served_codec_names` pass with no
  row re-pinned).
- Generated Rust and Go behaviour for the reproduction keeps the stored field on missing, clears it
  on `null` and sets it on a value; the synthesized suite of
  `story:conformance-holds-missing-keeps-and-null-clears` passes against the generated Rust and Go
  targets.
- Entity Runtime lowering refuses `missing: unchanged` and `sent()` by name, recorded in
  `website/docs/reference/entity-runtime-lowering.md`.

## Scope

- cited: crates/generate/ess-synth/src/rust/behaviour.rs
- cited: crates/generate/ess-synth/src/go/http.rs (`decode_member`, and the generated `optional`
  helper that reads an absent and a `null` member as one answer)
- cited: crates/generate/ess-synth/src/go/layout.rs (`go_type`: `Optional` is one pointer today)
- cited: crates/generate/ess-synth/src/go/behaviour.rs (`input_value`: the copy and `sent()` reads)
- inferred: crates/generate/ess-synth/src/go/items.rs (input struct members)
- cited: crates/generate/ess-gen/src/types.rs (input type realization, nullable fields)
- inferred: crates/generate/schema-contract/src/realize/ts.rs (TypeScript `?:` member from
  `required`)
- inferred: crates/generate/ess-entity-runtime/src/lib.rs (lowering refusal)
- cited: website/docs/reference/entity-runtime-lowering.md

confidence: low

Shared surfaces moved: proposes the `CHANGELOG.md` sentence; no byte-pin row moves.

## Fit review

1. **Need.** Generated code carries the third input state.
2. **Class: gap** (follows from the construct).
3. **Already expressible?** No.
4. **Fit.** Every target handles the construct or refuses it by name (assessing skill, question 4).
   Only inputs a construct reads change, so other generated APIs keep their bytes.
5. **Second adopter.** Any adopter generating a server for a merge-patch endpoint.
6. **Cost.** A generated-API change for affected inputs only (breaking for code that read those
   inputs as one `Option`); no format.
7. **Alternatives.** (a) Generated code refuses the constructs: works on the interpreter only.
   (b) Chosen.

## Decisions

**accept, redesigned**, as the parent story (ADR `a-missing-input-keeps-and-null-clears`, option A).

## Source

github:beyond10x/ess#678
