---
format: aep.planning-md/3
id: story:a-first-match-chain-is-synthesized-within-a-work-bound
kind: story
status: active
title: A first-match chain over finite inputs is synthesized within a work bound and reaches every reachable branch
summary: Directed search charges its budget per node, so synthesis time stops growing exponentially with earlier guards, and a reachable branch is no longer refused (row 40).
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-22T23:52:42Z", actor: "human:timo", revision: 2, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-22T23:52:42Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
---
## Need

Reported by adopter B (tracker row 40, 2026-10-23, re-tested on 0.68.0). A command over 8 finite inputs
(one four-valued enum, one three-valued enum, six Booleans) whose outcomes are N-1 guarded refusals in
first-match order and one creating outcome takes time that grows 3 to 6 times per added guard: on
0.67.0 N=15 10.3 s, N=16 35.2 s; the adopter measured N=16 31.8 s and N=17 over 120 s on 0.68.0.
The same shape also refuses a reachable branch: at N=13 `refuse-10` (`b4 == false`, `e2 == Y`) is
reached by `e1=B, e2=Y, b1=true, b2=false, b3=false, b4=false` and is refused as ESS-SYNTH-003, "no
candidate of the 85 tried" (`synthesize.rs:1620`).

## Where it lives

- `crates/verify/ess-conformance/src/witness.rs:1984-2034` (`Directed::search`): the budget
  (`MAX_CANDIDATES`, `:158`) is charged only on a complete assignment, so branches pruned by a
  contradiction cost nothing and the search is unbounded in work; the target guard's atoms are popped
  last (`goals()`, `:1936-1981`).
- The pruning the adopter proposed (earlier branches' guards narrow later candidates) exists as
  `searched_guards` / `admits_plain` (`synthesize.rs:7535-7569`, `:7854-7903`).

## Acceptance

- A first-match chain of 16 guarded refusals over the adopter's input shape is synthesized under a
  stated work bound (`WORK`, `witness_memo.rs:32`), asserted in `tests/witness_directed_search.rs`.
- Every branch reachable by some input in that chain gets its scenario; an unreachable branch is
  refused naming itself.
- No `ess/N` change. Suite digests that move are re-pinned with the reason in the change.
