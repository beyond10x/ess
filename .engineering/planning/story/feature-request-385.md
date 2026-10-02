---
format: aep.planning-md/3
id: story:feature-request-385
kind: story
status: active
title: Generated behavior guidance states the canonical guard precedence
refs:
- provider: github
  reference: beyond10x/ess#385
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/behaviour.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/declared_behaviour.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T09:37:08Z", actor: "human:timo", revision: 6, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T09:37:08Z", actor: "human:timo", revision: 7, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

Generated plan prose and Rust/Go comments explain outcome precedence consistently with the synthesized conformance contract, without implying unsupported generated related guards.

## Fit review

1. Need: an implementor reading PLAN.md must choose missing-related-record refusal before input refusal where the canonical selector requires it. Issue385 reports declaration-order guidance and the comment before anything else is read.
2. Class: documentation defect. docs/design/cross-record-and-stored-field-guards.md:605-617 defines conditional precedence; plan.rs:673-703 currently lists declarations, Rust behaviour.rs:839 and Go behaviour.rs:1378 use the misleading comment.
3. Existing expression: when, when_related absent and existing_instance already express the model; declared_behaviour.rs:218-223/:303 and adversary_go_behaviour_pass2.rs:455 cover related-guard obligations. No new surface is needed.
4. Fit: explain the exact conditional order. Existing-instance handling on related-guard commands and related absence precede input refusals; ordinary accepting/external guards retain their declared-order rules. Do not blindly sort every related branch first. Related guards currently leave the command owed (determined.rs:148); emitted Rust/Go comments must describe only their supported subset, e.g. before the addressed subject is loaded. Label declaration-order inventories as such.
5. Second adopter: an order command can see both a missing customer and an invalid amount; hand-written behavior needs the canonical missing-customer precedence.
6. Cost: generated prose bytes/snapshots change, no runtime API or authored format. Inspect Markdown and JSON plan contracts and keep target-independent output consistent.
7. Alternatives: retain ambiguous declaration order; incorrectly reorder every related guard; selected canonical explanation plus precisely scoped generated comments.

## Decisions

Accept as proposed, using the binding design's actual conditional order. This does not add generated when_related behavior or change selection semantics.

## Acceptance

- plan_contract_states_related_absence_before_input_refusal.
- plan_contract_distinguishes_related_presence_from_absence.
- plan_contract_states_existing_instance_exception_for_related_commands.
- generated_refusal_comments_do_not_claim_universal_read_precedence.
- The related-guard command remains an obligation; plan projections stay consistent across targets.

## Scope

Cited: crates/generate/ess-synth/src/plan.rs; src/rust/behaviour.rs; src/go/behaviour.rs; tests/declared_behaviour.rs and Go behavior tests. Existing design is the authority, not a duplicated new definition.

## Verification 2026-10-02

Committed259b49a0efe5f312f41218853594222ca2d496a9 after independent source/test review against the binding precedence design. Meaningful red16cases:12pass/4fail; treatment16passed. Strict all-target Clippy/fmt/diff checks passed. Four frozen hashes verified and bot author/committer confirmed. Full server-group package and projection verification now running for316/379/385 together; no remote publication of these fixes yet. Evidence: managed tree ess-backlog-servers-20261002, target/backlog-input/385-report.md and review-result:consumer-precedence-385-pass1-20261002.
