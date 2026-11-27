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
- depends_on: story:rust-binding-wraps-a-plain-copy-into-an-optional-member
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/generate/ess-synth/src/failure.rs
- confidence: cited
  path: crates/generate/ess-synth/src/lib.rs
- confidence: inferred
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/system.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/bounded_retry.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/fixtures/refusal-policy-harness
- confidence: cited
  path: crates/generate/ess-synth/tests/refusal_policy.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/refusal_policy_runtime.rs
- confidence: inferred
  path: docs/design/conditional-binding-failure-policies.md
revision: 4
---
## Outcome

The dispatch that `ess generate synthesize --target rust` writes for a binding whose `on_failure:`
selects a policy per refusal of the invoked command (`drop: [...]`, `retry: {outcomes, attempts,
final}`, `escalate: {emits, except}`) selects the policy from the refusal the command answered,
counts the attempts of one occurrence against the policy's bound, and answers any other failure with
the fallback, as the conformance interpreter does. Such a binding no longer refuses the Rust
workspace. The Go and web targets keep their refusal (RP-D1); they follow in their own story.

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

Where it comes from: `crates/generate/ess-synth/src/failure.rs`, `fn refusal_policy` (:303), raises
`MissingRepresentation` for every binding that has `refusal_policy`, for every target but the
command-line one. `fn binding_policies` (:285) runs it and then `fn retry_bound` (:340), and is
called before any target emits: by `lib.rs::emit` (`lib.rs:494`) and by each direct entry
(`rust/mod.rs:201`, `go/mod.rs:329`, `web/mod.rs:307`). The Rust dispatch has the arms but not the
behaviour: `rust/system.rs` answers `ResolvedFailure::BoundedRetry` and `ResolvedFailure::ByRefusal`
with `unreachable!` (:859-863) and the shared `fn refused` (:1681), so the attempt counter is half
built. The tests holding the refusal are `crates/generate/ess-synth/tests/refusal_policy.rs` on the
model `crates/verify/ess-conformance/tests/fixtures/refusal-policy.yaml` (a minimal reproduction:
binding `notify-ledger` drops `wrong-state`, retries `demo.ledger.Unavailable` and `rejected` up to 3
attempts with `rejected` final, and escalates the rest with `demo.ledger.RecordEscalated`) and
`crates/generate/ess-synth/tests/bounded_retry.rs`.
`docs/design/conditional-binding-failure-policies.md` (:42-49) records it as owed: "Not done:
generated Rust, Go and web dispatch refuse a refusal-selected policy by name ... Generated-runtime
controls for duplicate escalation remain owed with the generated dispatch".

