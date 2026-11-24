---
format: aep.planning-md/3
id: story:the-refusal-of-an-input-whose-identity-type-several-entities-share-on-a-branch-with-no-subject-names-the-forms
kind: story
status: draft
title: 'The refusal of an input whose identity type several entities share, on a branch with no subject, names the forms that say which entity: preserves: with instance: for a read, and the row-set selector by identity'
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view
revision: 2
---
## Need

The seventh report, attempt 1. A read-only command writes
`unknown-card: when_related: {via: input.card_id, exists: false}`. `demo.cards.CardId` is the
identity type of `Card` and of `Badge`, and validation refuses it:

```text
error[ESS-COMMAND-004]: `input.card_id` is `demo.cards.CardId`, the identity of `demo.cards.Badge` and `demo.cards.Card`
  `ess-domain` refuses this as `conflicting_declaration`
  help: give the entities distinct identity types, or store the input in a field of the created subject that a `references` relation carries
```

- The refusal is raised at `crates/specify/ess-domain/src/command/related_guard.rs:1726-1745`
  (`entity_or_refusal`).
- The entity is resolved at `related_guard.rs:651-678` (`related_entity`, with `input_carrier`,
  `command/related_value.rs:292-311`) and `related_value.rs:79-97` (`referenced_entity`).
- The value-read sibling, `{related: {via: input.x, field}}` on a branch with no subject, is refused
  the same way, at `command/value_expression.rs:814-889`. Its hint for that case is "this branch
  acts on no entity, so give the entities distinct identity types instead" (`:878-881`).

This is a deliberate rule. `docs/design/value-expressions.md:248-262` says that an input read on a
branch with no subject has no relation to say which entity it names, and that the remedy is
distinct identity types. `related_guard.rs:13-16` keeps `when_related` a one-hop lookup by
identity, and `docs/design/filtered-related-reads.md:39` keeps `via` out of the row-set mapping.

The hint is incomplete for a command that only reads. Its second remedy needs a created subject,
and a read creates none. Its first remedy changes another entity's identity type, which ripples
through every position typed with it. Two forms that name the entity already validate:

- `preserves: <entity>` with `instance: <field>`, with `unknown_instance: true` on the refusal. This
  is the read-only command's own form, and `story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view` makes synthesis witness it.
- the row-set selector by identity, `{entity: <entity>, where: <identity> == input.<field>,
  exists: false}`. `rd2-identity-row-set` makes synthesis witness it.

## Outcome

On a branch that acts on no entity, both hints name these forms as well as distinct identity types:

- `related_guard.rs:1741-1744` when no outcome of the command has a subject;
- `value_expression.rs:878-881`.

The refusal, its code and its message are unchanged.

## Acceptance

- **The hint names the read form.** This is the failing test. Transition: `ess specify validate`
  on a brand-free `demo.cards` model with `Card` and `Badge` sharing `CardId` and the `ReadCard`
  above. Before: the help reads as above. After: the help names, in this order:
  `preserves: demo.cards.Card` with `instance: card_id` (when the branch is meant to read that
  row); then `{entity: demo.cards.Card, where: card_id == input.card_id, exists: false}`; then
  distinct identity types. Test: `crates/specify/ess-domain/tests/shared_identity_hint.rs` (new),
  case `a_subjectless_related_guard_names_the_forms_that_say_which_entity`.
- **Every remedy it names validates.** After: the model with each named form in place of the
  `via` guard validates with no refusal. Same file, case `every_named_remedy_validates`.
- **The value read names them too.** After: `{related: {via: input.card_id, field: label}}` on a
  branch with no subject carries the same help, without the `exists: false` form, and with
  `{related: {entity, where, field}}` instead. Same file, case
  `a_subjectless_related_value_names_the_forms_that_say_which_entity`.
- **Other branches keep their hints.** A branch with a creating or addressed subject keeps the
  `references` remedy word for word. The existing tests that print it keep their verdicts.
- `task test-xtask` and `cargo test -p ess-compiler --test typed_diagnostics` pass. A changed
  diagnostic text can move pins outside the crate (`AGENTS.md`, "A new `ValidationCode` or
  diagnostic moves pins outside the crate that declares it").

Each test validates a model of one domain, in well under 5 s. The test step's bounds apply: 60 s
per test, 120 s for the step, and 10 minutes for any build-pool job.

## Fit review

1. **Need.** An author whose identity type two entities share must be told a form that names the
   entity and fits a command acting on no subject. The requester proposed `when_related` taking
   `entity:` beside `via:` (their option a).
2. **Class.** Deliberate rule with an incomplete hint. The refusal is right, and the language
   already has two forms that say which entity.
3. **Already expressible?** Yes, by the two forms above. Each becomes a complete answer once its
   synthesis story lands.
4. **Fit.** No grid cell opens. The requester's `entity:` beside `via:` is refused by
   `filtered-related-reads.md:39`, because it would be a second spelling of the row-set selector
   by identity. It is put to the operator as RD-D2 rather than dropped.
5. **Second adopter.** Any model with an entity that extends another one to one under the same key
   type.
6. **Cost.** A refusal-only change: no format, not counted under policy 3, and no new diagnostic
   code.
7. **Alternatives.** RD-D2 B, `references:` declared on an input field, and RD-D2 C, the
   requester's form, both at `ess/39`.

Covers: the seventh report, attempt 1.

## Decisions

- RD-D2: A (this story, no new surface), as default.
- Verdict: **decline the requested syntax, with the idiom; improve the hint.**

## Scope

- cited: `crates/specify/ess-domain/src/command/related_guard.rs` (`:1726-1745`)
- cited: `crates/specify/ess-domain/src/command/value_expression.rs` (`:841-882`)
- inferred: `crates/specify/ess-domain/tests/shared_identity_hint.rs` (new)
- `docs/design/value-expressions.md:262`: one sentence naming the two forms
- `CHANGELOG.md` `## [Unreleased]` `### Changed`

Size: S, about 120 changed lines.

## Depends on

- `story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view`: its hint must not name a form that synthesis refuses.
- `rd2-identity-row-set`: for the row-set form. Should rd2 slip, rd3 may land naming only the
  `preserves:` form, and rd2 then adds the second.
- The hint text is in `entity_or_refusal`. s1 of the wave of 2026-11-24 edits that same function
  (its story, "Where", `entity_or_refusal` 1660-1750, to resolve a path `via`). Land rd3 after s1,
  and keep the hint correct for a member path too.
