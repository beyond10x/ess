---
format: aep.planning-md/3
id: story:a-view-field-from-names-the-related-spelling
kind: story
status: draft
title: 'A view field from: names the related spelling'
summary: 'Adopter report N22: from: on a view field gives no hint (ess#675).'
refs:
- provider: github
  reference: beyond10x/ess#675
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

A view field written `from: book.cover` is refused with ``unknown field `from` ``, and the adopter concluded joins are unsupported. Joins exist since ess/28 (`field: {related: {via, field}}`) and row-computed values since ess/27 (`expr:`). Adopter report N22 (ess#675), ess 0.68.0.

## Acceptance

- `a_view_field_from_names_the_related_spelling`: the refusal for `from:` on a view field names `field: {related: {via: ..., field: ...}}` and `expr:`.
- `the_hint_appears_only_on_view_fields`: `from:` elsewhere keeps its current refusal text.
