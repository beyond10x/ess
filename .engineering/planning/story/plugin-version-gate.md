---
format: aep.planning-md/1
id: story:plugin-version-gate
kind: story
status: draft
title: The gate refuses a plugin that disagrees with the workspace
relations:
- decomposes: epic:ess-agent-plugin
revision: 1
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
