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
- depends_on: story:feature-request-268
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
revision: 31
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:16Z", actor: "human:timo", revision: 25, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

A binding's failure policy may differ per refusal of the bound command.

## Acceptance

- Source22 admits policy-keyed selectors: drop/retry/escalate, each with outcomes or except, with list shorthand for positive drop/unbounded retry. Escalation still requires emits; retry retains attempts/final. Exactly one explicit except fallback covers untyped failures. The complete normative grammar is docs/design/conditional-binding-failure-policies.md.
- Resolve outcome/error aliases before checking disjoint/exhaustive refusal coverage. Unknown names, accepting outcomes, selector overlap and invalid final subsets refuse. No condition-kind-keyed map or implicit fallback is admitted. Old universal policies preserve bytes and behavior.
- Actual refusal chooses the policy after every attempt; mixed refusals never reset total retry budget. False binding conditions remain zero-invocation skips. Drop/escalate/final/exhaustion all terminate as specified.
- Synthesis witnesses each forceable selected refusal and named runtime controls exercise mixed-refusal and untyped-failure paths. Required native/generated Rust/generated Go and native/Go/TS suite controls kill incorrect policy, omission, extra retry and duplicate-escalation behavior. Unsupported is not successful acceptance.
- Typed IR, diff14 vocabulary, old-reader controls, projections and full acceptance names are bound in docs/design/conditional-binding-failure-policies.md. Independent design review precedes implementation.

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

- **accept, redesigned (coordinator, 2026-09-30; the downstream answered the blocker: direct callers must still see the refusal):** the binding's failure policy may name refusals of the bound command the way `retry.final` already does (by outcome or by error, BC:87-89), with `except:` as in `outcome_groups:`, e.g. `on_failure: {escalate: {except: [<outcome>]}, drop: [<outcome>]}`; no map keyed by condition kinds such as `wrong_state`, and no mixing of outcome names with the policy keywords. Its syntax ships in the same format version as #229 and #268.

## Current coordinated binding contract

For the operator-authorized remaining bundle, docs/design/conditional-binding-failure-policies.md resolves the pending binding syntax, presence proof, observation, refusal alias, fallback and retry decisions. Source22 is the shared syntax allocation. New zero-invocation suite vocabulary uses ordinary36/inventory37; new predicate/refusal-policy diff kinds use diff14. Existing source21, held suites34/35, universal policies and unchanged projections retain their meanings/bytes. Historical illustrative syntax and scope doubts are superseded by that explicit contract. Independent design review is pending; this paragraph is not implementation evidence.

## Design revision 2

Review-result:conditional-binding-design-20261003-r1 is answered in docs/design/conditional-binding-failure-policies.md. Refusal policies apply only after a valid mapped input reaches the command port, where every logical attempt is counted once before the call. Untyped port failures consume this budget. Pre-input mapping/host/selection failures are explicit obligations with zero attempts and no policy/retry/escalation, avoiding an unadvanceable budget and fabricated escalation input. Escalation uses the actual complete failed input and the existing typed host builder; builder failure neither publishes nor reenters retry. Named controls cover these boundaries. Story268's Outcome/title now promise event-payload conditioning, not indistinguishable source-outcome selection. All three design findings are fixed; second independent design review remains due, implementation remains pending.
