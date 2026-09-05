---
format: aep.planning-md/1
id: story:review-infra-ir-invariants
kind: story
status: draft
title: Keep resolved infrastructure handles valid for the IR lifetime
tags:
- P0
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli
- confidence: inferred
  path: crates/infra/infra-analyze
- confidence: cited
  path: crates/infra/infra-compiler
- confidence: inferred
  path: crates/infra/infra-project
- confidence: inferred
  path: crates/infra/infra-spec
- confidence: inferred
  path: docs/design/review-infra-ir-invariants.md
revision: 8
---
## Finding and source

F02 (P0) from `docs/reviews/2026-09-05-architecture-review.md:188`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/infra/infra-compiler/src/ir.rs:470`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Public consumers cannot invalidate a resolved infrastructure handle by mutating its owning IR collections after compilation.

## Implementation boundary

Make invariant-bearing model collections private and provide read queries or validated transformations required by existing consumers. Inventory every public mutation route, including deserialization. Preserve total compiler-minted lookup behavior; migrate each actual consumer to the read API in the same integration unit.

## Validation

A compile-fail or equivalent public-API test rejects the review's clear-after-resolve mutation; positive lookup/transform tests cover all handle families. Run infra compiler and downstream infra/CLI tests. Confirm no persisted infra-ir bytes change.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

This is a deliberate multi-package API migration and cannot join the first small wave; no EssIr/InfraIr merger or new infra-ir version is implied by privacy alone.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/infra/infra-compiler` — cited; owning implementation or documented surface.
- `crates/infra/infra-analyze` — inferred; planned edit surface, verify before dispatch.
- `crates/infra/infra-spec` — inferred; planned edit surface, verify before dispatch.
- `crates/infra/infra-project` — inferred; planned edit surface, verify before dispatch.
- `crates/edge/ess-cli` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

## Pre-dispatch inventory

Read-only coordinator inventory at wave 3 opening 45832cc885377b2d61845ee33af14f0293d99e67; this is preparation, not independent scoping or implementation evidence.

- InfraIr exposes its owning model at crates/infra/infra-compiler/src/ir.rs:535. InfraModel's public maps and nested values are mutable through that field; the six handles remain privately minted. Keeping only the owner private with a shared model query can protect all nested collections without gratuitously privatizing detached data.
- The supported reader is infra_compiler::read_document, not a Deserialize implementation on InfraIr. Its private mirrors reject unknown fields and remint six handle families after target checks. Its documented boundary deliberately does not revalidate all domain values or derive unresolved accounting; do not claim privacy fixes these separate semantics.
- Actual in-workspace consumers are infra-analyze, infra-spec, infra-project and ess-cli. The projector is a writer, not just a read migration: project.rs:600 holds a cloned working InfraIr and :658 mutates its model for fixed-point simulation. Its four current changes are replicas, resources, probes and a new disruption budget. Replacing the public field with a public mutable accessor would retain the defect.
- Define the smallest checked transformation required by that projector in a binding design before code. A candidate is a detached model edit followed by the existing relational admission into a new IR, leaving the source IR unchanged on failure; compare it with narrow capability methods before choosing. Preserve the single owner of reference validation, total lookups, projector patch/model parity and induced-obligation behavior. Do not expose a general unchecked replacement or mutable borrow.
- The current handle documentation explicitly excludes mixing handles from different IRs. Do not silently expand this story to generative lifetimes or cross-IR identity redesign.
- Existing read.rs fixture resolves all six families and tests rehashed dangling references. Extend positive coverage across compile/read/clone/valid transformation; public compile-fail probes must reject clear, replacement, nested mutation and unchecked construction for the actual owner surface. Preserve canonical infra-ir/1 fixture bytes/digest, independently comparing the old writer where possible rather than only a new self-round-trip.
- infra-project's tests/round_trip.rs applies actual projected patches back to a bundle and recompiles; preserve that check for all supported transformations. A refused transform must leave the previous valid IR usable. No shared CLI collision with wave 3 is authorized.

Independent story-scoper confirmation is still required before selection. A new docs/design/review-infra-ir-invariants.md is an inferred design surface, not an existing cited file.
