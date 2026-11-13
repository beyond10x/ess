---
format: aep.planning-md/3
id: story:a-form-field-edits-a-list-of-records
kind: story
status: active
title: A form field edits a list of records, and a column or record draws one
summary: 'as: list with fields: edits and shows a List of structs; bounds from the model''s .count invariant (UI adopter composition report, P1)'
tags:
- backlog
- external-request
- queue-0.93.0
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
scope:
- confidence: inferred
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: inferred
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: inferred
  path: crates/ui/ess-ui/src/model.rs
- confidence: inferred
  path: docs/design
- confidence: inferred
  path: schemas/ui/ess-ui.schema.yaml
- confidence: inferred
  path: website/docs/reference/ess-ui.md
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-08T15:49:57Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-08T15:49:58Z", actor: "human:timo", revision: 4}
---
## Need

A command input often holds a list of records: opening hours as a list of `{open, close}` ranges, a
rule list of `{expression, operator, value}` rows, a list of prompt entries, a list of metrics. An
`ess-ui/1` form cannot edit one. `as: list` edits a list of scalars only: the React runtime draws
one text input per entry and writes the entry back as a string
(`crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl:241-268`), so an element that is a
record is shown as JSON text and saved as a string. The same list cannot be drawn as nested records
in a column or a record either. Every such list stays a hand-written host component.

Evidence: a UI adopter's collected report (2026-11-08) classified 531 gap lines of one manager front
end; a list field of records closes 29 of them (25 host-component lines, 4 checklist lines), second
after a composed read. 16 of the 38 lines of its largest host component (a step editor) are lists of
records, some nested (a weekday with its list of ranges).

Minimal reproduction (brand-free): a command `hours.SetWeek` whose input `ranges` is
`List<hours.Range>`, `hours.Range` a struct `{open: String, close: String}`. Writing the field
`{field: ranges, as: list, item: [open, close]}` in a form is refused by `ess ui check` (ess 0.90.0):
`error document_loads …/fields/ranges/item/open: the Composite shorthand accepts ref:
composite_kind, not open`. Without `item`, the form saves each range as a string.

## Fit review

Tree `b33454b88`, binary ess 0.90.0.

