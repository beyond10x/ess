---
format: aep.planning-md/3
id: story:generated-targets-decide-a-guard-on-a-union-variant
kind: story
status: active
title: Generated targets decide a guard on a union variant
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#637
- provider: github
  reference: beyond10x/ess#764
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key
- depends_on: story:synthesis-and-the-interpreter-decide-a-guard-on-a-union-variant
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/union_variant_guard.rs
- confidence: cited
  path: crates/generate/ess-synth/src/determined.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/union_variant_guard_behaviour.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/predicate.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/predicate.test.ts
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/predicate.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/union_variant_guard_runtimes.rs
- confidence: inferred
  path: website/docs/reference/entity-runtime-lowering.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:44:47Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:44:48Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Every generated target must decide a guard on a union's variant label, or refuse it by name, once
`story:a-guard-reads-a-union-tag-and-a-union-names-its-content-key` admits it. Generated Rust and Go
render a supported subset of guards and refuse the rest as obligations
(`crates/generate/ess-synth/src/determined.rs`). Entity Runtime refuses what it does not lower
(`crates/generate/ess-entity-runtime/src/subset.rs`). The Go and TypeScript suite runtimes
re-implement the predicate evaluator (`crates/verify/ess-conformance/src/go/predicate.go`,
`crates/verify/ess-conformance/src/ts/predicate.ts`).

Minimal model: the parent story's `example.g.Add` over `example.g.Kind`.

## Acceptance

- `cargo test -p ess-synth --locked --test union_variant_guard_behaviour`: the generated Rust
  behaviour and the generated Go behaviour for the minimal model each pass
  `example.g.Add/outcome/fetched` and `example.g.Add/outcome/stored` under `ess verify conform run`
  (exit 0, both scenarios `passed`), and neither scenario is answered as an unmet obligation.
- `cargo test -p ess-conformance --locked --test union_variant_guard_runtimes`: the Go runtime
  answers `kind in [web_crawl]` as `True` over `{type: web_crawl, value: {…}}` and `False` over
  `{type: text}`, as the interpreter's evaluator does; `cargo test -p ess-conformance --locked
  --test typescript_runtime` runs the same two cases in `src/ts/predicate.test.ts` with the same
  answers.
- `cargo test -p ess-entity-runtime --locked --test union_variant_guard`: lowering the minimal
  model's component returns `Err` whose diagnostics hold `LoweringCode::UnionVariantGuardUnsupported`
  naming `example.g.Add/fetched`.
- `cargo test -p ess-entity-runtime --locked --test lowerable_subset` passes with a refused row for
  the union variant guard under `UnionVariantGuardUnsupported`, rendered into
  `website/docs/reference/entity-runtime-lowering.md`.

## Scope

- cited: `crates/generate/ess-synth/src/determined.rs`
- inferred: `crates/generate/ess-synth/src/rust/behaviour.rs`
- inferred: `crates/generate/ess-synth/src/go/behaviour.rs`
- inferred: `crates/generate/ess-synth/tests/union_variant_guard_behaviour.rs`
- inferred: `crates/verify/ess-conformance/src/go/predicate.go`
- inferred: `crates/verify/ess-conformance/src/ts/predicate.ts`
- inferred: `crates/verify/ess-conformance/src/ts/predicate.test.ts`
- inferred: `crates/verify/ess-conformance/tests/union_variant_guard_runtimes.rs`
- cited: `crates/generate/ess-entity-runtime/src/lib.rs` (`LoweringCode`, the union lowering)
- inferred: `crates/generate/ess-entity-runtime/src/subset.rs`
- inferred: `crates/generate/ess-entity-runtime/tests/union_variant_guard.rs`
- inferred: `website/docs/reference/entity-runtime-lowering.md` (rendered from `subset.rs`)

confidence: low

Moves no shared surface.

## Fit review

1. **Need.** A construct the parent story admits must not be ignored by a target. Reproduction: the
   parent story's model.
2. **Class.** Gap, following the parent story.
3. **Already expressible?** Not applicable.
4. **Fit.** Grid O union column. Every target answers or refuses by name
   (`.agents/skills/assessing-external-requests/SKILL.md`, question 4).
5. **Second adopter.** As the parent story.
6. **Cost.** No format. Generated code changes only for models with a union guard.
7. **Alternatives.** (a) Refuse it in every generated target: rejected for Rust and Go, which already
   hold the union's variant as a Rust enum or a Go type switch. (b) Implement it in each target: chosen.
   Entity Runtime may refuse by name.

## Decisions

accept as proposed, as the parent story. This story carries the generated targets and the suite
runtimes.

2026-11-10 (plan-critic round 1): Entity Runtime refuses the guard by name, with a new
`LoweringCode::UnionVariantGuardUnsupported`, rather than lowering it, because its lowering of a
union field refuses a unit variant (`UnitVariantUnsupported`, which the minimal model's `text`
is) and walks a union payload untyped, so no lowered form of the tag comparison is established to
hold what ESS decides.

## Source

github:beyond10x/ess#637
