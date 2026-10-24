---
format: aep.planning-md/3
id: story:related-rows-of-one-command-share-their-owner
kind: story
status: active
title: Related rows of one command share their owner
summary: 'Adopter report N15: Synthesis builds a second owner for a related row and the wrong branch answers (ess#669).'
refs:
- provider: github
  reference: beyond10x/ess#669
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T09:48:33Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-23T09:48:33Z", actor: "human:timo", revision: 3}
---
## Problem

A command reading two related rows, one owned by the other, with a predicate joining them (`box_id != input.box_id`) is arranged one row at a time (`crates/verify/ess-conformance/src/synthesize/related_guard.rs:26-31`). The item's owner box is created apart from the box the input names, so `elsewhere` answers instead of `labelled` and `not-a`. Adopter report N15, ess 0.67.0 and 0.68.0 (7 of 9 failures in the adopter's specification). Cause inferred from code, not traced on a run.

## Acceptance

- The reproduction (a box owning items, an item owning labels, `LabelItem`, `ShakeItem`; neutral names) is added as a conformance fixture under `crates/verify/ess-conformance/tests/`.
- `label_item_labelled_passes`: `LabelItem/labelled` synthesizes with the item inside the box the input names and passes on the interpreted target.
- `label_item_not_a_passes`: `LabelItem/not-a` passes the same way.
- `label_item_elsewhere_still_answers_elsewhere`: `LabelItem/elsewhere` keeps the item in another box and passes.
- No pinned suite digest moves except the fixture's own row.
