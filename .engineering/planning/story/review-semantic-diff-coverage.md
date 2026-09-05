---
format: aep.planning-md/1
id: story:review-semantic-diff-coverage
kind: story
status: draft
title: Propagate every reviewed semantic change into diff and impact
tags:
- P0
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- informed_by: obligation:review-contract-rollout-coordination
scope:
- confidence: cited
  path: crates/specify/ess-compiler
- confidence: cited
  path: crates/verify/ess-diff
- confidence: inferred
  path: docs/design/ess-semantic-diff-impact-evolution-design-v0.1.md
- confidence: inferred
  path: generated/asyncapi
- confidence: inferred
  path: generated/docs
- confidence: inferred
  path: generated/openapi
- confidence: inferred
  path: generated/schema
- confidence: inferred
  path: generated/site
revision: 10
---
## Finding and source

F01 (P0) from `docs/reviews/2026-09-05-architecture-review.md:160`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/verify/ess-diff/src/diff.rs:1000`, `crates/specify/ess-compiler/src/graph.rs:550`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Each mutation of relations, component reach/CLI, outcome sets/refusals or view parameters/order produces a semantic delta with conservative artifact obligations instead of the current narrowed-empty result.

## Implementation boundary

Add comparison and dependency coverage for every omission enumerated in F01, including relation targets. If model differences remain unclassified, return a conservative unknown/full obligation result using existing vocabulary where possible. Distinguish known presentation-only/canonical equivalence from semantic differences; an unexplained digest mismatch must never prove no work is owed.

## Validation

Replay the billing relation-cardinality mutation and independent mutations for every listed field; assert changed paths, dependency closure and generated artifact obligations, plus an unchanged control and documented normalization controls. Run the diff and compiler package suites and compare generated contract differences.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No source_digest byte change or new semantic format without the separate identity design. review-consumer-coverage owns the long-term matrix; this story owns the concrete missing comparisons.

## Scope

Derived 2026-09-05 by `aep-drive:story-scoper`; source and callers inspected read-only at `3d8d6c6b287ce1c462cc50ea74f1ba5c171b827b` — cited.

- **Primary surface:** `crates/verify/ess-diff` — cited; comparison, typed change vocabulary, canonical reader/rendering and conservative impact fallback. Owning files are `src/diff.rs`, `src/change.rs`, `src/impact.rs`, with package-local canonical, family, graph and artifact tests.
- **Dependency surface:** `crates/specify/ess-compiler` — cited; `src/graph.rs` owns `SemanticDependencyGraph`, `DependencyRelation`, `walk_entities`, `walk_views` and `walk_components`; `src/ir.rs` establishes relation carriers and resolved references.
- **Symbols:** `compare_entities`, `component_changes`, `outcome_changes`, `compare_views`, `uncompared_families`, `WholeAnswer::UncomparedFamilyChanged`, `EntityChange`, `ComponentChange`, `CommandChange`, `ViewChange` — cited.
- **Documents:** `docs/design/ess-semantic-diff-impact-evolution-design-v0.1.md` — inferred; record the additional typed comparisons, normalization rules, dependency direction and format compatibility decision before extending persisted change/edge vocabulary.
- **Generated outputs:** `generated/schema` — inferred; corrected graph closure changes contract digests stamped on affected entity schemas, including the normative billing relation.
- **Possible generated outputs:** `generated/docs`, `generated/site`, `generated/openapi`, `generated/asyncapi` — inferred; these generators also use dependency-closed provenance. Regenerate through the existing Rust task and retain only actual differences; no generator rewrite is established.
- **Confidence:** high for the two implementation packages; medium for the exact generated-file set because regeneration was outside this read-only task — cited.
- **Would collide with:** changes to semantic comparisons, canonical change types, compiler dependency walks, the binding semantic-diff design, or the listed generated projection subtrees — inferred.

## Scoping decisions and open compatibility work

The independent scoper found reverse relation-carrier dependencies: `EssIr::relations_carried_by` at `crates/specify/ess-compiler/src/ir.rs:1408` places an owns annotation on the target field, consumed by `crates/generate/ess-gen/src/types.rs:905`. A forward source-to-target edge alone is insufficient. View parameter types and both grouped and top-level CLI view references also require graph coverage.

The generator's `ProvenanceMint::digest_of` at `crates/generate/ess-gen/src/provenance.rs:249` already closes seeds through the graph. No generator, CLI or xtask source repair is established; generated output changes must be measured. Mixed classified/unclassified edits need conservative fallback, while parsed-equivalent predicates and explicit naming defaults retain normalization controls.

Compatibility remains unresolved before implementation: adding serialized change kinds or dependency relation names requires a binding format decision, and corrected graph slices can change contract digests even when source_digest stays unchanged. Coordinator inference: this story cannot be dispatched as an assumed byte-preserving repair. Resolve the design and relying-reader consequences under `obligation:review-contract-rollout-coordination` before new vocabulary or default writers; do not disguise semantics under unrelated existing variants.