---
format: aep.planning-md/1
id: story:review-cache-origin
kind: story
status: draft
title: Verify cached bundle bytes against their OCI identity
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-persisted-delivery-validation
scope:
- confidence: cited
  path: crates/edge/ess-cli
revision: 2
---
## Finding and source

F11 (P1) from `docs/reviews/2026-09-05-architecture-review.md:394`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-cli/src/main.rs:1457`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A self-consistent cached bundle substituted under another OCI digest is rejected or refetched before use.

## Implementation boundary

Retain/revalidate enough manifest and layer bytes to verify the cache key-to-content chain at cache hits. Treat old cache entries lacking proof as misses or explicit trusted-local-only input; do not silently upgrade local consistency into registry identity. Keep external fetch clients injectable.

## Validation

Fake registry/process fixtures cover correct manifest/layers, wrong manifest digest, altered layer, self-consistent replacement bundle and incomplete old cache; test cold and warm paths with no live registry.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

This establishes content binding, not publisher signature trust.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/edge/ess-cli` — cited; owning implementation or documented surface.
- Confidence: high — cited; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

