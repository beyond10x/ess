---
format: aep.planning-md/3
id: story:a-success-beside-two-related-rows-held-to-one-input-is-witnessed
kind: story
status: draft
title: A success beside two related rows held to one input is witnessed
summary: Two when_related predicates that compare their rows with one input validate, and synthesis writes no scenario for the success (ESS-SYNTH-003) on 0.94.0; a consumer's suite falls one scenario under its floor
tags:
- backlog
- conformance
- defect
- external-request
- from-consumer-report
- severity-medium
relations:
- serves: vision:O2
- informed_by: story:a-command-with-two-related-selectors-arranges-both
- informed_by: story:a-row-arranged-beside-is-created-once-under-its-name
revision: 1
---
## Defect

A command reads two related rows, each named by an input, and refuses when either row's parent is
not the parent a third input names. The specification validates. Synthesis writes a scenario for
every refusal and none for the success: the one outcome where both rows belong to the named parent.

Reported by a consumer (2026-11-13) on a specification of its own: the suite then has one scenario
fewer than its floor, and the consumer keeps an `UNMAPPED` marker for the rule. Reproduced on ess
0.94.0 with a model of our own, `.engineering/repro/two-related-rows-one-input/model.yaml`: a room,
a seat and a badge that each store `room_id`, and `Pair(seat_id, badge_id, room_id)`:

```yaml
outcomes:
  - {name: no-seat, when_related: {via: input.seat_id, exists: false}, error: hall.floor.SeatNotFound}
  - {name: no-badge, when_related: {via: input.badge_id, exists: false}, error: hall.floor.BadgeNotFound}
  - {name: seat-elsewhere, when_related: {via: input.seat_id, predicate: room_id != input.room_id}, error: hall.floor.NotInRoom}
  - {name: badge-elsewhere, when_related: {via: input.badge_id, predicate: room_id != input.room_id}, error: hall.floor.NotInRoom}
  - {name: paired, creates: hall.floor.Pairing, instance: pairing_id, ...}
```

```text
$ ess specify validate --path model.yaml
hall v1 — 1 file(s), valid
$ ess verify conform synthesize --path model.yaml --out suite.json
refused: refusal[ESS-SYNTH-003]: outcome hall.floor.Pair/paired has no scenario `hall.floor.Pair/outcome/paired`
  no candidate of the 1 tried satisfies ``hall.floor.Seat` stored related row selecting this branch, over the rows 1 bounded arrangements left`
  help: write the branch's condition over values a candidate can carry, or supply a fixture for it
7 scenario(s) (0 authored), 1 refusal(s), written to suite.json
```

The success needs one arrangement: a room, a seat created with that room's id, a badge created with
the same id, and a call whose `room_id` is that id. `AddSeat` and `AddBadge` both write `room_id`
from an input, so every value is one a candidate can carry; the help does not apply.

Neighbours, both implemented and both still refusing this shape on 0.94.0:
`story:a-command-with-two-related-selectors-arranges-both` (two related rows, each compared with a
literal) and `story:a-row-arranged-beside-is-created-once-under-its-name` (guards on two related
inputs). What differs here: both rows are compared with the same input, so the two arrangements
have to share one value.

## Acceptance (sketch, to be fixed by the fit review)

- The reproduction is a conformance fixture; `Pair/paired` synthesizes, with the seat and the badge
  created in one room and the call naming that room, and passes on the interpreted target.
- The four refusals keep their scenarios; no pinned suite digest moves except the fixture's own row.
- Where no arrangement exists, the refusal names the input both rows are compared with.

## Meanwhile

The consumer's form (an input that names the parent, compared with each row) is the form this
story keeps. The refusal's help names a fixture or an authored scenario for the one outcome; we
have not tried either on the reproduction.

## Fit review

Owed. This draft records the report and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the acceptance is fixed there.
