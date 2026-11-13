---
format: aep.planning-md/3
id: story:a-ui-test-reads-what-a-copy-action-copied
kind: story
status: draft
title: A UI test reads what a copy action copied
summary: ess-ui-test/1 has no step for the clipboard; the terminal's notice `copied <value>` is the only trace (a consumer specification's UI list, item 5)
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

A journey test pins what an action does: `expect_command` reads the command an action sent,
`expect_export` the export it started. A `copy` action has no such step. What it copied is visible
only as the terminal's notice `copied <value>`, read as text of the page.

Evidence: a consumer specification's UI document (2026-11-13, ess 0.94.0), item 5 of its third collected list.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`, no parameter), a page `requests.inbox` whose section `list` is a `collection` over that
view with the row action `approve`, and a page `requests.detail` with a param `id`. Fixture rows
`r1` (`request-one`) and `r2` (`request-two`). `ess ui check --model` on the base:
`0 error(s), 0 warning(s)`. Each reproduction adds one row action
`{name: link, copy: "<expression>", label: Copy the link}` to the section `list`.

With `copy: "row.id"`, after `act` at `…/rows/r1/row_actions/link`:

```
- expect_clipboard: r1
tests[1] (expect_clipboard): step 3: `expect_clipboard` is not a step; a step is one of open, select, mark, type, choose, act, move, upload, page, expect, play, advance, expect_command, expect_export
```

What works today, in the terminal run: `expect: {at: pages/requests.inbox, text: copied r1}`
passes. It reads the notice, so it holds for any text that contains the value, cannot tell an
empty copy (`copied`) from a notice cut at the edge, and depends on the notice's wording.

## What a design has to settle

- The step: a sibling of `expect_command` and `expect_export` that reads the last value a `copy`
  action copied, exact and whole, and what it reports when no copy happened since the last step.
- Whether the same test runs against the generated React project, where the browser's clipboard
  needs a permission and a test stand-in.
- Whether the notice's wording stays free: once a step reads the value, no test needs the text
  `copied`.

## Acceptance (sketch, to be fixed by the fit review)

- A test states the value a `copy` action copied and fails with both values when it differs; an
  empty copy fails against a non-empty expectation; a test with no copy before the step fails and
  says so.
- The reference of `ess-ui-test/1` lists the step beside `expect_command` and `expect_export`.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.
