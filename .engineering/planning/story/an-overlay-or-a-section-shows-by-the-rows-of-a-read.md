---
format: aep.planning-md/3
id: story:an-overlay-or-a-section-shows-by-the-rows-of-a-read
kind: story
status: draft
title: An overlay or a section appears and disappears by the rows of a read, without a user action
summary: An overlay opens only from an action and a visible expression reads no rows, so a prompt the server raises cannot appear and go by data (a consumer's UI report, item 3)
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

Some things on screen exist because of data, not because the user asked: an incoming request that
waits for an answer, a banner while a job is failing, a prompt another party raised. They appear
when a row exists (or a live event arrives) and go when it is gone.

`ess-ui/1` has two places where this could be said, and neither says it:

- an overlay is "shown on demand" and opened by an action (`opens`, `confirm`); its `visible` is an
  extra condition and opens nothing;
- a section's `visible` is an expression, and the expression forms
  (`website/docs/reference/ess-ui.md`, Binding expressions) read state, params, a row inside a
  collection, a selection, a channel field, shell state and the actor. None reads whether a read
  has rows, or a field of a row of another section.

Evidence: a consumer's UI document (2026-11-13, ess 0.94.0), item 3 of its collected report.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`), a page `requests.inbox` whose section `list` is a `collection` over that view, and a page
`requests.detail` with a param `id`. `ess ui check --model` on the base: `0 error(s), 0 warning(s)`.

Reproduction 1, a page overlay `prompt` (`kind: dialog`, `component: record` over
`PendingRequests`) with `visible: section.list.rows.count > 0`, and a journey test that opens the
page and expects the overlay's text:

```
ui check: 0 error(s), 0 warning(s)
failed ... expect at pages/requests.inbox/overlays/prompt: overlay prompt is not open
```

Reproduction 2, a section `banner` (`component: record` over the same view) with
`visible: row.state == Pending`, and the same with `visible: section.list.row.state == Pending`:

```
ui check: 0 error(s), 0 warning(s)
failed ... expect at pages/requests.inbox/sections/banner: section banner is not drawn
```

In all three the checker accepts an expression that no renderer can evaluate to true. That part is
the subject of `story:ui-check-resolves-the-root-and-section-of-every-binding-path` and
`story:a-visible-expression-resolves-its-row-and-actor-paths-wherever-an-action-stands`; this draft
is the missing capability, not the missing finding.

## What a design has to settle

- One spelling for "this read has a row" (and "has no row") that `visible` can use, on a section
  and on an overlay, and whether an overlay that is visible by data needs an opener at all.
- The read it stands on: the node's own `reads`, another section's, or a channel; and what a live
  event that removes the row does to an overlay the user is typing in.
- Loading and error states: a node that shows by data is neither shown nor hidden while its read
  is loading or has failed.
- The terminal renderer, the React project and `ess-ui-test/1` (`expect` at an overlay nothing
  opened).

## Acceptance (sketch, to be fixed by the fit review)

- With one row in the fixture the prompt is drawn on open; a played event (or a fixture change)
  that removes the row removes the prompt; with no row it is never drawn.
- The same holds for a section.
- An expression of this kind that names a read the node cannot see is refused by `ess ui check`
  with the node path.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.
