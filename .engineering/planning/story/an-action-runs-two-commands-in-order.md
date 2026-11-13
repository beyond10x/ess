---
format: aep.planning-md/3
id: story:an-action-runs-two-commands-in-order
kind: story
status: draft
title: An action runs two commands in order, the second with values of the first's answer
summary: An action's does names one command, so create-then-act is two user steps or one host component (a consumer's UI report, second list, item 1)
tags:
- backlog
- ess-ui
- external-request
- feature-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
- informed_by: story:an-action-runs-a-command-and-then-opens-a-page
revision: 2
---
## Need

A user step is sometimes two commands of the model: create a thing, then act on the thing that was
created (create a draft and submit it, open a session and send its first message). The second
command's input is a value of the first command's answer, the created row's identity.

`ess-ui/1` cannot say this. `does` names one ESS command and an action has exactly one effect
(`website/docs/reference/ess-ui.md`, section Action), so the document offers two actions and the
user presses both, or the step becomes a host slot.

Evidence: a consumer's UI document (2026-11-13, ess 0.94.0), item 1 of its second collected report.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`, no parameter), a page `requests.inbox` whose section `list` is a `collection` over that
view with the row action `approve`, and a page `requests.detail` with a param `id`. Fixture rows
`r1` (`request-one`) and `r2` (`request-two`). `ess ui check --model` on the base:
`0 error(s), 0 warning(s)`.

Reproduction 1, the row action with `does: [desk.requests.ApproveRequest, desk.requests.ApproveRequest]`:

```
error document_loads pages/requests.inbox/sections/list/row_actions/approve: invalid type: sequence, expected a string
```

Reproduction 2, the row action with
`then: {does: desk.requests.ApproveRequest, bind: {id: result.id}}` beside its `does`:

```
error document_loads pages/requests.inbox/sections/list/row_actions/approve: unknown field `then`, expected one of `name`, `does`, `bind`, `opens`, `navigate`, `export`, `upload`, `copy`, `sets`, `label`, `as`, `choice`, `loads`, `checked`, `confirm`, `optimistic`, `bulk`, `visible`, `endpoint`
```

## What a design has to settle

- Whether this belongs in the UI document at all. Two commands in a row are not atomic: the first
  may be accepted and the second refused, and the user is left with the created thing. A model
  command with both effects is atomic and checked by conformance; the fit review says when the
  answer is "write that command" and when a chain is the right tool.
- If it does: this is the follow-up question of
  `story:an-action-runs-a-command-and-then-opens-a-page` (a second effect that reads the first
  command's answer). One vocabulary for "then open a page", "then open an overlay" and "then run a
  command", or none of them.
- What a refusal of the second command shows, and where; what `optimistic` means on a chain; what
  `ess ui check --model` resolves (the answer member on every outcome that continues, the second
  command's grant for the page's actor).
- What `ess-ui-test/1` pins: `expect_command` for each command in order, and a refused first
  command sends no second one.

## Acceptance (sketch, to be fixed by the fit review)

- Either the reference says that a step of two commands is a model command and `ess ui check`
  answers the two documents above with a finding that says so, or a document states the chain, both
  renderers send the second command only after the first was accepted and with the value of its
  answer, and a test pins both commands in order.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.

## Reported again (2026-11-13, a third list)

The same need from a consumer specification's UI document (ess 0.94.0, item 7 of its third
collected list): an action cannot run a second command with the outcome of the first. The consumer
carries an `unmapped` marker for the step in its document until this is decided. No new
reproduction; the two above stand.
