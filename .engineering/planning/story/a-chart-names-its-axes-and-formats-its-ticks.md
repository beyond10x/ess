---
format: aep.planning-md/3
id: story:a-chart-names-its-axes-and-formats-its-ticks
kind: story
status: active
title: 'A chart''s axes take the field shape: titled, ticks in their format'
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-declared-field-format-renders-or-is-refused
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/composites/chart.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/tests/chart_axes.rs
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: cited
  path: crates/ui/ess-ui/src/model.rs
- confidence: cited
  path: schemas/ui/ess-ui.schema.yaml
- confidence: inferred
  path: website/docs/reference/ess-ui.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:37:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:37:51Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A chart's x axis prints each row's raw value, so a time series shows ISO timestamps
(`2026-01-02T00:00:00Z`) under every bar or point, and neither axis has a title. The chart takes
`x: <name>` and `series: [<name>]` only (`schemas/ui/ess-ui.schema.yaml` `chart.fields`), and the
React chart formats the x value with no format
(`crates/ui/ess-ui-react/templates/runtime/composites/chart.tsx.tmpl:25-31`:
`formatValue(display(row, x))`); a series is named by its field name read as words.

Minimal reproduction (ess 0.92.0, `ess generate ui --target react`), fixture rows
`{at: "2026-01-02T00:00:00Z", count: 3}`:

```yaml
- {name: chart, component: chart, chart: line, reads: {view: example.order.Events}, x: at, series: [count]}
```

The generated `chart.tsx` writes `formatValue(display(row, x))` for each tick, with no format and
no axis title.

## Acceptance

- `x` and each `series` entry take a field, as a column does: a bare name (today's form, unchanged
  output) or `{field, label, as, unit}`; `ess ui check` refuses any other field key there by path.
- With `x: {field: at, label: Day, as: date}`, the React chart and its table fallback show each
  tick as a date and the axis titled `Day`; a series `{field: count, label: Orders}` is titled
  `Orders` in the legend and the table header (`crates/ui/ess-ui-react/tests/chart_axes.rs`).
- The terminal's chart (and its table fallback) shows the same tick text and titles.
- A document whose chart uses bare names generates the same bytes.

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/composites/chart.tsx.tmpl`
- cited: `crates/ui/ess-ui/src/model.rs`
- cited: `schemas/ui/ess-ui.schema.yaml`
- inferred: `crates/ui/ess-ui-react/src/emit.rs`
- inferred: `crates/ui/ess-ui-tui/src/view.rs`
- inferred: `crates/ui/ess-ui-react/tests/chart_axes.rs`
- inferred: `website/docs/reference/ess-ui.md`
- confidence: medium

## Fit review

Tree `995e29579d`, binary ess 0.92.0. Covers #723 item G5.

1. **Need.** A reader of a chart sees each tick in the form of its value (a date, a duration) and
   knows what each axis measures. Reproduction above. The requester's words, theirs: "chart x ticks
   print raw ISO timestamps; no axis titles".
2. **Class.** *Gap*: an axis has no place to say its format or title.
3. **Already expressible?** No: `x` is a `name` (`chart.fields.x`), and a view cannot publish a
   formatted string without moving a display decision into the model.
4. **Fit.** The language grids type the specification language and do not apply
   (`docs/design/language-grids.md:15`, `:63`). The sibling that already says "this value, with this
   caption, in this form" is `Field` (`field`, `label`, `as`, `unit`,
   `schemas/ui/ess-ui.schema.yaml` `Field`), used by columns and record fields. Reusing it on the
   axes adds no word; the bare-name shorthand stays, as it does for columns (`shorthands.index`,
   Field accepts a string).
5. **Second adopter.** A monitoring dashboard plotting latency per hour: ticks as times, the y axis
   titled in milliseconds.
6. **Cost.** Additive in `ess-ui/1`: `x` and `series` widen to the Field one-of; no migration; the
   chart template and the terminal chart change.
7. **Considered.** (a) Change nothing. (b) New keys `x_label`, `x_format`, `series_labels`: a second
   spelling of what `Field` already says. (c) Chosen: the Field shape on the axes.

## Decisions

**Accept, redesigned.** The axis takes the existing Field shape instead of new chart keys; tick
formats are the `as` words of `story:a-declared-field-format-renders-or-is-refused`, which this
story depends on for `timestamp` and the duration units. No new format version.

## Source

`github:beyond10x/ess#723`
