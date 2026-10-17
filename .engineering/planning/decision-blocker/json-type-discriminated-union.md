---
format: aep.planning-md/3
id: decision-blocker:json-type-discriminated-union
kind: decision-blocker
status: open
title: Decide whether and how a value told apart by JSON type is declared
relations:
- blocks: story:json-type-discriminated-union
revision: 1
---
## Approved requirement blocked

story:json-type-discriminated-union (beyond10x/ess#594): a value written either as a short scalar
form or as an object long form, told apart by JSON type, cannot be declared; `kind: union` requires
an adjacent `tag`.

## Concrete decision

1. Does ESS admit such a value, and in which form: (b) a union whose variants are discriminated by
   JSON type, refused when two variants share one; or (c) a struct that may also be written as one
   scalar filling one named field; or neither (the long form stays the only form)?
2. What is "one JSON type" for the collision rule: `Decimal`, `Timestamp`, `Duration`, `Uuid`,
   `Bytes` and enums are strings on the wire; `Integer` and `Binary64` are numbers; `Optional` adds
   `null`; a newtype has its representation's type.
3. Which source format carries it: none of G1–G4 in `docs/design/generic-language-features.md`
   covers a type body form, and policy 2 there allows one format per design decision.

Evidence: the story's fit review; `docs/design/ess-review-v0.1.md:192-198` (tagged-only, untagged as
a separate form later).
