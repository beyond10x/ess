---
format: aep.planning-md/3
id: story:a-chart-draws-its-rows-in-the-order-its-sort-declares-and-a-live-insert-keeps-that-order
kind: story
status: active
title: A chart draws its rows in the order its sort declares, and a live insert keeps that order
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-chart-names-its-axes-and-formats-its-ticks
- depends_on: story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T00:39:06Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T00:39:08Z", actor: "human:timo", revision: 4}
---
## Need

A chart draws its rows in the order its read holds them, and nothing in an `ess-ui/1` document can
change that order. A live `insert_or_patch` or `insert_top` puts a new row first, so a line chart
fed by a channel shows its newest points at the left of the x axis, before the rows the view
returned in order. A collection does not have this problem, because it sorts after the live
effects. A chart has no `sort` key.

- The React chart places point `at` at `at * step` and draws `x` only as a label
  (`crates/ui/ess-ui-react/templates/runtime/composites/chart.tsx.tmpl:38`, `:47`, `:52-56`).
  Bars are drawn in row order too (`:63`).
- The terminal chart draws its sparkline in row order, and its footer shows the first and last
  row's `x` (`crates/ui/ess-ui-tui/src/view.rs:2629-2662`).
- A new row goes first under `insert_or_patch`, in React
  (`crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl:414-416`, `return [payload, ...rows]`)
  and in the terminal (`crates/ui/ess-ui-tui/src/app.rs:4970-4972`, `rows.insert(0, …)`).
- `Chart` has `reads`, `chart`, `x` and `series` only (`crates/ui/ess-ui/src/model.rs:2044-2057`,
  `schemas/ui/ess-ui.schema.yaml:812-822`). `deny_unknown_fields` refuses any other key.
- A collection's client sort runs after the live rows
  (`crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl:118-122`).

Minimal reproduction. This is written fresh, not taken from the requester, and was not run here
because the assessment was read-only with no build. The result follows from the lines above. A
section reads a placeholder whose fixture holds rows `seq` 1, 2, 3, and a channel script inserts
`seq` 4 and then `seq` 5:

```yaml
- name: trend
  component: chart
  chart: line
  reads: {placeholder: probe.Points, fixture: fixtures/points.yaml, key: seq}
  x: seq
  series: [value]
  live: {channel: points, effect: insert_or_patch}
```

After both inserts, the React x axis reads 5, 4, 1, 2, 3, and the terminal footer reads
`line · seq: 5 … 3`.

The requester proposed, in their words: "a chart orders its rows by a declared `sort` (default:
`x` ascending for line and bar) in every renderer, and live inserts keep that order; an
`ess-ui-test/1` step can assert the order of the x axis."

## Outcome

A chart that declares `sort: {by, dir}`, the collection's key and record, draws its rows in that
order in React, in the terminal and in the table fallback, and keeps that order after every live
effect. A chart that declares no sort draws exactly what it draws today. The step that asserts a
chart's drawn order in a test is the second half of this split, `g06b-chart-rows`.

## Acceptance

- First: `chart` accepts `sort: {by: <name>, dir: asc|desc}`, with `dir` defaulting to `asc`, in
  `schemas/ui/ess-ui.schema.yaml` and `Chart` in `crates/ui/ess-ui/src/model.rs`; a loader case
  with `sort: {by: seq}` on a chart is refused today (`deny_unknown_fields`, `model.rs:2045-2046`)
  and loads after. `ess ui load` refuses `sort.allowed` and `sort.mode` on a chart, naming the node
  path (`…/sort/allowed`, `…/sort/mode`), in a `check_chart_sort` beside `check_unit`
  (`crates/ui/ess-ui/src/path.rs:501`).
- In the reproduction with `sort: {by: seq}`, the generated React chart draws its x axis as
  1, 2, 3, 4, 5 after both inserts, for `chart: line`, `chart: bar` and the `no_charts` table
  fallback. These are cases in `crates/ui/ess-ui-react/tests/chart_legend.rs` that run the
  generated runtime, as `read_filter.rs` does, in one test function (one compile); each test 60 s
  or less, the test step 120 s or less.
- The terminal chart draws the same order: its sparkline in sort order, and its footer
  `line · seq: 1 … 5`. Its table fallback draws its rows in the same order.
- Sorting happens after the live effects and `reads.filter`. It is stable, so equal keys keep
  their order. The chart and the collection use one comparison, moved from
  `collection.tsx.tmpl` (`compareValues`, `:575`, `numeric`, `:589-597`) into a new template
  `templates/runtime/compare.ts`. A chart compares with `asNumber` false: a number by value (10
  after 9), a text by its text; a collection keeps its numeric-column rule.
