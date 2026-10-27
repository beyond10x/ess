---
format: aep.planning-md/3
id: story:a-converted-binding-input-arranges-the-related-row
kind: story
status: implemented
title: A binding input reached through a declared conversion arranges the related row
summary: ESS-SYNTH-010 refuses a binding whose event field converts to the input a when_related guard reads; 5 scenarios / 3 refusals, 8 / 0 without the conversion.
tags:
- adopter-report
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T21:52:55Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-26T21:52:55Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-27T11:23:18Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:b10x-chef-orchestrator-ess-0710"}
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, neutral repro (format ess/25, `demo.*` names; committable as a fixture). Counts are scenarios / refusals from `ess verify conform synthesize --suite-format 5 --compact`; reproduced here on 0.69.2 and on the 0.70.0 integration head 3f7602f05 (same refusal counts).

A binding maps an event field of type `demo.x.GroupRef` onto the invoked command's input `group_id: demo.x.GroupId`; a declared conversion `GroupRef -> GroupId` exists, and the invoked command reads the input through `when_related: {via: input.group_id, exists: false}`. Synthesis refuses `binding/flow`, `binding/delivery` and `binding/refusal/group-missing` (ESS-SYNTH-010, "whose branch is decided by an input the event fills"): 5 / 3. Retyping the input to `GroupId` gives 8 / 0, so the conversion is what stops the arrangement, and the message does not say so.

Classification: defect (synthesis can arrange the row: define a group, then send its id through the conversion).

## Acceptance

- `a_converted_binding_input_arranges_the_related_row`: the repro as a fixture synthesizes 8 scenarios with 0 refusals, as its retyped twin does.
- Where a conversion cannot be followed, the refusal names the conversion.
