---
format: aep.planning-md/3
id: story:expect-error-checks-generated-field-presence
kind: story
status: draft
title: expect_error checks that a generated error field is present
tags:
- adopter-report
- defect
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.ts
revision: 9
---
## Outcome

`expect_error` asserts the presence of every error-payload field the model declares
`{generated: true}`, as `expect_direct_response` already does, in the Rust and Go runners.

## Evidence

An adopter on ess 0.56.0: an error field such as `iss`, declared `{generated: true}`, gets no
presence check, so a target that omits it passes. On `main`, the Go runner
(`crates/verify/ess-conformance/src/go/runtime.go:2457-2467`, `payloadCarries` at `:2654-2670`)
and the Rust runner (`src/runner.rs:1758-1773`) compare only the named fields, by value;
`expect_direct_response` checks generated fields for presence (`src/go/prerequisites.go:356`).

## Acceptance

- A runner test in each language: a target whose refusal omits a generated error field fails the
  scenario naming the field; one that carries any value passes.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave W1 on b81deac78. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance` (error-field comparison in the runners and the synthesizer) — cited.
- **Files:** `src/runner.rs:1821` (`expect_error`) and `src/go/runtime.go:2613` (`expectError`, comparator `errorCarries` at `:2837`); the story's `runner.rs:1758-1773` and `payloadCarries :2654-2670` cites are stale — cited.
- **Also likely:** `src/ts/runtime.ts:4405` (same named-field-only compare via `rowMatches`); `src/scenario.rs:2519` (`ScenarioStep::ExpectError` gains a shape, as `ExpectEvent` has); `src/synthesize.rs:9370` (`expect_error` skips `{generated: true}` by its own comment) and `src/authored.rs:4263`; `src/go/mod.rs:429` (`NEWEST_ADMITTED_SUITE_MAJOR = 63`) and `src/ts/runtime.ts:7075` (`SUITE_MAJORS`) for a new suite major — inferred.
- **Documents:** none; byte-pinned `generated/` example trees would be regenerated — inferred.
- **Confidence:** high on location and gap (`synthesize.rs:9362-9369`, `scenario.rs:2531-2533`); the fix shape is inferred.
- **Would collide with:** any unit touching `ExpectError` or `expect_error` in the three runners or the synthesizer, and any unit adding a suite major (62 and 63 are taken) — inferred.
- **Premise correction:** `expect_direct_response` does not check generated fields for presence (`prerequisites.go:356` is not a presence check; direct-response presence is the declared absence policy, `prerequisites.go:132`, `:434`, `direct_response.rs:283`); the real precedent is the event `shape` (`response.rs:308` `event_shape`), which errors lack. A runner-only change cannot meet "a runner test in each language", because runners hold only the suite — cited.
- **Size:** large, about 250 lines; a new suite major (next free ordinary 64) — cited.
- **ess-conformance targets:** `error_payload_sources`, `field_presence`, `field_presence_go`, `presence_suite_versions_go`, `typescript_runtime`, `typescript_suite_versions` — inferred.
