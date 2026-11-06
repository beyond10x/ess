---
format: aep.planning-md/3
id: story:a-client-row-filter-keeps-its-rows-without-paging-and-by-path-id
kind: story
status: active
title: A client row filter keeps its rows on an unpaged read and by the page path id
summary: Runtime fix taken upstream ("rowFilters paging none + path id"); reproduce first
tags:
- queue-0.90.0
relations:
- serves: vision:O2
revision: 3
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
