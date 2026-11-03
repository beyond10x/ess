---
format: aep.planning-md/3
id: story:component-explorer-walks-only-its-component
kind: story
status: active
title: A component's Go explorer walks only that component's commands
tags:
- adopter-report
- defect
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/coverage.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/explore.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/mod.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T05:57:19Z", actor: "human:timo", revision: 9}
- {from: "proposed", to: "active", at: "2026-11-03T05:57:20Z", actor: "human:timo", revision: 10}
---
## Outcome

The Go package `ess generate synthesize --component <c>` writes explores only that component's
commands, so a single-component target passes without `AllowExcluded`, and `AllowExcluded` keeps
meaning that a real exclusion was accepted.

## Evidence

An adopter on ess 0.56.0: the component package's explorer walks the other component's commands.
On `main`, `crates/edge/ess-cli/src/main.rs:4600` passes the whole IR to `emit_with_model`, which
`crates/verify/ess-conformance/src/go/mod.rs:136` writes as `ir.json`; `explore.go` has no
component filter. https://github.com/beyond10x/ess/pull/518 fixed authored scenarios outside
`--component` only.

## Acceptance

- A test synthesizes the Go package for one component of a two-component specification; the
  explorer's command set holds only that component's commands and its run passes with
  `AllowExcluded` unset.

## Scope

Derived 2026-11-03 by `aep:story-scoper` for wave W2 on b89faaa41. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance/src/go/` (the Go explorer) — cited
- **Files:** `crates/verify/ess-conformance/src/go/explore.go` — inferred (edit target: `exploreLoad` at explore.go:289-291 walks every `ir["commands"]` with no component filter; a component-filtered `ir.json` would fail the digest check at explore.go:279-287, so the filter lives in the explorer or in a scoped digest)
- **Files:** `crates/verify/ess-conformance/src/go/mod.rs:136` — inferred (`with_model` writes the full-model `ir.json`; edited only if the component has to be passed in)
- **Symbols:** `exploreLoad`, `emit_with_model`, `ErrModelMismatch` — cited
- **Also likely:** `crates/edge/ess-cli/src/main.rs:5043` — inferred (`synthesize_suite` passes the full `&ir` to `write_suite`, which calls the Go emitter at main.rs:4847)
- **Also likely:** `crates/verify/ess-conformance/src/coverage.rs:88` — inferred (the suite-format-5 path passes the full `&ir` to `go::emit_input_with_model`)
- **Also likely:** `crates/verify/ess-conformance/src/ts/explore.ts` — inferred (TypeScript twin; the Acceptance names Go only)
- **Documents:** none
- **Confidence:** high for the surface; medium for the write set beyond `explore.go`
- **Would collide with:** any unit editing `go/explore.go`; any unit touching `synthesize_suite`/`write_suite` in `ess-cli` `main.rs` or the format-5 path in `coverage.rs`; any unit changing TS explorer output — inferred
- **Safety fact:** a whole-system suite carries no component, so a component filter is a no-op for the `tests/explore*.rs` callers that pass whole suites (`tests/explore.rs:157`, `:722`) — inferred
- **Not established:** the story's citation `crates/edge/ess-cli/src/main.rs:4600` is stale (the emit calls are at main.rs:4847 and 4854); whether a component-scoped suite records its component in provenance (`scenario.rs:500`); the fix route (explorer filter or scoped model digest).
- **Size:** medium, about 150-220 hand-written lines; no format version (`COVERAGE_SUITE_FORMAT = "ess-conformance/5"`, coverage.rs:15; `SCENARIO_FORMAT = "ess-scenario/6"`, member_paths.rs:52) — inferred
- **Targeted run:** ess-conformance `--test explore`, a new `--test explore_component`, and `--test mutation_component` if component emission bytes move — inferred
