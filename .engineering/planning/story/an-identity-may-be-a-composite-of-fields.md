---
format: aep.planning-md/3
id: story:an-identity-may-be-a-composite-of-fields
kind: story
status: draft
title: 'An entity may be identified by several fields, and instance: addresses it by all of them'
summary: 'A composite identity is refused both as instance: map and as a published struct (a2a wave 1, M1).'
tags:
- adopter-report
- ess-0.71.0
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a, in the a2a wave-1 gaps the
orchestrator relayed on 2026-10-22 (ESS 0.67.0, a2a main 3a18050).

- **M1** (a2a wave-1 gaps list, line 34): an entity identified by a pair of fields (a push
  notification config by `task_id` and `config_id`) cannot be stated either way. `instance: {…}`
  is refused with "invalid type: map, expected a string"; a struct identity published by the
  creating event is refused with ESS-SYNTH-001 "a struct held by reference has no member
  projection" (the adopter's `ess/tasks.yaml:360-366`).

## Where it lives

Not traced. Not the same need as `story:via-and-instance-may-name-a-member-of-a-struct-input`:
that story lets `instance:` name one reference nested in a struct input; this one needs an identity
made of several fields (a composite key).

## Fit review

Owed: assessed with `.agents/skills/assessing-external-requests/SKILL.md` before dispatch.
Alternatives to weigh: a composite identity on the entity, against a synthetic single id with a
uniqueness constraint over the pair.

## Acceptance (provisional)

- An entity may be identified by two fields; `instance:` addresses it by both; synthesis creates,
  addresses and publishes it without ESS-SYNTH-001.
- An `instance:` that omits one identity field is refused by name.

## Schedule

Gap; tagged `ess-0.69.0` as filed from a2a wave 1. Likely a new format version.
