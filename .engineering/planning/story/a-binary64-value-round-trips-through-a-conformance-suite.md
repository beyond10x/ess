---
format: aep.planning-md/3
id: story:a-binary64-value-round-trips-through-a-conformance-suite
kind: story
status: draft
title: A Binary64 value round-trips through a conformance suite
summary: 'Split from the Binary64 admission story: a reached Binary64 value is written by synthesis and compared by every runner under a stated suite contract.'
tags:
- adopter-report
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: story:a-binary64-field-is-admitted-by-a-conformance-run
revision: 1
---
## Problem

Split from `story:a-binary64-field-is-admitted-by-a-conformance-run` (fit review 2026-10-26): that
story admits a `Binary64` position nothing reaches. Carrying a reached `Binary64` value through a
conformance suite (synthesis writes it, runners compare it, the exact observers admit it) needs a
suite contract that `docs/design/model-binary64.md` § "Admission, equality and conformance
publication decisions" does not state yet.

## Acceptance

- Acceptance lines 2-4 of the parent story as written before the split, with "non-finite" read as
  an overflowing token such as `1e400` (JSON has no NaN token).
- The exact observers that `story:a-view-may-publish-a-decimal` opens for `Decimal` admit
  `Binary64` under the same suite format pair, compared by value.
- `docs/design/model-binary64.md` states the suite contract.
