---
format: aep.planning-md/3
id: story:a-view-field-computes-from-now-and-from-a-view-parameter
kind: story
status: draft
title: A computed view field reads now and the view's parameters
summary: '`due - now` and `param.k` are refused in computed view fields (workbench report item 12)'
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: story:a-suite-fixes-the-targets-clock
revision: 1
---
## Need

A view cannot publish a waiting time or a value relative to its parameters:

- `due - now`: `ESS-VIEW-009 \`due - now\` reads \`now\` as a value; a value read at the current time
  has no clock a suite can fix yet` (example `12a`).
- `param.k` in a computed field: `ESS-VIEW-001 \`param.k\` is no field of \`gap.demo.Item\`` and
  `ESS-VIEW-003` (example `12b`).

Item 12 of the 2026-11-09 workbench report. Suites can already fix the target's `now` for a step
(story:a-suite-fixes-the-targets-clock), so the clock reason may no longer hold.

## Acceptance

- A computed view field may read `now`. Synthesis fixes the clock, so the expected value is
  deterministic.
- A computed view field may read a declared view parameter.
- Generated queries compute both, or keep the view an obligation naming the read.

## Scope

- inferred: `crates/specify/ess-domain/src/view.rs`, the expression typing, synthesis view
  expectations, `crates/generate/ess-synth/src/view_query.rs`.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
