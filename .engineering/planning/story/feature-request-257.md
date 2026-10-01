---
format: aep.planning-md/3
id: story:feature-request-257
kind: story
status: active
title: An aggregate group key copied from a related row is witnessed
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#257
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/aggregate_views.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/aggregate_views_mutants.rs
- confidence: inferred
  path: docs/design/aggregate-views.md
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T13:04:15Z", actor: "human:timo", revision: 11}
- {from: "proposed", to: "active", at: "2026-09-30T13:04:17Z", actor: "human:timo", revision: 12}
---
## Outcome

An aggregate view whose group key the creating command copies from a related row (`{related: …}` in `sets:`) is witnessed: synthesis creates the related row with the key it wants, then runs the creating command.

## Acceptance

- Scenario `<view>/aggregate` is synthesized for a view grouped by a key set from a related row; ESS-SYNTH-017 no longer names such a key.
- The ESS-SYNTH-017 message says "does not set" only when the command sets the key from nothing.
- A mutant that aggregates under the wrong key fails the synthesized scenario.

## Origin

beyond10x/ess#257, reported downstream on 0.46.1.

## Scope

Derived 2026-09-30 by `aep:story-scoper`. Every line is **cited** (read from the story or the tree) or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance` (aggregate synthesis) — cited, beyond10x/ess#257 names `synthesize/aggregate.rs` around line 674
- **Files:** `crates/verify/ess-conformance/src/synthesize/aggregate.rs:652-676` — cited, the group-key chain ending in the ESS-SYNTH-017 "does not set the group key" refusal
- **Files:** `aggregate.rs:530-544` — cited, `mapped` keeps only `ResolvedPayloadValue::InputField` sets, so a `RelatedField` key never counts as set
- **Files:** `aggregate.rs:1067-1290` (`arrange_and_observe`, `arrange_row`), `:1340` (`held`), `:1291` (`kept_as_planned`) — inferred
- **Also likely:** `crates/verify/ess-conformance/src/synthesize/related.rs` (`arrange`, `key`) — inferred
- **Also likely:** `crates/verify/ess-conformance/tests/aggregate_views.rs`, `tests/aggregate_views_mutants.rs` — inferred
- **Documents:** `docs/design/aggregate-views.md`, `website/docs/reference/diagnostics.md` (ESS-SYNTH-017 row), `CHANGELOG.md` — inferred
- **Confidence:** high
- **Would collide with (every in-epic pair `aep plan artifact waves` reports, 2026-09-30):** 270 on `ess-conformance/src/synthesize/related.rs`; 272 on `ess-conformance/src/synthesize/aggregate.rs`, `ess-conformance/tests/aggregate_views.rs`, `docs/design/aggregate-views.md`
- **Safety fact:** the new key kind is chosen only where the chain now falls through to ESS-SYNTH-017 (`aggregate.rs:671-676`), so every view witnessed today keeps its scenario — unproven

## Fit review

Fit review from `docs/design/review-external-requests-2026-09.md` (2026-09-30), per `.agents/skills/assessing-external-requests/SKILL.md`.

- Need: an aggregate witness for a key copied from a related row. Class: defect (synthesis only; the ESS-SYNTH-017 message was false). No authored surface.

## Decisions

- **accept as proposed.**
