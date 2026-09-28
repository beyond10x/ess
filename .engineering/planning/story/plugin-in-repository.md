---
format: aep.planning-md/3
id: story:plugin-in-repository
kind: story
status: archived
title: The ESS plugin and its marketplace live in this repository
relations:
- decomposes: epic:ess-agent-plugin
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-25T06:20:56Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"test_result":1}}, imported: true}
- {from: "proposed", to: "active", at: "2026-09-25T06:21:18Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}, imported: true}
- {from: "active", to: "implemented", at: "2026-09-25T06:21:46Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}, imported: true}
- {from: "implemented", to: "archived", at: "2026-09-25T06:27:22Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}, imported: true}
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
