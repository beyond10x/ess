---
format: aep.planning-md/3
id: story:feature-request-269
kind: story
status: proposed
title: A binding failure policy may differ per refusal
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#269
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
  path: crates/generate/ess-synth/src/failure.rs
- confidence: cited
  path: crates/generate/ess-synth/src/go/system.rs
- confidence: cited
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/system.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/web
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: cited
  path: crates/specify/ess-domain/src/binding.rs
- confidence: cited
  path: crates/specify/ess-domain/src/binding/retry.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/bounded_retry.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/delivery_context.rs
- confidence: inferred
  path: docs/design/binding-delivery-guarantees.md
revision: 27
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:16Z", actor: "human:timo", revision: 25, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

A binding's failure policy may differ per refusal of the bound command.

## Acceptance

- `on_failure` accepts a map keyed by refusal outcome with a default (`{wrong_state: drop, at-limit: escalate, default: …}`); validate refuses a key the bound command does not declare.
- Synthesis witnesses each entry; generated Rust and Go dispatch apply it.

## Origin

beyond10x/ess#269.

## Scope

Derived 2026-09-30 by `aep:story-scoper`; **cited** = read in the tree, **inferred** = a reading.

- **Declaration:** `crates/specify/ess-domain/src/binding.rs` (`RawFailure` :444, `Failure` :351, validate :1151–1170), `binding/retry.rs` (`names_refusal` :71) — cited
- **IR:** `crates/specify/ess-compiler/src/ir.rs` (`ResolvedFailure` :1945, `ResolvedBinding::on_failure` :1982), `resolve.rs` (:1202, :3922) — cited
- **Dispatch:** `crates/generate/ess-synth/src/rust/system.rs`, `go/system.rs`, `plan.rs` — cited
- **Synthesis:** `crates/verify/ess-conformance/src/synthesize.rs` (`BindingAspect::OnFailure` :10257, `fn on_failure` :10493), `synthesize/delivery_context.rs` (:695), `synthesize/bounded_retry.rs` — cited
- **Also likely:** `ess-domain/src/system.rs` (format gate), `ess-synth/src/failure.rs`, `ess-synth/src/web/`, `ess-gen/src/{asyncapi,docs,graph}.rs`, `ess-conformance/src/scenario.rs`, `schemas/generated/ess.schema.json`, `docs/design/binding-delivery-guarantees.md` — inferred
- **Confidence:** medium — the syntax (outcome vs error keys), combination with `retry:`, and a format bump are undecided
- **Would collide with (every in-epic pair `aep plan artifact waves` reports, 2026-09-30):** 229 on `ess-domain/src/system.rs`; 265 on `ess-synth/src/plan.rs`, `ess-conformance/src/synthesize.rs`; 266 on `ess-conformance/src/synthesize.rs`; 267 on `ess-conformance/src/synthesize/delivery_context.rs`, `ess-conformance/src/synthesize.rs`, `docs/design/binding-delivery-guarantees.md`; 268 on `ess-gen/src/asyncapi.rs`, `ess-gen/src/docs.rs`, `ess-gen/src/graph.rs`, `ess-synth/src/go/system.rs`, `ess-synth/src/rust/system.rs`, `ess-compiler/src/ir.rs`, `ess-compiler/src/resolve.rs`, `ess-domain/src/binding.rs`, `ess-conformance/src/synthesize.rs`, `docs/design/binding-delivery-guarantees.md`; 273 on `ess-conformance/src/synthesize.rs`, `ess-conformance/src/scenario.rs`
- **Safety fact:** every consumer reads the policy through `ResolvedBinding::on_failure()` and matches `ResolvedFailure` exhaustively, so a per-refusal policy shows as compile errors, not a silent fallback; `failure.rs:237` reads `binding.retry` directly — git grep, unproven

## Fit review

Fit review from `docs/design/review-external-requests-2026-09.md` (2026-09-30), per `.agents/skills/assessing-external-requests/SKILL.md`.

- Need: escalate one refusal of a bound command and drop another. Class: gap or convenience, unconfirmed: if the refusal is benign, the existing idiom is an accepting branch with `when_subject_state: [..]` and `preserves:` in the bound command (GP:39-50), so there is no failure to route.

## Decisions

- **defer:** `decision-blocker:per-refusal-failure-need` asks the downstream whether direct callers must still see a refusal there. If they must, the design reuses `retry.final`'s refusal naming (by outcome or error, BC:87-89) with `except:`, not a map keyed by condition kinds such as `wrong_state`.
