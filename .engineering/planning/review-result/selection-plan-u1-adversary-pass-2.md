---
format: aep.planning-md/3
id: review-result:selection-plan-u1-adversary-pass-2
kind: review-result
status: active
title: Adversary pass 2, wave 2 unit U1 (selection plan design and type)
relations:
- reviews: story:selection-plan-design-and-type
revision: 1
---
unit: story:selection-plan-design-and-type, adversary pass 2 (the last one for U1), uncommitted working tree on unit/selection-plan-u1-design-and-type (base b7be935f87)
verdict: NEEDS-CHANGE
cases: executed 1749→1751, red 2
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths (part 6)
needs-coordinator: no

The new `Rank::Unguarded` holds on every composition I checked, and C1 and C2 now pass. Two of the marker rules on the revised design page give a different answer from the interpreter on models that validate. The R1 rule is new in this correction and gets wrong a case the pass-1 wording got right; the R3 rule has been on the page since pass 1. The T check does fail on an unpinned model, but its re-pin instruction is incomplete.

**1. Diff stat**
```
 crates/specify/ess-compiler/src/ir.rs    | 2 ++
 crates/specify/ess-domain/src/command.rs | 1 +
 2 files changed, 3 insertions(+)
```
These two tracked files are the implementor's edits. My only addition is one untracked test file, `crates/verify/ess-conformance/tests/adversary_selection_plan_u1_pass2.rs`, which is one new test binary. No implementation file was touched. I applied rustfmt to my file after the red runs; the change is whitespace only.

**2. Cases added** (both models validate; both cases were run alone before the suite)

| case | the page's rule it drives | red output, run alone |
|---|---|---|
| `adv_u1_p2_r1_…` :131 | `selection-plan.md:60-62`: "`unknown_instance:` answers where the addressed row is not held: at once … on a row-set command (`addressed_row`)" | ``the page's rule gives `unknown-book` where the interpreter answers `refused` `` |
| `adv_u1_p2_r3_…` :223 | `selection-plan.md:71-72`: "On a row-set command it also answers at once where no accepting branch acting on the row moves from the state it holds" | ``the page's rule gives `already-closed` where the interpreter answers `annotated` `` |

- **R1 model:** the ess#462 adversary's input-guarded upsert variant of `Shelve`. Request: an absent book, `mode: new`, and an open library of the curator.
- **R3 model:** the row-set `Close` plus an `annotated` branch that updates the attempt where `delay > 0`. Request: an attempt already closed.

**3. Suite run, after the cases existed**
- `cargo test -p ess-domain --locked`: EXIT=0, 1438 passed.
- `cargo test -p ess-compiler --locked`: EXIT=0, 307 passed.
- `cargo test -p ess-conformance --locked --no-fail-fast --test precedence_plan_override --test adversary_selection_plan_u1_pass1 --test adversary_selection_plan_u1_pass2`: EXIT=101.
  - `precedence_plan_override`: 1 passed.
  - `adversary_selection_plan_u1_pass1`: 3 passed.
  - `adversary_selection_plan_u1_pass2`: 2 failed.

The 1749 before-count comes from the implementor's logs `c1-gate-domain.log`, `c1-gate-compiler.log` and `c1-gate-conformance.log`.

**4. Findings** (they cover the working tree named in the header)

