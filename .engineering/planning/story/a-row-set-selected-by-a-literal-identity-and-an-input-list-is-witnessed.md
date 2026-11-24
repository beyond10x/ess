---
format: aep.planning-md/3
id: story:a-row-set-selected-by-a-literal-identity-and-an-input-list-is-witnessed
kind: story
status: draft
title: A row-set guard that selects by a literal identity and by membership in an input list is witnessed on every branch
summary: A row-set guard that pins a literal identity and tests membership in an input list is refused ESS-SYNTH-001 on every outcome, so synthesis writes no scenario for the command
tags:
- defect
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 2
---
## Summary

A command whose row-set guard selects rows by a literal identity and by membership of that
identity in an input list, `code == "classics"` beside
`exists: {in: input.genres, as: g, that: code == g}`, gets no scenario for any of its outcomes.
`ess verify conform synthesize` refuses each one `ESS-SYNTH-001`: the selector "selects rows by no
equality between a String or Uuid field and the input or the addressed subject". The membership is
that equality, element by element, so the refusal is wrong and synthesis owes the scenarios.

## Acceptance

- Synthesis of a model with such a guard (one entity whose `String` identity the guard pins to a
  literal and tests for membership in a `List` input, one creating command, an accepting branch
  under the guard and a default) writes a scenario for every outcome and reports no refusal.
- The accepting branch's scenario sends a list holding the literal and at least one other
  element, creates the row the literal names, and creates a decoy row holding the other element.
- The default's scenario selects no row and stands.
- The reference interpreter passes every scenario of that suite.
- A target that drops the literal conjunct fails a scenario of that suite, and so does a target
  that drops the membership conjunct.
- Below `ess/23` such a selector keeps its refusal, and every repository model and component keeps
  its synthesized bytes (slow lane `every_repository_model_keeps_its_synthesized_bytes`,
  `every_repository_component_keeps_its_synthesized_bytes`).
- `website/docs/reference/predicates.md` says that membership in an input list scopes a selector.

## Fit review

A downstream application reported it with a reproduction on 0.98.0: one command with a row-set
guard of this shape, both outcomes refused `ESS-SYNTH-001`, "1 scenario(s) (0 authored),
2 refusal(s), nothing written". It reproduces at the head of the integration branch of
2026-11-24, after that day's fixes for stored-list and input-list guards, which covered
`when_subject:` guards only.

- **Class: defect.** `website/docs/reference/predicates.md:1805`: "A selector needs an equality
  between a `String` or `Uuid` field and the input or the subject, so a scenario counts only its
  own rows." `exists: {in: input.<list>, as: x, that: <field> == x}` is an equality between the
  field and each element the scenario sends, so the rows it selects are the scenario's own.
- **Cause.** `scoped_at` in `crates/verify/ess-conformance/src/synthesize/row_set.rs` counted only
  a top-level `<field> == <input or subject>` comparison of two facts, so `decidable` refused every
  branch. Behind that refusal the arrangement could not decide the membership either:
  `set_effects::written_in` writes the input into comparisons only, so a quantifier over an input
  list stayed unknown on every row, and the plain witness sends an empty list, so no row can be
  selected.
- **Already expressible?** No. Pinning the identity to an input field changes the command's
  input. Dropping the membership changes the rule.
- **Help text.** The refusal's help, "give the field a type that has a finite value, or drop it
  from the command's input", is the generic `NoWitness` help and does not fit an unscoped
  selector. It no longer reaches this shape. Its wording for the shapes that stay refused is a
  separate question, not this story's.

## Decisions

- Membership in an input list scopes a row-set selector from `ess/23`, as the literal input and
  struct-identity members already do (`subject_fact::identity_selectors`). A document below
  `ess/23` keeps the suite and the refusals it had.
- Only an `exists` membership over an input list scopes. A membership in a subject list, a
  `forall`, or a body that is more than one equality between the field and the element does not.
- The arrangement writes such a quantifier out element by element over the list the send holds:
  `exists` becomes `any`, `forall` becomes `all`. The arrangement and the reader that decides the
  send do this alike.
- The plain input is tried first. Where the selector also pins the compared field to a literal and
  the plain input leaves the branch without an arrangement, or with no selected row, a second
  attempt sends a list holding each literal and an element that is none of them. The literal's row
  is then the selected row, and the other element's row is the decoy refuting the literal.
- A membership with no literal beside it is scoped too. Whether its accepting branch can be
  arranged then depends on the creating command steering the field to a listed element. This
  story does not add that steering.

## Scope

- `crates/verify/ess-conformance/src/synthesize/row_set.rs`: `scoped_at` (membership counts),
  new `elementwise_input`, `Listing`, `listed`, `input_toward`, `input_guards`, `reaches`,
  `listed_input`, the `Arranging::listed` attempt in `prepared` and `prepare_with`, and
  `Reader::closed`.
- `crates/verify/ess-conformance/tests/row_sets_suite.rs`: two tests, and the fixture
  `crates/verify/ess-conformance/tests/fixtures/row-set-literal-and-input-list.yaml`.
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest`: the stamp.
- `website/docs/reference/predicates.md`: one paragraph under the row-set section.
- `CHANGELOG.md`: one `### Fixed` bullet under Unreleased.
