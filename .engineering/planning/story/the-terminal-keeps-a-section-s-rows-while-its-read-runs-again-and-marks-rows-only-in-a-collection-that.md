---
format: aep.planning-md/3
id: story:the-terminal-keeps-a-section-s-rows-while-its-read-runs-again-and-marks-rows-only-in-a-collection-that
kind: story
status: draft
title: The terminal keeps rows while refreshing, marks rows only where selection allows, and keeps a read's cache by its own store
tags:
- gap-p0
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:the-loader-fills-the-defaults-a-renderer-reads-differently-today-and-both-renderers-then-show-what-the-schema
- depends_on: story:the-loader-resolves-every-state-s-store-and-scope-and-every-read-s-cache-store-from-the-placement-profile-once
revision: 3
---
## Need

**A defect story outside P0.** These are terminal defects that exist today, whatever the document
says. The P0 fill writes these keys, but the terminal reads none of them, so the fill moves
nothing (scope finding S3).

- **`SectionStates.refreshing`, default `keep_rows`** (`schemas/ui/ess-ui.schema.yaml:590`).
  - A read that runs again is `Lifecycle::Loading` in the terminal
    (`crates/ui/ess-ui-tui/src/app.rs:1340`) and draws its loading state (`view.rs:730-736`).
  - The rows it holds are not drawn (`app.rs:2039-2040`).
  - The field is read nowhere in `ess-ui-tui`. React keeps the rows
    (`crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:601`).
- **`collection.selection`, default `none`** (`:612`). The terminal toggles a row with space in
  every collection, whatever `selection` says (`app.rs:3216-3227`), so an absent value acts as
  `multiple`. React (`composites/collection.tsx.tmpl:111-114`) and the test runner
  (`crates/ui/ess-ui-test/src/runner.rs:787-794`) treat it as `none`.
- **A read's own cache store.** The terminal keeps or clears its read cache by the document-level
  `view_cache` class only (`app.rs:2769-2771`, `placement.rs:130-139`). It ignores the `cache`
  that `g03-placement-in-loader` fills on each read from its section and page (scope finding S5).

Not covered here, and each a defect outside P0: the terminal ignores `Preload.policy`
(`app.rs:1555-1570`); no renderer reads `Navigation.visibility`, `references.navigates` or
`Channel.scope`; no renderer clears by `State.clear_on`.

## Outcome

- While a section's read runs again, the terminal keeps drawing the rows it holds when
  `refreshing` is `keep_rows`. It draws its loading state only for `refreshing: skeleton`, or on
  the first load.
- Space marks a row only in a collection whose `selection` is `single` or `multiple`, or in a
  record form whose `enabled` holds. In a `single` collection, marking a row unmarks the other.
- The footer hint shows the mark key only where marking is possible.
- The terminal keeps or clears each read's cache across a page change by that read's written
  `cache.store`.

## Acceptance

New test file `crates/ui/ess-ui-tui/tests/schema_behaviour.rs`. Each test is expected under 3 s and
is timed; none may run over 60 s, and the crate's test step stays within 120 s. The file joins the
crate's single test binary of `story:the-test-step-meets-the-test-budget` if that has landed;
otherwise it is a new file, which that story moves.

1. `a_refresh_keeps_the_rows_drawn`. Red today. A section with `refresh: 5s` and a read latency of
   1 s, after its first answer, then the clock moved by 5.5 s:
   - the rows are still drawn, and neither skeleton bars nor a "loading…" line are;
   - with `refreshing: skeleton`, skeleton bars are drawn (a control).

   This holds whichever order this story and `g03-defaults-that-move-behaviour` land in, because
   both cases name the text they draw.
2. `a_collection_without_selection_marks_nothing`. Red today. Space on a row of a collection that
   writes no `selection` marks nothing, and the hint offers no mark key. With `selection:
   multiple`, two rows are marked. With `single`, the second mark replaces the first.
3. `the_runner_and_the_spec_agree_with_the_terminal`. For `none`, `multiple` and `single`, the
   runner's `mark` step is refused or taken exactly where the terminal marks. The emitted
   Playwright spec holds a mark click for `multiple` and `single`, and none for `none`, asserted
   on the spec text.
4. `a_thin_section_keeps_its_reads_cached_on_the_server`. In a `fat` document, the cache of a read
   in a `thin` section follows that read's `cache.store` across a page change, not the document's.
5. `ess ui test` gives the same verdict for every test file of the partner portal and of the
   ess-ui-test fixtures. A test whose verdict depended on marking in a collection without
   `selection` is listed, and its document writes the `selection` it meant.

Gate:
- `task fmt-check`;
- `cargo clippy -p ess-ui-tui --all-targets --locked -- -D warnings`;
- `cargo test -p ess-ui-tui -p ess-ui-test --locked`.

Golden files: none of P0's golden files moves, because the terminal's behaviour is not in them.

`CHANGELOG.md`, under Changed: the terminal keeps rows while refreshing, marks rows only where
`selection` allows, and keeps a read's cache by its own store, as `ess-ui/1` documents.

## Fit review

Read at tree `bcfc51555`. Found while answering report item 3 of a downstream application
(2026-11-24). It is outside item 3: the defects exist whatever the fill writes.

1. **Need.** The terminal behaves as the resolved document says, and as React does.
2. **Class.** These are *defects*: the terminal contradicts the documented defaults and React.
3. **Already expressible?** No. Writing the key changes nothing, because the terminal does not
   read it.
4. **Fit.** No new key. The terminal follows React and the reference.
5. **Second adopter.** Any live dashboard with `refresh:` read in the terminal, which today blinks
   to its loading state on every poll.
6. **Cost.** Terminal behaviour moves for documents that leave these keys out. No format.
7. **Considered.**
   - (a) Correct the schema to the terminal's behaviour. Rejected: React and the runner already
     follow the schema.
   - (b) Chosen: fix the terminal.

## Decisions

None. It leaves P0 by the operator's decision of 2026-11-24 (scope finding S3).

## Scope

- `crates/ui/ess-ui-tui/src/app.rs`: 1340 lifecycle, 2039-2040 held rows, 2769-2771 read cache,
  3216-3227 mark key.
- `crates/ui/ess-ui-tui/src/view.rs`: 730-736 loading draw; the footer hint.
- `crates/ui/ess-ui-tui/src/placement.rs`: 130-139, the read cache store.
- new: `crates/ui/ess-ui-tui/tests/schema_behaviour.rs`.
- `CHANGELOG.md`.
- Overlap: the active stories `story:the-terminal-pages-a-server-read-through-its-pager-state` and
  `story:a-form-groups-own-action-runs-from-its-key` edit `app.rs` and `view.rs`; sequence with
  them.
- Size: M, about 340 changed lines: code about 120, tests about 220.

## Depends on

- `g03-defaults-that-move-behaviour`: it changes the loading draw in `view.rs`.
- `g03-placement-in-loader`: the filled `cache.store`.

It runs in no P0 round.
