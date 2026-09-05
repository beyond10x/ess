---
format: aep.planning-md/1
id: story:review-glossary-boundaries
kind: story
status: draft
title: Disambiguate ESS logical, interface and delivery concepts
tags:
- P2
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: inferred
  path: docs/design/review-concept-boundaries.md
- confidence: inferred
  path: website/docs
revision: 5
---
## Finding and source

F12 (P2) from `docs/reviews/2026-09-05-architecture-review.md:424`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/specify/ess-domain/src/component.rs:1`, `crates/specify/ess-domain/src/component.rs:103`, `crates/generate/ess-deployment/src/component.rs:27`, `crates/specify/ess-realization/src/lib.rs:1`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

For each F12 concept or entrypoint example, the reference names one logical, interface or delivery owner or an explicit supported limitation.

## Implementation boundary

Use the review's glossary as a proposal, comparing each term with current Rust owners. Separate semantic component/deliverable, composition selection/stack service, requirements/runtime mapping and realization. Decide whether reach/CLI layout is contract-level or implementation-level; for multiple entrypoints, document the current supported limitation and criteria for an additive typed design.

## Validation

Trace the glossary's terms to code and walk CLI, HTTP, composition and delivery examples; no example implies a capability the model lacks. Validate links and run site-build for public doc edits.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No crate rename, universal component registry or semantic/infra merger. Persisted API changes need their own design and migration, not a terminology edit.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `docs/design/review-concept-boundaries.md` — inferred; planned edit surface, verify before dispatch.
- `website/docs` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.


## Boundary-map validation rows

Check semantic component versus deliverable, composition alias versus stack service, requirements versus runtime choices, and realization versus interface layout. Include CLI-only, HTTP-only and simultaneous-entrypoint examples; each must name the owner/disposition or a current unsupported limitation, with a source citation. The interface-ownership decision remains required, not removed by the single-result acceptance.