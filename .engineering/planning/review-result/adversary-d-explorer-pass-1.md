---
format: aep.planning-md/3
id: review-result:adversary-d-explorer-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.42 unit explorer
relations:
- reviews: story:explorers-decide-input-guards-before-wrong-state
revision: 1
---
unit: explorer (story:explorers-decide-input-guards-before-wrong-state, beyond10x/ess#235); working tree ~/.local/state/worktree/trees/b10x/ess/ess-e-explorer, uncommitted diff on 300bfd3f5
verdict: NEEDS-CHANGE
cases: executed 1720→1723, red 1
origin: introduced 0 / pre-existing 1 / undecided 0
wrote-outside-worktree: 7 paths (logs and this review under ~/.cache/ess-wave-n2/explorer/adv1/)
needs-coordinator: whether the #235 class ("a branch that moves nothing answers in every state") covers an arranged external branch, which would pull a pre-existing defect into this unit

## 1. `git --no-pager diff --stat` (tracked; unchanged from the implementor's)

```
 crates/verify/ess-conformance/src/go/explore.go    | 53 ++++++++++++++++++++--
 crates/verify/ess-conformance/src/ts/explore.ts    | 51 +++++++++++++++++++--
 .../fixtures/explore-external-exits-target.mjs     |  9 +++-
 .../tests/fixtures/explore-external-exits.yaml     | 26 +++++++++--
 .../fixtures/explore_external_exits_target.go      | 15 ++++--
 docs/design/mutation-audit-and-model-runner.md     | 42 ++++++++++++-----
 6 files changed, 167 insertions(+), 29 deletions(-)
```

All of those are the implementor's. What I added is untracked, and every file is a test or a test fixture:
`crates/verify/ess-conformance/tests/adversary_explore_order_pass1.rs`,
`crates/verify/ess-conformance/tests/fixtures/{explore-adv-order.yaml, explore-adv-overlap.yaml, explore_adv_order_target.go, explore-adv-order-target.mjs}`,
`crates/generate/ess-entity-runtime/tests/adversary_explorer_order_pass1.rs`. I changed no implementation file.

## 2. The cases I added

| Case | What it asserts | Now |
|---|---|---|
| `adversary_explore_order_pass1::an_arranged_external_that_moves_nothing_answers_in_a_state_no_move_starts_from` | Over `explore-adv-order.yaml` (`Close`: moving default `closed`, external `bounced` that moves nothing, `wrong-state`), a target in Entity Runtime's order passes in both languages | **RED** |
| `ess-entity-runtime adversary_explorer_order_pass1::entity_runtime_answers_an_arranged_external_that_moves_nothing_in_a_wrong_state` | The oracle. Billing `CancelInvoice` with an external `blocked` added, lowered and run through entity-core: Issued/no verdict → `cancelled`; Issued/verdict → `blocked`; Paid/no verdict → `wrong-state`; **Paid/verdict → `blocked`** | green |
| `adversary_explore_order_pass1::a_refusal_answers_before_an_arranged_external_verdict` | `Rate` (refusal `too-high`, external `throttled`, default `rated`) passes, and `too-high` and `throttled` are both reached. A mutant applied to the emitted explorer, where the refusal carries the eligible externals, is caught in both languages | green (covers a mutant) |
| `adversary_explore_order_pass1::two_moving_overlapping_guards_on_a_stranded_subject_answer_wrong_state` | `explore-adv-overlap.yaml` reaches `Resolve/wrong-state` only through the new `stranded && all move` branch. The unmutated explorer reaches it with no failure; the mutant that drops the branch does not | green (covers a mutant) |
| `adversary_explore_order_pass1::origin_probe_base_explorer` (`#[ignore]`) | The red case, run with `explore.go`/`explore.ts` from `300bfd3f5`, with `Rate` not exposed | red at base, so the finding is pre-existing |

The red output, captured from the first run of this file on its own
(`cargo test -p ess-conformance --test adversary_explore_order_pass1`), verbatim:

```
thread 'an_arranged_external_that_moves_nothing_answers_in_a_state_no_move_starts_from' (2884231) panicked at crates/verify/ess-conformance/tests/adversary_explore_order_pass1.rs:258:9:
assertion `left == right` failed: typescript: a target in Entity Runtime's order is reported as disagreeing
{"ambiguous":[],"excluded":[],"excludedOutcomes":[],"executed":95,"external":[{"cause":"the mail gateway bounces the closing notice","outcome":"exploreadv.desk.Close/bounced","reach":"reached"},{"cause":"the rating service throttles","outcome":"exploreadv.desk.Rate/throttled","reach":"reached"}],"failure":{"message":"outcome: the target answered `bounced`, the specification says `wrong-state` (step 7: exploreadv.desk.Close {\"ticket_id\":\"00000000-0000-4000-8000-000000000001\"})","originalLength":35,"seed":2,"shrinkComplete":true,"trace":["exploreadv.desk.Open {\"size\":1131}","exploreadv.desk.Close {\"ticket_id\":\"00000000-0000-4000-8000-000000000001\"}","exploreadv.desk.Open {\"size\":100}","exploreadv.desk.Open {\"size\":2}","exploreadv.desk.Open {\"size\":3}","exploreadv.desk.Close {\"ticket_id\":\"00000000-0000-4000-8000-000000000004\"} [external: bounced]","exploreadv.desk.Close {\"ticket_id\":\"00000000-0000-4000-8000-000000000001\"}"]},"reached":[...],"sequences":2,"steps":60,"undetermined":[],"unreached":[]}
  left: "failed: explore: seed 2: outcome: the target answered `bounced`, the specification says `wrong-state` (step 7: exploreadv.desk.Close {\"ticket_id\":\"00000000-0000-4000-8000-000000000001\"})"
 right: "ok"
test result: FAILED. 1 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 5.95s
```

The origin probe, run with `--ignored` against the base sources:
`left: "failed: explore: seed 1: outcome: the target answered `bounced`, the specification says `wrong-state` (step 5: exploreadv.desk.Close {\"ticket_id\":\"00000000-0000-4000-8000-000000000001\"})"`.

## 3. The suite, run after the cases existed

`cargo test -p ess-conformance --no-fail-fast` (log `~/.cache/ess-wave-n2/explorer/adv1/suite-conformance.log`):

```
test an_arranged_external_that_moves_nothing_answers_in_a_state_no_move_starts_from ... FAILED
test result: FAILED. 2 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 11.61s
error: 1 target failed:
    `-p ess-conformance --test adversary_explore_order_pass1`
EXIT=101
```

Totals from summing the `test result:` lines: 1722 passed, 1 failed, 4 ignored, so 1723 executed. The
1720 before is the implementor's `conformance-2.log`, which exited 0. The suite is red, and that is
the result this pass was looking for.

`cargo test -p ess-entity-runtime --test adversary_explorer_order_pass1`: `test result: ok. 1 passed; 0 failed`.

Both of my files pass `cargo clippy --test <file> -- -D warnings` and `rustfmt --check`.

## 4. Findings

| file:line | severity | verdict | origin | finding | fix |
|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/go/explore.go:992 (also :1047, and ts/explore.ts:739, :786) | warning | NEEDS-CHANGE | pre-existing | `wrongState()` takes the `wrong_state` branch without looking at the eligible external branches. An arranged external branch that moves nothing is the branch Entity Runtime selects: the lowering sorts it before the default, and `admit_state` admits a branch that makes no move. So on a subject resting where no move starts, the explorer expects `wrong-state` and a correct target answers the external. With no arrangement in force, the explorer also never offers that external in such a state. | In `wrongState()`, take `exploreEligible(...)` as it is: externals that move nothing stay eligible in a stranded state, and those that move are already filtered out. Return `{take, outcome: wrong, externals}` so that `exploreChoices` offers them, and a forced arrangement expects the external. Apply the same change in both languages. |

- **What was measured:** `adversary_explore_order_pass1.rs:258`, red in the TypeScript lane with exit 101. The Go lane ran in the same call; its result was not asserted separately. The Entity Runtime half is green: `ess-entity-runtime/tests/adversary_explorer_order_pass1.rs`, Paid invoice with the verdict set → `blocked`.
- **What reaches it:** any command that has `wrong_state:` plus an external branch that moves nothing, explored against a target whose arranged verdict stays in force for the command. The explorer itself assumes that (`exploreChoices`: "an earlier arrangement … may still hold"), and so do the repository's own doubles (`explore-external-exits-target.mjs` keeps `forced` until the scenario ends). The billing example has no such command today; I added one in the oracle.
- **Why it matters here although it is pre-existing:** the changelog says "a branch that moves nothing answers in every state", and the #235 decision says `wrong_state` answers "only when the branch selected … moves from a state its move does not start from". An arranged external is a selected branch, so this is the same class, one sibling path over.

## 5. Attacked and could not break

- **Go vs TS vs Entity Runtime:** refusal + wrong_state, two overlapping refusals, a refusal overlapping an accepting branch (refusals sorted first by the lowering at `ess-entity-runtime/src/lib.rs:1476-1493`), an accepting branch that moves nothing in a wrong state, and one that moves in a wrong state all agree. The unit's modes and my Close/Rate cases cover them.
- **`when_subject_state`, `when_subject`, `unknown_instance`:** the explorer still excludes these commands (`explore.go:616`, #221), so this unit changes nothing there.
- **Refusal before the "no record" check:** this matches the written order (step 2 before step 3) and entity-core `decide_before_load`, which refuses before it loads. I could not break it. One aside: entity-core also answers a *default* refusal (an `otherwise` with `error:`) before loading, and the explorer skips that draw as "no record". That skip is correct for the explorer, but the written order does not state it.
- **Refusal vs an arranged external:** the explorer answers the refusal first, as Entity Runtime does. The base explorer failed my `Rate` case (`the target answered too-high, the specification says throttled`), so the unit fixed that. Before my case, nothing in the suite pinned it; it is pinned now, with a mutant.
- **The changed `explore-external-exits` fixture:** its purpose, the overlapping-guards exit taking `throttled`, survives because its branches are now accepting ones. The refusal version could not stay: with refusals first, a refusal always holds and `throttled` becomes unreachable. The only coverage lost is "an input refusal next to an external", which the old test never asserted. My `Rate` case now covers it.
- **The new `stranded && all move` overlap branch:** nothing in the unit's fixtures reached it. My overlap case covers it with a mutant.
- **Determinism:** the unit's test asserts that TypeScript and Go give identical results for `correct`. My red case reproduces at seed 2 (seed 1 at base).
- **The three classes:** equal witnesses and only-first do not apply to the explorer's decision. Silent drops: every undecided draw is still recorded in `ambiguous`.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/explorer/adv1/red-conformance.log
- ~/.cache/ess-wave-n2/explorer/adv1/red-conformance-2.log
- ~/.cache/ess-wave-n2/explorer/adv1/origin-probe.log
- ~/.cache/ess-wave-n2/explorer/adv1/overlap.log
- ~/.cache/ess-wave-n2/explorer/adv1/er-oracle.log
- ~/.cache/ess-wave-n2/explorer/adv1/suite-conformance.log
- ~/.cache/ess-wave-n2/explorer/adv1/review.md (this file)
- Build output went to the assigned `~/.cache/b10x-target/ess-e-explorer`. The tests' scratch directories (`$TMPDIR/ess-adv-explore-order-*`) are deleted by the tests themselves.

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/go/explore.go
  line: 992
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: wrongState() answers the wrong_state branch and drops the eligible external branches that move nothing, so an arranged external that Entity Runtime selects and admits in a state no move starts from is reported as a disagreement (the same in explore.ts:739).
```
