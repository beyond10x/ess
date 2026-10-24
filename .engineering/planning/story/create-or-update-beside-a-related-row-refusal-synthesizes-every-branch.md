---
format: aep.planning-md/3
id: story:create-or-update-beside-a-related-row-refusal-synthesizes-every-branch
kind: story
status: draft
title: Create or update beside a related row refusal synthesizes every branch
summary: 'Adopter report G6: Create-or-update beside a related-row refusal does not synthesize every branch (ess#641).'
refs:
- provider: github
  reference: beyond10x/ess#641
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

A create-or-update command (an `unknown_instance: true` creator beside update and preserve branches) with a `when_related: {via: input.<id>, exists: false}` refusal validates, but synthesis finds no candidate for the creator (ESS-SYNTH-003: no stored related row selecting the branch, 1 arrangement tried) and then cascades ESS-SYNTH-004 onto the refusal and every route through the command. Without the related refusal the command synthesizes. Adopter report G6, ess 0.68.0; the adopter's call scenarios dropped from 156 to 107.

## Acceptance

- `the_creator_beside_a_related_refusal_arranges_the_related_row`: the adopter shape yields `Bind/outcome/bound` with an `Agent` row arranged for `input.agent_id`.
- `the_related_refusal_beside_a_create_or_update_is_witnessed`: `Bind/outcome/no-agent` is synthesized with no `Agent` row and no ESS-SYNTH-004.
- `the_update_branches_reach_their_row_through_the_creator`: `already-bound` and `rebound` are synthesized on a `Binding` the creator stored.
- `all_four_branches_pass_on_the_interpreted_target`: the four scenarios pass; a target that skips the related check fails `no-agent`.
