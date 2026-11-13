---
format: aep.planning-md/3
id: story:the-terminal-addresses-every-node-the-browser-addresses
kind: story
status: active
title: The terminal addresses every node the browser addresses
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#722
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:the-ui-test-runner-acts-on-an-action-inside-an-overlay
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/ui/ess-ui-test/src/parity.rs
- confidence: cited
  path: crates/ui/ess-ui-test/src/runner.rs
- confidence: cited
  path: crates/ui/ess-ui-test/src/screen.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/tests/journey_regions.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: inferred
  path: website/docs/reference/ess-ui-test.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:48:16Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:48:17Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A journey test cannot read or drive several nodes the generated React project addresses by
`data-ui-path`, because the terminal draws them without a region of their own or not at all
(beyond10x/ess#722 items 12, 17, 18, 19; 0.84.0, reproduced on 0.92.0):

- the rows of a collection that is an overlay's body, and their row actions (item 12): `expect rows`
  and `act` at `…/overlays/<o>/rows/<key>/…` are refused (`rows are a section's collection's`,
  `the terminal addresses rows of a section's collection`), although the reference says the runner
  addresses "rows of a section's or an overlay's collection";
- a form's `result` node (item 17): with `submit: {closes: false}` its text is drawn inside the
  overlay, but `expect` at `…/overlays/<o>/result` fails, "the terminal does not draw this node on
  cells of its own";
- a board's placed widgets (item 18): the React project draws each at `<board>/rows/<key>`
  (`crates/ui/ess-ui-react/templates/runtime/composites/board.tsx.tmpl:39`); the terminal draws the
  widget inside the section box, and `expect` there fails with `no node at …/rows/w-1`;
- the columns of a table wider than the test terminal (item 19): the headless screen is fixed at
  160 by 240 cells (`crates/ui/ess-ui-test/src/screen.rs:15-16`), a column past the edge is not
  drawn, and a test cannot widen it.

Minimal reproduction on 0.92.0 (a dialog `pick` with `component: collection`, rows `c-1`, `c-2` and
a row action `choose`; a drawer form with `submit: {closes: false}` and `result: {name: done,
primitive: text, text: Order created}`; a board `widget_by: type` placing one metric; a table of 13
columns with 20-to-30-character values):

```
failed  item 12 ... act pages/orders.list/overlays/pick/rows/c-2/row_actions/choose: ... the terminal addresses rows of a section's collection
failed  item 17b ... expect at pages/orders.list/overlays/create_stay/result: ... the terminal does not draw this node on cells of its own
failed  item 18a ... expect at pages/orders.board/sections/board/rows/w-1: no node at pages/orders.board/sections/board/rows/w-1
failed  item 19 ... expect at pages/orders.list/sections/list/rows/o-1/columns/c10: ... the terminal does not draw this node on cells of its own
```

A further node of the same class, from a consumer's UI document (2026-11-13, reproduced on ess
0.94.0, items 6 of its first and 9 of its second report): a node of a collection's `item`. With
`item: [{name: tile, component: tile, args: {title: row.title}}]`, the widget `tile` a body of a
heading `args.title`, a text `TILE-MARK` and a badge `FLAG-MARK`, the terminal draws the item once,
for the row under the cursor, in one line below the rows, in every `style` (`table`, `cards`,
`list`):

```
  request-one · Pending
  request-two · Pending
▸ request-one TILE-MARK [FLAG-MARK]
```

No row's own line holds its item, so a card grid reads as a list of column values, and a test
reads the item neither at the row nor at its own path:

```
failed ... expect at pages/requests.inbox/sections/list/rows/r1: expected "TILE-MARK" ...; it shows: request-one · Pending
failed ... expect at pages/requests.inbox/sections/list/rows/r1/item/tile: ... the terminal does not draw this node on cells of its own, so its text cannot be told from its neighbours'; expect text at the row, section, overlay or page instead
```

An `item` of primitives (`{name: mark, primitive: text, text: ITEM-MARK}`) is drawn the same way.
A tile with several body nodes cannot be tested node by node headless, and no row but the one under
the cursor shows its tile. (An earlier text of this case said the tile's text is drawn in the
row's line; that reading came from a tile whose text equalled the row's `title` column.)

## Acceptance

Journey tests in `crates/ui/ess-ui-test/tests/`, each red on 0.92.0:

- With an overlay whose body is a collection open, `expect: {at: …/overlays/<o>, rows: [c-1, c-2]}`
  holds, `select` and `mark` reach `…/overlays/<o>/rows/<key>`, and `act` at its
  `…/rows/<key>/row_actions/<a>` sends the action's command with the row's binds.
- After a submit of a form with `submit: {closes: false}`, `expect: {at: …/result, text: …}` reads
  the result node's own cells.
- `expect: {at: <board>/rows/<key>, text: …}` reads the placed widget's own cells, the path the
  React project draws it at.
