---
format: aep.planning-md/3
id: story:synthesis-covers-input-guards-beside-an-existing-instance-branch
kind: story
status: draft
title: Synthesis witnesses an input-guarded refusal on a creator that also has an existing_instance branch
refs:
- provider: github
  reference: beyond10x/ess#479
relations:
- serves: vision:O2
revision: 1
---
## Outcome

`ess verify conform synthesize` witnesses an input-guarded refusal outcome on a creating command
that also declares an `existing_instance: true` outcome, instead of refusing it with ESS-SYNTH-019
(https://github.com/beyond10x/ess/issues/479). Measured on 0.55.0 with the issue's two-file
`catalog` reproducer: every guard form refuses; removing the `existing_instance` outcome
synthesizes it with `item_id: ""`; an authored scenario covering it is not counted ("0 authored").

## Acceptance

- The issue's reproducer synthesizes `catalog.items.CreateItem/outcome/invalid-item-id` with an
  input that satisfies its guard, for the guard forms the issue lists, and 0 refusals.
- An authored ess-scenario/1 that covers the outcome is counted.
