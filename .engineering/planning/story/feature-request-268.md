---
format: aep.planning-md/3
id: story:feature-request-268
kind: story
status: implemented
title: A binding invokes only when its event payload condition holds
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#268
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- depends_on: story:feature-request-267
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
- confidence: cited
  path: crates/specify/ess-domain/src/selection.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts
- confidence: inferred
  path: crates/verify/ess-diff/src/change.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/diff.rs
- confidence: inferred
  path: docs/design/binding-delivery-guarantees.md
- confidence: cited
  path: docs/design/conditional-binding-failure-policies.md
revision: 33
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:16Z", actor: "human:timo", revision: 20, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-04T15:32:47Z", actor: "human:timo", revision: 32, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:00Z", actor: "human:timo", revision: 33, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Outcome

A binding invokes its command only when a finite typed predicate over the published event payload is true. The condition can prove an Optional path present for a required mapped input, covering #194. It cannot distinguish source-command outcomes that publish identical payloads; those events need a declared discriminator if consumers must tell them apart.

## Acceptance

- Source22 admits `when: {event: E, where: <finite event-payload predicate>}` for local/external event bindings; old source versions and undeclared/unsupported paths refuse. Periodic causes reject where.
- The predicate is evaluated before selection/conversion/mapping/invocation. True invokes, false skips only this binding, Unknown remains a reported unmet obligation. Existing event logs retain their shape.
- #194 is implemented by a sound presence implication from the condition to every Optional ancestor/leaf required by a mapped command input; absent values skip before mapping and unsafe sibling/child refinements refuse.
- Positive and negative scenarios observe actual command attempts. New ExpectNoInvocation in suite36/37 waits the full eventual window, refuses old readers, and fails unwanted or late attempts even when they emit no event. No ExpectQuiet event-log proxy is accepted.
- Named controls and required real native/generated Rust/generated Go, native/Go/TypeScript suite execution and browser composition are bound in docs/design/conditional-binding-failure-policies.md. Independent design review precedes implementation.

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

## Current coordinated binding contract

For the operator-authorized remaining bundle, docs/design/conditional-binding-failure-policies.md resolves the pending binding syntax, presence proof, observation, refusal alias, fallback and retry decisions. Source22 is the shared syntax allocation. New zero-invocation suite vocabulary uses ordinary36/inventory37; new predicate/refusal-policy diff kinds use diff14. Existing source21, held suites34/35, universal policies and unchanged projections retain their meanings/bytes. Historical illustrative syntax and scope doubts are superseded by that explicit contract. Independent design review is pending; this paragraph is not implementation evidence.

## Design revision 2

Review-result:conditional-binding-design-20261003-r1 is answered in docs/design/conditional-binding-failure-policies.md. Refusal policies apply only after a valid mapped input reaches the command port, where every logical attempt is counted once before the call. Untyped port failures consume this budget. Pre-input mapping/host/selection failures are explicit obligations with zero attempts and no policy/retry/escalation, avoiding an unadvanceable budget and fabricated escalation input. Escalation uses the actual complete failed input and the existing typed host builder; builder failure neither publishes nor reenters retry. Named controls cover these boundaries. Story268's Outcome/title now promise event-payload conditioning, not indistinguishable source-outcome selection. All three design findings are fixed; second independent design review remains due, implementation remains pending.

## Current design disposition

Final independent design reviews at aec396fe6 approved the arrangement/drop contract and the conditional/per-refusal contract (review-result:binding-arrangement-drop-design-20261003-r2 and review-result:conditional-binding-design-20261003-r2). All four and three first-round findings, respectively, were fixed. The matching docs/design pages now bind implementation. Prior pending-design wording is historical; implementation, decisive target controls and independent source review are still required. Serial #266 -> #267 -> #268/#194 -> #269 order and the one bundle PR remain unchanged.
