---
format: aep.planning-md/3
id: story:feature-request-278
kind: story
status: implemented
title: An input guard beside a stored-field guard on one branch is synthesized
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#278
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: docs/design/input-guard-overlap-precedence.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T06:31:26Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-01T11:17:36Z", actor: "human:timo", revision: 6}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:06Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
## Outcome

A branch guarded by both an input condition and a stored-field condition, beside a sibling with the same input condition, gets its scenarios; derived wrong-state conditions never contradict themselves.

## Acceptance

- On the minimal specification attached to #278, `RecordResult/outcome/held-for-promotion`, its transition scenario and the three `state/*/refuses/RecordResult` scenarios are synthesized with no ESS-SYNTH-003.
- No synthesized condition has the form `c and none of: c, …`; a test asserts it over the fixture set.

## Origin

beyond10x/ess#278, reported downstream on 0.48.0.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`: class defect (synthesis only; no authored surface).

## Decisions

- **accept** (coordinator, 2026-09-30).

## Scope

Derived 2026-10-01 by `aep:story-scoper`; **cited** = read in the tree, **inferred** = a reading. Coordinator-owned at merge, not scope entries: `CHANGELOG.md`, `changes/`, derived outputs.

- **Files:** `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` — cited: `selects` (:1610) needs exactly one selected branch (~:1664); with `result == Healthy` both `held-for-promotion` and `promoted` are selected, so neither is picked (ESS-SYNTH-003). `refusal_input` (:1787) / `refusal_unsatisfied` (~:1925) refute every sibling's input guard, including one equal to `own`, which renders `c and none of: c, …`
- **Symbols:** `selects`, `refusal_input`, `refusal_unsatisfied`, `input_selects` (:1682, same single-pick rule) — cited
- **Also likely:** `synthesize.rs` `rendered` (:5050) builds the `none of:` text, expected untouched; `docs/design/input-guard-overlap-precedence.md:87` — inferred
- **Confidence:** medium
- **Would collide with:** 229, 266, 271 (merged), 272 (`input_selects`, the same function) on `subject_fact.rs`
- **Safety fact:** `selects` returns `None` today whenever more than one branch is selected; taking the first declared changes only inputs refused now — unproven
