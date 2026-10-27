---
format: aep.planning-md/3
id: story:a-scenario-beside-a-prefixed-conversion-target-runs-on-the-interpreter
kind: story
status: draft
title: A scenario beside a binding into a prefixed conversion target runs on the interpreter
summary: 'Adversary 0.71.0 F5: with a prefix on the conversion target, the trigger and mapping scenarios are Unsupported on the interpreter, as on 0.70.0.'
tags:
- ess-0.72.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

Found by the adversary pass over wave 1 of 0.71.0 (`adversary_0710_pass1.rs`,
`u4_the_prefixed_model_s_remaining_scenarios_pass_the_interpreter`); the 0.70.0 binary gives the
same result, so it predates the wave. A binding maps an event field onto an invoked command's input
across a declared conversion whose target newtype carries a `prefix:`. Synthesis refuses the binding's
flow and names the conversion (correct since 0.71.0), but the scenarios it does write,
`CreateMember/outcome/created` and `binding/mapping`, come back `Unsupported` on the model
interpreter: "binding input does not satisfy its declared type". The interpreter checks the mapped
value against the target's prefix and finds the unconverted source value, so a scenario synthesis
wrote for a valid model cannot run on the reference target.

## Acceptance

- `u4_the_prefixed_model_s_remaining_scenarios_pass_the_interpreter`: every scenario synthesis writes
  for the prefixed model passes the interpreted target, or synthesis refuses the scenario and names
  the conversion.
- A model whose conversion joins two unconstrained newtypes keeps its suite bytes.
