---
format: aep.planning-md/1
id: story:review-output-containment
kind: story
status: draft
title: Validate output paths and page uniqueness before writing
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
  path: crates/generate/ess-gen
revision: 3
---
## Finding and source

F10 (P0) from `docs/reviews/2026-09-05-architecture-review.md:365`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-cli/src/main.rs:2016`, `crates/edge/ess-cli/src/main.rs:2054`, `crates/edge/ess-cli/src/main.rs:2088`, `crates/generate/ess-gen/src/document.rs:87`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

An escaping or colliding artifact/page path is refused before any generated output changes inside or outside the requested root.

## Implementation boundary

Preflight the entire artifact set at the filesystem sink and validate public page/artifact constructors where feasible. Cover traversal, absolute and platform-prefixed forms, normalized duplicates, included/generated collisions and pre-existing symlink parents/destinations. Define the supported containment threat model explicitly; preflight alone must not claim race-proof filesystem isolation.

## Validation

Reproduce the ../../escaped include in a temporary fixture; sentinel files inside/outside root stay unchanged. Add duplicate include/generated-page and symlink escape cases and valid nested-path control; test normal writers and the special site map path.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Atomic replacement, owned-file retirement and input discovery belong to separate F10 stories; no public site publication occurs during this fix.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/edge/ess-cli` — cited; owning implementation or documented surface.
- `crates/generate/ess-gen` — cited; owning implementation or documented surface.
- Confidence: high — cited; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

