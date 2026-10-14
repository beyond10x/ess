---
format: aep.planning-md/3
id: story:an-entity-carries-a-second-state-axis
kind: story
status: draft
title: Two independent lifecycles on one entity (proposed decline, with the idiom)
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#559
relations:
- serves: vision:O2
revision: 1
---
## Outcome

Recorded request: two independent state axes on one entity, each with its own transitions, and
guards that name the axis. Proposed answer: decline, with the idiom (one owned entity per axis).

Fits none of the four generic language features.

## Fit review

Finding covered: https://github.com/beyond10x/ess/issues/559 (a downstream incident-ledger
specification, ess 0.59.0, `ess/24`).

1. **Need.** A record has two independent lifecycles (the incident's own, and its review's).
   Requester's syntax (theirs): a second key `review_lifecycle:` beside `lifecycle:`; refused as an
   unknown field.
2. **Class.** Convenience: the reporter's own model states it with a separate `Review` entity owned
   by the incident, one lifecycle per entity, and that model validates
   (https://github.com/beyond10x/ess/issues/553, "The spec validates"). The reporter calls it low
   priority and keeps the separate entity on its merits (reviews come in rounds).
3. **Already expressible?** Yes: an owned entity per axis, guarded through `when_related` or a stored
   reference. Not run in this pass (no commands were run); the command output owed by a decline is
   the reporter's validated model until a neutral one is run.
4. **Fit.** A second axis would touch every consumer of the held state (precedence, `wrong_state`,
   moves, diff, generated state machines). If a later adopter needs it, it belongs to
   `epic:generic-guard-precedence` (guards that name the axis).
5. **Second adopter.** I don't know of one; a document with a publication state and an independent
   translation state is a candidate.
6. **Cost.** Decline: none. Accept: a new key on every entity reader.
7. **Alternatives.** (a) Change nothing, with the idiom (proposed). (b) Named axes. (c) Parallel
   regions as in statecharts.

## Decisions

Decline, with the idiom (proposed; a `specification` artifact stating the idiom with
`ess specify validate` output on a neutral model is owed before this story is archived). The
visibility issue stays open until then.
