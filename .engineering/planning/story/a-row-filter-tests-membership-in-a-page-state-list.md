---
format: aep.planning-md/3
id: story:a-row-filter-tests-membership-in-a-page-state-list
kind: story
status: draft
title: A row filter tests membership in a list held in page state
summary: '`row.id in state.picked` is refused; `in` takes literal lists only (workbench report item 3)'
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 1
---
## Need

A collection cannot filter its rows by a multi-select held in page state. `row.id in state.picked`
is refused: `filter_expr ... a filter must parse as a comparison, membership, not, and or or
expression`, because `in` takes literal lists only. `state.picked contains row.id` passes the check,
but `contains` is a text operator, and its runtime meaning is untested. Item 3 of the 2026-11-09
workbench report; examples `ui03`, `ui03b`.

## Acceptance

- A row filter may test a row value's membership in a page-state list: `row.<field> in
  state.<list>`. The check types the element against the list, and React and the terminal filter
  by it.
- `contains` over a list is either the same membership test or refused by name. It is never
  accepted as a text operator over a list.

## Scope

- inferred: `crates/ui/ess-ui` (filter expression parser and typing), `crates/ui/ess-ui-check`,
  the React and terminal filter runtimes.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
