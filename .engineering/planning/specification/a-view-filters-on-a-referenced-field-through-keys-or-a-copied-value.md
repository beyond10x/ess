---
format: aep.planning-md/3
id: specification:a-view-filters-on-a-referenced-field-through-keys-or-a-copied-value
kind: specification
status: draft
title: A view filters on a referenced entity's field through its keys or a value copied at write
relations:
- serves: vision:O2
revision: 1
---
## Idiom

A view keeps one source. A filter on a referenced entity's field is written one of two ways
(https://github.com/beyond10x/ess/issues/492, gap 4; fit review on
`story:a-view-filter-follows-a-reference`):

- **Current values, group or tag membership.** The caller reads a view of the referenced entity,
  resolves the names it has into the referenced rows' keys, and sends those keys to a list filter on
  the reference field (`story:a-list-parameter-selects-rows-outside-a-group-key`). An authored
  scenario checks it until synthesis witnesses that selector.
- **Values fixed for the row's life, such as a public id.** Copy the value at write with
  `{related: {via: input.<key>, field: <field>}}` and filter on the copy. Synthesis witnesses it with
  a scalar selector today: 4 scenarios, 0 refusals, 4 passed on the interpreter
  (reproduction `g4/i2c-copy-scalar.yaml` of the fit review).

A live cross-entity filter, if needed later, starts from the shape named for a view field:
`{related: {via: <reference field>, field: <field>}}` read at query time, one hop.
