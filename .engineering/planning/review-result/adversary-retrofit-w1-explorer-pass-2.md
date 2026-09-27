---
format: aep.planning-md/2
id: review-result:adversary-retrofit-w1-explorer-pass-2
kind: review-result
status: active
title: 'Adversary pass 2: an unarrangeable exit draw is dropped from ambiguous'
relations:
- reviews: story:explorer-takes-external-branches
revision: 1
---
Adversary pass 2 against story:explorer-takes-external-branches (beyond10x/ess#156), aep:adversary, 2026-09-27, after correction round 1.

verdict: NEEDS-CHANGE (warning)
cases: executed 651→655, red 1
origin: introduced 1, pre-existing 0, undecided 0

New cases in `crates/verify/ess-conformance/tests/explore_external_exits_adversary.rs` (fixture
`explore-external-exits.yaml`): both new exits (no-move state, overlapping guards) take their
external branch with an arranging double, and reverting either exit is caught (mutants); TS and Go
byte-identical. Red: with a double that cannot arrange, both exit draws are missing from
`ambiguous` (`src/ts/explore.ts:1250`, `src/go/explore.go:1558`), contradicting the documented
meaning at `explore.ts:97` / `explore.go:105`.

Coordinator decisions: the warning goes back to the implementor (no third attack); the design note
is decided as "an overlapping-guard draw is also recorded in `ambiguous` when the external branch
executes", so a spec defect stays visible.

```findings
[{"file": "crates/verify/ess-conformance/src/ts/explore.ts", "line": 1250, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "When every eligible external branch at a no-ordinary-branch exit is unarrangeable, the draw is silently dropped instead of recorded in ambiguous, hiding overlapping-guard and no-move-state spec defects (same at src/go/explore.go:1558)."},
 {"file": "crates/verify/ess-conformance/src/ts/explore.ts", "line": 683, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "With an arranging double, overlapping ordinary guards are never reported because the draw is executed via the external branch, so a spec's overlapping-guard defect is invisible whenever the command declares an external branch."}]
```
