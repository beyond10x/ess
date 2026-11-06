---
format: aep.planning-md/3
id: story:a-collection-draws-the-controls-it-declares
kind: story
status: implemented
title: A collection draws the controls it declares
summary: Numeric sort, actions header, wide-table scroll, card layout, sort on cards, expand/reorder/bulk controls, item and toggle columns, default column subset (beyond10x/ess#723)
tags:
- from-consumer-report
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T18:11:26Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-05T18:11:26Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-06T07:40:10Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Need

A downstream adopter's review of the generated React runtime
(https://github.com/beyond10x/ess/issues/723, on 0.84.0) found that a `collection` declares controls the
runtime does not draw or draws wrongly. The cited code is still present at 0.87.0, in
`crates/ui/ess-ui-react` `composites/collection.tsx` and the collection rules of `styles.css`:

- client-side sort of an `as: number` column compares the JSON text, so 5, 10, 1 sorts as 1, 10, 5;
- the row-actions column has no header;
- on a wide table the sticky actions cell covers declared columns, and at narrow widths the table
  overflows with no scroll cue;
- on `style: cards` the row actions overlap and squeeze the title, and a toggle on a card is not drawn;
- `sort` on `style: cards` or `list` draws no sort control;
- `expand` draws no expand control, `reorder` no drag handle, and `selection: multiple` with
  `bulk_actions` no select-all or bulk menu;
- a row `item` or a `sets:` toggle column is drawn inside the actions cell, without heading or sort;
- a `state.columns` default subset is ignored, and every column renders.

## Acceptance

Each bullet is a named test in `crates/ui/ess-ui-react/tests/`, red before the change:

- `a_number_column_sorts_by_value`: rows 5, 10, 1 under `as: number` sort to 1, 5, 10.
- `the_row_actions_column_has_a_header`.
- `a_wide_table_scrolls_inside_its_frame`: the actions cell covers no declared column, and an
  overflowing table scrolls inside its section with a visible cue.
- `card_actions_leave_the_title_whole`: row actions sit beside or below the title, and a toggle on a
  card is drawn in its state.
- `cards_and_lists_draw_their_declared_sort`.
- `expand_reorder_and_bulk_actions_draw_their_controls`: an expand control, a drag handle, and a
  select-all with a bulk menu, each sending what it declares.
- `a_row_item_and_a_toggle_column_get_their_own_column`: with a heading and, when declared, sort.
- `a_default_column_subset_renders_only_its_columns`.

## Out of scope

The terminal renderer's collection, the `ess ui test` steps that act on these controls (their own
story), and any new `ess-ui/1` key.

## Decisions

The 0.88.0 adversary pass on this unit found six introduced defects; the coordinator decided on
2026-11-06 that one correction in this wave fixes them all, committed with the pass's cases:

- A drag moves a row only onto a neighbour in the drawn order within its group, so a grouped
  collection (a board) never moves a row across rows the group does not show.
- The keyboard move refuses past the first and the last row.
- A select-all selects only its group's rows, or the collection has one select-all.
- The drag handle does not pass its click to the row.
- Under a client sort on another field a reorder is refused or its handle hidden, whichever is the
  smaller change.
- The client sort's comparison is a total order (numbers before text), so mixed columns sort
  consistently.
- An item column sorts by the field the item shows, or is not sortable unless `sort.allowed` names a
  row field.

## Review

Adversary pass 1 (2026-11-06): NEEDS-CHANGE, 6 introduced findings (drag across a group, the keyboard move past either end, select-all across groups, the handle's click, a reorder under a client sort, a non-transitive comparison, the item column's sort), decided as recorded above. One correction fixed all; the pass's 8 cases are committed and green beside `collection_controls` (8 passed). Pre-existing, filed: story:move-buttons-refuse-past-either-end.
