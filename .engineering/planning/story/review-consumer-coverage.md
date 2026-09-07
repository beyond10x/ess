---
format: aep.planning-md/1
id: story:review-consumer-coverage
kind: story
status: draft
title: Require explicit consumer coverage for model extensions
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-semantic-diff-coverage
scope:
- confidence: inferred
  path: Cargo.lock
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-xtask
- confidence: inferred
  path: docs/design/review-consumer-coverage.md
revision: 9
---
## Finding and source

F16 (P1) from `docs/reviews/2026-09-05-architecture-review.md:522`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `Taskfile.yml:138`, `crates/edge/ess-xtask/src/main.rs:54`, `docs/reviews/2026-09-05-architecture-review.md:532`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

The extension gate fails when a new semantic construct or field has no tested support or explicit refusal for an inventoried consumer.

## Implementation boundary

Maintain a typed consumer matrix for validation, IR, references, diff/impact, projections, synthesis and conformance, with links to executable cases or explicit unsupported/refusal evidence. Add a Rust xtask gate that checks the matrix against the actual authoritative model surface; avoid a parallel hand-maintained list silently omitting fields. A coverage record is not a substitute for running its behavioral test.

## Validation

Mutation proof: add a representative semantic field/consumer obligation without coverage and observe failure, then supply supported/refused behavior evidence and pass. Include concrete F01 rows. The existing fuzz story owns general document fuzzing rather than this matrix.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Do not rewrite passing gates or claim every target supports every construct; unknown is a valid visible matrix entry with owned follow-up work.

## Scope

Derived 2026-09-07 by `aep-drive:story-scoper` against frozen ESS source d1fe6755e842c8ef5486a90493530a39050dae48 and draft story revision 8 — cited.

- **Primary write reservation:** `crates/edge/ess-xtask` — cited; owns the existing Rust command dispatcher and compiled RawSpecFile JSON Schema provider, and is the story's named location for the model/consumer coverage gate.
- **Gate write reservation:** `Taskfile.yml` — cited; owns the existing projection and repository checks and the new separately named consumer-coverage lane.
- **Dependency write reservation:** `Cargo.lock` — inferred; package-local AST, serialization or provenance dependencies may alter the resolved lockfile without requiring a workspace-root manifest change.
- **Binding write reservation:** `docs/design/review-consumer-coverage.md` — inferred; the exact intended internal binding path remains absent, while the reviewed v3 candidate remains an unaccepted preparation artifact.
- **Model inventory boundary:** RawSpecFile, Specification, EssIr, private EssIrParts, EssSemanticRef and SemanticDependencyGraph, plus their reachable production declarations and the separate compiled RawSpecFile Draft 7 wire graph — cited; the six declaration owners and schema provider are unchanged from the preceding source survey.
- **First-stage implementation:** build the closed source/profile extractor, explicit consumer classification and proposed finite baseline output before enforcing baseline eligibility — inferred; the coordinator must review and freeze that output at a named checkpoint before any BaselineUnknown record can qualify.
- **Integrated consumer refresh:** separately classify suite5 construction, authored batch compilation and merging, admitted original-byte input and selection lineage, Rust/Go coverage execution, paired replay admission and browser playback, and coverage-aware impact input — cited; these integrated surfaces invalidate the old Scope's suite1–4-only account.
- **Behavioral evidence boundary:** each supported/refused cell identifies its exact model shape, consumer profile and actually executed case; model names, fixture presence, package-level green results and outer tests that skipped nested execution do not establish coverage — cited.
- **Conditional scope expansion:** add exact owner-package test reservations only if mandatory behavioral cells cannot be established by existing assertions and observable executions; no such test edit is selected by this refresh — inferred.
- **Compatibility:** preserve existing source-model visibility, persisted formats, source defaults, browser fidelity limits and the composition byte-buffer contract — cited.
- **Validation:** retain the repository gate and separate site-build requirement for the validation-workflow change; include extractor/accounting controls and same-source behavioral mutation evidence under the accepted binding — cited.
- **Confidence:** medium — inferred; the four write surfaces are established, but the reachable inventories, exact eligibility set, case attribution and execution-profile mechanics have not been implemented or measured.
- **Would collide with:** any unit changing the xtask package, Taskfile validation sequence, dependency lockfile or exact internal binding document — inferred; consumer/model/test owners are read dependencies rather than write reservations, but changes to their bound inputs require a refreshed inventory and evidence checkpoint.

## Candidate binding review

The coordinator accepted the four refreshed write reservations on 2026-09-06 and applied only
this Scope replacement; the story remains draft. The independent report is
target/review-boundaries-11/next-scope/consumer-coverage-report.md, SHA256
2194fea88d194d9c54b22e38f5990d6e2b00b64ed633ac039ebf833ca29abd61. Root independently
verified all 53 inputs and 49 opening Git blobs. Two document-only reviews are recorded as
review-result:consumer-coverage-binding-pass1 and review-result:consumer-coverage-binding-pass2;
verification-report:consumer-coverage-binding-wording records the narrow correction to actual
Draft7 definitions/#/definitions/ vocabulary. The current candidate is
target/review-boundaries-11/next-scope/consumer-binding-draft-v3.md, SHA256
63d6075781dc7f348fff838962c4392df3c8e9335eaf7b5d06f8a690f630812c. Its two-stage baseline
selection, exact wire/model inventory and actual attributed case execution remain requirements,
not measured gate results. Before implementation selection, refresh integrated coverage source
and case/profile identities, accept the binding and select the concrete first-stage work order.
No new story selection or baseline-unknown eligibility approval is implied by this scope update.

## Coverage-integrated scope refresh

The original independent report is retained at
`target/review-boundaries-12/preparation/scopers/consumer-report.md`, SHA256
`76b42814f61e3d7603e5e934b38f82ed9dda518478d60e6b1d46b636caccf7b1`. Root verified all 65
read inputs and 59 frozen Git blobs before applying this Scope. The same four machine
reservations remain. Its proposed first-stage profiles include coverage construction, original
input and lineage admission, paired replay, actual Rust/Go execution and coverage impact.
This source inspection does not accept the binding or any BaselineUnknown eligibility.
The story remains draft and is not selected by wave 12. Later selection must refresh any
intervening model, consumer, test or profile changes, including the pending cache implementation.
