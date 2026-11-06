---
format: aep.planning-md/3
id: story:an-external-outcome-guarded-on-a-later-state-is-witnessed-from-that-state
kind: story
status: draft
title: An external outcome guarded on a later state is witnessed from that state
summary: An external outcome with when_subject state == S is witnessed from the initial state, so the model selects wrong-state and every implementation fails it (beyond10x/ess#739)
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#739
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 1
---
## Need

An outcome that is external: and also guarded by when_subject state == S is synthesized from a freshly created subject in the initial state; the guard does not hold, the model selects wrong-state, and the scenario fails against every correct implementation, the interpreter included.

Reported in https://github.com/beyond10x/ess/issues/739 (ess 0.86.0); the issue carries the
reproduction and the verbatim output.

## Acceptance

- The issue's reproduction is a failing test first, then passes: the scenario it names is
  synthesized (or the refusal it names is gone), and a correct target passes it.

## Verdict (wave lead, 2026-11-05, for the coordinator to accept)

Accept, high: a scenario every correct implementation fails; the witness should move the subject to S first.
