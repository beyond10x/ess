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
- confidence: inferred
  path: crates/generate/ess-gen
- confidence: cited
  path: crates/specify/ess-compiler
- confidence: cited
  path: crates/verify/ess-diff
- confidence: cited
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
revision: 14
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

Derived 2026-09-05 by independent story-scoper at ESS 6685b1991bdee19b28316ccb531a4dbfa1c20f1d; relevant sources remained identical through f98db6bd2d31ce3c81cccf00c3fa8b8d6a9fe806. Every line is cited or inferred.

- **Primary surface:** crates/verify/ess-diff — cited; diff.rs comparisons, change.rs closed persisted vocabulary/derived IDs, delta.rs format, raw.rs checked reads and impact.rs residual fallback/artifact obligations.
- **Comparison omissions:** entity relations, component reach/CLI surface, outcome sets/refusal behavior, view parameters/ranking order — cited at diff.rs:742,1000,1210,1343 against resolved IR fields.
- **Current contracts:** ess-diff/1 supports major 1 with checked RawEssDelta conversion; ess-impact/2 embeds that delta and dependency edges, with no impact deserializer/parser — cited at delta.rs:13, raw.rs:44,67 and impact.rs:96.
- **Residual accounting:** handle unexplained differences even beside classified changes, preserving parsed-predicate equality and naming-default equivalence — inferred; uncompared_families currently checks only conversions, domain naming and workloads.
- **Dependency surface:** crates/specify/ess-compiler — cited; graph.rs owns relation vocabulary and entity/view/component walks, closure and slice; ir.rs owns resolved semantic fields.
- **Dependency direction:** dependent to dependency, reverse impact closure and forward artifact slice; analysis uses before/after graph union — cited at graph.rs:312,358.
- **Relation edges:** include source-to-target relations and the reverse dependency required by owns target-field annotations — inferred from ir.rs:1408 relations_carried_by and generator types::carried.
- **Additional graph omissions:** view parameter types and top-level/grouped CLI view references — cited; walk_views reads source/output fields and walk_components reads owns/accepts/publishes. CLI command placement needs comparison but commands already occur in accepts.
- **Persisted edge vocabulary:** DependencyRelation has 21 variants and an ALL reachability guard — cited; new serialized relation names need an explicit impact format decision.
- **Provenance surface:** crates/generate/ess-gen — inferred if versioned digest membership or stamp interpretation is required; provenance.rs:248 closes seeds through the compiler graph and :411 reads unversioned source/contract stamps — cited.
- **Frozen identity:** source_digest at ir.rs:1575 hashes compact serialized IR; preserve its algorithm and bytes — cited. Whole-model contract hashing bypasses graph slicing, so membership correction alone need not change suite digests.
- **Binding design:** docs/design/ess-semantic-diff-impact-evolution-design-v0.1.md — cited existing file introduced at a06f2a7e49968c4af8113e6fed8f933282704ad3; extend or explicitly supersede its relevant decisions and stale AEP heading, rather than treating its proposed formats as current contracts.
- **Generated reservations:** generated/schema, generated/docs, generated/site, generated/openapi, generated/asyncapi — inferred; retain only measured regeneration differences. A broader stamp envelope change needs its additional target scopes before dispatch.
- **Verification surface:** tests/families.rs, graph.rs, impact.rs, artifacts.rs, canonical.rs and revision_pair.rs in ess-diff — cited; add F01 mutations, relation-carrier propagation, mixed residual differences, normalization controls and version-specific reader vectors.
- **Cross-repository prerequisite:** Atlas ADR naming actual relying parties, version/profile decisions, compatibility evidence and order — cited current clean Atlas 9f3b42f6d990d849be918936039d7dd5567653c8 AGENTS Cross-repo changes.
- **Confidence:** high for current contracts, omissions and direct callers; medium for final migration surface until provenance/version design is settled — cited.
- **Would collide with:** comparison/change types, delta reads, compiler graph, provenance interpretation, existing design and measured generated subtrees — inferred. Atlas and downstream changes require separately managed units and coordinator-owned store writes.

## Scoping decisions and open compatibility work

The independent scoper found reverse relation-carrier dependencies: `EssIr::relations_carried_by` at `crates/specify/ess-compiler/src/ir.rs:1408` places an owns annotation on the target field, consumed by `crates/generate/ess-gen/src/types.rs:905`. A forward source-to-target edge alone is insufficient. View parameter types and both grouped and top-level CLI view references also require graph coverage.

