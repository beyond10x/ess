---
format: aep.planning-md/1
id: story:review-conformance-format-design
kind: story
status: draft
title: Design the conformance suite and report migration
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: inferred
  path: docs/design/review-conformance-coverage.md
revision: 2
---
## Finding and source

F03 at `docs/reviews/2026-09-05-architecture-review.md:217`; the current v1 report is `crates/verify/ess-conformance/src/evidence.rs:18`. The scoper independently found different Rust error/unsupported and Go skipped vocabulary.

## Acceptance

A binding migration design specifies unambiguous old/current suite and report semantics for every current producer state and coverage category before a new writer is implemented.

## Required decisions

Name failure, skipped, error and unsupported counts and list semantics without inventing equivalence; preserve aggregate execution verdicts. Define exact-suite digest canonicalization and generated/authored/outside/refused coverage accounting, version-specific validation and strict-mode behavior. Inventory every relying reader and write the staged migration order, including the independently owned AEP adapter. Existing v1 meanings and valid bytes remain frozen.

## Validation

Review a matrix of Rust/Go producer states, complete/partial/empty suites, old/new readers and mismatched digests; every row has one explicit expected result. Identify the Atlas ADR needed for cross-repository byte changes before a new default is released. This is design evidence, not implementation evidence.

## Scope

- `docs/design/review-conformance-coverage.md` — inferred; new binding design only.
- Confidence: high — cited; the acceptance is wholly a document.
- Would collide with: any update to that design page — inferred.
- Source and consumer files are evidence references, not implementation scope.

## Ownership

This story owns the common design. The existing skipped-count story owns producer count migration; review-conformance-coverage owns durable coverage and exact-suite binding. The narrow v1 reader validation needs neither design nor a format change.

