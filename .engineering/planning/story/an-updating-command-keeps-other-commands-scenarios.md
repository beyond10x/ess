---
format: aep.planning-md/3
id: story:an-updating-command-keeps-other-commands-scenarios
kind: story
status: draft
title: An added updating command keeps the other commands' derived scenarios
summary: ESS-SYNTH-001 on one command never removes derived scenarios of another command on the entity
refs:
- provider: github
  reference: beyond10x/ess#679
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---

- An entity `Agent` has a creating command `CreateAgent` and an updating command `UpdateAgent`; synthesis derives `CreateAgent/accepted` and `UpdateAgent/updated`.
- Two more commands, `MintEmbedToken` and `RevokeEmbedToken`, are added with `updates:` on `Agent`.
- From then on synthesis no longer emits `CreateAgent/accepted` or `UpdateAgent/updated`, and reports `ESS-SYNTH-001`.
- The adopter works around it with an authored scenario covering the mint and revoke flow.

## Expected

Adding a command that updates an entity never removes derived scenarios of the other commands on that entity. If one branch cannot be arranged, the refusal names that branch and the other commands' scenarios are still emitted.

## Priority

High: a valid change to a specification silently lowers coverage of commands it did not touch.
