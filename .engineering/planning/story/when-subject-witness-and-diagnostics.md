---
format: aep.planning-md/3
id: story:when-subject-witness-and-diagnostics
kind: story
status: active
title: when_subject branches are witnessed without an immediate view, and ESS-SYNTH-008 names an author action
refs:
- provider: github
  reference: beyond10x/ess#172
- provider: github
  reference: beyond10x/ess#173
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-round-3
scope:
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
- confidence: inferred
  path: crates/verify/ess-conformance/src/coverage_build.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/stored_field_guards.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T22:15:42Z", actor: "human:timo", revision: 5, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T22:17:11Z", actor: "human:timo", revision: 6, imported: true}
---
## Scope

#172: subject-fact selection witnessed through eventual views (or the fallback branch at least), so reachable states after the branch stay reachable. #173: `StrategyWithoutGuard` in `crates/verify/ess-conformance/src/synthesize.rs` (~789) prints guidance, or the refusal scenario is written for a guard-less fallback.

## Acceptance

Both repros in #172 and #173 synthesize with no ESS-SYNTH-001/004/008 refusal, or the refusal names a change the author can make.

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance` (synthesis) — cited
- **Files:** `crates/verify/ess-conformance/src/synthesize.rs`, `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` — cited
- **#173:** `RefusalCause::StrategyWithoutGuard` `synthesize.rs:601`, hint `:788-789`; path `refused_here` `:4998` → `refusal_arrangement` `:5673` → `reach` `:3408` → `plain_guards` `:3496`, arm `ObserveSubjectFact` `:3511` — inferred (read, not run). Fix: choose the refusal input through `subject_fact::input_selects` (`subject_fact.rs:472`) or the default-branch driver — inferred
- **#172:** `subject_fact::observer` `:1095` requires an immediate unfiltered view; `observe_fields` `:1139`; `prepare` `:960`; `successors` `:732` (no successors when `observe_fields` fails, the ESS-SYNTH-004 cascade) — cited/inferred. Reusable: `eventual_observation` `subject_fact.rs:1782`, `require` `synthesize.rs:4505` (`EventuallyView`) — inferred
- **Help text:** the view-aware `NoWitness` hint (0.37.0, `dda521887`) probably already fixes #172's help line — inferred
- **Also likely:** `coverage_build.rs:409`, `consumer_coverage/entry-classifications.json` (only on a variant rename), `tests/stored_field_guards.rs` + fixture — inferred
- **New format:** none — inferred
- **Confidence:** high for #173, medium for #172
- **Would collide with:** units in `synthesize.rs` input choice (`reach`, `plain_guards`, `refused_here`, `RefusalCause::hint`) or `synthesize/subject_fact.rs`
