---
format: aep.planning-md/3
id: story:a-collection-nested-in-a-row-reads-with-a-param-of-the-row
kind: story
status: draft
title: A collection nested in a row reads with a param of the row
summary: In the terminal a row's nested collection whose read takes a row param stays loading; without the param, and as a record, it draws (a consumer's UI report, second list, item 8)
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
revision: 1
---
## Defect

A row of a collection shows rows of its own: the lines of an order, the replies of a ticket. The
document writes a `collection` in the row's `item` whose read takes a param from the row
(`params: {id: row.id}`). In the terminal renderer the nested collection never leaves `loading…`.

Evidence: a consumer's UI document (2026-11-13, ess 0.94.0), item 8 of its second collected report.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`, no parameter), a page `requests.inbox` whose section `list` is a `collection` over that
view with the row action `approve`, and a page `requests.detail` with a param `id`. Fixture rows
`r1` (`request-one`) and `r2` (`request-two`). `ess ui check --model` on the base:
`0 error(s), 0 warning(s)`.

Reproduction, the section with
`item: [{name: more, component: collection, reads: {view: desk.requests.PendingRequests, params: {id: row.id}}, columns: [title]}]`,
a test that opens the page, advances 5 s and prints the section:

```
┌ list ─────────────
  title        state
  request-one  Pending
  request-two  Pending
▸ loading…
page 1/1 · 2 rows
```

The same with `fixtures.views` answering `by_params: {id: r1}`: `▸ loading…`.

Controls, same document and test:

- the nested collection without `params`: `▸   title         request-one   request-two page 1/1 · 2 rows`;
- a nested `record` with `params: {id: row.id}`: `▸ state            Pending`;
- the nested collection as the row's `expand`, without `params`: drawn as a table under the row.

The reproduction's view declares no parameter, so `ess ui check --model` refuses the document
(`read_params … `id` is not a parameter of view …`); `ess ui test` runs it. The fit review repeats
it with a view that declares the parameter before the acceptance is fixed, and says whether the
React project shows the same.

Two smaller things the controls show, for the same review:

- the terminal draws a row's `item` once, for the row under the cursor, in one line below the rows
  (the `▸` line), not in each row;
- a nested collection is drawn in that one line, header, rows and pager run together.

## Acceptance (sketch, to be fixed by the fit review)

- A journey test in `crates/ui/ess-ui-test/tests/`, red on 0.94.0: a row's nested collection whose
  read takes `params: {<p>: row.<field>}` shows the rows its fixture answers for the row under the
  cursor, and other rows after the cursor moves.
- A nested read that fails shows `failed` and its message in the row's detail, not `loading…`.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the acceptance is fixed there.
