---
format: aep.planning-md/3
id: story:an-inexact-authored-decimal-is-refused
kind: story
status: draft
title: An authored Decimal that binary64 cannot hold exactly is refused at validation
relations:
- serves: vision:O2
- informed_by: decision-blocker:decimal-canonical-serialization-crosses-repositories
revision: 1
---
## Outcome

`ess specify validate` refuses, by name and at the literal's location, an authored `Decimal`
literal whose value binary64 cannot represent exactly (for example `0.1000000000000000000001`,
which today is written and displayed as `0.1`). Serialization is unchanged: no persisted byte
moves, no format version changes.

## Decision

Taken 2026-10-07 on `decision-blocker:decimal-canonical-serialization-crosses-repositories`: a
value is refused by name rather than changed silently. The exact-decimal writer (Decimal as a JSON
string, `ess-conformance/6`, `ess-conformance-report/2`, a reader in AEP first) stays parked on
`obligation:review-contract-rollout-coordination`.

## Not covered

A `Decimal` that reaches ESS at run time (an implementation's reply, a conformance report value)
is not authored and is not refused by this story. It still collapses to the nearest binary64 value
when written. That case stays with the parked blocker.

## Acceptance

- A specification carrying `0.1000000000000000000001` as a `Decimal` literal fails
  `ess specify validate` with a refusal naming the literal, its location and the nearest binary64
  value; `0.1`, `0.5` and `1e2` still validate.
- The refusal is modelled in the repository's ESS specification before the validator changes
  (spec first), and the change is one conformance scenario or test per accepted and refused case.
- No vector in `crates/specify/ess-primitives/tests/vectors/primitive-semantics.json` changes its
  `json` bytes.

No GitHub issue reports this case and no fixture carries such a literal (measured in wave 22).
