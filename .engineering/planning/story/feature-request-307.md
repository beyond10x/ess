---
format: aep.planning-md/3
id: story:feature-request-307
kind: story
status: active
title: when_subject over a field copied from a related row is witnessed
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#307
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/subject-guard-copied-field.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/subject_guard_copied_field.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T12:22:56Z", actor: "human:timo", revision: 5, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T12:22:57Z", actor: "human:timo", revision: 6, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

A `when_subject` predicate over a field the creating command copied from a related row is witnessed.

## Acceptance

- On a reduction of #307, both policy branches and their transitions are synthesized with no ESS-SYNTH-003 or -004.

## Origin

beyond10x/ess#307, reported downstream on 0.49.0.

## Fit review

- Class: defect (synthesis only; no surface). The stored-row search must arrange the related source row before creating the subject.

## Decisions

- **accept as proposed** (coordinator, 2026-10-01). Priority 1; next wave after w2.

## Existing implementation and remaining acceptance evidence, 2026-10-02

Source8d9139d4a was integrated into28aeddddf and is now in mainb4da64e38b770fe74103409fe1fef7ae6ca214f4. subject_guard_copied_field.rs and fixtures/subject-guard-copied-field.yaml exercise a copied auto_promote field; retained current-session package evidence is5 passed/0 failed/0 ignored. Do not rebuild that seam merely because this story remained draft.

The original issue and Acceptance require both auto_promote and automatic_rollback policy branches and their transitions. The current reduction declares only auto_promote, with promoted and an unconditional finished branch; tests name only those two outcomes and do not assert the two-policy transition IDs. Its interpreted test permits Unsupported. This is a concrete acceptance-evidence gap, not yet a demonstrated remaining production defect. Extend the minimal model to both Optional flags and named transitions, run honest and copy/branch mutants, and diagnose only if it goes red. Coordinate that validation with360's related-value view binding; preserve existing implementation and keep the original full request.

## Next grouped verification and repair

Read-only assessment by scope_boolean performed zero builds or test executions. Implement sequentially in one worktree after the frozen PR387 source: first complete issue307 evidence for both copied Optional Boolean policies (auto_promote and automatic_rollback), explicitly named transitions, true/false/absent and input-result conjunct controls. Assert the actual captured related source identity. Unsupported target answers do not satisfy acceptance. Keep production unchanged unless fresh tests establish a defect.

Then reproduce issue360 separately for direct creation and later subject arrangement. Current prepare_in arranges related rows whereas invoke_with settles against an empty map; this is a hypothesis, not a reproduction. Require a matching and nonmatching subject even when the copied foreign field is not an ownership link. Exercise copied ordinary values and generated identities, source decoys before and after the selected source, and first-source/last-source/ignored-parameter mutants. Compose stored guards and view selection in the same reduction.

Keep changes within existing bounded related arrangement, settled values and view binding. Preserve cycle/search bounds: do not blindly recurse from invoke_with because arrange_except currently starts nested arrangement with an empty chain. No new syntax, format, ownership edge, arbitrary identity literal, general join or runner semantics are accepted. One group package validation and eventual grouped PR; no separate #307 remote gate.
