---
format: aep.planning-md/3
id: story:a-branch-arranged-around-the-other-row-below-a-predicate-over-the-addressed-row-is-written-as-a-scenario-its-model-accepts-not-refused-ess-synth-008
kind: story
status: draft
title: A branch arranged around the other row below a predicate over the addressed row is written as a scenario its model accepts, not refused ESS-SYNTH-008
tags:
- defect
- issues-2026-11-27
relations:
- serves: vision:O2
- informed_by: story:an-unknown-instance-scenario-is-sent-an-identity-no-row-carries-where-a-related-guard-reads-the-addressed-row
revision: 1
---
## Need

0.99.8 refuses, as `ESS-SYNTH-008`, a related-guard branch of a command that reads a row of another
entity through its input (`when_related: {via: input.owner_id, predicate: active == false}`) and
declares an earlier predicate over the row it addresses (`when_related: {via: input.item_id,
predicate: owner_id != input.owner_id}`). Before 0.99.8 synthesis wrote a scenario for that branch
that set up the addressed row twice, once as the subject and once as the related row, and sent the
related one; the predicate over the addressed row answered, and the scenario failed on the
interpreter; 0.99.8 fixed this with
`story:an-unknown-instance-scenario-is-sent-an-identity-no-row-carries-where-a-related-guard-reads-the-addressed-row`.
The refusal is correct for what synthesis builds and the suite loses that branch's scenario: a model
author who reads the refusal has a branch the suite does not witness.

Whether an arrangement exists is not known: the branch needs the addressed row and the other row to
be two rows, with the first predicate false for the addressed row and the second true for the other
row, and the input naming each. A hand-written scenario can do it when the predicates allow it.

## Outcome

For a branch of this shape for which a setup exists, synthesis writes a scenario that sets up the
addressed row and the other row apart, sends the input naming the other row, and passes the model's
interpreter; where no setup exists (the predicates cannot both hold), it keeps the refusal
`ESS-SYNTH-008` and says why.

## Acceptance

- On the reduced model of the story above, `ess verify conform synthesize` writes a scenario for the
  branch and `ess verify conform run --target interpreted` passes it.
- A model whose predicates cannot both hold keeps `ESS-SYNTH-008` with a help line that names the
  two predicates.
- A scenario written before this change for a branch of another shape keeps its bytes; the
  committed byte tables move only by the rows of the new fixture.

## Fit review

1. Need: a branch the model declares gets a scenario where one can exist. Reproduction above.
2. Class: gap in synthesis (a refusal where an arrangement may exist), not a language gap.
3. Already expressible: an authored scenario for the branch.
4. Fit: it extends the arrangement the related-guard path builds, which already arranges a related
   row apart from the subject for other shapes.
5. Second adopter: any command that moves a row between owners and also reads the new owner.
6. Cost: a scenario gained for the branch; pinned suites of such a model move when synthesized
   again; no format key.
7. Considered: keep the refusal (today); write the scenario (this story). Which one is right where
   the predicates can hold is open, below.

## Decisions

- SY-D1, open. A: write the scenario where a setup exists and keep the refusal where none does
  (recommended: the suite witnesses every branch it can). B: keep the refusal for the whole shape;
  cost: a branch with no scenario in the suite. B fails closed, A fails closed where no setup
  exists.

## Scope

- inferred: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- inferred: crates/verify/ess-conformance/tests/related_guard.rs
- inferred: CHANGELOG.md
