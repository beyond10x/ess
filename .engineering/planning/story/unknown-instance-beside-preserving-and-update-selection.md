---
format: aep.planning-md/3
id: story:unknown-instance-beside-preserving-and-update-selection
kind: story
status: draft
title: 'Under format A unknown_instance: answers beside preserving branches and every when_subject: update selection'
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#530
- provider: github
  reference: beyond10x/ess#534
- provider: github
  reference: beyond10x/ess#535
relations:
- decomposes: epic:generic-guard-precedence
- serves: vision:O2
- depends_on: story:the-precedence-rule-places-every-branch
- depends_on: story:every-guard-combination-validates-under-the-rule
- informed_by: story:read-only-command-answers-an-unknown-instance
- informed_by: story:create-or-update-selects-update-branches-by-stored-fields
- informed_by: story:refusal-above-held-state-branch-validates
revision: 1
---
## Outcome

From format A, `unknown_instance:` is a step-2 answer (`not exists(subject)` in the rule's words,
`docs/design/generic-language-features.md` § G4 "What changes", third and fourth items) on every
command whose acting branches address an instance its input names, whether they preserve, refuse
or select an update by `when_subject:`. What 0.60.0 shipped (`ess/25`) is the special cases; this
story is the residue.

Shipped in 0.60.0, and not redone here: a read-only command declares `unknown_instance:`
(https://github.com/beyond10x/ess/issues/535, `CHANGELOG.md` 0.60.0 "Changed"); a create-or-update
command selects its update branches with `when_subject:` beside a creating `unknown_instance:`
(https://github.com/beyond10x/ess/issues/530); an `external:` refusal answers before the row with
`before_row: true` (https://github.com/beyond10x/ess/issues/534). The changelog names what it left
refused, and that is the residue:

- **Create-or-preserve.** A creating `unknown_instance:` beside a `preserves:` branch on the
  identity it creates ("ensure the record exists") is refused `conflicting_declaration`: the
  pairing admits only `moves:` and `updates:`
  (`crates/specify/ess-domain/src/command/outcome_shapes.rs:292-338`, `validate_existence_pair`;
  reproduced on ess 0.60.0 at `ess/25`: "creates `demo.desk.Seat` from input `user_id` when no
  record carries it, and no branch of `demo.desk.Ensure` acts on the `demo.desk.Seat` that input
  names"). At format A it validates; a `deletes:` sibling stays refused (a create-or-delete toggle
  is not the construct, `outcome_shapes.rs:327-328`).
- **`when_subject:` as generally as `when_subject_state:`.** At format A, wherever
  `when_subject_state:` admits an `unknown_instance:` branch beside it (refusing or creating,
  `crates/specify/ess-domain/src/command/subject_state.rs:74, 159`), `when_subject:` admits it too;
  #530's create-or-update is `ess/25`'s special case (`subject_fact.rs:219-250`,
  `admits_create_or_update` at `:386-390`). The stored-row partition rule of #530 stays.
- **#534 under the renumbering.** The two #534 models (the `demo.desk.SignIn` reproduction and
  the revocation model in `story:refusal-above-held-state-branch-validates`), declared at format
  A, answer as they do at `ess/25`: `before_row:` at step 1, `unknown_instance:` at step 2. No
  other #534 residue was found in the code: `before_row:` beside `when_related:` is story 2's, its
  "declared after a later step" rule is generalised by story 3, and Entity Runtime's refusal by
  name (`crates/generate/ess-entity-runtime/src/subset.rs:318-323`) stands under answer 10.

Below format A each refusal stays, its hint naming format A. Entity Runtime keeps refusing a
creation selected by existence by name (`ExistenceSelectionUnsupported`, `subset.rs:325-333`),
and the lowered-definitions table names the create-or-preserve form.

## Acceptance

At format A a create-or-preserve command and every `when_subject:` selection beside
`unknown_instance:` that `when_subject_state:` admits validate and are witnessed on an identity no
record carries and on a stored one. Each line names the failing test written first.

- `crates/specify/ess-domain/tests/outcome_shapes.rs`:
  `format_a_admits_a_creating_unknown_instance_beside_a_preserving_branch`,
  `a_creating_unknown_instance_beside_a_deleting_branch_stays_refused`, and
  `below_format_a_create_or_preserve_keeps_its_refusal_naming_format_a`.
- `crates/specify/ess-domain/tests/subject_fact_create_or_update.rs`:
  `format_a_when_subject_admits_unknown_instance_wherever_when_subject_state_does` (each
  `when_subject_state:` + `unknown_instance:` shape the `subject_state.rs` tests admit, rewritten
  with `when_subject: {predicate: state == …}`, validates).
- `crates/verify/ess-conformance/tests/unknown_instance.rs`:
  `create_or_preserve_is_witnessed_on_an_unknown_and_a_stored_identity`: synthesis writes both
  scenarios, and the interpreted target and the generated Rust and Go behaviour pass them.
- `crates/verify/ess-conformance/tests/external_beside_held_guard.rs`:
  `issue_534_models_answer_alike_at_format_a`.
- `crates/generate/ess-entity-runtime/tests/upsert_by_existence.rs`:
  `create_or_preserve_is_refused_by_name`.
- Tests are modules of existing test binaries; no new tests/*.rs file.

## Scope

- `crates/specify/ess-domain/src/command/outcome_shapes.rs`: `validate_existence_pair`
  (`:292-338`), `names_existing` (`:434-441`), `addresses_existing` (`:448-454`).
- `crates/specify/ess-domain/src/command/subject_fact.rs:219-250, 386-399`
  (`admits_create_or_update`, `creates_on_unknown_identity`).
- `crates/verify/ess-conformance/src/synthesize.rs:11997` (`names_existing`, synthesis's own copy)
  and `crates/verify/ess-conformance/src/synthesize/existence.rs:35` (`creates_unknown`).
- `crates/verify/ess-conformance/src/interpret/execute.rs` (`creation_takes_absent`, inferred);
  `crates/generate/ess-synth/src/{rust,go}/behaviour.rs` (the creating answer beside a preserving
  branch, inferred).
- `crates/generate/ess-entity-runtime/src/subset.rs` (`EXISTENCE` row, `:325-333`).
- Docs: `docs/design/typed-literals-and-unknown-instances.md` § 2b, `CHANGELOG.md` `[Unreleased]`.
- Adversary: yes (a relaxation on the precedence order).

## Order

After `story:the-precedence-rule-places-every-branch` (format A) and
`story:every-guard-combination-validates-under-the-rule` (both edit
`crates/specify/ess-domain/src/command/subject_fact.rs`). Parallel with
`story:declaration-order-agrees-with-every-step-pair` (no shared file) and with
`story:the-language-policy-is-counted-and-cited`.
