---
format: aep.planning-md/3
id: story:a-client-row-filter-keeps-its-rows-without-paging-and-by-path-id
kind: story
status: active
title: A client row filter keeps its rows on an unpaged read and by the page path id
summary: Runtime fix taken upstream ("rowFilters paging none + path id"); reproduce first
tags:
- defect
- needs-repro
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/tests/read_filter.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/tests/read_filter.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T22:19:00Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-06T22:19:01Z", actor: "human:timo", revision: 3}
---
## Need

A second runtime fix that a downstream adopter carries in its copy of the generated React
runtime; 0.90.0 takes it upstream. The coordinator's record names it in only these words:

> rowFilters paging none + path id

The failing behaviour is not yet reproduced here. The read is that a client row filter
(`reads.filter`) misbehaves on a read with `paging: none`, or when the filter compares against
the page's path id (`params.id`), or in both cases. `filterRead`
(`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl`) and the section read in `core.tsx.tmpl`
apply the filter.

## Acceptance

- `cargo test -p ess-ui-react --locked --test read_filter` passes a new test
  `an_unpaged_read_keeps_the_filtered_rows_and_counts_them`: a collection with
  `reads: {view: t.All, paging: none, filter: row.group == one}` over the rows
  `[{id: a, group: one}, {id: b, group: two}, {id: c, group: one}]` shows rows `a` and `c`, and
  its total is `2`.
- The same target passes a new test `a_path_id_filter_keeps_the_rows_of_each_id`: a filter
  `row.group == params.id` on a page reached at `/p/one` shows rows `a` and `c`; after navigating
  to `/p/two` it shows row `b` only.
- `cargo test -p ess-ui-tui --locked --test read_filter` passes the same two cases on the same
  document and rows, with the same kept rows and total.
- With `filterRead` in `data.ts.tmpl` changed to return `read` unfiltered, both React tests fail
  (the mutation is run once and reverted; its output is recorded in the story's evidence).

## Decisions

- The two cases land as regression tests whether or not they fail first on 0.92.0: they passed
  on 0.89.0 (Status note), so one result, "the four checks above pass", replaces the earlier
  "fix it, or stop and report"; a case that fails first is fixed in `filterRead` or the section
  read until it passes.

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl` (`filterRead`)
- cited: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl` (section read, `filterRead(own, reads.filter, scope.values)`)
- cited: `crates/ui/ess-ui-react/tests/read_filter.rs`
- cited: `crates/ui/ess-ui-tui/tests/read_filter.rs`

confidence: medium

The story moves no shared surface unless a fix lands, in which case it proposes its
`CHANGELOG.md` sentence.

## Status note

Not reproduced on 0.89.0. In unit U3 of wave-2026-11-07-ess, both candidate cases passed in the
React project and in the terminal:

- a client row filter on a read with `paging: none`;
- a filter on the page's path id, including after navigating to another id.

A mutation of `filterRead` turned the check red, so the check can fail. The repro tests were not
applied. The lead keeps them outside the repository as `story2-repro-tests.patch`.

By the coordinator's decision of 2026-11-07, the story leaves 0.90.0. The reporter is asked for
its exact document and data, and the story waits for that answer (tag `needs-repro`).
