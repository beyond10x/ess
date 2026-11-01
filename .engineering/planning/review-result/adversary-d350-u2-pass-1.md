---
format: aep.planning-md/3
id: review-result:adversary-d350-u2-pass-1
kind: review-result
status: active
title: 'D350 U2 adversary pass 1: related-guard rows follow bindings, wrong state beside external'
relations:
- reviews: story:related-guard-wrong-state-beside-external
- reviews: story:an-outcome-arrangement-follows-the-bindings-its-steps-set-off
revision: 1
---
unit: D350 U2 (story:an-outcome-arrangement-follows-the-bindings-its-steps-set-off, story:related-guard-wrong-state-beside-external), unit commit 462151a7d on unit/ess-d350-u2, plus the adversary's uncommitted test file
verdict: CONFIRMED (1 warning, story 7; story 6 not broken)
cases: executed 68→75, red 3
origin: introduced 1 / pre-existing 0 / undecided 1
wrote-outside-worktree: 1 path
needs-coordinator: whether the exchanged-order requirement in story 7's acceptance makes F1 hold the unit; no source format reaches F1

**1. Diff stat.** `git --no-pager diff --stat` is empty; one untracked test file, `crates/verify/ess-conformance/tests/adversary_d350_u2_pass1.rs`. No implementation file touched.

**2. Cases** in `crates/verify/ess-conformance/tests/adversary_d350_u2_pass1.rs`:
- Green, story 6: `s6_a_binding_that_moves_the_arranged_row_back_is_followed` (:133), `s6_a_binding_that_removes_the_arranged_row_is_followed` (:165), `s6_a_binding_two_hops_down_that_rewrites_the_row_is_followed` (:177). In all three `Give/outcome/not-flagged` is refused as ESS-SYNTH-003 naming `demo.d.Lower`, the binding and the writing command, and no `Give` scenario fails.
- Green, story 7: `s7_the_moved_ess20_model_passes_in_every_order` (:355): `claim-search/related.yaml` passes in the real order and 8 exchanged orders.
- Red, story 7, one defect (F1): `s7_unknown_identity_names_a_present_list_with_existence_before_the_related_row` (:318), `s7_wrong_state_beside_external_passes_under_exchanged_orders` (:304), `s7_wrong_state_with_an_accepting_when_before_the_external_passes` (:339). Output: `list_id sent as Literal { value: Text("00000000-0000-4000-8000-9e9c01d66df7") }`; `[related_row<->existence] demo.desk.CheckPick/outcome/wrong-state: Failed ... expected: ["outcome = wrong-state"], observed: ["outcome = no-list"]`.
- Dropped as invalid fixtures, not findings: `affects:` beside `creates:`; `unknown_instance:` beside `when_related:` before ess/27; `wrong_state:` with only `exists: false`.

**3. Suite run** through the wave wrapper: `test -p ess-conformance --locked --no-fail-fast --test adversary_d350_u2_pass1` plus the 14 unit targets. Exit 101. Unit targets 68 executed, 3 ignored in `_open`; the adversary file 4 passed, 3 failed. The 68 is the per-target sum of the same run. Peak `target/` 2G, lowest / 87G.

**4. Findings.**
- F1 (CONFIRMED, introduced, warning), `crates/verify/ess-conformance/src/synthesize/related_guard.rs:1940`: `unknown_beside_external` always compares the related row's `exists: false` against `Phase::Existence`. Without `unknown_instance:` the unknown-identity scenario's expected branch is `wrong-state`, answered at HeldState; with RelatedRow exchanged with Existence, `no-list` is read after Existence but before HeldState, no list is arranged and the target answers `no-list`. Fix (not applied): pass the precedence phase of the expected branch. Reached only through the test hook `with_phase_order`; below ess/27 every format reads RelatedRow first and ess/27 moves `exists: false` after HeldState. CONFIRMED rather than INFEASIBLE because the acceptance names exchanged phase orders.
- O1 (out of scope, undecided, note): `raise-on-lowered/binding/delivery` comes back `Unsupported` ("binding invocation budget exhausted") when the second delivery's `Raise` refuses under an unbounded `on_failure: retry`. Raise has no related guard; likely not the unit's doing; not checked on a base tree.

**5. Attacked, not broken:** story 6 rewrites by state move, by removal and two bindings down, with the refusal naming step, binding and command; story 7 held-state refusals under 8 exchanged orders; the moved ess/20 model's bytes. Not run: an `affects:` rewrite (no valid fixture); a binding set off by the own arrangement or a row beside.

**6. Paths outside the worktree:** `the wave scratch directory u2/adv-scratch/cargo.log`; the wrapper's lines in `steps.tsv`.

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize/related_guard.rs","line":1940,"category":"acceptance","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"unknown_beside_external tests the related row's exists:false against Phase::Existence instead of the expected branch's phase, so with RelatedRow and Existence exchanged the wrong-state unknown-identity scenario sends a literal list_id and the target answers no-list (reached only through with_phase_order)"},
{"file":"crates/verify/ess-conformance/tests/adversary_d350_u2_pass1.rs","line":91,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"undecided","message":"out of scope - a binding delivery scenario is synthesized where the second delivery's command refuses under an unbounded retry policy and the interpreter reports Unsupported (invocation budget exhausted)"}
]
```
