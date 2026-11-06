---
format: aep.planning-md/3
id: story:the-terminal-draws-a-search-box-only-when-a-read-takes-it
kind: story
status: draft
title: The terminal draws a search box only when a read takes it
summary: With no read taking a filter bar's search, React draws no box but the terminal draws one (0.88.0 search-box unit, pre-existing)
tags:
- backlog
- from-finding
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 1
---
## Need

Found by the 0.88.0 search-box unit, pre-existing: when no read on the page takes a filter bar's
search state, the generated React app draws no search box, while the terminal renderer still draws
one (`crates/ui/ess-ui-tui/src/view.rs` and `app.rs`, the filter-bar search). The two targets show a
different page for the same document.

## Acceptance

- `a_search_no_read_takes_is_not_drawn_in_the_terminal`: the terminal draws the search box only
  when a read takes the search state, as the React app does since 0.88.0.
