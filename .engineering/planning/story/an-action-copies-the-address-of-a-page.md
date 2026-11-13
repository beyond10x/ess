---
format: aep.planning-md/3
id: story:an-action-copies-the-address-of-a-page
kind: story
status: draft
title: An action copies the address of a page of the app
summary: copy takes an expression and no expression gives the address a navigate would open, so a link to a page is written by hand from parts (a consumer specification's UI list, item 3)
tags:
- backlog
- ess-ui
- external-request
- feature-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
- informed_by: story:ess-ui-check-judges-navigation-and-overlay-parameters-by-their-use
revision: 1
---
## Need

A user copies a link to a thing and sends it to someone: the address of the page of this request,
this order, this conversation. `ess-ui/1` has a `copy` action whose value is an expression, and a
`navigate` action that names a page and its params. Nothing gives the address a `navigate` would
open, so `copy` cannot take it. The author writes the route by hand out of parts, which the
expression language cannot join (`story:a-ui-expression-joins-texts`) and whose first part, the
address the app is served at, no expression reads
(`story:a-ui-expression-reads-a-value-the-deployment-sets`). A link written by hand also breaks
when the page's route or params change, and the check cannot see it.

Evidence: a consumer specification's UI document (2026-11-13, ess 0.94.0), item 3 of its third collected list.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`, no parameter), a page `requests.inbox` whose section `list` is a `collection` over that
view with the row action `approve`, and a page `requests.detail` with a param `id`. Fixture rows
`r1` (`request-one`) and `r2` (`request-two`). `ess ui check --model` on the base:
`0 error(s), 0 warning(s)`. Each reproduction adds one row action
`{name: link, copy: "<expression>", label: Copy the link}` to the section `list`.

Reproduction 1, `copy: "page_address(requests.detail, {id: row.id})"`: `ess ui check --model`
prints `0 error(s), 0 warning(s)` and the terminal's notice after `act` is `copied` with an empty
value (the check's part is
`story:ui-check-reports-an-expression-that-reads-a-path-and-is-no-expression`).

Reproduction 2, `copy: {navigate: {to: requests.detail, params: {id: row.id}}}`:

```
error document_loads pages/requests.inbox/sections/list/row_actions/link: an expression is a scalar, not Mapping {"navigate": Mapping {"to": String("requests.detail"), "params": Mapping {"id": String("row.id")}}}
```

## What a design has to settle

- The form: a `copy` that takes the page and params a `navigate` takes, or an expression form that
  gives a page's address anywhere a text stands. The first adds no expression; the second also
  serves a link shown as text or handed to a host slot.
- What the address is in each renderer. The generated React project has a route per page and an
  origin; the terminal has neither. The terminal's notice and a test need one stated form (the
  route with its params, without an origin).
- Whether the address is absolute. A link sent to another person needs the origin the app is
  served at; the browser knows it at run time, a base path is known at build time
  (`story:a-generated-react-app-runs-under-a-base-path`).
- Query state: whether a page's shareable state (a filter, a tab) is part of the address, as the
  reference's `url` durability already puts it there.
- What `ess ui check` resolves: the page exists, every required param is given, each param's
  expression resolves, as for `navigate`
  (`story:ess-ui-check-judges-navigation-and-overlay-parameters-by-their-use`).

## Acceptance (sketch, to be fixed by the fit review)

- A document states "copy the address of page P with these params" once. The generated React
  project copies the absolute address that opens that page with those params; the terminal's
  notice and a test read the same route and params. `ess ui check` refuses an unknown page and a
  missing or unresolvable param.
- A page whose route changes needs no change in the action.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.
