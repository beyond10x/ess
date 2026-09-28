---
format: aep.planning-md/3
id: story:ess-skill-command
kind: story
status: archived
title: ess skill prints the skill set the binary was built with
relations:
- decomposes: epic:ess-agent-plugin
- depends_on: story:plugin-in-repository
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-25T06:22:46Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"test_result":1}}, imported: true}
- {from: "proposed", to: "active", at: "2026-09-25T06:23:12Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}, imported: true}
- {from: "active", to: "implemented", at: "2026-09-25T06:23:36Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}, imported: true}
- {from: "implemented", to: "archived", at: "2026-09-25T06:27:46Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}, imported: true}
---
# `ess skill` prints the skill set the binary was built with

## Acceptance

`ess skill` prints the front-door skill and an index of every embedded skill and agent;
`ess skill <path>` prints one embedded file; an unknown path exits 2 naming the valid paths; and a
test holds that the embedded set equals the files under `plugins/ess/`.

## Scope

- `crates/edge/ess-cli/build.rs`, `crates/edge/ess-cli/src/skill.rs`, `crates/edge/ess-cli/src/main.rs`
- `crates/edge/ess-cli/tests/command_surface.rs`, `crates/edge/ess-cli/tests/skill_cli.rs`

`skill` is the one first-level verb outside the four areas; the surface tests say so.
