---
format: aep.planning-md/3
id: story:a-field-draws-a-video
kind: story
status: draft
title: A field draws a video, as a field draws audio
summary: Field.as lists audio and no video (a consumer's UI report, smaller item)
tags:
- backlog
- ess-ui
- external-request
- feature-request
- from-consumer-report
- severity-low
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
revision: 1
---
## Need

A record or a column that holds a recording draws a player. `Field.as` has `audio` and no `video`
(`website/docs/reference/ess-ui.md`, the `as` row of Field), so a video recording is a host slot or
a plain link.

Evidence: a consumer's UI document (2026-11-13, ess 0.94.0), a smaller item of its collected report.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`), a page `requests.inbox` whose section `list` is a `collection` over that view, and a page
`requests.detail` with a param `id`. `ess ui check --model` on the base: `0 error(s), 0 warning(s)`.

Reproduction, a record section with `fields: [{field: title, as: video}]`:

```
error enum_values pages/requests.inbox/sections/media/fields/title: `video` is not a value of `as`; the schema's `Field.as` lists audio, badge, bytes, choice, color, cron, date, date_range, duration, file, json, link, list, map, number, percent, rich_text, secret, tags, text, time, timestamp, toggle
```

The same field with `as: audio` checks clean.

## What a design has to settle

- Whether `video` is one more value beside `audio`, with the same source rule (what the field's
  value is: an address, a file reference), or whether both become one media value with a kind.
- What the terminal renderer draws for it and what `degrades` says where a renderer cannot play.

## Acceptance (sketch, to be fixed by the fit review)

- The field above checks clean, the React project draws a player, and the terminal draws what it
  draws for `audio`.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.