The interpreter already does it (the same page: it "selects the policy from each attempt's actual
answer, counts the occurrence's total attempts against the bound, answers an untyped port failure
with the fallback"), and the conformance suite holds one scenario per declared refusal,
`<binding>/binding/refusal/<outcome>`. No generated-Rust conformance runner exists
(`ess-conformance` runs Go and TypeScript only), so this story checks the generated dispatch through
a harness, not through the suite.

## Acceptance

All tests below run under the test budget: each case under 60 s, and no test spawns cargo
(`crates/edge/ess-xtask/tests/no_test_builds.rs`; the new file is not added to its list).

- **Synthesis, `crates/generate/ess-synth/tests/refusal_policy.rs`.** New case
  `rust_target_synthesizes_a_refusal_selected_policy`: for the fixture model,
  `synthesize_for(.., Target::Rust)`, `rust::workspace` and `rust::single_crate` return output and
  no `TargetFailure`. `every_code_target_refuses_the_selected_policy_by_name` (:29) loops over
  `[Target::Go]` only: Rust leaves the refused list, Go stays. In
  `every_direct_workspace_entry_that_delivers_bindings_refuses_the_selected_policy_by_name` the
  `rust` row goes, the `go` and `web` rows stay. `a_universal_policy_is_not_refused_for_being_selected`
  is unchanged.
- **Bounded retry, `crates/generate/ess-synth/tests/bounded_retry.rs`** (RP-D3 A):
  `every_code_target_refuses_the_bound_by_name` loops over `[Target::Go]` only, the direct-entry
  test loses its `rust` row, and a new case `rust_target_synthesizes_a_bounded_retry` holds that
  `synthesize_for(.., Target::Rust)` returns a workspace.
- **Runtime, `crates/generate/ess-synth/tests/refusal_policy_runtime.rs` (new).** It runs the
  harness binary named by `ESS_REFUSAL_POLICY_HARNESS`, built once by the new Taskfile task
  `refusal-policy-harness` in the form of `integer-spellings-harness` (`Taskfile.yml:180`): build
  `ess`, synthesize `crates/generate/ess-synth/tests/fixtures/refusal-policy-harness/model.yaml.in`
  (the fixture model with one component added, plus one binding with a plain bounded `retry:`),
  build `harness.toml` and `main.rs` from the same directory against the generated system crate,
  and copy the binary beside the archives. The variable is set where
  `ESS_INTEGER_SPELLINGS_HARNESS` is (`Taskfile.yml:81`, `:115`, `:413`) and the task runs where
  that harness's does (`:92`, `:126`, `:166`). Without the variable the test fails naming the task,
  as `integer_spellings.rs` does. The harness links a scripted command port that answers each
  attempt from a script on its standard input and prints one line per attempt (refusal, policy
  applied, attempt number, events published). Cases:
  - `rust_dispatch_selects_the_policy_per_refusal`: `wrong-state` is dropped with no publication;
    `busy` (`demo.ledger.Unavailable`) is retried; `rejected` is final at its first answer;
    `at-limit` and an untyped port failure take the fallback and publish `RecordEscalated` exactly
    once per occurrence. These are the verdicts the suite's `notify-ledger/binding/refusal/<outcome>`
    scenarios state for the fixture, with their `expect_publication_count` and
    `expect_no_publication` steps.
  - `rust_dispatch_counts_attempts_of_one_occurrence`: a third `unavailable`/`busy` answer of one
    occurrence ends its retries at the bound of 3 attempts, across the two outcomes that select the
    retry; a second occurrence starts at attempt 1.
  - `rust_dispatch_stops_a_plain_bounded_retry`: the plain bounded binding stops after its
    `attempts:` and at once on a `final:` refusal.
- No Rust synthesis prints "cannot select the policy per refusal of the invoked command" or "cannot
  stop after `attempts:`". A binding with one universal policy generates the bytes it generated:
  the byte pins of `AGENTS.md` § Gate (`ess-synth`'s `served_codec_names`, `go`, `http_binding_rust`)
  stay unchanged.
- Three mutants of the dispatch in `rust/system.rs`, each applied and the harness rebuilt with
  `task refusal-policy-harness`, turn a case of `refusal_policy_runtime.rs` red: the policy of the
  wrong refusal, a retry that ignores the count, an escalation published twice. The result is
  recorded as evidence on this story.

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
4. Fit: no new source key. It follows the interpreter's semantics; the siblings are the Go and web
   dispatch (same refusal function), left to their own story by RP-D1.
5. A second adopter: any model with a bound command that has both a final refusal and a transient
   one; `docs/design/binding-delivery-guarantees.md` motivates the attempt bound.
6. Cost: no format version (RP-D2 A keeps the count in memory). Generated Rust dispatch code and its
   runtime change; a pinned generated tree of a model with this shape appears where it was refused.
   One harness task and one harness fixture.
7. Considered: change nothing (the model keeps one policy for all refusals); per-refusal dispatch
   with attempt counts (chosen); an obligation stub for the dispatch (rejected: the interpreter
   semantics are fully specified, so there is nothing to leave underdetermined).

CHANGELOG: checked 2026-11-27 on `main` (`e29c029fab`), sections 0.99.3 to 0.99.8 and
`[Unreleased]`: no entry changes this refusal; the story is not delivered.

## Decisions

- **accept as proposed**: it passes question 4 unchanged, because it adds no surface.
- RP-D1, open. Targets in the change:
  - A (recommended): Rust only. Go and web keep the refusal and get
    `story:the-go-and-web-dispatch-select-a-binding-failure-policy-per-refusal-and-count-attempts`.
    Costs a second story and targets that are unequal until it lands; matches the need, which is
    the Rust target.
  - B: Rust, Go and web in this story. Costs acceptance rows and a harness for Go and web that no
    consumer asked for, and `go/system.rs` and `web/` in scope.
  - C: change nothing. Fails closed.
  A and B fail closed for a target not yet changed, as today.
- RP-D2, open. The attempt count when a process restarts:
  - A (recommended): in memory, one occurrence's count lives as long as its held event.
  - B: persisted with the held event. Costs a persisted field and an old-reader compatibility test
    (`AGENTS.md`, "Determinism and formats").
  A fails open to one more retry after a restart; B does not.
- RP-D3, open. A plain bounded `retry:` (`ResolvedFailure::BoundedRetry`, refused by
  `failure::retry_bound`):
  - A (recommended): it leaves the refusal in Rust in this story. A refusal-selected `retry:` is
    itself a bounded retry (`attempts:`, `final:`), so the counter this story builds is the one the
    plain shape needs, and `rust/system.rs` answers both arms through one `fn refused` today.
    Keeping it refused would leave Rust refusing the simpler shape while generating the richer one.
    Costs one harness binding, one runtime case and the `bounded_retry.rs` changes above.
  - B: it stays refused in Rust. Costs nothing here and leaves `retry_bound` refusing Rust for a
    shape whose counter exists.

## Order

- `depends_on story:rust-binding-wraps-a-plain-copy-into-an-optional-member`: both change
  `crates/generate/ess-synth/src/rust/system.rs` (that story at the binding copy, `:537`; this one at
  the dispatch, `:859-863` and `:1681`). That story is active and approved, so it lands first.
- `story:the-rust-target-writes-a-one-time-response-outcome-as-an-obligation-that-carries-its-contract-instead-of-refusing-the-workspace`
  depends on this one: both change `src/failure.rs`, `src/plan.rs` and `src/lib.rs`.

## Scope

- cited: crates/generate/ess-synth/src/failure.rs
- cited: crates/generate/ess-synth/src/lib.rs
- cited: crates/generate/ess-synth/src/rust/mod.rs
- cited: crates/generate/ess-synth/src/rust/system.rs
- cited: crates/generate/ess-synth/tests/refusal_policy.rs
- cited: crates/generate/ess-synth/tests/bounded_retry.rs
- cited: Taskfile.yml
- inferred: crates/generate/ess-synth/src/plan.rs
- inferred: crates/generate/ess-synth/tests/refusal_policy_runtime.rs
- inferred: crates/generate/ess-synth/tests/fixtures/refusal-policy-harness
- inferred: docs/design/conditional-binding-failure-policies.md
- inferred: CHANGELOG.md
