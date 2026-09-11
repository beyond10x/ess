---
format: aep.planning-md/1
id: story:literal-representation-walk-exhaustion
kind: story
status: draft
title: Do not admit an unchecked literal when representation traversal exhausts its bound
relations:
- serves: vision:O2
- decomposes: epic:review-boundary-remediation
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/binding.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
revision: 2
---
## Finding

The first adversary pass for story:docs-literal-mapping-claims-unchecked demonstrated that normal source parsing, assembly and compilation admit not_a_variant through32 Optional wrappers and33 named newtypes ending in an enum. The current binding representation walker stops after32 iterations and returns None; check_literal treats None as another pass's error, but no error is emitted for those finite fixtures. The admission code is unchanged from the docs unit's base; no separate historical binary was executed. The new documentation overclaim is fixed in that unit; this artifact tracks the underlying admission gap separately.

Evidence: review-result:priority-literal-pass1-20260911; local-evidence:ess-evolution-20260910/priority-wave/literal/adversary-{first,docs}.log. Both cases are in the public input domain; no forged IR is involved.

## Acceptance

Every admitted binding and outcome payload literal must undergo its declared representation check, including finite Optional/newtype mixtures at and beyond the current internal walk boundary. Exhausted or cyclic resolution must not masquerade as a missing declaration diagnosed elsewhere. Use one representation authority and deterministic, source-addressed failure where a supported representation cannot be established; retain existing valid enum/String semantics and nontext refusals. Cover shared binding/payload consumers, exact boundary, deeper named chains and cycles with red-first cases. Do not silently strengthen unrelated String invariant or external-resource claims.

## Scope

crates/specify/ess-domain/src/binding.rs representation/check_literal and crates/specify/ess-domain/src/command.rs payload literal consumer — cited. Their existing tests and compiler integration fixtures — inferred. This is a follow-up, outside the currently selected priority wave; it does not hold the docs correction once the introduced overclaim is removed. No new entity or persisted format is introduced.
