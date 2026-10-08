---
format: aep.planning-md/3
id: decision-blocker:ess-058-501-mutate
kind: decision-blocker
status: open
title: 'Does #501 touch mutate.rs before the other session''s next synthesis wave?'
relations:
- blocks: story:witness-nested-connectives-and-decide-optional-presence-equivalence
revision: 3
---
## Question

The second half of #501 (score a precedence swap of two guards no input satisfies together as `equivalent`) needs `crates/verify/ess-conformance/src/mutate.rs` (`satisfiable`, `equality_tests`). Another session's next synthesis wave edits `mutate.rs` and `synthesize.rs`; this story should land before it. The first half (witness nested connectives) needs only `synthesize.rs`, which is free.

| option | does | costs |
|---|---|---|
| A | the whole story lands before that wave, touching `mutate.rs` | that wave merges over one changed file |
| B | split: the witness half lands now; the equivalence half waits for that wave | #501 stays open after 0.58.0 until the second half ships |
| C | the whole story waits for that wave | #501 and its three surviving mutants wait for that wave |

## Decided

A: all of #501 lands before the other session's next synthesis wave, editing `satisfiable` and `equality_tests` in `mutate.rs`. If that wave needs `mutate.rs` first, the equivalence half falls back to B and lands after it.
