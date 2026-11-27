---
format: aep.planning-md/3
id: story:the-rust-target-writes-a-one-time-response-outcome-as-an-obligation-that-carries-its-contract-instead-of-refusing-the-workspace
kind: story
status: draft
title: The Rust target writes a one-time response outcome as an obligation that carries its contract instead of refusing the workspace
tags:
- adopter-report
- rust-target
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-generated-code
- depends_on: story:the-rust-dispatch-selects-a-binding-failure-policy-per-refusal-of-the-invoked-command-and-counts-attempts
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-synth/src/failure.rs
- confidence: cited
  path: crates/generate/ess-synth/src/lib.rs
- confidence: cited
  path: crates/generate/ess-synth/src/plan.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/mod.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/one_time_response.rs
- confidence: inferred
  path: docs/design/one-time-response-values.md
revision: 5
---
## Outcome

`ess generate synthesize --target rust` writes a command whose outcome declares
`one_time_response:` with its behaviour as an implementation obligation whose contract carries the
one-time duty (atomic durable consumption, fresh issuance, no replay), to be written by hand, and no
longer refuses the whole workspace because of it. Every other capability of the model is generated
as it is for a model without the key. The Go and web targets keep their refusal (OT-D1); they follow
in their own story.

## Problem

A downstream control plane decided on 2026-11-27 that the Rust code `ess generate synthesize
--target rust` writes is its control plane. On ess 0.99.2 its model gets exit 1, "rust target cannot
emit this workspace", with 51 refusals of three kinds. One kind is 8 outcomes of `one_time_response:`:

```
commands.<command>.outcomes.<outcome>.one_time_response: one_time_response requires implementation-owned atomic durable consumption and fresh issuance; this code target cannot implement that policy
```

With this kind removed from the model, the second kind flattened to a plain retry and the third
kind's one binding dropped, the same model synthesizes with exit 0 in 15 s: 3019 capabilities, 2735
generated, 284 obligations, 0 refused, 7 crates. So these three kinds are the whole gap, and this
one is 8 of the 51 refusals.

Where it comes from: `crates/generate/ess-synth/src/failure.rs`, `fn one_time_response` (:260),
raises `MissingRepresentation` for every outcome that has the key, for every target. It is called
by `lib.rs::emit` (`lib.rs:490`) before any target emits, and by each direct entry
(`rust/mod.rs:195`, `go/mod.rs:323`, `web/mod.rs:303`, `clap/mod.rs:65`). Its test is
`crates/generate/ess-synth/tests/one_time_response.rs`
(`all_implementation_entry_points_refuse_an_unimplemented_disclosure_policy`, :31), whose model is a
minimal reproduction: one command `credentials.api.Issue` with `response: [secret]` and one outcome
`issued` returning `one_time_response: [secret]`.

The plan already owes that command's behaviour. `SynthesisPlan::of` is target-independent and
`determined::command` (`determined.rs:55`) keeps every command with a typed `response:` an
obligation; `one_time_response:` names response fields, so every outcome that has the key sits in a
command whose behaviour is an obligation. Run on 0.99.2, the refusal document
(`ess-target-failure/1`) of the test model embeds that plan: 2 capabilities,
`command_contract` generated and `command_behavior` an obligation of class `undetermined`
("a typed response (`response:`)"), whose contract does not mention the one-time duty. The Rust
target writes each plan obligation as a stub that returns `UnmetObligation` and carries the
contract in its documentation (`rust/obligation.rs`), never the value.

`docs/design/one-time-response-values.md` (:219-225) says every projection must "preserve the
policy and state its implementation/verification obligations, or refuse it explicitly by name", and
"Generated server/library code must not silently present a policy-free implementation as complete";
the Rust target took the refusal branch.

## Acceptance

- **`crates/generate/ess-synth/tests/one_time_response.rs`, new case
  `rust_target_carries_a_one_time_response_outcome_as_an_obligation`.** For the model of that file,
  `synthesize_for(.., Target::Rust)`, `rust::workspace` and `rust::single_crate` return output and
  no `TargetFailure`. The plan's counts are exactly 1 generated, 1 obligation, 0 refused: the
  command contract of `credentials.api.Issue` is generated, and its command behaviour is the one
  obligation, class `undetermined`, as today; no second obligation is added. That obligation's
  contract names `issued`, the field `secret`, atomic durable consumption, fresh issuance and no
  replay. The generated stub of that behaviour carries the same text and returns `UnmetObligation`;
  the case reads the generated source and fails when the stub's body names `secret` or builds the
  response.
- **`all_implementation_entry_points_refuse_an_unimplemented_disclosure_policy` (:31)** loops over
  `[Target::Go, Target::Web, Target::Clap]`, and its direct-entry list keeps `go::workspace`,
  `web::workspace` and `clap::workspace`: Rust leaves both lists (`rust::workspace`,
  `rust::single_crate`), the others stay refused with the same text.
  `ordinary_direct_returns_keep_their_existing_implementation_path` and
  `direct_browser_catalog_refuses_dispatch_without_losing_policy_location` are unchanged.
- No synthesis of the Rust target prints "this code target cannot implement that policy".
- A model without `one_time_response:` generates the bytes it generated: the byte pins of
  `AGENTS.md` § Gate stay unchanged, and so does its plan.
