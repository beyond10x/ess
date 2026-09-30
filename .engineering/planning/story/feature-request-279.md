---
format: aep.planning-md/3
id: story:feature-request-279
kind: story
status: draft
title: A stored-guarded moving command does not count as rewriting a group key
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#279
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Outcome

A command that moves a row without writing its group key does not make the aggregate witness refuse the view, whatever guards it carries.

## Acceptance

- A minimal specification (key set from input by the creating command; a moving command with two `when_subject` guards that does not touch the key) synthesizes `<view>/aggregate` with no ESS-SYNTH-017.

## Origin

beyond10x/ess#279, reported downstream on 0.48.0.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`: class defect (synthesis only; no authored surface).

## Decisions

- **accept once reduced** (coordinator, 2026-09-30): the implementor builds the minimal reproduction first; if it does not reproduce, the story is closed with that evidence.
