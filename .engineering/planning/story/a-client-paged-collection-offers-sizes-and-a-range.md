---
format: aep.planning-md/3
id: story:a-client-paged-collection-offers-sizes-and-a-range
kind: story
status: implemented
title: A client-paged collection offers page sizes, a from-to range and resets on filter
summary: 'Gap B4-7 (G-27): client pager sizes, range label for every pager, page reset on filter change'
tags:
- queue-0.91.0
- rank-12
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/tests/checks.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-test/src/playwright.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/src/runner.rs
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: inferred
  path: crates/ui/ess-ui/src/model.rs
- confidence: inferred
  path: schemas/ui/ess-ui.schema.yaml
- confidence: inferred
  path: website/docs/reference/ess-ui.md
revision: 18
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T22:19:03Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-06T22:19:04Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-08T22:10:12Z", actor: "human:timo", revision: 18, decided_on: {"recorded":{"test_result":1}}}
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

Derived 2026-11-07 by `story-scoper`. Every line is **cited** (read from the story or the tree) or
**inferred** (a reading that could be wrong).

- **Primary surface:** the `paging: client` pager in the four UI crates that carry `reads.pager`:
  `ess-ui`, `ess-ui-check`, `ess-ui-react` and `ess-ui-tui` — inferred.
- **Files:**
  - `crates/ui/ess-ui/src/model.rs` (`Pager`; `page` becomes optional) — inferred.
  - `schemas/ui/ess-ui.schema.yaml` (the `pager` record and the `pager_*` rules) — inferred.
  - `crates/ui/ess-ui-check/src/rules.rs` (`fn pagers`, `pager_paging`) — inferred.
  - `crates/ui/ess-ui-check/src/lib.rs` (rule registration, only if a new rule id is added) —
    inferred.
  - `crates/ui/ess-ui-react/src/emit.rs` (`fn pager`) — inferred.
  - `crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl` (the `Pager` footer,
    client slicing, the range label, reset on filter) — inferred.
  - `crates/ui/ess-ui-tui/src/app.rs` (`page_size`, `page_of`, `reset_page`) and
    `crates/ui/ess-ui-tui/src/view.rs` (the footer) — inferred.
- **Also likely:** `crates/ui/ess-ui-test/src/playwright.rs` and `runner.rs`, which parse the pager
  text; `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl`; and the tests that pin today's
  refusal (`ess-ui-check/tests/checks.rs`, `adversary_w086_u3_pass2.rs`) — inferred.
- **Documents:** `website/docs/reference/ess-ui.md`, re-rendered through `crates/ui/ess-ui-docs` —
  inferred.
- **Confidence:** medium. The story cites no path; the predecessor story
  `story:a-server-paged-collection-has-a-page-size-control` scoped the same files with citations.
- **Safety fact:** both test runners read the pager text, the terminal one by the `page N/M`
  prefix and Playwright by the `.ui-pager` span. A new range label must leave both readings intact.
