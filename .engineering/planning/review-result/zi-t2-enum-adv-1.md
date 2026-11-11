---
format: aep.planning-md/3
id: review-result:zi-t2-enum-adv-1
kind: review-result
status: active
title: Turn 2 adversary pass 1 on story:an-input-guard-on-an-enum-variant-leaves-the-related-row-family-arrangeable
relations:
- reviews: story:an-input-guard-on-an-enum-variant-leaves-the-related-row-family-arrangeable
revision: 1
---
unit: story:an-input-guard-on-an-enum-variant-leaves-the-related-row-family-arrangeable, commit 5375f809d8 (tree zi-t2-2, plus one new untracked test file)
verdict: holds
new failing cases: none
mutants survived: none run (one is likely to survive, see finding 1)
cases: executed 2→13, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (`adv/`)
needs-coordinator: none

1. **Diff stat:** `git diff --stat` is empty. `git status --short` shows one new file, `?? crates/verify/ess-conformance/tests/adversary_t2_enum_guard_inputs.rs`. It is a test file, and no other path changed.

2. **Cases added:** 11 tests in `~/.local/state/worktree/trees/b10x/ess/zi-t2-2/crates/verify/ess-conformance/tests/adversary_t2_enum_guard_inputs.rs`. They are green now. Each one, except the timestamp case and the two refusal-text cases, checks three things: no refusal, the `no-decision` scenario exists, and every scenario passes on the interpreted target. What they cover:
   - an `all:` guard over two enum inputs;
   - an `any:` guard over two of three variants (the scenario is sent `Partial`);
   - two disjoint refusals over two enum inputs (sent `Unsupported` and `High`);
   - integer, boolean and two-input comparison guards (`amount < 1000`, `urgent == false`, `amount >= limit`);
   - `defined(note)` on an optional input;
   - `due_at < now`: the send is checked to be `kind: Unsupported` with a positive `NowOffset`;
   - a guard negation cannot satisfy (`any:` over both variants): ESS-SYNTH-003 naming `not-permitted`;
   - the exact wording in the acceptance: ``every such input taken first by `not-permitted` (kind == Satisfied)`` together with `none of: kind == Satisfied, kind == Unsupported`.

   Run 1 (`adv/run1.txt`, exit 101) and the first timestamp run (`adv/run3.txt`) were errors in my own test, not findings:
   - The compiler refuses a refusal declared after the `no-decision` branch and two enum refusals that overlap (`conflicting_declaration`). That layout cannot be written, so I removed that case.
   - The connective syntax was wrong at first.
   - The interpreted target answers every send relative to `now` with `Unsupported`. That is a limitation of the interpreter and not part of this unit, so the timestamp case checks the send instead.

3. **Final run:** `build.sh adv/run4-suite.txt <tree> test -p ess-conformance --locked --test adversary_t2_enum_guard_inputs --test missing_related_row_beside_input_refusal`
   - `test result: ok. 11 passed; 0 failed` / `test result: ok. 2 passed; 0 failed` / `exit 0` / `slot B waited 80s ran 19s`

4. **Findings:**
   - **Weak test (note):** `missing_related_row_beside_input_refusal.rs:111` checks only the prefix ``every such input taken first by ` `` of the ESS-SYNTH-003 text. A change that named the wrong refusal or the wrong guard would stay green. My case `adversary_t2_enum_guard_refusal_text_names_branch_and_guard` now pins the full wording from the acceptance. I did not run the mutant (no cargo runs left), so "would survive" is my reading of the code, not a measurement. Nothing reaches this in practice; it is a gap in the suite, not a defect.

5. **Attacked and could not break:**
   - The directed send really reaches the `exists: false` branch at run time: interpreted target, all scenarios pass, in every case except timestamps.
   - Guards with connectives, several enum inputs, integer, boolean, optional and two-input comparisons.
   - A guard that negation cannot satisfy gives ESS-SYNTH-003, not ESS-SYNTH-008.
   - The refusal text matches the acceptance and the shape in `subject_fact.rs:3888`.
   - Byte pins: the committed `synthesis-suite-bytes-table.tsv` diff only adds the new fixture's row, and the implementor's other three pin runs exited 0. I did not rerun them, because of the 4-run budget.
   - A test-only change that moves the input refusal below the branch: the compiler rejects that order, so nobody can reach it.

6. **Path written outside the worktree:** `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/an-input-guard-on-an-enum-variant-leaves-the-related-row-family-arrangeable/adv/`, which holds `run1.txt`–`run3.txt`, `run4-suite.txt`, their `.touch` files and `tmp/`.

7. **Findings block:**
```findings
- file: crates/verify/ess-conformance/tests/missing_related_row_beside_input_refusal.rs
  line: 111
  category: mutant
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the ESS-SYNTH-003 test asserts only the "taken first by `" prefix, so a change naming the wrong refusal or guard would stay green; adversary_t2_enum_guard_refusal_text_names_branch_and_guard now pins the full wording
```
