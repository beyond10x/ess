---
format: aep.planning-md/3
id: story:a-column-may-show-a-plain-string-row
kind: story
status: active
title: 'A column or record field may show a plain-string row with field: .'
summary: 'Gap B4-40 (G-51): field: . names the row itself'
tags:
- queue-0.91.0
- rank-11
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/ui/ess-ui-check/src/names.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/composites/collection.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/expr.rs
- confidence: inferred
  path: crates/ui/ess-ui/src/model.rs
- confidence: cited
  path: schemas/ui/ess-ui.schema.yaml
- confidence: cited
  path: website/docs/reference/ess-ui.md
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-06T22:19:02Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-06T22:19:03Z", actor: "human:timo", revision: 3}
---
## Need

Gap item B4-40 (G-51): a view whose rows are plain strings. A choice already offers such a row as
both value and label (`Choice::option`, `crates/ui/ess-ui/src/model.rs`), and since 0.89.0 an
`as: tags` input's `suggests` takes it too. A column or record field still has no way to name the
row itself, so a list of names cannot be shown as a table.

## Decision (lead, under the 0.90.0 approval)

`field: .` names the row itself, in a collection column and in a record field. On a row that is
an object, it reads as an absent value. Choice options keep their current rule.

## Acceptance

- A collection whose read answers `["alpha", "beta"]` with one column `field: .` draws two rows,
  `alpha` and `beta`, in React and in the terminal.
- `ess ui check --model` accepts `field: .` on a read whose row type is a string.
- `ess ui check --model` refuses `field: .` by name on a read whose rows are entities.
- Sort by that column orders the strings. Row keys fall back to the row itself.
- The `ess-ui/1` reference page states the rule.

## Scope

Derived 2026-11-07 by `story-scoper`. Every line is **cited** (read from the story or the tree) or
**inferred** (a reading that could be wrong).

- **Primary surface, `crates/ui/ess-ui-check`** — cited. `field_in_model` resolves columns and
  record fields in `src/names.rs` (`Known::fields`, `field_head`). Today `field: .` splits into
  empty heads and is checked against nothing.
- **Primary surface, `crates/ui/ess-ui-react/templates/runtime`** — cited.
  - `display` in `core.tsx.tmpl` draws a cell.
  - The client sort in `composites/collection.tsx.tmpl` calls it.
  - `rowKey` already falls back to `stableKey(row)`.
- **Primary surface, `crates/ui/ess-ui-tui/src`** — cited.
  - `field_path` in `expr.rs` draws cells and record fields.
  - The sort and the row key (`row.get(key_field)`) are in `app.rs`.
- **Documents:** `schemas/ui/ess-ui.schema.yaml` (the `Field.field` note) and
  `website/docs/reference/ess-ui.md`, generated from the schema by `crates/ui/ess-ui-docs` — cited.
- **Also likely:** `crates/ui/ess-ui/src/model.rs` (the `Field` doc, and the node-name default for
  `.`) and `expand.rs` — inferred.
- **Also likely:** new tests in `crates/ui/ess-ui-check/tests`, `ess-ui-react/tests` and
  `ess-ui-tui/tests` — inferred.
- **Confidence:** medium. The `--model` acceptance for a string row has no model surface: every
  view has a source entity and named fields.
- **Would collide with:** story:a-client-paged-collection-offers-sizes-and-a-range, on
  `collection.tsx.tmpl`, the terminal `app.rs`, the schema and the reference page.
