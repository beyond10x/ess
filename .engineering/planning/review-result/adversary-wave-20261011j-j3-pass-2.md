---
format: aep.planning-md/3
id: review-result:adversary-wave-20261011j-j3-pass-2
kind: review-result
status: active
title: Adversary pass 2, j3 mutate --scenarios corrections (wave 2026-10-11j)
relations:
- reviews: story:mutate-covers-subject-related-guards-sets-and-authored
revision: 1
---
## Adversary pass 2 on unit j3 (wave 2026-10-11j)

```
unit: j3 (mutate --scenarios) pass 2, findings cover 801fb4df42 (correction round 1); cases committed as 9ca3e2203e on unit/ess-wave-20261011j-a
verdict: red
cases: executed 0→3 in adversary_j3_pass2, red 2
origin: introduced 1 / pre-existing 1 / undecided 0
wrote-outside-worktree: 2 paths in ~/.cache/ess-wave-20261011j/scratch/a/j3-adv2/
needs-coordinator: yes, whether the introduced finding is fixed in this unit beside the pre-existing one (same root)
```

Cases in `crates/verify/ess-conformance/tests/adversary_j3_pass2.rs`: a binding whose failure policy
drops one of two identically answering refusals and escalates the other. A control case shows the
authored scenario passes on the specification and fails on the swap (a real published event).
Attacked without a break: the shared scenario-clock runner, the executed-only witness rule, the
identical-answer rule where no binding policy exists, report determinism.

```findings
[{"file": "crates/verify/ess-conformance/src/mutate.rs", "line": 4033, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "label_only discards an authored failure on an identical_answer precedence swap even when a binding's refusal-selected policy makes the swapped interpreter publish an escalation the specification does not, so a mutant the authored scenario kills is reported equivalent"},
 {"file": "crates/verify/ess-conformance/src/mutate.rs", "line": 2456, "category": "property", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "pre-existing", "message": "identical_answer compares only the two outcomes and ignores bindings that select drop, retry or escalate by outcome name, so it calls a swap identical and says neither branch emits anything while one is escalated"}]
```

Coordinator's routing: both fixed in this unit at their shared root. `identical_answer` returns
nothing when a binding invoking the command selects a different failure policy, or a different
retry `final` membership, for the two outcomes; the label rule of correction round 1 is then sound.