- **New case `the_one_time_clause_names_only_its_outcome`, same file**, on a second model
  `SIBLINGS` beside `MODEL`: `credentials.api.Issue` keeps `issued` with `one_time_response:
  [secret]` and gains a second outcome without the key (a declared refusal `denied` with its error
  type), and a second command `credentials.api.Rotate` has `response: [{name: secret, type:
  String}]` (the same field name) and one outcome `rotated` without the key. The plan has exactly
  2 obligations (the two behaviours) and 0 refused; the clause sits in `Issue`'s contract, naming
  `issued` and not `denied`, and `Rotate`'s contract and stub hold no clause.
- Mutants killed: the clause written on every outcome of the command or on every command whose
  response has the field (`the_one_time_clause_names_only_its_outcome`), the clause dropped
  (`rust_target_carries_a_one_time_response_outcome_as_an_obligation`), and the Rust refusal
  skipped while the clause is missing (the same case).

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`.

1. Need: a model whose commands issue a one-time secret must still get its other 2,700-odd
   capabilities generated in Rust, with the one-time issuance owned by hand code. Minimal
   reproduction: the model in `crates/generate/ess-synth/tests/one_time_response.rs`. Requester's
   proposal: such outcomes "become obligations that carry the one-time contract, written by hand".
2. Class: gap. The language states the policy; the Rust target has no representation for it, and
   the only alternative a model has is to delete the key, which moves a checked figure out of the
   specification.
3. Already expressible: no. Removing `one_time_response:` loses the disclosure scenarios
   (`docs/design/one-time-read-cells-for-other-actors.md`) and the policy itself, so it is no idiom
   (skill question 3).
4. Fit: it reuses the obligation the plan already holds for the command's behaviour; no new
   construct. Siblings: the Go, web and command-line targets raise the same refusal from the same
   function; OT-D1 leaves them to their own story.
5. A second adopter: any model that issues a credential, an invitation code or a reset token has
   the same marked field.
6. Cost: no source key, no format version (OT-D2 A). The contract text of such a command grows a
   clause; the generated API of a workspace that was refused changes from absent to present.
7. Considered: change nothing (the downstream deletes the key and loses the policy); an in-library
   atomic store (refused: `docs/design/one-time-response-values.md` says a structural library cannot
   enforce temporal disclosure and must not claim to); the obligation (chosen).

CHANGELOG: checked 2026-11-27 on `main` (`e29c029fab`), sections 0.99.3 to 0.99.8 and
`[Unreleased]`: no entry changes this refusal; the story is not delivered.

## Decisions

- **accept, redesigned** (the fit review above): the requester wanted obligations; the design is the
  obligation the plan already holds, not a new construct.
- OT-D1, open. Which targets stop refusing:
  - A (recommended): Rust only. `failure::one_time_response` skips `Target::Rust`, so `lib.rs:490`
    and `rust/mod.rs:195` pass; Go and web keep the refusal and get
    `story:the-go-and-web-targets-carry-a-one-time-response-outcome-as-the-command-behaviour-obligation-instead-of-refusing`.
    Costs a second story and targets that are unequal until it lands; matches the need, which is
    the Rust target.
  - B: Rust, Go and web in this story. Costs acceptance rows for Go's stubs and the browser
    catalog, which no consumer asked for.
  - C: change nothing. Fails closed, and keeps the whole workspace refused.
  C fails closed; A and B fail closed per outcome, because the stub returns `UnmetObligation`,
  never the value.
- The command-line target keeps its refusal under every option; it is not part of this story.
- OT-D2, open. The format consequence. The plan is target-independent: `plan.json` in every
  generated tree, the plan embedded in `ess-target-failure/1`, and the plans `ess-service-contract`
  and `ess-entity-runtime` read are all `SynthesisPlan::of`, and an obligation's reason is a tagged
  enum (`class`).
  - A (recommended): no new reason. The one-time duty is a clause of the contract string of the
    command behaviour's existing obligation (`plan.rs`, `behavior_contract`). No key, class,
    capability kind or value set changes, so no format version (`AGENTS.md` § Determinism and
    formats). Cost: the contract string of a command with `one_time_response:` changes for every
    target and in every reader of the plan; no committed tree holds such a model today, because
    every code target refuses it; and a reader learns the duty from prose, not from a class it can
    switch on.
  - B: a new reason class `one_time_response` (or a new capability kind per outcome). A new value of
    a persisted tagged enum changes names and meaning: a new format version of the plan, which
    carries no version key of its own today, so the envelope gains one or `ess-target-failure/1`
    moves on, with an old-reader compatibility test. It also replaces the `undetermined` reason the
    behaviour has, or makes the count 2.
  - C: an obligation only in the Rust output, plan unchanged. Breaks the plan-to-stub bijection
    `rust/obligation.rs` holds; refused.
  What the Go and web plans carry: under A, the same plan with the same contract clause, embedded
  in their unchanged refusal documents until their story lands; under B, the new class in the same
  places.

## Order

- `depends_on story:the-rust-dispatch-selects-a-binding-failure-policy-per-refusal-of-the-invoked-command-and-counts-attempts`:
  both change `crates/generate/ess-synth/src/failure.rs`, `src/plan.rs` and `src/lib.rs`
  (`emit` calls both refusals, `lib.rs:490` and `:494`), `src/rust/mod.rs` (`:195`, `:201`) and
  `CHANGELOG.md`.

## Scope

- cited: crates/generate/ess-synth/src/failure.rs
- cited: crates/generate/ess-synth/src/lib.rs
- cited: crates/generate/ess-synth/src/rust/mod.rs
- cited: crates/generate/ess-synth/src/plan.rs
- cited: crates/generate/ess-synth/tests/one_time_response.rs
- inferred: docs/design/one-time-response-values.md
- inferred: CHANGELOG.md
