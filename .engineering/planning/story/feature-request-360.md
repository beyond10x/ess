---
format: aep.planning-md/3
id: story:feature-request-360
kind: story
status: draft
title: A parameterized view witnesses fields copied from a related row
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#360
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests
revision: 3
---
## Outcome

A synthesized parameterized view selects a stored field copied from a related row by arranging that related source before creating the subject. Dropping the parameter or omitting its scenario is not completion.

## Fit review

1. Need: the issue describes a run that copies an objective identity from its task, then a view parameter selecting that copied field. A brand-free reduction will use a shipment copying a depot identifier from its order. No requester-proposed syntax is required; existing params, sets and related sources express the contract.
2. Class: a synthesis witness gap in existing semantics, not a new language construct. The existing related.rs module contract says it arranges the referenced row and settles its copied values; subject_fact.rs bounds goal-directed stored-row arrangement. Source review alone has not yet reproduced this precise gap.
3. Existing idiom: removing the parameter exposes every row and changes the desired contract. A handwritten authored scenario may test the desired view, but does not repair synthesis. Existing related.rs::arrange, ::key and ::arrange_except provide related-row setup and distinguish decoys; synthesize.rs::bound binds a view's selected values from settled fields.
4. Fit: reuse the existing typed related-row setup, settled expression evaluation and view parameter binding. Keep source identity relations intact and avoid inventing a literal identity or an ownership edge. Coordinate with feature-request-307: the same copied field feeds stored guards and view selection. Both need branch-correct arrangement and complete predicate verification. All Rust/Go/TypeScript conformance runners consume the existing steps; no runtime or generated-server behavior is implied by this witness repair.
5. Second adopter: an inventory reservation copies a warehouse identity from its order, and a warehouse-filtered reservations view must include only matching rows. This is distinct from the originating report and uses the same existing contract.
6. Cost: no new authored vocabulary, persisted field or format is intended. Scenario setup and expectations will change where the old synthesis refused. Preserve existing deterministic ordering, bounded search and explicit refusals for unsupported arrangements; any proposed format change returns to design review.
7. Alternatives: leave the parameter out (changes observable semantics); duplicate special-purpose literals inside view synthesis (breaks captured identity and source truth); chosen reuse of related-row arrangement/settled values at the existing bounded search and view-binding seams. Minimal tests must establish the actual missing seam before changing it.

## Decisions

Accept as proposed for existing related sources and view parameters, coordinated with accepted issue307. This is planned follow-on work; no implementation or successful reproduction is claimed yet. Do not fold in Optional/chained-related language additions from issues304/285 or general cross-row joins.

## Acceptance

- copied_related_identity_binds_view_parameter: minimal validated model synthesizes and executes a nonempty scenario without ESS-SYNTH-005, selecting the related row's captured identity.
- copied_related_value_filters_matching_rows: distinct source and subject decoys establish both inclusion and exclusion, and the expected row carries the copied source value.
- wrong_related_row_and_ignored_param_mutants_fail: honest target passes; first-row/last-row substitution and an ignored parameter each fail a decisive assertion.
- copied_field_guard_and_view_share_arrangement: a reduction composed with307 witnesses both stored-field branches and view selection consistently.
- existing_direct_field_parameter_bytes_and_refusals_stay_stable: unaffected fixtures remain unchanged, unsupported/cyclic arrangements retain named bounded refusal, and no new skip is introduced.

## Scope

Cited at b05007e49: crates/verify/ess-conformance/src/synthesize.rs (prepare_subject/related arrangement, bound), src/synthesize/related.rs (arrange, key, settle), src/synthesize/subject_fact.rs (bounded goal search). Inferred: focused regression fixture/tests in crates/verify/ess-conformance/tests. Inspect exact shared seam before editing; serialize with307 and current298 work on synthesize.rs.
