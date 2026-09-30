---
format: aep.planning-md/3
id: story:ess-ui-type-grammar-aligns
kind: story
status: draft
title: ess-ui types are spelled as ESS types
relations:
- serves: vision:O2
- decomposes: epic:language-consistency-after-adoption
revision: 1
---
## Outcome

ess-ui documents name types and callers in ESS's vocabulary, not a second grammar (review S22).

## Acceptance

- ess-ui accepts `List<T>`, `String` and the ESS type names beside its structured forms, or its schema maps each to one ESS type by table
- the ess-ui `actor` field refers to the model's actors or states why a third notion is needed

## Origin

Evidence: `docs/design/review-external-requests-2026-09.md` (ess-ui-type-grammar-aligns). Drafted, not scheduled; a fit review precedes dispatch.
