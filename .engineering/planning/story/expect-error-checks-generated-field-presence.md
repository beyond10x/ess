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
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
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
