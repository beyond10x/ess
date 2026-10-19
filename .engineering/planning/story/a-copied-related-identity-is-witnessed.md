---
format: aep.planning-md/3
id: story:a-copied-related-identity-is-witnessed
kind: story
status: draft
title: A row-set guard over a copied related identity is witnessed or refused by name
relations:
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Outcome

A row-set guard over a `{related:}` copy of the related row's identity is witnessed, or refused with
a cause that names the copy.

## Evidence

The 2026-10-19 adversarial review of the integration branch (finding T3): a fixture that copies the
related row's identity through `{related: {via: input.tier_id, field: <identity>}}` and guards a
`Check` branch on it is refused by synthesis as `Check/qualified` "its own row set is False". The
identity arms of `held` and `after_referenced` in
`crates/verify/ess-conformance/src/synthesize/row_set/copied.rs` are therefore not reached by any
test. Not yet traced: whether the refusal is correct.

## Acceptance

- `tests/row_set_related_copy.rs` has a copied-identity case that is either witnessed (and passes
  the interpreter) or refused with a reason naming the copy.
- A mutant removing `before.extend(read)` in the update wiring of `synthesize/row_set.rs` fails a
  test.
