---
format: aep.planning-md/3
id: review-result:adversary-a-mutate-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.41 unit mutate
relations:
- reviews: story:gained-refusals-are-unwitnessed-not-survived
- reviews: story:mutate-scores-a-baseline-with-skipped-scenarios
revision: 1
---
unit: mutate (#203, #210)
verdict: red
cases: executed 50→58, red 2
origin: introduced 2, pre-existing 0, undecided 0
wrote-outside-worktree: none (scratch emptied)
needs-coordinator: yes

Adversary pass 2 (`aep:adversary`), 2026-09-28, head 1df10acba + `tests/adversary_mutate_pass2.rs` (6 ok, 2 red). Pass-1 F1–F4 fixed.

```findings
[{"file":"crates/verify/ess-conformance/src/mutate.rs","line":1591,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"any excluded scenario makes an unkilled mutant inconclusive, including one byte-identical to the baseline that cannot kill it, so a real survivor is reported inconclusive"},{"file":"crates/verify/ess-conformance/src/mutate.rs","line":1070,"category":"property","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"the subject is unique only for ESS-SYNTH-011; ESS-SYNTH-005 and -014, recorded once per view at one scenario, share one key"}]
```

Trend: pass 1 → 4, pass 2 → 2, carried 0. Coordinator routing: both fixed in correction 2 (last; coordinator verifies by diff read).
