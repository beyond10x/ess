---
format: aep.planning-md/3
id: specification:a-view-field-reads-a-referenced-row-through-field-related
kind: specification
status: draft
title: A view field reads a referenced row through field related
summary: 'The idiom for a view field showing a referenced row field: field related via the reference field, ess/28; expr for the source row own fields, ess/27; no from.'
relations:
- serves: vision:O2
revision: 1
---
## Idiom

A view field that shows a field of a row the source references is written `field: {related: {via: <reference field>, field: <field>}}` (`ess/28`), read at query time, one hop. A value computed from the source row's own fields is written `expr:` (`ess/27`). There is no `from:` on a view field; `story:a-view-field-from-names-the-related-spelling` refuses it at its line with this idiom as the hint.

## Evidence

Reproduction `n22-idiom.yaml` of the fit review (a shelf view showing the cover of the book it references), ess 0.69.2, 2026-10-26: `ess verify conform synthesize` 4 scenarios, 0 refusals; `ess verify conform run --target interpreted` all passed.
