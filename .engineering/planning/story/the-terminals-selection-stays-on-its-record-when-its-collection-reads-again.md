---
format: aep.planning-md/3
id: story:the-terminals-selection-stays-on-its-record-when-its-collection-reads-again
kind: story
status: draft
title: The terminal's selection stays on its record when its collection reads again
summary: After a re-read that answers in another order or fails, the terminal's section.<name>.selection moves to another record or is dropped while the React project keeps the record picked; two ignored tests hold the cases
tags:
- backlog
- defect
- ess-ui
- from-finding
- review-finding
- severity-medium
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
- informed_by: story:a-sections-first-row-is-a-binding-in-the-reference-and-both-renderers-or-in-neither
revision: 1
---
## Defect

In the terminal renderer `section.<name>.selection` is the row under the cursor of the rows shown,
by position. When the collection reads again for the same request (`R`, a poll, a command's
re-read) and its answer differs, the selection moves to another record or is dropped, and every
section that reads by it follows. The generated React project keeps the selection on the record
the user picked. The same document then binds different records in the two renderers.

Found by the third review pass of
`story:a-sections-first-row-is-a-binding-in-the-reference-and-both-renderers-or-in-neither`. That
unit holds the selection while the read is under way; what happens when the answer arrives was so
before it (the selection was the cursor's position at the base) and is this story.

Document of the cases (`crates/ui/ess-ui-tui/tests/adversary_section_row_pass3.rs`, the two tests
marked `#[ignore]` with this story's id; the React half is
`crates/ui/ess-ui-react/tests/adversary_section_row_pass3.rs`, green): a collection `pick` over
orders `o-1`, `o-2`; a record `order` reading by `section.pick.selection.id`; a record `customer`
reading by `section.order.row.customer_id`. `o-2` is picked, then `pick` reads again.

| second answer of `pick` | React, `customer` reads | terminal, `customer` reads |
|---|---|---|
| `o-3, o-1, o-2` (another order, a new row first) | `c-2` | `c-1`: the cursor kept its position, the selection is `o-1` |
| the read fails | `c-2`: the selection is kept | `c-1`: no selection, `order` reads with no id and shows its first row |

Also found, by reading the code, no test:

- `j`/`k` while the collection reads again: the cursor is clamped to the first position because no
  rows are shown, so the held selection becomes the first held row.
- `selection: multiple`: the terminal's `section.<name>.selection` is the cursor's row, the React
  project's is the list of marked rows.
- The terminal's header title falls back to the literal title while its section reads again for
  the same params; the React project keeps the title.
- `R` on an `on_demand` section that shares a read with a loaded section shows `Loading` while its
  row already resolves from the other section's held answer.
- In the React project, in a shell with overlays, the first render after a page change reads the
  previous page's sections in a shell overlay until the page frame clears them (one render).

## What a design has to settle

- What a selection is in the terminal: a position, as today, or the key of a row. The reference
  says a selection is "the row selected"; a row has a key in both renderers (`rows/<key>`).
- What a selection is after a failed re-read: kept (React today) or dropped (the terminal today).
  One answer for both renderers.
- What happens when the selected record is no longer in the answer.
- Whether `selection: multiple` gives an expression one row or a list, in both.

## Acceptance (sketch, to be fixed by the fit review)

- The two ignored tests pass with their `#[ignore]` removed and their assertions unchanged, or the
  fit review decides the other renderer moves and the React halves change with a changelog line.
- The cursor is on the selected record after a re-read that answers in another order.
- Both renderers bind the same record in each row of the table.

## Fit review

Owed. This draft records the finding and its red cases only.
