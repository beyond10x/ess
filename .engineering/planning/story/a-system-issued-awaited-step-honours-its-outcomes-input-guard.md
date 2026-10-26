---
format: aep.planning-md/3
id: story:a-system-issued-awaited-step-honours-its-outcomes-input-guard
kind: story
status: draft
title: An awaited system-issued step honours its outcome input guard
summary: The rewrite to awaited steps drops the outcome input guard, so a correct target fails; reporter repro system-guard, 2 of 7 fail.
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

When synthesis rewrites a step the system issues into an awaited step (`crates/verify/ess-conformance/src/synthesize.rs` about :2887), the awaited input gets no constraint from the outcome's input guard: an outcome guarded `when: verdict == Refused` is awaited with an input the guard does not admit, so a correct target fails. Reporter repro `system-guard` (7 scenarios, 2 fail): the run report shows the awaited `RecordCheck` input checked against the wrong branch.

Reproduced from the reporter's synthesized suite and run report; not re-run against a target here.

## Acceptance

- `an_awaited_system_step_carries_its_outcomes_input_guard`: the neutral repro, checked in as a fixture, synthesizes awaited steps whose expected input satisfies the outcome guard, and the reference target passes all 7 scenarios.
- A guard that cannot be expressed as an awaited-input constraint is a typed refusal naming the scenario, not a scenario that fails a correct target.
