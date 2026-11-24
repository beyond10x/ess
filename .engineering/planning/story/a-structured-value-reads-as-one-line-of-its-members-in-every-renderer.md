---
format: aep.planning-md/3
id: story:a-structured-value-reads-as-one-line-of-its-members-in-every-renderer
kind: story
status: draft
title: A structured value reads as one line of its members in every renderer
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-number-with-no-declared-number-format-is-drawn-as-written-in-every-renderer
revision: 1
---
## Need

The two renderers draw a field whose value is an object in two different ways. The generated React
project writes the object's JSON, `{"kind":"pickup","name":"north"}`. The terminal writes its
members as `kind: pickup, name: north`. Neither renderer reads `as: map`.

- React: `FieldValue` has no `map` case, so the field falls to `formatValue`
  (`crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl:200-201`), which returns
  `JSON.stringify(value)` for an object that is not money
  (`crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:825-831`).
- Terminal: `cell_value` has no `map` case, so the field falls to `display`
  (`crates/ui/ess-ui-tui/src/view.rs:1899`), which joins `key: value` pairs with `, `
  (`crates/ui/ess-ui-tui/src/expr.rs:256-265`). It nests a member object the same way, and a list
  member joins with `, `.
- The reference lists `map` among the values of `as` and says nothing about how it is drawn
  (`website/docs/reference/ess-ui.md:1505`). `json` is the value that shows the value as data:
  React draws a `<pre>` block (`fields.tsx.tmpl:190-191`) and the terminal draws YAML on one line
  (`view.rs:1890-1892`).

The renderers disagree on one document, so the object half of the second report's G5 is a defect.
This story fixes that half. The variant reading of a tagged union is
`sl05-union-variant`.

Minimal reproduction, written fresh and not run here (read-only assessment):

```yaml
columns:
  - {field: place, as: map}        # fixture row: {place: {kind: pickup, name: north}}
```

React shows `{"kind":"pickup","name":"north"}`. The terminal shows `kind: pickup, name: north`.

The requester's proposal, restated: a union reads as its variant and its fields
(`pickup north`), or `labels` per variant.

## Outcome

A field whose value is an object, under `as: map` or under `as: text` (the default), reads as one
line of `key: value` pairs, in the order the answer holds them, joined with `, `. A member object
is drawn the same way inside braces, `place: {kind: pickup, name: north}`. A list member is joined
with `, ` inside brackets. Money keeps its own form. `as: json` keeps showing the value as data.
Both renderers draw the same text.

## Acceptance

- Red first, in `crates/ui/ess-ui-react/tests/value_formats.rs`: `{field: place, as: map}` over
  `{place: {kind: pickup, name: north}}` renders `kind: pickup, name: north`. It fails before the
  change, rendering the JSON. The same case passes in `crates/ui/ess-ui-tui/tests/value_formats.rs`.
- A nested value `{a: {b: 1}, c: [x, y]}` renders `a: {b: 1}, c: [x, y]` in both renderers. The
  terminal's `display` gains the braces and brackets for a nested member. A top-level list stays
  as `display` draws it today.
- `as: text` over the same object renders the same line. `as: json` keeps its `<pre>` block in
  React and its one-line YAML in the terminal.
- A money object `{amount, currency}` keeps its currency form in both renderers.
- The `ess-ui/1` reference says how `map` is drawn and that `json` shows data.

## Fit review

Tree `bcfc51555`, read-only. Second report, G5.

1. **Need.** An object-valued cell must read the same in every renderer, and as text a reader can
   scan rather than as JSON. Reproduction above.
2. **Class.** *Defect*: two renderings of one document (lines above).
3. **Already expressible?** No. `labels` keys only a string, number or boolean
   (`fields.tsx.tmpl:127-134`), and `as: json` is the data view.
4. **Fit.** No grid applies (`language-grids.md:15`, `:63`). The terminal's form is taken because it
   already exists, it is one line, and it reads as text. No key is added.
5. **Second adopter.** An inventory list shows a shelf location `{aisle: 4, bay: B}`. It reads
   `aisle: 4, bay: B` in both renderers instead of JSON in one of them.
6. **Cost.** No format version. Generated React runtime bytes change. The terminal's nested-object
   text changes from `a: b: 1` to `a: {b: 1}`: the old text was ambiguous, and no pinned test reads
   it (to be confirmed by the implementor with `git grep` over `crates/ui/*/tests`).
7. **Considered.** (a) Change nothing. (b) React's JSON in both: it is data, not text, and `as: json`
   already says data. (c) Chosen: the terminal's pairs in both, with nesting made unambiguous.

## Decisions

**Defect fix, accept redesigned.** No new key. The variant reading the requester asked for is the
separate story `sl05-union-variant`, because it needs the model's union tag.

## Scope

- cited: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl` (825-831)
- cited: `crates/ui/ess-ui-tui/src/expr.rs` (`display` 249-267)
- cited: `crates/ui/ess-ui-react/tests/value_formats.rs`, `crates/ui/ess-ui-tui/tests/value_formats.rs`
- inferred: `schemas/ui/ess-ui.schema.yaml` (the `as` note), `website/docs/reference/ess-ui.md`
- Size: S, about 150 changed lines.

## Depends on

`sl03-number-as-written` (the same `formatValue`, landed first).

## Source

A downstream application, 2026-11-24, second report, G5 (object half).
