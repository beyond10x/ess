---
format: aep.planning-md/3
id: story:an-actor-expression-reads-the-fields-of-the-session-view
kind: story
status: draft
title: An actor expression reads the fields of the session view, a display name among them
summary: The reference lists user_id, account_id, roles, signed_in and may; any field of the preloaded session row is readable and none is checked (a consumer's UI report, second list, item 5)
tags:
- backlog
- ess-ui
- external-request
- feature-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
- informed_by: story:a-visible-expression-resolves-its-row-and-actor-paths-wherever-an-action-stands
revision: 1
---
## Need

A page greets its user, a header shows who is signed in, a row says "assigned to you (Ada)". The
document needs the signed-in actor's display name, and often an avatar address or a locale.

The reference gives `actor.<field>` as a closed list: `user_id`, `account_id`, `roles`,
`signed_in`, `may(<command>)` (`website/docs/reference/ess-ui.md`, section Expressions). A consumer
read that as "nothing gives the display name".

Evidence: a consumer's UI document (2026-11-13, ess 0.94.0), item 5 of its second collected report.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`, no parameter), a page `requests.inbox` whose section `list` is a `collection` over that
view with the row action `approve`, and a page `requests.detail` with a param `id`. Fixture rows
`r1` (`request-one`) and `r2` (`request-two`). `ess ui check --model` on the base:
`0 error(s), 0 warning(s)`.

What 0.94.0 does (the base with `actor: from_session`, a shell preload of the view, and a section
child `{name: who, primitive: text, text: actor.<field>}`; the test replaces the preloaded view's
first row by `{id: r1, …, user_id: u-7, account_id: acc-3, display_name: Ada Example, roles: [approver]}`
and reads the child's text):

- `actor.user_id` shows `u-7`, `actor.account_id` shows `acc-3`, `actor.roles` shows `approver`;
- `actor.display_name` shows `Ada Example`: the actor is the first row of the first view the home
  page's shell preloads (`website/docs/reference/ess-ui-test.md`, section Fixtures), and every field
  of that row is readable;
- without a preload every one of them shows nothing;
- `ess ui check --model` prints `0 error(s), 0 warning(s)` for `actor.user_id`,
  `actor.display_name`, `actor.name` and `actor.email` alike, although the view has none of these
  fields. No check reads an `actor.` path.

So the display name is reachable and nothing says so, nothing checks it, and the reference's list
reads as closed. `story:a-visible-expression-resolves-its-row-and-actor-paths-wherever-an-action-stands`
(active) makes a name outside that list an error; with it, the display name stops being reachable
unless this is settled first. A note there says so.

## What a design has to settle

- What the actor is: the row of a declared session view (then `actor.<field>` resolves against that
  view's fields with `--model`, the reference names the view, and `user_id`, `account_id` and
  `roles` are fields the view must have), or a closed list that grows by `display_name` and what
  else a UI shows of its user.
- Where a document names its session view. Today it is "the first view the home page's shell
  preloads", a position, not a declaration.
- What a served build reads the actor from, and that both renderers agree with the fixture run.

## Acceptance (sketch, to be fixed by the fit review)

- The reference says which fields `actor.` has and where they come from, and a document shows the
  signed-in actor's display name in both renderers.
- `ess ui check --model` reports an `actor.<field>` the session view (or the closed list) does not
  have, naming the fields it has.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.
