---
format: aep.planning-md/3
id: story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed
kind: story
status: draft
title: A forced external outcome whose guard relates an input list to a stored list is witnessed
summary: An external branch whose when_subject relates an input list to a stored list gets no outcome scenario, and every scenario through its state is refused ESS-SYNTH-004 (beyond10x/ess#727)
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#727
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 1
---
## Need

An external: branch whose when_subject predicate relates an input list to a stored list gets no outcome scenario. Its target state then counts as unreachable, so every scenario that has to pass through it is refused too (ESS-SYNTH-004).

Reported in https://github.com/beyond10x/ess/issues/727 (ess 0.86.0); the issue carries the
reproduction and the verbatim output.

## Acceptance

- The issue's reproduction is a failing test first, then passes: the scenario it names is
  synthesized (or the refusal it names is gone), and a correct target passes it.

## Verdict (wave lead, 2026-11-05, for the coordinator to accept)

Accept: same arrangement family as #734 (stored list against input list); one unit for both.
