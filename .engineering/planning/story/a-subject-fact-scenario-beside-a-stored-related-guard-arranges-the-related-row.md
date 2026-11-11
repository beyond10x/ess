---
format: aep.planning-md/3
id: story:a-subject-fact-scenario-beside-a-stored-related-guard-arranges-the-related-row
kind: story
status: draft
title: A subject-fact scenario beside a stored related guard arranges the related row
summary: Example 28's no-default scenario is decided by the no-parent related guard because the subject-fact family does not arrange the parent row, so an inverted no-default guard passes (turn-3 adversary, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

The adversary pass on
`story:the-interpreted-target-passes-a-related-run-digest-and-a-default-step` (turn 3) found this
problem. It lies outside that story's acceptance.

Example 28 (`crates/generate/ess-synth/tests/fixtures/generation-by-name/28.yaml`) has a
`TakeDefault/outcome/no-default` scenario. The scenario is reached through the subject-fact family
(`crates/verify/ess-conformance/src/synthesize/subject_fact.rs`), which arranges the item but not
the parent row its stored `parent_id` names. The stored-reference related guard `no-parent`
therefore refuses the step first for every target, and the `no-default` claim is never witnessed.
A target with an inverted `no-default` guard passes.

The red case is kept, ignored with this story's id:
`crates/verify/ess-conformance/tests/system_issued_unclaimed_send.rs::adversary_a_target_wrong_about_the_no_default_guard_fails_no_default`.

## Acceptance

- A subject-fact scenario whose deciding branch sits beside an earlier related guard through a
  stored reference arranges the related row, so the earlier guard passes. The scenario's own branch
  then decides the outcome.
- The ignored case runs and passes: a target with the `no-default` guard inverted fails
  `TakeDefault/outcome/no-default`.
- The honest interpreted run of example 28's suite still passes that scenario.
- Every suite-bytes row the change moves is a model with such a guard pair.
