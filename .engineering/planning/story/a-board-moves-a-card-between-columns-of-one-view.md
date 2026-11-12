---
format: aep.planning-md/3
id: story:a-board-moves-a-card-between-columns-of-one-view
kind: story
status: active
title: A board draws columns over one view and moves a card between them
summary: Reorder across groups sends no target group; there is no kanban composite (workbench report items 4, 5)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:48:03Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
- {from: "proposed", to: "active", at: "2026-11-12T21:48:04Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
---
## Need

A kanban, cards in columns by a group field and dragged between columns, cannot be drawn. `board` is a
widget dashboard, not columns over one view. `group_by` with `reorder`, which the reference text
calls a kanban, sends `{id, delta}`, and `dropDelta` returns `undefined` when source and target
groups differ, so a drop on another column does nothing and nothing refuses it. Items 4 and 5 of the
2026-11-09 workbench report; example `ui04`.

## Acceptance

- A collection grouped by a field can declare a move command that takes the target group. A drop
  on another group sends that command with the row and the target group's value, and the check
  holds the command's inputs to the group field.
- A reorder within one group keeps sending `{id, delta}`.
- A drop across groups with no declared move command is refused by `ess ui check`, never ignored
  at run time.

## Scope

- inferred: `crates/ui/ess-ui` (collection grouping, reorder), `crates/ui/ess-ui-check`,
  the React collection runtime (`dropDelta`), the terminal renderer.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.

## Decisions

**Accepted under the operator's procedure for external-request drafts (2026-11-12).** He set
it that day: the session that maintains the repository recommends per draft, an independent
adversarial review reads each draft, a draft both read as accept is accepted on his behalf, and
a draft where the two differ goes to him. This draft was number 33 of a second list of 13
(numbers 26 to 38); both read accept. The two transitions carry the correlation
`operator-2026-11-12-round-2` and the executor `agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).

Question carried into the fit review, not a condition of the acceptance: whether acceptance
point 3, read literally, refuses documents in use that combine `group_by` and `reorder` and
declare no move command, and what such a document is told.
