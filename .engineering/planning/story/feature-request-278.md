---
format: aep.planning-md/3
id: story:feature-request-278
kind: story
status: draft
title: An input guard beside a stored-field guard on one branch is synthesized
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#278
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Outcome

A branch guarded by both an input condition and a stored-field condition, beside a sibling with the same input condition, gets its scenarios; derived wrong-state conditions never contradict themselves.

## Acceptance

- On the minimal specification attached to #278, `RecordResult/outcome/held-for-promotion`, its transition scenario and the three `state/*/refuses/RecordResult` scenarios are synthesized with no ESS-SYNTH-003.
- No synthesized condition has the form `c and none of: c, …`; a test asserts it over the fixture set.

## Origin

beyond10x/ess#278, reported downstream on 0.48.0.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`: class defect (synthesis only; no authored surface).

## Decisions

- **accept** (coordinator, 2026-09-30).
