---
format: aep.planning-md/3
id: story:a-list-column-draws-one-value-per-line
kind: story
status: draft
title: 'A column or record field as: list draws one value per line'
summary: 'list values are comma-joined in both renderers; as: list on a column has no meaning yet'
tags:
- backlog
- external-request
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
revision: 1
---
## Need

A UI adopter reported this on 2026-11-07, against 0.90.0.

On an input, `as: list` is a list editor (0.87.0). On a column or a record field, a list value is
joined with ", " in both renderers (`core.tsx.tmpl` FieldValue, `ess-ui-tui/src/expr.rs`). The
adopter's source application draws one value per line, and the document cannot ask for that.

## Acceptance

- A column or record field `as: list` draws each element on its own line in React and in the
  terminal, without the separator.
- An empty list draws nothing. Any other `as` keeps today's joined text.
- The reference page documents what `as: list` means on a column.

## Scope

- inferred: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl`, `fields.tsx.tmpl`,
  `styles.css.tmpl`, and the terminal cell display in `crates/ui/ess-ui-tui/src/view.rs`.
