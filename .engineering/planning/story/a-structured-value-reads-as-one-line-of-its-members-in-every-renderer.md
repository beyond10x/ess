---
format: aep.planning-md/3
id: story:a-structured-value-reads-as-one-line-of-its-members-in-every-renderer
kind: story
status: active
title: A structured value reads as one line of its members in every renderer
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T00:39:15Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T00:39:17Z", actor: "human:timo", revision: 4}
---
## Need

The two renderers draw a field whose value is an object in two different ways. The generated React
project writes the object's JSON, `{"kind":"pickup","name":"north"}`. The terminal writes its
members as `kind: pickup, name: north`. Neither renderer reads `as: map`.

- React: `FieldValue` has no `map` case, so the field falls to `formatValue`
  (`crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl:200-201`). That returns
  `JSON.stringify(value)` for an object that is not money (`core.tsx.tmpl:825-831`). An object is
  money there only when `amount` is a number (`:827`).
- Terminal: `cell_value` has no `map` case, so the field falls to `display`
  (`crates/ui/ess-ui-tui/src/view.rs:1899`). `display` joins `key: value` pairs with `, `, and it
  treats an object as money whenever it holds both `amount` and `currency`
  (`crates/ui/ess-ui-tui/src/expr.rs:256-265`).
- `display` is more than drawing. It is the terminal's value-to-text function for comparisons and
  requests:
  - fixture search (`crates/ui/ess-ui-tui/src/data.rs:522-524`, `display(row)…contains`);
  - HTTP query values (`crates/ui/ess-ui-tui/src/http.rs:392`, `:701`);
  - session matching (`crates/ui/ess-ui-tui/src/app.rs:2352`).
- The reference lists `map` among the values of `as` and does not say how it is drawn
  (`website/docs/reference/ess-ui.md:1505`).
- Test verdicts disagree too. A `text: "kind: pickup"` check passes in the terminal and fails in
  the browser, against "one verdict" (`website/docs/reference/ess-ui-test.md:412-417`).

That is a defect. This story fixes the object half of the second report's G5. Reading a tagged
union by its variant is `sl05-union-variant`.

Minimal reproduction (fresh, not run here):

```yaml
columns:
  - {field: place, as: map}        # fixture row: {place: {kind: pickup, name: north}}
```

React shows `{"kind":"pickup","name":"north"}`. The terminal shows `kind: pickup, name: north`.

The requester's proposal, restated: a union reads as its variant and its fields
(`pickup north`), or `labels` per variant.

## Outcome

A field whose value is an object, under `as: map` or under `as: text` (the default), reads as one
line of `key: value` pairs joined with `, `. A nested object is drawn the same way inside braces,
`place: {kind: pickup, name: north}`. A list member is joined with `, ` inside brackets.

Keys come in one order in both renderers: integer-like keys first, in ascending order, then the
other keys in the order the answer holds them. This is the order a JavaScript object keeps, so the
terminal adopts it.

Money is an object holding `currency` and an `amount` that is a number or a decimal text. It is
drawn as money in both renderers, by the money rule each renderer has today. React formats a
decimal text amount from the text, without converting it to a binary64. `as: json` keeps showing
the value as data.

The terminal draws through a new drawing function in `expr.rs`, used by `cell_value`'s default arm
(`view.rs:1899`) and the metric. `display` is unchanged, so search results, query bytes and session
matching keep their behaviour.

Not covered: React draws money with `Intl` currency style (`€12.50`), and the terminal draws
`12.50 EUR`. That existing difference stays, and it is named here so that it is not mistaken for
part of this fix.

## Acceptance

- Red first, in `crates/ui/ess-ui-react/tests/value_formats.rs`, with all new React cases in one
  test function so that the project compiles once. Each test runs in 60 s or less, the step in
  120 s or less.
  - `{field: place, as: map}` over `{place: {kind: pickup, name: north}}` renders
    `kind: pickup, name: north`. It fails before the change, rendering the JSON.
  - The same case passes in `crates/ui/ess-ui-tui/tests/value_formats.rs`.
- A nested value `{a: {b: 1}, c: [x, y]}` renders `a: {b: 1}, c: [x, y]` in both renderers.
- `{b: 1, "2": x}` renders `2: x, b: 1` in both renderers.
- `as: text` over the same object renders the same line. `as: json` keeps its `<pre>` block in
  React and its one-line YAML in the terminal.
- Money:
  - `{amount: 12.5, currency: EUR}` and `{amount: "12.50", currency: EUR}` are both drawn as money
    in React and as `12.5 EUR` and `12.50 EUR` in the terminal.
  - Before the change, React draws the second as JSON.
- `display` keeps its bytes:
  - a fixture search with `q` over an object-valued row keeps the same rows (a terminal case beside
    `crates/ui/ess-ui-tui/tests/read_filter.rs`);
  - an object-free HTTP query keeps its exact request line (`crates/ui/ess-ui-tui/tests/http_adapter.rs`);
  - both pass before and after.
