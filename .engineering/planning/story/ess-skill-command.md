---
format: aep.planning-md/2
id: story:ess-skill-command
kind: story
status: draft
title: ess skill prints the skill set the binary was built with
relations:
- decomposes: epic:ess-agent-plugin
- depends_on: story:plugin-in-repository
revision: 1
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
