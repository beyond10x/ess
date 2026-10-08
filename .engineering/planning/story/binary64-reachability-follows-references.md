---
format: aep.planning-md/3
id: story:binary64-reachability-follows-references
kind: story
status: draft
title: Binary64 reachability may miss a value under a multi-type term (suspected)
relations:
- serves: vision:O2
revision: 1
---
## Suspected finding (not measured)

Read in code by the U2 implementor of wave 2026-10-07b, not run: `reaches_binary64`
(`crates/generate/schema-contract/src/realize.rs:535`) skips nodes it has seen by pointer, and a
multi-type term shares its parent's pointer, so a binary64 value under such a term might not be
found. The U2 fix (`55a03d553e`) uses a reachability walk that tracks references instead.

## Acceptance

A test with a binary64 value under a multi-type term either shows the walk finds it (and this
story is archived as not reproduced) or fails, and the walk is changed to track references.
