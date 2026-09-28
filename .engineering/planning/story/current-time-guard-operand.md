---
format: aep.planning-md/3
id: story:current-time-guard-operand
kind: story
status: draft
title: A Timestamp guard can compare with the current time and a tolerance
refs:
- provider: github
  reference: beyond10x/ess#171
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

A `now` operand with duration arithmetic in guards, with a clock the conformance adapter controls (see `timestamp-clock-provenance.md`).

## Acceptance

The #171 repro validates; scenarios at now-61s and now-59s require refusal and acceptance.
