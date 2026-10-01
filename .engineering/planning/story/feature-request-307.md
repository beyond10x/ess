---
format: aep.planning-md/3
id: story:feature-request-307
kind: story
status: draft
title: when_subject over a field copied from a related row is witnessed
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#307
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A `when_subject` predicate over a field the creating command copied from a related row is witnessed.

## Acceptance

- On a reduction of #307, both policy branches and their transitions are synthesized with no ESS-SYNTH-003 or -004.

## Origin

beyond10x/ess#307, reported downstream on 0.49.0.

## Fit review

- Class: defect (synthesis only; no surface). The stored-row search must arrange the related source row before creating the subject.

## Decisions

- **accept as proposed** (coordinator, 2026-10-01). Priority 1; next wave after w2.
