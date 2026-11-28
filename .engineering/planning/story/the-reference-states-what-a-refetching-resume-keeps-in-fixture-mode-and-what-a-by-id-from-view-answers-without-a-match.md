---
format: aep.planning-md/3
id: story:the-reference-states-what-a-refetching-resume-keeps-in-fixture-mode-and-what-a-by-id-from-view-answers-without-a-match
kind: story
status: draft
title: The reference states what a refetching resume keeps in fixture mode and what a by_id_from view answers without a match, and both renderers follow it
tags:
- ess-ui
- external-request
- gap-reports-2026-11-25
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
revision: 1
---
## Outcome

Two behaviours that a renderer has to pick today are stated once in the `ess-ui/1` reference, and
the terminal, the test runner and the generated React app follow the statement: what a
`resume: refetch` channel keeps when its section reads fixtures, and what a derived fixture view
(`by_id_from`) answers when no row matches its key. A third behaviour the requester names, where
`insert_or_patch` puts a key the section does not hold, is already owned by another story (below).

## Context

A downstream page generator found them on 0.98.0 while writing its own generator; it had to choose
for each, and nothing tells it whether it chose as ESS means.

- **`resume: refetch` in fixture mode.** After a reconnect the terminal marks every read the channel
  feeds as read again (`crates/ui/ess-ui-tui/src/app.rs:2136-2138`, `refetch_fed` `:2384-2422`) and
  keeps what the read answered while it loads (`ReadState::again`, `:117-127`); React calls the read's
  `refetch` for each section that listens (`crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl:292-294`,
  `:462-463`). The fixture adapter answers from files and holds no row an event made. Whether rows
  applied from live events survive that re-read is written nowhere: the reference says only
  "behaviour after reconnect" (`website/docs/reference/ess-ui.md:2262`) and "the channel's resume
  decides replay or re-read" (`:2434`). The requester reports that React keeps the live rows
  only because the re-read rows compare equal (not checked here), so the two renderers may differ.
- **A `by_id_from` view with no matching row.** React answers the source's first row
  (`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl:229-235`, the fallback `source.rows.slice(0, 1)`).
  The terminal answers no row (`crates/ui/ess-ui-tui/src/data.rs:466-478`, `retain` at `:475`).
  The reference says only "views answered from another fixture" (`ess-ui.md:2197`). The two
  renderers disagree here today, against "a test file means the same in the terminal and in the
  browser" (`ess-ui-test.md:412-417`).
- **`insert_or_patch` of a key the section does not hold.** Both renderers put the row first. The
  active story
  `story:a-chart-draws-its-rows-in-the-order-its-sort-declares-and-a-live-insert-keeps-that-order`
  already has the reference's `Live` section say where `insert_or_patch` and `insert_top` put a
  new row and that a declared sort orders the rows after the effect; this story adds nothing for it
  and that story's fit review names the finding.

Neighbours, already stories and not repeated: the loop period of a looping script
(`story:a-looping-fixture-script-repeats-at-its-last-entry-s-offset-in-the-browser-too-as-in-the-terminal-and-the-test`),
an event before the first read answers
(`story:in-the-browser-a-live-event-that-arrives-before-the-section-s-read-has-answered-is-applied-once-it-answers-as`,
whose decision SP-D4 also names what a refetch does with applied events during an outstanding
read), and `insert_top` of a held key
(`story:a-live-insert-of-a-key-the-section-already-holds-shows-one-row-in-both-renderers`).

## Acceptance

1. Red first: a test in `crates/ui/ess-ui-test/tests/` over fixtures where `by_id_from` finds no
   match gives the same rows in the terminal and in the Playwright spec's expected rows; on
   `2e055bcd0c` the terminal shows none and React the first row.
2. A test plays an event that sets a field of a held row, takes the channel down and back up with
   `resume: refetch`, and holds the rows; the terminal and the React runtime give the same rows,
   the ones decision RS-D1 names.
3. The reference states, for each of the two, one behaviour: in the `Channel` and `Live` text for
   the re-read, and in the fixture index's `derived` text for the missing match. Both are generated
   from `schemas/ui/ess-ui.schema.yaml`, so the schema text changes and `ess ui docs` is run.
4. For each, an `ess-ui-test/1` test pins it in the terminal and the Playwright spec pins it in
   the generated React app, or is `test.fixme` with a reason from the parity rules.
5. `CHANGELOG.md` names each behaviour that changed in a renderer, under `### Fixed`.

## Fit review

Tree `2e055bcd0c` (branch `plan/gap-reports-2026-11-24`), read-only. This story covers requirement 2
of a downstream page generator's report of 2026-11-25, and its third item is a covered finding of
the chart story named above.

1. **Need.** A generator, or a second renderer, has two cases where ESS says nothing and ESS's own
   two renderers may answer differently. Reproduction for the second (fresh, not run here): a
   fixture index with `derived: {orders.ById: {by_id_from: orders.Page, key: id}}`, a page that
   reads `orders.ById` with `params: {id: o-9}` while `orders.Page` holds `o-1` and `o-2`. The
   requester's proposed fixes are not stated: it asks for statements.
2. **Class.** *Defect* for `by_id_from` (the renderers disagree, and the parity rule says they
   must not: `ess-ui-test.md:412-417`); *gap* for the re-read (unspecified, and not shown to
   differ yet).
3. **Already expressible?** No. No document can say what a missing match answers or what a re-read
   keeps; both are renderer behaviour.
4. **Fit.** No grid applies (`docs/design/language-grids.md:15`). It adds no key. It follows the
   existing rule that one verdict holds in both renderers, and the documents it changes are the
   reference pages already generated from the schema. Every target is named: the terminal, the
   test runner and the React runtime.
5. **Second adopter.** Any application with a detail page over a derived fixture view, or with a
   section that reads fixtures behind a reconnecting channel.
6. **Cost.** A documented behaviour change in one renderer for `by_id_from` and possibly for the
   re-read; no format change and no new diagnostic unless RS-D2 C is chosen.
7. **Considered.** (a) Change nothing. (b) State what each renderer does today and leave them
   different. (c) State one behaviour, hold both renderers and the tests to it (chosen).

**Accept as proposed** for the outcome; each behaviour is an open decision below.

## Decisions

Would add to `ess-ui/1`: nothing. Would add to `ess-ui-test/1`: nothing.

- **RS-D1. What does a `resume: refetch` re-read keep in fixture mode?** (A) The re-read answer
  replaces the held rows: rows applied from live events do not survive it, as a served re-read
  would return what the server holds. A test of resume uses a fixture answer that holds what the
  events set. (B) Rows applied from live events stay and the answer is merged under them. (C) The
  fixtures absorb events as fixture state, as `fixture-mode-applies-a-confirmed-commands-effect`
  does for a confirmed command, so the re-read answers them. **Recommended: A.** Open, the
  coordinator's. C is a larger design and belongs in its own story.
- **RS-D2. What does a `by_id_from` view answer without a matching row?** (A) No row. (B) The
  source's first row, as React does. (C) Refuse the run: a derived fixture view whose key finds no
  row for a page's params is an error of the run (a finding, as a mistyped fixture is). A is the one
  that fails closed on the page: it shows no row, not another record's; C fails closed on the
  run. **Recommended: A.** Open, the coordinator's.

## Source

A downstream page generator, report of 2026-11-25, requirement 2.
