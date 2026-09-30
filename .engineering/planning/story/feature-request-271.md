---
format: aep.planning-md/3
id: story:feature-request-271
kind: story
status: active
title: when_related over an owns via field is witnessed on both sides
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#271
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:16Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-09-30T13:04:18Z", actor: "human:timo", revision: 9}
---
## Outcome

A `when_related` test over the `via` field of an `owns` relation is witnessed on both sides: a row owned by another owner for the refusal, one owned by the input owner for success.

## Acceptance

- `<command>/outcome/<mismatch refusal>` and `<command>/outcome/<success>` are synthesized; no ESS-SYNTH-003, and no cascading ESS-SYNTH-004.

## Origin

beyond10x/ess#271, reproduced on 0.48.0.

## Scope

Derived 2026-09-30 by `aep:story-scoper`; **cited** = read in the tree, **inferred** = a reading.

- **Files:** `crates/verify/ess-conformance/src/synthesize/related_guard.rs` — cited: `with_row` (:757) searches inputs with plain `candidates`; `prepare_at` (:486) binds only the `via` field
- **Files:** `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` — cited: `links` (:493) reads only `hints(command)` (:184), and `stored` (:67) returns `None` for `ResolvedCondition::Related`, so the predicate stays `Unknown` → ESS-SYNTH-003
- **Precedent:** the fix exists for `when_subject` (beyond10x/ess#193, `db92e9fa9`) and is not wired into `when_related` — cited
- **Also likely:** new tests in `crates/verify/ess-conformance/tests/`; `related_guard::boundary_goals` (:392) — inferred
- **Confidence:** high
- **Would collide with (every in-epic pair `aep plan artifact waves` reports, 2026-09-30):** 229 on `ess-conformance/src/synthesize/related_guard.rs`, `ess-conformance/src/synthesize/subject_fact.rs`; 266 on `ess-conformance/src/synthesize/subject_fact.rs`; 270 on `ess-conformance/src/synthesize/related_guard.rs`; 272 on `ess-conformance/src/synthesize/related_guard.rs`, `ess-conformance/src/synthesize/subject_fact.rs`
- **Safety fact:** link comparison is gated on `links(...)` being non-empty, empty today for every command without a `when_subject` stored guard, so existing suites do not change — walked, unproven

## Fit review

Fit review from `docs/design/review-external-requests-2026-09.md` (2026-09-30), per `.agents/skills/assessing-external-requests/SKILL.md`.

- Need: both-sided witness for `when_related` over an owns `via`. Class: defect (synthesis only).

## Decisions

- **accept as proposed**, sharing the #193 link-comparison path (`db92e9fa9`) rather than copying it.
