---
format: aep.planning-md/3
id: review-result:selection-plan-w5-u2-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave 5 unit U2 (synthesis before-query)
relations:
- reviews: story:synthesis-reads-selection-plan
revision: 1
---
```
unit: story:synthesis-reads-selection-plan U2, uncommitted working tree on base 3bfa27aed1
verdict: NEEDS-CHANGE
cases: executed 241→245, red 2
origin: introduced 1 / pre-existing 1 / undecided 0
wrote-outside-worktree: ~/.cache/ess-selection-plan/w5-u2-scratch/adversary-1/
needs-coordinator: whether A1's new refusal is accepted as a correction or reverted
```

Added `adversary_selection_plan_w5_u2_pass1.rs`: a1a (interpreter answers `empty-note` before a stored `exists: false`, green), a1b (an authored act claiming `blocker-missing` there is accepted as at base, red: ESS-AUTHOR-041), a2 sanity (green), a2_exchanged (InputRefusal↔Accepting exchanged, `too-many` fails on the interpreter, red at base too).

Not broken: the format choice moves only `when_related:` predicate refusals, which every helper filters by kind; mutants m1 (plan ignored), m2 (`answers_after` empty), m3 (`first_before_held_and_accepting` None), m4 (before/after inverted) all turn `synthesis_suite_bytes_table` red; `selected_in_state`, `held_state_claims`, `later_refusals` keep base answers under the real order.

Coordinator decision: A1 kept as a correction (the interpreter could never pass the act); CHANGELOG entry added. A2 fixed by taking membership from the plan.

```findings
[
  {"file": "crates/verify/ess-conformance/src/authored.rs", "line": 1377, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "removing not_taken's early return makes an authored act claiming a stored exists:false beside an input refusal newly refused (ESS-AUTHOR-041) on a validating model, and the comments still say nothing claims exists:false"},
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 6800, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "pre-existing", "message": "the kind gate in earlier_accepting_branches decides that an input refusal never answers after an accepting branch, so with InputRefusal and Accepting exchanged too-many's witness fails on the interpreter"}
]
```
