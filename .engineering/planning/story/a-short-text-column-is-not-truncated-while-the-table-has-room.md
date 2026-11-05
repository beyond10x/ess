---
format: aep.planning-md/3
id: story:a-short-text-column-is-not-truncated-while-the-table-has-room
kind: story
status: active
title: A short text column is not truncated while the table has room
summary: A collection table truncates a short text column while other columns have room to give (adopter report)
tags:
- ess-w1-0870
- from-consumer-report
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T10:58:25Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-04T10:58:25Z", actor: "human:timo", revision: 3}
---
## Outcome

A short text column in a collection table shows its whole value whenever the table has room for it.

## Context

Adopter report adopter report: short text columns are truncated with an ellipsis while other columns of the same table
have room to give. The report named the React collection template and styles. Reading them while implementing
(0.87.0 wave) found no cell truncation rule there: `.ui-table` is `width: 100%` with no fixed layout, and the only
ellipsis rule is on chart axis labels. The terminal renderer (`crates/ui/ess-ui-tui/src/view.rs`) capped every
column at 28 cells whatever the box width, which produces the reported symptom. Which renderer the adopter saw
is inferred from the ellipsis, not confirmed with them.

## Acceptance

Named test, red first: a table with one short text column and wide room does not truncate that column. Long
values still truncate as they do today.

## Out of scope

- Column width declarations in the ess-ui/1 grammar.

## Schedule

Ride-along of the 0.87.0 wave, after U4 merges (it shares the collection templates).

## Browser side (adopter answer)

The adopter answered on 2026-11-05: their adopter report screen was the browser. In headless Chrome at 1440 px, an
11-column table measured 1357 px inside a 1114 px box. Every cell was `white-space: nowrap`, so the box scrolled
and a downstream adopter's theme, which pins one column, covered the last columns.

Checked against the stock runtime (`git grep white-space` over `crates/ui` at tag 0.85.0 and at the 0.87.0
integration head): neither sets `white-space: nowrap` on a table cell. The only `nowrap` rule in the React
runtime is on chart axis labels (`.ui-chart-x-label`), and `.ui-table` is `width: 100%` with automatic layout. The
nowrap the adopter measured comes from their theme (their own diff notes a theme rule
`.ui-table td { white-space: nowrap }`). With the stock stylesheet, text cells already wrap at spaces.

So the browser side needs no upstream change: the stock runtime already does what the overlay restores. The
story is complete with the terminal fix (99c4939b89).
