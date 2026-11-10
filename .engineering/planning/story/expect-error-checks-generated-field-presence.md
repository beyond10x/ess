---
format: aep.planning-md/3
id: story:expect-error-checks-generated-field-presence
kind: story
status: active
title: expect_error checks that a generated error field is present
tags:
- adopter-report
- defect
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
- depends_on: story:view-and-error-assertions-carry-their-value-shape
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/mod.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/response.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/runtime.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/expect_error_checks_generated_field_presence.rs
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:44:05Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:44:06Z", actor: "human:timo", revision: 16, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

`expect_error` asserts the presence of every error-payload field the model declares
`{generated: true}`, as `expect_event` does through its `shape`, in the Rust, Go and TypeScript runners.

## Problem

An adopter on ess 0.56.0: an error field such as `iss`, declared `{generated: true}`, gets no
presence check, so a target that omits it passes. Still so on 0.92.0.

The synthesizer leaves a `{generated: true}` error field out of `expect_error` by design
(`crates/verify/ess-conformance/src/synthesize.rs`, `expect_error` / `determined_fields`), and
`ScenarioStep::ExpectError` (`src/scenario.rs`) carries only `fields` to compare by value, no shape.
The runners compare only the named fields: Rust `expect_error` (`src/runner.rs`), Go `expectError` /
`errorCarries` (`src/go/runtime.go`), TypeScript `expect_error` (`src/ts/runtime.ts`). Events have the
precedent: `ExpectEvent` carries a typed `shape` (`event_shape`, `src/response.rs`), which a runner
checks for presence. (The earlier claim that `expect_direct_response` checks generated fields was
wrong: direct-response presence is the absence policy, `src/go/prerequisites.go`.)

Minimal reproduction (ess 0.92.0):

```yaml
format: ess/19
system: demo
version: v1
domain: demo.tokens
types:
  - {name: demo.tokens.TokenId, kind: newtype, of: Uuid}
errors:
  - name: demo.tokens.Rejected
    fields: [{name: reason, type: String}, {name: iss, type: String}]
events:
  - {name: demo.tokens.Issued, fields: [{name: token_id, type: demo.tokens.TokenId}]}
commands:
  - name: demo.tokens.Issue
    input: [{name: token_id, type: demo.tokens.TokenId}, {name: scope, type: String}]
    outcomes:
      - name: rejected
        when: {scope: {eq: ""}}
        error: demo.tokens.Rejected
        payload: {demo.tokens.Rejected: {reason: "empty", iss: {generated: true}}}
      - name: issued
        emits: [demo.tokens.Issued]
        payload: {demo.tokens.Issued: {token_id: input.token_id}}
```

```
$ ess verify conform synthesize --path model.yaml --target ir --out suite.json
2 scenario(s) (0 authored), 0 refusal(s), written to suite.json
```

The one `expect_error` step:
`{"step": "expect_error", "error": "demo.tokens.Rejected", "fields": {"reason": "empty"}}`. Nothing
in it requires `iss`, so a refusal carrying only `reason` passes.

## Acceptance

All cases are in `cargo test -p ess-conformance --locked --test
expect_error_checks_generated_field_presence`, over the model above.

- `ess verify conform synthesize --target ir` writes an `ess-conformance/64` suite whose
  `Issue/outcome/rejected` `expect_error` step carries the value shape of
  story:view-and-error-assertions-carry-their-value-shape with `iss` a `String` leaf that must be
  present, and still compares `reason` by value.
- The Rust runner fails `Issue/outcome/rejected` against a target whose `Rejected` omits `iss`,
  with a mismatch naming `iss`, and passes it against a target carrying any `String` in `iss`.
- The Go runner gives the same two answers on the same two targets.
- The TypeScript runner gives the same two answers on the same two targets.
- The same `expect_error` step in an `ess-conformance/63` suite passes against the target that
  omits `iss` in each of the three runners.

## Decisions

- The presence check rides on the error shape story:view-and-error-assertions-carry-their-value-shape
  adds rather than a separate presence list, so `ExpectError` gains one shape under one suite major
  (`ess-conformance/64`, coverage `65`).
- This story lands after story:view-and-error-assertions-carry-their-value-shape (`depends_on`)
  and owns the combination: it marks a `{generated: true}` error field as a present leaf of the
  shape that story introduces, and changes no comparator that story owns.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/scenario.rs
- cited: crates/verify/ess-conformance/src/runner.rs
- cited: crates/verify/ess-conformance/src/go/runtime.go
- cited: crates/verify/ess-conformance/src/ts/runtime.ts
- inferred: crates/verify/ess-conformance/src/authored.rs
- inferred: crates/verify/ess-conformance/src/response.rs
- inferred: crates/verify/ess-conformance/src/go/mod.rs
- inferred: crates/verify/ess-conformance/tests/expect_error_checks_generated_field_presence.rs

confidence: high

Re-pins the suite-bytes rows whose `expect_error` steps gain a generated field, and proposes the
`CHANGELOG.md` sentence naming them.
