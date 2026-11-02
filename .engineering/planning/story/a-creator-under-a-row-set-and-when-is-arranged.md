---
format: aep.planning-md/3
id: story:a-creator-under-a-row-set-and-when-is-arranged
kind: story
status: draft
title: A creator guarded by a row set and a when is arranged
tags:
- ess-d368-close
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by wave D368 U4 (first-round implementor note), out of that story's Acceptance: a creator branch guarded by a row set and a `when:` together is never arranged, so the rows it lays are not reached through it (`synthesize.rs`, `arrange_toward_with`).

## Acceptance

- Synthesis arranges a creator branch whose guard holds a row set and a `when:`, with an input that holds the `when:`, or refuses it as `ESS-SYNTH-003` naming the guard.
- A test model with such a creator is witnessed, and committed models whose suites move are named.
