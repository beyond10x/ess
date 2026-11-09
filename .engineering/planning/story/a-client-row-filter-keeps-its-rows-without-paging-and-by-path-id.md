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
revision: 6
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

- A case reproduces the failure first. Candidates:
  - a collection with `paging: none` and a `filter` keeps exactly the rows the filter holds for,
    and its total counts them;
  - a filter `row.<field> == params.id` on a page reached at `/<page>/<id>` keeps the rows of
    that id, and keeps them again after navigating to another id.
- The fix makes the reproducing case pass in the React project. The terminal agrees on the same
  document.
- If neither candidate fails on this release, the unit stops and reports that. The story then
  goes back to the coordinator for the adopter's exact case.

## Scope

- inferred: `crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl` (`filterRead`),
  `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl` (section read),
  `crates/ui/ess-ui-react/tests/read_filter.rs`.

## Status note

Not reproduced on 0.89.0. In unit U3 of wave-2026-11-07-ess, both candidate cases passed in the
React project and in the terminal:

- a client row filter on a read with `paging: none`;
- a filter on the page's path id, including after navigating to another id.

A mutation of `filterRead` turned the check red, so the check can fail. The repro tests were not
applied. The lead keeps them outside the repository as `story2-repro-tests.patch`.

By the coordinator's decision of 2026-11-07, the story leaves 0.90.0. The reporter is asked for
its exact document and data, and the story waits for that answer (tag `needs-repro`).
