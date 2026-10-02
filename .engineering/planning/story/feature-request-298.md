---
format: aep.planning-md/3
id: story:feature-request-298
kind: story
status: active
title: A Boolean input is not treated as a closed domain
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#298
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/ess-synth/tests/finite_boolean.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/finite.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_guard.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/subject_fact.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/subject_state.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/finite_enum.rs
- confidence: cited
  path: docs/design/closed-enum-outcome-coverage.md
- confidence: cited
  path: website/docs/guides/specify/fields-and-invariants.md
revision: 21
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T10:49:58Z", actor: "human:timo", revision: 13, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T10:49:58Z", actor: "human:timo", revision: 14, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

Resolve beyond10x/ess#298: A Boolean input is not treated as a closed domain.

## Origin

beyond10x/ess#298, from a downstream hardening run on ess 0.48.0; reproduced minimally (triage item 9, `~/.cache/ess-gaps/triage-cb/`).

## Fit review

1. Need: required Boolean input partitioned by pause == true and pause == false must validate without a default and produce typed executable witnesses. The existing issue reproduction introduces no syntax. finite.rs:49-65 excludes Boolean literals, :102-108 admits enum domains only, command.rs:2799-2807 defers only named roots, and witness.rs:477-482 emits text values.
2. Class: gap in the existing finite proof, with a misleading uncovered-input diagnostic. docs/design/closed-enum-outcome-coverage.md:3-17 deliberately describes the initial enum-only fragment. Do not describe documented Boolean support as a regression.
3. Existing idiom: one Boolean guard plus an unguarded default validates (command.rs:2738-2765), but does not prove the explicitly authored two-guard partition. Existing predicate vocabulary already supports Boolean comparisons and truthiness; no new predicate syntax.
4. Fit: use the existing resolver's ScalarKind::Bool domain [false,true], typed FactValue assignments, existing evaluator and witness input builder. Required transparent wrappers/dotted struct paths use existing resolution; Optional traversal remains outside the proof. Preserve enum ordering/diagnostics, bounds and runtime representation. Stored/related/state proof callers also use this mechanism and must not silently gain new overlap rejection policy when defaults are present.
5. Second adopter: a shipping command explicitly partitions signature_required to emit corresponding handling events. Same existing Boolean type and outcome selection, no local policy.
6. Cost: transient public Rust Case/FieldCase values become typed instead of String, a source API consequence even without serialized changes. No new authored keyword or persisted proof/format expected. Sample generated Rust/Go/TypeScript execution; source inspection alone is not parity evidence. Preserve existing enum witness bytes and order.
7. Alternatives: retain the default idiom and unsupported explicit partition; special-case two guards, duplicating authority and failing mixed domains; selected bounded typed finite-domain extension, without general satisfiability or invariant solving.

## Decisions

Accept as proposed: required Boolean domains within the existing finite proof, including direct Boolean predicate siblings and executable witnesses. Preserve default selection, Optional/open-domain refusals, existing enum behavior, 64-assignment/128-node bounds and unrelated proof exclusions. Do not extend rejection policy of stored/related/state validators merely because their shared proof learns Boolean values. Implementation remains pending in the synthesis batch sequence.

## Acceptance

- boolean_no_default_partition_has_two_typed_witnesses: both guards validate, Boolean inputs select exactly one branch, interpreted command scenarios pass.
- mixed_enum_boolean_partition_covers_the_product: cover every assignment; missing/overlapping combinations name actual typed values.
- boolean_transparent_wrapper_preserves_closed_domain and boolean_struct_path_preserves_closed_domain: required nested wrappers/fields work; Optional fields/containers still require a default.
- boolean_membership_negation_and_truthiness_are_typed: existing equality/inequality/membership and Boolean combinations use values, never text true/false.
- mixed_domain_bounds_remain_64_assignments_and_128_nodes: boundary and boundary-plus-one controls, including state/stored-input products.
- boolean_finite_witness_respects_wrapper_invariants: excluded candidates are never executed, without making coverage an invariant solver.
- existing_enum_and_default_candidates_keep_order_and_bytes; boolean_shared_proof_callers_preserve_default_semantics.
- Generated target sampling confirms actual admitted Boolean command behavior where existing target support applies; unsupported target constructs retain named obligations/refusals.

## Scope confirmation during implementation

subject_state.rs::analyze_partition (lines264-294 at b05007e49) is another shared finite-proof caller and is added as cited scope. The implementor established that stored/related/state callers invoke the shared proof even with defaults; unrestricted Boolean admission there would introduce new overlap rejection. Preserve their existing enum-only proof for commands with defaults while admitting Boolean in their no-default proof. This preserves current rejection policy; demonstrate it with before/after default and enum regression controls. Existing top-level explicit Boolean partition support remains required.

## Current-main integration for target validation

The conformance batch now has bot merge3ee06ca31b099c59db703820f6d3b40dbc59392d, incorporating mainb4da64e into reviewed coreb05007e49. The worker stopped every build before integration; all ten owned uncommitted source hashes matched before and after. This corrects stale target sampling: the old batch base preceded the generated Go behavior implementation delivered inPR386. That old obligation is not a current limitation and must not be reported as the target verdict. Native Go validation resumes on the integrated generator. Actual Go and TypeScript conformance runtime samples already exercised both Boolean branches, and the TypeScript ignored-flag mutant failed as expected; final evidence remains pending the complete frozen baseline/treatment pair and independent review.

## Final implementation and source review

Bot commit d1e3bed41 contains the ten frozen paths on current-main-integrated 3ee06ca. Identical final test bytes execute 138 cases on baseline and treatment: baseline127 passed/11 failed, treatment138 passed/0 failed/0 ignored. Actual generated Rust and Go commands execute both Boolean outcomes; generated Go conformance passes2/2, TypeScript honest runner passes2/2 and ignored-flag mutant fails1/2. Strict all-target Clippy for domain/conformance/synth and formatting passed. Source review consumer-boolean-298-pass1-20261002 found no additional counterexample, with zero reviewer executions. Final affected-package checks and integration remain pending; no completion move yet.

## Combined documentation integration correction

The combined fba30d2af candidate passed ci-lint and all six projection checks. Its site build exited successfully but warned that guides/write-a-specification links to fields-and-invariants#cover-every-declared-enum-value, removed by the Boolean heading rename. Preserve that published fragment with an explicit heading id and rebuild the site before publishing the combined PR 387 head. This is a documentation compatibility correction within the existing fields-and-invariants scope.

## Documentation integration result

The explicit heading-id attempt was rejected by this site's MDX parser, and a separate HTML anchor still triggered its link checker. Retaining the original heading text preserves the published fragment without changing the Boolean coverage explanation. The corrected site build stage now exits zero without broken-anchor warnings (combined-anchor-site-final.log), after the same candidate's WASM/browser-lab stages passed. The combined service-contract package also passed all seven tests and formatting passed. No remote rerun was spent on either local documentation correction.
