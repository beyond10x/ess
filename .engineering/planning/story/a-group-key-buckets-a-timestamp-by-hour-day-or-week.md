---
format: aep.planning-md/3
id: story:a-group-key-buckets-a-timestamp-by-hour-day-or-week
kind: story
status: draft
title: A group key buckets a Timestamp by hour, day or week
summary: There is no time bucket for an aggregate group key (workbench report item 15)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: story:aggregate-group-keys-are-expressions
revision: 1
---
## Need

An aggregate view cannot group by a time bucket of a Timestamp, such as items per week. A guess
`expr: week(due)` gives `ESS-VIEW-009 ... \`week\` computes from one row; arithmetic over measures is
not admitted yet`. Item 15 of the 2026-11-09 workbench report; example `15`.
story:aggregate-group-keys-are-expressions makes a group key an expression, and a bucket function
fits there.

## Acceptance

- A group key may be a Timestamp truncated to an hour, a day or an ISO week, in UTC unless a zone
  is declared.
- The interpreted target, synthesis and generated queries bucket alike, and synthesis puts rows in
  two buckets and at a bucket boundary.

## Scope

- inferred: `crates/specify/ess-domain/src/view.rs` (group keys), the expression functions,
  `crates/verify/ess-conformance/src/interpret/views.rs`, `crates/generate/ess-synth/src/view_query.rs`.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
