---
format: aep.planning-md/3
id: story:a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes
kind: story
status: draft
title: A stored-field guard beside an input guard on a system step synthesizes
summary: 'ESS-SYNTH-024: a stored-field guard beside an input guard on a system-issued step is refused.'
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

ESS-SYNTH-024: on a step the system issues, a guard over a stored field beside a guard over an input field is refused instead of synthesized.

Not yet reproduced: no specification came with this item.

## Acceptance

- First a neutral reproduction, checked in as a fixture.
- `a_stored_guard_beside_an_input_guard_on_a_system_step_synthesizes`: synthesis arranges the stored field and constrains the awaited input, and the reference target passes; a combination it cannot arrange is refused naming the guard.
