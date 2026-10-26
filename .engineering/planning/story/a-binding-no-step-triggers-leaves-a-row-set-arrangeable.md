---
format: aep.planning-md/3
id: story:a-binding-no-step-triggers-leaves-a-row-set-arrangeable
kind: story
status: draft
title: A binding no step triggers leaves a row-set guard arrangeable
summary: 'ESS-SYNTH-001: any binding writing the selector entity makes a row-set guard opaque model-wide; 7 / 2, 5 / 0 without bindings.'
tags:
- adopter-report
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, neutral repro (format ess/25, `demo.*` names; committable as a fixture). Counts are scenarios / refusals from `ess verify conform synthesize --suite-format 5 --compact`; reproduced here on 0.69.2 and on the 0.70.0 integration head 3f7602f05 (same refusal counts).

A row-set guard `when_related: {entity: demo.y.Ticket, where: holder == input.holder, exists: true}` becomes opaque as soon as any binding writes `demo.y.Ticket`, even when no step of the scenario publishes that binding's trigger event. 7 / 2, ESS-SYNTH-001 "a step of the scenario changes rows of the selector's entity". Deleting the `bindings:` line gives 5 / 0. Reporter hypothesis (unverified): `crates/verify/ess-conformance/src/synthesize/row_set.rs` about :1258, `opaque: bound_writes(ir)`, marks every binding-written entity model-wide instead of per scenario step.

Classification: defect.

## Acceptance

- `a_binding_no_step_triggers_leaves_the_row_set_arrangeable`: the repro as a fixture synthesizes the `kept` and `removed` outcomes with 0 refusals.
- A scenario that does publish the trigger event still treats the selector as changed by that step.
