---
format: aep.planning-md/3
id: story:each-row-and-each-waiting-read-draws-what-the-document-declares
kind: story
status: active
title: A row condition is judged per row, a waiting read shows its empty state, a board row without a widget names its kind
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/composites/board.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/composites/record.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: inferred
  path: schemas/ui/ess-ui.schema.yaml
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:43:36Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:43:37Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Three places draw something other than what the document declares for a row or a read. On ess
0.92.0 (`ess generate ui --target react`, document and fixtures in the triage's `repro/`):

- **A column's row condition.** `{field: note, visible: row.has_note}` is evaluated once for the
  whole column against the page scope, where `row` is absent:
  `(column.visible === undefined || truthy(evaluate(column.visible, scope.values)))`
  (`crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl:157`), so the column
  vanishes for every row. `ess ui check` reports nothing.
- **A read that has not started.** A section with `depends_on: list` and
  `states: {empty: {message: Pick an order}}` reads only once the list has a selection; until then
  its read is `idle` and the frame draws loading skeletons (`templates/runtime/core.tsx.tmpl:444`,
  `data.status === "loading" || data.status === "idle"`), never the declared empty text.
- **A board row with no widget.** A board `widget_by: kind` whose row has no `kind` draws
  `No widget for undefined` (`templates/runtime/composites/board.tsx.tmpl:28`, `:41`); a row whose
  kind names no entry of `widgets` draws `No widget for <kind>`. Fixture row `{id: w2, value: 7}`.

## Acceptance

Named tests in `crates/ui/ess-ui-react/tests/row_states.rs` and the terminal's
`crates/ui/ess-ui-tui/tests/row_states.rs`:

- `a_column_condition_reading_the_row_is_judged_per_row`: a column `visible:` that reads `row.` is
  evaluated per row, and the cell of a row where it is false is empty; the column and its header
  stay. A `visible:` that reads no `row.` path still hides the whole column, as today. The record
  field and the card field follow the same rule.
- `a_read_that_has_not_started_shows_its_empty_state`: an `idle` read draws `states.empty` (title,
  message, action) when declared, else a neutral line `Nothing to show yet`; never a skeleton.
  `loading` still draws the skeleton.
- `a_board_row_without_a_widget_names_its_kind`: a row whose `widget_by` field is absent draws a
  placeholder naming the field (`no <field>`), one whose kind names no widget names the kind; and
  `ess ui check` warns `board_widget_kinds` at the board's path for a fixture row of its read whose
  kind no `widgets` entry names.
- The terminal draws the same three outcomes for the same document.

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl`
- cited: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl`
- cited: `crates/ui/ess-ui-react/templates/runtime/composites/board.tsx.tmpl`
- inferred: `crates/ui/ess-ui-react/templates/runtime/composites/record.tsx.tmpl`
- inferred: `crates/ui/ess-ui-tui/src/view.rs`
- inferred: `crates/ui/ess-ui-tui/src/app.rs`
- inferred: `crates/ui/ess-ui-check/src/rules.rs`
- inferred: `schemas/ui/ess-ui.schema.yaml`
- confidence: medium

## Fit review

Tree `995e29579d`, binary ess 0.92.0. Covers #723 items D15 (its second half, an `item:` replacing
the column cells on cards, does not reproduce: since 0.88.0 cards draw both), G18/D10 and G26/D8
("a board renders one of three widgets" is the same cause: rows whose kind names no widget).

1. **Need.** A cell hides when its row says so, a section that waits for a selection says what it
   waits for, and a board row the document cannot draw says why. Reproductions above
   (`repro/run.txt`).
2. **Class.** *Defect*, all three: `Field.visible` "shows the field only when true"
   (`schemas/ui/ess-ui.schema.yaml` `Field`) and a `row.` path is a row's
   (`expressions.forms`: "the current row of a collection, record or board"); `states.empty` is "ready
   with no rows" and `loading` "first load" (`SectionStates`), and an idle read is neither loading
   nor failed; a board row draws "the widget its type names" (`board.tsx.tmpl:8`).
3. **Already expressible?** A per-row cell: an `item:` text node with `visible: row.has_note` draws
   per row, but as a separate column. The rest: no.
4. **Fit.** The language grids do not apply (`docs/design/language-grids.md:15`, `:63`). No new key:
   each fix gives an existing key its documented meaning. The per-row rule is the one `item:` nodes
   already follow (a `Visible` wrapper inside the row scope, `repro/idioms/`).
5. **Second adopter.** An orders list hiding the refund amount on rows that have none; a detail
   panel saying "Pick an order" beside the list; a dashboard whose saved layout names a widget kind
   a newer document dropped.
6. **Cost.** Runtime template changes in both renderers and one new warning; no key, no format
   change. A document whose column `visible:` reads `row.` renders differently, which is the fix.
7. **Considered.** (a) Change nothing. (b) Refuse `row.` in a column `visible:` by check: keeps the
   meaning nobody can use. (c) Chosen: judge it per row. For the idle read: a new `states.waiting`:
   a second empty state; `empty` already says what a section with no rows shows.

## Decisions

**Accept, redesigned.** The requester asked for no syntax; each is fixed by giving an existing key
its documented meaning, plus one warning (`board_widget_kinds`). No new format version.

## Source

`github:beyond10x/ess#723`
