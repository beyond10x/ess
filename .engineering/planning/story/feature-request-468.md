---
format: aep.planning-md/3
id: story:feature-request-468
kind: story
status: draft
title: A CLI result field may be typed Json
tags:
- ess-0.55.0
refs:
- provider: github
  reference: beyond10x/ess#468
relations:
- decomposes: epic:downstream-reported-gaps
revision: 1
---
# A CLI result field may be typed Json

## Acceptance

`ess generate cli` admits a CLI result (output) field typed `Json`: the generated adapter writes
the value as JSON, not as JSON text, and its result check accepts any well-formed JSON value there
and refuses malformed bytes as `cli_result`; a `Json` input field is still refused by name.

## Context

beyond10x/ess#468, filed 2026-10-06 against 0.53.0 by the connectors repository. The CLI contract
resolver maps only `String`, `Boolean` and `Integer`
(`crates/specify/ess-cli-contract/src/resolve.rs:37-42`); `Json` is an `ess/15` primitive. It
blocks beyond10x/connectors#105, whose `describe` and `invoke` answers carry JSON as text today.

## Fit review

Owed. Run `.agents/skills/assessing-external-requests/SKILL.md` before dispatch; settle whether
`ess-cli/1` needs a new presentation-binding version for the shape.

## Milestone

`release-plan:ess-055`. Reply to the connectors session when a release carries it.
