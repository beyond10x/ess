---
format: aep.planning-md/3
id: story:a-page-shows-its-title-total-badge-and-live-status-and-marks-the-current-page
kind: story
status: draft
title: A page shows its title, a counted total, a badge and its live status, and marks the current page
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- decomposes: epic:zero-open-issues-2026-11
revision: 2
---
## Need

The page header and the navigation of the generated React app do not tell the reader where they
are or what the header's figures mean. On ess 0.92.0 (`ess generate ui --target react`, minimal
document in the triage's `repro/base.ui.yaml`):

- `header.total` is a bare number beside the title: `<span className="ui-total">{total}</span>`
  (`crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:516`); the terminal writes `(N)`
  (`crates/ui/ess-ui-tui/src/view.rs:403-407`).
- `header.live` prints `<channel>: <lifecycle>` as raw text (`core.tsx.tmpl:518-519`), e.g.
  `orders: connecting`.
- `header.switch` and `switch_to` leave the current page out (`crates/ui/ess-ui-react/src/emit.rs:2463-2468`
  filters `target != name`), so the switch on `orders.list` offers only `Orders as cards`, and no
  entry is marked active.
- No navigation link marks the current page: `grep -rn aria-current` over the generated `src/`
  finds nothing; the terminal marks it with `▸` (`view.rs:263-268`).
- A header cannot carry a badge: `header: {…, badge: {text: Beta, tone: info}}` is refused with
  `unknown field `badge``.
- A page with no `header` shows no title at all: `PageFrame` draws only its children
  (`core.tsx.tmpl:286-291`), while the terminal draws the page title (`view.rs:383-387`). An
  auth-shell page whose kind gives it an empty section is then a blank page.

## Acceptance

Each a named test in `crates/ui/ess-ui-react/tests/page_header.rs` unless it says otherwise:

- `a_total_reads_as_a_count`: `header.total` draws the section's total grouped by the locale in a
  count mark after the title, with an accessible name `<total> <section title, else "rows">`.
- `live_status_is_a_labelled_mark`: each `header.live` channel draws a status mark whose text is
  the lifecycle in words (`Live`, `Connecting`, `Reconnecting`, `Stale`, `Closed`) and whose title
  names the channel; `data-lifecycle` stays.
- `the_switch_offers_and_marks_the_current_page`: the switch lists every page of `switch` and
  `switch_to` and the current page, in declared order, as one segmented control; the current entry
  has `aria-current="page"` and does not navigate.
- `navigation_marks_the_current_page`: the navigation link of the current page has
  `aria-current="page"`.
- `a_header_badge_draws_its_text_and_tone`: `header.badge: {text: <expr>, tone: <tone>}` loads
  (`crates/ui/ess-ui/tests/`), an unknown tone is refused by path, and React draws a badge in that
  tone after the title.
- `a_page_without_a_header_shows_its_title`: a page with `title` and no `header` draws the title as
  its heading.
- The terminal draws the badge after the title, the live status as `● <lifecycle>` and marks the
  current switch entry (`crates/ui/ess-ui-tui/tests/page_header.rs`).
- A generated page file changes only where its page has a header `switch` or `switch_to` (the
  current entry is added) or a `badge`; the runtime templates change.

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl`
- cited: `crates/ui/ess-ui-react/templates/runtime/navigation.tsx.tmpl`
- cited: `crates/ui/ess-ui-react/src/emit.rs`
- cited: `crates/ui/ess-ui-tui/src/view.rs`
- cited: `crates/ui/ess-ui/src/model.rs`
- cited: `schemas/ui/ess-ui.schema.yaml`
- inferred: `crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl`
- inferred: `crates/ui/ess-ui-react/tests/page_header.rs`
- inferred: `crates/ui/ess-ui-tui/tests/page_header.rs`
- inferred: `website/docs/reference/ess-ui.md`
- confidence: high

## Fit review

Tree `995e29579d`, binary ess 0.92.0. Covers #723 items G19, B13 (the unpaged half; a paged list
shows `1-10 of 30` since 0.91.0), G20/B4, G21, G22, D14 and "navigation has no current-page mark";
D7 does not reproduce (a `settings_page` with `header: from_page` draws its `h1` on 0.92.0).

1. **Need.** A reader sees which page they are on, what a header figure counts, whether live
   updates flow, and a status of the page as a whole. Reproductions above and in
   `repro/run.txt`. The requester's proposals, theirs: a total "with a label", a switch "with the
   active mark", a header badge.
2. **Class.** Total, live, switch, navigation and a page with no header: *defect* (the React
   renderer draws less than the terminal draws for the same document, `view.rs:263-268`, `:383-407`,
   and the schema calls `switch` "pages … shown as a view switch", `schemas/ui/ess-ui.schema.yaml`
   `Page.doc`). Header badge: *gap*.
3. **Already expressible?** The badge: no (refused above); a `metrics` node in the header can
   show a number, not a status word with a tone. The rest needs no authored surface.
4. **Fit.** The language grids do not apply (`docs/design/language-grids.md:15`, `:63`). The badge
   reuses the badge primitive's words (`text` an expression, `tone` from the closed tone set,
   `schemas/ui/ess-ui.schema.yaml` `badge`), as decided in `approval-record:ess-0870-wave` (G-3,
   answer a: `header.badge: {text, tone}`, `text` an expression). Every other change brings React
   to what the terminal already draws.
5. **Second adopter.** An admin console whose list page shows `1,204 tickets`, a live mark for its
   event feed, a `Read only` badge while a migration runs, and a list/board switch.
6. **Cost.** One optional `header.badge` key, additive in `ess-ui/1`; runtime template changes; no
   migration.
7. **Considered.** (a) Change nothing. (b) A `header.total_label` key and a `switch.style` key:
   new surface for what the renderer can derive. (c) Chosen: render from what the document holds,
   one decided key for the badge.

## Decisions

**Accept, redesigned.** The badge is built as decided (G-3 answer a). The total, live status,
switch and navigation are renderer fixes with no new key: the requester's label for the total is
derived from the section's title. No new format version.

## Source

`github:beyond10x/ess#723`
