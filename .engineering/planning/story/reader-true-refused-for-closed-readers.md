---
format: aep.planning-md/3
id: story:reader-true-refused-for-closed-readers
kind: story
status: draft
title: '`reader: true` is refused where the reader is closed'
relations:
- serves: vision:O2
- decomposes: epic:language-consistency-after-adoption
revision: 1
---
## Outcome

A composition that marks a field subset `reader: true` against an ESS-generated closed type is refused, because that reader rejects unknown keys (review S1).

## Acceptance

- validate refuses `reader: true` when the consumer's type is generated closed, naming the type
- CL:943-947's caveat becomes the refusal's help text

## Origin

Evidence: `docs/design/review-external-requests-2026-09.md` (reader-true-refused-for-closed-readers). Drafted, not scheduled; a fit review precedes dispatch.
