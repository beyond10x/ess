---
format: aep.planning-md/3
id: story:a-multi-state-wrong-state-error-branch-is-synthesized
kind: story
status: draft
title: A wrong-state error branch over several states is synthesized in each of them
summary: 'when_subject_state: [A, B] on an error branch validates, but synthesis refuses it as selected in a held state (beyond10x/ess#732)'
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#732
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 1
---
## Need

An error branch with when_subject_state listing two states validates, but synthesis refuses it, saying it is selected in a held state of another branch. A small two-state reduction did not reproduce it.

Reported in https://github.com/beyond10x/ess/issues/732 (ess 0.86.0); the issue carries the
reproduction and the verbatim output.

## Acceptance

- The issue's reproduction is a failing test first, then passes: the scenario it names is
  synthesized (or the refusal it names is gone), and a correct target passes it.

## Verdict (wave lead, 2026-11-05, for the coordinator to accept)

Keep as a draft: the reporter's reduction did not reproduce; it needs a reproduction from the reporter's full specification first.
