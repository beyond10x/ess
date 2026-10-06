---
format: aep.planning-md/3
id: story:feature-request-276
kind: story
status: implemented
title: Declarations added or removed leave no residual in the diff
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#276
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/verify/ess-diff/src/diff.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T06:31:25Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-01T06:31:54Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:05Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Outcome

`ess verify diff` reports no `unclassified-changed` for a declaration added or removed on one side, and actors' `attributes` are compared or named.

## Acceptance

- For every family (types, entities, commands, events, errors, views, actors, bindings, components), a declaration added or removed on one side produces only `<family>/<name>/added|removed`, never `system/<system>/unclassified-changed`; one test per family, including an added actor with `attributes`.
- An `attributes` edit on an actor present on both sides still reports `unclassified-changed` (whole obligations apply); no new change kind and no `ess-diff` format bump.

## Origin

beyond10x/ess#276, reported downstream on 0.48.0; same class as #256.

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md`: class defect, diff only, no authored surface.

## Decisions

- **accept, fixed for the class** (coordinator, 2026-10-01): one-side declarations leave the residual. Classifying `attributes` edits would need new change kinds and `ess-diff/13`; the least-surface answer keeps them `unclassified-changed`, which is conservative. A later story may classify them if an adopter needs finer obligations.

## Scope

Derived 2026-10-01 by `aep:story-scoper`; **cited** = read in the tree, **inferred** = a reading. Coordinator-owned at merge, not scope entries: `CHANGELOG.md`, `changes/`, derived outputs.

- **Already done:** the added-view-with-`aggregation` case (#256, `be1e58b3fa`; `crates/verify/ess-diff/tests/aggregate_view_added.rs`) — cited
- **Files:** `crates/verify/ess-diff/src/diff.rs` — cited: `residual_construct` strips only `may` from actors (:2351), so `attributes` stays; `residual` (:2069) and its one comparison (:123) take one revision, so the class fix needs the other side's key set
- **Confidence:** high for the defect site
- **Would collide with:** 268 on `ess-diff/src/diff.rs` (inferred on 268's side)
- **Safety fact:** every `*_changes` already emits `Added`/`Removed` for a key in one map only (`diff.rs:820-821`, `keys` :140), so dropping one-side declarations from the residual hides no unclassified content — unproven for the other eight families; a test per family is part of the acceptance
