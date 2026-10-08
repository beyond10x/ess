---
format: aep.planning-md/3
id: review-result:selection-plan-u1-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave 2 unit U1 (selection plan design and type)
relations:
- reviews: story:selection-plan-design-and-type
revision: 1
---
unit: story:selection-plan-design-and-type, uncommitted working tree on unit/selection-plan-u1-design-and-type (base b7be935f87)
verdict: NEEDS-CHANGE
cases: executed 1744→1748, red 2
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 9 paths (part 6)
needs-coordinator: yes. Decide where a `when: true` + `error:` refusal answers on a row-set or stored-reference command: change the plan, or have validation refuse the combination.

I found two commands that validate where the plan's order differs from the interpreter's answer. Both need a refusal written `when: true` + `error:`. No repository model writes one, so the pinned table cannot show either.

**1. Diff stat**
```
 crates/specify/ess-compiler/src/ir.rs    | 2 ++
 crates/specify/ess-domain/src/command.rs | 1 +
 2 files, 3 insertions(+)
```
These two tracked files are the implementor's edits, the same as when I started. My additions are untracked test files only:
- `crates/specify/ess-domain/tests/adversary_selection_plan_u1_pass1.rs`
- `crates/verify/ess-conformance/tests/adversary_selection_plan_u1_pass1.rs`

No implementation file was touched. I wrote one more case, `crates/specify/ess-compiler/tests/adversary_selection_plan_u1_pass1.rs`, and deleted it (see part 2). After the red runs I applied rustfmt to my two files; the change is whitespace only.

**2. Cases added**

| case | asserts | now | red output, run alone |
|---|---|---|---|
| conformance `adv_u1_control_…` | on the committed stored-reference fixture, plan and interpreter both answer `blocker-missing` (checks the harness) | green | — |
| conformance `adv_u1_c1_…` :162 | stored-reference fixture plus a guarded `completed` and a `declined` refusal written `when: true`; dangling blocker: plan answer == interpreter answer | **red** | `the plan reads `declined` first where the interpreter answers `blocker-missing`` |
| conformance `adv_u1_c2_…` :235 | row-set fixture `Close` plus a guarded `closed` and a `when: true` refusal; attempt already closed: plan == interpreter | **red** | `the plan reads `declined` first where the interpreter answers `already-closed`` |
| domain `adv_u1_one_row_…` :19 | `Composition::of(Join)` does not order its refusals (one row read by two branches) | green; catches a mutant (finding 3) | — |
| compiler `adv_u1_the_plan_reads_the_format…` | deleted: its model does not validate (`wrong_state` beside `when_related` is refused below ess/22), so it was not a finding | — | — |

Both red models pass `Specification::assemble` and `compile`.

**3. Suite run, after the cases existed**
- `cargo test -p ess-domain --locked`: EXIT=0, 1437 passed.
- `cargo test -p ess-compiler --locked`: EXIT=0, 307 passed.
- `cargo test -p ess-conformance --locked --no-fail-fast --test precedence_plan_override --test adversary_selection_plan_u1_pass1`: EXIT=101.
  - `precedence_plan_override`: 1 passed.
  - adversary target: 1 passed, 2 failed (c1, c2).

The 1744 before-count comes from the implementor's gate logs (`gate-domain.log`, `gate-compiler.log`, `gate-conformance.log`).

**4. Findings** (they cover the working tree named in the header)

1. **C2 is not documented anywhere.**
   - **What was measured:** `crates/verify/ess-conformance/tests/adversary_selection_plan_u1_pass1.rs:270`. On a row-set command the interpreter's `addressed_row` (`execute.rs:1016-1060`) answers `wrong_state` before the head of `select` reads the `when: true` refusal.
   - **What the page and plan say:** the design page's `HeldState` row (`docs/design/selection-plan.md:26`) says "that `when:` refusal first … then `wrong_state:`", and the plan follows it.
   - **What reaches it:** any author's model; it validates. No repository model.
   - **Possible fixes:** put `wrong_state` before that refusal on row-set commands, or have validation refuse `when: true` + `error:` beside a row set. Choosing is the coordinator's call.
   - Verdict NEEDS-CHANGE; origin introduced.
