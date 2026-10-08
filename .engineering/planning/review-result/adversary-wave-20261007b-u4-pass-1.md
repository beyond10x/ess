---
format: aep.planning-md/3
id: review-result:adversary-wave-20261007b-u4-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave 2026-10-07b unit 4 (root lock)
relations:
- reviews: story:creating-an-output-root-locks-only-what-it-creates
revision: 1
---
unit: story:creating-an-output-root-locks-only-what-it-creates (covers unit commit 1630b291ac plus one untracked test file)
verdict: green
cases: executed 114→125, red 0 (1 red against a mutant copy, none against 1630b291ac)
origin: introduced 2, pre-existing 0, undecided 0
wrote-outside-worktree: 2 harness background-task logs
needs-coordinator: no

New file `crates/edge/ess-cli/tests/adversary_root_lock_pass1.rs` (417 lines, standalone, test-only). No change under `src/`.

- `a_root_nested_in_a_root_being_created_is_refused_busy_before_its_state_exists` (:165) is the finding. It runs before the new root's `.ess-output` exists, so only the exclusive lock on the new root can refuse the nested run. Mutant (`filesystem.rs:396` in a scratch copy, `NonBlockingLockExclusive` → `NonBlockingLockShared`), run alone: `a root was published inside a root being created: ()`, `test result: FAILED. 0 passed; 1 failed; ... EXIT=101`. The same mutant against every existing target that compiles the engine (5 targets, 109 cases, including `ownership_root_lock::*`): all green.
- Safety probes, all green: a nested root published before the creator locks; sibling roots under one missing parent (both orders); 8 concurrent `generate` runs under a missing `common/` (7 of 8 refused busy naming their own root, every refused root absent, no leftover staging, the published root complete and settled); a symlink swapped in at two points; a root published between locking and admission; adoption racing a publish.

Suite: 7 output-ownership binaries, 114 passed plus 11 in the new file, exit 0; `cargo test -p ess-cli --locked --test adversary_root_lock_pass1` 11 of 11; clippy on it clean.

| file:line | verdict / origin | measured | what reaches it |
|---|---|---|---|
| tests/ownership_root_lock/mod.rs:216 | CONFIRMED / introduced, warning | the probe for "exclusive lock on the new root" also passes when the root is locked shared; with that mutant the whole existing suite stays green and a nested root is published inside a root still being created | any run creating `<new root>/<child>` before the creator writes its `.ess-output` |
| src/output_ownership/filesystem.rs:348 (and design page line 69) | CONFIRMED / introduced, note | the docs say a refused run's directories stay "empty and unenrolled"; the probe at :193 shows the refused creator's `n` holding another run's enrolled `m` | documentation only |

Shared missing parent raced by two runs: busy refusals only, no safety break. U3 settled-record rules: no interaction found (creating a root refuses any entry that is not an admission orphan).

```findings
[
  {"file": "crates/edge/ess-cli/tests/ownership_root_lock/mod.rs", "line": 216, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "The new root's exclusive lock in create_root (filesystem.rs:393-400) is the only guard against a nested root being enrolled before .ess-output exists, and weakening it to shared fails no existing case; adversary_root_lock_pass1.rs:165 fails on that mutant."},
  {"file": "crates/edge/ess-cli/src/output_ownership/filesystem.rs", "line": 348, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The doc comment and design page line 69 say a refused run's directories stay empty and unenrolled, but when another run takes them they hold that run's enrolled root, as adversary_root_lock_pass1.rs:193 shows."}
]
```
