---
format: aep.planning-md/3
id: story:a-field-or-view-may-be-internal-to-the-model
kind: story
status: draft
title: 'A field or view may be internal: read by synthesis, never served or published'
summary: Identity must be published on the wire for synthesis (a2a wave 1, M2, M4, M4b).
tags:
- adopter-report
- ess-0.70.0
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a, in the a2a wave-1 gaps the
orchestrator relayed on 2026-10-22 (ESS 0.67.0, a2a main 3a18050). Three gaps, one need: synthesis
must see an identity or a field that the protocol does not put on the wire.

- **M2** (a2a wave-1 gaps list, line 35): the `row` input leaks onto the wire; it is served at the
  view `allPushNotificationConfigs` and published in `PushNotificationConfigCreated`. There is no
  internal-only field.
- **M4** (a2a wave-1 gaps list, line 37): dropping `row` from the published event or view is
  refused with ESS-SYNTH-001 "published by no immediate, unfiltered, unparameterised, unpaged view
  carrying its identity" (the adopter's `ess/tasks.yaml:1693`).
- **M4b** (a2a wave-1 gaps list, line 38): the `AllTasks` view has no source in the protocol;
  removing it makes synthesis refuse 35 outcomes, because no other view publishes the identity
  (the adopter's `ess/tasks.yaml:1628-1630`).

## Where it lives

Not traced beyond the refusal text. Related: `story:field-sensitivity-class` (a field classified as
personal or secret), which classifies but does not hide a field from the wire.

## Fit review

Owed: assessed with `.agents/skills/assessing-external-requests/SKILL.md` before dispatch. Two
shapes to weigh: a field or view marked internal (compiled, read by synthesis and conformance,
never served or published), or synthesis observing identity through another route. The first
must keep the conformance target able to answer an internal view.

## Acceptance (provisional)

- A field or view marked internal is absent from every served surface, published event, OpenAPI and
  AsyncAPI document, and synthesis still uses it to witness identity, so the adopter's 35 outcomes
  synthesize without `AllTasks` on the wire.
- An internal field read by a served view or a published payload is refused by name.

## Schedule

Gap; tagged `ess-0.69.0` as filed from a2a wave 1. Likely a new format version.
