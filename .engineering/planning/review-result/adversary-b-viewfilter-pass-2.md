---
format: aep.planning-md/3
id: review-result:adversary-b-viewfilter-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.41 unit viewfilter
relations:
- reviews: story:view-filters-bind-identity-and-link-fields
revision: 1
---
unit: viewfilter (beyond10x/ess#193) correction 1, uncommitted working tree on base e20db842e7 in ess-b-viewfilter
verdict: CONFIRMED
cases: executed 1525→1539, red 3
origin: introduced 2 / pre-existing 0 / undecided 1
wrote-outside-worktree: 3 paths (see part 6)
needs-coordinator: none

## 1. Diff stat (worktree)

```
 crates/verify/ess-conformance/src/synthesize.rs    | 341 +++++++++++++++++----
 .../ess-conformance/src/synthesize/aggregate.rs    |  94 +++++-
 .../ess-conformance/src/synthesize/paging.rs       |   7 +-
?? src/synthesize/identity.rs, tests/view_filter_identity.rs, tests/adversary_view_filter_pass1.rs  (earlier states')
?? crates/verify/ess-conformance/tests/adversary_view_filter_pass2.rs   (mine, the only path I wrote in the tree)
```

No non-test path touched by this pass.

## 2. Cases added (tests/adversary_view_filter_pass2.rs), run alone

`cargo test -p ess-conformance --test adversary_view_filter_pass2`: 11 passed, 3 failed, EXIT=101
(first run hit a filter-syntax typo, `&&` in compact form; fixed to structured `all:` before any finding was counted).

| case | asserts | now |
|---|---|---|
| a_by_id_read_scoped_to_its_account_that_ignores_the_id_fails (:674) | `all: [id == param.id, account_id == param.account]`: a target answering every row of `param.account` fails | red |
| an_owner_of_at_most_one_row_is_never_given_two (:683) | with `owns … cardinality: one`, no scenario posts two entries under one account instance | red |
| a_link_filtered_list_without_the_identity_kills_ignoring_the_owner (:720) | `account_id == param.account`, fields `[account_id, memo]`: a target ignoring the filter fails | red |
| 11 others | see part 5 | green |

Red lines, verbatim:

```
[OfAccountIgnoresId] pass every one of 6 scenario(s)
    "ledger.book.EntriesPerAccountMemo/aggregate: 3 entries under account-2",
    "ledger.book.EntriesPerAccountMemo/aggregate: 2 entries under account-5",
    "ledger.book.Entry/transition/void/by/ledger.book.VoidEntry/voided: 2 entries under account",
    "ledger.book.PostEntry/outcome/posted: 2 entries under account",
    "ledger.book.VoidEntry/outcome/voided: 2 entries under account",
[MemosIgnoresAccount] pass every one of 6 scenario(s)
```

## 3. Suite

`cargo test -p ess-conformance --no-fail-fast`: 1536 passed, 3 failed (all in adversary_view_filter_pass2), `error: 1 target failed`, EXIT=101. Before: 1525 (implementor's test-c1.log).

## 4. Findings

- synthesize.rs:4696. The same-owner rule is skipped whenever the filter reads the link. A filter reading both identity and link gets only a companion under another owner, and the link clause excludes that companion whatever the target does. So `GET /accounts/{a}/entries/{id}` ignoring the id passes. Reached by any nested by-id view. Fix: when the filter reads the identity, arrange the companion under the subject's owner, and add the other-owner companion only in addition. CONFIRMED / introduced / warning.
- synthesize.rs:2966 (under_owner, used by created_owned at :3014) and aggregate.rs:1079. F1, F2 and F4 give one owner several rows with no check of `relation.cardinality`. With `owns … cardinality: one`, the suite posts 2–3 entries to one account. A target with a unique owner key (the usual 1:1 table) then fails its own spec's suite. Fix: share an owner only for `cardinality: many`. For `one`, owner-rows is the by-id answer, so nothing is lost. CONFIRMED / introduced / warning.
- synthesize.rs:4671 (`identified` gate in arrange_matching). A link-filtered list that projects the link and not the identity gets no other-owner row. A target ignoring the filter passes. At base this view was refused, so it is newly reached; the gate itself predates the unit. Fix: count the projected link as telling rows apart. CONFIRMED / undecided / warning.

## 5. Attacked, not broken

- ordered link list: ignoring the owner or reversing the order is killed
- paging with a link filter: ignoring the owner or the page is killed. Go parity holds
- group_by [link, memo, note], [memo, link], [link, state], [link, counterparty]: every dropped key is killed
- two count_distinct over link-typed fields: each counts-rows mutant is killed
- `defined(account_id)`: the subject is now Contains, and a target answering nothing is killed. No existing expectation flipped (suite green apart from my cases)
- determinism: identical JSON on two syntheses over all pass-2 views
- TypeScript runtime: verdict parity with the Rust runner, and every mode caught, for correct / by-id-owners / ignores-memo / paged-ignores-account
- set_effects callers: arrange_toward_filter still passes no owner. set_effects*.rs green in the suite
- mutants of the correction: dropping `all_scoped`, the aggregate-input share condition or the F1 same-owner rule is killed by pass-1 cases (PerAccountMemoIgnoresAccount, DistinctCountsRows, ByIdReturnsOwnersRows). Reasoned from those cases, not run on mutated copies
- not attacked: an owner whose lifecycle guards creation. arrange_owner and under_owner both pin the owner to its initial state, so they agree

## 6. Paths outside the worktree

- ~/.cache/b10x-target/ess-b-viewfilter (assigned build dir), including tmp/adversary-view-filter-pass2 (TS packages)
- ~/.cache/ess-wave-n2/viewfilter/adv2/red-run.log, suite.log, review.md
- Go parity packages under $TMPDIR (~/.cache/claude-tmp/ess-go-parity-viewfilter2-*), removed by the harness (none left)

```findings
[
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4696, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "a filter reading both the identity and the link gets only an other-owner companion, so a nested by-id read that ignores the id passes"},
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 2966, "category": "property", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "owners are shared by by-id, ordered and aggregate arrangements regardless of cardinality, so an owns-cardinality-one owner is given two or three rows"},
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4671, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "undecided", "message": "a link-filtered list projecting the link but not the identity gets no other-owner row, so a target ignoring the filter passes"}
]
```
