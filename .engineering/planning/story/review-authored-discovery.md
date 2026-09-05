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
revision: 3
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

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/edge/ess-cli` — cited; owning implementation or documented surface.
- `docs/design/review-authored-discovery.md` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

