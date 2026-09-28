---
format: aep.planning-md/3
id: story:generated-runtimes-run-every-emitted-suite-version
kind: story
status: active
title: The generated Go and TypeScript runtimes run every suite version the release synthesizes
refs:
- provider: github
  reference: beyond10x/ess#188
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/predicate.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/reading.go
- confidence: cited
  path: crates/verify/ess-conformance/src/go/replay.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/response.go
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/predicate.ts
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/reading.ts
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/response.ts
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.test.ts
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/presence_suite_versions_go.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/typescript_runtime.rs
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T10:17:58Z", actor: "human:timo", revision: 14}
- {from: "proposed", to: "active", at: "2026-09-28T10:17:59Z", actor: "human:timo", revision: 15}
---
# Story: the generated runtimes run every suite version the same release synthesizes

## Outcome

A project that runs its conformance suite through the Go or TypeScript package ESS generates gets a
verdict for every scenario of every suite version that release's `ess verify conform synthesize`
can write — `ess-conformance/22` to `/27` included — instead of `suite admission: unsupported suite
version` and zero verdicts.

## Why

beyond10x/ess#188 (operator, 2026-09-28), confirmed by a downstream adopter's measurement on 0.38.0:
an unchanged specification (a nested `sets:` struct with one `{generated: true}` leaf) synthesizes
`ess-conformance/26` because #179 asserts determined leaves under dotted paths, and both generated
runtimes admit only `/2`–`/21`:

- Go: `crates/verify/ess-conformance/src/go/runtime.go:80`, `:1572`, `:3588-3600` (version table),
  `go/replay.go:57` — cited
- TypeScript: `crates/verify/ess-conformance/src/ts/runtime.ts:738`, `:2358-2359`, `:4310-4330`
  (version table, `unsupported suite version` throw) — cited

The adopter loses 424 scenarios. The-5-waves (0.38.0) recorded the refusal as a deliberate choice
("Go and TypeScript runtimes refuse /26 and /27 by version, as they do /20–/25",
`.engineering/waves/the-5-waves.md`); #188 reverses it.

## Acceptance

- For each of `/22`, `/23`, `/24`, `/25`, `/26`, `/27`, the Go and the TypeScript runtimes execute
  every step and expectation the Rust reference runner executes for that version, and give the same
  verdict per scenario on the repository's own fixtures (each version's `used_by` constructs:
  0.37 outcome shapes — `unknown_instance`, `deletes`, `into`, `accepts: nothing`, system
  preconditions — and field presence; 0.38 dotted-path leaves, `execute_command_without_input`,
  `related`, `changed_by`, `page`, `caller`, `now_offset`, set effects / `affects`, bounded retry).
- A test fails when the synthesizer can write a suite version that either generated runtime does not
  admit, so the gap cannot reopen silently.
- `/26` with dotted-leaf expectations is delivered first; a construct that cannot be executed in a
  runtime refuses that one scenario with a named reason instead of refusing the suite.

## Scope

Derived 2026-09-28 by the coordinator from the files above — cited unless marked.

- `crates/verify/ess-conformance/src/go/runtime.go`, `go/replay.go`, `go/predicate.go`,
  `go/reading.go`, `go/response.go` — cited (runtime.go/replay.go), inferred (others)
- `crates/verify/ess-conformance/src/ts/runtime.ts`, `ts/runtime.test.ts`, `ts/predicate.ts`,
  `ts/reading.ts`, `ts/response.ts`, `ts/presence.test.ts` — cited (runtime.ts), inferred (others)
- tests: `crates/verify/ess-conformance/tests/presence_suite_versions_go.rs`,
  `tests/typescript_runtime.rs`, new per-language parity tests — inferred
- not touched: `go/mod.rs`, `ts/mod.rs`, `go/explore.go`, `ts/explore.ts` (the concurrent-history
  branch edits them) — decision
