---
format: aep.planning-md/3
id: story:a-copied-related-identity-is-witnessed
kind: story
status: draft
title: A row-set guard over a copied related identity is witnessed or refused by name
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
revision: 4
---
## Outcome

A row-set guard over a field a creating branch fills with a `{related:}` copy of the related row's
identity is witnessed, and the interpreted model passes the scenario.

## Problem

An adversarial review of an integration branch (2026-10-19, finding T3) found a copied identity
guard refused; re-checked on 0.92.0 and still refused.

`crates/verify/ess-conformance/tests/fixtures/row-set-related-copy.yaml`, changed so the seat
copies the tier's identity rather than its `qualifies` flag:

```yaml
# Seat
fields:
  - {name: holder, type: demo.seats.HolderRef}
  - {name: tier, type: demo.seats.TierId}
# Issue/issued
sets: {holder: input.holder, tier: {related: {via: input.tier_id, field: tier_id}}}
# Check: input gains {name: tier_id, type: demo.seats.TierId}; the qualified branch reads
when_related:
  entity: demo.seats.Seat
  where: {all: [holder == input.holder, tier == input.tier_id]}
  exists: true
```

`ess verify conform synthesize --path spec.yaml --out suite.yaml` on 0.92.0:

```
refused: refusal[ESS-SYNTH-001]: outcome demo.seats.Check/qualified has no scenario `demo.seats.Check/outcome/qualified`
  no witness: `demo.seats.Check/qualified: its own row set is False` is `row set`, which has no arrangement of rows the declared commands produce on which the rows its selector selects take this branch
  help: let a command that creates the entity write each field the filter reads from an input or from a caller attribute (`{caller: <attribute>}`) rather than from a literal or a generated value, so the arranged rows can hold values on both sides of the filter
```

Declaring `{name: tier_ref, kind: references, target: demo.seats.Tier, cardinality: one, via: tier}`
on the seat gives the same refusal. The unchanged fixture (copying `qualifies`) is witnessed.

The refusal is wrong: define a tier T, issue a seat under T for holder H, then `Check(H, T)` selects
that seat. The help line is misleading too, because `Issue` already writes `tier` from an input
through the copy. The identity arms of `held` (`crates/verify/ess-conformance/src/synthesize/row_set/copied.rs:61-68`)
and `after_referenced` (`copied.rs:318-345`) are reached by no test.

## Acceptance

- `crates/verify/ess-conformance/tests/row_set_related_copy.rs`,
  `a_copied_related_identity_is_witnessed`: for the model above, synthesis records no refusal,
  `demo.seats.Check/outcome/qualified` exists, the `tier_id` it sends to `Check` is the instance an
  earlier `DefineTier` step created and an earlier `Issue` step was sent, and the interpreted target
  passes it. Red on 0.92.0.
- A mutant that makes the identity arm of `after_referenced` settle nothing fails that test.
- A mutant removing `before.extend(read)` in the update wiring of
  `crates/verify/ess-conformance/src/synthesize/row_set.rs:1596` fails a test in
  `row_set_related_copy.rs`.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/row_set/copied.rs
- cited: crates/verify/ess-conformance/src/synthesize/row_set.rs
- cited: crates/verify/ess-conformance/tests/row_set_related_copy.rs
- inferred: crates/verify/ess-conformance/tests/fixtures/row-set-related-copy-identity.yaml

confidence: medium
