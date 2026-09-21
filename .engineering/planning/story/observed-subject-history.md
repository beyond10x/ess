---
format: aep.planning-md/1
id: story:observed-subject-history
kind: story
status: active
title: A command can observe a subject fact and complete without a change
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
Represent session-owned history independently of lifecycle state. A completed repeat command may be silent while its declared subject remains unchanged.

## Acceptance
- A typed enum subject fact is read independently from command inputs and lifecycle state; undeclared or mistyped facts are refused.
- A silent branch explicitly preserves its subject and emits no event/error; state-only substitution is not accepted as history.
- Synthesis establishes and observes the fact through a declared view before invoking the branch, and checks the preserved fields/state afterward.
- Equal lifecycle states reached through different history paths remain distinguishable; unknown history yields a refusal, not a guessed verdict.
- Legacy source formats refuse the new semantics and existing projection bytes remain unchanged.

## Scope
crates/specify/ess-domain; crates/specify/ess-compiler; crates/verify/ess-conformance; dependent exhaustive matches; schema and public support documentation. Serial implementation beside guarded-external-outcomes because these share command and synthesis types.

## Checkpoint
Local semantic and emitted-runtime tests pass. An internal adopter retains all previous named passes. Full consumer accounting remains refused on inherited finite registry gaps. No release is claimed; see docs/conformance-core-checkpoint-2026-09-21.md.
