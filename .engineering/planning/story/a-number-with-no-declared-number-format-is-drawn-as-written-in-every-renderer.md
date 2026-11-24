---
format: aep.planning-md/3
id: story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer
kind: story
status: active
title: A number with no declared number format is drawn as written in every renderer
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-per-value-channel-script-that-names-no-session-is-reported
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T00:39:12Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T00:39:14Z", actor: "human:timo", revision: 4}
---
## Need

A whole number shown by a field whose `as` is `text`, the default, is drawn differently by the two
renderers. The terminal draws it as written, `48213`. The generated React project draws it with
the locale's digit grouping, `48,213`. An identifier, an order number or a port number then reads
as a quantity in the browser and as itself in the terminal.

The difference is documented, and test files tolerate it. `ess-ui-test.md` says that "the terminal
prints `1840`, the browser groups it by its locale (`1,840`), and either spelling in the test
matches either on the screen" (`website/docs/reference/ess-ui-test.md:220-223`). The Playwright
emitter matches a number of four or more digits with or without grouping
(`crates/ui/ess-ui-test/src/playwright.rs:820-826`, `crate::numbers::pattern`). So a test file
gets one verdict today. What is wrong is what a reader sees: an identifier drawn as a quantity, in
one renderer only. This story removes the documented difference wherever the node declares no
number format.

Where each renderer draws a number:

- React: a field with no `as`, or `as: text`, reaches `formatValue(value, field.as, …)`
  (`crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl:200-201`). Every number whose format
  is not currency, percent, duration or bytes goes to `new Intl.NumberFormat().format`
  (`crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:805-820`, line 819).
- The same `formatValue`, called without a number format, draws:
  - the header title (`core.tsx.tmpl:712`) and the header badge (`:718`);
  - a badge (`primitives/badge.tsx.tmpl:32`) and a link (`primitives/link.tsx.tmpl:16`);
  - a text primitive (`primitives/text.tsx.tmpl:20`, whose schema default is `format: plain`);
  - a chart tick and its table fallback (`composites/chart.tsx.tmpl:26`, `:70`, `:94-96`).
- A metric goes through `formatValue` with its `format` (`composites/metric.tsx.tmpl:65`), whose
  schema default is `number` (`schemas/ui/ess-ui.schema.yaml:804`).
- The header's `total` count is grouped on its own (`core.tsx.tmpl:722`), and three tests pin that
  (`adversary_t3_page_header.rs:184-186`, `adversary_t3_page_header_pass2.rs:109`,
  `page_header.rs:80`).
- Terminal: `cell_value` sends `text` and every unnamed `as` to `display`
  (`crates/ui/ess-ui-tui/src/view.rs:1899`), which writes `number.to_string()`
  (`crates/ui/ess-ui-tui/src/expr.rs:249-253`). A metric does the same (`view.rs:2486-2500`).
- The `ess-ui/1` reference names `text` the default `as` (`website/docs/reference/ess-ui.md:1505`).

Minimal reproduction (fresh, not run here):

```yaml
- name: orders
  component: collection
  reads: {placeholder: probe.Orders, fixture: fixtures/orders.yaml}
  columns:
    - {field: number, label: Order}          # fixture row: {number: 48213}
```

The React cell reads `48,213` and the terminal cell `48213`. A test step
`expect: {text: "48213"}` passes in both, by the documented tolerance.

The requester's proposal, restated: `as: text` on an integer draws it as written, or an identity
field or a newtype of `Integer` is never grouped.

## Outcome

A number is drawn with the locale's grouping, in React only, where the node declares a number
format or its schema default is one:

- a field `as: number`;
- a metric, whose `format` defaults to `number`;
- a text primitive `format: number`;
- the header's `total` count, a count rather than a value of the node, which is kept as it is.

Everywhere else a number is drawn as written, in its shortest decimal form, in both renderers:

- a field with no `as` or `as: text`;
- the header title and badge, a badge and a link;
- a text primitive with no format or `format: plain`;
- a chart tick of a bare-name axis.

The terminal keeps drawing every number as written, as it keeps ISO order for a timestamp. The
result is the same before and after the first report's `g03-loader-fills-defaults` writes the
defaults into the document: that unit writes `format: number` for a metric and `format: plain` for
a text, and this rule already reads those values as the defaults.

`ess-ui-test`'s tolerance stays, because grouped numbers remain where a number format is declared
(metrics, `as: number`, the header total). Its sentence in `ess-ui-test.md:220-223` stays in
substance. Its example changes to say that the browser groups a number where the node declares a
number format.

## Acceptance

- Red first, in `crates/ui/ess-ui-react/tests/value_formats.rs`, with all new React cases in one
  test function so that the project compiles once. Each test runs in 60 s or less, the step in
  120 s or less. The cases:
  - `{field: number}` and `{field: number, as: text}` over `{number: 48213}` render `48213`. Both
    fail before the change.
  - A text primitive with `format: plain` and with no format renders `48213`.
  - The same cases in `crates/ui/ess-ui-tui/tests/value_formats.rs` pass before and after.
