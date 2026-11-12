---
format: aep.planning-md/3
id: story:board-widget-kinds-reads-every-fixture-answer
kind: story
status: draft
title: board_widget_kinds reads every fixture answer
summary: 'Follow-up of story:each-row-and-each-waiting-read-draws-what-the-document-declares: the board_widget_kinds check reads only a view''s plain fixture rows, not its by_params answers; and the renderers differ on when a dependent read waits (adversary pass 1, backlog)'
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

`story:each-row-and-each-waiting-read-draws-what-the-document-declares` adds the `ess ui check`
warning `board_widget_kinds`. It fires for a fixture row of a board's read that has no
`widget_by` field, or whose kind no `widgets` entry names.

`view_rows` in `crates/ui/ess-ui-check/src/rules.rs` reads only a plain `rows:` mapping. So board
rows that a fixture answers under `by_params` are never checked.

The same adversary pass recorded a difference between the renderers. When a section
`depends_on` a list that has rows but no clicked selection, React draws the section's empty state
until a row is clicked. The terminal treats its cursor row as the selection and reads at once.
This comes from how each renderer models selection.

## Acceptance

1. `ess ui check` warns `board_widget_kinds` for a board row inside a `by_params` fixture answer,
   at the board's path, once per distinct kind. A case in `crates/ui/ess-ui-check/tests/checks.rs`
   pins it.
2. The reference page states how each renderer decides that a dependent read waits for a
   selection. A test pins the terminal's rule.