- A table whose columns need more than 160 cells draws every column in the headless run: `expect`
  at `…/rows/<key>/columns/<last>` reads its value. The test file needs no key for it.
- Every node the terminal still does not address is refused by one parity rule with one reason in
  both renderers, never by `no node at` for a path the document declares.
- With a collection whose `item` holds a widget instance, `expect: {at: …/rows/<key>/item/<name>,
  text: …}` reads that item's text only, and a node of the widget's body is addressed below it as
  the React project addresses it by `data-ui-path`.

## Scope

- cited: crates/ui/ess-ui-test/src/runner.rs
- cited: crates/ui/ess-ui-test/src/screen.rs
- cited: crates/ui/ess-ui-tui/src/view.rs
- inferred: crates/ui/ess-ui-test/src/parity.rs
- inferred: crates/ui/ess-ui-test/tests/journey_regions.rs
- inferred: website/docs/reference/ess-ui-test.md
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A test reads and drives a node by the same path in both renderers
   (`website/docs/reference/ess-ui-test.md` § One verdict in both renderers); four kinds of node the
   React project addresses have no terminal region. Reproduction in § Need (`ess ui test` on 0.92.0).
   Requester's ask: a step or adapter option per item, or a statement that it is out of scope.
2. **Class.** Defect for items 12, 17 and 18: the reference claims overlay collections are addressed
   (`ess-ui-test.md` § Design notes, Limits) and the node path grammar names `result`, `rows` and the
   board's widgets (`website/docs/reference/ess-ui.md` § NodePath, `board_widgets`). Defect for item
   19: the reference says a cell is read whole "even where the terminal's screen cuts the value"
   (`ess-ui-test.md` § expect), but a column past the edge has no cell at all.
3. **Already expressible?** Partly. Item 17's text can be read at the overlay with `submit:
   {closes: false}` (a passing test in the triage reproduction); item 18's at the section box. Item
   12 and the item 19 cell cannot be reached.
4. **Fit.** Extends the existing region record (`crates/ui/ess-ui-tui/src/app.rs:59-71`): each node
   gets the region React's `data-ui-path` gives it. For item 19 the runner sizes the headless screen
   to the widest drawn table instead of adding a test key: no authored surface, and the browser
   already draws a table at its natural width in a scrolling frame (CHANGELOG 0.87.0 entry on
   table columns). The language grids do not apply (no ESS specification construct).
5. **Second adopter.** An inventory app whose "Pick a warehouse" dialog lists warehouses with a
   "Use" row action, and whose dashboard board places a stock metric per warehouse.
6. **Cost.** Terminal region recording and the runner's path resolution; a computed screen width in
   `screen.rs`. No format change in `ess-ui/1`, `ess-ui-test/1` or `ess-ui-test-report/1`
   (`website/docs/reference/formats.md:196-199`).
7. **Considered.** (a) Change nothing and list these nodes as refused: rejected for items 12 and 18,
   the reference already promises them. (b) A `size: {width, height}` key per test for item 19:
   rejected, it is new authored surface for a fact the runner can compute, and a too-small value
   would still hide columns. (c) Chosen: record the regions and size the screen to what is drawn.

## Decisions

Accept, redesigned (triage of beyond10x/ess#722, 2026-11-09): items 12, 17, 18 and 19 become one
feature, every node the browser addresses has a terminal region. Item 17's closing drawer is the
declared behaviour of a form that closes on submit; the idiom is `submit: {closes: false}`, and this
story makes its `result` addressable. Item 19 gets no test key; the headless screen grows to the
widest table.

## Source

github:beyond10x/ess#722

## Reported case (2026-11-13, a third list)

A consumer specification's UI document (ess 0.94.0, item 6 of its third collected list): a
collection nested in a record's `item`, with a row action. On a record section `summary` of the
page `requests.detail` with
`item: [{name: more, component: collection, reads: {view: …PendingRequests}, columns: [title], row_actions: [{name: approve, does: …ApproveRequest, bind: {id: row.id}, label: Approve}]}]`
the terminal draws the nested rows and the action's key under the record's fields:

```
title            request-one
state            Pending
  title
  request-one
  request-two
page 1/1 · 2 rows
a Approve
```

and a test reaches neither the collection nor its rows nor the action:

```
failed ... expect at pages/requests.detail/sections/summary/item/more: … the terminal draws a collection nested inside a record, a tab or a node without addressing it or its rows; address a section's or an overlay's own collection instead
failed ... act pages/requests.detail/sections/summary/item/more/rows/r1/row_actions/approve: … (the same reason)
```

The refusal is one rule with one reason, as this story's acceptance asks for a node the terminal
does not address. What the report adds: the action is drawn with a key and no journey can run it,
so a command a user can send from the terminal has no test.

Proposed acceptance line: `expect rows`, `select` and `act` reach the rows and row actions of a
collection nested in a record's `item` at `…/sections/<s>/item/<name>/rows/<key>/…`, the path the
React project addresses, or the terminal does not offer a key for an action no test can run.
