---
format: aep.planning-md/3
id: story:plugin-version-gate
kind: story
status: archived
title: The gate refuses a plugin that disagrees with the workspace
relations:
- decomposes: epic:ess-agent-plugin
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-25T06:24:21Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"test_result":1}}, imported: true}
- {from: "proposed", to: "active", at: "2026-09-25T06:24:45Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}, imported: true}
- {from: "active", to: "implemented", at: "2026-09-25T06:25:10Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}, imported: true}
- {from: "implemented", to: "archived", at: "2026-09-25T06:28:11Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}, imported: true}
---
# The gate refuses a plugin that disagrees with the workspace

## Acceptance

`cargo xtask plugin check` exits non-zero when either plugin manifest version differs from
`[workspace.package] version`, when a marketplace does not declare `ess` or list `./plugins/ess`,
when a skill folder, its frontmatter `name` and its path disagree, or when an agent wrapper names no
existing skill; `task check` and `cargo xtask release verify` run it.

## Scope

- `crates/edge/ess-xtask/src/main.rs` (or a new `plugin.rs` module beside it)
- `Taskfile.yml`
- `AGENTS.md` (version-bump list), `README.md` (agent onboarding block)
