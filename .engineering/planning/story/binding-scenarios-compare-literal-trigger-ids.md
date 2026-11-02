---
format: aep.planning-md/3
id: story:binding-scenarios-compare-literal-trigger-ids
kind: story
status: draft
title: Every binding scenario compares the literal trigger ids it sent
tags:
- ess-d368-close
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by wave D368 U3's stricter sweep, out of that story's Acceptance: 15 non-refusal binding scenarios expect the trigger's event without comparing an id sent as a literal for a field of an entity's identity type. They are pinned in `literal_trigger_id_residue()`: `billing` notify-on-invoice-created, `binding-arrangement` created-starts, `toolchain` check-on-tag and `set-effect-binding-causes` sync-on-flag-set (`synthesize.rs`, `expect_trigger_event`).

## Acceptance

- Every binding scenario compares each literal identity id it sent with the trigger event's identity fields, so `literal_trigger_id_residue()` is empty and removed.
- A target publishing a wrong literal id in any of those scenarios fails it.
- The models whose suites move are named, including the committed `billing` suite.
