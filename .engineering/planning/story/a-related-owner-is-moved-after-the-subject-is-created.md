---
format: aep.planning-md/3
id: story:a-related-owner-is-moved-after-the-subject-is-created
kind: story
status: draft
title: A related owner is moved after the subject is created
summary: 'Adopter report N16: Synthesis locks a related owner before the subject is created in it (ess#670).'
refs:
- provider: github
  reference: beyond10x/ess#670
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

`ShakeItem/locked` reads the item's stored `box_id` and needs a locked box. Synthesis locks the box before `AddItem` creates the item in it, and `AddItem` refuses a locked box. The `ordered` split (`crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs:615-672`) handles this shape but is not applied here, presumably because the box is also the item's owner. Adopter report N16, ess 0.67.0 and 0.68.0 (2 of 9 failures). Cause inferred from code, not traced.

## Acceptance

- Uses the same conformance fixture as the related-rows story, under `crates/verify/ess-conformance/tests/` with neutral names.
- `shake_item_locked_passes`: the scenario opens the box, adds the item, locks the box, then sends `ShakeItem`, and passes on the interpreted target.
- `shake_item_no_box_and_shaken_still_pass`: the sibling scenarios are unchanged.
- All three fixture scenarios named in the reports (`LabelItem/labelled`, `LabelItem/not-a`, `ShakeItem/locked`) pass in one run.
