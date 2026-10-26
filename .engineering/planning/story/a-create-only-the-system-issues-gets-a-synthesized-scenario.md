---
format: aep.planning-md/3
id: story:a-create-only-the-system-issues-gets-a-synthesized-scenario
kind: story
status: draft
title: A create only the system issues gets a synthesized scenario
summary: 'ESS-SYNTH-023: a creating command only the system issues gets no scenario.'
tags:
- adopter-report
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, from a downstream repository specification (which quarantines 9 scenarios for gaps 1 and 2).

ESS-SYNTH-023: a creating command that only the system issues gets no synthesized scenario, so the create and everything downstream of it is unexercised.

Not yet reproduced: no specification came with this item.

## Acceptance

- First a neutral reproduction, checked in as a fixture.
- `a_system_only_create_is_awaited_in_a_scenario`: the create is reached through the trigger that issues it and its outcome, events and view rows are checked; where no trigger reaches it, the refusal names the missing trigger.
