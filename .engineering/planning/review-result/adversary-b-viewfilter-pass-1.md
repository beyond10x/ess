---
format: aep.planning-md/3
id: review-result:adversary-b-viewfilter-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.41 unit viewfilter
relations:
- reviews: story:view-filters-bind-identity-and-link-fields
revision: 1
---
unit: viewfilter (beyond10x/ess#193), uncommitted working tree on base e20db842e7 in ess-b-viewfilter
verdict: CONFIRMED
cases: executed 1518→1525, red 4
origin: introduced 2 / pre-existing 0 / undecided 2
wrote-outside-worktree: 4 paths (see part 6)
needs-coordinator: none

## 1. Diff stat (worktree)

```
 crates/verify/ess-conformance/src/synthesize.rs    | 203 ++++++++++++++++-----
 .../ess-conformance/src/synthesize/aggregate.rs    |  73 +++++++-
 .../ess-conformance/src/synthesize/paging.rs       |   7 +-
?? crates/verify/ess-conformance/src/synthesize/identity.rs      (implementor's)
?? crates/verify/ess-conformance/tests/view_filter_identity.rs   (implementor's)
?? crates/verify/ess-conformance/tests/adversary_view_filter_pass1.rs  (mine, the only path I wrote)
```

## 2. Cases added (tests/adversary_view_filter_pass1.rs)

Ledger fixture target with one mutant at a time. Red:

| case | asserts | now |
|---|---|---|
| a_by_id_read_answering_the_owners_rows_fails | `id == param.id` target answering every entry of the named entry's account fails | red: `ByIdReturnsOwnersRows passes every one of 6 scenario(s)` |
| count_distinct_over_a_link_field_is_told_from_count | `count_distinct: account_id` (grouped by memo) target counting rows fails | red: `DistinctCountsRows passes every one of 7 scenario(s)` |
| grouping_by_link_and_value_drops_neither_key | `group_by: [account_id, memo]` target grouping by memo only fails | red: `PerAccountMemoIgnoresAccount passes every one of 7 scenario(s)` |
| an_ordered_read_of_an_owners_rows_is_synthesized | `account_id == param.account` + `order_by: [memo asc]` is synthesized | red: 3x `refusal[ESS-SYNTH-014] ... can put 1 row(s) in it: comparing two rows needs two ledger.book.Entry`s` |

Green: `id != param.id` answering nothing is killed; determinism (two syntheses, identical JSON); Go runtime parity for None / ByIdReturnsOwnersRows / PerAccountMemoIgnoresMemo.

## 3. Suite

`cargo test -p ess-conformance --no-fail-fast`: passed 1521, failed 4 (all in adversary_view_filter_pass1), `error: 1 target failed`, EXIT=101. Before: 1518 passed (implementor's test-final.log).

## 4. Findings

- synthesize.rs:4590-4594 (arrange_matching identity case): the companion is created under a different owner, so a by-id read that returns the owner's rows passes. Reached by any owned entity with a by-id view. Fix: arrange the companion under the subject's owner so only the identity differs. CONFIRMED / introduced / warning.
- aggregate.rs:1066-1068: owners are shared only when the link is in `group_by`; with `count_distinct` over the link every row gets its own owner, so distinct == count in every group. Fix: share owners by planned link value whenever the view reads the link. CONFIRMED / introduced / warning.
- aggregate.rs:1063-1070 / pattern: for `[account_id, memo]` the memo values never repeat across owners (A,A,A | B,B2), so grouping by memo alone passes. The pattern code is unchanged by the diff; not run at base. CONFIRMED / undecided / warning.
- synthesize.rs:4542-4567 (arrange_ranked): an ordered, link-filtered list is still refused (ESS-SYNTH-014): no companion can be arranged under the subject's owner. Issue part 2 names comparisons on link fields. Not run at base. CONFIRMED / undecided / warning.

## 5. Attacked, not broken

- cross-entity tokens of one type: unreachable. The language refuses `id != account_id` (a bare right side is a literal), so tokens only meet a param bound from the same field
- `!=` identity filter: Excludes subject; the answer-nothing mutant is killed
- determinism: identical in-process
- Go runtime: verdict and request parity
- not attacked: TS runtime, `paging:` page/size with an id filter, the `defined(<link>)` flip (not run at base)

## 6. Paths outside the worktree

- ~/.cache/b10x-target/ess-b-viewfilter (assigned build dir)
- ~/.cache/ess-wave-n2/viewfilter/adv1/red-run.log, red-run-final.log, suite.log, review.md
- Go parity packages under $TMPDIR (~/.cache/claude-tmp/ess-go-parity-*), removed by the harness

```findings
[
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4590, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "the by-id companion is created under a different owner, so a target answering every row of the named row's owner passes the suite"},
  {"file": "crates/verify/ess-conformance/src/synthesize/aggregate.rs", "line": 1066, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "owners are shared only for a link in group_by, so count_distinct over the link equals count in every group and a target counting rows passes"},
  {"file": "crates/verify/ess-conformance/src/synthesize/aggregate.rs", "line": 1063, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "undecided", "message": "for group_by [link, value] the planned value never repeats across owners, so a target grouping by the value alone passes"},
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4542, "category": "acceptance", "severity": "warning", "verdict": "CONFIRMED", "origin": "undecided", "message": "a link-field filtered view with order_by is still refused (ESS-SYNTH-014) because no second row can be arranged under the subject's owner"}
]
```
