---
format: aep.planning-md/3
id: review-result:declared-fault-injection-adversary-pass-2
kind: review-result
status: active
title: Adversary pass 2, story:declared-fault-injection
relations:
- reviews: story:declared-fault-injection
revision: 1
---
# Adversary pass 2 — story:declared-fault-injection

Dispatched as `aep:adversary` after correction round 1, 2026-09-28. Header and findings verbatim.

unit: story:declared-fault-injection, uncommitted working tree at crates/ (base acd6a0862, branch impl/declared-fault-injection)
verdict: NEEDS-CHANGE
cases: executed 886→891, red 3
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 scratch log plus 25 test temp dirs, all deleted
needs-coordinator: yes. The unit's own delay injection hides the unit's own RetryCreatesSecondEntity fault in 7 of 16 histories that apply a request twice, so this goes back to the implementor.

Cases (tests/faults_adversary_pass2.rs): a replay after an unanswered original does not let a new
request create what it replayed (RED); an original that never answered but took effect and a retry
answering the origin branch is applied twice (RED); RetryCreatesSecondEntity is caught when the
command also declares another external branch (RED, 7 of 16 missed, all with a delayed Seed); a
retry of a retry is one request (green); a replay of a generated identity is searched beside the
record it replays (green).

Suite: `cargo test -p ess-conformance --locked --no-fail-fast` EXIT=101, 888 passed, 3 failed (the
three new). ess-cli lanes green.

Coordinator routing: both `introduced` → correction round 2. Decided: the search records per request
whether its origin branch was taken; at most one origin step per request, and a replay only after it.

```findings
- file: crates/verify/ess-conformance/src/linearize.rs
  line: 1096
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "applied_twice counts origin answers only on Returned operations and the search lets an Indeterminate member of the same request also take `seeded`, so RetryCreatesSecondEntity with its original delayed by the unit's own injection is judged Linearizable in 7 of 16 histories whose reads show a request applied twice"
- file: crates/verify/ess-conformance/src/linearize.rs
  line: 639
  category: judgement
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: "a replay may be ordered after an Indeterminate retainer that took no branch, so a new request creating the identity the replay proves was already created is accepted; the module doc's claim that this is judged leniently rather than falsely does not hold"
```
