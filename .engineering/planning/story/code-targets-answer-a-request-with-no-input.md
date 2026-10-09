---
format: aep.planning-md/3
id: story:code-targets-answer-a-request-with-no-input
kind: story
status: draft
title: Go and Rust synthesis answer an outcome for a request with no input
tags:
- adopter-report
- feature
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

The Go and Rust synthesis targets generate a seam for an outcome declared `input_absent: true`, so
a command that answers a request with no input at all can be synthesized instead of refused.

## Evidence

An adopter on ess 0.55.0 and 0.56.0: `ess generate synthesize` refuses such an outcome with "this
target has no seam for a request with no input at all" (`outcomes.<name>.input_absent`).
https://github.com/beyond10x/ess/issues/170 added `input_absent` to the model
(`story:absent-command-input-outcome`), not to the code targets.

On `main` the refusal is deliberate: `crates/generate/ess-synth/src/failure.rs:178-216`
(`input_absent`), raised from the shared `emit()` (`lib.rs:419`) and in every target
(`go/mod.rs:297`, `rust/mod.rs:157`, `web/mod.rs:301`, `clap/mod.rs:64`); the generated seams
decode the request into the command's input before any branch is chosen, so a request with no body
never reaches one, and treating it as `{}` would answer a different request.
`crates/generate/ess-synth/tests/absent_input.rs` holds every target to the refusal.
`ess-entity-runtime` (`src/lib.rs:2529`) refuses it as well; only the OpenAPI projection handles
it.

## Acceptance

- A design page states where the absent request is recognised (before decoding) and what each
  target's seam receives; the refusal stays for a target the design does not cover.
- A Go and a Rust synthesis test: a command with an `input_absent` outcome synthesizes, and the
  generated server answers a request with no body with that outcome while a request with `{}` is
  decoded as before.

## Decisions

- Design first: this changes the generated seam's contract in two targets.
