---
format: aep.planning-md/3
id: review-result:adversary-wave-20261011j-j5-pass-1
kind: review-result
status: active
title: Adversary pass 1, j5 ess-http/1 templates and collisions (wave 2026-10-11j)
relations:
- reviews: story:a-command-binds-to-a-protocol-or-deployment-http-route
revision: 1
---
## Adversary pass 1 on unit j5 (wave 2026-10-11j)

```
unit: j5 (wave 2026-10-11j), ess-http/1 templates, parameters, collisions; findings cover d03c9a56c6, cases committed as 151464a132 on unit/ess-wave-20261011j-b
verdict: red
cases: executed 30→37, red 6
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths, all in ~/.cache/ess-wave-20261011j/scratch/b/j5-adv/ (red.log, red2.log, suite.log, commit-msg.txt)
needs-coordinator: yes, findings F4 and F6 need a design call (should one source be readable from two variables, and is the -011 wording drift)
```

Cases in `crates/specify/ess-http/tests/adversary_j5.rs` (commit 151464a132): six of seven red against
d03c9a56c6; `cargo test -p ess-http --locked --no-fail-fast` exited 101 (`tests/adversary_j5.rs
test result: FAILED. 1 passed; 6 failed`). Attacked without a break: the path grammar, reserved
paths (refused only at synthesis, stage 2), duplicate YAML keys, `-009` source types, `-010`
mechanics, the IR's omitted empty `derived_routes`.

```findings
[{"file": "crates/specify/ess-http/src/lib.rs", "line": 1217, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a named operation whose route earns any refusal keeps its derived route on the surface, so another route on that path gets a spurious ESS-HTTP-010 (tests/adversary_j5.rs:81 and :99)"},
 {"file": "crates/specify/ess-http/src/lib.rs", "line": 781, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a route with a parsed template but a -009/-014 refusal skips collision checks, so its -010 is not reported at once (tests/adversary_j5.rs:118 and :139)"},
 {"file": "crates/specify/ess-http/src/lib.rs", "line": 978, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "one input field read by two path variables compiles, giving a request two values for one field (tests/adversary_j5.rs:193)"},
 {"file": "crates/specify/ess-http/src/lib.rs", "line": 541, "category": "property", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "HttpIr::from_json accepts derived_routes out of canonical order and writes them back as read (tests/adversary_j5.rs:216)"},
 {"file": "docs/design/http-presentation-binding.md", "line": 172, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the -011 table wording would refuse /items/{item_id}/a beside /items/current/b, which the implementation correctly accepts (tests/compile.rs:903); the design wording should be narrowed"}]
```

Coordinator's routing: all five back to an implementor. F3 is refused (one source read by two
variables is `ESS-HTTP-008`); F4 is refused (`from_json` refuses `routes` and `derived_routes`
out of canonical order); F5 narrows the design table to the implementation's reading.
