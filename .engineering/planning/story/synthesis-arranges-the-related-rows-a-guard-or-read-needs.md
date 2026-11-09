---
format: aep.planning-md/3
id: story:synthesis-arranges-the-related-rows-a-guard-or-read-needs
kind: story
status: draft
title: Synthesis arranges the related and filtered rows a guard or read needs
summary: ESS-SYNTH-001 for guards over related row states and counts, ESS-SYNTH-005 for an optional-filtered read (workbench report items 21, 23, 24a)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

Commands whose guards or reads validate get no witness:

- A guard over the states of related rows: `ESS-SYNTH-001 no witness: ... is \`row set\`, which has no
  arrangement of rows its selector selects: the command creating them refuses a missing related
  row through the input the selector compares` (item 21, example `21b`; a whole-model form reports
  `its own row set is Unknown`).
- A guard counting related rows whose `where` names no input: `ESS-SYNTH-001 no witness ... a
  refused subject is observed through a view of the entity projecting its identity, and none shows
  this row` (item 24a, example `24a`).
- A view filtered on an optional field, read by a command: `ESS-SYNTH-005 outcome ... has a
  scenario ... that skips its read of \`gap.demo.HighScores\`` (item 23, example `23`).

ESS-SYNTH-001 appears 27 times and ESS-SYNTH-005 14 times in the reporter's whole model. Items 21,
23 and 24a of the 2026-11-09 workbench report.

## Acceptance

- Synthesis creates the related rows a guard selects, in the states it compares, through their
  creating command's admitted inputs, for each example above.
- A read of a view filtered on an optional field is witnessed with a row inside and outside the
  filter.
- The examples synthesize scenarios the interpreted target passes.

## Scope

- inferred: `crates/verify/ess-conformance/src/synthesize.rs` (guard witnesses, related-row setup,
  view reads).

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
