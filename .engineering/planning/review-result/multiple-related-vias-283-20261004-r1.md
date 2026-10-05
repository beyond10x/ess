---
format: aep.planning-md/3
id: review-result:multiple-related-vias-283-20261004-r1
kind: review-result
status: active
title: 'Several related vias adversary pass 1: other-row arrangement and absent-before-missing'
relations:
- reviews: story:feature-request-283
revision: 1
---
unit: W3-2 #283 several related vias, pass 1
verdict: NEEDS-CHANGE
cases: executed 543→553, red 5
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: review scratch (probe, base/unit outputs, logs); build dir cleaned
needs-coordinator: F5 — fix the generated obligation contract here or in #319

Publication copy of the only adversary pass on the #283 unit (worktree `<worktrees>/ess/ess-w3-283-multiple-related-vias-20261004`, base `d1026d1f0`, uncommitted diff of 11 files). The reviewer added `crates/verify/ess-conformance/tests/adversary_283_pass1.rs` and `crates/specify/ess-domain/tests/adversary_283_pass1.rs`; no production file edited. Byte drift against base: 10/10 identical (7 single-via fixtures, billing, gatepass, oracle-fixture).

F1 (blocker) `crates/verify/ess-conformance/src/synthesize/related_guard.rs:301` (`beside`): demands a row selecting none of its branches, so on an admitted no-default model where every capability row selects a branch, synthesis refuses `no-such-switch` and `switch-paused` (ESS-SYNTH-003). The implementor's own domain test admits this model.

F2 (warning) `related_guard.rs:1320`: a related predicate reading `input.strict` on the other row makes synthesis refuse `no-such-capability` and `capability-revoked` although `strict: false` witnesses both.

F3 (warning) `related_guard.rs:1845` (`overlaps`): no overlap sends an earlier-declared absent Optional reference beside a missing later row, so a target ending the row read at an absent reference passes every scenario.

F4 (note) `crates/specify/ess-domain/src/command/related_guard.rs:1584`: beside a default, two accepting branches over different rows that both hold are refused at 64 joint cases and admitted silently at 68, contrary to the documented `conflicting_declaration`.

F5 (warning, read only) `crates/generate/ess-synth/src/plan.rs:708`: the owed obligation contract for a several-row command without wrong_state omits refusal-before-acceptance and the missing-row order the interpreter applies.

Attacked and not broken: two rows of one entity (reverse-order target caught); existing_instance beside two rows; three rows; the 64-case cap with and without an Optional row; interpreter vs domain order (read); ess-synth at base keeps a two-via command owed.

```findings
[
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard.rs", "line": 301, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "beside() demands a row selecting none of its branches, so on an admitted no-default model where every capability row selects a branch synthesis refuses no-such-switch and switch-paused (ESS-SYNTH-003)"},
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard.rs", "line": 1320, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a related predicate reading input.strict on the other row makes synthesis refuse no-such-capability and capability-revoked although strict=false witnesses both"},
  {"file": "crates/verify/ess-conformance/src/synthesize/related_guard.rs", "line": 1845, "category": "mutant", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "no overlap sends an earlier-declared absent Optional reference beside a missing later row, so a target ending the row read at an absent reference passes every scenario"},
  {"file": "crates/specify/ess-domain/src/command/related_guard.rs", "line": 1584, "category": "boundary", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "beside a default, two accepting branches over different rows that both hold are refused at 64 joint cases and admitted silently at 68, contrary to the documented conflicting_declaration"},
  {"file": "crates/generate/ess-synth/src/plan.rs", "line": 708, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the owed obligation contract for a several-row command without wrong_state omits refusal-before-acceptance and the missing-row order the interpreter applies"}
]
```
