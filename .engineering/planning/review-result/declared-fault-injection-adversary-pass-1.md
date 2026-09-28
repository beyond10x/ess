---
format: aep.planning-md/2
id: review-result:declared-fault-injection-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, story:declared-fault-injection
relations:
- reviews: story:declared-fault-injection
revision: 1
---
# Adversary pass 1 — story:declared-fault-injection

Dispatched as `aep:adversary` against `ess-chc-faults` (uncommitted tree on
`impl/declared-fault-injection`, base `acd6a0862`, with the unit's interpreter patch applied by the
coordinator), 2026-09-28. Header and findings verbatim.

unit: story:declared-fault-injection, uncommitted working tree in the faults unit worktree (base acd6a0862, branch impl/declared-fault-injection)
verdict: NEEDS-CHANGE
cases: executed 881→886, red 2
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (scratch log, deleted)
needs-coordinator: yes. The relaxed interpreter accepts a replay that nothing earlier was answered for, so the unit goes back to the implementor.

Cases (tests/faults_adversary.rs): a replay answered for a request nothing retained is not
linearizable (RED); a replay answered before its request was ever applied is not linearizable (RED);
a command declaring only a replays external branch is never delayed or left unanswered (green);
nothing is injected into a prefix of external calls (green); an event an at_most_once binding also
reacts to is never delivered twice (green).

Suite: `cargo test -p ess-conformance --locked --no-fail-fast` EXIT=101, 884 passed, 2 failed (the
two new).

Coordinator routing: `introduced` → same implementor. Decided: the search keeps the set of retained
requests; a replay step exists only for a request already retained.

```findings
- file: crates/verify/ess-conformance/src/interpret/execute.rs
  line: 454
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "under Generated::Recorded a replays: branch is stepped unconditionally, so a Seed answered `replayed` with no earlier `seeded` answer for its request (alone, or before its retry's origin answer) is checked Linearizable instead of a violation"
```
