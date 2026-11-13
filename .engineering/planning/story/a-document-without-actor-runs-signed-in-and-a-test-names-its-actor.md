---
format: aep.planning-md/3
id: story:a-document-without-actor-runs-signed-in-and-a-test-names-its-actor
kind: story
status: draft
title: A document without actor runs signed in as the reference says, and a test names its actor
summary: 'The terminal signs in only a document that writes actor: from_session, the reference''s default; ess-ui-test/1 has no key for the actor (a consumer''s UI report, second list, item 10)'
tags:
- backlog
- defect
- ess-ui
- external-request
- from-consumer-report
- severity-medium
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
- informed_by: story:an-actor-expression-reads-the-fields-of-the-session-view
revision: 1
---
## Defect

`ess-ui-test/1` gives a test no way to say who is signed in, and the one route the reference names
does not work for a document that leaves `actor` out.

Evidence: a consumer's UI document (2026-11-13, ess 0.94.0), item 10 of its second collected report.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`, no parameter), a page `requests.inbox` whose section `list` is a `collection` over that
view with the row action `approve`, and a page `requests.detail` with a param `id`. Fixture rows
`r1` (`request-one`) and `r2` (`request-two`). `ess ui check --model` on the base:
`0 error(s), 0 warning(s)`.

1. The reference gives the document's `actor` the default `from_session`
   (`website/docs/reference/ess-ui.md`, section Document: `actor`, one of `from_session`,
   `anonymous`, default `from_session`). The terminal renderer signs the actor in only when the
   document writes the key: `crates/ui/ess-ui-tui/src/app.rs:4565-4566` sets `signed_in` to
   `app.doc.actor == Some(ess_ui::ActorSource::FromSession) && !app.signed_out`.

   Reproduction: the base with a shell preload of the view and a section child
   `{name: who, primitive: text, text: actor.signed_in}`. Without `actor:` the child shows `false`;
   with `actor: from_session` it shows `true`. With the shell guard
   `{name: signed_in, when: not actor.signed_in, then: {redirect: requests.detail}}` and no `actor:`
   key, a test's first step fails: `opening requests.inbox shows requests.detail instead`.

2. A test cannot name its actor.

   ```
   tests[1] (a test names its actor at the test): unknown key `actor`; expected one of name, fixtures, latency, steps
   tests[1] (a test names its actor under fixtures): fixtures: unknown key `actor`; expected one of views, commands, scripts
   ```

   What works today (`website/docs/reference/ess-ui-test.md`, section Fixtures): the actor of a
   fixture run is the first row of the first view the home page's shell preloads, so a test that
   replaces that view's rows under `fixtures.views` replaces the actor (shown with `user_id`,
   `account_id`, `roles` and a further field in
   `story:an-actor-expression-reads-the-fields-of-the-session-view`). A document whose shell
   preloads nothing has no actor in a test, and no test can run signed out, or as a second actor
   with other grants, without replacing a view's rows that the pages may also show.

## What the fit review has to settle

- Point 1 is a renderer that disagrees with the reference: the acceptance is the reference's
  default, in the terminal and, where it differs, in the React project.
- Point 2 is a request: whether a test names its actor (`signed_in`, the fields, the grants `may`
  answers), where (the test, its `fixtures`), and how that stays one verdict in both renderers. The
  review may split it off.

## Acceptance (sketch, to be fixed by the fit review)

- A journey test, red on 0.94.0: a document without `actor:` whose shell preloads a session view
  shows `actor.signed_in` as `true`, and a guard on `not actor.signed_in` does not redirect.
- A test runs the same page as a signed-in actor and as a signed-out one without touching the rows
  the page shows.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the acceptance is fixed there.
