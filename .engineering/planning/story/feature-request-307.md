---
format: aep.planning-md/3
id: story:feature-request-307
kind: story
status: draft
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
revision: 3
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
