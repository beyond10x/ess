---
format: aep.planning-md/3
id: story:an-absent-optional-under-false-and-unknown-is-arranged
kind: story
status: draft
title: An absent Optional under false-and-unknown is arranged, and the refusal names the right creator
summary: Arrangement cannot decide false and unknown over an absent Optional input, and its refusal blames an unrelated creating command (beyond10x/ess#728)
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#728
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 1
---
## Need

A creating command guards an optional enum input with not defined(...) beside comparisons. Arrangement cannot decide the three-valued false and unknown over the absent Optional, and the refusal names an unrelated creator as the cause.

Reported in https://github.com/beyond10x/ess/issues/728 (ess 0.86.0); the issue carries the
reproduction and the verbatim output.

## Acceptance

- The issue's reproduction is a failing test first, then passes: the scenario it names is
  synthesized (or the refusal it names is gone), and a correct target passes it.

## Verdict (wave lead, 2026-11-05, for the coordinator to accept)

Accept: two defects, the three-valued arrangement and the misattributed refusal; the refusal text is the smaller fix and can land first.
