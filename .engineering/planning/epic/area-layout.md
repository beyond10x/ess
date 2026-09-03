---
format: aep.planning-md/1
id: epic:area-layout
kind: epic
status: implemented
title: Crates grouped by bounded context
summary: Group the 20 crates into specify, generate, verify, infra and edge; no crate renamed, no consumer re-pins.
revision: 4
---
# Epic: Crates grouped by bounded context

## Outcome

`crates/` shows the four contexts the repository already has — specify, generate, verify, infra —
plus `edge/` for the binary and repo tooling. Crate names are unchanged, so the four consumers that
pin `ess-compiler`, `ess-domain`, `ess-gen` and `ess-synth` by git revision (`agentide`,
`service-sdk`) are unaffected.

## Why now

Twenty crates sit flat under `crates/`. `infra-*` is a second bounded context that shares only
`ess-primitives` with the specification side, and `AGENTS.md` keeps `EssIr` and `InfraIr` apart by
rule; nothing in the tree shows either fact. Same operator finding as AEP's `epic:area-layout`;
analysis in `~/.cache/beyond10x-notes/2026-09-03-aep-ess-structure.md`.

## Scope

Area directories under `crates/`; workspace member and dependency paths; the seven fixture paths
in `Taskfile.yml`; the crate list in `README.md` and `AGENTS.md`.

## Out of scope

Renaming any crate; splitting `infra/` into its own repository; moving `examples/`, `generated/`,
`schemas/` or `suites/`.
