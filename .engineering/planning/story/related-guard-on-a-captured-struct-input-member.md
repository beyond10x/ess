---
format: aep.planning-md/3
id: story:related-guard-on-a-captured-struct-input-member
kind: story
status: draft
title: Synthesis honours a related guard on a member of a captured struct input
tags:
- adopter-report
- defect
refs:
- provider: github
  reference: beyond10x/ess#521
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

Synthesis honours a `when_related:` guard whose predicate reads a member of a struct input
(`team != input.key.team`) when the struct input is a captured instance, as it already does when
the struct is a literal: no synthesized scenario expects the success branch for an input the guard
refuses.

## Evidence

https://github.com/beyond10x/ess/issues/521 (ess 0.56.0, `ess/23`), with a complete neutral
reproducer in the issue (`demo.desk`: a `Seat` keyed by a `SeatKey {team, seat}` struct,
referencing a `Person` with a `team` field). With the struct input given as a literal the guard is
honoured; with it captured from an earlier step it is ignored, the scenario expects success, and
the interpreted target fails it.

## Acceptance

- A synthesis test over the issue's reproducer: every synthesized scenario whose struct input is a
  captured instance expects the refusal where the related row's `team` differs from the captured
  key's `team`, and the interpreted target passes the suite.
- The literal-struct case keeps its scenarios byte for byte.
