---
format: aep.planning-md/3
id: review-result:zero-issues-w1-a-create-or-update-writer-is-no-proof-a-stored-reference-is-present-adversary-1
kind: review-result
status: archived
title: Turn 1 adversary pass 1 on story:a-create-or-update-writer-is-no-proof-a-stored-reference-is-present
revision: 3
transitions:
- {from: "active", to: "archived", at: "2026-11-10T05:55:40Z", actor: "human:timo", revision: 2}
---
unit: story:a-create-or-update-writer-is-no-proof-a-stored-reference-is-present (tree zi-t1-7, HEAD 13560f3065 plus my uncommitted test cases)
verdict: holds
new failing cases: 1 (crates/verify/ess-conformance/tests/adversary_i_create_or_update.rs::adv_the_ess30_hint_below_the_one_rule_is_not_a_promise_ess30_breaks)
mutants survived: none

```
 .../tests/adversary_i_create_or_update.rs          | 69 ++++++++++++++++++++++
 1 file changed, 69 insertions(+)
```
Only the test file changed. I also touched the modification time of 37 `crates/*/src/{lib,main}.rs` files, with no content change, to force a rebuild (see finding 1).

1. **The shared build slots link other trees' code (needs the coordinator).** My first three runs through `build.sh` recompiled only ess-conformance. The binaries they produced (`targets/A` and `targets/B` `.../adversary_i_create_or_update-8b5db41fa0b36d07`) fail the unit's own acceptance cases: "ess/31 must not prove the stored agent present: ()". So they linked an ess-domain built from another unit's tree. Cargo hashes workspace crates by path relative to the workspace root, and freshness is checked by file time, so two trees on the same version share artifacts. After touching the crate roots, run 4 (`adv/suite.txt`) rebuilt ess-yaml through ess-conformance from zi-t1-7, and both acceptance cases pass. The implementor's g1–g4 did recompile ess-domain from this tree, so they are trustworthy. g5–g8 (the byte pins) recompiled only ess-conformance and should be rerun with a forced rebuild. This affects every unit of the wave.
2. **Red case: a hint below ess/27 promises something ess/30 does not keep.** Command: `build.sh adv/suite.txt <tree> test -p ess-conformance --locked --test adversary_i_create_or_update`, exit 101, 12 passed, 1 failed. At ess/26 the hint on `Rename` says "declare `format: ess/30`, which needs none here". At ess/30 the same command is refused because "`integrity.board.Move/relinked` writes `node_id` without first refusing…". Cause: `reference_integrity.rs:208` builds the plan in the document's own format, and the hint arm at `:56` trusts that result. The fix I'd suggest is to compute the proof under ess/30's plan when the format is below it.
   - Introduced by this change: at the base, the ess/30 proof ignored order, so the hint was true then.
   - Probably unreachable: ess/26 already refuses this `Move` command (`conflicting_declaration`, "declare `format: ess/27`"). So the wrong hint only appears on a document that is refused for another reason. That makes it INFEASIBLE, a backlog item, not a blocker.
3. **New passing case, kept:** `adv_a_held_state_writer_is_no_proof_at_ess30` pins a writer that is not a creator. A branch selected by `when_subject:` answers at step 3, before `no-node` at step 4, and is named. The suite did not cover that before.
4. **Attacked and could not break:**
   - A refusal carrying `when:` beside `exists: false`, which would make validation and synthesis disagree: validation refuses that shape outright (`conflicting_declaration`).
   - Writer identity by `std::ptr::eq`: the writer always comes from `command.outcomes`.
   - Synthesis and domain building different plans: `PrecedencePlan::new` and `Composition::of` produce the same plan.
   - Several writers before and after the refusal: run 4 shows `moved` is not named while `relinked` is.
5. **Case count:** 11 → 13 (11 is the implementor's g1), 1 red. My first isolated runs were against the stale build from finding 1, so the first valid red is from run 4, the whole test target.
6. **Written outside the worktree:** everything is under `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/a-create-or-update-writer-is-no-proof-a-stored-reference-is-present/adv/`: `append.rs`, `orig.rs`, `probe.rs`, `run1.sh`, `cases.txt`, `cases2.txt`, `probe.txt`, `suite.txt`. I also added build output to the shared slots `targets/A` and `targets/B`.

```findings
- file: ~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/build.sh
  line: 41
  category: concurrency
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: the shared CARGO_TARGET_DIR slots reuse another worktree's crate artifacts (relative-path hash, mtime freshness), so a gate run that does not recompile a crate may measure another unit's code; this unit's g5-g8 need a forced-rebuild rerun
- file: crates/specify/ess-domain/src/command/reference_integrity.rs
  line: 208
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: below the one rule the proof is computed in the document's own precedence order, so the hint promises ess/30 needs no branch where ess/30 refuses, though only on a document ess/26 already refuses
```
