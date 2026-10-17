---
format: aep.planning-md/3
id: story:expression-roots-a-branch-establishes-are-readable
kind: story
status: draft
title: 'A value reads what its branch established: the addressed record''s fields, and an Optional narrowed by its own guard'
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#555
- provider: github
  reference: beyond10x/ess#556
relations:
- serves: vision:O2
- decomposes: epic:generic-expression-language
revision: 1
---
## Outcome

A value position reads every root its branch has established: once a branch's guards have
established the addressed record (an existence or `via:` read of the subject's stored reference),
`{subject: <field>}` reads its stored fields on a refusal too; and a value narrowed by its own
guard reads as its inner type: an `Optional` input in its own `exists: false` branch, a stored
`Optional` field under `defined(field)`, and `{related: {via, field}}` under
`when_related: {predicate: defined(field)}`.

Binding design: `docs/design/generic-language-features.md` (being drafted), feature 2, roots and
narrowing. Parent: `epic:generic-expression-language`.

## Fit review

Findings covered: https://github.com/beyond10x/ess/issues/555 and
https://github.com/beyond10x/ess/issues/556 (a downstream incident-ledger specification, ess 0.59.0,
`ess/24`).

1. **Need.** (a) A refusal taken after the addressed record's existence is established names that
   record's stored fields; four errors of the model cannot name their incident. Refusal:
   `ESS-COMMAND-001 "{subject: …} reads the entity an outcome acts on, and outcome not-settled acts on
   no entity"` (`crates/specify/ess-domain/src/command/value_expression.rs:1031`). (b) A value its
   own guard has proved present is typed as present; today `ESS-COMMAND-002 "… has type
   Optional<…>, and … requires …; no conversion is declared"`
   (`crates/specify/ess-domain/src/command/set_effects.rs:1543`), and adding `when: defined(x)` to an
   `exists: false` branch is refused (`ESS-COMMAND-004`). Requester's syntax (theirs): the payload
   entries as written in the issues.
2. **Class.** (a) Gap. (b) Defect against the narrowing rule's own reasoning: `ess/16` narrows an
   Optional input after a refusal of its absence (`docs/design/optional-input-narrowing.md:1-40`);
   the branch whose guard reads the value present is the same proof and is not narrowed.
3. **Already expressible?** No: the model drops the identities from its errors.
4. **Fit.** One rule, stated once in the expression language: a guard's established facts
   (existence, `defined`) narrow the roots the branch's values read. It replaces the per-position
   narrowing of `CommandSpec::narrowed_input` (`docs/design/optional-input-narrowing.md:55-56`) and
   the "acts on no entity" rule. Siblings: `sets:`, response fields and event payloads get the same
   narrowing. Targets: validation, interpreter, synthesis (the witness already holds the value
   present), Rust/Go/TypeScript generated types (the inner type), Entity Runtime lowering, diff.
5. **Second adopter.** A not-found error that echoes the optional customer id it could not find; an
   "already assigned" error that names the stored assignee.
6. **Cost.** A source format version for the new admissions; a model that narrows nothing keeps its
   IR bytes (`docs/design/optional-input-narrowing.md:79`).
7. **Alternatives.** (a) Change nothing. (b) Type the error fields `Optional`: the reporter refuses
   it, an error field that is always present would be typed as maybe-absent. (c) Narrowing by
   established facts, one rule (chosen).

## Decisions

Accept, redesigned into one narrowing rule of the expression language.

## Acceptance

- `subject_field_readable_on_a_refusal_after_existence`: the #555 shape validates and its scenario
  asserts the stored incident id in the error.
- `optional_input_narrowed_in_its_own_exists_false_branch`: the #556 (a) shape validates; the
  generated error field is the inner type.
- `stored_optional_narrowed_under_defined`: the #556 (b) shape, and its `{related:}` sibling,
  validate.
- `unnarrowed_reads_stay_refused`: an Optional read on a branch that does not establish it is
  refused with today's message.
