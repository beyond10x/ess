---
format: aep.planning-md/3
id: story:the-terminal-draws-a-search-box-only-when-a-read-takes-it
kind: story
status: draft
title: The terminal draws a search box only when a read takes it
summary: With no read taking a filter bar's search, React draws no box but the terminal draws one (0.88.0 search-box unit, pre-existing)
tags:
- backlog
- defect
- from-finding
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

Found by the 0.88.0 search-box unit, pre-existing and reproduced on 0.92.0: when no read on the page
takes a filter bar's search state, the generated React app draws no search box, while the terminal
renderer still draws one. The two targets show a different page for the same document.

React decides with `read_takes` (`crates/ui/ess-ui-react/src/emit.rs`, the filter bar's `search`
prop): a read whose params name the search state, a field inside it, or a record holding it. The
terminal draws `search [...]` whenever `bar.search` is set (`crates/ui/ess-ui-tui/src/view.rs`, the
filter-bar lines) and makes it a focus stop (`crates/ui/ess-ui-tui/src/app.rs`, the filter-bar items).

Reproduction (0.92.0): a `list_page` with page state `q: {type: string, class: page_state}`, a
section `{name: filters, component: filter_bar, binds: [state.q], search: {binds: state.q}}` and a
collection `{reads: {view: items.All}}` whose read takes no params.

```
$ ess ui test --path ui.yaml t.yaml      # open: items; expect: {at: pages/items/sections/filters, not_text: "search ["}
failed  no read takes the search, so no search box is drawn (step 2)
        expected no "search [" at pages/items/sections/filters; it shows:
        search [/ search]
$ ess generate ui --target react --path ui.yaml --out gen && grep -n "<FilterBar" gen/src/pages/Items.tsx
47:      <FilterBar binds={["state.q"]} />
```

## Acceptance

- `a_search_no_read_takes_is_not_drawn_in_the_terminal`: on the reproduction the terminal draws no
  `search [` line and the search is no focus stop of the filter bar.
- When a read takes the search state (its params name `state.q`, a field inside it, or a record
  holding it), the terminal draws the search box as before; the same rule decides both renderers.

## Scope

- cited: `crates/ui/ess-ui-tui/src/view.rs` (filter-bar lines, `bar.search`)
- cited: `crates/ui/ess-ui-tui/src/app.rs` (filter-bar focus items, `bar.search`)
- cited: `crates/ui/ess-ui-react/src/emit.rs` (`read_takes`, the rule to share, possibly moved into `crates/ui/ess-ui`)
- inferred: `crates/ui/ess-ui-test/tests/`
- inferred: `CHANGELOG.md`

confidence: high
