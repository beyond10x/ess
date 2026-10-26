---
format: aep.planning-md/3
id: story:a-set-effect-binding-refusal-names-each-cause
kind: story
status: draft
title: A set-effect binding refusal names each of its causes
summary: ESS-SYNTH-010 on a per-row set effect keyed by another entity row names none of four independent causes; 7 / 3, 9 / 0 only with all four removed.
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

A per-row set effect (`each: changed`) whose filter key names another entity's row. Four independent causes each refuse it: the trigger addresses that row; the creating command guards the key with `exists: false`; it copies `{related:}` from it; the trigger payload uses a literal for the written value. 7 / 3 / 3; only all four edits together give 9 / 0 / 0 (repro-r5c-fixed), any three leave 3. Both conditions the help asks for already hold.

Classification: defect, at least in the message, which names none of the four causes.

## Acceptance

- `a_set_effect_binding_refusal_names_each_cause`: each of the four shapes gets a refusal that names its own cause; the help is never one whose conditions already hold.
- Each cause synthesis can arrange is arranged: the repro with any one cause left gives 0 refusals where the cause is arrangeable.
