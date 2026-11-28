---
format: aep.planning-md/3
id: story:a-collection-a-widget-draws-takes-states-and-is-addressed-by-a-test-like-a-sections-collection
kind: story
status: draft
title: A collection a widget draws takes states for its read and is addressed, with its rows, by a test like a section's collection
summary: A collection in a widget body refuses states, and ess ui test cannot address it or its rows; a section's collection does both
tags:
- adopter-report
- ess-ui
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
- informed_by: story:the-terminal-addresses-every-node-the-browser-addresses
revision: 1
---
## Need

A widget is the unit of reuse of an `ess-ui/1` document (`website/docs/reference/ess-ui.md` § Widget).
A widget whose body is a collection, the rows of one view, can do less than the same collection
written as a section's own body, in two things a document and a test do with a collection.

Reported by a downstream specification of reusable building blocks, which generates one widget per
view (ess 0.99.2, findings G2 and G3 of one collected report). Both reproduce on 0.99.9 with a
neutral model and document: `.engineering/repro/a-widgets-collection-is-a-sections-collection/`
(`run.sh`, `transcript.txt`).

- **G2, no states.** `states: {empty: {message: …}}` on a collection inside a widget body is refused:
  `document_loads widgets/item_list/body/rows: collection: unknown field states, expected one of
  reads, columns, sort, style, selection, row_actions, bulk_actions, actions, expand, item,
  reorder, group_by, group_order, show_empty_groups`. `states` is a key of `Section`
  (`crates/ui/ess-ui/src/model.rs:872`); `Collection` denies unknown fields
  (`model.rs:1338`). The same refusal holds for `load`, `title` and `depends_on`; `live`,
  `visible` and `degrades` are accepted on the nested collection. Writing `states` on the section
  that draws the widget is accepted and does nothing for the widget's read: the section reads
  nothing, so a test with a widget that reads no row finds the section `ready`, not `empty`
  (`transcript.txt`, third command). A widget with a list has no empty, loading, failed or stale
  text of its own.
- **G3, not addressed.** `ess ui test` cannot address a collection nested in a widget, or its rows.
  `expect` at `pages/home/sections/items/body/rows` and at `…/rows/rows/it-02` fails with "the
  terminal draws a collection nested inside a record, a tab or a node without addressing it or its
  rows; address a section's or an overlay's own collection instead"
  (`crates/ui/ess-ui-test/src/parity.rs:169`; the reference lists the rule in
  `website/docs/reference/ess-ui-test.md` § One verdict in both renderers). The same two steps at a
  section's own collection pass. The text of the section that draws the widget is the only place a
  test can read the rows, and it is cut at the screen width: a value at the end of a long row is
  not found (`transcript.txt`, last command: the last word of a label of about 90 characters
  is missing from the box), while the row of a section's own collection is read whole.

The reporter's wording is theirs: "states on a collection inside a widget" and "address a collection
nested inside a widget, or its rows".

## Outcome

A collection a widget's body draws is a collection like a section's for what a document declares and
what a test reads:

- it takes `states` for its own read, in the shape a section's `states` has, and shows them in both
  renderers;
- a test addresses it and its rows by node path, the path the generated React project draws as
  `data-ui-path`, and reads a row whole.

## Acceptance

Per finding. Tests in `crates/ui/ess-ui/tests/` (the loader), `crates/ui/ess-ui-test/tests/` (the
journey) and the React project's Playwright spec; each is timed and expected under 5 s; none runs
over 60 s, and the crates' test steps stay within 120 s.

- **G2.** The widget of the reproduction with `states: {empty: {message: No items yet}}` on its
  collection passes `ess ui check --model` with 0 errors (red today: the `document_loads` error
  above). With a fixture that answers no row, `expect: {at: <section that draws the widget>, text:
  No items yet}` holds in the terminal and in the Playwright spec, and the nested read's `loading`
  state shows its declared skeleton while the read is in flight (latency). A key that stays a
  section's (the decision below) is refused naming the node and saying it belongs to a section.
- **G3.** `expect: {at: pages/home/sections/items/body/rows, rows: [it-01, it-02]}` and `expect: {at:
  …/body/rows/rows/it-02, text: Robot}` hold in the terminal (red today: the parity refusal above);
  the Playwright spec for the same file is not `test.fixme`. A label longer than the screen is read
  whole at its row: the repro's `zebra` test passes.
- **Refusals.** A path under a widget's body that names no node fails with `no node at <path>`; the
  parity table of `ess-ui-test.md` no longer lists a collection nested in a widget's body; a
  collection nested in a record's `item` stays as the decision D111 of
  `story:the-terminal-addresses-every-node-the-browser-addresses` leaves it.
