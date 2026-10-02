---
format: aep.planning-md/3
id: story:feature-request-308
kind: story
status: active
title: 'A constrained newtype identity refuses replay scenarios: complete subject requires a finite exact typed observer'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#308
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/replay.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/subject.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/retained_replay.rs
- confidence: cited
  path: docs/design/retained-command-results.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T10:32:09Z", actor: "human:timo", revision: 9, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T10:32:09Z", actor: "human:timo", revision: 10, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

Resolve beyond10x/ess#308: A constrained newtype identity refuses replay scenarios: complete subject requires a finite exact typed observer.

## Origin

beyond10x/ess#308, found during wave w2 (#287 / #272).

## Fit review

1. Need: compare complete actual subject snapshots when the identity is a constrained primitive newtype. Issue #308 reports refusal of otherwise valid replay scenarios; no new authored syntax is proposed.
2. Class: capability gap, correcting the earlier one-line classification. docs/design/retained-command-results.md:221-222 explicitly excludes invariant and reading observers recursively; replay.rs:177 enforces that policy. This is not merely an accidental implementation refusal.
3. Existing expression: unconstrained wrappers work; weakening the declared identity or reverting to partial snapshots loses the consumer's guarantee. SubjectShape::of (subject.rs:40-53) delegates to the restrictive shared declaration extractor.
4. Fit: complete-subject preservation proves exact represented-value equality, separately from invariant satisfaction. Preserve nominal wrapper representation and structural row admission (subject.rs:89-107). Give subject observation its own explicit declaration profile, retaining unsupported reading/recursive/numeric/resource boundaries. Retained-result responses keep their existing stricter profile; do not globally remove replay::declarations_for's constraint refusal.
5. Second adopter: preserved deployment configuration keyed by a constrained deployment-name type. Existing SubjectShape identity selection and comparison have the same need.
6. Cost: existing Declaration::Newtype { of } represents the structural comparison without inventing an invariant evaluator. Generated descriptors must make no claim to validate erased invariants. No serialized field is planned; verify old-reader admission and show that existing descriptor meaning remains structural equality before concluding no format bump. Update the binding design's explicit limitation. A change to descriptor meaning instead requires format review.
7. Alternatives: keep the named refusal, leaving the need unmet; remove all shared constraints globally, silently broadening retained responses; selected bounded subject-only structural observation with explicit limits and tests for the unchanged sibling contract.

## Decisions

Accept, redesigned: distinguish structural complete-subject comparison from invariant evaluation, and restrict the new admission to the subject profile. Preserve exact identity and field comparison, malformed-value refusals, and retained-response refusal behavior. The previous 2026-10-01 one-line acceptance lacked this distinction; the source-backed review supersedes that rationale without claiming implementation. No user decision is missing for this routine design choice within the authorized backlog work.

## Acceptance

- issue_308_constrained_identity_has_an_exact_subject_snapshot: valid constrained String identity produces a real replay/preservation scenario with complete subject observations, not a downgrade to legacy steps.
- issue_308_constrained_identity_snapshot_detects_changed_row: honest target passes and a target mutating a preserved field fails exact comparison.
- issue_308_nested_newtype_representation_is_preserved: required/Optional nested wrappers retain wire representation and type admission.
- Decimal/Binary64, reading semantics, recursion/resource bounds, missing fields and malformed values preserve their existing refusals. Separate retained-response test proves that profile was not broadened.
- Old-reader/descriptor test and binding design establish the exact format consequence; do not assert invariant checking from structural admission.
