---
format: aep.planning-md/3
id: story:a-binding-into-an-updating-command-follows-its-related-guards
kind: story
status: implemented
title: A binding into an updating command follows its related guards
summary: 'ESS-SYNTH-010: the guard-following arrangement is skipped when the invoked command updates an existing row; 8 / 4, 11 / 0 without updates.'
tags:
- adopter-report
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-27T14:44:58Z", actor: "human:timo", revision: 2, executor: "agent:b10x-chef-orchestrator-ess-0720"}
- {from: "proposed", to: "active", at: "2026-10-27T14:44:59Z", actor: "human:timo", revision: 3, executor: "agent:b10x-chef-orchestrator-ess-0720"}
- {from: "active", to: "implemented", at: "2026-10-27T19:43:33Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, neutral repro (ess/25, `demo.*` names, committable as a fixture; the reporter confirmed each "after" on the full adopter specification with no new refusal). Counts are selected scenarios / refusals / of which ESS-SYNTH-010, reproduced here on 0.69.2.

The invoked command `updates:` a row named by one input and branches on `via:` guards over another entity. The guard-following arrangement is skipped when the command acts on an existing row (`binding_effects::prepare` handles the unaddressed case only). 8 / 4 / 4; replacing the two guards by `external:` gives 11 / 0 / 0, and deleting only the `updates:` lines also gives 11 / 0 / 0, so the rows are arrangeable. The message blames "an input the event fills" although the trigger's own input fills it. Two disclosure cells through the same command are refused for the same reason.

Classification: defect (a scope limit synthesis can lift) with a misleading message.

## Acceptance

- `a_binding_into_an_updating_command_follows_its_related_guards`: the repro synthesizes 11 scenarios with 0 refusals, as its twin without `updates:` does.
- Where a guard still cannot be followed, the message names the guard and the row, not the event.
