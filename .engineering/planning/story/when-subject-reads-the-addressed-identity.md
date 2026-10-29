---
format: aep.planning-md/3
id: story:when-subject-reads-the-addressed-identity
kind: story
status: implemented
title: A when_subject on the identity is refused with the idiom that replaces it
summary: 'Adopter report R3 (#643), declined: ESS-COMMAND-003 names the idiom (drop when_subject, keep unknown_instance: true).'
tags:
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command/subject_fact.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/subject_guard_input.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/subject_guard_input.rs
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T11:01:22Z", actor: "human:timo", revision: 10}
- {from: "proposed", to: "active", at: "2026-10-28T11:01:23Z", actor: "human:timo", revision: 11}
- {from: "active", to: "implemented", at: "2026-10-29T00:13:55Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Report

An adopter retrofitting an existing service (2026-10-23, ess 0.67.0, ess/29): a `when_subject` predicate naming the addressed entity's identity field (`parent_id == input.parent_id`, or `defined(parent_id)` as the stored-row test) is refused as ESS-COMMAND-003, because the identity is not a stored field; the same command draws ESS-COMMAND-005. Related: #505 (same refusal for `when_related: {via: …}`, closed).

## Fit review

1. **Need.** One command branches on "the addressed row is stored" (update) versus "it is not" (create). `parent_id == input.parent_id` and `defined(parent_id)` are the requester's syntax for that test, not the need. Minimal reproductions, kept outside the repository: the adopter shape with a default, a no-default shape, and a no-default shape with `defined(parent_id)`.

2. **Class.** Convenience. The identity selects the row, so on every row `when_subject:` sees, both predicates are true. The identity is left out by design: `crates/specify/ess-domain/src/command/subject_fact.rs:455` ("minus the identity"); `readable_fields` (subject_fact.rs:400) clones only `entity.fields`, and the identity is a separate `Field` (`crates/specify/ess-domain/src/entity.rs:985`).

3. **Already expressible.** Yes. Drop `when_subject:` from the update and keep `unknown_instance: true` on the creator. The idiom is in the `specification:a-stored-row-test-is-the-existence-step` artifact, with its command output. On 0.74.0 it validates, and `ess conform run --target interpreted` passes its three scenarios (`Mint/outcome/first`, `Mint/outcome/already`, `AddParent/outcome/added`). It is valid from `ess/16` (checked at `ess/20` and `ess/24`); at `ess/14` it is refused, naming `ess/16` for `unknown_instance:` on `creates:`. So it does not need `ess/25`: the earlier "(create-or-update, ess/25)" wording was wrong, because `ess/25` covers a `when_subject:` update beside the creator, which the idiom drops. Both branches are still witnessed, so no checked figure leaves the specification.

4. **Grids and targets.** Grid R (`docs/design/language-grids.md`) has no identity cell for `when_subject:`. Opening one would add a cell where every value is constant. The sibling `when_related` identity is a separate draft story, `story:related-identity-readable-in-when-related`. No target is affected: refused today, refused after.

5. **Second adopter.** Not needed: the meaning already exists under the existence step.

6. **Cost.** One hint branch and one skipped proof. No format, no code generation change, no IR, no ValidationCode.

7. **Alternatives.** (a) Admit the identity read: every predicate over it is constant; rejected. (b) Admit it and refuse constant predicates with a new code: same answer at higher cost; rejected. (c) Decline with the idiom: name it in the ESS-COMMAND-003 hint and stop the duplicate ESS-COMMAND-005; chosen. (d) Change nothing: two diagnostics for one cause and no pointer to the idiom; rejected.

## Decisions

**Decline, with the idiom.** "A row is stored" is the existence step: write the update without `when_subject:` and beside an `unknown_instance: true` creator. The suite witnesses both branches. The idiom is recorded as `specification:a-stored-row-test-is-the-existence-step`, which carries the idiom and the command output that shows it works.

- **003 hint.** In `check` (`crates/specify/ess-domain/src/command/subject_fact.rs`, about lines 519-537), when the refused root equals `entity.identity.name`, the hint becomes: "`<id>` is the identity that addresses this row, so on every row `when_subject:` reads it equals `input.<id>` and is defined; answer an identity no row carries with an `unknown_instance: true` branch and drop `when_subject:` from the update". The message is unchanged. The hint names no format.
- **005 suppression.** In `validate` (subject_fact.rs, about lines 178-281), a command whose `when_subject:` predicate drew an error-severity ESS-COMMAND-003 from `check` skips `validate_partition` (the call at about line 275). Such a predicate always makes the prover decline (`crates/specify/ess-domain/src/command/finite.rs`, about lines 203-209: a checker error gives `Decline::Unsupported`), so the ESS-COMMAND-005 raised about lines 607-626 only repeats the same cause.
- **Safety.** The partition is skipped only for a command that already carries an error-severity ESS-COMMAND-003, so nothing refused today becomes valid. Not yet proven by a test: the first red test must show it.
- **Changed from the earlier story text.** The hint loses "ess/25"; line citations now refer to 0.74.0.
- **Format.** None: no IR, no ValidationCode, no source format.
- **Reply to the adopter.** "Declined, with the idiom: the identity selects the row, so it always equals the input and is always defined there. Stored or not stored is the existence step: drop `when_subject:` from the update and keep `unknown_instance: true` on the creator; the suite witnesses both."

## Acceptance

In `crates/specify/ess-domain/tests/subject_guard_input.rs`:

- `an_identity_read_in_when_subject_names_the_existence_step` (first red test): both shapes (`parent_id == input.parent_id`, `defined(parent_id)`) yield exactly one error, ESS-COMMAND-003, whose hint carries the identity hint. `cargo test -p ess-domain --locked --test subject_guard_input an_identity_read_in_when_subject_names_the_existence_step`
- `a_refused_when_subject_predicate_draws_no_coverage_refusal`: the no-default shape has no ESS-COMMAND-005. `cargo test -p ess-domain --locked --test subject_guard_input a_refused_when_subject_predicate_draws_no_coverage_refusal`
- `a_non_identity_unknown_root_keeps_the_stored_fields_hint`: an unknown non-identity root keeps today's "stored fields: …" hint. `cargo test -p ess-domain --locked --test subject_guard_input a_non_identity_unknown_root_keeps_the_stored_fields_hint`

In `crates/verify/ess-conformance/tests/subject_guard_input.rs`:

- `the_identity_idiom_witnesses_the_stored_and_the_unknown_row`: the idiom validates, and both branches are synthesized and pass on the interpreted target. `cargo test -p ess-conformance --locked --test subject_guard_input the_identity_idiom_witnesses_the_stored_and_the_unknown_row`

Package gates: `cargo test -p ess-cli --locked` and `task test-xtask` (the diagnostic text changed).

## Scope

- `crates/specify/ess-domain/src/command/subject_fact.rs`: the 003 hint (about lines 519-537) and the skipped partition (about lines 178-281, call at 275). Cited.
- `crates/specify/ess-domain/tests/subject_guard_input.rs`: three tests. Cited (file exists).
- `crates/verify/ess-conformance/tests/subject_guard_input.rs`: the idiom witness test. Inferred (placement).
- `CHANGELOG.md`: Fixed entry. Inferred.
- Confidence: high. The defect site and the message are in the tree, and the defect was reproduced on 0.74.0.
- Shared with `story:defined-and-not-defined-branches-are-exhaustive` (same unit U2): both edit `crates/specify/ess-domain/tests/subject_guard_input.rs`, and that story may edit `subject_fact.rs`. Other units: unknown.
- Safety fact: the partition is skipped only for a command that already has an error-severity ESS-COMMAND-003, so nothing refused today becomes valid. Level 2, unproven: subject_fact.rs:275 and the skip at about line 219.

## Batch 1 integration

- Batch 1 runs U1, U2, U3 and U4 at once (`approval-record:ess-issues-batch-1`). This unit does not edit `CHANGELOG.md`, `changes/*.yaml` or `WHATS-CHANGED.md`: the implementor returns its changelog entry in its report, and the integration branch `integration/ess-issues-1` writes every entry once, after the last unit merges, so four entries under the empty `[Unreleased]` heading do not collide.
