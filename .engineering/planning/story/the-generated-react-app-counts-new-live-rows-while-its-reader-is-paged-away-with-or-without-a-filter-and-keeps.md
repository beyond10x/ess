---
format: aep.planning-md/3
id: story:the-generated-react-app-counts-new-live-rows-while-its-reader-is-paged-away-with-or-without-a-filter-and-keeps
kind: story
status: draft
title: React counts paged-away live rows without a filter and keeps a selection by the read's key
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema
revision: 3
---
## Need

**A defect story outside P0.** These are React defects that exist today, whatever the fill writes
(scope finding S4). The fill of `Live.when_paged_away` is a row of `g03-loader-fills-defaults`.

- **`Live.when_paged_away`, default `count_new`** (`schemas/ui/ess-ui.schema.yaml:1422`).
  - The emitter passes `keep` to the live runtime only when the read has a `filter`
    (`crates/ui/ess-ui-react/src/emit.rs:2381-2390`). The runtime counts new rows only then
    (`templates/runtime/live.ts.tmpl:470-487`).
  - Without a filter, the value matches neither `count_new` nor `ignore` (`live.ts.tmpl:497-509`),
    written or absent. So an event inserts its row while the reader is on page two.
  - The terminal counts it (`crates/ui/ess-ui-tui/src/app.rs:2511-2516`).
- **Row identity of the selection.** Rows are keyed by the read's `key`, else `live.match`, else
  `id` (`core.tsx.tmpl:233-234`, `rowKey`). But a collection writes its selected rows into its
  selection state by `row.id` (`composites/collection.tsx.tmpl:107`). So a read that writes `key`
  selects by `id` in React, and by the key in the terminal (`app.rs:2648-2661`).

## Outcome

- The React live runtime applies `count_new`, `ignore` and `insert` in the same way whether or not
  the read has a filter. While the reader is not on page one, a `count_new` event adds to the
  "new rows" count and inserts nothing.
- A collection writes its selected rows by `rowKey`, as the terminal does.

## Acceptance

New test file `crates/ui/ess-ui-react/tests/schema_behaviour.rs`, which runs the generated runtime
as `read_filter.rs` does. Each test is expected under 10 s and is timed; none may run over 60 s,
and the crate's test step stays within 120 s. The file joins the crate's single test binary of
`story:the-test-step-meets-the-test-budget` if that has landed; otherwise it is a new file, which
that story moves.

1. `a_paged_away_reader_counts_new_rows_without_a_filter`. Red today. A live section with
   `insert_top`, no `filter`, client paging, and the reader on page 2. A scripted insert raises
   the new-rows count by one and leaves page 2's rows unchanged. The same document in the
   terminal gives the same count (a control).
2. `ignore_and_insert_are_obeyed_without_a_filter`. With `when_paged_away: ignore`, the count stays
   0. With `insert`, the row is inserted.
3. `a_selection_is_kept_by_the_reads_key`. A collection whose read writes `key: code`, with rows
   whose `id` and `code` differ. Selecting a row writes its `code` into the selection state, and a
   section that reads that selection receives the `code`.
4. The live and selection tests of `ess-ui-react` pass: `live_binding.rs`, `live_composites.rs`,
   `adversary_live_binding_pass1.rs`, `adversary_live_binding_pass2.rs`, `section_row.rs` and
   `collection_controls.rs`. A moved expectation in one of them names this story.

Gate:
- `task fmt-check`;
- `cargo clippy -p ess-ui-react --all-targets --locked -- -D warnings`;
- `cargo test -p ess-ui-react --locked`.

Golden files: none of P0's golden files moves; they hold no emitted React code.

`CHANGELOG.md`, under Changed: "**Breaking for a generated React project**: a paged-away live list
counts new rows without a filter, and a selection holds the read's `key`, as `ess-ui/1`
documents."

## Fit review

Read at tree `bcfc51555`. Found while answering report item 3 of a downstream application
(2026-11-24). It is outside item 3: the defects exist whatever the fill writes.

1. **Need.** React behaves as the resolved document says, and as the terminal does.
2. **Class.** These are *defects*.
3. **Already expressible?** No. Writing `when_paged_away: count_new` does not help without a
   filter, because `keep` is not passed. Writing `key` does not change what React selects by.
4. **Fit.** No new key.
5. **Second adopter.** Any live feed list that is paged by the client and has no filter.
6. **Cost.** React behaviour moves for such documents, and generated runtime bytes move.
7. **Considered.**
   - (a) Correct the schema to "count only with a filter". Rejected: the terminal and the
     reference agree on `count_new`.
   - (b) Chosen: fix React.

## Decisions

None. It leaves P0 by scope finding S4.

## Scope

- `crates/ui/ess-ui-react/src/emit.rs`: 2381-2390.
- `crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl`: 470-509.
- `crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl`: 107.
- new: `crates/ui/ess-ui-react/tests/schema_behaviour.rs`.
- `CHANGELOG.md`.
- Overlap: the active stories
  `story:a-client-row-filter-keeps-its-rows-without-paging-and-by-path-id` (`core.tsx.tmpl`,
  `data.ts.tmpl`) and `story:move-buttons-refuse-past-either-end` (`collection.tsx.tmpl`) touch
  the same templates; sequence with them.
- Size: M, about 200 changed lines: code about 50, tests about 150.

## Depends on

- `g03-defaults-that-move-behaviour`: `key` filled, for test 3.

It runs in no P0 round.
