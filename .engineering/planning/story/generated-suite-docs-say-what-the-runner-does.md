---
format: aep.planning-md/3
id: story:generated-suite-docs-say-what-the-runner-does
kind: story
status: active
title: The generated suite and synthesize --help describe what the runner does
refs:
- provider: github
  reference: beyond10x/ess#186
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/mod.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: website/docs/reference/cli.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T11:29:55Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-09-28T11:29:55Z", actor: "human:timo", revision: 9}
---
# Story: the generated suite and `synthesize --help` describe what the runner does

## Outcome

An adopter who follows the generated Go/TypeScript package's comments and README, and
`ess verify conform synthesize --help`, writes a target and a run that work the first time: a
refusal returns its outcome name, the IR target names the suite version it writes, and
`ESS_REPORT_FORMAT=2` is stated as required for suites `/8` and later.

## Why

beyond10x/ess#186 (b10x-bot, from the agentplugins ESS trial round against 0.38.0):

1. `crates/verify/ess-conformance/src/go/runtime.go:1159`: `// Outcome is the branch it took,
   empty when it refused.` — the runner compares `Outcome` for refusals too (`:2029`); a target
   that follows the comment fails every refusal scenario. Same text in the legacy fixture
   `crates/edge/ess-cli/tests/fixtures/go-count-legacy/runtime.go:130` — cited.
2. `crates/edge/ess-cli/src/main.rs:777`: `--target ir` help says "The canonical
   `ess-conformance/1` document"; a plain `ess/1` spec writes `ess-conformance/4` — cited.
3. `crates/verify/ess-conformance/src/go/mod.rs:323` and `src/ts/mod.rs:504`: the README puts
   `ESS_REPORT_FORMAT=2` under "The report" as if optional; without it `go test` stops before any
   scenario (`suite/8 through /21 require explicit ESS_REPORT_FORMAT=2`) — cited.

## Acceptance

- The `Outcome` doc comment (Go, and the TypeScript equivalent if it says the same) states that a
  refusal returns the refusing outcome's name, and a test fails if the emitted comment says
  "empty when it refused".
- `synthesize --help` for `--target ir` names no fixed suite version, or names the rule; the CLI
  reference page matches.
- The emitted README states `ESS_REPORT_FORMAT=2` is required before execution for the versions
  that require it, in the run instructions, not under the report section; a test pins it.

## Scope

- `crates/verify/ess-conformance/src/go/runtime.go` (comment at :1159) — cited; shared with the
  runtime-parity go unit — the coordinator applies this one line at integration
- `crates/verify/ess-conformance/src/go/mod.rs:323`, `src/ts/mod.rs:504` — cited
- `crates/verify/ess-conformance/src/ts/runtime.ts` (the `outcome` doc, if it says the same) — inferred; shared with the runtime-parity ts unit — coordinator applies
- `crates/edge/ess-cli/src/main.rs:777`, `website/docs/reference/cli.md` — cited / inferred
