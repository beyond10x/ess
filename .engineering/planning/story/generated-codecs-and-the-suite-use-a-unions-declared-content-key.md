---
format: aep.planning-md/3
id: story:generated-codecs-and-the-suite-use-a-unions-declared-content-key
kind: story
status: active
title: Generated codecs and the suite use a union's declared content key
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#637
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:projections-use-a-unions-declared-content-key
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/wire.rs
- confidence: cited
  path: crates/generate/ess-synth/src/web/catalog.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/accessor.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/one_time_execution.go
- confidence: cited
  path: crates/verify/ess-conformance/src/go/response.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/selection.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/response.ts
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:44:12Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:44:13Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

The generated codecs, the web catalogue and the conformance suite must read and write a union's
payload under its declared `content:` key, once
`story:projections-use-a-unions-declared-content-key` carries it in the IR. The suite's structural
declarations carry a union's `tag` and no content key. The Go and TypeScript runtimes recompute the
key from the tag (`crates/verify/ess-conformance/src/go/response.go:720-723`,
`crates/verify/ess-conformance/src/go/one_time_execution.go:366-368`). So an older runner would read
`value` where the suite means `config`, which needs a new suite format pair.

Minimal model: the parent story's `example.g.Kind` with `content: config`.

## Acceptance

- Generated Rust, Go and TypeScript types for the minimal model encode `web_crawl` as
  `{"type": "web_crawl", "config": {…}}`, decode it, and refuse `{"type": "web_crawl", "value": {…}}`
  naming `config`.
- `ess verify conform synthesize` writes `config` in every union value it sends or expects. The
  union's suite declaration carries `content: config` under a new `ess-conformance` pair (the next
  after `/63`). A model without the key keeps its suite format and bytes, and admission refuses the
  member below the new pair.
- `ess verify conform run --target interpreted` and the Go and TypeScript runtimes pass the
  synthesized suite. Each refuses a payload under `value` as an undeclared member of a closed union.
- The web catalogue sends the payload under `config`.

## Scope

- cited: crates/verify/ess-conformance/src/go/response.go
- cited: crates/verify/ess-conformance/src/go/one_time_execution.go
- cited: crates/verify/ess-conformance/src/accessor.rs
- cited: crates/verify/ess-conformance/src/selection.rs
- cited: crates/verify/ess-conformance/src/input.rs
- cited: crates/verify/ess-conformance/src/witness.rs
- cited: crates/generate/ess-synth/src/rust/wire.rs
- cited: crates/generate/ess-synth/src/go/http.rs
- cited: crates/generate/ess-synth/src/web/catalog.rs
- inferred: crates/verify/ess-conformance/src/ts/response.ts
- inferred: crates/verify/ess-conformance/src/go/runtime.go
confidence: medium

## Fit review

1. **Need.** Every lane must agree on the content member, or a target that writes `value` passes
   against a runner that reads `value`. Reproduction: triage run, `content-key`.
2. **Class.** Gap, following the parent story.
3. **Already expressible?** No.
4. **Fit.** The same lanes as the unit-variant change (`docs/design/union-unit-variants.md:74-91`),
   each with a named behaviour.
5. **Second adopter.** A webhook envelope `{event, data}`.
6. **Cost.** A suite format pair, which follows AGENTS.md "Determinism and formats" because the
   persisted envelope's meaning changes. Generated code changes only for models with the key.
7. **Alternatives.** (a) Leave the suite without the key and have runners read the IR: rejected,
   because runners read the suite, not the IR. (b) Put the key in the suite declaration under a new
   pair: chosen.

## Decisions

accept as proposed, as the parent story. This story carries the codecs, the web catalogue and the
suite.

## Source

github:beyond10x/ess#637
