---
format: aep.planning-md/3
id: story:a-host-slot-in-a-collection-item-is-handed-its-row
kind: story
status: draft
title: A host slot standing in a collection's item is handed its row
summary: A host slot takes reads and commands only; inside a collection's item it gets no row, so a host-drawn tile cannot be told which row it draws (a consumer's UI report, item 2)
tags:
- backlog
- ess-ui
- external-request
- feature-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
revision: 1
---
## Need

A collection drawn as cards often needs one card body the generator cannot draw yet (a player, a
map, a bespoke tile). The document declares it as a host slot inside the collection's `item`. The
host component then has to draw one row, and nothing hands it the row: a host slot declares
`host`, `reason`, `stands_in_for`, `reads` and `commands`
(`website/docs/reference/ess-ui.md`, section HostSlot), while a widget instance in the same place
takes `args: {partner: row}`.

Evidence: a consumer's UI document (2026-11-13, ess 0.94.0), item 2 of its collected report.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`), a page `requests.inbox` whose section `list` is a `collection` over that view, and a page
`requests.detail` with a param `id`. `ess ui check --model` on the base: `0 error(s), 0 warning(s)`.

Reproduction, the collection in `style: cards` with
`item: [{name: tile, host: RequestTile, reason: a bespoke tile, stands_in_for: none, args: {request: row}}]`:

```
error host_slots pages/requests.inbox/sections/list: `collection`: host slot `RequestTile`: unknown field `args`, expected one of `host`, `reason`, `stands_in_for`, `reads`, `commands`
```

Without `args` the slot loads (`host pages/requests.inbox/sections/list/item/tile RequestTile
stands in for none`). A slot's `reads.params` may be written with `row.id`: the checker resolves it
and refuses it only where the view declares no such parameter. So today a slot in an item can make
its own read per row, one request per card, and cannot be given the row the collection already
holds.

## What a design has to settle

- Whether a host slot takes `args` as a widget instance does (typed against what: the slot declares
  no parameters today), or receives the row of its position without declaring anything.
- The same question at the slot's other scopes: a slot in a record, in an overlay opened from a row,
  in a reused template (`story:a-host-slot-in-a-reused-template-is-one-registration`).
- What the host registration receives in the React project, what the terminal renderer draws in the
  slot's place, and how `ess ui check` counts a slot that draws a row (it stays uncovered).

## Acceptance (sketch, to be fixed by the fit review)

- The document above, in the chosen form, loads and checks clean, and the generated React project
  passes each tile's row to the registered component without a second read.
- A value the slot is handed that the row does not have is refused by `ess ui check --model`, by
  node path.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.
