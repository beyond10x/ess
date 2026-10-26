---
format: aep.planning-md/3
id: story:a-binding-tries-every-publisher-of-its-trigger
kind: story
status: draft
title: A binding tries every publisher of its trigger event
summary: 'ESS-SYNTH-010: synthesis tries only the first publisher of the trigger by name, so a rename changes the result; 6 / 4, 9 / 1 renamed.'
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

Two commands publish the trigger event. Synthesis takes only the first by name order; that one fills the field from `{changed:}`, so the binding is refused. 6 / 4 / 3; renaming the lower-named command, or filling the field from `input.<id>`, gives 9 / 1 / 0 (the remaining refusal is an unrelated ESS-SYNTH-001: no `read_your_writes` view). A rename changes the result, and the other publisher is never tried.

Classification: defect.

## Acceptance

- `a_binding_tries_every_publisher_of_its_trigger`: synthesis tries each publisher of the trigger event in a deterministic order and uses the first that witnesses the binding; the repro gives no ESS-SYNTH-010 under either name.
- Renaming a command does not change which binding scenarios are synthesized.
