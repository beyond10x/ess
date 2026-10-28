---
format: aep.planning-md/3
id: story:every-set-move-only-refusal-names-the-state
kind: story
status: implemented
title: Every refusal a set-move-only state causes names the state
summary: ChangedRowUnarranged and the affects entry refusal name a state only a set move reaches, as the instances and trigger refusals do since 0.74.0
tags:
- ess-0.75.0
relations:
- informed_by: review-result:ess-0740-wave-adversary-pass-1
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T08:42:10Z", actor: "human:timo", revision: 2, executor: "agent:b10x-chef-orchestrator-ess-0720"}
- {from: "proposed", to: "active", at: "2026-10-28T08:42:11Z", actor: "human:timo", revision: 3, executor: "agent:b10x-chef-orchestrator-ess-0720"}
- {from: "active", to: "implemented", at: "2026-10-28T10:58:30Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Problem

Found while fixing the 0.74.0 wave's adversary pass (F4).

0.74.0 names a state that only a set move reaches in two refusals:
- the `instances:` set-move refusal (ESS-SYNTH-001);
- the per-row trigger refusal `TriggerRowUnarranged`.

Two other refusals can meet the same cause, and still give the remedy for a filter key that is
not written from an input:
- `ChangedRowUnarranged`, when the invoked command's per-row rows rest only in such a state;
- the `affects:` entry refusal for too few matching rows (`synthesize/set_effects.rs`, about
  :2830).

The cause comes from reading the code. No adopter model has shown it yet.

## Acceptance

- In a model where the invoked command's per-row rows can rest only in a state that a set move
  reaches, the `ChangedRowUnarranged` refusal names that state and the set move, and gives the
  same help as the `instances:` refusal.
- The same holds for the `affects:` entry refusal, on a model where the affected rows can rest
  only in such a state.
- A refusal whose filter is unmet in every state keeps its old text. A named control case pins
  this.
