---
format: aep.planning-md/3
id: review-result:adversary-d-overlap-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.42 unit overlap
relations:
- reviews: story:overlapping-accepting-guards-have-declared-precedence
revision: 1
---
unit: overlap-precedence (beyond10x/ess#217), tree ess-d-overlap: uncommitted diff on 4e7c3867e
verdict: NEEDS-CHANGE
cases: executed 1615→1621, red 5
origin: introduced 1 / pre-existing 4 / undecided 0
wrote-outside-worktree: 3 paths (below); base copy and base build dir deleted
needs-coordinator: rows 2 and 3 are red at base only because the feature is missing there. They are gaps in this story's acceptance, not base defects; decide the routing.

## 1. Diff stat

```
 crates/generate/ess-entity-runtime/src/lib.rs      |   4 +-
 .../tests/input_guard_overlap.rs                   | 139 ++++++++++++++----
 .../ess-conformance/src/interpret/execute.rs       |  19 ++-
 crates/verify/ess-conformance/src/synthesize.rs    | 155 ++++++++++++++++++++-
 docs/design/input-guard-overlap-precedence.md      |  63 ++++++++-
 website/docs/guides/write-a-specification.md       |   5 +
 website/docs/reference/predicates.md               |   9 ++
 7 files changed, 360 insertions(+), 34 deletions(-)
?? crates/verify/ess-conformance/tests/adversary_overlap_precedence_pass1.rs   (mine, untracked, test only)
```
This is the implementor's diff, unchanged. My only path is the untracked test file.

## 2. Cases (crates/verify/ess-conformance/tests/adversary_overlap_precedence_pass1.rs)

| line | case | now | base |
|---|---|---|---|
| 159 | every_reordering_of_three_overlapping_accepting_branches_fails_the_suite: 5 permutations of a/b/c, including last-declared-wins, each fail the suite | green | red |
| 195 | a_narrow_decimal_overlap_is_witnessed_or_reported | RED | red |
| 240 | the_interpreter_answers_the_first_declared_branch_when_a_later_guard_is_undecided | RED | red |
| 278 | an_external_branch_after_an_accepting_guard_is_sent_an_input_the_guard_does_not_claim | RED | red |
| 317 | a_branch_reachable_between_ladder_values_is_not_refused_as_claimed_entirely | RED | green |
| 344 | a_plain_overlap_in_a_command_with_a_state_guarded_sibling_is_witnessed | RED | red |

Red lines, verbatim, from running the file alone before the suite:
```
---- a_narrow_decimal_overlap_is_witnessed_or_reported stdout ----
the overlap of `small` and `flagged` is neither sent nor reported.
sent to small: [ ( { "amount": Literal { value: Number( Number( Exact { units: 11, scale: 0, binary: 11.0 } ) ) } }, "small" ) ]
refusals: []
notes: []

---- the_interpreter_answers_the_first_declared_branch_when_a_later_guard_is_undecided stdout ----
  left: Err("the guard of `demo.orders.PlaceOrder/b` is Unknown over this input: Compare { left: Fact(FactPath(note)), op: Eq, right: Literal(Text(\"rush\")) }")
 right: Ok({"demo.orders.PlaceOrder/a"})

---- an_external_branch_after_an_accepting_guard_is_sent_an_input_the_guard_does_not_claim stdout ----
`declined` is required at an input `small`, declared first, claims: [ ( "demo.orders.PlaceOrder/outcome/declined", { "amount": Literal { value: Number( Number( Exact { units: 1, scale: 0, binary: 1.0 } ) ) } } ) ]

---- a_branch_reachable_between_ladder_values_is_not_refused_as_claimed_entirely stdout ----
`b` is refused as claimed entirely by `a`: [
    "ESS-SYNTH-003 refusal[ESS-SYNTH-003]: outcome demo.orders.PlaceOrder/b has no scenario `demo.orders.PlaceOrder/outcome/b`\n  no candidate of the 14 tried satisfies `(amount > 11 and amount < 13) outside a ((amount <= 11.5 or amount >= 11.6)), the accepting branch declared first`\n  help: write the branch's condition over values a candidate can carry, or supply a fixture for it",
]

---- a_plain_overlap_in_a_command_with_a_state_guarded_sibling_is_witnessed stdout ----
no input in the overlap of `low` and `high` is sent requiring `low`.
Report invocations (scenario, level, required): high 10 high; low 1 low; enriched -1 enriched; ringing/refreshed/preserved 1
refusals: []
```
The stateful case's output is condensed from the full printout. The level values are as printed.

## 3. Suite (run after the cases existed)

`cargo test -p ess-conformance --no-fail-fast`: 228 result lines, summed passed 1616, failed 5, ignored 3; `EXIT=101`. The only failing target is `-p ess-conformance --test adversary_overlap_precedence_pass1`, which contains the 5 red cases above. Last result line: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.60s`.
The `before` figure, 1615, is derived: 1621 minus my 6 cases. It is not a separate deselected run.
`cargo xtask generate --check`: `projections are up to date`, EXIT=0.
Clippy on the new test file (`-D warnings`) is clean. The file is rustfmt-clean.

## Findings

| file:line | severity | verdict | origin | finding | fix |
|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/synthesize.rs:8764 | warning | NEEDS-CHANGE | pre-existing | A decidable, nonempty overlap that the ladder misses (Decimal `10<a<12` / `11<a<13`) produces no overlap input, no refusal and no note. A target that answers the later branch passes. This is brief class 2, and the docs call it intended ("adds nothing and is not refused") | when `found` is None or `candidates` errs at :8754, record a synthesis note naming both branches |
| crates/verify/ess-conformance/src/synthesize.rs:8567 | warning | NEEDS-CHANGE | pre-existing | In a command with any state-guarded sibling, two plain accepting `when:` branches that overlap (`5<level<10`) are never sent an overlap input. `keep` calls `selects_branch(..., None, ...)`, which errs to false, and `selected_in_state` needs uniqueness. The docs exclude only a `when:` beside `when_subject:`, so this drop is silent and undocumented | route overlap rows through the held state, or note the pair as unwitnessed |
| crates/verify/ess-conformance/src/interpret/execute.rs:362 | warning | CONFIRMED | pre-existing | The interpreter still evaluates every `when:` and returns Undecidable when a later guard is Unknown (absent optional `note`), although the first-declared branch `a` holds and the declared precedence determines the answer. What reaches it: any `execute` caller with the optional absent. No synthesized scenario was found that sends one | stop at the first accepting branch that holds once no refusal can hold, or evaluate the refusals first and short-circuit |
| crates/verify/ess-conformance/src/synthesize.rs:3994 | warning | CONFIRMED | pre-existing | The inject-fault scenario for external `declined`, declared after `small: 0<=amount<100`, sends `amount: 1`, which `small` claims. The interpreter under Forced answers `declined`. Entity Runtime keeps both in category 1 by source index (lib.rs:1482), so it would answer `small`. The Entity Runtime half is inferred from the sort and was not run. The agreement between accepting and external branches that the story asks about does not hold | decide the order between accepting and external branches, then make reach_external refute earlier accepting guards, or state and test the opposite in Entity Runtime |
| website/docs/reference/predicates.md:815 | note | INFEASIBLE | introduced | The docs (and design doc :125) say a branch "every input of which an earlier one claims" is refused. The new Shadow wording also fires when only the ladder is covered: `b: 11<a<13` after `a: a<=11.5 or a>=11.6` is refused although 11.55 reaches `b`. The refusal text itself says "no candidate of the 14 tried", which is accurate. The guards are constructed, and nobody was shown to write them | reword the docs to "every candidate tried"; no code change needed |

## 4. Judgement findings
None beyond the table. All findings cover the working tree on 4e7c3867e.

## 5. Attacked and could not break
- Mutant targets: last-declared-wins, every other order of 3 pairwise-overlapping branches (the specificity orders among them), and refusal-after-accepting (#178's existing case). The suite fails each one.
- Shadow as a breaking change: `ess conform` exits SUCCESS with refusals printed. A truly covered later branch was, at base, a scenario that Entity Runtime already failed, so the refusal replaces a scenario that was wrong under Entity Runtime.
- Class 3 (only the first pair witnessed): each later sibling gets its own overlap search. B∩C outside A is witnessed for B.
- Byte stability: `generate --check` is clean. The committed suites are unchanged.
- Refusal + accepting, two accepting, accepting + default: the interpreter and Entity Runtime agree (unit tests plus the permutation case).

## 6. Paths written outside the worktree
- ~/.cache/ess-wave-n2/overlap/adv1/suite-conformance.log
- ~/.cache/ess-wave-n2/overlap/adv1/generate-check.log
- ~/.cache/ess-wave-n2/overlap/adv1/base-run.log
- this review.md. Deleted: ~/.cache/ess-wave-n2/overlap/adv1/base (a `git archive` copy of the base) and ~/.cache/b10x-target/ess-d-overlap-base.
- Build dir used: ~/.cache/b10x-target/ess-d-overlap (shared with the implementor, not deleted).

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 8764
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: a decidable accepting overlap no ladder value reaches is skipped with no refusal or note, so a target answering the later branch passes
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 8567
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: two overlapping plain accepting when branches in a command with a state-guarded sibling are never sent an overlap input, and nothing says so
- file: crates/verify/ess-conformance/src/interpret/execute.rs
  line: 362
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: the interpreter errors Undecidable on a later branch's Unknown guard although the first-declared accepting branch holds and the precedence decides the answer
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 3994
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: an external branch declared after an accepting guard is required at an input that guard claims, where Entity Runtime's source-order selection answers the accepting branch
- file: website/docs/reference/predicates.md
  line: 815
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the docs promise a shadow refusal only when every input is claimed, but it fires when only the ladder candidates are claimed
```
