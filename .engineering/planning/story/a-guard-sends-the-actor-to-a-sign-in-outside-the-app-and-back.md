---
format: aep.planning-md/3
id: story:a-guard-sends-the-actor-to-a-sign-in-outside-the-app-and-back
kind: story
status: draft
title: A guard sends the actor to a sign-in outside the app and back to the link they opened
summary: A guard's then is redirect to a page, refuse or set; sign in through an identity provider and return to this link cannot be said (a consumer's UI report, item 5)
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

An application whose users sign in through an identity provider has no sign-in page of its own. A
user who opens a deep link without a session must be sent to the provider and, after signing in,
land on the link they opened.

A guard's `then` is one of three (`website/docs/reference/ess-ui.md`, section Guard): `redirect`
to a page of the document with `params`, `refuse` with a text, or `set` state. A redirect target
outside the document cannot be named, and nothing carries "the place the actor wanted" across the
round trip. The consumer's document keeps a page that exists only to leave the app from host code.

Evidence: a consumer's UI document (2026-11-13, ess 0.94.0), item 5 of its collected report.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`), a page `requests.inbox` whose section `list` is a `collection` over that view, and a page
`requests.detail` with a param `id`. `ess ui check --model` on the base: `0 error(s), 0 warning(s)`.

Reproduction, a shell guard
`{name: signed_in, when: not actor.signed_in, then: {redirect: "https://sso.example/authorize", params: {return_to: url.query.next}}}`:

```
error page_refs shells/app/guards/signed_in/then: `https://sso.example/authorize` names no page
```

and with a `then` of another shape (`{external: "https://sso.example/authorize"}`):

```
error document_loads shells/app: data did not match any variant of untagged enum GuardThen
```

## What a design has to settle

- Whether the document names an address at all, or names a sign-in that the host or the HTTP binding
  supplies (an address in a renderer-neutral document is deployment configuration;
  `story:a-generated-ui-reads-the-answers-its-http-binding-declares` already has the UI renew its
  credential on a 401).
- How "the link the actor opened" is kept and restored: which value the guard may read for it, and
  that a restored link is a page of this document and nothing else (an open redirect otherwise).
- What the terminal renderer and `ess-ui-test/1` do at such a guard (a journey test must be able to
  say "signed out, opens the link, is sent to sign in, comes back to the link").

## Acceptance (sketch, to be fixed by the fit review)

- A document states the requirement without a page that exists only to leave the app.
- A journey test opens a deep link signed out, observes the sign-in hand-over, completes it, and
  reads the page of the link with its parameters.
- A return target that is no page of the document is refused.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.
