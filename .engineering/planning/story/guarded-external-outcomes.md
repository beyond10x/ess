---
format: aep.planning-md/1
id: story:guarded-external-outcomes
kind: story
status: implemented
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
revision: 15
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

Implemented in PR #56 at 68581d70bb47a048cd399e55c68f225b80303977 and published as ESS 0.28.0. The exact source passed the operator-approved task check SKIP_CONSUMER_CHECKS=true profile, task site-lab, required shared and repository checks, and the release workflow. All four native archives and SHA256SUMS were downloaded and verified; the Linux binary reports ess 0.28.0. The annotated tag belongs to main and task release-status passes. The full default consumer-accounting refusal remains recorded; no accounting baseline was relaxed. See docs/conformance-core-checkpoint-2026-09-21.md and https://github.com/beyond10x/ess/actions/runs/35639713420.
