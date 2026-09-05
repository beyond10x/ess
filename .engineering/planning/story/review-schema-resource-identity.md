---
format: aep.planning-md/1
id: story:review-schema-resource-identity
kind: story
status: draft
title: Define the boundary between generated schemas and registry resources
tags:
- P2
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-format-catalog
scope:
- confidence: inferred
  path: docs/design/review-schema-resource-identity.md
- confidence: inferred
  path: website/docs
revision: 3
---
## Finding and source

F13 (P2) from `docs/reviews/2026-09-05-architecture-review.md:449`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-xtask/src/main.rs:423`, `crates/generate/ess-gen/src/schema.rs:154`, `crates/generate/schema-contract/src/typescript.rs:69`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

The schema workflow documents an executable supported path from generated syntax/contract schemas to adopter-owned registry entries without implying an existing stable schema endpoint.

## Implementation boundary

Design/document the current boundary first: syntax schema is not whole-system semantic validation, self-contained generated contracts are not automatically registry resources. Decide whether a concrete consumer needs logical immutable IDs or digest identities; record owner, collision rules, offline resolution and publication prerequisites before adding generated $id values. Provide a worked local registry example using an adopter-chosen absolute ID if that is the supported path.

## Validation

Validate the worked example with the existing schema registry CLI, including missing-id refusal and offline resolution. Mark any proposed organization namespace as unshipped.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No new public schema service or silent changes to released schemas; any automated ID projection is follow-on work only after ownership is decided.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `docs/design/review-schema-resource-identity.md` — inferred; planned edit surface, verify before dispatch.
- `website/docs` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

