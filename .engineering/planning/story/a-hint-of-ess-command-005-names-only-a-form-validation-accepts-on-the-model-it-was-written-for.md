---
format: aep.planning-md/3
id: story:a-hint-of-ess-command-005-names-only-a-form-validation-accepts-on-the-model-it-was-written-for
kind: story
status: draft
title: A hint of ESS-COMMAND-005 names only a form validation accepts on the model it was written for
summary: 'The missing-row hint says to write via: input.<a>.<b> where validation refuses that form as ESS-COMMAND-001 (reported on 0.99.6)'
tags:
- defect
- downstream-report
- issues-2026-11-26
relations:
- serves: vision:O2
- informed_by: story:via-and-instance-may-name-a-member-of-a-struct-input
revision: 1
---
## Need

Reported on 0.99.6 by a downstream product: a `when_related` branch written `via: input.<a>.<b>` (a member of a struct input) is refused as `ESS-COMMAND-001` (`undeclared_reference`, "`<a>.<b>` is not an input of `<command>`"), while the `ESS-COMMAND-005` (`non_exhaustive_branches`) hint of the same command says to declare a branch `when_related: {via: input.<a>.<b>, exists: false}`. The author follows the hint and meets the refusal it leads into.

## What main does today (origin/main 14248d409e)

Still so, read from source (not run):

- The hint of the missing-row refusal interpolates the `via` as the document wrote it: `crates/specify/ess-domain/src/command/related_guard.rs:1008-1011` (`declare the branch taken when it does not: when_related: {via: {via}, exists: false}`); `RelatedVia`'s `Display` gives `input.` plus the written path (`related_value.rs`, `RelatedVia::Input`).
- That refusal comes from the shape check, which does not read the specification (`related_guard.rs:748-830`, `one_related_row` at `:892-926` calling `missing_row_answers` at `:934`).
- The check that reads the specification looks the path up as one top-level input name: `entity_or_refusal` at `related_guard.rs:1672` calls `command.input_field(via)` (`crates/specify/ess-domain/src/command.rs:2891`, an exact name match), so `a.b` is not found and the refusal is `UndeclaredReference` (`:1673-1685`).
- A member path is accepted for a value source (`CommandSpec::read_input`, `command.rs:2899-2909`, from `ess/22`) and for no addressing position. `story:via-and-instance-may-name-a-member-of-a-struct-input` (active) plans it for `via:`, `all:`, `instance:`, `each.in` and `{missing:}` at `ess/38`; main has no `ess/38`, so nothing yet accepts the form the hint names.
- Not fixed on main: `related_guard.rs` last changed on 2026-11-22 and is the same at tag 0.99.6 and at the head of `main`.
- Other stories on `ESS-COMMAND-005` hints (`story:a-link-copied-from-a-related-row-s-proved-reference-is-itself-proved-so-a-command-reading-it-needs-no-dead-exists-false-branch`, `story:when-subject-reads-the-addressed-identity`) are other sites; none checks that a hint names an accepted form.

## Outcome

A hint of a diagnostic names only a form that validation accepts on that model. For the missing-row refusal that means: where the `via` is not an input of the command, the hint names the declared inputs, as the `UndeclaredReference` hint does, and does not echo the `via`. From the format that accepts a member path in `via:`, the echo is accepted and stays.

## Acceptance

- A test that every form a hint of `ESS-COMMAND-005` names validates on the model the hint was written for: for each case of the refusal (a related branch with no `exists: false` branch, a second row, a list), the hint's `when_related: {...}` is parsed, placed on the model, and `validate` exits 0 on it.
- Red on main with a struct input and `via: input.<a>.<b>`: the hint names a form that `validate` refuses as `ESS-COMMAND-001`.
- No other diagnostic's text or code moves; the list of known refusal codes is unchanged.

## Fit review

1. Need: an author who does what a diagnostic says must not be refused for it. Reporter's proposal: none beyond the report.
2. Class: defect (a hint contradicts the validation it belongs to).
3. Already expressible: not applicable.
4. Fit: the same family as the rule that a hint names the source that declares the system (`story:expression-format-hints-and-the-header-refusal-name-the-system-source`): a hint is held to what the model accepts.
5. A second adopter: any specification with a struct input whose member is an identity.
6. Cost: text only; no format step, no new code. The wider feature (accept the member path) is the story named above and is not part of this one.
7. Considered: wait for `ess/38` (rejected: below `ess/38` the hint stays wrong); drop the hint (rejected: it is right for a top-level input); the chosen outcome keeps the hint where its form is accepted.

## Decisions

- None open. Taken as a defect: a hint names only a form validation accepts.

## Scope

- `crates/specify/ess-domain/src/command/related_guard.rs` (`missing_row_answers`, `entity_or_refusal`), `crates/specify/ess-domain/tests/` (a hint test).
