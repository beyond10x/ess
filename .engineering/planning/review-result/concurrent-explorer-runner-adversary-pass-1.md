---
format: aep.planning-md/2
id: review-result:concurrent-explorer-runner-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, story:concurrent-explorer-runner
relations:
- reviews: story:concurrent-explorer-runner
revision: 1
---
# Adversary pass 1 — story:concurrent-explorer-runner

Dispatched as `aep:adversary` against `ess-chc-runner` (uncommitted tree on
`impl/concurrent-explorer-runner`, base `b08ed292d`), 2026-09-28. Header and findings verbatim.

unit: story:concurrent-explorer-runner, uncommitted working tree on b08ed292d (impl/concurrent-explorer-runner)
verdict: NEEDS-CHANGE
cases: executed 768→772, red 4
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 log files under the brief's scratch dir, all deleted
needs-coordinator: none

Cases (crates/edge/ess-cli/tests/explore_concurrent_adversary.rs), all RED: a target whose every
call fails does not pass (200/200 linearizable in both languages); a target without CloseTicket does
not pass (200/200); one client is refused (accepted); a negative client count is an error, not a
panic (Go panicked with makeslice).

Suite: `cargo test -p ess-cli --locked --no-fail-fast` EXIT=101, 768 passed, 4 failed (the four
new), 4 ignored.

Not broken: Go and TS bytes equal for Indeterminate histories (60 seeds) and for seeds 2^53−1 and
2^32+1; checker exit codes 2, killed and 3 all surface as errors or Unknown; lost-update not caught
without the concurrent mode; the sequential explorer unchanged; dependencies standard-library only.

Coordinator routing: all four `introduced` → same implementor. Decided: only a genuinely unknown
result is Indeterminate; unsupported commands follow the sequential exclusion rule; Clients outside
2..4 refused in both languages.

```findings
- file: crates/verify/ess-conformance/src/go/explore.go
  line: 1997
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "every failed call is written Indeterminate, so a target that answers none of its calls passes 200 of 200 seeds as Linearizable in Go and TypeScript and CheckConcurrent/concurrentProblem report nothing"
- file: crates/verify/ess-conformance/src/ts/explore.ts
  line: 1678
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "ErrUnsupported/unsupported is recorded as Indeterminate rather than refused or excluded, so the close-unsupported mutant passes concurrent exploration 200/200 with no mention that CloseTicket was never exercised"
- file: crates/verify/ess-conformance/src/go/explore.go
  line: 1879
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "Clients is documented as 2 to 4 but Clients 1 is accepted in both languages and records a one-client history that passes"
- file: crates/verify/ess-conformance/src/go/explore.go
  line: 2135
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "a negative Clients panics in Go with makeslice: len out of range while TypeScript writes clients -1 and fails on the checker's refusal, so the two ports diverge"
```
