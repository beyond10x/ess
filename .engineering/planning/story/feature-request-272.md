---
format: aep.planning-md/3
id: story:feature-request-272
kind: story
status: proposed
title: A when_related guard on the creating command does not refuse the aggregate view
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#272
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/aggregate_views.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/related_guard.rs
- confidence: inferred
  path: docs/design/aggregate-views.md
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:17Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

An aggregate view whose creating command has a `when_related` guard is witnessed: the aggregate witness arranges the related row the guard needs, then varies the input group key.

## Acceptance

- With an input-set key and a `when_related` guard on the creating command, `<view>/aggregate` is synthesized instead of ESS-SYNTH-017.
- The guard's refusal branches are still witnessed by their own scenarios.

## Origin

beyond10x/ess#272, reproduced on 0.48.0 with a minimal specification.

## Scope

Derived 2026-09-30 by `aep:story-scoper` on 1bd946d6b; **cited** = read in the tree, **inferred** = a reading. `CHANGELOG.md` and `changes/` are the coordinator's at merge and are not scope entries.

- **Cause:** `crates/verify/ess-conformance/src/synthesize/aggregate.rs:1075` `arrange_and_observe` takes its base input from `reach()`, which refuses a related-guarded creator (`synthesize.rs:4473`) → ESS-SYNTH-017 — cited
- **Files:** `aggregate.rs:1192` (`arrange_row`), `:1214` (`subject_fact::input_selects` ignores `Related` branches, `subject_fact.rs:1548`) — cited
- **Files:** `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (`plain_input` :856, `selects` :189, `drive` :290) — inferred
- **Tests:** `tests/aggregate_views.rs` or a new file; `tests/related_guard.rs` regression — inferred
- **Documents:** `docs/design/aggregate-views.md` §Arrangement (:501) — inferred
- **Confidence:** medium
- **Would collide with (every in-epic pair `aep plan artifact waves` reports, 2026-09-30):** 229 on `ess-conformance/src/synthesize/related_guard.rs`, `ess-conformance/src/synthesize/subject_fact.rs`; 257 on `ess-conformance/src/synthesize/aggregate.rs`, `ess-conformance/tests/aggregate_views.rs`, `docs/design/aggregate-views.md`; 266 on `ess-conformance/src/synthesize/subject_fact.rs`; 270 on `ess-conformance/src/synthesize/related_guard.rs`; 271 on `ess-conformance/src/synthesize/related_guard.rs`, `ess-conformance/src/synthesize/subject_fact.rs`
- **Safety fact:** `reach()`'s related-guard refusal is shared by every family without a related row; the fix gives the aggregate path its own base input and leaves `reach()` as it is — unproven

## Fit review

Fit review from `docs/design/review-external-requests-2026-09.md` (2026-09-30), per `.agents/skills/assessing-external-requests/SKILL.md`.

- Need: aggregate witness when the creating command has a `when_related` guard. Class: defect (synthesis only).

## Decisions

- **accept as proposed.**
