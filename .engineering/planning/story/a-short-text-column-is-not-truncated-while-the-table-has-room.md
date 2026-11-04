---
format: aep.planning-md/3
id: story:a-short-text-column-is-not-truncated-while-the-table-has-room
kind: story
status: active
title: A short text column is not truncated while the table has room
summary: A collection table truncates a short text column while other columns have room to give (adopter item B8-11)
tags:
- ess-w1-0870
- from-consumer-report
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T10:58:25Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-04T10:58:25Z", actor: "human:timo", revision: 3}
---
## Outcome

A short text column in a collection table shows its whole value whenever the table has room for it.

## Context

Adopter report B8-11: short text columns are truncated with an ellipsis while other columns
of the same table have room to give. The table layout is in `crates/ui/ess-ui-react/templates/runtime/collection.tsx.tmpl`
and `styles.css`.

## Acceptance

Named test, red first: a table with one short text column and wide room does not truncate that column. Long
values still truncate as they do today.

## Out of scope

- Column width declarations in the ess-ui/1 grammar.

## Schedule

Ride-along of the 0.87.0 wave, after U4 merges (it shares the collection templates).
