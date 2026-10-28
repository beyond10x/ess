---
format: aep.planning-md/3
id: story:a-seeded-scenario-notes-its-skipped-reversed-caller-run
kind: story
status: active
title: A seeded scenario notes its skipped reversed-caller run
summary: A suite note names a seeded scenario whose fixed caller skips the reversed-caller run
tags:
- ess-0.75.0
relations:
- informed_by: story:a-synthesis-seed-is-accepted-for-a-caller-model
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T08:42:12Z", actor: "human:timo", revision: 3, executor: "agent:b10x-chef-orchestrator-ess-0720"}
- {from: "proposed", to: "active", at: "2026-10-28T08:42:12Z", actor: "human:timo", revision: 4, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Problem

Raised by the 0.74.0 wave's U2 implementor.

0.74.0 admits a synthesis seed for a model whose actors declare caller attributes. The seeded row implies the caller of the seeded run.

For attribute-free actors, `docs/design/synthesis-seeds.md` § Arrangement and obligation already does two things:
- it skips the reversed-caller rerun of a scenario that establishes a row by setup;
- it records the note `CrossCallerUnswapped`.

It is not checked whether a seeded scenario on a caller model, which takes its caller from the seed, gets the same note when its swapped-caller run is skipped. The implementor asked for one. It is unclear whether a caller model has a reversed-caller run to skip at all; the first step is to read `caller.rs` `run_seeded` and decide.

## Acceptance

- If a seeded scenario on a caller model skips a run that its unseeded twin would get, the suite carries a note naming the scenario and the seed row that fixed the caller.
- If no such run is skipped, a named case pins that, and the story closes with that evidence.
- An unseeded model's suite bytes are unchanged.
