---
format: aep.planning-md/3
id: story:a-struct-with-invariants-in-a-read-your-writes-view-is-observed
kind: story
status: draft
title: A struct with type invariants in a read_your_writes view is observed as a complete subject
summary: A struct field with invariants in a read_your_writes view breaks complete-subject observation, and leaving it out is ESS-SYNTH-013 (beyond10x/ess#730)
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#730
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 1
---
## Need

An entity field whose type is a struct with invariants breaks complete-subject observation when the field is in a read_your_writes view; leaving the field out of the view is refused as ESS-SYNTH-013.

Reported in https://github.com/beyond10x/ess/issues/730 (ess 0.86.0); the issue carries the
reproduction and the verbatim output.

## Acceptance

- The issue's reproduction is a failing test first, then passes: the scenario it names is
  synthesized (or the refusal it names is gone), and a correct target passes it.

## Verdict (wave lead, 2026-11-05, for the coordinator to accept)

Accept: check first whether 0.87.0's unreached-invariant note already clears the ESS-SYNTH-013 half; the observation half is a defect.
