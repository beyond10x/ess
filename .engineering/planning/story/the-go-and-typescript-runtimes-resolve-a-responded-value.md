---
format: aep.planning-md/3
id: story:the-go-and-typescript-runtimes-resolve-a-responded-value
kind: story
status: active
title: The Go and TypeScript runtimes resolve a Responded value
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#498
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:synthesis-witnesses-a-guard-over-a-stored-response-value
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/replay.go
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.test.ts
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/stored_response_values_go.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/stored_response_values_typescript.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:47:32Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:47:33Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

`story:synthesis-witnesses-a-guard-over-a-stored-response-value` adds the scenario value
`Responded {field}` (an earlier step's actual response field) under a new suite format pair and
resolves it in the Rust runner. The Go and TypeScript runtimes written by
`ess verify conform synthesize --target go | typescript` admit suite majors up to 63
(`crates/verify/ess-conformance/src/go/runtime.go:104`, `src/ts/runtime.ts:7177`) and refuse the
pair by major until they resolve the value, so a Go or TypeScript implementation of a model that
stores a returned value cannot run its suite.

## Acceptance

- Both runtimes admit the pair and resolve `Responded {field}` from the retained capture, refusing
  it under a lower major before any target effect.
- On the reproduction suite of the synthesis story, each runtime gives the Rust runner's verdicts,
  including the failing verdicts for the two mutants (parity tests under
  `crates/verify/ess-conformance/tests/`, and a `node --test` case in `src/ts/runtime.test.ts`).

## Scope

- cited: crates/verify/ess-conformance/src/go/runtime.go
- cited: crates/verify/ess-conformance/src/ts/runtime.ts
- inferred: crates/verify/ess-conformance/src/go/mod.rs
- inferred: crates/verify/ess-conformance/src/go/replay.go
- inferred: crates/verify/ess-conformance/src/ts/runtime.test.ts
- inferred: crates/verify/ess-conformance/tests/stored_response_values_go.rs
- inferred: crates/verify/ess-conformance/tests/stored_response_values_typescript.rs
confidence: medium

## Fit review

1. **Need.** Runtime parity for the suite value of the synthesis story (#498). Requester's
   proposal (theirs): the construct only.
2. **Class.** Gap; a runtime ignoring the value would be a silent target (assessing skill, red
   flags), refusing it by major is the interim state.
3. **Already expressible?** n/a.
4. **Fit.** Each runtime already resolves `observed` and retains a command result for replay
   (`go/replay.go`, `ts/runtime.ts`); `Responded` reads the same capture.
5. **Second adopter.** Any Go or TypeScript service whose model stores a returned code.
6. **Cost.** No format of its own; admits the synthesis story's pair.
7. **Alternatives.** Rust only: Go and TypeScript adopters cannot run the suite. Chosen: parity.

## Decisions

Accept, redesigned (Go and TypeScript targets of `story:stored-field-equals-returned-response-value`).

## Source

github:beyond10x/ess#498
