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
scope:
- confidence: cited
  path: crates/specify/ess-compiler
- confidence: cited
  path: crates/verify/ess-diff
revision: 3
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

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/verify/ess-diff` — cited; owning implementation or documented surface.
- `crates/specify/ess-compiler` — cited; owning implementation or documented surface.
- Confidence: high — cited; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

