---
format: aep.planning-md/3
id: story:a-related-guard-over-an-absent-optional-input-is-witnessed
kind: story
status: active
title: A related guard that needs an Optional input absent is witnessed
summary: A when_related predicate true when an Optional input is left out gets no witness (ESS-SYNTH-003); as when_subject it does (row 29, 2026-10-23).
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T04:52:43Z", actor: "human:timo", revision: 2, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T04:52:43Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
---
## Need

Reported by an adopter (row 29, triaged on 0.69.0). A `when_related:` predicate that needs an
`Optional` input absent, such as `not defined(input.tenant) or tenant != input.tenant`, gets no
witness: synthesis refuses the outcome with ESS-SYNTH-003 ("no candidate of the 4 tried"). The same
guard written as `when_subject:` synthesizes with 0 refusals. With the guard on the creator, two
further outcomes that arrange rows through that creator lose their witnesses (ESS-SYNTH-001).

## Where it lives

- `crates/verify/ess-conformance/src/synthesize/related_guard.rs:3525` (`with_row`: the input is
  searched with row/input comparisons grounded on the row, never with the input left out).
- `related_guard.rs:2032-2125` (`boundary_goals`, `isolating`).
- `synthesize.rs:1606-1609` (the refusal message).

## Acceptance

- A brand-free minimal spec (a creator `MakeChild` with an `other-tenant` `when_related:` refusal over
  the parent row, `not defined(input.tenant) or tenant != input.tenant`) synthesizes
  `MakeChild/outcome/other-tenant` with 0 refusals, and the suite passes on `--target interpreted`.
- A target that ignores the `not defined(input.tenant)` disjunct fails that suite.
- Boundary goals that no row can satisfy under three-valued comparison (for example
  "`defined(tenant)` false and `tenant != input.tenant` true") are pruned or classified, not searched.
- Outcomes that arrange rows through the guarded creator keep their witnesses.

## Format

None.
