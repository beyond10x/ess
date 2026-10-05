---
format: aep.planning-md/3
id: review-result:captured-identities-273-20261004-r1
kind: review-result
status: active
title: 'Captured identities adversary pass 1: comparisons hold; literal half not shape-checked'
relations:
- reviews: story:feature-request-273
revision: 1
---
unit: W3-6 #273 event expectations check identity fields against captured instances, pass 1
verdict: INFEASIBLE (one note; identity comparisons held against every fault applied)
cases: executed 82→89, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: review scratch (logs, env); build dir and Go cache removed
needs-coordinator: none

Publication copy of the only adversary pass on the #273 unit (worktree `<worktrees>/ess/ess-w3-273-captured-identities-20261004`, base `4b8a74244`, uncommitted diff of 34 files). The reviewer added `crates/verify/ess-conformance/tests/adversary_273_pass1.rs` and fixtures `adversary-273-tree.yaml`, `adversary-273-runtime.go`, `adversary-273-runtime.mjs`; no production file or committed suite edited.

Green: a sweep over 34 models and 164 identity comparisons never fails the reference interpreter; 154 copied input identities are all compared; a ticket and its parent ticket (one type) swapped three ways fail; four desk faults (other queue, null Optional, nil UUID, swapped pair) fail with `ESS-CF-PAYLOAD` in Rust, Go and TypeScript; 24 mutants of the regenerated billing suite (drop, null, zero, replace per compared identity) are all killed. The regenerated suites differ from base only in 11 hunks turning `expect_event` into `expect_event_values`; headers stay /34; oracle-fixture unchanged. Old suites (/17 and below) are untouched by runner, Go and TS changes.

Finding (note, infeasible, introduced) `crates/verify/ess-conformance/src/admission.rs:66` and `assets/coverage-admission.js:448`: literals in an `expect_event_values` step are not shape-checked at admission, and #273 now writes every identity-carrying event's literals in that step; red case `adversary_273_pass1.rs:644` (a Uuid field holding `"not a uuid"` is refused as `expect_event` but admitted as `expect_event_values`). Reached by nothing found except a hand-made suite.

```findings
[
  {"file": "crates/verify/ess-conformance/src/admission.rs", "line": 66, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "literals in an expect_event_values step are not shape-checked at admission (Rust :66, browser coverage-admission.js:448), and #273 now writes every identity-carrying event's literals in that step; only a hand-made suite reaches it"}
]
```
