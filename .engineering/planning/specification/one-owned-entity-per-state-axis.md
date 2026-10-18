---
format: aep.planning-md/3
id: specification:one-owned-entity-per-state-axis
kind: specification
status: draft
title: Two independent state axes on one record are one owned entity per axis
summary: 'Decline of #559 with the idiom, validated and synthesized on 0.62.0'
refs:
- provider: github
  reference: beyond10x/ess#559
relations:
- specifies: story:an-entity-carries-a-second-state-axis
revision: 1
---
## Statement

Two independent state axes on one record are written as one owned or referenced entity per axis,
each with its own `lifecycle:`, and a command on one axis reads the other through a `when_related:`
guard over the related row's `state`. No second lifecycle key on an entity is needed, and the suite
checks the cross-axis rule.

Answers https://github.com/beyond10x/ess/issues/559, the requester's `review_lifecycle:` beside
`lifecycle:` (theirs; refused as an unknown field). Decision recorded in
`story:an-entity-carries-a-second-state-axis`: **decline, with this idiom**.

## The idiom, run

Neutral model `.engineering/repro/559/one-entity-per-axis.yaml` (`ess/26`, written for this record,
not an adopter's file): `Case` with `Detected -> Resolved -> Sealed`, `Review` with
`Pending -> Approved` referencing its case, and `Seal {case_id, review_id}` refusing unless the
named review is `Approved`:

```yaml
  - name: repro559.ledger.Seal
    input:
      - {name: case_id, type: repro559.ledger.CaseId}
      - {name: review_id, type: repro559.ledger.ReviewId}
    outcomes:
      - name: no-review
        when_related: {via: input.review_id, exists: false}
        error: repro559.ledger.NoReview
      - name: not-approved
        when_related: {via: input.review_id, predicate: state != Approved}
        error: repro559.ledger.ReviewNotApproved
      - name: sealed
        moves: repro559.ledger.Case.seal
        instance: case_id
        emits: [repro559.ledger.CaseSealed]
        payload: {repro559.ledger.CaseSealed: {case_id: input.case_id}}
      - {name: wrong-state, wrong_state: true, error: repro559.ledger.WrongState}
```

ess 0.62.0 (release binary), 2026-10-18:

```console
$ ess specify validate --path .engineering/repro/559/one-entity-per-axis.yaml
repro559 v1 — 1 file(s), valid
$ ess verify conform synthesize --path .engineering/repro/559/one-entity-per-axis.yaml --out suite.json
18 scenario(s) (0 authored), 1 refusal(s), written to suite.json
```

The 18 scenarios include `Seal/outcome/no-review`, `Seal/outcome/not-approved`,
`Seal/outcome/sealed`, `Case/transition/seal/by/Seal/sealed`, `Review/transition/approve/by/Approve/approved`
and the held-state refusals of both axes, so the cross-axis rule ("seal only with an approved
review") is a checked figure, not one the idiom moves out of the specification (policy 1 of
`docs/design/generic-language-features.md:468-473` is met). The one refusal is `ESS-SYNTH-008` on
`Seal/outcome/wrong-state` (the `arrange_related_row` family arranges no wrong-state row; an authored
scenario covers it) and is unrelated to the second axis.

## Why decline

- The reporter's own model already keeps a separate `Review` entity on its merits ("reviews come in
  rounds") and calls the request low priority (issue #559, "What the model needs").
- A second axis on one entity would touch every consumer of the held state: the precedence plan
  (`wrong_state:` and step 3 of `docs/design/generic-language-features.md` § G4), `moves:`,
  `when_subject_state:`, `ess verify diff`, the generated state machines and the explorers; one
  owned entity per axis costs none of that and composes with every existing guard.
- No second adopter has asked for independent axes; a document with a publication state and an
  independent translation state would be one, and is written the same way.

## Reply to the requester

Declined with the idiom: one entity per state axis, each with its own `lifecycle:`; a command on
one axis reads the other with `when_related: {via: <reference>, predicate: state == …}` (or
`exists: false` for a missing row). The model above validates and synthesizes on 0.62.0; the issue
can close with this record.
