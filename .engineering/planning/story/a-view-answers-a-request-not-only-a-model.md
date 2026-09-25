---
format: aep.planning-md/2
id: story:a-view-answers-a-request-not-only-a-model
kind: story
status: archived
title: A view answers a request, not only a model
relations:
- decomposes: epic:specification-runs-as-a-fake-backend
revision: 3
---
## The gap

`ViewSpec` carries `source`, a static `filter`, `order_by`, `fields` and `consistency`
(`crates/specify/ess-domain/src/view.rs:297-329`). It takes no parameters, so a view cannot express:

- `?page=2&max=50` — every list endpoint in the adopting project
- `?q=support` — search
- by-id — `GET /queues/{id}`
- scoping to the caller — every endpoint, which pins `customer` on every list

## What that cost

All four became hand-written JavaScript sitting between the module and the browser: a paginator, an
id resolver, and an account filter re-derived per request. A defect hid there for three rounds —
`/api/v2/queues` was never scoped to a customer at all, though the controller it was lifted from
pins one, and no test caught it because no test had two accounts.

That defect is the argument. A hand-written filter is a filter nothing validates against the model.

## Acceptance

A view may declare typed inputs — at minimum a page window, an equality filter on a projected field,
and selection by the source's identity — the generated query obligation takes them as arguments, and
the conformance suite exercises them.

## Smaller alternative, if the above is too much

Declaring only the page window would remove the largest single piece of hand-written code, because
every list endpoint has one and they are all the same.

## Retirement under the revised ESS evolution scope

The operator explicitly excluded this epic and all twelve dedicated stories on 2026-09-15 in approved plan ess-evolution-20260915 revision 1. Retire this draft through AEP without deleting its original content, relations or journal history. Its acceptance is no longer a requirement of the current initiative. Generic future protocol/UI work is recorded separately in task:deferred-protocol-ui-bindings and does not reopen this artifact.
