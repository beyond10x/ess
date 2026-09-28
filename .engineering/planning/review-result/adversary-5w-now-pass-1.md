---
format: aep.planning-md/3
id: review-result:adversary-5w-now-pass-1
kind: review-result
status: active
title: Adversary pass 1, now (the-5-waves)
relations:
- reviews: story:current-time-guard-operand
revision: 1
---
Adversary pass 1 against story:current-time-guard-operand (#171), aep:adversary, 2026-09-28, the-5-waves wave 3.

verdict: NEEDS-CHANGE
cases: added 9, red 3 (executed 1746→1755)
origin: introduced 3, pre-existing 0, undecided 0

New cases in `adversary_now_synthesis_runs.rs` (ess-conformance) and `adversary_now_guard_sites.rs` (ess-domain). Red: a fixed-instant cutoff beside a now rule on the same field is rewritten as a now_offset; now inside exists/forall over List<Timestamp> is neither rewritten nor refused; an input when: beside when_subject_state refuses now with a self-contradicting message. Green: all orderings at 0 and ±876600h, slow service, operand on the left, Optional<Timestamp>.

Coordinator routing: all three to the implementor.

```findings
[{"file": "crates/verify/ess-conformance/src/now_offset.rs", "line": 516, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "every whole-second value chosen for a now-guarded field is rewritten as a now_offset, so a fixed-instant guard on the same field is witnessed years off"},
 {"file": "crates/verify/ess-conformance/src/now_offset.rs", "line": 475, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "now inside an exists/forall body over a List<Timestamp> input is admitted but neither rewritten nor refused"},
 {"file": "crates/specify/ess-domain/src/command.rs", "line": 2374, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the input when: beside when_subject_state is refused with a message telling the author to use the when: they already wrote"}]
```
