---
format: aep.planning-md/3
id: story:the-terminal-keeps-a-section-s-rows-while-its-read-runs-again-and-marks-rows-only-in-a-collection-that
kind: story
status: draft
title: The terminal keeps a section's rows while its read runs again, and marks rows only in a collection that declares a selection, as ess-ui/1 documents
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-ess-ui-loader-writes-every-constant-default-the-schema-declares-into-the-document-before-reading-it-from
- depends_on: story:a-program-lists-the-ess-ui-test-1-steps-a-ui-state-offers-applies-one-and-forks-the-state-through-ess-ui-test
revision: 1
---
## Need

Two defaults the loader fills (`g03-loader-fills-defaults`, rows marked T) are values the terminal
never reads, so it does the opposite of the documented default whatever the document says.

- **`SectionStates.refreshing`, default `keep_rows`** (`schemas/ui/ess-ui.schema.yaml:590`). A read
  that runs again is `Lifecycle::Loading` in the terminal (`crates/ui/ess-ui-tui/src/app.rs:1340`)
  and draws "loading…" (`view.rs:730`). The rows it holds are not drawn (`app.rs:2039-2040`). The
  field is read nowhere in `ess-ui-tui`. React keeps the rows
  (`crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:601`).
- **`collection.selection`, default `none`** (`:612`). The terminal toggles a row with space in
  every collection, whatever `selection` says (`app.rs:3216-3227`), so an absent value acts as
  `multiple`. React (`composites/collection.tsx.tmpl:111-114`) and the test runner
  (`crates/ui/ess-ui-test/src/runner.rs:787-794`) treat it as `none`.

With the defaults filled, an own generator reading `ess-ui-resolved/1` sees `keep_rows` and `none`.
The terminal must mean the same, or the resolved document describes a behaviour one stock renderer
does not have.

## Outcome

- While a section's read runs again, the terminal keeps drawing the rows it holds when
  `refreshing` is `keep_rows`. It draws its loading state only for `refreshing: skeleton`, or on
  the first load.
- Space marks a row only in a collection whose `selection` is `single` or `multiple` (or a record
  form whose `enabled` holds). In a `single` collection, marking a row unmarks the other.
- The footer hint shows the mark key only where marking is possible.

## Acceptance

New test file `crates/ui/ess-ui-tui/tests/schema_behaviour.rs`. Each test is expected under 3 s and
is timed. None may run over 60 s, and the file's step stays within 120 s.

1. `a_refresh_keeps_the_rows_drawn`. Red today. A section with `refresh: 5s` and a read latency of
   1 s, after its first answer: move the clock by 5.5 s. The rows are still drawn and "loading…" is
   not. With `refreshing: skeleton`, the loading state is drawn (a control).
2. `a_collection_without_selection_marks_nothing`. Red today. Space on a row of a collection that
   writes no `selection` marks nothing, and the hint offers no mark key. With `selection: multiple`
   two rows are marked. With `single` the second mark replaces the first.
3. `ess ui test` gives the same verdict for every corpus file. A corpus test whose verdict depended
   on marking in a collection without `selection` is listed, and its document writes the
   `selection` it meant.
4. The test runner's `mark` step and the Playwright spec agree with the terminal for each case of
   test 2 (`runner.rs:787-794` already refuses where React has no selection).

Also: `CHANGELOG.md`, under Changed, lists both as fixes to what `ess-ui/1` documents.

## Fit review

Read at tree `bcfc51555`. The request came from a downstream application on 2026-11-24, report
item 3. These are two renderer disagreements that the default fill exposes.

1. **Need.** The terminal behaves as the resolved document says.
2. **Class.** These are *defects*: the terminal contradicts the documented defaults and React.
3. **Already expressible?** No. Writing the key changes nothing, because the terminal does not
   read it.
4. **Fit.** There is no new key. The terminal follows React and the reference.
5. **Second adopter.** Any live dashboard with `refresh:` read in the terminal, which today
   blinks to "loading…" on every poll.
6. **Cost.** Terminal behaviour moves for documents that leave these keys out. No format.
7. **Considered.**
   - (a) Correct the schema to the terminal's behaviour. Rejected: React and the runner already
     follow the schema.
   - (b) Chosen: fix the terminal.

## Decisions

None.

## Scope

- cited: `crates/ui/ess-ui-tui/src/app.rs` (1340 lifecycle, 2039-2040 held rows, 3216-3227 the mark
  key), `crates/ui/ess-ui-tui/src/view.rs` (730-736 loading draw; the footer hint)
- new: `crates/ui/ess-ui-tui/tests/schema_behaviour.rs`
- `CHANGELOG.md`
- Overlap: `g01-session-actions` adds `next_due` to `app.rs`, a different function. The active
  stories `story:the-terminal-pages-a-server-read-through-its-pager-state` and
  `story:a-form-groups-own-action-runs-from-its-key` edit `app.rs` and `view.rs`; sequence with
  them.
- Size: M, about 260 changed lines (code about 90, tests about 170).

## Depends on

- `g03-loader-fills-defaults` (the values are present)
- `g01-session-actions` (same file, lands first)
