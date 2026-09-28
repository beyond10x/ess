---
format: aep.planning-md/2
id: review-result:linearizability-checker-over-the-interpreter-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, story:linearizability-checker-over-the-interpreter
relations:
- reviews: story:linearizability-checker-over-the-interpreter
revision: 1
---
# Adversary pass 1 — story:linearizability-checker-over-the-interpreter

Dispatched as `aep:adversary` against `ess-chc-checker` (uncommitted tree on
`impl/linearizability-checker-over-the-interpreter`, base `540d47ad6`), 2026-09-28. Header and
findings verbatim.

unit: story:linearizability-checker-over-the-interpreter, uncommitted working tree on base 540d47ad6
verdict: CONFIRMED (2 red cases)
cases: executed unknown→406+ (suite run cut off by a full disk), red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 log file and 10 test temp dirs in the assigned scratch, all deleted
needs-coordinator: `/` is at 100% (1.4–2.7G free). The suite could not finish, so it needs a rerun once there is space.

Cases (tests/linearizability_adversary.rs): a sequential history the interpreter answered is
Linearizable when an event carries a generated field (RED); a declared refusal recorded from the
interpreter is explained (green); a three-client lost update shrinks to at most two clients and four
operations (RED, seed 5: 3 clients, 4 operations).

Suite: `cargo test -p ess-conformance --locked` EXIT=101, cut off by ENOSPC after 46 binaries
(406 passed, 0 failed); not evidence about the new cases.

Not broken: real-time order (Fig. 1 fixture); cache keyed on the full store; Indeterminate as no
step or any branch; cross-subject interaction; Unknown/Violation precedence; budget boundaries;
determinism; LostUpdate not caught by the suite path.

Coordinator routing: both `introduced` → same implementor. Finding 1: the unit may add a supply mode
to `execute.rs` in which slots not given are minted.

```findings
- file: crates/verify/ess-conformance/src/linearize.rs
  line: 218
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "The checker gives the interpreter only the created identity, so an event field declared generated (other than the identity) matches no step, and a sequential history the model's own interpreter produced is reported as a Violation."
- file: crates/verify/ess-conformance/src/linearize.rs
  line: 590
  category: property
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "The shrinker never reassigns clients, so a three-client lost-update race shrinks to 3 clients and 4 operations, which breaks the at-most-2-clients acceptance bound whenever the setup client is not one of the racers."
```
