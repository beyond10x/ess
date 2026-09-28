---
format: aep.planning-md/3
id: review-result:adversary-n-mixedguard-pass-2
kind: review-result
status: active
title: Adversary pass 2, ess-next unit mixedguard
relations:
- reviews: story:a-wrong-state-witness-may-miss-a-mixed-guard-sibling-by-its-subject-guard
revision: 1
---
unit: story:a-wrong-state-witness-may-miss-a-mixed-guard-sibling-by-its-subject-guard
verdict: red
cases: executed 1392→1395, red 3
origin: introduced 1, pre-existing 3, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-c1/mixedguard/adv2/ (logs, suites/, base-a4b422e7e/)
needs-coordinator: yes

Adversary pass 2 (`aep:adversary`), 2026-09-28, head 69884aecf + `adversary_mixed_guard_pass2.rs` (3 cases, red on head AND on base a4b422e7e). Pass-1 F1–F3 fixed (7/7). No new wrong scenario; `cargo xtask generate --check` EXIT=0; billing, gatepass, oracle-fixture suites byte-identical to suites/generated.

```findings
[{"file":"crates/verify/ess-conformance/src/synthesize/subject_fact.rs","line":698,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"the wrong-state witness checks only the moving branch input half, so a row its when_subject rejects lets a non-moving default answer instead of wrong_state on an Entity-Runtime-ordered target"},{"file":"crates/verify/ess-conformance/src/synthesize/subject_fact.rs","line":660,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"answered_by_state exempts a moving sibling whatever its guard evaluates to, but a guard Unknown on the row (optional field left absent) stops entity-core selection with OutcomeUnobservable before wrong_state"},{"file":"docs/design/cross-record-and-stored-field-guards.md","line":286,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"the doc says a moving sibling needs no refuting selected or not, which is false when its stored guard is Unknown on the witness row"}]
```

Coordinator routing: the two pre-existing findings go to story:wrong-state-witness-unknown-and-own-stored-guards (outcome no-op for this unit; the red test file is archived at ~/.local/state/worktree/archives/ess/mixedguard-adv2/ as that story's reproduction, not merged); the doc sentence was fixed by the coordinator (8f5c3022b, outcome fixed). Trend: pass 1 → 3, pass 2 → 3, carried 0.
