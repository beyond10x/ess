---
format: aep.planning-md/3
id: story:a-synthesis-seed-is-accepted-for-a-caller-model
kind: story
status: active
title: A synthesis seed is accepted for a model with caller attributes
summary: 'Adopter batch 3 P: --synthesis-seed is refused [caller-model] for any model whose actors carry caller attributes; lift the refusal.'
tags:
- adopter-report
- ess-0.73.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T00:21:16Z", actor: "human:timo", revision: 2, executor: "agent:b10x-chef-orchestrator-ess-0720"}
- {from: "proposed", to: "active", at: "2026-10-28T00:21:17Z", actor: "human:timo", revision: 3, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Problem

Reported by an adopter against the 0.71.0 integration head (reduced reproduction, `demo.*`
names). `ess verify conform synthesize --synthesis-seed <file> <scenario>` refuses every model whose
actors declare caller attributes:

> InvalidSynthesisSeeds: synthesis seed refused [caller-model]: the model's actors carry caller
> attributes; seeded synthesis of a caller assignment is not supported

The same seed, without the caller-sourced field, is accepted on the model without the attribute.
Seeds are the documented way out of the "no three rows the filter selects" refusals, so a model
that scopes its rows by a caller attribute has no way out of them. This is a limitation, not a
wrong answer: lift it.

## Acceptance

- Named conformance case `a_synthesis_seed_is_accepted_for_a_caller_model` (neutral fixture: an actor
  with `attributes: [account_id]`, a creator setting `account: {caller: account_id}`, and a seed
  arranging one row with that field set): the seed is admitted, the seeded row arranges the
  scenarios it is offered for, and those scenarios pass the interpreted target with the caller
  assignment the seed's row implies.
- A seed whose row contradicts the caller assignment (a field the caller attribute sets holding a
  value no caller can give it) is still refused, with a reason that names the field.
- Seeded synthesis of a model without caller attributes keeps its suite bytes.
