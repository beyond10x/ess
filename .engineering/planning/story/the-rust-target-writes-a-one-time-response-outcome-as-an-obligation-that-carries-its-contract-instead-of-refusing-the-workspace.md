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
revision: 1
---
## Outcome

`ess generate synthesize --target rust` writes an outcome that declares `one_time_response:` as an
implementation obligation that carries the one-time contract (atomic durable consumption, fresh
issuance, no replay), to be written by hand, and no longer refuses the whole workspace because of
it. Every other capability of the model is generated as it is for a model without the key.

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

Where it comes from: `crates/generate/ess-synth/src/failure.rs`, `fn one_time_response`, is called
by the Rust, Go, web and command-line entry points (`rust/mod.rs`, `go/mod.rs`, `web/mod.rs`,
`clap/mod.rs`) and raises `MissingRepresentation` for every outcome that has the key. Its test is
`crates/generate/ess-synth/tests/one_time_response.rs`
(`all_implementation_entry_points_refuse_an_unimplemented_disclosure_policy`), whose model is a
minimal reproduction: one command `Issue` with one outcome `issued` returning
`one_time_response: [secret]`.

`docs/design/one-time-response-values.md`, "Projection and implementation scope", says every
projection must "preserve the policy and state its implementation/verification obligations, or
refuse it explicitly by name"; the Rust target took the second branch, and the first has no
implementation. The same page lists "implementation-owned obligations" beside the explicit
unsupported restart and concurrency cells ("Record these implementation-owned obligations"), and the
synthesis design (`docs/design/ess-structural-synthesis-obligations-realizations-design-v0.1.md`)
has the general rule "Generate it, create an explicit obligation, or refuse synthesis". The Rust
target already writes obligations as stubs (`rust/obligation.rs`, `ImplementationObligation`).

## Acceptance

- `rust_target_carries_a_one_time_response_outcome_as_an_obligation`, in
  `crates/generate/ess-synth/tests/one_time_response.rs`: for the model of that file,
  `synthesize_for(.., Target::Rust)` returns a workspace and no `TargetFailure`; its obligation
  module holds one obligation for `commands.credentials.api.Issue.outcomes.issued` whose text names
  atomic durable consumption, fresh issuance and the marked field `secret`; the obligation is listed
  in the workspace's obligation inventory with the count the plan reports.
- No synthesis of the Rust target prints "this code target cannot implement that policy".
- The generated code does not present a policy-free implementation as complete: the stub for such an
  outcome does not compile into a body that returns the response without issuing it once
  (`docs/design/one-time-response-values.md`, "Generated server/library code must not silently
  present a policy-free implementation as complete"); a test fails when the stub returns the value.
- A model without `one_time_response:` generates the bytes it generated.
- The mutation that removes the obligation (the stub emitted for a different outcome, or none) fails
  a test.

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
4. Fit: it reuses the obligation vocabulary the plan and `rust/obligation.rs` already have. Siblings:
   the Go, web and command-line targets raise the same refusal from the same function; the decision
   below asks whether they follow.
5. A second adopter: any model that issues a credential, an invitation code or a reset token has
   the same marked field.
6. Cost: no source key, no format version. A new obligation kind or text in the generated crate;
   the generated API of a workspace that was refused changes from absent to present.
7. Considered: change nothing (the downstream deletes the key and loses the policy); an in-library
   atomic store (refused: `docs/design/one-time-response-values.md` says a structural library cannot
   enforce temporal disclosure and must not claim to); the obligation (chosen).

CHANGELOG: checked 2026-11-27 on `main` (`e29c029fab`), sections 0.99.3 to 0.99.8 and
`[Unreleased]`: no entry changes this refusal; the story is not delivered.

## Decisions

- **accept, redesigned** (the fit review above): the requester wanted obligations; the design is the
  obligation the plan already knows, not a new construct.
- OT-D1, open. Which targets carry the obligation:
  - A (recommended): Rust, Go and web together, one change to `failure::one_time_response`, one test
    row per target. Costs a wider change; the three share the function and the page's parity rule.
  - B: Rust only, the others keep the refusal. Costs a second story and a target that refuses what
    its sibling generates.
  - C: change nothing. Fails closed, and keeps the whole workspace refused.
  Option C fails closed; A and B fail closed per outcome, because the obligation stub refuses to
  return the value (acceptance 3).
- The command-line target keeps its refusal under every option; it is not part of this story.

## Scope

- cited: crates/generate/ess-synth/src/failure.rs
- cited: crates/generate/ess-synth/tests/one_time_response.rs
- inferred: crates/generate/ess-synth/src/rust/obligation.rs
- inferred: crates/generate/ess-synth/src/rust/mod.rs
- inferred: crates/generate/ess-synth/src/plan.rs
- inferred: docs/design/one-time-response-values.md
- inferred: CHANGELOG.md
