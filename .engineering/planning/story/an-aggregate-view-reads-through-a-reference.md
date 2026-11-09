---
format: aep.planning-md/3
id: story:an-aggregate-view-reads-through-a-reference
kind: story
status: draft
title: An aggregate view and a generated query read through a reference
summary: Aggregate views refuse fields read through a reference; generated queries keep such reads as obligations (workbench report item 13, new on 0.92.0)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

- An aggregate view cannot read through a reference: `view.gap.demo.ItemsByParent.fields: ... reads
  through a reference of \`gap.demo.Item\`, and an aggregate view is not read through one in this
  cut` (item 13, example `13`).
- In generated Rust queries, reads through a reference (`field: {related: {via, field}}`) are
  obligations in 5 views of the reporter's model (new on 0.92.0).

Next to #564, following a reference in guards, values, effects and view fields, whose text does not
name aggregate views or generated queries. From the 2026-11-09 workbench report.

## Acceptance

- An aggregate view may group by or measure a field read through a reference of its source.
- A generated Rust view query applies a read through a reference instead of leaving an obligation.
- Synthesis witnesses both.

## Scope

- inferred: `crates/specify/ess-domain/src/view.rs` (aggregate validation),
  `crates/generate/ess-synth/src/view_query.rs`, `crates/verify/ess-conformance` (view expectations).

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
