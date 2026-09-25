---
format: aep.planning-md/2
id: epic:ess-agent-plugin
kind: epic
status: archived
title: ESS ships its own agent plugin
summary: One tag carries the ess binary, the Claude/Codex plugin and its skills at one version.
relations:
- serves: vision:O2
revision: 6
---
# ESS ships its own agent plugin

## Outcome

An agent pointed at this repository onboards itself: one release tag carries one `ess` binary, one
Claude Code / Codex plugin and one skill set, all at the same version.

## Why

The agent guidance for ESS lives in `beyond10x/agentplugins` as the `ess-specify` plugin, released
on its own cadence. It has drifted: agentplugins `0.9.2` tells adopters to install ESS `0.22.1`
while this workspace is at `0.29.0`. Nothing checks that the skills describe the binary an adopter
runs.

## Scope

- An in-repository marketplace (`.claude-plugin/marketplace.json`, `.agents/plugins/marketplace.json`,
  identity `ess`) with one plugin, `plugins/ess/`.
- Skills moved from `ess-specify` (`specify`, `coverage`), plus a front-door `ess` skill and a
  `retrofit` skill; thin Claude agent wrappers `author`, `retrofitter`, `conformance`.
- `ess skill [<path>]`: prints the embedded skill set of the running binary, so an agent that does
  not install a plugin still reads the matching guidance.
- A gate that refuses a plugin manifest version that differs from the workspace version.

## Not in scope

- Eval cases for the plugin. ESS has no AEP dependency and runs no eval corpus.
- Removing `ess-specify` from agentplugins; that is agentplugins' own change, made after this lands.


## Reversed (2026-09-25)

Built and released in ESS 0.30.0 (PR #65, `67058c4a`), then reversed: every Beyond10x plugin lives in `beyond10x/agentplugins`, so ess#68 (merge `b88fdead`) removed `plugins/ess`, both marketplace files, `ess skill` and the xtask plugin check. The ESS plugin ships from agentplugins as `ess@b10x` (agentplugins #17 `f03fa470`, #18 `7574cb86`, release 0.14.1). The epic and its three stories are archived as implemented-then-moved; the removal ships with the next ESS release.
