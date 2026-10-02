---
format: aep.planning-md/3
id: review-result:consumer-ui-filter-composition-pass3-20261002
kind: review-result
status: active
title: UI filter live and numeric composition source review
relations:
- reviews: story:feature-request-365
revision: 1
---
unit: #365 UI read-filter, HEAD 55061600bd2be2d5be71daae40a637f8d00ddb74 plus frozen patch 175bd72919ef096e9619e238647043739b28ab9d7ebcf8ef287eaaf36dca931b
verdict: NEEDS-CHANGE
cases: executed 0→0, red 0 (reviewer executions; owner evidence reviewed separately)
origin: introduced 2 / pre-existing 0 / undecided 1
wrote-outside-worktree: none
needs-coordinator: owner-run red/green probes; route the authorization/live composition case after measurement

Own diff: none. The handed-over owner diff contains 24 tracked files, 771 insertions/187 deletions, plus six new files; all 30 original frozen file hashes and the complete patch hash matched before review. No reviewer source/test edits, test runs, builds, AEP writes or remote calls. Existing owner implementation changes are not reviewer mutations.

No cases were added or executed by this read-only reviewer. Owner logs 25/27, 30/31 and 37 were read: stale completed refresh and authorization cases went 6 pass/2 fail→8 pass/0 fail; stalled-poll case went 0 pass/1 fail→9 pass/0 fail; composition log has React 10, UI test 1, TUI 8 passing. These remain owner-produced evidence, not independent test execution.

Judgement findings covering the immutable snapshot:
1. live.ts.tmpl:467 (also listener :510), NEEDS-CHANGE / introduced: a filtered reader with effect=refetch and only_if=row.group==one re-fetches for an event with group=other, because the filtered branch returns before only_if and the listener skips its prior intake check whenever keep exists. Reachability: normal collection useLive call with Reads.filter and declared Live.refetch; checker admits this combination. Source deduction only; measured red has been assigned to owner. TUI app.rs:1598 checks the predicate before :1638 refetch.
2. ess-ui/src/filter.rs:140, NEEDS-CHANGE / introduced: scalar numeric equality diverges between Rust's f64 display and React's String(Number). `row.code == 0.0000001` with code string `0.0000001` is true in Rust and false in React (`1e-7`); same problem at numeric 1e21 versus decimal text. Reachability: the admitted comparison grammar, with operand typing explicitly outside checker scope. Source deduction only; owner will measure. No separate array/object failure is claimed.
3. live.ts.tmpl:443 (return :547), CONFIRMED / undecided: live-local rows are reset by serialized raw rows only, so initial raw [], a live insertion, then new authorization returning [] can retain the prior actor's inserted row through the new useRead isolation. Reachability: existing live insert plus public setAuthorization; no special cache mutation. Source deduction only. The same reset mechanism exists at base, but no base execution was made, so origin is formally undecided rather than presumed pre-existing; owner/root to measure and route.

Other attack coverage: read/filter grammar and checker scope/refusals, widget expansion, row/page/state projection, shared request identity, subscription release, refresh/poll/auth corrections, filtered totals/empty handling, hidden-row live patches, count_new, selection pruning including key-only rows, dynamic menus/choices, graph edges, generated test coverage, old-format/no-filter controls and the explicit TUI open_page remount exception. No further concrete counterexample found in this source pass. Actual browser execution and complete runtime parity are not claimed.

```findings
- file: crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl
  line: 467
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: A filtered refetch reader bypasses only_if and refetches on an event that its intake predicate rejects.
- file: crates/ui/ess-ui/src/filter.rs
  line: 140
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Rust numeric-to-text equality disagrees with React for admitted scalar comparisons at exponent-format boundaries.
- file: crates/ui/ess-ui-react/templates/runtime/live.ts.tmpl
  line: 443
  category: concurrency
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: Live-local rows can retain an old actor's inserted row when authorization changes and both raw reads serialize to the same empty array.
```