- With `dir: desc`, the order is reversed, and a test covers this in both renderers.
- A chart without `sort` keeps its output. Every committed example and test document generates
  the same bytes, including `examples/partner-portal/ui.yaml` and the chart tests
  `chart_legend.rs` and `partner_portal.rs`.
- With `--model`, `ess ui check` reports an error at a chart's `sort.by` when the field is not a
  row field of the view.
- The `ess-ui/1` reference, generated by `ess ui docs`, lists the chart's `sort`. Its `Live`
  section says where `insert_or_patch` and `insert_top` put a new row (first) and that a declared
  sort orders the rows after the effect. `CHANGELOG.md` names the key under `### Added`.

## Fit review

Tree `fad42d26f` (workspace 0.98.0 and 59 commits). The request was received on 2026-11-24 from a
downstream application. It is report item 6 of that application's collected report of
2026-11-24 (17 items against 0.98.0), and the story covers report item 6 and no other item.
Re-read at `bcfc51555` (`integration/wave-2026-11-24`): no file under `crates/ui/` or
`schemas/ui/` changed since `fad42d26f`, so every citation below holds. This fit review covers
both halves of the split: this story (the key) and `g06b-chart-rows` (the test step).

1. **Need.** A chart of rows that live effects maintain cannot keep the domain's order on its
   axis. A sequence or time series reads out of order as soon as one row is inserted. See the
   reproduction above. The requester's proposal is quoted separately under Need.
2. **Class.** This is a *gap*. No key on a chart orders its rows, and `Chart` refuses every key
   it does not declare (`model.rs:2045-2046`). It is not a *defect*: the reference does not say
   where `insert_or_patch` puts a row (`website/docs/reference/ess-ui.md:2303-2337`), and the
   chart's documentation promises no order (`ess-ui.md:1263-1287`). Both renderers agree that a
   new row goes first.
3. **Already expressible?** No. A model view's `order_by:` orders what the read returns
   (`examples/billing/domains/invoice.yaml:440-441`), but neither renderer places a live insert
   by it. The UI binding carries no view order (`crates/ui/ess-ui/src/binding.rs:26-39`,
   `:177-195`). `reads.filter` selects rows but does not order them (`model.rs:2907-2908`).
   `refetch` instead of an insert reads the whole view again on every event, which changes the
   behaviour and does not fix the order of the drawn rows.
4. **Fit.** The language grids cover the specification language, not `ess-ui/1`
   (`docs/design/language-grids.md:15`, `:61-63`), so no cell applies.
   - The concept "order these rows by a field" is already spelled `sort: {by, dir}` on the
     collection (`schemas/ui/ess-ui.schema.yaml:610`, `model.rs:1397-1410`). The chart takes the
     same key and the same record.
   - `allowed` is a user's choice of sort, and `mode: server` asks the view to sort through
     `state.sort` (`collection.tsx.tmpl:175-179`). A chart draws no sort control, so both are
     refused on a chart, by name.
   - It composes with `reads.filter` (sorting runs after the filter), with every live effect
     (sorting runs after the effect), and with `degrades: {no_charts: table}` (the fallback draws
     the same order).
   - Siblings:
     - The collection already sorts.
     - `record` and `metric` show one row or one aggregate. Which row is shown is a separate
       question, already answered in the model by `order_by` with `first`. They get no `sort`.
     - `board` places rows by its layout.
   - Every renderer that draws a chart handles the key: React (`emit.rs:1742-1758` and
     `chart.tsx.tmpl`) and the terminal (`view.rs:2560-2665`). No other renderer draws one.
     `ess-ui-check` reads the chart's kind only (`crates/ui/ess-ui-check/src/enums.rs:114-116`).
5. **Second adopter.** An operations dashboard plots queue depth per minute from a channel that
   inserts each new minute. It needs `sort: {by: minute}`. A sales board draws a bar per stage
   in the pipeline's order, as the example's `pipeline` chart does
   (`examples/partner-portal/ui.yaml:207-213`, `x: stage`). It needs no sort, and an implicit
   x-ascending default would reorder it alphabetically.
6. **Cost.**
   - One optional key in `ess-ui/1`. Without it, a chart's meaning and generated bytes stay the
     same, so no new format version is needed under "a new format version is required when
     meaning … changes" (`AGENTS.md` § Determinism and formats). There is precedent: the chart's
     axes widen within `ess-ui/1` in `story:a-chart-names-its-axes-and-formats-its-ticks`.
   - An older `ess` refuses a document that uses the key, as it refuses every key `ess-ui/1`
     added later.
   - No migration and no new diagnostic code.
   - One extended check subject.
   - `ess-ui-test/1` gains a target for an existing check.
