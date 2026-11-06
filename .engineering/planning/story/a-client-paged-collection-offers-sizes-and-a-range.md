---
format: aep.planning-md/3
id: story:a-client-paged-collection-offers-sizes-and-a-range
kind: story
status: active
title: A client-paged collection offers page sizes, a from-to range and resets on filter
summary: 'Gap B4-7 (G-27): client pager sizes, range label for every pager, page reset on filter change'
tags:
- queue-0.91.0
relations:
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T22:19:03Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-06T22:19:04Z", actor: "human:timo", revision: 3}
---
## Need

Gap item B4-7 (G-27). Since 0.88.0, a server-paged collection has a page-size control
(story:a-server-paged-collection-has-a-page-size-control). A collection with `paging: client`
still has a fixed page size, shows no range, and keeps its page number when a filter shrinks the
rows, so it can show an empty page.

## Decision (lead, under the 0.90.0 approval)

- `paging: client` takes the sizes a server pager takes (`pager.sizes`) and draws the same
  control, with no params.
- Every paged collection, client or server, shows a "from-to of total" label.
- A change to the collection's filter, search or row filter returns a client-paged collection to
  its first page.

## Acceptance

- A `paging: client` collection with `pager: {sizes: [10, 25]}` over 30 rows:
  - draws 10 rows and the label `1-10 of 30`;
  - after choosing 25, draws 25 rows and `1-25 of 30`.
- On page 3, a search that keeps 4 rows shows page 1 and `1-4 of 4`.
- The terminal draws the same label and accepts the same size choice.
- `pager.page` becomes optional in the schema.
  - With `paging: server`, `ess ui check` still requires it.
  - With `paging: client`, `ess ui check` refuses `page` and `size` by name; `sizes` alone is
    allowed.
  - Every existing document keeps its meaning.

## Scope

- inferred: `crates/ui/ess-ui/src/model.rs` (Pager rules), `crates/ui/ess-ui-check/src/`,
  `crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl`,
  `crates/ui/ess-ui-tui/src/`.
