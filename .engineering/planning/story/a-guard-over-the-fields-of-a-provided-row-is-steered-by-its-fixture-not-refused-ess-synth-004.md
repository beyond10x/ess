---
format: aep.planning-md/3
id: story:a-guard-over-the-fields-of-a-provided-row-is-steered-by-its-fixture-not-refused-ess-synth-004
kind: story
status: draft
title: A guard over the fields of a provided row is steered by its fixture, not refused ESS-SYNTH-004, and the sibling success outcome keeps its scenario
summary: validate accepts a guard on a field of a provided row and synthesis refuses its branch and the success branch beside it on 0.99.9; the fixture names a state and a guard needs a value
tags:
- gap
- gap-reports-2026-11-28
relations:
- serves: vision:O2
- informed_by: story:a-fixture-may-be-a-row-the-deployment-already-holds
- informed_by: architecture-decision-record:a-deployment-may-provide-an-entitys-rows
revision: 2
---
## Need

A downstream meta-model specification (ess 0.99.9) has commands that refuse on a field of a row the
deployment provides (`provided: true`, `ess/33`): "the app is suspended", "the account is locked". It
turned eight such typed guards into `external:` causes so that its synthesized scenarios stay. An
`external:` cause moves a figure the specification's reader sees (which row refuses, on which
field) out of the specification, so the suite no longer checks it
(`.agents/skills/assessing-external-requests/SKILL.md`, question 3).

Neutral reproduction on 0.99.9:
`.engineering/repro/a-guard-over-the-fields-of-a-provided-row-is-steered-by-its-fixture-not-refused-ess-synth-004/system.yaml`
(`ess/37`): a provided `App` with `suspended: Boolean`, a command `CreateLink(app_id)` with the
outcomes `unknown-app` (`when_related: {via: input.app_id, exists: false}`), `suspended-app`
(`when_related: {via: input.app_id, predicate: suspended == true}`) and `linked`. `validate` says
valid; `synthesize` writes 1 scenario (`unknown-app`) and 2 refusals:

```text
refused: refusal[ESS-SYNTH-004]: outcome linking.links.CreateLink/suspended-app has no scenario `linking.links.CreateLink/outcome/suspended-app`
  it needs an instance of `linking.links.App` to change without moving, and the deployment provides its rows (`provided: true`) and the guards read fields of the row, whose values the deployment holds, so no lifecycle state of a provided row decides them
refused: refusal[ESS-SYNTH-004]: outcome linking.links.CreateLink/linked has no scenario `linking.links.CreateLink/outcome/linked`
  (the same cause)
```

Two losses from one guard: the guarded branch has no scenario, and the sibling success outcome
`linked`, which needs a row for which the guard does not hold, loses its scenario too. Without the
`suspended-app` outcome the same model synthesizes `linked` and `unknown-app`
(`.engineering/repro/a-fixture-may-be-a-row-the-deployment-already-holds/` carries the
provided-row forms).

The refusal is the decision recorded in
`architecture-decision-record:a-deployment-may-provide-an-entitys-rows` ("a guard over a provided
row's fields cannot be steered; refused by name") and in
`story:a-fixture-may-be-a-row-the-deployment-already-holds` (implemented). This story is the
follow-up that decision left open: the fixture names a state, and a guard over a field needs a
value.

## Outcome

Where a command's guard reads a field of a provided row, synthesis writes the scenario of the
guarded branch and of the sibling success branch, and the entity fixture of each scenario carries
what the guard decides on that row, so the provider is asked for a row that makes the guard hold
(or fail) and the interpreted target creates the row so. Where the guard reads a field whose
witness synthesis cannot choose, the refusal stays and names that field.

## Acceptance

- On the reproduction above, `ess verify conform synthesize` writes
  `CreateLink/outcome/suspended-app` and `CreateLink/outcome/linked` beside `unknown-app`, with 0
  refusals, and `ess verify conform run --target interpreted --report-format 2` passes all three.
- The prelude of `suspended-app` names an entity fixture of `App` whose `suspended` is `true`; the
  prelude of `linked` names one whose `suspended` is `false`.
- A target that reads another row of `App` than the one named is told so in the scenario's note,
  as for a provided row today; a target whose fixture provider cannot supply the named values
  answers `unsupported` and the run does not report a pass.
- A guard over a field whose type synthesis cannot choose a witness for keeps
  `ESS-SYNTH-004` and the help names the field.
- A model without a guard over a provided row keeps its suite bytes; the format key and suite
  version are decided with the fix and named in its pull-request line.

## Fit review

1. Need: a typed refusal on a field of a provided row keeps its scenarios. Reproduction above. The
   requester's proposal is to keep the scenarios by turning the guard into `external:`
   (theirs, a workaround).
2. Class: gap. A guard over a provided row's field validates and is refused at synthesis, with
   the success branch refused beside it.
3. Already expressible: the authored scenario (`ess-scenario/1`) the refusal's help names, one per
   branch, and `external:`, which moves the figure out of the specification (not an answer under
   question 3).
4. Fit: the interpreted target already creates the provided row by `establish_entity`; the
   fixture contract (`docs/design/pre-execution-fixture-values.md`) names an entity and a state
   and grows by what a guard on the row decides. The same question arises for a subject, an owner
   and a view's population, which the design page refuses by name; this story keeps those refused.
5. Second adopter: a ticket system that refuses assignment to a user an identity directory
   marks disabled; a payment service
   that refuses a card a vault marks expired.
6. Cost: a field in the entity fixture (suite pair, decided with the fix), a provider contract
   that can answer unsupported for values, and a note in the interpreted target; no change to a
   model's source format.
7. Considered: change nothing (today); fixture carries the guard's predicate for the provider to
   satisfy (A); fixture carries values synthesis chose (B); advise the authored scenario in
   `validate` and keep both refusals (C).

## Decisions

- FD1 (open): A, the entity fixture carries the guard's predicate over the row, the provider finds
  or makes a row for which it holds; B, the entity fixture carries field values synthesis chose
  from the guard, the provider makes or finds a row holding them; C, keep both refusals and have
  `validate` name the authored scenario at the guard. Recommended: B. A needs a predicate language
  in the provider contract of every target and runner; B reuses the witness synthesis already
  chooses for stored fields and keeps the contract to values. A and B fail closed (a provider that
  cannot supply the row answers unsupported); C keeps the refusal, which also fails closed, and
  leaves the success branch unwitnessed.

## Scope

- inferred: crates/verify/ess-conformance/src/synthesize/provided.rs
- inferred: crates/verify/ess-conformance/src/entity_fixtures.rs
- inferred: crates/verify/ess-conformance/src/fixtures.rs
- inferred: crates/verify/ess-conformance/src/interpret/provided.rs
- inferred: crates/verify/ess-conformance/tests/provided_rows.rs
- inferred: docs/design/pre-execution-fixture-values.md
