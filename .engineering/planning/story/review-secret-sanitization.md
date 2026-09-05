---
format: aep.planning-md/1
id: story:review-secret-sanitization
kind: story
status: active
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
revision: 5
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

Derived 2026-09-05 by `aep-drive:story-scoper` from the story, review and repository tree — cited.

- **Primary surface:** `crates/infra/ess-kubernetes` — cited; the story names this package's sanitizer and credential-boundary contract.
- **Files:** `crates/infra/ess-kubernetes/src/lib.rs:107` contains the sanitizer; `crates/infra/ess-kubernetes/src/lib.rs:60` contains collection, serialization and output writing; `crates/infra/ess-kubernetes/src/lib.rs:160` and `crates/infra/ess-kubernetes/src/lib.rs:194` contain existing redaction and malformed-list tests — cited.
- **Symbols:** `sanitize_secret_list`, `scan`, `secret_values_and_last_applied_configuration_never_survive_sanitization`, and `malformed_secret_lists_are_refused_before_any_output_can_be_written` — cited.
- **Additional package-local work:** extend the malformed-shape corpus and add fake-kubectl coverage for diagnostics, preserved destination contents on refusal, and valid-output compatibility; exact test placement remains an implementation choice within the primary surface — inferred.
- **Documents:** no documentation edit is required by this story's acceptance; the review and package contract provide evidence and constraints — inferred.
- **Scope limits:** preserve digest/length records, annotation removal, the collected kind list and observation format; collection scope and retry semantics are explicitly assigned elsewhere by the story — cited.
- **Confidence:** high — cited; both the story and F05 identify the sanitizer, and its caller plus existing tests reside in the same package.
- **Would collide with:** any unit touching `crates/infra/ess-kubernetes`, including collection/retry changes in `scan`; use this exact crate-directory token for collision computation, without narrower test tokens — inferred.

## Scoping decisions

Coordinator clarification: absent optional fields remain allowed; explicitly present unsupported shapes must be refused or removed under a stated test contract, never echoed. Freeze the fake clock/date response when comparing full observation bytes. The fake kubectl harness stays in this crate.