7. **Considered.**
   - (a) Change nothing. This was rejected: no idiom exists (question 3).
   - (b) `order` on `Reads`, beside `filter`, for every reader. This was rejected: it is a second
     spelling next to the collection's `sort`, and the red flag "names a concept ESS already
     spells differently elsewhere" applies.
   - (c) Live inserts take the place the model view's `order_by:` gives them, in every reader.
     This was deferred: it needs the binding to carry each view's order, it works only with
     `--model` (not on a placeholder), and it changes what existing documents show under live
     effects, which is a meaning change.
   - (d) The requester's default of x ascending for line and bar. This was refused: it changes
     the output of every existing chart whose view orders otherwise (for example `issued_at
     desc`), which would need `ess-ui/2`, and it is wrong for categorical bars (question 5).
   - (e) Chosen: the collection's `sort` on the chart, absent by default. The requester's
     "declared `sort`" and "every renderer" are kept. Their default is changed to "rows as
     read". Their test step is answered by the existing `rows` check rather than a new key.

## Decisions

**Accept, redesigned.** The chart takes the collection's `sort: {by, dir}`. When it is absent,
the chart draws rows as read, so existing documents are unchanged. No new `ess-ui/1` version.
What changed from the request: there is no x-ascending default, and order is asserted with the
existing `expect.rows` (in `g06b-chart-rows`). Open questions, each with the recommended answer:

- **D1. Where does the order live?** (a) On the chart, as `sort`, the collection's key. (b) On
  `Reads`, beside `filter`, for every reader. (c) In the model view's `order_by:`, which live
  inserts follow in every reader. **Recommended: (a).** (b) duplicates the collection's
  spelling. (c) changes existing output and works only with `--model`, so it gets its own story
  if a second need appears.
- **D2. Which parts of the sort record does a chart take?** (a) `by` and `dir` only, with
  `allowed` and `mode` refused by name. (b) The whole record, with `mode: server` sorting through
  the view. **Recommended: (a).** A chart has no control a user could sort with.
- **D3. What is the default?** (a) No sort, meaning rows as read and as the live effects leave
  them, which is today's behaviour. (b) `x` ascending for line and bar, as requested.
  **Recommended: (a).** (b) changes the meaning of existing documents (`ess-ui/2`) and reorders
  categorical bars.
- D4 (how a test asserts the order) moved to `g06b-chart-rows`.

## Scope

- cited: `schemas/ui/ess-ui.schema.yaml` (`chart.fields`, lines 812-822)
- cited: `crates/ui/ess-ui/src/model.rs` (`Chart` 2044-2057, `Sort` 1397-1410)
- cited: `crates/ui/ess-ui/src/path.rs` (a `check_chart_sort` beside `check_unit`, 501)
- cited: `crates/ui/ess-ui-react/src/emit.rs` (`Composite::Chart` 1742-1758)
- cited: `crates/ui/ess-ui-react/templates/runtime/composites/chart.tsx.tmpl`
- cited: `crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl` (its
  comparison, `compareValues` at 575 and its use at 118-122, moves out)
- inferred: `crates/ui/ess-ui-react/templates/runtime/compare.ts` (new) and its entry in
  `crates/ui/ess-ui-react/src/assets.rs`
- cited: `crates/ui/ess-ui-tui/src/view.rs` (`chart_lines` 2560-2665)
- inferred: `crates/ui/ess-ui-check/src/model.rs` (the `row_fields` check for `sort.by`)
- cited: tests `crates/ui/ess-ui-react/tests/chart_legend.rs` (cases, one compile), a terminal
  case in an existing `crates/ui/ess-ui-tui/tests/` binary, a loader case in `crates/ui/ess-ui/tests/`
- inferred: `website/docs/reference/ess-ui.md` (regenerated), `CHANGELOG.md`
- Not touched: `core.tsx.tmpl` (other groups' value-format stories edit it).
- Size: M (about 400 changed lines).

## Depends on

- Lands after `story:a-chart-names-its-axes-and-formats-its-ticks` (active; same `Chart`,
  `chart.fields`, `chart.tsx.tmpl`, `view.rs` `chart_lines`).
- Lands after the other group's number-format story (`sl03-number-as-written`): chart ticks call
  `formatValue`.
- `g06b-chart-rows` depends on this story.

## Source

A downstream application, 2026-11-24 (request 1 of 2).
