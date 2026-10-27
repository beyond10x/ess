---
format: aep.planning-md/3
id: story:a-caller-attribute-keeps-the-binding-refusal-scenario
kind: story
status: draft
title: A caller attribute keeps a binding's refusal-policy scenario
summary: 'Adopter batch 3 O: a caller-sourced field withdraws binding/refusal/missing as UnsupportedVocabulary; 9 / 1 without the attribute, 8 / 2 with it.'
tags:
- adopter-report
- ess-0.73.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

Reported by an adopter against the 0.71.0 integration head (reduced reproduction, `demo.*`
names). A binding invokes a command whose `missing` branch refuses on
`when_related: {via: input.<id>, exists: false}`. The model's only actor declares a caller
attribute, `attributes: [{name: account_id, …}]`, and the related row's creator sets a field from it,
`account: {caller: account_id}`. Synthesis then withdraws the binding's refusal-policy scenario,
`<binding>/binding/refusal/missing`:

> the suite's own admission refuses scenario … UnsupportedVocabulary: a scenario per selected
> refusal requires suite/36 or /37; the scenario is withdrawn rather than written

The same model without the caller attribute and the `caller:` source writes that scenario (9
scenarios, 1 refusal; with the attribute 8 scenarios, 2 refusals). The refusal's help itself says
"nothing to change in the specification; this is a defect in ess". The suite format synthesis
picks for a caller model, or the admission check, does not account for the per-refusal scenario.

## Acceptance

- Named conformance case `a_caller_attribute_keeps_the_binding_refusal_scenario` (neutral fixture of
  the shape above): `<binding>/binding/refusal/missing` is written, the suite is admitted, and the
  scenario passes the interpreted target, so the scenario and refusal counts equal the model
  without the caller attribute.
- The same fixture without the caller attribute keeps its suite bytes.
