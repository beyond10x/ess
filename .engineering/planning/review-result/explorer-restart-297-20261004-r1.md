---
format: aep.planning-md/3
id: review-result:explorer-restart-297-20261004-r1
kind: review-result
status: active
title: 'Explorer restart adversary pass 1: trailing restarts check nothing'
relations:
- reviews: story:feature-request-297
revision: 1
---
unit: W3-4 #297 explorer process restarts, pass 1
verdict: NEEDS-CHANGE
cases: executed 8→15, red 2
origin: introduced 3 / pre-existing 1 / undecided 0
wrote-outside-worktree: review scratch (logs, token mutant copies); build dir and Go cache removed
needs-coordinator: none

Publication copy of the only adversary pass on the #297 unit (worktree `<worktrees>/ess/ess-w3-297-process-restart-20261004`, base `d1026d1f0`, uncommitted diff of 7 tracked files). The reviewer added `crates/verify/ess-conformance/tests/explore_restart_adversary.rs` and restart fixtures under `tests/fixtures/adversary*restart*`; no production file edited. Results without `restartEvery` (absent or 0) are byte-identical to the base explorer in both languages, including a failing and shrunk run.

F1 (warning, introduced) `src/go/explore.go:2764` (placement `:2598`; TS `ts/explore.ts:2420`, `:2239`): a restart after a sequence's last command counts as performed, so with `restartEvery` equal to `steps`, or a counter lost on the trailing restart, a counter-reset target passes `CheckExplored`/`assertExplored` in both languages. Red: `typescript: a counter-reset target passed with 3 restart(s), none of them followed by a command: ok`.

F2 (note, introduced) `tests/explore_restart.rs`: no unit target returns a consistency token, so re-reading views at `""` after a restart survives all 8 unit cases; the reviewer's token case kills that mutant.

F3 (note, infeasible) `go/explore.go:99`: a restart that returns success without restarting is reported as performed; the explorer has no process visibility.

F4 (note, pre-existing) `ts/explore.ts:2498`: `exploreConcurrent` ignores `restartEvery` silently for untyped callers, as it ignores every unknown option (read, not run); Go `ConcurrentOptions` has no such field.

```findings
[
  {"file": "crates/verify/ess-conformance/src/go/explore.go", "line": 2764, "category": "boundary", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a restart after a sequence's last command counts as performed, so with restartEvery equal to steps (or a counter lost on the trailing restart) a counter-reset target passes CheckExplored/assertExplored in both languages"},
  {"file": "crates/verify/ess-conformance/tests/explore_restart.rs", "line": 1, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "no target in the unit's suite returns a consistency token, so reading views at an empty token after a restart survives all 8 cases; explore_restart_adversary.rs token case kills it"},
  {"file": "crates/verify/ess-conformance/src/go/explore.go", "line": 99, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "a restart that returns success without restarting is reported as performed and passes, and the explorer has no process visibility to tell it apart"},
  {"file": "crates/verify/ess-conformance/src/ts/explore.ts", "line": 2498, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "exploreConcurrent ignores restartEvery silently for untyped callers, as it ignores every unknown option, read and not run"}
]
```
