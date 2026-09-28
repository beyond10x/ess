---
format: aep.planning-md/3
id: review-result:session-and-eventual-view-checks-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, story:session-and-eventual-view-checks
relations:
- reviews: story:session-and-eventual-view-checks
revision: 1
---
# Adversary pass 1 — story:session-and-eventual-view-checks

Dispatched as `aep:adversary` against `ess-chc-session` (uncommitted tree on
`impl/session-and-eventual-view-checks`, base `b08ed292d`), 2026-09-28. Header and findings verbatim.

unit: story:session-and-eventual-view-checks, uncommitted working tree of impl/session-and-eventual-view-checks on b08ed292d
verdict: NEEDS-CHANGE
cases: executed 861→866, red 4
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: none left (test temp dirs in scratch deleted, 25 entries)
needs-coordinator: one decision — should `eventual` convergence be judged at all from a finite history (finding 3)? Also: I built under the 10G disk floor, see part 6

Cases (tests/view_consistency_adversary.rs): a read after the reader's own refused write is judged
against every order (RED, 7 of 256 seeds); a read showing a timed-out write that took effect is not a
future read (RED); one read per session after the writes stop does not fail a conformant eventual
projection (RED, 8 of 8 seeds); rows on an Indeterminate operation are refused (RED); a read listing
an invoice issued only after it returned is a future read (green, mutant-killer for the `hi` bound).

Suite: `cargo test -p ess-conformance --locked --no-fail-fast` EXIT=101, 862 passed, 4 failed (the
four new cases).

Not broken: both stale-read acceptance rows; `rows` agreement across schema, model, reader; unknown
ids reported as future-read; unanswered commands block quiescence.

Coordinator routing: all four `introduced` → same implementor. Decided: reads judged against every
reachable state including both branches of Indeterminate writes; convergence judged only after a
settle interval; rows on Indeterminate refused.

```findings
- file: crates/verify/ess-conformance/src/linearize.rs
  line: 836
  category: property
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "read_your_writes takes lo from the single order the search found, so the reference implementation is reported stale-read at 7 of 256 seeds when the reader's own refused write is concurrent with another client's in-flight issue"
- file: crates/verify/ess-conformance/src/linearize.rs
  line: 489
  category: concurrency
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the search takes an Indeterminate write as never happened first, and a read showing that timed-out write's effect is then reported future-read although an order in which it took effect explains the read"
- file: crates/verify/ess-conformance/src/linearize.rs
  line: 849
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "eventual convergence is demanded at each session's last read after quiescence, so the conformant reference at DEFAULT_LAG is reported not-converged at 8 of 8 seeds when each session reads once after the writes stop"
- file: crates/verify/ess-conformance/src/history.rs
  line: 606
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "rows on an Indeterminate operation are admitted by the reader and the schema, while the field is documented as written only on a Returned read and outcome in the same position is refused by name"
```
