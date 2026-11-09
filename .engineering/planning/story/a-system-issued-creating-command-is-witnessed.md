---
format: aep.planning-md/3
id: story:a-system-issued-creating-command-is-witnessed
kind: story
status: draft
title: A command a system actor holds that creates its subject is witnessed
summary: ESS-SYNTH-023 on both outcomes of a system-issued creating command (workbench report item 43)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

A creating command held by a `system: true` actor gets `ESS-SYNTH-023` on both outcomes. Item 43
of the 2026-11-09 workbench report; example `43`. #542 is related.

## Acceptance

- Synthesis witnesses both outcomes of a system-issued creating command, issuing it as its system
  actor, and the interpreted target passes them.

## Scope

- inferred: `crates/verify/ess-conformance/src/synthesize.rs` (system-issued steps).

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
