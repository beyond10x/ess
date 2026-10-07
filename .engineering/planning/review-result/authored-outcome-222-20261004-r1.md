---
format: aep.planning-md/3
id: review-result:authored-outcome-222-20261004-r1
kind: review-result
status: active
title: 'Authored outcome check adversary pass 1: current-time guards decided at a fixed instant'
relations:
- reviews: story:feature-request-222
revision: 1
---
unit: W2-8 #222 authored expected outcomes checked against guards, pass 1
verdict: NEEDS-CHANGE
cases: executed 109→116, red 3
origin: introduced 2 / pre-existing 1 / undecided 0
wrote-outside-worktree: review scratch logs; build dir removed; a 90M Go cache left
needs-coordinator: four ess-cli test targets with inline authored scenarios were not run by the reviewer or the implementor

Publication copy of the only adversary pass on the #222 unit (worktree `<worktrees>/ess/ess-w2-222-authored-outcome-check-20261004`, base `5064707da`, uncommitted diff of 8 files). The reviewer added `crates/verify/ess-conformance/tests/adversary_222_pass1.rs` (7 cases, 3 red); no production file edited. 12 committed authored files across 87 specifications raise no ESS-AUTHOR-041.

F1 (blocker) `crates/verify/ess-conformance/src/authored.rs:2416`: guards over the current-time operand are decided at synthesis's fixed reference instant `2019-12-30T23:59:59Z` (`input.rs:1304`) instead of left undecided, so valid authored acts sending literal instants after it are refused ESS-AUTHOR-041. Red: `start-in-past is not taken: its own guard (starts_at < now - 60s) refutes it` for `starts_at: 2025-01-01`.

F2 (warning) `authored.rs:1376`: the related-row early return in `not_taken` is reached by no unit test; disabling it leaves `authored_outcome_guards` green.

F3 (note, pre-existing) `src/go/runtime.go:347`, `src/ts/runtime.ts:1175`, `assets/coverage-admission.js:498`: no test sends an authored code above 001 through the Go, TypeScript or browser coverage readers.

Attacked and not broken: precedence step by step; absent and null Optional inputs; exact numeric comparison; Unicode `.count`; `{$instance}` and references unknown; quantifier binders and caller attributes excluded; old reports with codes up to 040 still read.

```findings
[
  {"file": "crates/verify/ess-conformance/src/authored.rs", "line": 2416, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "guards over the current-time operand are decided at synthesis's fixed reference instant 2019-12-30T23:59:59Z instead of left undecided, so valid authored acts sending literal instants after it are refused ESS-AUTHOR-041"},
  {"file": "crates/verify/ess-conformance/src/authored.rs", "line": 1376, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "the related-row early return in not_taken is reached by no unit test; disabling it leaves authored_outcome_guards green and is caught only by adversary_222_pass1 line 367"},
  {"file": "crates/verify/ess-conformance/src/go/runtime.go", "line": 347, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "no test sends an authored code above 001 through the Go, TypeScript or browser coverage readers, so reverting the 41 bound in any of them fails nothing"}
]
```