- The `ess-ui/1` reference says how `map` is drawn, in which key order, and that `json` shows data.
- `CHANGELOG.md` names it under Changed: React draws an object field as pairs instead of JSON, and
  the terminal braces a nested object.

## Fit review

Tree `bcfc51555`, read-only. Second report, G5.

1. **Need.** An object-valued cell reads the same in every renderer, as text a reader can scan.
   Reproduction above.
2. **Class.** *Defect*. One document gets two drawings and two test verdicts (lines above).
3. **Already expressible?** No. `labels` keys only a scalar (`fields.tsx.tmpl:127-134`), and
   `as: json` is the data view.
4. **Fit.** No grid applies (`language-grids.md:15`, `:63`). The terminal's form is taken, because
   it already exists, is one line and reads as text. No key is added. Drawing gets its own function
   so that comparison and request text do not move with it.
5. **Second adopter.** An inventory list shows a shelf location `{aisle: 4, bay: B}`. It reads
   `aisle: 4, bay: B` in both renderers.
6. **Cost.** No format version. Generated React runtime bytes change. The terminal's drawn nested
   text changes from `a: b: 1` to `a: {b: 1}`, and no request or search byte changes. Whether a
   pinned terminal test reads the old nested text was not established by reading.
7. **Considered.** (a) Change nothing. (b) React's JSON in both: that is data, not text, and
   `as: json` already says data. (c) Change `display` itself: it moves search results and HTTP query
   bytes, which would be a wire change. (d) Chosen: the terminal's pairs in both, drawn by a
   function of their own.

## Decisions

**Defect fix, accept redesigned.** No new key. The variant reading is `sl05-union-variant`.

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl` (825-831; money test 827)
- cited: `crates/ui/ess-ui-tui/src/expr.rs` (a new drawing function beside `display` 249-267;
  `display` unchanged)
- cited: `crates/ui/ess-ui-tui/src/view.rs` (`cell_value` 1899; metric 2500)
- cited: `crates/ui/ess-ui-react/tests/value_formats.rs`, `crates/ui/ess-ui-tui/tests/value_formats.rs`,
  `crates/ui/ess-ui-tui/tests/read_filter.rs`, `crates/ui/ess-ui-tui/tests/http_adapter.rs`
- inferred: `schemas/ui/ess-ui.schema.yaml` (the `as` note), `website/docs/reference/ess-ui.md`, `CHANGELOG.md`
- Size: S, about 220 changed lines.

## Depends on

Builds in the second group, after `sl03-number-as-written`, which edits the same `formatValue` and
the `as` note. It goes beside the first report's `g06-chart-sort` first unit, with which it shares
only the schema and the regenerated reference. It lands before `sl04-list-separator`.

## Source

A downstream application, 2026-11-24, second report, G5 (object half).

## Downstream report 2026-11-26

A downstream application reported on 0.99.2 (received 2026-11-26) that a list of records is drawn by the two renderers in two different ways, and that a path through a list means two things.

Facts, read on `main` at `c0b5f560cb`:

- `ess ui check --model` accepts a dotted field path through a list, `{field: lines.title}` over a view row whose `lines` is a list of records, with 0 errors (reported; not rerun here).
- React reads the member of every entry: `getPath` maps a segment over an array, so `lines.title` is the list of the titles (`crates/ui/ess-ui-react/templates/runtime/json.ts.tmpl:31-43`). The cell then goes through `formatValue`, which joins a list with `, ` and writes an object as JSON (`crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:822-831`).
- The terminal prints nothing: `walk` asks a sequence for a member by name, finds none and returns an absent value (`crates/ui/ess-ui-tui/src/expr.rs:215-224`, `field_path` 229-237).
- The reference defines `field` as a name (`website/docs/reference/ess-ui.md:1511`) and says nothing of a path that crosses a list.

So one document gives a column of titles in the browser and an empty column in the terminal, and the checker accepts both.

The need, stated apart from any spelling: one rendering of a list of records shared by both renderers (a row per record, a member per column, a tone per member), or a path through a list that both renderers resolve alike. A second adopter: an order list whose rows hold their lines, or an inventory whose shelves hold their bins, each shown as a small table under the row.

This story draws an object as one line of its members and a list member inside brackets. It does not cover this need: a list of records read as a path, or drawn as rows with members as columns, stays outside its Outcome (a record in a list is one line of pairs inside the brackets). The divergence of the path through a list is a second defect-shaped fact beside the object drawing: both renderers must resolve a path alike or the checker must refuse it. The row-and-column rendering belongs to `story:a-form-field-edits-a-list-of-records` (`as: list` with `fields:`), which carries decision LR-D1 for the path.
