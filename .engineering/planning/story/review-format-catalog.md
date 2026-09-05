---
format: aep.planning-md/1
id: story:review-format-catalog
kind: story
status: draft
title: Catalog format identities and canonical byte contracts
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: inferred
  path: docs/design/review-format-catalog.md
- confidence: inferred
  path: website/docs
revision: 3
---
## Finding and source

F13 (P1) from `docs/reviews/2026-09-05-architecture-review.md:449`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/specify/ess-composition/src/lib.rs:277`, `crates/specify/ess-composition/src/lib.rs:548`, `crates/specify/ess-compiler/src/ir.rs:1569`, `crates/generate/ess-deployment/src/identity.rs:128`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Every currently persisted ESS format has a cited catalog entry separating its discriminator, semantic/release version, validation level and canonical digest profile.

## Implementation boundary

Inventory actual producers and readers, supported versions and strictness, including differing composition input/output shapes and realization type discriminator. Name source_digest accurately in prose as a compiled-model digest. Describe existing canonical bytes and inventory relying parties; do not change bytes merely for uniform naming.

## Validation

Cross-check the inventory against format constants/Serde envelopes and readers; canonical fixture examples identify compact versus pretty-plus-newline hashing. Each proposed successor names migration prerequisites and unknowns instead of presenting a new endpoint as shipped.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Schema resource identity has a separate design; no ess-ir/2 or mass format rename.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `docs/design/review-format-catalog.md` — inferred; planned edit surface, verify before dispatch.
- `website/docs` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

