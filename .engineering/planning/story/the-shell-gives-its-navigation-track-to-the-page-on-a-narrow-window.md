---
format: aep.planning-md/3
id: story:the-shell-gives-its-navigation-track-to-the-page-on-a-narrow-window
kind: story
status: draft
title: The shell gives its navigation track to the page on a narrow window
summary: 'Follow-up of story:a-wide-table-scrolls-inside-its-column: below 48rem the generated shell still reserves its 15rem navigation track, so on a phone the stacked page column is about 7.5rem wide (adversary pass 1, backlog)'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- ess-ui
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`story:a-wide-table-scrolls-inside-its-column` makes page columns and areas stack below `48rem`
in the React renderer. The generated shell (`shell.tsx`, `.ui-shell` in
`crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl`) always lays out a grid whose first
column is `var(--ui-size-nav-width)` (15rem), even when the shell has no navigation region.

So on a 22.5rem phone the stacked page column is about 7.5rem wide, less the outlet padding. The
story's need, "a page read on a narrow window shows every section whole, one under another",
does not hold there.

The first adversary pass over that story found this. The rule predates the story.

## Acceptance

1. `below_the_renderer_width_the_shell_gives_its_navigation_track_to_the_page` in
   `crates/ui/ess-ui-react/tests/adversary_narrow_layout_pass1.rs` passes, with its assertion
   unchanged and its `#[ignore = "backlog: narrow-shell-nav-track …"]` removed.
2. Below `48rem` the navigation stays reachable: it is stacked above the page or behind a
   control. A generator test pins whichever is chosen.
3. Above `48rem` the shell is unchanged.
