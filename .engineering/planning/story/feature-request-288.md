---
format: aep.planning-md/3
id: story:feature-request-288
kind: story
status: active
title: 'An affects: filter over subject identity is witnessed'
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#288
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/set_effects.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/set_effects.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T10:02:27Z", actor: "human:timo", revision: 8, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T10:02:27Z", actor: "human:timo", revision: 9, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

An `affects:` filter over `subject.<identity>` is witnessed as one over `input.<identity>` is.

## Acceptance

- issue_288_affects_subject_identity_uses_the_captured_user: generated subject identity, matching related rows and different-owner decoy synthesize and execute against an honest Rust target.
- issue_288_affects_does_not_touch_another_subjects_rows: mutant ignoring the subject predicate fails.
- issue_288_subject_identity_and_input_identity_witness_equivalently: equivalent declared filters preserve obligations/behavior without requiring identical scenario bytes.
- Test identity on either comparison side, mixed subject identity/stored-field conjuncts and nominal foreign-key types. Preserve other set-effect witnesses and named refusals.

## Origin

beyond10x/ess#288, found in the 2026-10-01 fit review.

## Fit review

1. Need: affect only related rows belonging to the addressed subject, including a subject whose identifier was generated during arrangement. Issue #288 supplies a minimal user/session example using existing subject.<identity> syntax.
2. Class: synthesis defect/gap under documented supported vocabulary. docs/design/set-effects-over-filtered-instances.md:32-35 admits subject fields and :75-77 promises subject-valued conjunct witnesses.
3. Existing expression: input.user_id may be a workaround when it is demonstrably the same identity, but does not replace arbitrary captured subject identity. set_effects.rs:206-219 currently requires before[first].value.as_literal(), while captured identities use ScenarioValue::Instance.
4. Fit: preserve symbolic identity through arrangement/filter evaluation, reusing the bound instance already held at set_effects.rs:778 and input at :789. Never invent a literal for a generated identity. Existing bound-instance scenario representation should suffice; preserve per-conjunct decoys, target identity authority and nominal representation. Subject-binding helpers in subject_fact.rs:621/:1397 are reuse candidates, not automatic scope for a rewrite.
5. Second adopter: cancel all active reservations belonging to a generated booking identity. Same declared cross-entity relation and filter.
6. Cost: no authored syntax or persisted field expected if existing ScenarioValue::Instance is reused. Any new representation needs separate format review. Scenario content changes must retain deterministic ordering and nonmatching rows.
7. Alternatives: change nothing leaves a documented filter unsynthesizable; newly refusing source removes a valid existing capability; selected symbolic-aware witness construction meets the established contract.

## Decisions

Accept as proposed, choosing synthesis support over new validation refusal. Preserve captured identity references rather than substituting invented literal values. Needs fresh red reproduction against the current batch before implementation; the original report predates this candidate.
