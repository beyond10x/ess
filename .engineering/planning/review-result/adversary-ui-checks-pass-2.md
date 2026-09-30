---
format: aep.planning-md/3
id: review-result:adversary-ui-checks-pass-2
kind: review-result
status: active
title: Adversary pass 2, ess-ui wave unit ui-spec-checks
relations:
- reviews: story:ui-spec-checks
revision: 1
---
unit: story:ui-spec-checks, working tree ess-ui-checks (branch impl/ui-checks, base 90ce849d7)
verdict: CONFIRMED, red 9
cases: executed 53→63, red 9
origin: introduced 9 / pre-existing 0 / undecided 0

## Findings

| # | file:line | severity | message |
|---|---|---|---|
| 1 | rules.rs:870 | warning | a widget on a cycle closing through an already-finished widget is not reported (a→{b,c}, b→a, c→b misses c) |
| 2 | rules.rs:235 | blocker | widget uses collected only from plain nodes; a widget containing itself through an inline confirm overlay passes |
| 3 | rules.rs:275 | warning | param defaults are never type-checked |
| 4 | rules.rs:322 | blocker | an UNMAPPED marker bound to an enum param is an error although unmapped_marker.accepted_by lists enum |
| 5 | rules.rs:899 | warning | a literal containing " and "/" or "/" in "/"==" is taken for an expression and skips the type check |
| 6 | rules.rs:482 | blocker | primitive exactly_one_of runs in widget declarations, failing a document valid at every use |
| 7 | rules.rs:399 | blocker | widget-declaration state resolves without a page profile; state_resolves error in a hybrid document |
| 8 | classify.rs:28 | warning | every duplicate YAML key is names_unique, including a property such as title |
| 9 | classify.rs:107 | warning | a duplicate key in an unnamed action is reported at <list>/<key>, not the derived path |

Tests: crates/ui/ess-ui-check/tests/adversary_pass2.rs (10 cases). Logs: ~/.cache/ess-ui-wave/checks/adv2-*.log.

## Correction 2 (coordinator-verified)

All 9 fixed: Tarjan SCC over the widget graph including overlay bodies; param defaults type-checked; UNMAPPED acceptance read from the schema; an expression parser (expr.rs) replaces the substring test; per-node checks run only on expanded bodies; duplicate keys classified by the schema map of named constructs; unnamed-action paths from raw::segment. Coordinator rerun of cargo test -p ess-ui-check: 63 passed, 0 failed (7 + 9 + 10 + 36 + 1).
