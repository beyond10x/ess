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
revision: 2
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

## Second report: validate refuses a reached Binary64 once authored scenarios are listed (2026-10-25)

Reported by the Loom specification session against ESS 0.69.0: `ess specify validate` refuses a
`Binary64` field with `ESS-AUTHOR-036` once the specification lists authored scenarios; Loom's gate
tests strip `scenarios:` from their copy as a workaround, and the specification itself types the
value as `Decimal`.

Triage on main (f692aab, 0.77.0), 2026-10-31: still refused. A copy of that specification with its
four `Optional<Decimal>` confidences retyped to `Optional<Binary64>`, authored scenarios kept:

```text
refusal[ESS-AUTHOR-036]: model
  command.….RecordSelection.input.confidence.of, command.….SelectAction.input.confidence.of, entity ….Selection.fields.confidence.of, entity ….SelectionRecord.fields.confidence.of: Binary64 requires a qualified conformance codec
  help: use checked format-5 normalization; finite Binary64 conformance codecs are not implemented
```

This is the reached-`Binary64` half this story owns; wave D325 leaves it for the next wave.
