---
format: aep.planning-md/3
id: review-result:adversary-wave-20261011j-j3-pass-1
kind: review-result
status: active
title: Adversary pass 1, j3 mutate --scenarios (wave 2026-10-11j)
relations:
- reviews: story:mutate-covers-subject-related-guards-sets-and-authored
revision: 1
---
## Adversary pass 1 on unit j3 (wave 2026-10-11j)

```
unit: j3, adversary pass 1, covering 903d0e51d1; cases committed as 0fdf6ecfd0 on unit/ess-wave-20261011j-a
verdict: red
cases: executed 22→34, red 4
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (~/.cache/ess-wave-20261011j/scratch/a/j3-adv/commit-message.txt)
needs-coordinator: yes, F3 needs a design call (does an outcome's label count when deciding a mutant is equivalent)
```

Cases in `crates/verify/ess-conformance/tests/adversary_j3_pass1.rs` (3 red of 5) and
`crates/edge/ess-cli/tests/adversary_j3_pass1.rs` (1 red of 7). Attacked without a break: report
determinism, an empty authored set against `audit_with`, the flag's refusals, the shared authored
loader used by `run --scenarios`, a baseline failure refusing the audit, authored and synthesized id
collisions (unreachable by id shape).

```findings
[{"file": "crates/verify/ess-conformance/src/mutate.rs", "line": 3680, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "authored scenarios run on mutant interpreters with no command clock, so one that run --target interpreted passes on a now guard is unsupported and cannot kill the mutant it pins"},
 {"file": "crates/verify/ess-conformance/src/mutate.rs", "line": 4072, "category": "property", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "an authored scenario the baseline did not execute never hides a kill yet counts as witnessing its outcome, turning an unwitnessed mutant into a survivor"},
 {"file": "crates/verify/ess-conformance/src/mutate.rs", "line": 4082, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "an identical-answer precedence swap is killed by an authored scenario through the outcome label alone and keeps identical_answer beside the kill"}]
```

Coordinator's routing: all three back to an implementor. F3 is decided by the rule the
identical-answer equivalence already states: an outcome name only names the branch, so a mutant
recorded with `identical_answer` stays equivalent and an authored scenario differing from it only
in the outcome label does not kill it.