1. **R1: the new `unknown_instance:` rule is wrong next to an upsert** (`selection-plan.md:60-62`).
   - **Why:** on a row-set command where a creation takes the absent identity, `addressed_row` hands the missing row to the row sets (`creation_takes_absent`, `execute.rs:1066`, ess#462). So the row-set refusal answers before `unknown_instance:`.
   - **What reaches it:** the ess#462 adversary's validating variant. No repository model.
   - **History:** the pass-1 wording ("where the row the branch selected by a later phase addresses is not held") gave the interpreter's answer here.
   - **Fix:** one clause, "unless a creation takes the absent identity".
2. **R3: the `wrong_state:` rule for row sets is wrong next to a branch that does not move** (`selection-plan.md:71-72`).
   - **Why:** `addressed_row`'s `may_act` (`execute.rs:1053`) counts any accepting branch that updates, preserves or deletes as able to act. So no `wrong_state:` answers at once, and the updating branch answers.
   - **What reaches it:** a validating variant. No repository model.
   - **History:** this wording dates from pass 1, so it is the unit's, not the correction's.
   - **Fix:** "where every accepting branch acting on the row moves, and none from the state it holds".
3. **T: re-pinning as the failure message says still leaves the test red** (`selection_precedence_table.rs:30-31,202-203,211-218`).
   - The write mode does not update the `MODELS` (194) and `COMMANDS` (329) constants, which the test asserts against the fixture.
   - So after re-pinning with a new model, "every pinned model is read" fails until a person edits both constants. Neither the message nor the page's "Pinned" section says so.
   - I could not make this red without adding a model to the tree.

**5. Attacked and could not break**
- **The `Unguarded` rank:**
  - **Before `select`:** on stored-reference and row-set commands, everything the interpreter answers before `select` is in `Existence`, in `HeldState`'s `wrong_state:` (row set) or in `PresentRelated`'s lead (stored `exists: false`).
  - **Elsewhere:** the refusal still leads `HeldState`, which is unchanged from before. That covers held-state commands, ordered input-related commands (ess/22 with `wrong_state:`, and several rows) and commands with no related read.
  - **With `existing_instance:`:** where creations take different identities, `existence::existing` looks ahead through `select`, and the refusal is taken first there too.
  - **Validation:** it admits no held-state guard on either kind of command.
  - **Count:** at most one unguarded refusal per command, because two unconditional branches are refused.
- **The T check:** it compares the label set `models()` finds with the pinned set, and runs before the comparison of pinned lines (the implementor's probe log shows it red).
- **Other page claims I checked against the code and found true:**
  - the phase and interpreter tables;
  - the `existing_instance:` look-ahead rule;
  - the `wrong_state:` look-ahead rule for stored references and for ordered present-related refusals;
  - "Validation admits no held-state branch on either command";
  - the claim about `related_guard::orders_present_refusals`.

**6. Paths written outside the worktree**, all in `~/.cache/ess-selection-plan/u1-scratch/`:
- `adv2-red-r1.log`
- `adv2-red-r3.log`
- `adv2-suite-domain.log`
- `adv2-suite-compiler.log`
- `adv2-suite-conformance.log`

The worktree tool also recorded my lease, `adversary-u1-pass2`, which I have released. Build output went to the worktree's own `target/`.

**7. Findings block**
```findings
[
  {
    "file": "crates/verify/ess-conformance/tests/adversary_selection_plan_u1_pass2.rs",
    "line": 159,
    "category": "contract-drift",
    "severity": "warning",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "the revised page says unknown_instance answers at once on every row-set command, but where a creation takes the absent identity addressed_row defers to the row sets and the interpreter answers the row-set refusal"
  },
  {
    "file": "crates/verify/ess-conformance/tests/adversary_selection_plan_u1_pass2.rs",
    "line": 254,
    "category": "contract-drift",
    "severity": "warning",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "the page says wrong_state answers at once on a row-set command where no accepting branch moves from the held state, but addressed_row counts an updating branch as able to act, so the interpreter answers that branch instead"
  },
  {
    "file": "crates/specify/ess-compiler/tests/selection_precedence_table.rs",
    "line": 202,
    "category": "judgement",
    "severity": "note",
    "verdict": "CONFIRMED",
    "origin": "introduced",
    "message": "re-pinning with SELECTION_PRECEDENCE_TABLE_WRITE as the failure message instructs leaves the MODELS and COMMANDS constants stale, so the test stays red after a model is added until both are edited by hand"
  }
]
```
