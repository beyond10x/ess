---
format: aep.planning-md/1
id: story:review-primitive-semantics
kind: story
status: draft
title: Align primitive admission and exact numeric semantics
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-format-catalog
scope:
- confidence: cited
  path: crates/generate/ess-gen
- confidence: cited
  path: crates/generate/ess-synth
- confidence: cited
  path: crates/specify/ess-primitives
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: docs/design/review-primitive-semantics.md
revision: 6
---
## Finding and source

F08 (P1) from `docs/reviews/2026-09-05-architecture-review.md:326`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/specify/ess-primitives/src/facts.rs:33`, `crates/verify/ess-conformance/src/input.rs:561`, `crates/generate/ess-gen/src/types.rs:145`, `crates/generate/ess-synth/src/rust/wire.rs:1`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

The normative primitive corpus produces equivalent admission and comparison results across facts, conformance, generated codecs and schemas without losing promised integer or decimal precision.

## Implementation boundary

Write the abstract-value/predicate-value/wire-encoding matrix first. Decide exact integer/decimal representations and canonical serialization, then migrate named readers/writers with versioned compatibility fixtures. Include UUID and other constrained primitives rather than retaining arbitrary text admission. The identity catalog supports inventory but does not choose new bytes.

## Validation

Cover 2^53 and 2^53+1, i64 extrema, decimals, invalid UUIDs and supported encodings using shared vectors in Rust, generated Go, schemas and browser adapters. Test old/current readers and explicit refusals for unsupported domains.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

This is a semantic migration, not a first-wave refactor; cross-repository adapters are inventory/ADR prerequisites to release.

## Scope

Derived 2026-09-05 by the coordinator from review citations; independently re-scope before future dispatch. Directory tokens cover source and tests within the named package; references used only as evidence are excluded.

- `crates/specify/ess-primitives` — cited; owning implementation or documented surface.
- `crates/verify/ess-conformance` — cited; owning implementation or documented surface.
- `crates/generate/ess-gen` — cited; owning implementation or documented surface.
- `crates/generate/ess-synth` — cited; owning implementation or documented surface.
- `docs/design/review-primitive-semantics.md` — inferred; planned edit surface, verify before dispatch.
- Confidence: medium — inferred; exact package-local test filenames remain an implementation choice.
- Would collide with: stories sharing any of these exact tokens — inferred; see the complete pair list in `docs/plan/2026-09-05-review-remediation.md` before concurrent scheduling.
- Shared integration files: planning journal, wave page and final change record belong to the coordinator — inferred execution assignment.

