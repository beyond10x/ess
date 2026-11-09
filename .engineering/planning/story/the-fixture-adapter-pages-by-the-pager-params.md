---
format: aep.planning-md/3
id: story:the-fixture-adapter-pages-by-the-pager-params
kind: story
status: draft
title: The fixture adapter pages a read by the params its pager names
summary: The fixture adapter slices server-paged reads by the param names page and size, not by the pager (UI-adopter wave 1 U3 review note, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- informed_by: review-result:w086-u3-adversary-pass-1
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

The generated fixture adapter pages a server-paged read by the param names `page` and `size`
(`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl:143-145`), not by the params the read's
`pager` names. The pager never reaches it: `ReadRequest` carries `view`, `params` and `paging` only
(`data.ts.tmpl:31-38`), and the page passes `useRead({ view, params, paging })`. So a collection
whose `pager` names other params shows "Page 2 of 3" against the fixture adapter but lists every
row on every page, and a fixture preview does not show what a server would return. Found by the
UI-adopter wave 1 adversary pass on the page-size unit (review-result:w086-u3-adversary-pass-1,
note); pre-existing at 83391ebaeb, unchanged on 0.92.0.

Reproduction on 0.92.0, seven fixture rows `r1`..`r7`:

```yaml
state:
  pg: {type: integer, class: page_state, store: url, default: 1}
  sz: {type: integer, class: page_state, store: url, default: 3}
sections:
- name: list
  component: collection
  reads:
    view: blog.posts.All
    params: {p: state.pg, n: state.sz}
    paging: server
    pager: {page: p, size: n, sizes: [3, 10]}
  columns: [title]
```

`ess ui check` passes; the generated `fixtureAdapter.read` answers:

```
pager names p/n; page p=1 size n=3: total 7, rows [r1 r2 r3 r4 r5 r6 r7]
pager names p/n; page p=2 size n=3: total 7, rows [r1 r2 r3 r4 r5 r6 r7]
params named page/size; page=2 size=3: rows [r4 r5 r6]
```

## Acceptance

- The fixture adapter slices a server-paged read by the params the read's `pager` names (and by
  the `page` param of a read without a `pager`), never by a param's name.
- For the document above, the fixture adapter answers `p=2, n=3` with rows `r4 r5 r6` and total 7,
  pinned by a generator test.
- A read whose params are named `page` and `size` keeps its current answer.

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl` (`ReadSpec` :21, `ReadRequest` :31, `fixtureAdapter.read` :126-147, `useRead` :648)
- cited: `crates/ui/ess-ui-react/src/emit.rs` (`reads` :716, the `ReadSpec` it writes, which must carry the pager's param names)
- inferred: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl` (`ReadSpec` consumers)
- inferred: `crates/ui/ess-ui-react/tests/adversary_w086_u3_pass1.rs` or a new pager test

confidence: medium — the slice is cited; how the pager's names reach the adapter (a `ReadRequest`
field or the spec) is a choice.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
