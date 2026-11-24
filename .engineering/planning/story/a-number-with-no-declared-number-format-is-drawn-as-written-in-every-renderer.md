---
format: aep.planning-md/3
id: story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer
kind: story
status: draft
title: A number with no declared number format is drawn as written in every renderer
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A whole number shown by a field whose `as` is `text`, the default, is drawn differently by the two
renderers. The terminal draws it as written, `48213`. The generated React project draws it with the
locale's digit grouping, `48,213`. An identifier, an order number or a port number then reads as a
quantity in the browser and as itself in the terminal.

- React: a field with no `as`, or `as: text`, falls to `formatValue(value, field.as, …)`
  (`crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl:200-201`), and every number with a
  format other than currency, percent, duration and bytes goes to `new Intl.NumberFormat().format`
  (`crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:805-820`, line 819).
- The same `formatValue` without a format draws the header title (`core.tsx.tmpl:712`), the header
  badge (`:718`), a badge (`primitives/badge.tsx.tmpl:32`), a link (`primitives/link.tsx.tmpl:16`),
  a chart tick and its table fallback (`composites/chart.tsx.tmpl:26`, `:70`, `:94-96`), and a
  metric with no `format` (`composites/metric.tsx.tmpl:65`).
- Terminal: `cell_value` sends `text` and every unnamed `as` to `display`
  (`crates/ui/ess-ui-tui/src/view.rs:1846-1900`, line 1899), which writes a number with
  `number.to_string()` (`crates/ui/ess-ui-tui/src/expr.rs:249-253`). A metric with no format
  does the same (`view.rs:2486-2500`).
- The reference names `text` the default `as` and says nothing about grouping
  (`website/docs/reference/ess-ui.md:1505`). It names the locale only for `time` and `timestamp`
  ("the locale's order in React, ISO order in the terminal").

The two renderers disagree on one document, so this is a defect. The precedent is
`story:a-declared-field-format-renders-or-is-refused`, which classed the two renderings of
`as: time` as a defect.

Minimal reproduction, written fresh. It was not run here, because the assessment was read-only
and nothing was built. The result follows from the lines above:

```yaml
- name: orders
  component: collection
  reads: {placeholder: probe.Orders, fixture: fixtures/orders.yaml}
  columns:
    - {field: number, label: Order}          # fixture row: {number: 48213}
```

The React cell reads `48,213` and the terminal cell reads `48213`. An `ess-ui-test/1` step
`expect: {at: …/orders/rows/<key>/number, text: "48213"}` passes in the terminal and fails in
Playwright.

The requester's proposal, restated: `as: text` on an integer draws it as written, or an identity
field or a newtype of `Integer` is never grouped.

## Outcome

A number is drawn as written, in its shortest decimal form, wherever its node declares no number
format. This holds for a field with no `as` or `as: text`, a header title and badge, a badge, a
link, a chart tick of a bare-name axis, and a metric with no `format`. It is drawn with the
locale's grouping only where the node declares `as: number` (a field) or `format: number` (a
metric, a text). That grouping is drawn by React only. The terminal keeps the number as written,
as it keeps ISO order for a timestamp.

## Acceptance

- Red first, in `crates/ui/ess-ui-react/tests/value_formats.rs`: a column `{field: number}` over a
  row `{number: 48213}` renders `48213`, and so does `{field: number, as: text}`. Both fail before
  the change, rendering `48,213`. The same cases in `crates/ui/ess-ui-tui/tests/value_formats.rs`
  pass before and after.
- `{field: number, as: number}` renders `48,213` in React under the `en-US` locale the test sets,
  and `48213` in the terminal.
- A metric with no `format` over the value `12345` renders `12345` in both renderers. With
  `format: number`, React renders `12,345`.
- A header `title_from` a field holding `48213`, a header badge `text: section.s.row.number`, and a
  bare-name chart axis show `48213` in React.
- A fraction keeps its digits: `0.1` and `1234.5` render `0.1` and `1234.5` in both renderers.
- The `ess-ui/1` reference, regenerated from `schemas/ui/ess-ui.schema.yaml`, says in the note of
  `as` that a number under `text` is drawn as written and under `number` with the locale's grouping
  in React. `CHANGELOG.md` names the change under a fix.

## Fit review

Tree `bcfc51555` (`integration/wave-2026-11-24`), read-only. The request came on 2026-11-24 from a
downstream application (second report, G3).

1. **Need.** An identifier stored as a whole number must read the same in every renderer, as the
   digits it is. Reproduction above. The requester's proposal is quoted under Need.
2. **Class.** *Defect*: the two renderers draw one document differently (lines above), the class
   `story:a-declared-field-format-renders-or-is-refused` gave the two `time` renderings.
3. **Already expressible?** No. No `as` makes React draw a number without grouping: `text`,
   `number` and every unnamed value reach the same `Intl.NumberFormat` call (`core.tsx.tmpl:819`).
4. **Fit.** No language grid applies (`docs/design/language-grids.md:15`, `:63`). The rule reuses
   the words the document already has: `as: text` means text, `as: number` and `format: number`
   mean a number. No key is added. The requester's "no grouping for an identity field or a newtype
   of `Integer`" needs the model's types in the renderer. It would draw one integer two ways
   depending on `--model`, so it is not taken.
5. **Second adopter.** A support console shows ticket `100234` and a port `8080`. Both read as
   `100,234` and `8,080` in React today.
6. **Cost.** No format version: the reference never promised grouping, and the terminal already
   draws this way. Generated React runtime bytes change in `core.tsx` for every project. Committed
   example trees carry no generated runtime (`git ls-files` holds no `core.tsx`). A document that
   wants grouping writes `as: number` or `format: number`. That is a visible change for metrics and
   chart ticks of large numbers in React, and it is the price of parity (decision SL-D1).
7. **Considered.** (a) Change nothing: the renderers keep disagreeing. (b) The terminal groups as
   React does: the terminal has no locale, and an identifier still reads as a quantity. (c) The
   requester's identity-or-newtype rule: it depends on the model. (d) Chosen: as written unless a
   number format is declared.

## Decisions

**Defect fix, accept redesigned.** No new key. What changed from the request: the rule follows
the declared `as`, not the model type.

- **SL-D1. Where does grouping stay?** (A) Only where `as: number` or `format: number` is
  declared. Every other number is drawn as written in both renderers. (B) The same for fields and
  text, but a metric and a chart tick keep React's grouping by default. The terminal stays as
  written, so those two still disagree. **Recommended: A.**

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl` (`formatValue` 775-833)
- cited: `crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl` (200-201)
- cited: `crates/ui/ess-ui-react/templates/runtime/composites/metric.tsx.tmpl` (65)
- cited: `crates/ui/ess-ui-react/tests/value_formats.rs`, `crates/ui/ess-ui-tui/tests/value_formats.rs`
- inferred: `schemas/ui/ess-ui.schema.yaml` (the `as` note), `website/docs/reference/ess-ui.md`
  (regenerated), `CHANGELOG.md`
- Size: S, about 150 changed lines.

## Depends on

Nothing. It lands before `story:a-chart-names-its-axes-and-formats-its-ticks` (active), which then
draws an axis `as: number` with grouping and a bare-name axis as written. Both edit `formatValue`,
so they land one after the other. It lands before `sl05-object-one-rule`, `sl04-list-separator`
and `sl06-time-zone`, which edit the same function.

## Source

A downstream application, 2026-11-24, second report, G3.
