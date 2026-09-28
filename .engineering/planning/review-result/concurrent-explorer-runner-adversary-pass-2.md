---
format: aep.planning-md/2
id: review-result:concurrent-explorer-runner-adversary-pass-2
kind: review-result
status: active
title: Adversary pass 2, story:concurrent-explorer-runner
relations:
- reviews: story:concurrent-explorer-runner
revision: 1
---
# Adversary pass 2 — story:concurrent-explorer-runner

Dispatched as `aep:adversary` after correction round 1, 2026-09-28. Header and findings verbatim.

unit: story:concurrent-explorer-runner, uncommitted working tree on b08ed292d (impl/concurrent-explorer-runner)
verdict: NEEDS-CHANGE
cases: executed 775→784, red 6
origin: introduced 6 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 logs and the suite's temp dirs, all under the brief's scratch dir, all deleted; Go build cache (default GOCACHE)
needs-coordinator: none

Cases (crates/edge/ess-cli/tests/explore_concurrent_adversary_pass2.rs): negative seed count refused
(RED); replaying a seed reproduces the batch bytes (RED, seeds 6, 9, 12); an indeterminate error
joined with another is indeterminate in both languages (RED, TS); an indeterminate throw at invoke is
Indeterminate (RED); an unsupported throw at invoke excludes (RED); PendingCommand docs match the
correction (RED); exclusion from seed 5 keeps Go and TS bytes equal (green); lost update still a
violation with CloseTicket left out (green); a plain error or thrown string stops the run with one
message (green).

Suite: `cargo test -p ess-cli --locked --no-fail-fast` EXIT=101, 778 passed, 6 failed (the six new),
4 ignored.

Coordinator routing: all six `introduced` → correction round 2. Decided: an excluded command stays in
every later seed's draw and only its calls are left out of the history, so seeds stay independent.

```findings
- file: crates/verify/ess-conformance/src/go/explore.go
  line: 2269
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "a negative Seeds (TS seeds) is accepted and records zero histories, which CheckConcurrent/concurrentProblem pass as none, while negative Calls is refused"
- file: crates/verify/ess-conformance/src/go/explore.go
  line: 2289
  category: property
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "once a command is excluded mid-run, later seeds draw from the shrunk command list, so replaying a reported seed with Seed writes different bytes than the batch run did (seeds 6, 9 and 12 all differ)"
- file: crates/verify/ess-conformance/src/ts/explore.ts
  line: 1708
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "a throw from a TypeScript target's invokeCommand bypasses classification, so indeterminate() fails the run instead of writing Indeterminate and unsupported fails it instead of excluding the command"
- file: crates/verify/ess-conformance/src/ts/explore.ts
  line: 1614
  category: concurrency
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "isIndeterminate follows only cause, so an AggregateError holding indeterminate() stops the TypeScript run while Go reads errors.Join(e, ErrIndeterminate) as Indeterminate"
- file: crates/verify/ess-conformance/src/go/explore.go
  line: 1848
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the emitted PendingCommand.Complete doc still says an error is a call that never answered and is written Indeterminate, which the correction made false"
- file: crates/verify/ess-conformance/src/ts/explore.ts
  line: 1526
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the emitted PendingCommand.complete doc still says a throw is a call that never answered and is written Indeterminate, which the correction made false"
```
