---
format: aep.planning-md/3
id: story:synthesis-witnesses-aggregate-conditions-over-optional-and-unset-fields
kind: story
status: draft
title: Synthesis witnesses aggregate conditions over optional, unset and state-filtered fields
summary: 'ESS-SYNTH-017 for where: on Optional fields, avg over an unset field, four state-filtered counts in one group (workbench report items 20, 22, 24b)'
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

Aggregate views that validate get no scenario:

- A measure `where:` on an Optional field: `ESS-SYNTH-017 ... no arrangement of one group's rows
  gives the condition of \`with_x\` rows it admits and rows it refuses` (item 20, example `20`).
- An average over a field the creating command does not set: `ESS-SYNTH-017 view
  gap.demo.MeanScore has no scenario` (item 22, example `22`).
- Four state-filtered counts in one group: `ESS-SYNTH-017 ... no arrangement of one group's rows
  gives the condition of \`idle\`, \`paused\`, \`running\`, \`waiting\` rows` (item 24b, example `24b`).

ESS-SYNTH-017 appears 8 times in the reporter's whole model. Items 20, 22 and 24b of the 2026-11-09
workbench report.

## Acceptance

- Synthesis arranges a group's rows so each measure condition admits and refuses rows: for an
  Optional field (present and absent), for a field set only by a later command, and for several
  state filters in one group (rows moved into each state).
- Each example above synthesizes a scenario that the interpreted target passes.

## Scope

- inferred: `crates/verify/ess-conformance/src/synthesize.rs` (aggregate arrangement).

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
