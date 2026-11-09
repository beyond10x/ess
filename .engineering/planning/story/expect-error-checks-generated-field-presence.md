---
format: aep.planning-md/3
id: story:expect-error-checks-generated-field-presence
kind: story
status: draft
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
revision: 12
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

- Over the model above, `synthesize --target ir` writes `Issue/outcome/rejected` with an
  `expect_error` that requires `iss` to be present (a shape or presence list, under a new suite
  major; the current next free ordinary major is 64).
- A runner test in each of Rust, Go and TypeScript: a target whose `Rejected` omits `iss` fails
  `Issue/outcome/rejected` with a diagnostic naming `iss`; a target carrying any `String` value for
  `iss` passes.
- A suite in an earlier major keeps today's behaviour in all three runners.
- The byte-pinned generated example suites that move are regenerated and named in the CHANGELOG.

Overlap: `story:view-and-error-assertions-carry-their-value-shape` also gives `ExpectError` a shape
under a new suite major; the two should land in one unit or one after the other.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/scenario.rs
- cited: crates/verify/ess-conformance/src/runner.rs
- cited: crates/verify/ess-conformance/src/go/runtime.go
- cited: crates/verify/ess-conformance/src/ts/runtime.ts
- inferred: crates/verify/ess-conformance/src/authored.rs
- inferred: crates/verify/ess-conformance/src/response.rs
- inferred: crates/verify/ess-conformance/src/go/mod.rs

confidence: high
