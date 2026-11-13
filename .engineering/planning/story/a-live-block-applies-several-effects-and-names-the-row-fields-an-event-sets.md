---
format: aep.planning-md/3
id: story:a-live-block-applies-several-effects-and-names-the-row-fields-an-event-sets
kind: story
status: draft
title: A live block applies an effect per event and names the row fields an event sets
summary: Live takes one effect for all its events and has no mapping from event fields to row fields (a consumer's UI report, second list, item 3)
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

One channel carries several events about the rows of one section, and each does something else to
them: a "created" event inserts a row, a "changed" event patches one, a "closed" event removes it.
And an event's payload does not spell its fields as the view's row does: the event says `subject`
where the row says `title`, or carries the row's identity under another name than the row's key.

`ess-ui/1` cannot say either. A section has one `live` block, the block has one `effect` for every
event it lists in `on`, and it has no key that maps event fields to row fields: `match` names the
row's identity field only (section Live of `website/docs/reference/ess-ui.md`: `channel`, `on`,
`effect`, `match`, `only_if`, `coalesce`, `when_paged_away`, `max_rows`). A consumer's document
therefore uses `effect: refetch` for every event, which reads the whole view again per event, or
declares one channel per effect.

Evidence: a consumer's UI document (2026-11-13, ess 0.94.0), item 3 of its second collected report.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`, no parameter), a page `requests.inbox` whose section `list` is a `collection` over that
view with the row action `approve`, and a page `requests.detail` with a param `id`. Fixture rows
`r1` (`request-one`) and `r2` (`request-two`). `ess ui check --model` on the base:
`0 error(s), 0 warning(s)`.
The base gets a channel `requests` that carries `desk.requests.RequestApproved`
(`direction: server_to_client`, `delivery: every_event`, `resume: refetch`). With
`live: {channel: requests, effect: remove_row}` on the section the document checks with one warning
(`script_per_channel`, no fixture script).

Reproduction 1, `effect: [remove_row, refetch]`:

```
error document_loads pages/requests.inbox/sections/list: data did not match any variant of untagged enum Effect
```

Reproduction 2, `live` as a list of two blocks (`{channel: requests, on: [desk.requests.RequestApproved], effect: remove_row}` and `{channel: requests, effect: refetch}`):

```
error document_loads pages/requests.inbox/sections/list: invalid type: sequence, expected struct Live
```

Reproduction 3, `live: {channel: requests, effect: patch_row, map: {title: event.subject}}` (the same
with `fields:` for `map:`):

```
error document_loads pages/requests.inbox/sections/list: unknown field `map`, expected one of `channel`, `on`, `effect`, `match`, `only_if`, `coalesce`, `when_paged_away`, `max_rows`
```

## What a design has to settle

- Whether the model already says it. An ESS view is a projection of entities, and an event's
  payload is declared; which row an event changes and what it sets may be derivable from the model
  (the outcome that emits the event names the entity it changed), as an action needs no reload
  declared today. If so, the answer is a derived effect and no authored mapping.
- If authored: several blocks per section or an effect per event inside one block; what a mapping
  may read (event fields only) and write (row fields of the read's view), and that
  `ess ui check --model` resolves both sides against the event and the view.
- What an event does that names a row the section does not hold, per effect.
- What the terminal renderer, the React project and a fixture script in `ess-ui-test/1` do with
  it: `play` of each event and `expect rows` after it.
- Reproductions 1 and 2 are refused with a Rust type name (`untagged enum Effect`,
  `struct Live`) and not with the values or the shape the key takes.

## Acceptance (sketch, to be fixed by the fit review)

- A section fed by one channel inserts on one event, patches on a second and removes on a third,
  with no `refetch`; a test plays the three events and holds the rows after each.
- An event whose field names differ from the row's sets the row's fields; a mapping that names no
  field of the event or of the view is an error of `ess ui check --model`.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.
