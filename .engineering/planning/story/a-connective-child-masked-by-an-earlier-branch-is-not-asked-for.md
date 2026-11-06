---
format: aep.planning-md/3
id: story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for
kind: story
status: draft
title: A connective child masked by an earlier branch is not asked for a witness
summary: ESS-SYNTH-022 asks for a witness of defined(x) beside a comparison on x that an earlier branch already masks, which no input can give (beyond10x/ess#729)
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#729
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 1
---
## Need

With defined(x) beside a comparison on x, and the absent case already taken by an earlier branch, ESS-SYNTH-022 asks for a witness of a child that no input can reach and refuses the outcome.

Reported in https://github.com/beyond10x/ess/issues/729 (ess 0.86.0); the issue carries the
reproduction and the verbatim output.

## Acceptance

- The issue's reproduction is a failing test first, then passes: the scenario it names is
  synthesized (or the refusal it names is gone), and a correct target passes it.

## Verdict (wave lead, 2026-11-05, for the coordinator to accept)

Accept: the connective-coverage rule should treat a child masked by an earlier branch as covered, not as unwitnessed.