- `{field: number, as: number}` renders `new Intl.NumberFormat().format(48213)`, and the test
  asserts that this differs from `48213`, as `adversary_t3_page_header.rs:184-185` does (the node
  helper sets no locale). The terminal renders `48213`.
- A metric with no `format`, and the same metric with `format: number`, render the same grouped
  text in React. The pair shows the result does not depend on whether the loader has filled the
  default.
- A header `title_from` a field holding `48213`, a header badge `text: section.s.row.number`, a
  badge and a link show `48213` in React. The header `total` stays grouped, and its three pins do
  not move.
- A fraction keeps its digits: `0.1` and `1234.5` render as written under `as: text` in both
  renderers.
- The existing pins `crates/ui/ess-ui-react/tests/value_formats.rs:150` (`started: 1234`, no `as`)
  and `:442` (`talk: 73000`, no `as`) are re-pinned to the number as written. Their comments, "A
  number without `as: …` stays a number", are rewritten to say it is drawn as written.
- The `ess-ui/1` reference, regenerated from `schemas/ui/ess-ui.schema.yaml`, says in the note of
  `as` that a number under `text` is drawn as written and under `number` with the locale's grouping
  in React. `ess-ui-test.md:220-223` keeps its rule, with its example worded as above.
- `CHANGELOG.md` names the change under Changed, not `**Breaking**`, as the repository labels a
  change in what a renderer draws (0.93.0's metric `h:mm:ss`): "React draws a number in a field,
  header, badge, link, text or chart tick without grouping unless a number format is declared".

## Fit review

Tree `bcfc51555`, read-only. Second report, G3.

1. **Need.** An identifier stored as a whole number reads as the digits it is, in every renderer.
   Reproduction above. The requester's proposal is restated under Need.
2. **Class.** *Gap*: a documented difference between the renderers (`ess-ui-test.md:220-223`)
   that this story removes where the node declares no number format. An identifier reads as a
   quantity in one renderer. Test verdicts already agree, so this is not a defect.
3. **Already expressible?** No. `text`, `number` and every unnamed `as` reach the same
   `Intl.NumberFormat` call (`core.tsx.tmpl:819`).
4. **Fit.** No grid applies (`docs/design/language-grids.md:15`, `:63`). The rule reads the words
   the document already has: `as: text` and `format: plain` mean text, while `as: number` and
   `format: number`, written or by schema default, mean a number. No key is added. The requester's
   identity-or-newtype rule needs the model's types in the renderer. It would draw one integer two
   ways depending on `--model`, so it is not taken.
5. **Second adopter.** A support console shows ticket `100234` and a port `8080`, drawn as
   `100,234` and `8,080` in React today.
6. **Cost.** No format version. Generated React runtime bytes change, in `core.tsx` and the
   templates that call `formatValue`. No committed tree holds a generated runtime. Two pins move
   (`value_formats.rs:150`, `:442`). Metrics and header totals do not change.
7. **Considered.** See SL-D1. The terminal grouping as React does was also considered: it has no
   locale, and an identifier would still read as a quantity.

## Decisions

**Accept, redesigned.** No new key. What changed from the request: the rule follows the declared
or default format, not the model type.

- **SL-D1.** Which nodes keep grouping? Default A: where a number format is declared or is the
  schema default (metric), plus the header total. See `DECISIONS.md`.

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl` (`formatValue` 775-833; `total` 722 unchanged)
- cited: `crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl` (200-201)
- cited: `crates/ui/ess-ui-react/templates/runtime/composites/metric.tsx.tmpl` (65: passes `format`,
  else `number`)
- cited: `crates/ui/ess-ui-react/templates/runtime/primitives/text.tsx.tmpl` (20)
- cited: `crates/ui/ess-ui-react/tests/value_formats.rs` (new cases; re-pins 150, 442),
  `crates/ui/ess-ui-tui/tests/value_formats.rs`
- cited: `website/docs/reference/ess-ui-test.md` (220-223, example only)
- inferred: `schemas/ui/ess-ui.schema.yaml` (the `as` note), `website/docs/reference/ess-ui.md`
  (regenerated), `CHANGELOG.md`
- Gate: `cargo test -p ess-ui-react --test value_formats`, `--test page_header`,
  `--test adversary_t3_page_header`, `--test adversary_t3_page_header_pass2`;
  `cargo test -p ess-ui-tui --test value_formats`; `task fmt-check`. These run under the 60 s per
  test and 120 s per step bounds, and the 10-minute pool limit applies.
- Size: S, about 180 changed lines.

## Depends on

Lands after the first report's `g07-session-script`, which goes first in that group. Both edit
`schemas/ui/ess-ui.schema.yaml` and regenerate `ess-ui.md`, so sl03 rebases on it. It builds in the
first group beside `g04-step-trace`, with which it shares no file. It lands before
`sl05-object-one-rule` (second group) and `sl04-list-separator` (third), which edit the same
`formatValue`. It lands before `story:a-chart-names-its-axes-and-formats-its-ticks` (active). Before
or after `g03-loader-fills-defaults` gives the same result (acceptance above).

## Source

A downstream application, 2026-11-24, second report, G3.
