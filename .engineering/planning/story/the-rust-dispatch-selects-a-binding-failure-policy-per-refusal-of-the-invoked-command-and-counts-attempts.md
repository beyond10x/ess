---
format: aep.planning-md/3
id: story:the-rust-dispatch-selects-a-binding-failure-policy-per-refusal-of-the-invoked-command-and-counts-attempts
kind: story
status: draft
title: The Rust dispatch selects a binding's failure policy per refusal of the invoked command and counts attempts
tags:
- adopter-report
- rust-target
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-generated-code
- informed_by: story:feature-request-269
revision: 1
---
## Outcome

The dispatch that `ess generate synthesize --target rust` writes for a binding whose `on_failure:`
selects a policy per refusal of the invoked command (`drop: [...]`, `retry: {except: [...]}`,
`escalate: {emits: ..., outcomes: [...]}`) selects the policy from the refusal the command answered,
counts the attempts of one occurrence against the policy's bound, and answers any other failure with
the fallback, as the conformance interpreter does. Such a binding no longer refuses the workspace.

## Problem

A downstream control plane decided on 2026-11-27 that the Rust code `ess generate synthesize
--target rust` writes is its control plane. On ess 0.99.2 its model gets exit 1, "rust target cannot
emit this workspace", with 51 refusals of three kinds. One kind is 41 bindings whose `on_failure:`
names a refusal per policy:

```
bindings.<binding>.on_failure: this target's dispatch answers every failure with one policy and counts no attempts, so it cannot select the policy per refusal of the invoked command
```

With the other two kinds removed and this one flattened to a plain retry, the same model synthesizes
with exit 0 in 15 s (3019 capabilities, 2735 generated, 284 obligations, 0 refused, 7 crates): this
kind is 41 of the 51 refusals and the largest.

Where it comes from: `crates/generate/ess-synth/src/failure.rs`, `fn refusal_policy`, raises
`MissingRepresentation` for every binding that has `refusal_policy`, for the Rust and Go targets
(the command-line target delivers no binding). Its test is
`crates/generate/ess-synth/tests/refusal_policy.rs`
(`every_code_target_refuses_the_selected_policy_by_name`), on the model
`crates/verify/ess-conformance/tests/fixtures/refusal-policy.yaml`, a minimal reproduction.
`docs/design/conditional-binding-failure-policies.md` records it as owed: "Not done: generated Rust,
Go and web dispatch refuse a refusal-selected policy by name ... Generated-runtime controls for
duplicate escalation remain owed with the generated dispatch". A bounded `retry:` is refused for the
same reason by `fn retry_bound` ("counts no attempts"); a refusal-selected policy's retry bound is
one of its own policies, so the two meet here.

The interpreter already does it (`docs/design/conditional-binding-failure-policies.md`: it "selects
the policy from each attempt's actual answer, counts the occurrence's total attempts against the
bound, answers an untyped port failure with the fallback"), and the conformance suite already holds
one scenario per declared refusal, `<binding>/binding/refusal/<outcome>`.

## Acceptance

- `rust_dispatch_selects_the_policy_per_refusal`, in
  `crates/generate/ess-synth/tests/refusal_policy.rs`: for the fixture model,
  `synthesize_for(.., Target::Rust)` returns a workspace and no `TargetFailure`; the generated
  dispatch run against a scripted command port that answers refusal A, then B, then an untyped
  failure, applies A's policy, B's policy and the fallback, and publishes an escalation event
  exactly once per escalated refusal.
- `rust_dispatch_counts_attempts_of_one_occurrence`: a `retry` policy with a bound stops after the
  bound, across refusals of different outcomes that each select a retry, and a retry of one
  occurrence does not use another occurrence's count.
- The generated runtime passes the suite's `<binding>/binding/refusal/<outcome>` scenarios, with the
  `expect_publication_count` and `expect_no_publication` steps, through the generated-Rust
  conformance runner.
- No synthesis of the Rust target prints "cannot select the policy per refusal of the invoked
  command" for a model of declared refusals; a binding with one universal policy generates the bytes
  it generated.
- Mutations are killed by a test: the policy of the wrong refusal, a retry that ignores the count,
  an escalation published twice.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`.

1. Need: a generated Rust dispatch has to do what the language says for a binding that drops one
   refusal, retries another and escalates a third. Minimal reproduction: the fixture of
   `refusal_policy.rs`. Requester's proposal: "the Rust dispatch selects the policy per refusal and
   counts attempts".
2. Class: gap (the language construct is released since ess/22, beyond10x/ess#269; its generated
   representation is owed).
3. Already expressible: only by flattening to one policy, which moves the checked figure (which
   refusals are final) out of the specification. No idiom (skill question 3).
4. Fit: no new source key. It follows the interpreter's semantics; siblings are the Go dispatch (same
   function, same refusal) and the web target; the decision below asks whether Go
   follows in the same change.
5. A second adopter: any model with a bound command that has both a final refusal and a transient
   one; `docs/design/binding-delivery-guarantees.md` motivates the attempt bound.
6. Cost: no format version. Generated dispatch code and its runtime change; a pinned generated tree
   of a model with this shape appears where it was refused.
7. Considered: change nothing (the model keeps one policy for all refusals); per-refusal dispatch
   with attempt counts (chosen); an obligation stub for the dispatch (rejected: the interpreter
   semantics are fully specified, so there is nothing to leave underdetermined).

CHANGELOG: checked 2026-11-27 on `main` (`e29c029fab`), sections 0.99.3 to 0.99.8 and
`[Unreleased]`: no entry changes this refusal; the story is not delivered.

## Decisions

- **accept as proposed**: it passes question 4 unchanged, because it adds no surface.
- RP-D1, open. Targets in the change:
  - A (recommended): Rust, then Go in the same story, from one plan of the dispatch; costs a larger
    change, and the refusal is one function for both.
  - B: Rust only; Go keeps the refusal. Costs a second story and leaves the targets unequal.
  - C: change nothing. Fails closed.
  A and B fail closed for the target not yet changed, as today.
- RP-D2, open. The attempt count when a process restarts:
  - A (recommended): in memory, one occurrence's count lives as long as its held event.
  - B: persisted with the held event. Costs a persisted field and an old-reader compatibility test
    (`AGENTS.md`, "Determinism and formats").
  A fails open to one more retry after a restart; B does not.

## Scope

- cited: crates/generate/ess-synth/src/failure.rs
- cited: crates/generate/ess-synth/tests/refusal_policy.rs
- cited: crates/generate/ess-synth/src/rust/system.rs
- inferred: crates/generate/ess-synth/src/go/system.rs
- inferred: crates/generate/ess-synth/src/plan.rs
- inferred: docs/design/conditional-binding-failure-policies.md
- inferred: CHANGELOG.md