The generator's `ProvenanceMint::digest_of` at `crates/generate/ess-gen/src/provenance.rs:249` already closes seeds through the graph. No generator, CLI or xtask source repair is established; generated output changes must be measured. Mixed classified/unclassified edits need conservative fallback, while parsed-equivalent predicates and explicit naming defaults retain normalization controls.

Compatibility remains unresolved before implementation: adding serialized change kinds or dependency relation names requires a binding format decision, and corrected graph slices can change contract digests even when source_digest stays unchanged. Coordinator inference: this story cannot be dispatched as an assumed byte-preserving repair. Resolve the design and relying-reader consequences under `obligation:review-contract-rollout-coordination` before new vocabulary or default writers; do not disguise semantics under unrelated existing variants.
## Published consumer inventory and design decisions

The scoper inspected advertised remote objects for 35 sibling repositories without changing any checkout, source, store or lifecycle. No sibling executable delta/impact parser was found. This is bounded source inspection, not proof about external adopters or executed compatibility.

- ESS raw.rs:44,67 is the actual checked delta reader. A dual-version reader can precede a new writer, with v2-only variants rejected when labeled v1. Existing delta bytes, IDs, order and interpretation remain frozen.
- ESS CLI main.rs:1959,1984 recomputes diff/impact from models; it does not read persisted delta/impact. impact.rs:1035,1060 actually verifies persisted artifact stamps via ess-gen::Provenance::read_digests. xtask main.rs:571,579 reads documentation index stamps. None establishes an impact reader.
- ESS conformance scenario.rs:236 and impact.rs:797 consume suite provenance; current suite writer is v4 and the declared support range is v1-v4. SuiteFormat deserialization at scenario.rs:423 only parses syntax; this is not proof that execution refuses unsupported future versions (the conformance design story owns that admission gap). Graph membership alone does not change whole-model hashing. Keep this separate from the conformance migration.
- Service SDK published 48833c6d14ec37cb3b614fca05cf7dd78f63b743, service-builder/src/lib.rs:87,120 and tree.rs:64,86, calls generators and checks complete output bytes; it is not a delta/impact or stamp parser. Its ESS pin is d1a66772a91b5411d942d7a45bbf08dfc5de4651 (0.13.1). service-runtime-ir/src/lib.rs:367, service-conformance/src/lib.rs:238 and service-connectors/src/lib.rs:288 use source identity, which remains frozen.
- Agentide published 176a57f58457a7c16f105584c66964263b3c2e41, agentide-xtask/src/main.rs:1061, recompiles and compares canonical IR; no delta/impact reader was found. Direct ESS tag 0.9.2 resolves to 6ef4af76b99a8d2cd861a3cc76140c88c1361129, and Service SDK transitively uses d1a6677. Atlas observed pin records were stale relative to these exact manifests/locks.
- AEP published 00c742e4179593738a2e8aa69e2ecc07d3c89402, aep-ess-evidence/src/lib.rs:15,25, reads ess-conformance-report/1 and spec_digest. No delta/impact parser was found. ESS acquires no AEP dependency.
- Agentplugins 2a08a69e4265783f041ba344113ceb47556cd090 has ESS process guidance; Website 9b5a64f8af5929077e06b5b0da5f5c2c43b69608 and Atlas 9f3b42f have documentation lock/catalog consumers. These are not semantic report readers.

Coordinator design work must settle new delta and impact versions (proposed ess-diff/2 and ess-impact/3), version-gated vocabulary, and distinction between corrected graph membership and legacy unversioned stamps. A profile/versioned stamp or explicit legacy recognition with conservative regeneration needs a binding decision; neither may silently reinterpret old stamps. A legacy writer must refuse changes it cannot express. The staged order is Atlas ADR, dual-version delta reader/necessary provenance admission, actual compatibility vectors and downstream byte checks, new default writers, controlled regeneration and Atlas shipping log. Do not invent an impact reader to claim reader-first work where none exists.

The residual comparator must account for mixed classified/unclassified changes, projection shape and external references while preserving established predicate/naming equivalences. The exact changed generated files and external adopters remain unverified until implementation experiments run. Refresh stale Atlas observations through its owner when publishing the coordinated migration, not by editing catalog records here.
