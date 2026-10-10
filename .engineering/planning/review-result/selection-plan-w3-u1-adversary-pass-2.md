---
format: aep.planning-md/3
id: review-result:selection-plan-w3-u1-adversary-pass-2
kind: review-result
status: active
title: Adversary pass 2, wave 3 unit U1 (lowering reads the plan)
relations:
- reviews: story:entity-runtime-lowering-reads-selection-plan
revision: 1
---
The correction holds on entity-core: no answer differs between the base order and the plan's order for the new fixture or any variant I could get to validate. But the new fixture will turn another crate's test red in CI.

```
unit: story:entity-runtime-lowering-reads-selection-plan, uncommitted tree ~/.local/state/worktree/trees/b10x/ess/ess-selection-plan-w3-u1 (base 1046621a8)
verdict: NEEDS-CHANGE
cases: executed 136→137, red 1
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (part 6)
needs-coordinator: yes. Re-pinning `crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv` is in ess-compiler, which is not this unit's crate.
```

**1. `git --no-pager diff --stat`**
```
 crates/generate/ess-entity-runtime/src/lib.rs      |  42 ++--
 .../tests/selection_precedence.rs                  | 269 ++++++++++++++++++++-
```
Both are the implementor's edits. My only addition is a new untracked test file, `crates/generate/ess-entity-runtime/tests/adversary_selection_plan_w3_u1_pass2.rs`. I touched no implementation file. A probe file I wrote stayed green and is deleted (part 5).

**2. The case I added** (red now)

`adv_w3u1_p2_every_ess_fixture_of_this_crate_is_pinned_by_the_compiler_selection_table` reads the compiler's selection-precedence table. It then applies that table's own "every model the tree holds is pinned" rule to this crate's ESS fixtures. Run alone:
```
assertion `left == right` failed: ESS fixtures of this crate the compiler's selection-precedence table does not pin; its test `every_repository_command_keeps_its_plan_and_every_model_its_ir_bytes` (ess-compiler) fails on each
  left: ["crates/generate/ess-entity-runtime/tests/fixtures/held-state-after-disjoint-accepting.yaml"]
 right: []
```

**3. Suite run**, after the case existed:
- Command: `CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --no-fail-fast`
- Result: 136 passed, 1 failed (my case), EXIT=101: `error: 1 target failed: -p ess-entity-runtime --test adversary_selection_plan_w3_u1_pass2`
- The correction's own tests passed: the 16-request test, the F1 rewrite and the lowered-definitions table.
- Clippy with `-D warnings` and rustfmt are clean on my file.

**4. Findings**

| # | file:line | verdict / origin | finding | what reaches it |
|---|---|---|---|---|
| P2-1 | `tests/adversary_selection_plan_w3_u1_pass2.rs:77` | NEEDS-CHANGE / introduced | The compiler's `selection_precedence_table.rs:235-244` fails on any ESS file in the tree that it doesn't pin. The new fixture is the only unpinned one. The unit's gate only runs this crate's tests, so the failure would first show in CI. | The full gate: CI, or `cargo test -p ess-compiler`. |
| P2-2 | `tests/lowered_definitions_table.rs:9` | CONFIRMED / introduced | The doc says the table "was written at the story's base". The 4 lines for the new fixture were written after the change. That is the whole difference from the base snapshot in scratch. | Anyone reading the doc. |

The fix for P2-1 is to re-pin the compiler table with `SELECTION_PRECEDENCE_TABLE_WRITE`, as the compiler table's own test message says. Because that is ess-compiler's file, it needs your decision.

**5. Attacked and not broken**

- **Answers in base vs plan order:** a probe, now deleted, compared both orders on six variants that validate. Each got 32 requests (state × reopened × note × rush × evidence). It compared the pre-load decision, which is what a host answers for a missing row, and the answer on a loaded row. There were 0 differences. The variants:
  - the fixture as it is;
  - a refusal as the default;
  - an `external:` + `when:` accepting branch;
  - that branch with a refusal default;
  - a held-state refusal after the accepting branch;
  - an in-state version: an accepting branch that updates, a moving `when_subject_state: Open` branch, and a `Closed` refusal.
- **Variants validation refuses:** the in-state forms that also declare `wrong_state`, and in-state forms without a real default.
- **Does the table go red under the base sort?** Yes. It pins `invalid-note, first-close, rushed, closed, not-open`, and the base order is `invalid-note, rushed, first-close, closed, not-open`. The probe printed the base order, and the unit's own test asserts it.
- **F1 rewrite:** it is sound. It pins the order and both answers, which match the interpreter on an existing row. For a missing row, Entity Runtime answers `paused` before loading while the interpreter checks existence first. That was the same at the base and goes away once #490 refuses the shape.
- **Other walkers in ess-conformance:** `external_beside_held_guard`, `adversary_244b` and `adversary_285` only compare models they already pin, so the new fixture doesn't break them. `enum_presence_guard` adds it to its corpus; I did not run that crate.

**6. Paths written outside the worktree**
- `~/.cache/ess-selection-plan/w3-u1-scratch/adversary-p2-suite.log`

My lease `w3-u1-adversary-2` is released.

**7. Findings block**
```findings
[
  {"file": "crates/generate/ess-entity-runtime/tests/adversary_selection_plan_w3_u1_pass2.rs", "line": 77, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the new fixture held-state-after-disjoint-accepting.yaml is a repository model the ess-compiler selection-precedence table does not pin, so that crate's every_repository_command_keeps_its_plan_and_every_model_its_ir_bytes fails at the full gate"},
  {"file": "crates/generate/ess-entity-runtime/tests/lowered_definitions_table.rs", "line": 9, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the doc says the table was written at the story's base, but the new fixture's four lines were pinned after the lowering read the plan"}
]
```
