---
format: aep.planning-md/3
id: story:a-view-field-counts-or-lists-the-rows-related-to-its-source-row
kind: story
status: draft
title: A view field may count or list the rows related to its source row
summary: 'count: on a view field is an unknown field and a related-row path is ESS-VIEW-001 on 0.93.0 (adopter report)'
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
- depends_on: story:a-view-filter-tests-rows-related-to-its-source-row
revision: 1
---
## Need

A downstream adopter's reads answer, beside each row's own fields, a figure or a list read from
the rows of another entity that name it (the number of links of an item, the identities of its
links). `story:a-view-filter-tests-rows-related-to-its-source-row` lets a view filter test such
rows; `story:aggregate-views` counts rows per group in a view of its own. Neither puts a count or
a list of related rows on a row view's field. The adopter counts 12 open notes on it.

Refusals on 0.93.0 (`format: ess/37`, entity `demo.t.Link` with field `item: demo.t.ItemId`, view
`demo.t.Items` over `demo.t.Item`):

```yaml
fields:
  - {name: links, type: Integer, count: {entity: demo.t.Link, where: "item == id"}}
```

```
model was refused:
  - t.yaml: unknown field `count`
```

```yaml
fields:
  - {name: links, type: "List<demo.t.ItemId>", expr: "link.item"}
```

```
error[ESS-VIEW-001]: `link.item` is no field of `demo.t.Item`, the row `demo.t.Items` computes `links` from
```

## Acceptance

- At a new source format a row view's field may hold the count of the rows of another entity
  tied to the source row by a field equality, and a list of one field of those rows in a stated
  order; the spelling follows the row-set quantifier of
  `story:a-view-filter-tests-rows-related-to-its-source-row` and is stated in the design page.
- The figure is read when the view is read: a scenario writes a related row after the source row
  and reads the view again.
- Synthesis arranges a source row with no related row, one with two, and a related row of another
  source row; `ess verify conform run --target interpreted` passes, and a target that counts
  every row of the other entity fails.
- JSON Schema and OpenAPI type the field (`integer`, `array`); generated Go and TypeScript
  targets answer the same rows as the interpreter.
- A tie that names no field of the other entity, and a list over a field that does not exist, are
  refused naming the field. Below the new format both spellings keep today's refusals.

## Evidence

Both refusals were rerun for this story on `ess 0.93.0` and are quoted verbatim. The count of
notes is the adopter's.
