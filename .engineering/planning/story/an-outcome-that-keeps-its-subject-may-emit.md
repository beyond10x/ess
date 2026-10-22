---
format: aep.planning-md/3
id: story:an-outcome-that-keeps-its-subject-may-emit
kind: story
status: draft
title: An outcome that keeps its subject unchanged may emit an event
summary: A state-preserving emit is refused with ESS-COMMAND-004 (a2a wave 1, G4).
tags:
- adopter-report
- ess-0.70.0
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- depends_on: story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view
revision: 2
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a, in the a2a wave-1 gaps the
orchestrator relayed on 2026-10-22 (ESS 0.67.0, a2a main 3a18050).

- **G4** (a2a wave-1 gaps list, line 13): an outcome that keeps its subject unchanged and emits an
  event is refused with ESS-COMMAND-004, so the protocol's subscription response
  (`TaskSubscriptionResponse`) has a channel and no emitter.

## Where it lives

Not traced. The rendered ESS-COMMAND-004 line was not relayed; the implementor reproduces it first.
Related: `story:a-response-stream-follows-one-row-until-a-terminal-state`, which a
subscription's later events would ride on; this story is only the first, state-preserving emit.

## Fit review

Owed: assessed with `.agents/skills/assessing-external-requests/SKILL.md` before dispatch.

## Acceptance (provisional)

- An outcome that preserves its subject's state may emit an event; validation admits it,
  synthesis witnesses the emit with the subject unchanged, and the AsyncAPI projection names the
  outcome as an emitter of that event.

## Schedule

Gap; tagged `ess-0.69.0` as filed from a2a wave 1.
