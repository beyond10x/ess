---
format: aep.planning-md/3
id: story:a-chart-s-rows-are-addressable-in-both-renderers-and-a-test-asserts-the-order-a-chart-draws-them-in-with
kind: story
status: active
title: A chart's rows are addressable in both renderers, and a test asserts the order a chart draws them in with `expect.rows`
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-chart-draws-its-rows-in-the-order-its-sort-declares-and-a-live-insert-keeps-that-order
- depends_on: story:the-terminal-addresses-every-node-the-browser-addresses
- depends_on: story:a-per-value-channel-script-that-names-no-session-is-reported
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T00:39:09Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T00:39:11Z", actor: "human:timo", revision: 3}
---
## Need

After `g06-chart-sort` a chart keeps a declared order under live inserts, but no test can assert it.
`expect: {at, rows: […]}` holds only at a section's collection: the runner refuses it elsewhere,
"is not a collection" (`crates/ui/ess-ui-test/src/runner.rs:1251-1262`, `collection_section`). In
the terminal, rows are found by regions keyed `<collection>/rows/<key>`, ordered by `(y, x)`
(`crates/ui/ess-ui-test/src/screen.rs:118-134`); a chart registers none. In the browser the spec
counts `.ui-row` elements under the node and reads each one's `data-ui-path` as `<at>/rows/<key>`
(`crates/ui/ess-ui-test/src/playwright.rs:811`, `:839-846`); the chart draws `ui-chart-x-slot` and
`ui-bar-group` elements with neither (`templates/runtime/composites/chart.tsx.tmpl:52-56`, `:63`).
Report item 6, the requester's "an `ess-ui-test/1` step can assert the order of the x axis".

## Outcome

A chart's rows are keyed as a collection's are (the read's `key`, else `live.match`, else `id`) and
addressed as `<chart section>/rows/<key>` in both renderers: the terminal registers a one-cell region
per drawn point (sparkline cell or bar), the generated React chart marks each x slot and bar group
with `ui-row` and its `data-ui-path`. `expect.rows` (count and keys in order) holds at a chart
section in both renderers.

## Acceptance

- First: in the reproduction of `g06-chart-sort` with `sort: {by: seq}`, a case in
  `crates/ui/ess-ui-test/tests/steps.rs` expects `{at: <chart section>, rows: [1, 2, 3, 4, 5]}` after
  both inserts. Today the runner refuses it ("is not a collection"); after, it holds, and it fails
  with `rows: [4, 5, 1, 2, 3]` and with the order before the inserts.
- The emitted Playwright spec for the same test checks five `.ui-row` elements under the chart with
  `data-ui-path` `<chart>/rows/1` … `/5` in order; a case in `crates/ui/ess-ui-test/tests/playwright.rs`
  pins the emitted lines, and a case in `crates/ui/ess-ui-react/tests/chart_legend.rs` (one compile)
  finds the attributes on the generated chart for `chart: line` and `chart: bar`.
- The `no_charts` table fallback is addressed as a collection already is; a case pins one key list
  there.
- `rows: <n>` (a count) at a chart holds in both renderers.
- A chart's drawing does not move: the terminal's regions carry no text and change no cell; the
  added class has no style of its own (`styles.css.tmpl` has no bare `.ui-row` rule). Every
  committed example generates the same visible output; the markup of every generated chart gains
  the two attributes.
- `ess-ui-test.md` lists a chart section as a target of `rows` and documents how its rows are keyed;
  `CHANGELOG.md` names it under `### Added`.

## Fit review

Tree `bcfc51555`, read-only. Covers report item 6 with `g06-chart-sort`; the seven questions are
answered in `g06-chart-sort` § Fit review. In short: no new word; the existing `rows` check gains a
target; the browser markup change is additive (no document's behaviour changes, only the chart's
DOM gains attributes).

## Decisions

- **P1-D13 (was r1's D4). How does a test assert a chart's order?** (a) `expect.rows` at a chart,
  rows addressed as `<chart>/rows/<key>` in both renderers, one-cell terminal regions per point.
  (b) A new `expect` key listing the axis values. (c) No step, Rust tests only.
  **Recommended: (a).** It adds no word.

## Scope

- cited: `crates/ui/ess-ui-test/src/runner.rs` (`collection_section` 1251-1262, `shown_rows` 938-947)
- cited: `crates/ui/ess-ui-test/src/playwright.rs` (`emit_check` 811-846),
  `crates/ui/ess-ui-test/src/parity.rs` (a chart's rows are rendered: no refusal),
  `crates/ui/ess-ui-test/src/target.rs` (a row under a chart section resolves)
- cited: `crates/ui/ess-ui-react/templates/runtime/composites/chart.tsx.tmpl` (each x slot and bar
  group carries `ui-row` and `data-ui-path`, 52-56 and 63)
- cited: `crates/ui/ess-ui-tui/src/view.rs` (`chart_lines` 2560-2665: a region per drawn point)
- cited: tests `crates/ui/ess-ui-test/tests/steps.rs`, `crates/ui/ess-ui-test/tests/playwright.rs`,
  `crates/ui/ess-ui-react/tests/chart_legend.rs`
- cited: `website/docs/reference/ess-ui-test.md`, `CHANGELOG.md`
- Size: M (about 350 changed lines).

## Depends on

- `g06-chart-sort`.
- Lands after `story:the-terminal-addresses-every-node-the-browser-addresses` (active; it changes
  which regions the terminal records, the same `regions` this story adds to).
- Lands after `g07-session-script` (shares `runner.rs`, `playwright.rs`, `parity.rs`).
