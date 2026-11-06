---
format: aep.planning-md/3
id: story:an-input-guard-on-an-enum-variant-leaves-the-related-row-family-arrangeable
kind: story
status: draft
title: An input guard on an enum input's first variant leaves the related-row family arrangeable
summary: An input guard on the first variant of an enum input blocks the arrange_related_row family with ESS-SYNTH-008 (beyond10x/ess#731)
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#731
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 1
---
## Need

When an earlier branch refuses kind == <first variant>, the arrange_related_row candidates keep offering that variant, are refused by the guard, and the related-row family is refused ESS-SYNTH-008.

Reported in https://github.com/beyond10x/ess/issues/731 (ess 0.86.0); the issue carries the
reproduction and the verbatim output.

## Acceptance

- The issue's reproduction is a failing test first, then passes: the scenario it names is
  synthesized (or the refusal it names is gone), and a correct target passes it.

## Verdict (wave lead, 2026-11-05, for the coordinator to accept)

Accept: the candidate should pick a variant the earlier guard does not take; a small, contained synthesis fix.
