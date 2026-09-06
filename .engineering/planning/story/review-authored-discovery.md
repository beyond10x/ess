---
format: aep.planning-md/1
id: story:review-authored-discovery
kind: story
status: draft
title: Define predictable discovery for co-located ESS documents
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:scenarios-directory-compiles-nothing
scope:
- confidence: cited
  path: crates/edge/ess-cli
- confidence: inferred
  path: docs/design/review-authored-discovery.md
- confidence: cited
  path: website/docs/guides/verify-conformance.md
- confidence: cited
  path: website/docs/guides/write-a-specification.md
revision: 6
---
## Finding and source

F10 (P1) from `docs/reviews/2026-09-05-architecture-review.md:365`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-cli/src/load.rs:25`, `crates/edge/ess-cli/src/main.rs:2458`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A documented mixed source/scenario/generated-output layout resolves the same intended authored inputs deterministically without ingesting generated YAML as source.

## Implementation boundary

Specify an explicit input manifest or uniform typed discovery contract before implementation; preserve supported existing layouts or supply actionable migration refusals. Resolve recursion, exclusions, duplicate documents and unknown kinds consistently. Retain the existing zero-authored explicit-path refusal when no scenario document is selected.

## Validation

Fixture the old layouts and a mixed tree with nested authored scenarios and generated YAML; compare resolved input identities over filesystem ordering changes and assert no silent empty success.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

The existing scenarios-directory-compiles-nothing story owns the immediate zero-result refusal and remains a prerequisite.

## Scope

Derived 2026-09-07 by `story-scoper` against ESS `25a1b47486190f24d5928e808d9baae7d6ce6d5e` — cited.

- **Primary surface:** `crates/edge/ess-cli` — cited; owns specification filesystem discovery, both current authored-scenario discovery implementations, their command dispatch and CLI help, and package-local regression tests.
- **Symbols:** `load::specification_files`, `load::specification`, `resolved`, `authored_sources`, `fresh_legacy_run_suite`, `synthesize_suite`, `author_suite`, `conform_web`, `coverage::sources`, `coverage::fresh`, `coverage::generate`, and `coverage::web` — cited; these select, read, or consume the authored input sets.
- **Existing tests:** the authored-scenarios and authored-scenarios-adversary integration matrices, coverage-CLI source identity tests, command-surface alias tests, and model-types/normalization model-input cases within the primary package — cited.
- **Binding:** `docs/design/review-authored-discovery.md` — inferred; the story requires a discovery contract before implementation, and this reserved design path does not exist at the frozen subject.
- **Public specification layout:** `website/docs/guides/write-a-specification.md` — cited; its Layout section owns the existing directory and single-file input guidance and is the concrete place to document the accepted mixed layout.
- **Public scenario discovery:** `website/docs/guides/verify-conformance.md` — cited; it expressly promises shallow authored discovery and documents suite/5 source identities, exact text digests and selected-symlink refusals.
- **Boundary:** imported specification and conformance types remain evidence inputs unless the accepted design changes their parsing, identity or persisted contracts — inferred; filesystem selection can be implemented in the owning CLI package without changing those libraries.
- **Confidence:** medium — inferred; the present owners and callers are established, but choosing a new persisted manifest instead of a package-local discovery contract could introduce additional owners and compatibility work.
- **Would collide with:** any unit changing the ess-cli package, the reserved discovery binding, specification layout guidance or conformance acquisition guidance — cited; the four paths above are the proposed machine-readable reservations.
