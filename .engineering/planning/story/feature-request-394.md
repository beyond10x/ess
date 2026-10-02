---
format: aep.planning-md/3
id: story:feature-request-394
kind: story
status: draft
title: Integer bounds an invariant states become JSON Schema keywords and native widths
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#394
relations:
- serves: vision:O2
- decomposes: epic:message-contract-clients
revision: 1
---
## Outcome

An integer bound or constant a struct's invariant states is a JSON Schema keyword a plain validator enforces, and a model integer with a complete range realizes at a native width in the generated Go and Rust types.

## Acceptance

- A struct invariant comparing a top-level `Integer` field with an integer literal (`>=`, `>`, `<=`, `<`, `==`) publishes `minimum`, `maximum` or `const` on that field's property; the tighter of two bounds wins; `==` on a field that may be absent or `null` publishes `minimum` and `maximum`, not `const`. `x-ess-invariants` is still published verbatim. `!=`, a `Decimal`, a dotted path and a disjunction are not lowered (`crates/generate/ess-gen/tests/integer_bounds.rs`).
- A JSON Schema validator refuses `amount: 2147483648` against `amount <= 2147483647` and `version: 3` against `version == 2`.
- `ess generate types` for a model integer with `minimum` and `maximum` inside `i32` emits Rust `i32` / Go `int32`, inside `i64` emits `i64` / `int64`; with one bound or none it keeps `serde_json::Number` / `EssNumber`. An integer `const` realizes at its value's width and stays a runtime obligation in `types-report.json` (`crates/generate/schema-contract/tests/integer_widths.rs`).
- Bundle input (an imported JSON Schema or OpenAPI document) is unchanged: native widths are inferred for model input only.

## Origin

beyond10x/ess#394.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`, 2026-10-03.

1. **Need.** A wire field is a 32-bit signed integer; a constant field is always `2`. The model states both (`amount >= -2147483648`, `amount <= 2147483647`, `version == 2`), and the published JSON Schema carries them only as `x-ess-invariants` (`crates/generate/ess-gen/src/types.rs`, module doc: "an invariant is published verbatim under `x-ess-invariants` and is visibly an annotation"), so a validator accepts `amount: 2147483648`. The generated Go and Rust types hold the field as an exact-number string, where a hand-written client holds `int32`. Requester's proposal: lower invariants, or add `Int32`/`Int64` primitives.
2. **Class.** Gap: the bound is stated and no projection enforces it.
3. **Already expressible?** No. `Integer` publishes `{"type": "integer"}` only (`types.rs` mapping table); the realizer classifies `minimum`/`maximum` as runtime obligations and maps every integer to `serde_json::Number` / `EssNumber` (`crates/generate/schema-contract/src/realize/rust.rs`, `go.rs`).
4. **Fit.** Reuses the invariant language; adds no authored key. The reason the module gives for not lowering — "`amount >= 0` is a predicate over a `Decimal`, which this mapping renders as a string, so `minimum` cannot express it" — does not hold for `Integer`, which is a JSON integer. Entities publish no invariants to the schema today (only to docs), so the scope is structs, where `x-ess-invariants` already appears. `types-only-realizations.md` says formats "do not infer integer storage widths"; bounds are not formats, and the design page is amended.
5. **Second adopter.** A paging contract: `page_size >= 1`, `page_size <= 500` on an `Integer` field; a validator should refuse `page_size: 0`, and a client should hold `int32`.
6. **Cost.** No format bump: the authored language is unchanged. Generated JSON Schema gains keywords where a struct already stated a bound; generated Go/Rust types change from an exact-number type to a native integer for fields with a complete range — a source-incompatible change for code that read those fields, noted in the changelog.
7. **Alternatives.** (a) Change nothing: validators keep accepting out-of-range values. (b) `Int32`/`Int64` primitives: a format bump, new keywords in every target, and a second way to say what invariants already say. (c) Lower invariants (chosen): no authored surface, the bound is said once.

## Decisions

- **accept, as lowering (2026-10-03):** lower integer bound/equality invariants on struct fields to `minimum`/`maximum`/`const`; infer native widths from a complete range for model input only. No new primitive.
