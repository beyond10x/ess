---
format: aep.planning-md/3
id: story:a-ui-expression-joins-texts
kind: story
status: draft
title: A UI expression joins texts
summary: An ess-ui/1 expression has no concatenation and no template, so a text built from parts is a model view field or a host slot (a consumer specification's UI list, item 1)
tags:
- backlog
- ess-ui
- external-request
- feature-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
- informed_by: story:text-functions-compute-values
revision: 1
---
## Need

A text shown or copied is sometimes built from parts: a fixed text and a value of the row
(`Request r1`), two values with a separator, an address made of a prefix and an identity. An
`ess-ui/1` expression is a path, a literal or a comparison; its operators are `==`, `!=`,
`in [..]`, `not`, `and`, `or` (`website/docs/reference/ess-ui.md`, Expressions). Nothing joins two
texts, so the part-built text is a field the model's view computes, or a host slot.

Evidence: a consumer specification's UI document (2026-11-13, ess 0.94.0), item 1 of its third collected list.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`, no parameter), a page `requests.inbox` whose section `list` is a `collection` over that
view with the row action `approve`, and a page `requests.detail` with a param `id`. Fixture rows
`r1` (`request-one`) and `r2` (`request-two`). `ess ui check --model` on the base:
`0 error(s), 0 warning(s)`. Each reproduction adds one row action
`{name: link, copy: "<expression>", label: Copy the link}` to the section `list`.

| `copy:` | `ess ui check --model` | the terminal's notice after `act` on row `r1` |
|---|---|---|
| `row.title + row.id` | 0 errors, 0 warnings | `copied` (empty) |
| `'request ' + row.id` | 0 errors, 0 warnings | `copied 'request ' + row.id` (the source text) |
| `{row.title}/{row.id}` | 0 errors, 0 warnings | `copied {row.title}/{row.id}` (the source text) |
| `concat(row.title, row.id)` | 0 errors, 0 warnings | `copied` (empty) |

The check passes all four; that part is the defect
`story:ui-check-reports-an-expression-that-reads-a-path-and-is-no-expression`.

What works today: a view field of the model computed with `+` over texts
(`story:text-functions-compute-values`, active, documents it) and shown or copied by its path. That
moves a text of the user interface into the model.

## What a design has to settle

- Whether the UI expression language computes at all, or stays paths and comparisons and the
  answer is the view field. The fit review weighs a text that belongs to one screen against a
  second expression language to keep equal with the model's.
- If it joins: one form, and the same in every `expr` position (`text`, `label`, `copy`, `args`,
  a navigate param). Candidates are the model language's `s + t` and a template with holes; two
  spellings for one idea are refused by the review's question 4.
- What an absent part gives (an absent Optional, a row still loading): the whole text absent, or
  the part empty.
- What `ess ui check` resolves and types: each part's path, and that each part is a text or has
  one documented text form.
- What both renderers and `ess-ui-test/1` show for it.

## Acceptance (sketch, to be fixed by the fit review)

- Either the reference says that a text built from parts is a view field of the model and
  `ess ui check` answers the four documents above with a finding that says so, or a document
  states the join once, both renderers show and copy the joined text, and a test pins it.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.
