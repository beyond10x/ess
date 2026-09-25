---
format: aep.planning-md/2
id: story:plugin-in-repository
kind: story
status: archived
title: The ESS plugin and its marketplace live in this repository
relations:
- decomposes: epic:ess-agent-plugin
- serves: vision:O2
revision: 5
---
# The ESS plugin and its marketplace live in this repository

## Acceptance

`plugins/ess/` holds the skills `ess`, `specify`, `coverage` and `retrofit` and the agents `author`,
`retrofitter` and `conformance`; both marketplace manifests at the repository root list it under
identity `ess`, and `claude plugin validate` accepts the plugin.

## Scope

- `plugins/ess/**`
- `.claude-plugin/marketplace.json`, `.agents/plugins/marketplace.json`

`specify` and `coverage` are moved from `beyond10x/agentplugins` `plugins/ess-specify/skills/` at
`68f4e08`.
