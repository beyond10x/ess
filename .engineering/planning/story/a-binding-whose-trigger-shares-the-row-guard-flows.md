---
format: aep.planning-md/3
id: story:a-binding-whose-trigger-shares-the-row-guard-flows
kind: story
status: draft
title: A binding whose trigger shares the invoked row guard still flows
summary: ESS-SYNTH-010 refuses flow and delivery when the trigger carries the same exists:false guard as the invoked command; 6 / 3, 8 / 0 without it.
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

The trigger command carries the same `when_related: {via: input.group_id, exists: false}` refusal as the invoked command, so the trigger's event is only published when the row exists. Synthesis still refuses `binding/flow`, `binding/delivery` and `binding/refusal/group-missing` (ESS-SYNTH-010): 6 / 3. Deleting the trigger's guard gives 8 / 0.

Classification: defect for flow and delivery (create the row, then send the trigger). The `binding/refusal/group-missing` cell is truly impossible, because the trigger never publishes without the row, and its refusal should say that.

## Acceptance

- `a_binding_whose_trigger_shares_the_row_guard_flows`: the repro as a fixture synthesizes `binding/flow` and `binding/delivery`.
- `binding/refusal/group-missing` stays refused, with a message naming the trigger's guard as the reason no trigger event reaches the refusal.