2. **C1 is the stored-reference case the page already concedes** (`selection-plan.md:90`). It contradicts the page's own line "The plan reproduces the interpreter" (:81) and the brief's instruction to stop rather than choose.
   - Measured at `crates/verify/ess-conformance/tests/adversary_selection_plan_u1_pass1.rs:183`.
   - The only branch the plan's `HeldState` lead slot can hold is a `when: true` refusal: validation refuses `error:` beside a `when:` subject (only `external:` may compensate) and beside `replays:`.
   - The page's wording, "an `error:` branch with no guard", describes a bare `error:` branch, which is `Otherwise` and which the plan puts in `Default`.
   - Verdict CONFIRMED; origin introduced.
3. **A dedup mutant in `Composition::of` survives the unit's suite** (`precedence.rs:301-302`).
   - Its only caller is `precedence_classification.rs:426`, run on two models where the branch count already equals the row count.
   - Counting branches instead of rows would move `Join`'s `banned-from-joining` to `PresentRelated`. My domain case catches this; it is green now.
   - Verdict CONFIRMED; origin introduced.
4. **The table test cannot see a model added to the tree** (`selection_precedence_table.rs:202,206`).
   - The loop walks pinned labels only, and the `MODELS` constant counts the fixture, not the tree.
   - A model added by a later story would never be compared, against the acceptance line "lists every command of every repository model".
   - Fix: assert that the set of labels `models()` finds equals the pinned set.
   - I could not make this red without adding a YAML model, which six other repository walkers would also read.
   - Verdict CONFIRMED; origin introduced.

**5. Attacked and could not break**
- The override:
  - It is thread-local and restored on unwind and when nested.
  - The constructor honours it.
  - No consumer crate builds on another thread.
- No wildcard arm in `shape` or `From<&OutcomeCondition>`.
- The digest lines equal `base-digests.tsv` (194 lines, identical).
- `existing_instance` placement, with or without a stored `via`, matches the interpreter.
- Held-state guards beside present-related refusals: validation refuses `when_subject`, `when_subject_state` and `when_state_changes` beside `when_related` and row sets.
- `HeldState` before `Accepting`: the ess#486 check covers all four held-state conditions.
- Several input rows, the stored `via` with `several`, and the ess/22 gate all match. The format gate is never decisive for a model that validates, so that mutant changes nothing.

**6. Paths written outside the worktree**, all in `~/.cache/ess-selection-plan/u1-scratch/`:
- `adv-base-sorted.txt`
- `adv-fixture-model-lines.txt`
- `adv-red-c1.log`
- `adv-red-c2.log`
- `adv-domain-case.log`
- `adv-compiler-case.log`
- `adv-suite-domain.log`
- `adv-suite-compiler.log`
- `adv-suite-conformance.log`

The worktree tool also recorded my lease, `adversary-u1`, which I have released. Build output went to the worktree's own `target/`.

**7. Findings block**
```findings
[
  {
    "file": "crates/verify/ess-conformance/tests/adversary_selection_plan_u1_pass1.rs",
    "line": 270,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "on a validating row-set command the plan reads a `when: true` refusal before `wrong_state`, while the interpreter's `addressed_row` answers `wrong_state` first, and the design page does not record this divergence"
  },
  {
    "file": "crates/verify/ess-conformance/tests/adversary_selection_plan_u1_pass1.rs",
    "line": 183,
    "category": "acceptance",
    "severity": "note",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "on a validating stored-reference command the plan answers a `when: true` refusal where the interpreter answers the stored `exists: false`, contradicting the page's \"The plan reproduces the interpreter\""
  },
  {
    "file": "crates/specify/ess-domain/src/command/precedence.rs",
    "line": 302,
    "category": "mutant",
    "severity": "note",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "dropping the row dedup in `Composition::of` would order `Join`'s refusal at step 5 and no existing case would fail"
  },
  {
    "file": "crates/specify/ess-compiler/tests/selection_precedence_table.rs",
    "line": 206,
    "category": "judgement",
    "severity": "note",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "the table compares pinned labels only, so a model added to the tree is never pinned and the test stays green"
  }
]
```
