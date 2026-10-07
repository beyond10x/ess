---
format: aep.planning-md/3
id: story:an-authored-text-literal-outside-its-alphabet-is-refused
kind: story
status: draft
title: ESS refuses an authored text literal that is outside its field's alphabet
relations:
- serves: vision:O2
revision: 2
---
## Finding

Measured by adversary pass 1 of wave 2026-10-07b
(`review-result:adversary-wave-20261007b-u1-pass-1`, F1), pre-existing on `main`: ESS checks an
authored literal against a String type's declared prefix but not against its `alphabet`
(`crates/specify/ess-domain/src/command.rs:4499`), so `sets: note: "kept!"` validates against an
alphabet without `!`. The Entity Runtime lowering now refuses such a literal by name; the
specification itself still admits it.

## Acceptance

`ess specify validate` refuses an authored text literal whose characters are not all in the
effective alphabet of the field or argument it is written into, naming the literal, the first
character outside the alphabet and its location, the way beyond10x/ess#146 refused prefixes.

## Wider case

The U1 implementor of wave 2026-10-07b found the same gap one step wider, pre-existing: ESS also
admits a `sets:` literal that breaks its type's `invariants:`, and the lowered Entity Runtime rule
then refuses that branch on every request. The acceptance covers both: a literal that breaks its
type's alphabet or its invariants is refused at validation.
