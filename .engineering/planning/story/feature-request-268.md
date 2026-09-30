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
revision: 24
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:16Z", actor: "human:timo", revision: 20, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

A binding may react to one outcome of its source command, so one branch can be bound without a separate event per branch.

## Acceptance

- A binding cause may carry `where:` over the event payload (`when: {event: E, where: <predicate>}`), using the existing bounded binding predicate fragment; `ess specify validate` refuses a predicate reading a field `E` does not carry.
- Synthesis emits the binding's flow scenario for a payload that satisfies `where:` and a scenario in which a payload that does not satisfy it triggers nothing (`ExpectQuiet`).
- The generated Rust and Go servers dispatch the binding only when `where:` holds; a mutant that ignores `where:` fails the second scenario. The published-event log keeps its shape.
- #194 (a binding that invokes only when an Optional path is present) is expressible with `where:` and its story says so.

## Origin

beyond10x/ess#268. The first draft chose an outcome filter; the fit review of 2026-09-30 replaced it with a payload `where:` predicate (see Decisions).

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

## Fit review

Fit review from `docs/design/review-external-requests-2026-09.md` (2026-09-30), per `.agents/skills/assessing-external-requests/SKILL.md`.

- Need: bind one branch of a command that publishes the same event from two branches. Class: convenience/gap. Red flag: an outcome filter adds a concept bindings do not have (bindings react to published events; an outcome is not on the wire and does not exist for externally delivered events, BC:240-242). Existing idiom: bindings already carry a bounded `where:` predicate fragment (BC:141-150), and #194 needs a payload condition too.

## Decisions

- **accept, redesigned (coordinator, 2026-09-30; supersedes the outcome filter):** a binding cause may carry `where:` over the event payload (`when: {event: E, where: <predicate>}`), using the existing bounded predicate fragment; this also closes #194. Where two branches publish identical payloads, the event lacks the field its consumers need, and the answer is to add it to the event, not to filter on the outcome. The generated event log keeps its shape.
