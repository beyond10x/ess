---
format: aep.planning-md/1
id: story:guarded-external-outcomes
kind: story
status: active
title: External outcomes preserve input eligibility
relations:
- decomposes: epic:model-driven-interpretation
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-xtask
- confidence: cited
  path: crates/generate
- confidence: cited
  path: crates/specify/ess-compiler
- confidence: cited
  path: crates/specify/ess-domain
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: cited
  path: docs/design
- confidence: cited
  path: models/toolchain
- confidence: cited
  path: schemas/generated
- confidence: cited
  path: website/docs/reference
revision: 13
---
## Goal
Permit an externally decided command outcome to declare the input eligibility that must hold before its cause can act. Preserve truthful observation of the resulting outcome.

## Acceptance
- An input guard and external cause survive parsing and typed IR without converting the cause into an input-selected outcome.
- Synthesis constructs an eligible input and retains ConfigureExternalOutcome; incompatible or unsatisfiable guards are refused.
- Ordinary default/finite branch selection ignores external eligibility as a competing deterministic branch.
- Old source formats refuse the new semantics and legacy canonical projections remain unchanged.
- Generated Go and TypeScript scenarios fail when the observed outcome contradicts the requested fault.

## Scope
crates/specify/ess-domain; crates/specify/ess-compiler; crates/verify/ess-conformance; dependent exhaustive matches; generated schema and support documentation. One coherent compiler change; no concurrent implementation scheduled.

## Authorization
Operator approved local implementation and a checkpoint after core model and consumer conformance validation on 2026-09-21. No release is claimed by local validation.

## Checkpoint
Local semantic and emitted-runtime tests pass. An internal adopter retains all previous named passes. Full consumer accounting remains refused on inherited finite registry gaps. No release is claimed; see docs/conformance-core-checkpoint-2026-09-21.md.
