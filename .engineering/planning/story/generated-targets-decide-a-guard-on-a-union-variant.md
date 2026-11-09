---
format: aep.planning-md/3
id: story:generated-targets-decide-a-guard-on-a-union-variant
kind: story
status: draft
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
scope:
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: cited
  path: crates/generate/ess-synth/src/determined.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/predicate.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/predicate.ts
- confidence: inferred
  path: website/docs/reference/entity-runtime-lowering.md
revision: 5
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

- The generated Rust and Go behaviour for the minimal model pass `example.g.Add/outcome/fetched`
  and `/stored` (`ess verify conform run` against each generated target).
- The Go and TypeScript runtimes evaluate `kind in [web_crawl]` over a union value as the
  interpreter does, with one test each on both variants.
- Entity Runtime lowering of the minimal model either lowers the guard or refuses it with a named
  lowering code that lists `example.g.Add/fetched`. The refusal table in
  `website/docs/reference/entity-runtime-lowering.md` names it.

## Scope

- cited: crates/generate/ess-synth/src/determined.rs
- inferred: crates/generate/ess-synth/src/rust/behaviour.rs
- inferred: crates/generate/ess-synth/src/go/behaviour.rs
- inferred: crates/verify/ess-conformance/src/go/predicate.go
- inferred: crates/verify/ess-conformance/src/ts/predicate.ts
- inferred: crates/generate/ess-entity-runtime/src/subset.rs
- inferred: website/docs/reference/entity-runtime-lowering.md
confidence: low

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

## Source

github:beyond10x/ess#637
