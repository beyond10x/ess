---
format: aep.planning-md/3
id: story:a-binding-arranges-the-row-a-set-guard-selects
kind: story
status: draft
title: A binding arranges the row a set guard of its invoked command selects
summary: ESS-SYNTH-010 refuses a binding whose invoked command branches on a set guard, and its help leads to ESS-COMMAND-005; 5 / 3, 8 / 0 with external.
tags:
- adopter-report
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, neutral repro (ess/25, `demo.*` names, committable as a fixture; the reporter confirmed each "after" on the full adopter specification with no new refusal). Counts are selected scenarios / refusals / of which ESS-SYNTH-010, reproduced here on 0.69.2.

The invoked command of a binding has an accepting branch behind a set guard `when_related: {entity:, where: f == input.x, exists: true}` with a default branch after it. 5 / 3 / 3, "whose branch is decided by an input the event fills: kept, dropped". The same command with a `via:` guard in that place synthesizes; replacing the set guard by `external:` gives 8 / 0 / 0. The help ("declare the others `external:`") cannot be followed: the default branch then fails validation with ESS-COMMAND-005, so the only way out deletes the guard.

Classification: correct refusal with misleading help (today synthesis does not arrange a set-guard row for a binding).

## Acceptance

- `a_binding_refusal_help_is_one_the_specification_can_follow`: the refusal names the set guard as the reason and offers a remedy that validates; it never asks to declare a default branch `external:`.
- `a_binding_arranges_the_row_a_set_guard_selects`: as for a `via:` guard, synthesis arranges a row matching `where:` before the trigger, so the repro gives 8 / 0.
