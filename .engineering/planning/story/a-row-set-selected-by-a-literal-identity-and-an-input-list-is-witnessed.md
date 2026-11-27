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
revision: 3
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

## Adopter report, 2026-11-27 (0.99.9): a membership whose body has a second conjunct

A downstream product specification, on 0.99.9, reduced a refusal of two success outcomes to a model of 157 lines
(its specification is not in the store). The shape is this story's membership, with one more test in the body.

- A command takes a list of struct elements. A `when_related` guard over the stored rows of another entity has a
  `where:` of two tests of the stored row's own fields and an `exists:` over that list, whose body is an `all:` of
  four tests: an equality of an element's `String` field with the stored row's field (the membership equality), a test
  of the element's kind against a literal, `defined(...)` of an element's optional field, and a `!=` of that optional
  field against the stored row's.
- `ess verify conform synthesize` refuses `ESS-SYNTH-001` "selects rows by no equality between a String or Uuid field
  and the input or the addressed subject" on the guard's outcome, on the success outcome after it and on the
  transition that outcome moves, with an equality of exactly the kind this story's fix counts.
- Impact on the adopter: two success outcomes of two commands (the preview and the release of a versioned record)
  carry the refusal, with the transitions they move. In its whole specification on 0.99.9 the synthesis writes 493
  scenarios and 673 refusals, 512 of them `ESS-SYNTH-001`; 88 of those 512 carry this reason text. Which of the 88 hold
  this shape, and what the other 424 are, was not sorted.
- The adopter's own reading: the cause is the guard, not the list input or the each-move of the outcome. Deleting the
  guard gives the outcome a scenario; deleting the each-move, or the list input with its `distinct` guard, leaves the
  message unchanged. Their hypothesis: the equality counts only when it is the one predicate of the body, not inside
  an `all:`.

Reproduced on 0.99.9 with a neutral model (`C1`), and the hypothesis tested directly on three more (`C2` to `C4`),
committed under
`.engineering/repro/a-row-set-selected-by-a-literal-identity-and-an-input-list-is-witnessed/`, each with its
`README.md`, commands and observed output. One entity with a draft-to-staged lifecycle, one entity of entries that
the staging command writes with `affects:` and `each:` over its list input, five outcomes.

| model | body of the `exists:` | `affects:` each | scenarios | refusals |
|---|---|---|---|---|
| C1 | `all: [e.kind == tiered, e.name == name, defined(e.unit), e.unit != unit]` | kept | 4 | 3 |
| C2 | `e.name == name` | kept | 5 | 2 |
| C3 | `e.name == name` | deleted | 6 | 1 |
| C4 | `all: [e.name == name, e.kind == tiered]` | deleted | 4 | 3 |

C1 and C4 refuse the guard's outcome, the success outcome and its transition, all with the "no equality" text. C2 and
C3 do not carry that text: the equality is counted. Run and not committed: a body that is an `all:` of the equality
alone gives the C2 and C3 counts; `all: [e.name == name, e.unit != unit]`, `all: [e.name == name, e.kind == kind]`
and `all: [e.kind == tiered, e.name == name]` (each-move deleted) give the C4 counts; an `all:` around the quantifier
(`all: [kind == tiered, {exists: {..., that: e.name == name}}]`) gives the C2 and C3 counts. The guard deleted from the
C1 model writes 6 scenarios and 0 refusals; the each-move deleted from C1 (guard kept) writes 4 and 3.

The adopter's hypothesis holds, with a sharper edge: the equality is counted when it is the whole body (or an `all:` of
one), and is not counted when any other predicate sits beside it, in either order and whatever that predicate is (a
literal equality, a `!=`, an enum equality with the input; `defined(...)` was run only inside C1). Cause, read at `main` (`fd6049733`):
`scoped_at` in `crates/verify/ess-conformance/src/synthesize/row_set.rs`, the `Predicate::Exists` arm, counts the
quantifier only where `quantified.body` is itself the one `Compare` equality; a `Predicate::All` body falls to
`false`. The decision of this story, "a body that is more than one equality between the field and the element does not
[scope]", is the limit; this shape has one equality and other tests, which narrow the rows the equality selects and
leave them the scenario's own.

Fit, in the words of `.agents/skills/assessing-external-requests/SKILL.md`:

1. Need: a guard over stored rows that selects them by an element's field and narrows by the element's other fields
   gets a scenario for itself and for the outcomes after it.
2. Class: defect against this story's own fit review (`website/docs/reference/predicates.md`: "A selector needs an
   equality ... so a scenario counts only its own rows"): rows chosen by a membership equality are the scenario's own
   with or without a further test of the element.
3. Already expressible: partly. An element test can move out of the body into a second `exists:` conjunct of the
   `where:`, which changes the rule (the two tests then name different elements). No idiom keeps the meaning.
4. Fit: no new key. The counted shape widens to a body whose conjuncts include the one equality; the arrangement that
   writes the quantifier out element by element reads any body, and the listed-input attempt of this story reads only
   the equality and the literals beside it (from its doc comment). Whether the second step needs a change is part of the work.
5. Second adopter: any model that compares an element's name with a stored name and also tests the element's kind.
6. Cost: no format bump; suites change only for models with such a body (more scenarios); `ess/23` and later only.

Acceptance added by this report:

- A body of `all:` with one equality of the field and the element and any other tests counts as that equality: C1 no
  longer prints the "no equality" text on any outcome, and neither does C4. Test: the
  four models, `C1` to `C4`, with the counts of each written down (C1 is then expected to stand where C2 does, 5 and 2, its success
  outcome refused for the cause named in `story:a-state-only-an-each-move-reaches-is-arranged-for-the-refusals-of-the-entitys-other-commands`).
- A body whose conjuncts hold no such equality, or an `any:` body, keeps its refusal; below `ess/23` the model keeps its
  refusal.
- Every repository model and component keeps its synthesized bytes.

CHANGELOG: checked 2026-11-27 on `main` (`fd6049733`), `[Unreleased]` and 0.99.1 to 0.99.8: 0.99.1 counts the
membership whose body is the one equality; no entry counts a body with a second predicate.
