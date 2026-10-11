---
format: aep.planning-md/3
id: review-result:adversary-wave-20261011j-j5-pass-2
kind: review-result
status: active
title: Adversary pass 2, j5 ess-http/1 correction (wave 2026-10-11j)
relations:
- reviews: story:a-command-binds-to-a-protocol-or-deployment-http-route
revision: 1
---
## Adversary pass 2 on unit j5 (wave 2026-10-11j), on correction 4dac19b7b6

```
unit: j5 (wave 2026-10-11j), pass 2 on the correction 4dac19b7b6; findings cover 4dac19b7b6, cases committed as add618cc4d on unit/ess-wave-20261011j-b
verdict: red
cases: executed 38→45, red 5
origin: introduced 3 / pre-existing 0 / undecided 2
wrote-outside-worktree: 9 paths, all in ~/.cache/ess-wave-20261011j/scratch/b/j5-adv/ (pass 1: red.log, red2.log, suite.log, commit-msg.txt; pass 2: p2-red.log, p2-red2.log, p2-suite.log, commit-msg-pass2.txt; the pass-1 findings had no separate file)
needs-coordinator: yes, decide whether HttpIr::from_json should hold hand-edited IR to compile's invariants or only to its stated order and grammar
```

Pass-1 cases all green on 4dac19b7b6; F1, F2, F3 and the narrowed F5 row held. Cases in
`crates/specify/ess-http/tests/adversary_j5_pass2.rs` (commit add618cc4d); `cargo test -p ess-http
--locked --no-fail-fast` exited 101 (`tests/adversary_j5_pass2.rs test result: FAILED. 2 passed; 5
failed`). Every red case reaches only hand-edited IR.

```findings
[{"file": "crates/specify/ess-http/src/lib.rs", "line": 547, "category": "property", "severity": "warning", "verdict": "CONFIRMED", "origin": "undecided", "message": "HttpIr::from_json reads IR routing one operation twice on two paths, which compile never writes and HttpIr::route answers with the first only (tests/adversary_j5_pass2.rs:135)"},
 {"file": "crates/specify/ess-http/src/lib.rs", "line": 547, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "HttpIr::from_json reads IR holding a derived route for an operation it also routes, against derived_routes meaning operations the binding does not name (tests/adversary_j5_pass2.rs:152)"},
 {"file": "crates/specify/ess-http/src/lib.rs", "line": 492, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "a derived route path is read back unchecked: a variable, no leading slash or an empty segment all read, though route paths are grammar-checked (tests/adversary_j5_pass2.rs:177)"},
 {"file": "crates/specify/ess-http/src/lib.rs", "line": 549, "category": "property", "severity": "note", "verdict": "CONFIRMED", "origin": "undecided", "message": "HttpIr::from_json reads a route whose parameters leave a path variable unmapped or map a variable the path does not have (tests/adversary_j5_pass2.rs:195)"},
 {"file": "crates/specify/ess-http/src/lib.rs", "line": 543, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the from_json doc says what reads back writes back the bytes it was given, but compact JSON reads back and writes back pretty bytes (tests/adversary_j5_pass2.rs:220)"}]
```

Ledger against pass 1 (computed by hand): pass 1 found 5, pass 2 found 5, 0 carried, 5 new, 5
resolved; every pass-2 finding is on `HttpIr::from_json` reading hand-edited IR. Coordinator's
routing: the undecided two are decided as introduced scope of this unit (from_json holds read IR to
every invariant compile guarantees that needs no specification: P1-P4); P5 narrows the doc comment
to "reads canonical or compact JSON and writes canonical bytes", and its case is rewritten to assert
that. One correction round; the coordinator verifies its diff; no third attack.
