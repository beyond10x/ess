---
format: aep.planning-md/3
id: story:the-terminal-pages-a-server-read-through-its-pager-state
kind: story
status: draft
title: The terminal pages a server read through its pager state
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#722
relations:
- decomposes: epic:zero-open-issues-2026-11
revision: 2
---
## Need

The terminal pages a server-paged collection as if the client held every row, so a journey test
can neither pin the page a command or a read is sent with nor see the declared page size
(beyond10x/ess#722 items 23 and 24, on 0.84.0; reproduced on 0.92.0):

- the page size is the run's fixed 10 rows (`crates/ui/ess-ui-tui/src/app.rs:44`); a declared
  pager's sizes and the size state's default are read only for `paging: client`
  (`app.rs:1708-1718`);
- `n` and `p` move a page index held in the terminal's own UI state (`app.rs:2811-2825`); the
  states the read's `pager: {page, size}` names are never written, so the read is never sent for
  another page and a bind of `state.page` stays at its default (item 24);
- a committed text filter returns the collection to its first page (`app.rs:4060-4085`), a
  filter-bar choice does not: after paging to page 2 and choosing an option that still leaves two
  pages, the collection stays on page 2 (item 23).

Minimal reproduction on 0.92.0:

```yaml
state:
  status: {type: {list: {enum: [open, closed]}}, class: page_state, store: url}
  page: {type: integer, class: page_state, default: 1}
  size: {type: integer, class: page_state, default: 5}
sections:
  - {name: filters, component: filter_bar, binds: [state.status],
     choices: [{name: status, component: choice, multiple: true, options: [open, closed]}]}
  - name: list
    component: collection
    reads: {view: example.order.Page, params: {status: state.status, page: state.page, size: state.size},
            paging: server, pager: {page: page, size: size, sizes: [5, 10]}}
```

```
failed  item 24a ... expected 5 rows at pages/orders.list/sections/list; it shows 10 [o-1, ..., o-10]
failed  item 24b ... command example.order.Report was sent with {account="acc-1", page=1, size=5}, not with {page=2, size=5}
failed  item 23 ... (30 rows, 20 open; page 2, then choose open) expected rows [o-1, ..., o-10]; it shows [o-11, ..., o-20]
```

## Acceptance

Journey tests in `crates/ui/ess-ui-test/tests/` and unit tests in `crates/ui/ess-ui-tui`, each red
on 0.92.0:

- A server-paged collection with a `pager` shows the size its `size` state holds (its default, else
  the first of `sizes`): 5 rows of 12 above.
- `page: {at: …, to: 2}` writes the `page` state the pager names (counted as the read counts it,
  from 1), the read is sent with `page: 2`, and an action binding `state.page` sends 2.
- The fixture adapter answers a server-paged read with the slice the pager's page and size params
  name, by the pager's param names, not by the names `page` and `size`.
- A change to any param of the read other than the pager's own (a choice, a search, a time window)
  returns the collection to page 1, the state included; the item 23 journey ends on rows o-1 to
  o-10.
- A server-paged read without a `pager` pages as on 0.92.0.

## Scope

- cited: crates/ui/ess-ui-tui/src/app.rs
- cited: crates/ui/ess-ui-tui/src/data.rs
- inferred: crates/ui/ess-ui-tui/src/view.rs
- inferred: crates/ui/ess-ui-test/tests/journey_paging.rs
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A journey test must see and pin the page a server-paged read is sent with, at the
   declared size, and see a narrowed list from its first page. Reproduction in § Need. Requester's
   ask: a step or a statement per item; no syntax proposed.
2. **Class.** Defect: `pager` "names, among `params`, the param that carries a server read's page
   and the one that carries its page size, each bound to a `state.<path>`"
   (`website/docs/reference/ess-ui.md` § Reads), and 0.91.0 states that a change to a read's params
   returns a client-paged collection to page 1 (CHANGELOG 0.91.0, `ess-ui/1` client pager entry);
   the terminal applies neither to a server pager.
3. **Already expressible?** No: `page` moves the terminal's own index only (§ Need, item 24b).
4. **Fit.** No authored surface: the pager already names the states. It extends the 0.91.0 rule
   for client pagers to server pagers in the terminal, and slices fixtures by the pager's params, the
   rule story:the-fixture-adapter-pages-by-the-pager-params asks of the React fixture adapter (a
   sibling; neither depends on the other). The language grids do not apply.
5. **Second adopter.** An audit log of thousands of entries, paged by the server 50 at a time,
   whose "Export this page" action binds the page state.
6. **Cost.** Terminal paging and its fixture adapter; no format change. A terminal run of a
   server-paged list now shows the declared page size instead of 10 rows.
7. **Considered.** (a) Change nothing and document that the terminal pages client-side: rejected,
   the page a read is sent with is the behaviour under test. (b) A test-level page size: rejected,
   the document already declares it. (c) Chosen: the terminal drives the pager's states.

## Decisions

Accept, redesigned (triage of beyond10x/ess#722, 2026-11-09): items 23 and 24 become one terminal
fix, a server pager driven through the states it names.

## Source

github:beyond10x/ess#722
