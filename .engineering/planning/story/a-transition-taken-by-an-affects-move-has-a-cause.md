---
format: aep.planning-md/3
id: story:a-transition-taken-by-an-affects-move-has-a-cause
kind: story
status: draft
title: 'A transition taken by another command''s `affects: moves:` has a cause'
summary: A lifecycle transition moved only as a side effect is refused as missing_causation (workbench report item 39)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

A transition that only another command's `affects:` entry takes is refused: `ESS-ENTITY-005
\`close\` moves \`gap.demo.Parent\` to \`Closed\`, and no command outcome takes it`
(`missing_causation`). Item 39 of the 2026-11-09 workbench report; example `39`. Item 40 of the same
report (create one row and update another in one outcome) is story:create-beside-a-move-on-the-same-key.

## Acceptance

- An `affects:` entry whose `moves:` takes a transition counts as that transition's cause.
- A transition no outcome and no `affects:` entry takes is still refused.

## Scope

- inferred: `crates/specify/ess-domain` (lifecycle causation, `ESS-ENTITY-005`).

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
