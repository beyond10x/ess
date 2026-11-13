---
format: aep.planning-md/3
id: story:an-action-runs-a-command-and-then-opens-a-page
kind: story
status: draft
title: An action runs a command and then opens a page, with values of the command's answer
summary: An action has exactly one of does, opens, navigate, export, upload, copy, sets, so a command cannot open the page of what it created (a consumer's UI report, item 1)
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

A user starts something and expects to be inside it: "start now" creates a run and the run's page
opens; "new ticket" saves and the ticket's page opens. The page to open is addressed by a value the
command's answer carries (the created row's identity), which the action does not know before it
runs.

`ess-ui/1` cannot say this. An action has exactly one effect
(`website/docs/reference/ess-ui.md`, section Action: `does`, `opens`, `navigate`, `export`,
`upload`, `copy`, `sets`), and a form's `submit` has `label`, `closes` and `auto` only. So the
consumer's document runs the command and leaves the user on the list, to find the new row and open
it by hand.

Evidence: a consumer's UI document (2026-11-13, ess 0.94.0), item 1 of its collected report.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`), a page `requests.inbox` whose section `list` is a `collection` over that view, and a page
`requests.detail` with a param `id`. `ess ui check --model` on the base: `0 error(s), 0 warning(s)`.

Reproduction: on the row action `approve` (`does: ApproveRequest`, `bind: {id: row.id}`), add
`navigate: {to: requests.detail, params: {id: row.id}}`:

```
error document_loads pages/requests.inbox/sections/list/row_actions/approve: an action has exactly one of does, opens, navigate, export, upload, copy, sets; this one has 2
```

## What a design has to settle

- Where the follow-up stands: on the action, on a form's `submit`, or on both, and whether the same
  key serves `opens` (run a command, then open an overlay over its answer).
- What the follow-up may read: the command's input, and which members of its answer. A command with
  several outcomes answers differently per outcome; a refusal must not navigate.
- What the terminal renderer, the React project, `ess ui check --model` (the answer member must
  exist on every outcome that navigates) and `ess-ui-test/1` each do with it.
- Whether a guard's `then` and an action's follow-up are one vocabulary.

## Acceptance (sketch, to be fixed by the fit review)

- The document above, written in the chosen form, loads and checks clean.
- A journey test acts on the action, sees the command sent, and then reads the target page with the
  parameter taken from the command's answer; a refused command stays on the page and shows its error.
- `ess ui check --model` refuses a follow-up that reads an answer member some navigating outcome
  does not carry, by node path.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.
