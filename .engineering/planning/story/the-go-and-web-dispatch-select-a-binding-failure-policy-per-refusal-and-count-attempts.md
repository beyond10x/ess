---
format: aep.planning-md/3
id: story:the-go-and-web-dispatch-select-a-binding-failure-policy-per-refusal-and-count-attempts
kind: story
status: draft
title: The Go and web dispatch select a binding's failure policy per refusal of the invoked command and count attempts
tags:
- rust-target-follow-up
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-generated-code
- depends_on: story:the-rust-dispatch-selects-a-binding-failure-policy-per-refusal-of-the-invoked-command-and-counts-attempts
scope:
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: crates/generate/ess-synth/src/failure.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/system.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/web
- confidence: inferred
  path: crates/generate/ess-synth/tests/bounded_retry.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/refusal_policy.rs
revision: 2
---
## Outcome

The dispatch the Go and web targets write for a binding whose `on_failure:` selects a policy per
refusal of the invoked command, or bounds a retry, selects the policy from the refusal the command
answered and counts the attempts of one occurrence, as the Rust dispatch does after
`story:the-rust-dispatch-selects-a-binding-failure-policy-per-refusal-of-the-invoked-command-and-counts-attempts`.
Such a binding no longer refuses the Go workspace or the web app.

## Problem

`crates/generate/ess-synth/src/failure.rs`, `fn refusal_policy` and `fn retry_bound`, refuse both
shapes for every target that delivers bindings (`go/mod.rs:329`, `web/mod.rs:307`, `lib.rs:494`).
Decision RP-D1 of the Rust story keeps that refusal for Go and web and files this story, so the
targets are unequal until it lands. `docs/design/conditional-binding-failure-policies.md` records
the generated dispatch of every target as owed.

## Acceptance

- `crates/generate/ess-synth/tests/refusal_policy.rs` and `bounded_retry.rs`: Go leaves the refused
  lists, and the direct-entry tests lose their `go` and `web` rows.
- A Go harness built once by a Taskfile task and read through an `ESS_*` variable, in the form of the
  Rust story's harness, gives the verdicts the suite's `<binding>/binding/refusal/<outcome>`
  scenarios state for `refusal-policy.yaml`, counts attempts per occurrence, and publishes an
  escalation once. No test spawns cargo or a build.
- The web target either dispatches the same verdicts or keeps a refusal that names the binding, as
  this story's decision says.

## Decisions

- Origin: RP-D1 A of the Rust story. Open until that decision is taken; archived if B is taken.
- Whether the web target dispatches bindings itself or keeps its refusal: open, decided when this
  story is refined.

## Scope

- inferred: crates/generate/ess-synth/src/failure.rs
- inferred: crates/generate/ess-synth/src/go/system.rs
- inferred: crates/generate/ess-synth/src/web
- inferred: crates/generate/ess-synth/tests/refusal_policy.rs
- inferred: crates/generate/ess-synth/tests/bounded_retry.rs
- inferred: Taskfile.yml