1. **Need.** A form edits, and a column or record shows, a `List` whose element is a struct: add,
   remove and move an entry, edit each entry's fields with the widgets those fields already have.
   Requester's proposed syntax, labelled as theirs: `{name: ranges, as: records, min: 0, max: 8,
   reorder: true, item: {fields: [...]}, add: {label: …}}`.
2. **Class.** *Gap*: the domain fact (a list of structured entries) is declared in the model and
   cannot be edited or shown by the UI document at all; the only route is a host slot, which no
   check reaches.
3. **Already expressible?** No. `as: list` is scalars only (fields.tsx.tmpl:241-268); `Field` has
   no key for an element's fields (`crates/ui/ess-ui/src/model.rs:2372-2408`); the check resolves
   enum variants and scalars only and returns `None` for a struct
   (`crates/ui/ess-ui-check/src/model.rs:2037-2068`); a nested `collection` reads a view, not a
   draft value (`model.rs:1227-1265`). Run above: refused.
4. **Fit.** Grid: this opens the whole "nested value in a field" column for `List<struct>` in every
   field position (form input, collection column, record field, tab field, form group field), not
   one cell. Vocabulary: the list stays `as: list` (no `records` widget beside it); the element's
   fields are written `fields:`, as a form, record, tab and form group already spell their fields
   (`model.rs:1392-1449`, `2600-2656`), each a `Field` checked against the element struct with
   `--model` as a form field is checked against the command input. Bounds come from the model's
   `.count` invariant on the input, not from an authored `min`/`max`, so the UI never states a
   second bound conformance does not check. Order: an ESS `List` is ordered, so moving an entry is
   offered; no `reorder` key (a collection's `reorder` names a command that saves the order,
   `model.rs:1382-1388`, a different thing). Nested: an element field may itself be `as: list` with
   `fields:`. Targets: React and the terminal renderer draw it; `ess ui check` checks it; the
   generated docs come from the schema; a target that cannot draw it refuses it by node path.
5. **Second adopter.** A shop's product form editing a list of `{name, price}` variants, each with
   a list of `{label, value}` attributes.
6. **Cost.** One optional key (`fields:`) on `Field`, admitted only beside `as: list`; additive in
   `ess-ui/1` as host slots were in 0.90.0; no ESS source format; no migration; new findings for a
   `fields:` on a non-list or naming no member of the element struct.
7. **Considered.** (a) change nothing: every list of records stays a host component. (b) the
   requester's `as: records` with authored `min`/`max`/`reorder`/`add`: a second widget name for a
   list, and bounds the model does not check. (c) chosen: `as: list` with `fields:`, bounds from the
   model's invariant.

## Decisions

**Accept, redesigned.** The need is adopted; the shape is changed: `as: list` + `fields:` instead
of `as: records` + `item:`; `min`/`max` read from the model's `.count` invariant instead of being
authored; no `reorder` key; the add button's text is the field's `label`. Design page under
`docs/design/` first.

## Acceptance

- `ess ui check` accepts a form field `{field: ranges, as: list, fields: [open, close]}`, and with
  `--model` refuses, by node path, an entry of `fields` that names no member of the element struct
  and a `fields:` on a field that is not `as: list` or whose element is not a struct.
- The generated React form shows one row per entry with an input per element field, add, remove
  and move up/down, and submits the command input as a JSON array of objects (a React test).
- A list whose element struct holds another `as: list` with `fields:` (weekday by ranges) edits and
  submits both levels.
- With `--model`, a `.count` invariant on the input disables remove at its minimum and add at its
  maximum in React; without an invariant, no bound is applied.
- A collection column or record field `as: list` with `fields:` draws each entry as a nested
  record, in React and in the terminal renderer.
- The terminal renderer edits the same list (add, remove, move, edit an entry's fields) in an
  `ess ui test` case.
- A document without `fields:` on any field keeps its generated React project byte-identical.
- The `ess-ui/1` reference (schema and `website/docs/reference/ess-ui.md`) documents the key.

## Scope

- `docs/design/` — new design page (inferred)
- `crates/ui/ess-ui/src/model.rs` — `Field.fields` (inferred)
- `schemas/ui/ess-ui.schema.yaml` — `Field.fields` (inferred)
- `crates/ui/ess-ui-check/src/model.rs`, `crates/ui/ess-ui-check/src/rules.rs` — element-struct
  checks (inferred)
- `crates/ui/ess-ui-react/templates/runtime/fields.tsx.tmpl`, `crates/ui/ess-ui-react/src/emit.rs`
  (inferred)
- `crates/ui/ess-ui-tui/src/view.rs` (inferred)
- `website/docs/reference/ess-ui.md` (inferred)

## Reported cases (2026-11-13)

From a consumer's UI document, second collected report, reproduced on ess 0.94.0 with a document of
our own. Recorded here because the case is this story's class; the acceptance above or below is
not changed by this section, and the proposed line is taken up when the story is scoped.

Item 4, a list field of a row shown as rows. A row of a collection holds a list (the lines of an
order) and the consumer wants it drawn as rows under the row, with columns. A nested `collection`
reads a view only:

```
error document_loads pages/requests.inbox/sections/list/item/lines: `collection`: unknown field `rows`, expected one of `reads`, `columns`, `sort`, `style`, `selection`, `row_actions`, `bulk_actions`, `actions`, `expand`, `item`, `reorder`, `group_by`, `group_order`, `show_empty_groups`
error view_in_model pages/requests.inbox/sections/list/item/lines/reads: `row.lines` names no view of model `desk`
```

(`item: [{name: lines, component: collection, rows: row.lines, columns: [title]}]`, and the same
with `reads: row.lines`; `expand` answers as `item` does.) This story's `as: list` with `fields:` in
a column or a record field is the same need. What the report adds: the consumer expected the
element's fields as columns of a table under the row, in a row's `item` or `expand`.

Proposed acceptance line: a column or record field `as: list` with `fields:` standing in a row's
`expand` draws the list as a table, one element per line and one field per column, in both
renderers.
