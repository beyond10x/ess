---
format: aep.planning-md/3
id: review-result:zi-t2-listlit-adv-1
kind: review-result
status: active
title: Turn 2 adversary pass 1 on story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation
relations:
- reviews: story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation
revision: 1
---
unit: story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation (HEAD a46122ba42, tree zi-t2-8)
verdict: weak
new failing cases: 2 (adversary_t2_list_literal_attack.rs::adversary_t2_list_literal_negation_beside_base_refuted_leaf, ::adversary_t2_list_literal_negation_beside_many_leaves_survives_the_bound)
mutants survived: none

**Tree changes (`git status --short`):**
```
 M crates/verify/ess-conformance/src/witness.rs
?? crates/verify/ess-conformance/tests/adversary_t2_list_literal_attack.rs
```
The `witness.rs` change is not mine. Another session edited it at 19:26:47: it adds a `type ListFill` alias and uses it in place of the tuple type, which changes no behaviour. Run 1 was against HEAD exactly. Run 2 ran on HEAD plus that alias edit. My only file is the untracked test file, which is uncommitted.

**Runs:** 2 of the 4 allowed.
- **Run 1, my file alone:** `build.sh adv/run1-own.txt <tree> test -p ess-conformance --locked --test adversary_t2_list_literal_attack` → `test result: FAILED. 3 passed; 2 failed`, `exit 101`.
- **Run 2, the suite afterwards:** same script, outfile `adv/run2-suite.txt`, with `--no-fail-fast` and six targets: adversary_u4_list_literals, the four byte-pin targets, and mine.
  - adversary_u4_list_literals: 12 passed.
  - synthesis_suite_bytes_table: 2 passed.
  - synthesis_suite_bytes_table_cli_read: 1 passed.
  - adversary_related_values_all_bytes: 1 passed.
  - external_beside_held_guard: 17 passed.
  - mine: 2 failed.
  - `exit 101`. No byte pin moved.
- **Case count:** 33 before (from the implementor's green and pin outputs) → 38 after, 2 red.

**Findings.** Both are outside the three variants the acceptance names, but inside the story title ("beside other inputs and negation"), so backlog items. Origin is undecided because I did not run them at the base. The base has none of this code path, so I expect both to be red there too.

1. **Negated literal beside an input whose base value refutes the guard.** Guard: `all [exists telephony, exists voice, not exists fax, level > 3]`, which is variant 2 plus one negated literal. The branch is not witnessed: `ESS-SYNTH-003: no candidate of the 39 tried satisfies …` (test file line 239, failing assertion `not witnessed: [".../updated-with-telephony"]`).
   - Cause: the leave-one-out fills are laid over the base input only (witness.rs:1086). Every candidate that raises `level` has all lists full, so it also holds `fax`.
   - What reaches it: any model guard that both excludes a literal and requires another input away from its base value.
2. **The bound drops the candidate the branch needs.** Same three list children, beside eight leaves `aN < 100` that the base satisfies. Result: `ESS-SYNTH-003: no candidate of the 141 tried satisfies …`, plus eight `ESS-SYNTH-022` refusals for `updated` (line 254).
   - The control, `negation_beside_base_satisfied_leaf` (one such leaf), is green and its negated-child mutant is caught. So the cause is the bound, not the guard.
   - Cause: the full fills over the 65 hosts come first and use up `added < MAX_CANDIDATES` (witness.rs:1081 and :1102). The leave-one-out fills are then never appended.
   - What reaches it: any command whose leaves push the walk to 64 candidates.
   - Fix (not applied): append the leave-one-out fills first, or reserve room for them inside the bound. Lay them over the same hosts as the full fill, which also fixes finding 1.
3. **Judgement (note):** module rule 4 says "At most MAX_CANDIDATES distinct inputs per outcome". The new block can append up to 64 inputs on top of the walk's 64 (141 were tried in finding 2). The looseness was already there in `extend_paired`.

**Attacked and could not break:**
- Variant 3's guarded branch under the full `check` (the unit's own case only asserts the else branch).
- Negation nested two levels deep (`not any [not exists, exists]`).
- Negation beside a leaf whose base value satisfies it, with the negated-child mutant caught.
- The four byte-pin targets: no bytes moved.

I left all three green cases in place. Each pins a case the unit's suite does not.

**Written outside the worktree:**
- `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation/adv/run1-own.txt` and `run1-own.txt.touch`
- `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation/adv/run2-suite.txt` and `run2-suite.txt.touch`
- I created the directory `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation/tmp`.

Test file: `~/.local/state/worktree/trees/b10x/ess/zi-t2-8/crates/verify/ess-conformance/tests/adversary_t2_list_literal_attack.rs`

```findings
- file: crates/verify/ess-conformance/src/witness.rs
  line: 1086
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: leave-one-out fills sit on the base input only, so a negated literal beside an input whose base refutes the guard (level > 3) is never witnessed (ESS-SYNTH-003)
- file: crates/verify/ess-conformance/src/witness.rs
  line: 1102
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: full fills over every host exhaust the added < MAX_CANDIDATES budget first, so with 64 walk candidates the leave-one-out candidate a negated literal needs is dropped (ESS-SYNTH-003, 8x ESS-SYNTH-022)
- file: crates/verify/ess-conformance/src/witness.rs
  line: 76
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: undecided
  message: rule 4 promises at most MAX_CANDIDATES inputs per outcome, yet the new block appends up to MAX_CANDIDATES more on top of the walk (141 tried in one search)
```
