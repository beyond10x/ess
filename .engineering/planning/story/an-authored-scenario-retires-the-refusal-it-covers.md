---
format: aep.planning-md/3
id: story:an-authored-scenario-retires-the-refusal-it-covers
kind: story
status: draft
title: An authored scenario retires the refusal it covers, and synthesize --path reads the inputs' scenarios
summary: 'Authored scenarios never retire the refusal they cover, and synthesize --path ignores ess-inputs.yaml scenarios: (beyond10x/ess#735)'
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#735
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 1
---
## Need

An authored ess-scenario/1 document that reaches the refused outcome passes against the interpreted target, but the refusal stays in the report; and synthesize --path does not read the scenarios: list of ess-inputs.yaml, only --scenarios.

Reported in https://github.com/beyond10x/ess/issues/735 (ess 0.86.0); the issue carries the
reproduction and the verbatim output.

## Acceptance

- The issue's reproduction is a failing test first, then passes: the scenario it names is
  synthesized (or the refusal it names is gone), and a correct target passes it.

## Verdict (wave lead, 2026-11-05, for the coordinator to accept)

Accept: two parts; reading ess-inputs.yaml scenarios: is small, retiring a covered refusal changes the coverage report and needs its rule written first.
