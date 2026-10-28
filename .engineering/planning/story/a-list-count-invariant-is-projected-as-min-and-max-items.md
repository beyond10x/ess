---
format: aep.planning-md/3
id: story:a-list-count-invariant-is-projected-as-min-and-max-items
kind: story
status: draft
title: 'A list or map .count invariant against a literal is published as minItems/maxItems (deferred from #685)'
summary: 'List and Map .count bounds against an integer literal, split out of the #685 text-length story'
refs:
- provider: github
  reference: beyond10x/ess#685
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

A list's or map's `.count` bound against a literal (for example `lines.count >= 1`, `lines.count <= 50`) is
not published as a JSON Schema bound, so a validator accepts an empty or oversized list that ESS refuses.
The text-length half of #685 is covered by its own story; this story is the list and map half that the
scoper deferred.

Known constraint for the design: realize turns `minItems`/`maxItems` into the Array shape
(`crates/generate/schema-contract/src/realize.rs:1517-1518`), and the Rust and Go behaviour for a bounded
non-tuple array was not read when this was drafted.

## Acceptance

- A `.count` bound on a List or Map field against an integer literal is published on that field's
  property, and a validator refuses a list outside the bound.
- A bounded non-tuple list realizes in Rust, Go and TypeScript without a shape change or refusal, and its
  bound is recorded as an obligation.
- The stories this one depends on in shape (the text-length story for #685) keep their behaviour.
