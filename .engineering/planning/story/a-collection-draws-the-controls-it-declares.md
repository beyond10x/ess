---
format: aep.planning-md/3
id: story:a-collection-draws-the-controls-it-declares
kind: story
status: draft
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
revision: 1
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
