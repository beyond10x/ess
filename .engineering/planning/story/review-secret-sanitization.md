---
format: aep.planning-md/1
id: story:review-secret-sanitization
kind: story
status: implemented
title: Refuse malformed Secret shapes before serialization
tags:
- P0
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/infra/ess-kubernetes
revision: 7
---
## Finding and source

F05 (P0) from `docs/reviews/2026-09-05-architecture-review.md:268`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Current source: `crates/infra/ess-kubernetes/src/lib.rs:107`, `crates/infra/ess-kubernetes/AGENTS.md:15`. Observations are attributed to that review.

## Acceptance

Every synthetic malformed Secret response in the boundary corpus is refused without emitting its sentinel value to observation bytes or diagnostics.

## Implementation boundary

Validate the Secret collection, items, data/stringData shapes, metadata and annotations before retaining output; valid maps keep the existing digest/length representation and last-applied annotations stay removed. Prefer strict refusal for malformed fields, with value-free diagnostics. Do not silently coerce unexpected values to an empty string. No live credentials or cluster are needed.

## Validation

Use a fake kubectl and synthetic in-memory sentinels covering string, number, null, array, object and nested malformed cases; assert no output replacement on refusal and byte-stable valid observations. Run `cargo test --locked -p ess-kubernetes`; mutate the sanitizer guard, observe the named test fail, then restore it as the package AGENTS.md requires.

## Compatibility and exclusions

Collection scope/retry semantics belong to review-observation-completeness (F06); do not change the collected kind list or observation format.

## Scope

Confirmed at implementation head `7130468ce5baa8178407bc02df557f27a7cae7ee` from the implementor's confirmation table, final diff and package runner outputs.

- **Primary surface:** `crates/infra/ess-kubernetes` — cited; all implementation and test edits remained inside the approved package.
- **Implementation:** `src/lib.rs`, `sanitize_secret_list` — cited; item, data/stringData, metadata and annotation shape validation, with fixed value-free diagnostic text.
- **Tests and fixtures:** `tests/secret_boundary.rs`, `tests/fixtures/fake_command.rs`, `tests/fixtures/valid-observation.json` — cited; the proposal's inferred package-local corpus/fake-process work is now confirmed. The original two library tests remain.
- **Documents:** no package document edit — cited; the implementor confirmed that the existing credential contract covers this repair. Engineering reports and planning remain coordinator-owned.
- **Scope correction:** none of the proposal's inferred package or document boundaries proved wrong — cited; exact test placement is now recorded instead of left inferred.
- **Limits:** kind list, retry policy, observation format and valid canonical bytes remained unchanged — cited; the adversarial stderr finding was replayed on the baseline and filed as `story:review-kubectl-diagnostic-sanitization`.
- **Confidence:** high — cited; final committed source and tests establish the scope.
- **Would collide with:** any unit touching `crates/infra/ess-kubernetes` — inferred; retain the existing crate-directory typed scope for future computation.

## Scoping decisions

Coordinator clarification: absent optional fields remain allowed; explicitly present unsupported shapes must be refused or removed under a stated test contract, never echoed. Freeze the fake clock/date response when comparing full observation bytes. The fake kubectl harness stays in this crate.