---
format: aep.planning-md/3
id: story:the-generated-react-app-counts-new-live-rows-while-its-reader-is-paged-away-with-or-without-a-filter-and-keeps
kind: story
status: draft
title: The generated React app counts new live rows while its reader is paged away, with or without a filter, and keeps selected rows by the read's key, as ess-ui/1 documents
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
- depends_on: story:the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema
revision: 1
---
## Need

Two React behaviours contradict a default the loader fills (`g03-loader-fills-defaults`, rows
marked R).

- **`Live.when_paged_away`, default `count_new`** (`schemas/ui/ess-ui.schema.yaml:1422`).
  - The emitter passes `keep` to the live runtime only when the read has a `filter`
    (`crates/ui/ess-ui-react/src/emit.rs:2381-2390`). The runtime counts new rows only then
    (`templates/runtime/live.ts.tmpl:470-487`).
  - Without a filter, an absent value matches neither `count_new` nor `ignore`
    (`live.ts.tmpl:497-509`), so an event inserts its row while the reader is on page two.
  - The terminal counts it (`crates/ui/ess-ui-tui/src/app.rs:2511-2516`).
  - The earlier draft found this by reading the code, and the default table confirms it.
- **Row identity of the selection.** Rows are keyed by the read's `key`, else `live.match`, else
  `id` (`core.tsx.tmpl:233-234`, `rowKey`). But a collection writes its selected rows into its
  selection state as `row.id` (`composites/collection.tsx.tmpl:107`). With `key` filled
  (`g03-defaults-that-move-behaviour`), a read keyed by another field selects by `id` in React and
  by the key in the terminal (`app.rs:2648-2661`).

## Outcome

- The fill step writes `when_paged_away: count_new` on every section `live` block that leaves it
  out. The loader already refuses it on a nested node (`crates/ui/ess-ui/src/model.rs:1121`).
- The React live runtime applies `count_new`, `ignore` and `insert` the same way whether or not
  the read has a filter. While the reader is not on page one, a `count_new` event adds to the
  "new rows" count and inserts nothing.
- A collection writes its selected rows by `rowKey`, as the terminal does.

## Acceptance

New test file `crates/ui/ess-ui-react/tests/schema_behaviour.rs`, which runs the generated runtime
as `read_filter.rs` does. Each test is expected under 10 s and is timed. None may run over 60 s,
and the file's step stays within 120 s.

1. `a_paged_away_reader_counts_new_rows_without_a_filter`. Red today. A live section with
   `insert_top`, no `filter`, client paging and the reader on page 2. A scripted insert raises the
   new-rows count by one and leaves page 2's rows unchanged. The same document in the terminal
   gives the same count (a control).
2. `ignore_and_insert_are_obeyed_without_a_filter`. With `when_paged_away: ignore` the count stays
   0. With `insert`, the row is inserted.
3. `a_selection_is_kept_by_the_reads_key`. A collection whose read writes `key: code`, with rows
   whose `id` and `code` differ: selecting a row writes its `code` into the selection state, and a
   section that reads that selection receives the `code`.
4. Every existing `ess-ui-react` live and selection test passes. One whose expectation moves names
   this story.

Also: `CHANGELOG.md`, under Changed, lists both as fixes to what `ess-ui/1` documents.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 3. These are React disagreements that the default fill exposes.

1. **Need.** React behaves as the resolved document says, and as the terminal does.
2. **Class.** These are *defects*.
3. **Already expressible?** Writing `when_paged_away: count_new` does not help without a filter,
   because `keep` is not passed.
4. **Fit.** There is no new key.
5. **Second adopter.** Any live feed list that is paged by the client and has no filter.
6. **Cost.** React behaviour moves for such documents. Generated runtime bytes move.
7. **Considered.**
   - (a) Correct the schema to "count only with a filter". Rejected: the terminal and the
     reference agree on `count_new`.
   - (b) Chosen: fix React.

## Decisions

None.

## Scope

- cited: `crates/ui/ess-ui-react/src/emit.rs` (2381-2390, 2155-2160),
  `templates/runtime/live.ts.tmpl` (470-509), `templates/runtime/composites/collection.tsx.tmpl`
  (107)
- cited: `crates/ui/ess-ui/src/fill.rs` (one key)
- new: `crates/ui/ess-ui-react/tests/schema_behaviour.rs`; `CHANGELOG.md`
- Overlap: `g03-placement-in-loader` edits `emit.rs` 366-445, a different function. The active
  stories `story:a-client-row-filter-keeps-its-rows-without-paging-and-by-path-id` (`core.tsx.tmpl`,
  `data.ts.tmpl`) and `story:move-buttons-refuse-past-either-end` (`collection.tsx.tmpl`) touch the
  same templates; sequence with them.
- Size: M, about 250 changed lines (code about 70, tests about 180).

## Depends on

- `g03-loader-fills-defaults`
- `g03-defaults-that-move-behaviour` (`key` filled, for test 3)
