---
format: aep.planning-md/3
id: review-result:adversary-chc-reads-pass-1
kind: review-result
status: active
title: Adversary pass 1, concurrent-history wave 5 unit reads
relations:
- reviews: story:explorers-record-view-reads
revision: 1
---
# Adversary pass 1 — story:explorers-record-view-reads

Dispatched as `aep:adversary` against `ess-chc-reads` (uncommitted tree on
`impl/explorers-record-view-reads`, base `e1b9468159` plus the coordinator's merge fix), 2026-09-28.
Header and findings verbatim.

unit: story:explorers-record-view-reads, uncommitted working tree on e1b9468159 (plus the coordinator's merge fix)
verdict: NEEDS-CHANGE
cases: executed 801→806, red 1
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: none remaining (scratch-dir contents made by my runs, deleted)
needs-coordinator: none

Cases (crates/edge/ess-cli/tests/explore_concurrent_reads_adversary.rs): a lost answer of a rejected
creation does not hide a stale read (RED: 16 of 75 violating seeds lost); a target lagging until a
token is demanded is clean and a deaf one is caught (green); an unexposed view is excluded (green); a
timed-out read is Indeterminate without rows (green); correct targets over 200 seeds, wider and
injected, give no violation (green).

Suite: `cargo test -p ess-cli --locked --no-fail-fast` EXIT=101, 805 passed, 1 failed (the new red
case), 4 ignored.

Coordinator routing: both `introduced`; finding 1 → same implementor (drop `rows` only when a read
shows a row no operation names); finding 2 covered by the adversary's green lagging-target case.

```findings
- file: crates/verify/ess-conformance/src/go/explore.go
  line: 2427
  category: property
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "Dropping rows from every read after any creation that never answered hides real stale reads: with the answers of rejected, nothing-creating OpenTicket calls lost, 16 of 75 stale-read violating seeds stop being violations, in Go and TypeScript alike (src/ts/explore.ts:2196)."
- file: crates/verify/ess-conformance/tests/fixtures/explore_target.go
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "No fixture target or lane reads the demanded token (AtLeast/atLeast), so an explorer that never sent read_your_writes tokens would keep every unit test green."
```
