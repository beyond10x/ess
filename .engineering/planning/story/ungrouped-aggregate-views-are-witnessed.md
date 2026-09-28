---
format: aep.planning-md/3
id: story:ungrouped-aggregate-views-are-witnessed
kind: story
status: active
title: An ungrouped, unparameterised aggregate view gets a scenario
relations:
- serves: vision:O2
- decomposes: epic:retrofit-findings-20260927
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/aggregate_optional_fields.rs
- confidence: inferred
  path: docs/design/aggregate-views.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-27T21:28:08Z", actor: "human:timo", revision: 5, imported: true}
- {from: "proposed", to: "active", at: "2026-09-27T21:29:37Z", actor: "human:timo", revision: 6, imported: true}
---
## Scope

An aggregate view with no `group_by` and no parameter, such as #148's `DurationTotal`
(`sum(duration)` over every `Order`), gets no scenario: synthesis refuses it with `ESS-SYNTH-016`
because nothing ties its one row to the rows the scenario created
(`crates/verify/ess-conformance/src/synthesize/aggregate.rs` ~651). This holds for every
ungrouped, unparameterised aggregate view, with or without `skip_absent`. Found by adversary pass 1
of the aggregates unit (`review-result:adversary-retrofit-w2-aggregates-pass-1`).

## Acceptance

The `DurationTotal` repro from #148 synthesizes a scenario whose expectation holds on a target
that starts empty and fails on a target that miscounts, or synthesis states in the suite why it
cannot (for example: the scenario asserts the change in the total, not its absolute value).

## Derived scope

Derived 2026-09-27 by `story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize/aggregate.rs` — cited
- **Refusal:** `synthesize/aggregate.rs:660-664` returns `AggregateUnscoped` (story pointer ~651 is off) — cited
- **Symbols:** `scenario` (473), `RefusalCause::AggregateUnscoped` (`synthesize.rs:577`), `aggregate::UNSCOPED` (`src/aggregate.rs:40`) — cited
- **Test:** `tests/aggregate_optional_fields.rs:207-218` asserts ESS-SYNTH-016 for `DurationTotal` — cited
- **Also likely:** `docs/design/aggregate-views.md:443-481` scoping rule; on a delta approach: `scenario.rs` `SnapshotView`/`ExpectViewUnchanged`, `runner.rs`, Go/TS, suite format — inferred
- **Open:** `aggregate-views.md:445-447` says synthesis never assumes the target is empty, so an absolute total is ruled out; the acceptance must be met by a delta assertion or a stated refusal
- **Confidence:** medium (refusal outcome), low (delta outcome)
- **Would collide with:** `synthesize/aggregate.rs`, `RefusalCause` in `synthesize.rs`