- Documents that passed keep their bytes: `ess ui load` of every document of the test corpus and of
  `examples/partner-portal` is unchanged.

## Fit review

Written with `.agents/skills/assessing-external-requests/SKILL.md`. The consumer is a downstream
specification of reusable building blocks.

1. **Need, apart from the syntax.** A reusable list needs its own empty and loading text, and a test
   needs to read its rows. Reproduction: the neutral model and document in the repro directory
   (a `shop.channel.Items` view, a widget `item_list` of one collection, a page that uses it).
   Theirs: allow `states` on a collection inside a widget; address that collection and its rows in
   `ess ui test`.
2. **Class.** G2: gap, the document cannot say it (a section's `states` do not reach a widget's
   read). G3: gap in the test runner, listed as a parity rule by design
   (`ess-ui-test.md` § One verdict), not a defect; the reference promises rows only of a section's or
   an overlay's collection.
3. **Already expressible?** G2: no. The idiom "put the collection in a section" gives up the widget,
   which is the point of the request; it moves the reusable arrangement out of the specification.
   G3: the section's box text, cut at the screen width (above), so no.
4. **Fit.** Grid R (what can be read where): a nested node with its own `reads` takes `live` today
   (`website/docs/reference/ess-ui.md` § Composites), so states are the sibling of a key already
   open there, not a new row. The siblings that read nested (a tab's node, a header metric, a record's
   `item`) are the same cell: the decision below says whether `states` opens for all of them. The
   path grammar already names a widget instance's expanded body (`ess-ui.md`, `widget_instances:
   {expanded_body_path: <instance path>/body/<node name>}`). Targets: the terminal, the React
   project, the Playwright spec, `ess ui check`, the schema and the generated reference page; the
   interpreter and the other generators do not read `ess-ui`. I do not know whether the React
   project already draws `data-ui-path` on a widget's nested collection and its rows; the first test
   of the story reads that.
5. **Second adopter.** A dashboard whose "recent orders" widget is reused on the overview page and on
   a customer page, and needs "No orders yet" in both.
6. **Cost.** An authored key that `ess-ui/1` refuses is accepted on a nested collection: schema
   (`schemas/ui/ess-ui.schema.yaml`), the generated `ess-ui.md`, the check list, `CHANGELOG.md`.
   Whether this stays inside `ess-ui/1` is the format owner's call; a release that carries this key
   names the decision's pick in its pull-request line. No change to `ess-ui-test/1`. Documents that
   passed stay valid.
7. **Considered.** (a) Change nothing: the author keeps a collection in a section and gives up the
   widget. (b) The reporter's shape, `states` on a collection in a widget body alone: taken in part,
   widened by the decision below so a nested record or metric is not left out. (c) A `states` key on
   the widget itself covering every read of its body: rejected as a second spelling of the section's
   key, and a widget may hold several reads.

## Decisions

All open; each names its recommended option.

- **WC-D1, which keys of a section's frame a nested node with its own `reads` takes.** A: `states`,
  for every nested composite with its own `reads` (collection, record, metric), in the section's
  shape; `load`, `depends_on` and `title` stay a section's, refused on a nested node naming why (they
  schedule or name a section). B: the whole frame. C: change nothing. Recommended: A. `live` is the
  precedent and `states` is what a reader of a read shows; `load` and `depends_on` decide when a
  section reads, which a node inside it does not. Fails closed: A refuses the keys it does not open.
- **WC-D2, the path of a widget's nested collection.** A: `<instance path>/body/<node name>` and its
  rows `<that path>/rows/<key>`, the path the expanded body already has. B: the rows addressed at the
  section that draws the widget when the body holds one collection. Recommended: A. B depends on how
  many collections the body holds and breaks when one is added; the path with `body/` is what the
  reference names.
- **WC-D3, reading a long value.** A: a row is read whole (it already is for a section's own
  collection); no test key; the headless screen follows the decision of
  `story:the-terminal-addresses-every-node-the-browser-addresses` for a wide table. B: a width key
  per test (rejected there). Recommended: A.

## Scope

- `crates/ui/ess-ui/src/model.rs` (`Collection`, nested `states`), `crates/ui/ess-ui/src/path.rs`
  (states of a nested node), `crates/ui/ess-ui/src/expand.rs` (expanded body)
- `crates/ui/ess-ui-test/src/parity.rs`, `crates/ui/ess-ui-test/src/runner.rs`
- `crates/ui/ess-ui-tui/src/view.rs`, `crates/ui/ess-ui-react/templates/runtime/`
- `schemas/ui/ess-ui.schema.yaml`, `website/docs/reference/ess-ui.md`,
  `website/docs/reference/ess-ui-test.md`, `CHANGELOG.md`
