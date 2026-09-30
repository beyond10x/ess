---
format: aep.planning-md/3
id: story:feature-request-275
kind: story
status: draft
title: The caller-swapped run draws fresh identity inputs
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#275
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Outcome

The caller-swapped run of a synthesized scenario draws fresh values for caller-supplied identity inputs, so a command with an `existing_instance` refusal is not sent the same identity twice while expecting success.

## Acceptance

- On a minimal specification with a caller-supplied identity input and an `existing_instance` refusal, every synthesized scenario passes against the reference target under both caller orders.
- No synthesized scenario sends one caller-supplied identity twice while expecting the accepting outcome the second time.

## Origin

beyond10x/ess#275, reported downstream on 0.48.0 with a reproducing patch; site `crates/verify/ess-conformance/src/synthesize/caller.rs`.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`. Need: the suite must not contradict `existing_instance`. Class: defect (synthesis only; no authored surface). Existing idiom: none needed.

## Decisions

- **accept as proposed** (coordinator, 2026-09-30).
