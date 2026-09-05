---
format: aep.planning-md/1
id: story:review-observation-completeness
kind: story
status: draft
title: Preserve observation scope and selector uncertainty
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-secret-sanitization
- depends_on: story:review-infra-ir-invariants
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/infra/ess-kubernetes
- confidence: cited
  path: crates/infra/infra-analyze
- confidence: cited
  path: crates/infra/infra-compiler
- confidence: cited
  path: crates/infra/infra-domain
- confidence: cited
  path: crates/infra/infra-project
- confidence: cited
  path: crates/infra/infra-spec
- confidence: inferred
  path: docs/design/review-observation-completeness.md
- confidence: inferred
  path: examples/k3d-dev-cluster
revision: 11
---
## Finding and source

F06 (P1) from `docs/reviews/2026-09-05-architecture-review.md:286`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/infra/infra-domain/src/raw.rs:159`, `crates/infra/ess-kubernetes/src/lib.rs:70`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Unsupported selectors or a reduced collection scope produce qualified unknown/refused infrastructure conclusions instead of complete match-all or absence claims.

## Implementation boundary

First design typed scope/per-kind completeness and unsupported-term accounting against the existing observation/IR formats, with strict old-reader fixtures. Preserve selector semantics or refuse their use; never reduce an unrecognized expression-only selector to a broad empty conjunction. Retry only when scope is preserved or visibly acknowledged; ensure scope qualifications reach analysis, drift and projection.

## Validation

Fake kubectl fixtures distinguish permission failure, namespace fallback and genuinely scope-preserving retries. Selector fixtures cover matchLabels, matchExpressions, mixed and unknown terms. Round-trip incomplete observations and assert consumers retain uncertainty.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No live cluster needed; keep the separate infra bounded context and obtain an explicit format decision before changing persisted fields.

## Scope

Derived 2026-09-05 by independent aep-drive:story-scoper from revision8, F06 and source at170fdfa1f3061af33f4de558d22de4711ab6194d. No writes, builds, live cluster or compatibility execution occurred during scoping.

- crates/infra/ess-kubernetes — cited; lib.rs:70 narrows every failed -A request to default namespace; :95 persists no requested/effective scope or per-kind completion. The17-kind inventory is collection configuration, not completeness evidence. Extend existing Rust fake-command tests while preserving Secret/diagnostic guards.
- crates/infra/infra-domain — cited; raw.rs:24 admits permissive observation; :159 drops structured selector terms except matchLabels. workload.rs:308 and policy.rs:69 collapse absent selectors to empty maps. Carry typed collection qualifications through observation.rs:297 and preserve absent optional kinds versus present empty collections.
- crates/infra/infra-compiler — cited; compile.rs:270 derives selector unresolved facts from observed pod absence and :294 onward other targets. ir.rs:383/463/517/562 owns unresolved/model/provenance/digest; read.rs:62 owns strict persisted admission. Qualifications need compilation, serialization, digest policy and reload. Unresolved means not observed, not proven absent.
- crates/infra/infra-analyze — cited; shared pdb_covers diagnose.rs:496 treats empty conjunction as universal; :511 can claim no observed matches guards nothing. properties.rs:123, invariants.rs:182 and graph.rs:571/769 use populations/matching/ownership. Qualify conclusions requiring complete populations while retaining positive witnesses.
- crates/infra/infra-spec — cited; simulate.rs:58 has unknown reasons, but :435 interprets missing workload as absence and :664 treats optional-kind presence as coverage. facts.rs:194/279 handles unresolved/unscanned kinds. drift.rs:458/517 derives removals after context checks without comparing collection scope.
- crates/infra/infra-project — cited; project.rs:639/694/739 mainly summarizes undecidable gaps; :1009 builds PDBs from workload selector maps and :1357 emits matchLabels. Persist actionable uncertainty and prevent patches based on unsupported selectors or unproven absence. Preserve qualifications through hypothetical transformations.
- crates/edge/ess-cli — cited; load.rs:138 dispatches only current observation/IR versions; main.rs:2797 writes IR, :2815 reports empty Kubernetes coverage/refusal lists and :2823 hardcodes infra-ir/1. Migration requires truthful import reporting, help and persisted CLI regressions.
- docs/design/review-observation-completeness.md — inferred; settle authority, requested/effective scope, per-kind completion, selector support/refusal, legacy reads, format/digest consequences, consumer qualification and rollout before code.
- examples/k3d-dev-cluster — inferred; simulation/drift determinism tests at infra-spec/tests/determinism.rs:45 and projection tests at infra-project/tests/determinism.rs:63 pin real products. Legacy-unknown policy may change derived products; keep historical observation evidence and never label it retrospectively complete.
- Dependencies: published Secret7130468ce5baa8178407bc02df557f27a7cae7ee and diagnosticb26829a571c0569ba2f63a5da495b987397b43a4 guards stay binding. Implement against the finalized InfraIr privacy/checked-transform interfaces and preserve qualifications through them — cited.
- Define requested versus effective namespaces/filters and complete-within-scope, incomplete, unscanned, legacy-unknown. Discovery only establishes available kinds, never successful collection. Nonempty Kubernetes continue means more results;17 sequential requests are not one atomic snapshot — cited primary API concepts: https://kubernetes.io/docs/reference/using-api/api-concepts/ .
- Smallest acquisition repair removes arbitrary -A fallback: refuse without destination replacement, or allow explicitly chosen narrower scope with persisted qualification. Same-scope retry is separate. Sanitized status cannot classify permission versus transient failure; never restore raw stderr for classification — inferred.
- Smallest selector repair can refuse unsupported expression/mixed/unknown terms before Serde discards them. If supported, one shared evaluator owns conjunction/operators. Service selectors remain direct equality maps — cited primary labels contract: https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/ .
- Preserve enough context for absent/null/empty selectors and API versions or refuse ambiguity. PDB policy/v1beta1 versus policy/v1 empty selector semantics differ; current raw types discard apiVersion — cited primary PDB docs: https://kubernetes.io/docs/tasks/run-application/configure-pdb/ .
- Choose operation-level refusal or precise scoped analysis. Missing workload/object, no PDB coverage, zero matching pods, universal invariants and drift removals require applicable coverage. Observed objects/matches remain useful under partial collection. A missing key inside an observed ConfigMap/Secret differs from an unobserved object — inferred.
- Observation/1 permits unknown fields but checks format; silently adding qualifications lets old readers ignore them. New envelope version and deliberate legacy-unknown admission required. infra-ir/1 mirrors are strict and remint handles/check digest but do not rederive all unresolved facts; bind new evidence validation/digest placement. Decide every affected graph/simulation/drift/projection/1 consequence without rewriting historical meanings — cited/inferred.
- Reader-first: new ESS readers before scanner writer; retained old readers must reject new versions and new readers must distinguish legacy-unknown/new qualified documents. Same-version roundtrip is insufficient; CLI dispatch must support the transition — inferred.
- Actual maintained writer is ess-kubernetes; observation readers are infra-domain through CLI/helpers; IR reader infra-compiler::read_document feeds typed consumers. k3d products are real byte consumers. Archived infra-scout c618843a4295e8fb414125a0a4d1115a2cc46b95 is a historical writer, not an active parser to migrate. No active parser found in bounded AEP/entity-runtime/service-sdk inspection; external installed readers unknown — cited.
- Atlas historical ADR0017 and archived scout instructions treat bundle changes as coordinated. Refresh Atlas authority and record actual relying parties/order in ADR before new defaults; no ESS→AEP dependency — cited/inferred.
- Red-first: fake argv/populations for all-namespace failure followed by apparent narrower success; refusal/qualification/destination assertions plus identical-scope retry control. Preserve all secret/diagnostic sentinels and mutation guards. Exercise selector terms, same objects under complete/incomplete scope, absence/PDB/pod/unresolved/drift/projection through observation→IR→persisted reload→consumer, retaining reasons rather than only counts — inferred.
- Byte checks: separate frozen legacy and new fixtures; deterministic new outputs and real example products. Comments naming cargo xtask infra are stale; current xtask lacks that command. Use actual package tests and authorized CLI generation. Coordinator owns full/site gates — cited/inferred.
- Confidence high for actual affected owners and consumers; new schema/example migration remain design decisions. Exact package/design/example tokens determine collisions; planning/lifecycle root-owned — cited/inferred.
