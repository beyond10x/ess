---
format: aep.planning-md/3
id: story:generated-code-decodes-a-missing-member-apart-from-null
kind: story
status: draft
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
revision: 2
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
  bytes.
- Generated Rust and Go behaviour for the reproduction keeps the stored field on missing, clears it
  on `null` and sets it on a value; the synthesized suite of
  `story:conformance-holds-missing-keeps-and-null-clears` passes against the generated Rust and Go
  targets.
- Entity Runtime lowering refuses `missing: unchanged` and `sent()` by name, recorded in
  `website/docs/reference/entity-runtime-lowering.md`.

## Scope

- cited: crates/generate/ess-synth/src/rust/behaviour.rs
- inferred: crates/generate/ess-synth/src/go/
- cited: crates/generate/ess-gen/src/types.rs (input type realization, nullable fields)
- inferred: crates/generate/ess-entity-runtime/src/lib.rs (lowering refusal)
- cited: website/docs/reference/entity-runtime-lowering.md
confidence: low

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
