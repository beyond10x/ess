---
format: aep.planning-md/3
id: review-result:session-and-eventual-view-checks-adversary-pass-2
kind: review-result
status: active
title: Adversary pass 2, story:session-and-eventual-view-checks
relations:
- reviews: story:session-and-eventual-view-checks
revision: 1
---
# Adversary pass 2 — story:session-and-eventual-view-checks

Dispatched as `aep:adversary` after correction round 1, 2026-09-28. Header and findings verbatim.

unit: story:session-and-eventual-view-checks, uncommitted working tree of impl/session-and-eventual-view-checks on b08ed292d (after correction round 1)
verdict: NEEDS-CHANGE
cases: executed 868→872, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (a suite log). The 25 test temp dirs in scratch are deleted
needs-coordinator: finding 1. Should `settle` be counted in operations after the writes stop rather than in clock instants, or should the format drop its claim that "instants compare only by order"? That is a design decision.

Cases (tests/view_consistency_adversary_pass2.rs): the same history on a finer clock gets the same
verdict (RED, 8 of 8 seeds); a never-converging projection whose convergence was never judged is not
a silent pass (RED, 8 of 8); a read listing one invoice twice is not linearizable (RED, 16 of 16); a
read leaving out another client's returned write is not a stale read (green, mutant-killer).

Suite: `cargo test -p ess-conformance --locked --no-fail-fast` EXIT=101, 869 passed, 3 failed (the
three new).

Not broken: the reachable-state set; the walk cache; budget accounting; the session-check mutant; the
`--settle` CLI parsing.

Also reported: rustfmt-only changes in seven `crates/edge/ess-cli/src/*` files. Those were the
coordinator's (a `rustfmt` run on `main.rs` followed its `mod` declarations) and were restored to
HEAD before correction round 2.

Coordinator routing: all three `introduced` → correction round 2. Decided: settle counts reads, not
instants; reads skipped by settle are listed in `not_judged`; duplicate rows are a violation.

```findings
- file: crates/verify/ess-conformance/src/linearize.rs
  line: 934
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "settle subtracts instants although ess-history/1 promises the checker compares instants only by order, so the reference at DEFAULT_LAG recorded on a nanosecond clock is reported not-converged at 8 of 8 seeds"
- file: crates/verify/ess-conformance/src/linearize.rs
  line: 931
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a projection that never converges, read fewer than settle instants after the writes stop, is reported Linearizable with nothing in not_judged, the same report a converging projection gets"
- file: crates/verify/ess-conformance/src/linearize.rs
  line: 360
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "a read's rows are folded into a set, so a projection that lists one instance twice is admitted by the reader and judged Linearizable"
```
