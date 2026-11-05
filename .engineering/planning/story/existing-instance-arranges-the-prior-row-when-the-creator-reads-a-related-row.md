---
format: aep.planning-md/3
id: story:existing-instance-arranges-the-prior-row-when-the-creator-reads-a-related-row
kind: story
status: draft
title: existing_instance arranges the prior row when the creating branch reads a related row
summary: 'existing_instance: cannot arrange the prior row when the creating branch guards on a related row, though another command creates it unguarded (beyond10x/ess#733)'
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#733
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
revision: 1
---
## Need

A creating command guards on a related row before it creates. Its existing_instance: refusal needs the row already present, and synthesis cannot arrange it, although another command creates the same entity under the same key with no guard.

Reported in https://github.com/beyond10x/ess/issues/733 (ess 0.86.0); the issue carries the
reproduction and the verbatim output.

## Acceptance

- The issue's reproduction is a failing test first, then passes: the scenario it names is
  synthesized (or the refusal it names is gone), and a correct target passes it.

## Verdict (wave lead, 2026-11-05, for the coordinator to accept)

Accept: the arrangement should take the unguarded creator; same surface as #731.
