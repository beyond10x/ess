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
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/infra/infra-analyze
- confidence: cited
  path: crates/infra/infra-compiler
- confidence: cited
  path: crates/infra/infra-project
- confidence: cited
  path: crates/infra/infra-spec
- confidence: inferred
  path: docs/design/review-infra-ir-invariants.md
revision: 10
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

Derived 2026-09-05 by independent story-scoper at coordinator 7fc7025740796ede6df9ef5c0c30919c12597cb4, story revision 8. Relevant implementation sources are unchanged through e0ea44383f00b05917bc24c828d3120d86afb3de. Every entry below is cited or inferred. Package tokens cover their source/tests.

- **Primary surface:** cited — crates/infra/infra-compiler; ir.rs:535 exposes InfraIr.model, and InfraModel at :463 exposes maps/nested values. External consumers can clear, replace, remove or mutate them after handles were minted.
- **Handles:** cited — ir.rs:113 declares Node, Service, ConfigMap, Secret, ServiceAccount and Claim handles. Constructors are private; accessors at :99 assume target keys remain. The documented :96 boundary excludes mixing different IRs.
- **Smallest privacy boundary:** inferred — private/crate-restricted owning model with model(&self) -> &InfraModel; detached InfraModel fields can remain public. No mutable nested query, unchecked constructor or unchecked replacement.
- **Construction/admission:** cited — compile.rs:409 and read.rs:1247 construct internally; neither InfraIr nor handles derive Deserialize. read_document at read.rs:62 uses closed private mirrors, checks membership at reference :793, remints all six families and returns only without accumulated errors.
- **Reader limitations:** cited — read.rs:19 excludes full domain-value and unresolved-accounting revalidation. Privacy does not repair those other semantics or authorize a silent reader change.
- **Read consumers:** cited — crates/infra/infra-analyze, diagnose.rs:153, graph.rs:345, invariants.rs:140, properties.rs:123; crates/infra/infra-spec, drift.rs:471, facts.rs:108, simulate.rs:438.
- **Mutable consumer:** cited — crates/infra/infra-project, project.rs:600 holds a cloned working InfraIr; reach_fixed_point at :632 simulates it and :658 passes the owning model to mutable apply. tests/secrets.rs:26 also reads the model.
- **Transformation inventory:** cited — Change at project.rs:413 has replicas, container resources, container probes and disruption-budget creation. apply at :1478 and container_mut at :1550 perform them; record at :1334 renders the same Change into artifacts.
- **Checked candidate:** inferred — operation takes &self, edits a detached clone, and returns Result<InfraIr, ValidationErrors> after existing relational admission; preserve provenance and never replace source before validation. Reusing read_document through the existing serialized shape avoids a second validator.
- **Alternative:** inferred — compare four narrow reference-preserving capabilities with broader checked clone/serialization cost in the binding design. Do not import the projector's private Change or reverse the dependency.
- **Projector transaction:** inferred — admit candidate before recording patches/dispositions/progress. A fallible transform must not leave a recorded patch with unchanged model. Settle failure handling under the currently total project API without silently adding a persisted refusal variant.
- **CLI:** cited — crates/edge/ess-cli/src/main.rs:2807 reads actual InfraIr model.unresolved during Kubernetes import reporting and requires query migration. Loader already uses compile/read_document; no loader edit established. Retain the package collision token.
- **Public API verification:** inferred — external compile-fail documentation/probes for clear, replacement, nested mutation, mutation through shared query, unchecked owner construction. Positive controls must compile so unrelated errors cannot satisfy negatives.
- **Handle tests:** cited — infra-compiler tests/read.rs:139 resolves all six families; :171/:192 cover dangling references with rehashed bytes; tests/resolution.rs covers compiled lookup.
- **New handle cases:** inferred — all six lookups after compile/read/clone/valid transformation; reject deleting a still-referenced target; refused transformation preserves original document, digest and obtained handles.
- **Byte tests:** cited — infra-compiler tests/determinism.rs:150 compares documents/reordering/provenance-independent digests; serialization/digest at ir.rs:534/:562/:574. Privacy needs no new persisted field or infra-ir/1 version.
- **Compatibility:** inferred — freeze old-writer model/document bytes and digest before correction, compare new writer/reader against them and the committed cluster IR; a new self-round-trip alone is insufficient.
- **Projector tests:** cited — tests/round_trip.rs:116 applies emitted files to the actual observation bundle, recompiles and compares outcomes/regressions/verdicts. tests/projection.rs covers four changes and induced gaps; tests/determinism.rs:63 compares committed generated artifacts.
- **New projector cases:** inferred — retain round trips for all four changes, especially stated probes, induced obligations, patch/model parity and corruption controls. Run all five package suites, formatting and strict Clippy, then coordinator integration gates.
- **Binding design:** inferred — docs/design/review-infra-ir-invariants.md before code: ownership, transform signature/validation owner, source-on-failure guarantee, projector failure policy, handle ownership and unchanged bytes.
- **Exclusions:** inferred — no infra-domain, scanner, root manifest/workflow or generated-fixture rewrite established. No EssIr merger, cross-IR identity redesign, unresolved-accounting redesign or new persisted format. External Rust consumers were not inventoried; do not claim their compatibility.
- **Confidence:** high for actual owner/constructors/reader/handles/consumer paths; transformation API and new tests remain design choices.
- **Collisions:** infra-compiler, infra-analyze, infra-spec, infra-project, ess-cli main and proposed design; reserve together even though CLI migration is small.

The independent scoper ran no builds and made no file/store/Git/lifecycle writes. Old/new bytes and compile-fail behavior are scoped validation, not executed evidence. The design must choose a transform and settle projector failure before implementation.

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
