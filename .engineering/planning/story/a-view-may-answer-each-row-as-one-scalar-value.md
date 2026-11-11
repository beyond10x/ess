---
format: aep.planning-md/3
id: story:a-view-may-answer-each-row-as-one-scalar-value
kind: story
status: draft
title: A view may answer each row as one scalar value, not an object
summary: A list whose items are bare strings has no spelling in a view or an ess-http/1 route on 0.93.0 (adopter report)
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-wire-and-answers
revision: 1
---
## Need

A downstream adopter's service has list routes whose items are bare strings (`["vip", "sales"]`).
A view types each row as an object, so the model promises `[{name: "vip"}]` where the wire holds
`["vip"]`. 0.91.0 added `field: .` for this shape
(`story:a-column-may-show-a-plain-string-row`), but in `ess-ui/1` only; `ess ui check --model`
refuses it on a view of the model.

Refusals on 0.93.0 (`format: ess/37`, view `demo.tags.Tags` over entity `Tag` with identity
`name: TagName`):

```text
fields: [{name: ., type: demo.tags.TagName}]
  invalid field name identifier ".": must start with a letter, optionally after underscores
fields: [{name: name, type: demo.tags.TagName, field: .}]
  invalid type: string ".", expected struct ViewFieldSource
shape: demo.tags.TagName
  error[ESS-VIEW-002]: `demo.tags.TagName` is not a struct, so it cannot describe the fields of `demo.tags.Tags`
fields: demo.tags.TagName
  invalid type: string "demo.tags.TagName", expected a sequence
# ess-http/1 route of the same view:
body: rows.name | array.name | item.name | response.name
  error[ESS-HTTP-024]: view `demo.tags.Tags` answers `rows`, `array` or `single`, and `rows.name` is not a view's body
row: name
  unknown field `row`, expected one of `command`, `view`, `operation_id`, `method`, `path`, `parameters`, `request`,
  `outcomes`, `errors`, `alike`, `body`, `empty`, `refusals`
```

## Acceptance

One of the two forms; the scoper picks, and the story names the one it did not take.

- Model form: a view may declare `row: <field>`. Each row is that field's value. The IR, the JSON
  Schema and OpenAPI give the row schema as the field's type (`items: {type: string}`), the
  interpreted target and synthesis answer and compare bare values, and `ess-ui/1` `field: .` reads
  such a view's rows. `order_by`, `filter` and `paging` keep reading the source row.
- Binding form: an `ess-http/1` view route may declare `body: rows.<field>` or `array.<field>`,
  answering each row as the value of that field, with the OpenAPI items schema being the field's
  type. A field that is `Optional` or a struct is refused by name.
- A model or binding without the key keeps its bytes.

## Evidence

The refusals above, observed by the reporting session on the 0.93.0 release binary on 2026-11-11.
Not rerun in this repository; the scoper reruns them before the story is proposed.
