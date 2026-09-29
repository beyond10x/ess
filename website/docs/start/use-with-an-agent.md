---
title: Use ess with an agent
description: Install the ess agent plugin for Claude Code or Codex, and what its skills do once it is installed.
---

# Use ess with an agent

The agent plugin is `ess@b10x` in [`beyond10x/agentplugins`](https://github.com/beyond10x/agentplugins).
It teaches a coding agent (Claude Code or Codex) to install `ess`, write and validate a
specification with it, and hold an implementation to the suite the specification obliges. The
agent runs the same `ess` commands these pages show; the plugin decides when to run them.

## Set it up

Tell the agent:

> Set up Beyond10x for specifications: follow
> https://github.com/beyond10x/agentplugins/releases/latest/download/SETUP.md

The setup installs the plugin and the `ess` CLI, prebuilt or with `cargo`. It lists every change
and asks before applying any of them. The plugin loads in the next session.

## Take the first step

Run `/ess:init` in your project. It checks that `ess` is installed, explains what ESS does, and
takes the first step of a specification with you.

## What the plugin does after that

| Skill | When the agent uses it |
|---|---|
| `ess:specifying` | a specification is written, validated or projected into a schema or OpenAPI document, or a new noun needs a typed home |
| `ess:retrofitting` | an existing system with no specification needs one, derived from its OpenAPI contract, its cluster or its code |
| `ess:testing-conformance` | a conformance suite exists and the question is whether it tests anything, and how to raise what it runs |
| `ess:upgrade` | the plugin or the CLI is behind its latest release |

Everything the agent writes is an ordinary file in your repository: `ess-inputs.yaml`, the
specification files, generated artifacts and the conformance package. [Install
ess](./install.md) and the tutorials that follow it are the same path by hand.
