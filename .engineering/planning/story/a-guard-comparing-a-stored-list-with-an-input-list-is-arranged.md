---
format: aep.planning-md/3
id: story:a-guard-comparing-a-stored-list-with-an-input-list-is-arranged
kind: story
status: draft
title: A guard comparing a stored list with an input list is arranged
summary: A when_subject predicate relating a stored list to an input list is refused ESS-SYNTH-003 after one candidate (beyond10x/ess#734)
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#734
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 1
---
## Need

A refusal whose when_subject predicate requires an element of a stored list with no match in a nested input list gets no scenario: synthesis tries one candidate and refuses ESS-SYNTH-003.

Reported in https://github.com/beyond10x/ess/issues/734 (ess 0.86.0); the issue carries the
reproduction and the verbatim output.

## Acceptance

- The issue's reproduction is a failing test first, then passes: the scenario it names is
  synthesized (or the refusal it names is gone), and a correct target passes it.

## Verdict (wave lead, 2026-11-05, for the coordinator to accept)

Accept: pair with #727 in one unit (list-relation arrangement).
