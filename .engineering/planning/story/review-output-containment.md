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
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/generate/ess-gen
- confidence: cited
  path: crates/generate/ess-gen/src/artifact.rs
- confidence: cited
  path: crates/generate/ess-gen/src/document.rs
- confidence: cited
  path: crates/generate/ess-gen/src/html.rs
revision: 8
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

Derived 2026-09-05 by `story-scoper` from the complete story, its parent epic, review F10 and implementation at `3d8d6c6b287ce1c462cc50ea74f1ba5c171b827b`. Every line is cited or inferred.

- **Primary surface:** `crates/edge/ess-cli` — cited; owns filesystem writes and caller-controlled included page identities.
- **Secondary surface:** `crates/generate/ess-gen` — cited; owns artifact paths, page identities, rendering and duplicate-path detection.
- **CLI file:** `crates/edge/ess-cli/src/main.rs` — cited; `write_artifacts` (2016), `included` (2054), `generate` (2063), `synthesize` (2137) and `conform_web` (2368). The special site branch collects rendered artifacts directly into a map before writing.
- **Artifact file:** `crates/generate/ess-gen/src/artifact.rs` — cited; public `Artifact` fields, `Artifact::new`, `Artifact::sliced`, `run` and `DuplicatePath`. Current duplicate checking compares exact strings only.
- **Page file:** `crates/generate/ess-gen/src/document.rs` — cited; public `PageId(String)`, `Document::new`, public page collections and derived deserialization currently permit unchecked identities.
- **Site file:** `crates/generate/ess-gen/src/html.rs` — cited; `Site::render` (187) and `Site::page` (214) convert page identities into output paths and append assets.
- **Tests:** regression cases within the two package scopes — inferred; CLI subprocess fixtures for traversal, duplicate includes/generated pages, symlink parents and destinations, unchanged sentinels and valid nested paths; library tests for artifact/page validation and collisions before map collection.
- **Additional sinks within the CLI scope:** `compose` (1871), `synthesize_suite` (2300), `write_projection_files` (2725), and `project_kubernetes` (2997) each independently join generated relative paths to an output root — cited; they can adopt common preflight without changing their producer packages.
- **Documents:** source API documentation should state accepted portable path forms, treatment of pre-existing symlinks and the exclusion of concurrent filesystem replacement attacks — inferred; a new typed construct would additionally require its binding design before implementation.
- **Confidence:** high — cited; the review names the defect sites and the complete rendering-to-filesystem call chain was read.
- **Would collide with:** changes to either package, particularly CLI dispatch/output handling, generation validation and document rendering — inferred; do not schedule another unit owning either package concurrently.

## Scoping decisions

The independent scoper confirmed the two-package boundary and found two places that need preflight before map construction: duplicate site pages can already have been discarded, and map keys carry a site/ prefix absent from Artifact.path. Validate actual destinations while preserving valid layouts.

Coordinator inference for the future brief: reject noncanonical portable paths rather than silently normalize, detect aliases and ancestor/file collisions, and route the confirmed CLI generated-tree sinks through shared preflight. The implementation must state symlink/root handling and concurrency exclusions, rather than claim race-proof isolation. Additive checked APIs plus mandatory sink checks can preserve current infallible public constructors without widening to unrelated producer packages; the implementor must verify this mechanism with a red case. Exact platform and hard-link policy remains to be documented in that implementation.