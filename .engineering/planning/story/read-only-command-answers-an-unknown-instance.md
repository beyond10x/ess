---
format: aep.planning-md/3
id: story:read-only-command-answers-an-unknown-instance
kind: story
status: draft
title: A read-only command declares its answer for an unknown instance
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 2
---
## Outcome

A command whose branches address an instance from its input only through `preserves:` (or a
`when_subject:` refusal) can declare an `unknown_instance:` branch, so a read-only command defines
its answer for an identity no row carries, and synthesis writes the scenario that sends one.

Closes https://github.com/beyond10x/ess/issues/535.

## Evidence

- https://github.com/beyond10x/ess/issues/535 (ess 0.59.0, ess/24): `demo.desk.Ring` with
  `no-seat: {unknown_instance: true, error: demo.desk.NoSeat}`, a `when_subject` refusal and a
  `preserves: demo.desk.Seat` branch is refused `unreachable_branch` ("no branch … acts on an
  instance its input names"). The implementation answers an unknown identity with its own error;
  the only form left is an `external:` branch that nothing ties to a missing row.
- An earlier adopter report (ess 0.56.0, neutral repro): `demo.token.Introspect(token)`, every
  outcome preserving the token, an `unknown_instance:` outcome returning `active: false`, refused
  the same way.
- Still on `origin/main` (260a3d1e3, read 2026-10-13): `names_existing`
  (`crates/specify/ess-domain/src/command/outcome_shapes.rs:434-441`) counts only `Moves`,
  `Updates` and `Deletes` with a command-input instance; the refusal is
  `outcome_shapes.rs:466-480`. Synthesis has its own copy, `names_existing` in
  `crates/verify/ess-conformance/src/synthesize.rs:11555-11567`, with the same three effects, and
  the unknown-instance witness filters on it (`synthesize.rs:11603`, `:11683`, `:11718`, `:11943`).

## Acceptance

Each line names the failing test written first.

- `crates/specify/ess-domain/tests/outcome_shapes.rs`,
  `a_read_only_command_declares_an_unknown_instance_answer`: the issue's `Ring` command validates;
  `a_command_naming_no_input_instance_still_refuses_unknown_instance` keeps today's
  `unreachable_branch` for a command with no input-addressed branch at all.
- `crates/verify/ess-conformance/tests/unknown_instance.rs`,
  `a_read_only_command_is_witnessed_on_an_identity_no_record_carries`: synthesis writes one
  scenario for `Ring` that sends an identity nothing stores and expects `no-seat` with
  `demo.desk.NoSeat`.
- `crates/verify/ess-conformance/tests/interpreted_not_found.rs`,
  `the_interpreter_answers_a_read_only_command_on_an_unknown_identity`: the interpreted target
  answers that scenario as declared; the Rust and Go synthesized targets pass it (same file or
  `unknown_instance_not_found.rs`, whichever already runs those targets).
- A preserving command without `unknown_instance:` is unchanged: its scenarios are byte-identical
  to `origin/main`'s (an existing fixture in `unknown_instance.rs` keeps its pinned output).

## Scope

- cited: `crates/specify/ess-domain/src/command/outcome_shapes.rs` (`names_existing`,
  `validate_command`); `crates/verify/ess-conformance/src/synthesize.rs` (`names_existing` and its
  callers); tests named above.
- inferred: `crates/verify/ess-conformance/src/interpret/execute.rs` (the absent-row answer for a
  preserving command); `crates/generate/ess-synth/src/{rust,go}/behaviour.rs` (the generated
  behaviour's not-found answer for a preserving command); `docs/design/typed-literals-and-unknown-instances.md`
  (the rule's wording); a CHANGELOG `[Unreleased]` "Changed" line.
- `validate_existence_pair` (`outcome_shapes.rs:292-338`) also calls `names_existing`; widening
  the shared helper would admit a creating `unknown_instance:` paired with a `preserves:` branch.
  Keep that pairing rule on moves/updates only (it already re-checks the effect at `:329-334`).

## Plan

- depends_on: none.
- Shares `synthesize.rs` with no other wave-l story; shares nothing with the explorer stories
  unless the Go explorer's `exploreDecide` unknown path needs the same widening (inferred: it reads
  `unknown_instance` from the IR, `explore.go:1444`), in which case it lands after
  `story:go-explorer-draws-read-only-commands`.
- du estimate: 10G (ess-domain, ess-conformance one `--test` at a time, ess-cli validation pins).
- adversary: no (a validation widening with a pinned old behaviour).
- Separate from `story:refusal-above-held-state-branch-validates` (branch precedence).
