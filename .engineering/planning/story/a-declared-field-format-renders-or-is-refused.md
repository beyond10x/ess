---
format: aep.planning-md/3
id: story:a-declared-field-format-renders-or-is-refused
kind: story
status: draft
title: 'Every declared value format renders, or ess ui check refuses it (minutes and hours, metric duration, timestamp, percent scale, as: metric)'
summary: '`as: metric` renders as text; durations lack minutes and hours; percent scale is undocumented (workbench report items 8, 41)'
tags:
- backlog
- external-request
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/composites/metric.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/tests/value_formats.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: inferred
  path: crates/ui/ess-ui-tui/tests/value_formats.rs
- confidence: cited
  path: crates/ui/ess-ui/src/model.rs
- confidence: cited
  path: schemas/ui/ess-ui.schema.yaml
- confidence: inferred
  path: website/docs/reference/ess-ui.md
revision: 6
---
## Need

A value the document types with a format is not shown in that form everywhere a value is shown,
and nothing refuses the format first. On ess 0.92.0:

- A record field `as: metric` renders as plain text: the React `FieldValue` switch has no `metric`
  case (`crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl:139-187`), and `ess ui check`
  reports nothing (workbench report item 8, example `ui08`).
- A duration cannot be counted in minutes or hours. `unit: m` is refused:
  `` `m` is no duration unit: `unit` is `ms` or `s` `` (`crates/ui/ess-ui/src/model.rs:2478`;
  workbench item 41; #723 D1, G1, whose seconds half shipped in 0.87.0).
- A metric `format: duration` still prints the raw number with an `s` suffix:
  `formatValue` returns `` `${value}s` `` (`crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:577-578`),
  while a field `as: duration` shows `h:mm:ss` since 0.87.0.
- `format: percent` goes to `Intl` percent style and expects a fraction, so 0.5 shows 50%; no page
  says so (workbench item 41, example `ui41`).
- `as: time` shows a full date and time in React (`toLocaleString`, `core.tsx.tmpl:565`) and the
  time of day in the terminal (`crates/ui/ess-ui-tui/src/view.rs:1603-1608`), so the two renderers
  disagree; and no `as:` word shows an instant as date and time (#723 G3, B6):
  `` `datetime` is not a value of `as` ``.

Minimal reproduction (`ess ui check`, ess 0.92.0), a collection column of
`example.order.Orders`:

```yaml
columns:
  - {field: placed_at, as: duration, unit: m}   # error document_loads …: `m` is no duration unit
  - {field: placed_at, as: datetime}            # error enum_values …: `datetime` is not a value of `as`
```

## Acceptance

- `ess ui check` refuses, at the field's path, every `as:` a record field, a collection column or
  a form field cannot render; every other `as:` the schema admits there renders in its form in
  React and in the terminal (`crates/ui/ess-ui-react/tests/value_formats.rs`,
  `crates/ui/ess-ui-tui/tests/value_formats.rs`).
- `unit:` on `as: duration` and on a metric `format: duration` takes `ms`, `s`, `m` or `h`:
  `{field: d, as: duration, unit: m}` with `90` shows `1:30:00`, and a metric `format: duration`
  with `73` shows `0:01:13`, in both renderers. A metric's `unit` defaults to `s`, the unit its
  `73s` text meant until now; a field's stays `ms`.
- `as: time` shows the time of day in both renderers; the new `as: timestamp` shows date and time
  (`2026-01-02T03:04:05Z` shows a date and `03:04`, the locale's order in React, ISO order in the
  terminal).
- A field or metric `percent` declares its scale: `scale: fraction` (default, today's behaviour,
  0.5 shows 50%) or `scale: hundredths` (50 shows 50%); the reference states both, and a test pins
  each.
- `website/docs/reference/ess-ui.md` is a fresh render of the schema.

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl`
- cited: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl`
- cited: `crates/ui/ess-ui-react/templates/runtime/composites/metric.tsx.tmpl`
- cited: `crates/ui/ess-ui-tui/src/view.rs`
- cited: `crates/ui/ess-ui/src/model.rs`
- cited: `schemas/ui/ess-ui.schema.yaml`
- inferred: `crates/ui/ess-ui-check/src/rules.rs`
- inferred: `crates/ui/ess-ui-react/tests/value_formats.rs`
- inferred: `crates/ui/ess-ui-tui/tests/value_formats.rs`
- inferred: `website/docs/reference/ess-ui.md`
- confidence: high

## Fit review

Tree `995e29579d`, binary ess 0.92.0. Covers #723 items D1/G1 (minutes, hours and the metric
half) and G3/B6, and workbench items 8 and 41.

1. **Need.** A value typed as a duration, a percentage, a time of day or an instant is shown in
   the form its type says, in both renderers, or the document is refused before it renders.
   Reproductions above and in the triage's `repro/run.txt`. The requesters' proposals, theirs:
   "minutes and hours", "a date-and-time form", "a documented percent scale".
2. **Class.** `as: metric` as text, the metric duration and the two `time` renderings: *defect*
   (the schema names the format, `schemas/ui/ess-ui.schema.yaml:423`, `:270`, and the renderers
   ignore it or disagree). Minutes and hours, the instant form and the percent scale: *gap*.
3. **Already expressible?** No. `unit` takes `ms | s` only (`model.rs:2478`); `as:` has no instant
   word (run above); a percent value in hundredths has no spelling.
4. **Fit.** No grid of `docs/design/language-grids.md` applies (grids R and O type the
   specification language, `language-grids.md:15`, `:63`); the sibling set here is every place a
   value is shown: a field, a column, a metric. Each gets the same words: `unit` is one key on a
   field and a metric; `timestamp` is the schema's own primitive name for an instant
   (`schemas/ui/ess-ui.schema.yaml` `type_rule.primitives.timestamp`), beside `date` and `time`;
   `scale` sits on whatever shows `percent`. Both renderers draw each, or check refuses.
5. **Second adopter.** A support console showing handling times in minutes, call starts as date and
   time, and an SLA as a percentage the backend stores in hundredths.
6. **Cost.** Additive in `ess-ui/1`: two `unit` values, one `as` value, one optional `scale` key;
   no migration; a document without them generates the same bytes.
7. **Considered.** (a) Change nothing: the reader sees `90s` and `0.5` misread. (b) A free format
   string (`format: "HH:mm"`): a second spelling per locale, untestable across renderers. (c)
   Chosen: close the existing vocabularies (`unit`, `as`, `percent`) over the missing values.

## Decisions

**Accept, redesigned.** Minutes and hours extend `unit`, also on the metric; the instant form is
`as: timestamp`, the schema's existing word, not `datetime`; the percent scale is a declared
`scale` with today's behaviour as its default. `as: metric` on a field either renders as the metric
composite does or is refused by check, whichever the implementor finds the smaller change, with
the reason in the commit. No new format version: additive within `ess-ui/1`
(`docs/design/ui-style-tokens.md` § "Format and version" states the rule).

## Source

`github:beyond10x/ess#723`; `.engineering/repro/workbench-2026-11-09/report.md` items 8 and 41.
