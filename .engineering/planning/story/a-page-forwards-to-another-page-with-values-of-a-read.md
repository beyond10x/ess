---
format: aep.planning-md/3
id: story:a-page-forwards-to-another-page-with-values-of-a-read
kind: story
status: draft
title: A page forwards to another page with values of a read
summary: A page has no effect on load; a shell guard forwards to a fixed page only, and its params read no view (a consumer's UI report, second list, item 2)
tags:
- backlog
- ess-ui
- external-request
- feature-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
- informed_by: story:a-guard-sends-the-actor-to-a-sign-in-outside-the-app-and-back
revision: 1
---
## Need

A link page: the address a mail, a chat message or another system carries names a thing by a value
the app's pages are not keyed by (a token, a number of another system, "my current item"). The page
reads one row for that value and forwards to the page of the row, with the row's identity as the
param. The user never sees the link page.

`ess-ui/1` has no place for it. A page has no effect on load (section Page of
`website/docs/reference/ess-ui.md`: `kind`, `shell`, `title`, `nav`, `params`, `aliases`,
`switch_to`, `visible`, `actor`, `layout`, `state`, `header`, `sections`, `overlays`, `profile`,
`source`, `unmapped`). `aliases` forwards a legacy path to the page itself. A shell guard forwards
before a page opens, to a page and with params (section Guard), and its expressions read the actor,
the address and the page, no view.

Evidence: a consumer's UI document (2026-11-13, ess 0.94.0), item 2 of its second collected report.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`, no parameter), a page `requests.inbox` whose section `list` is a `collection` over that
view with the row action `approve`, and a page `requests.detail` with a param `id`. Fixture rows
`r1` (`request-one`) and `r2` (`request-two`). `ess ui check --model` on the base:
`0 error(s), 0 warning(s)`.

Reproduction 1, `on_load: {navigate: {to: requests.inbox}}` on the page `requests.detail`:

```
error document_loads pages/requests.detail: unknown field `on_load`, expected one of `kind`, `shell`, `title`, `nav`, `params`, `aliases`, `switch_to`, `visible`, `actor`, `layout`, `state`, `header`, `sections`, `overlays`, `profile`, `source`, `unmapped`
```

What works today: the shell guard
`{name: forward, when: page == requests.detail, then: {redirect: requests.inbox}}` checks clean and
forwards. A test that opens `requests.detail` is answered
`opening requests.detail shows requests.inbox instead`. So a forward to a fixed page is expressible;
a forward whose target param comes from a read is not.

Reproduction 2, the guard
`{name: forward, when: url.query.go, then: {redirect: requests.detail, params: {id: row.id}}}`:
`ess ui check --model` prints `0 error(s), 0 warning(s)`, although a guard has no row. That part is
a case of `story:ui-check-resolves-the-root-and-section-of-every-binding-path` and is noted there.

## What a design has to settle

- Where it stands: a guard that may read one view before it decides, a page kind whose only
  content is one read and a target, or an effect of a page. A guard runs before the page opens and
  has no loading state; a read takes time and may fail, be forbidden or return no row.
- What the user sees while the read runs and when it returns no row or fails: the link page's own
  states, as a section has them.
- That the forward replaces the link page in the history, so "back" does not forward again.
- What `ess ui check --model` resolves (the read, its params, the row member the target param
  takes) and what `ess-ui-test/1` pins: `open` of the link page is expected to show the target page
  with its params, which today's `open` step reports as a failure.

## Acceptance (sketch, to be fixed by the fit review)

- A document states "open `<link page>` with a value, read one row by it, show `<page>` of that
  row"; both renderers do so; a read with no row shows the declared state and forwards nowhere.
- A test opens the link page and holds on the target page and its params.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.
