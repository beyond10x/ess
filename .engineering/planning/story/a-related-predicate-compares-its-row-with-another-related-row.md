---
format: aep.planning-md/3
id: story:a-related-predicate-compares-its-row-with-another-related-row
kind: story
status: draft
title: A related predicate compares its row with another related row
summary: A when_related predicate reads its own row, the input and the state; a rule over two related rows of one command (both belong to one parent) needs an extra input that names the parent (a consumer's report, ESS-COMMAND-003 on 0.94.0)
tags:
- backlog
- external-request
- from-consumer-report
- gap
relations:
- serves: vision:O2
- informed_by: story:a-success-beside-two-related-rows-held-to-one-input-is-witnessed
- informed_by: story:caller-attributes-are-readable-in-a-when-related-predicate
- informed_by: story:a-related-selector-compares-a-row-field-with-a-subject-field
revision: 4
---
## Need

A rule over two related rows of one command: "the seat and the badge belong to the same room", "an
order line and a voucher belong to the same order". A `when_related` predicate reads the stored
fields of its own row, the input and the row's state; it cannot name the other related row.

Reported by a consumer (2026-11-13). On ess 0.94.0, with a model of our own (the model of
`story:a-success-beside-two-related-rows-held-to-one-input-is-witnessed` without the `room_id`
input, and one refusal in place of two):

```yaml
- name: apart
  when_related: {via: input.seat_id, predicate: room_id != badge.room_id}
  error: hall.floor.NotInRoom
```

```text
error[ESS-COMMAND-003]: `badge.room_id` reads `badge`, which is not a declared observable root; `when_related` reads the stored fields of `hall.floor.Seat`, and `badge` is not one
  help: stored fields: `room_id`; the input is read as `input.<field>`, and the held lifecycle state as `state` (ess/20)
```

The requester proposed no syntax; `badge.room_id` above is our probe, not a proposal.

## What works today

An extra input that names the parent, compared with each row
(`predicate: room_id != input.room_id`, twice). It validates. It puts a value on the wire that the
server could derive, a caller can send a parent that is not its rows' parent and gets a refusal
where the rule could not be broken, and on 0.94.0 synthesis writes no scenario for the success
(`story:a-success-beside-two-related-rows-held-to-one-input-is-witnessed`).

## What the fit review has to settle

- Whether the language gets a way to name a second related row in a predicate, and how the row is
  named: by the input that selects it, by a relation, or by a name the outcome gives it.
- Whether this is the caller column's neighbour in the grid of guard positions
  (`story:caller-attributes-are-readable-in-a-when-related-predicate` opened `caller.` there), or a
  row-set selector over the second entity already states it
  (`when_related: {entity: ..., where: ...}` with a `subject.` read is
  `story:a-related-selector-compares-a-row-field-with-a-subject-field`, where the first row is the
  command's subject; here neither row is).
- What synthesis arranges for it: two rows that share a value neither the input nor a literal names.

## Acceptance (sketch, to be fixed by the fit review)

- The rule "both related rows carry one parent" is stated without an input that names the parent,
  validates, and every outcome it separates gets a scenario that passes on the interpreted target.
- The generated targets answer it alike.

## Fit review

Owed. This draft records the need and a probe only. It is an external request and it changes the
specification language: the seven questions of
`.agents/skills/assessing-external-requests/SKILL.md` are answered here, and the operator decides
before the story is accepted.

## Before acceptance (2026-11-13)

This story asks for a new construct of the language. It stays a draft until a design note with the
syntax has been put to the operator; nothing of it is built before that.

## A second consumer, 2026-11-28: a field of one related row compared with a field of another

A downstream meta-model specification (ess 0.99.9) guards a command with a comparison of a field of
one row the input names with a field of another (`a.x == b.x`), and turned the guard into an
`external:` cause so that the success outcome keeps its scenario. **Reproduced on 0.99.9** in the
form this story names: the neutral model
`.engineering/repro/a-related-predicate-compares-its-row-with-another-related-row/system.yaml`
(`ess/37`; two `Incident` rows named by `first` and `second`, `Incident.environment: String`, a
refusal that compares them) is refused at `validate`, not at synthesis:

```text
[unobservable_fact] command.ops.merge.MergeIncidents.outcomes.other-environment.when_related: `second.environment` reads `second`, which is not a declared observable root; `when_related` reads the stored fields of `ops.merge.Incident`, and `second` is not one
error[ESS-COMMAND-003]
```

`external-cause.yaml` beside it is the consumer's workaround: the same model with the comparison as
`external: the two incidents are in different environments`. It validates, synthesizes 5 scenarios
and the interpreted target passes all 5, and the comparison is checked by nothing: the
`other-environment` scenario is the forced external one. That is the idiom
`.agents/skills/assessing-external-requests/SKILL.md` (question 3) names as no answer, because a
figure the specification's reader sees (the rule) moves out of the specification.

The comparison is on an open type (`String`), so a branch per value, the idiom the section
"What works today" gives for an enum, does not reach it. The same consumer's need is the primary
story's: `story:a-guard-compares-fields-of-two-named-rows`, where both rows are read through the
input path. The two stories overlap on the construct (two related rows compared in one guard) and
differ in the spelling of the second row; whichever lands first decides the spelling of the other.
