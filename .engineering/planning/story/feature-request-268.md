---
format: aep.planning-md/3
id: story:feature-request-268
kind: story
status: proposed
title: A binding may react to one outcome of its source command
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#268
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: inferred
  path: crates/generate/ess-gen/src/asyncapi.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/docs.rs
- confidence: inferred
  path: crates/generate/ess-gen/src/graph.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/port.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/go/system.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/port.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/rust/system.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/binding.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/change.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/diff.rs
- confidence: inferred
  path: docs/design/binding-delivery-guarantees.md
revision: 20
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:16Z", actor: "human:timo", revision: 20, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

A binding may react to one outcome of its source command, so one branch can be bound without a separate event per branch.

## Acceptance

- A binding declares its cause as `when: {command: C, outcome: O}`; `ess specify validate` refuses an `O` that `C` does not declare, and an `O` that publishes no event.
- Synthesis emits the binding's flow scenario after `O`, and a scenario in which a sibling outcome of `C` that publishes the same event does not trigger the binding.
- The generated Rust and Go servers dispatch the binding only after `O`: the published-event log records the publishing outcome, and a mutant that dispatches on the event alone fails the sibling-outcome scenario.

## Origin

beyond10x/ess#268. Design chosen by the coordinator (2026-09-30): an outcome filter, not an event payload filter, because the downstream need is branch-specific; the generated event log gains the publishing outcome.

## Scope

Derived 2026-09-30 by `aep:story-scoper` on 1bd946d6b; **cited** = read in the tree, **inferred** = a reading. `CHANGELOG.md` and `changes/` are the coordinator's at merge and are not scope entries.

- **Declaration:** `crates/specify/ess-domain/src/binding.rs:164` (`BindingCause`), `:281` (`RawTrigger`, the `when:` block the chosen `when: {command, outcome}` cause extends) — inferred
- **Resolution:** `crates/specify/ess-compiler/src/resolve.rs:3885` (`event_binding`), `ir.rs:1804` (`ResolvedBindingCause`), `:1851` (`ResolvedBinding`) — inferred
- **Synthesis:** `crates/verify/ess-conformance/src/synthesize.rs` `bindings` :10205, `publisher` :10629, `binding_source` :10648 — inferred
- **Dispatch:** `crates/generate/ess-synth/src/rust/system.rs` (`dispatch_fn` :1172), `go/system.rs` (`dispatch_method` :1078), `rust/port.rs:298`, `go/port.rs` — inferred
- **Also likely:** `crates/verify/ess-diff/src/{diff,change}.rs`, `crates/generate/ess-gen/src/{docs,graph,asyncapi}.rs`, `schemas/generated/ess.schema.json`, `docs/design/binding-delivery-guarantees.md` — inferred
- **Confidence:** medium — syntax (`when: {command, outcome}` vs a payload filter) is undecided and decides whether the generated outbox changes shape
- **Would collide with (every in-epic pair `aep plan artifact waves` reports, 2026-09-30):** 265 on `ess-conformance/src/synthesize.rs`; 266 on `ess-conformance/src/synthesize.rs`; 267 on `ess-conformance/src/synthesize.rs`, `docs/design/binding-delivery-guarantees.md`; 269 on `ess-gen/src/asyncapi.rs`, `ess-gen/src/docs.rs`, `ess-gen/src/graph.rs`, `ess-synth/src/go/system.rs`, `ess-synth/src/rust/system.rs`, `ess-compiler/src/ir.rs`, `ess-compiler/src/resolve.rs`, `ess-domain/src/binding.rs`, `ess-conformance/src/synthesize.rs`, `docs/design/binding-delivery-guarantees.md`; 273 on `ess-conformance/src/synthesize.rs`
- **Safety fact:** the outcome is known at push time (`rust/port.rs:298`) and lost afterwards; an outcome filter changes the outbox entry shape, a payload filter does not — unproven
