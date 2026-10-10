---
format: aep.planning-md/3
id: review-result:selection-plan-w3-u1-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave 3 unit U1 (lowering reads the plan)
relations:
- reviews: story:entity-runtime-lowering-reads-selection-plan
revision: 1
---
I ran nothing after your hold arrived. Both runs below finished at 18:02, before it, with 11G free. The `cargo clippy` check right after started at 9.8G, which is under the 10G floor; it took 0.5s. My lease is released.

```
unit: story:entity-runtime-lowering-reads-selection-plan, uncommitted tree ~/.local/state/worktree/trees/b10x/ess/ess-selection-plan-w3-u1 (base 1046621a8)
verdict: NEEDS-CHANGE
cases: executed 134→135, red 1
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths (part 6)
needs-coordinator: yes. A `when: true` + `error:` refusal declared before an input refusal now gets a different answer on Entity Runtime. Either name it as a second epic exception, keep the base answer (that needs an ordering rule the acceptance forbids), or have validation refuse the shape.
```

**1. `git --no-pager diff --stat`**
```
 crates/generate/ess-entity-runtime/src/lib.rs      | 42 +++++-----
 .../tests/selection_precedence.rs                  | 96 ++++++++++++++++++++--
```
These are the implementor's edits and were already there when I started. My only file is new and untracked: `crates/generate/ess-entity-runtime/tests/adversary_selection_plan_w3_u1_pass1.rs`. I did not touch any implementation file.

**2. The case I added** (red now; no other cases written)

`adv_w3u1_p1_a_when_true_refusal_declared_before_an_input_refusal_keeps_its_base_answer` uses the billing example at ess/20. `CancelInvoice` is changed to `paused` (`when: true` + `error:`), then `rush-refused` (`when: rush == true` + `error:`), then `cancelled` (`when: rush == false`), then the existing `wrong-state`. The model validates, lowers and registers. The case asserts the base order and the base answers. Run alone:
```
assertion `left == right` failed: (lowered order, (answer with rush: true, answer with rush: false)) at the base lowering
  left: (["rush-refused", "paused", "cancelled", "wrong-state"], ("before load: rush-refused", "before load: paused"))
 right: (["paused", "rush-refused", "cancelled", "wrong-state"], ("before load: paused", "before load: paused"))
test result: FAILED. 0 passed; 1 failed
```

**3. Suite run**, after the case existed:
- Command: `CARGO_INCREMENTAL=0 cargo test -p ess-entity-runtime --locked --no-fail-fast`
- Result: 134 passed, 1 failed (my case), EXIT=101: `error: 1 target failed: -p ess-entity-runtime --test adversary_selection_plan_w3_u1_pass1`
- The before count of 134 matches the implementor's `after-suite.log`.
- The unit's own tests passed: the digest table, the exchanged-phases test and the default-last test.
- Clippy with `-D warnings` is clean, and rustfmt is clean on my file.

**4. Findings**

| # | file:line | verdict / origin | finding | what reaches it |
|---|---|---|---|---|
| F1 | new test `:218` | NEEDS-CHANGE / introduced | Base sort (`git show 1046621a8:…/lib.rs:1816-1828`): every `when:` + `error:` is category 0, in declaration order. The plan puts a `when: true` refusal after the input refusals. So on a model that validates, Entity Runtime's answer to `rush: true` changes from `paused` to `rush-refused`. This is not the epic's named exception. Origin is from reading the base, not from running it. | Validation accepts it (`when: true` is the one unconditional branch). No repository model writes `when: true`. |
| F2 | `ess-synth/src/rust/behaviour.rs:1211`, `ess-conformance/src/ts/explore.ts:1137` | CONFIRMED / introduced | The Rust emitter and the explorer (which claims to follow "Entity Runtime's order") still answer every `when:` + `error:` first, in declaration order. Before, Entity Runtime matched them and disagreed with the interpreter. Now it matches the interpreter (`refused_by_input` skips trivially-true guards) and disagrees with them. Found by reading the code, not by running it. | Same model as F1. |
| F3 | `tests/fixtures/lowered-definitions-table.tsv` | CONFIRMED / introduced | None of the models the table lowers has a held-state branch or a `when: true` refusal; I checked all 16 sources by grep. The shapes it does lower are input refusals, accepting `when:`, `external:`, the default and `wrong_state`, and the base and plan sorts order those the same way. So "no repository model's bytes moved" says nothing about the held-state ordering this story is about. Only the exchanged-phases test covers that. | Not applicable (it is about evidence). |

How F1 could be resolved:
- Record it as a second exception and flip the case to expect `rush-refused`.
- Have validation refuse `when: true` + `error:`. That would also close the wave-2 U1 pass-1 findings on the same shape.
- Keeping the base answer needs a third rule in `lower_command`, which the acceptance forbids.

**5. Attacked and not broken**
- **`ptr::eq` matching:** the plan and the compiled branches come from the same `&ResolvedCommand` (`service.operations()`). Every branch is lowered and every index lands in exactly one phase, so no match can be missed. The walk of a refused command does not sort.
- **Interaction with `decide_before_load`:** the phases before the input refusals only hold shapes the lowering already refuses. The named-exception swap pairs an accepting branch with a held-state branch, and both load the row before deciding, so pre-load answers don't change.
- **Named exception:** the finite prover only proves disjoint guards over required Boolean/enum input (it declines Optional). So one guard cannot evaluate Unknown on Entity Runtime while the other is True, and the answers stay the same.
- **Target-rule mutants:** the reversed-order test catches dropping either rule. In the default order, the digest table catches a misplaced `wrong_state`.
- **Digest table mechanics:** header counts, missing or unpinned models and the per-model line comparison all hold.
- **Other shapes:** I found no other base-vs-plan difference among shapes that validate and lower. Validation refuses `when:` + `error:` beside a subject or `replays:`.

**6. Paths written outside the worktree**
- `~/.cache/ess-selection-plan/w3-u1-scratch/adversary-p1-suite.log`
- `~/.cache/claude-tmp/claude-1000/-home-timo-beyond10x/649551ea-2c22-4172-bfe0-b0d47cfc46be/tasks/bqjbtrlk3.output`: empty, created by the harness for my disk-wait loop. That loop has exited.

The test binary went into the tree's own `target/`, which you are discarding.

**7. Findings block**
```findings
[
  {"file": "crates/generate/ess-entity-runtime/tests/adversary_selection_plan_w3_u1_pass1.rs", "line": 218, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a validating `when: true` + `error:` refusal declared before an input refusal is now lowered after it, so Entity Runtime answers `rush-refused` where the base answered `paused`, outside the epic's one named exception"},
  {"file": "crates/verify/ess-conformance/src/ts/explore.ts", "line": 1137, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the explorer's `isInputRefusal` and the Rust emitter (`ess-synth/src/rust/behaviour.rs:1211`) still answer every `when:` + `error:` first in declaration order, so on F1's model they now disagree with the Entity Runtime order they claim to mirror"},
  {"file": "crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv", "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "no pinned lowered model has a held-state branch or a `when: true` refusal, so the table cannot move under the reordering this story makes and is evidence only for refusal, accepting, external, default and wrong-state order"}
]
```
