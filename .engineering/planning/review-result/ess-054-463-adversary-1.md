---
format: aep.planning-md/3
id: review-result:ess-054-463-adversary-1
kind: review-result
status: active
title: 'Adversary pass 1, #463 struct-identity row-set selectors'
tags:
- ess-0.54.0
relations:
- reviews: story:feature-request-463
revision: 1
---
I found one gap, and two red cases show it. The unit's new code held up under every other attack. For a selector that pins only some members of the identity (`exists: true`), the suite does not catch a target that keys rows by the pinned member and keeps only the newest record.

```
unit: ess-054-463-adv — worktree ess-054-463-20261006, base 09ec1dc33 plus its uncommitted diff (8 files)
verdict: CONFIRMED (warning)
cases: executed 32→42, red 2
origin: introduced 0 / pre-existing 0 / undecided 1
wrote-outside-worktree: 9 paths (part 6)
needs-coordinator: yes: hold this unit for the gap or move it to its own story; and the two slips in part 6
```

**1. Diff stat.** `git --no-pager diff --stat` shows the same 8 files the unit handed me (249+/45-). My only addition is the untracked `crates/verify/ess-conformance/tests/adversary_463_pass1.rs`. No non-test path is mine.

**2. Cases added** (in `~/.local/state/worktree/trees/b10x/ess/ess-054-463-20261006/crates/verify/ess-conformance/tests/adversary_463_pass1.rs`)

| Case | Asserts | Now |
|---|---|---|
| `adv_partial_member_selector_latest_wins_unique_target_fails` | with `at.region` alone and `exists: true`, a target keying by region, newest open wins, fails some scenario | **red** |
| `adv_plain_field_selector_latest_wins_unique_target_fails_at_ess22` | the same shape over a plain `region` field at `ess/22`, through no gate this unit added | **red** |
| `adv_partial_member_selector_whole_struct_target_fails` | a target selecting by the whole struct fails the partial suite | green |
| `adv_nested_struct_identity_members_…` | `Place{region, slot{row, col}}`: the suite synthesizes, the interpreter passes, and dropping any of the 3 members fails | green |
| `adv_optional_member_…` / `adv_integer_member_beside_string_…` | an `Optional<String>` member, and an `Integer` member beside a `String` one: dropping it fails | green |
| `adv_integer_member_alone_is_refused_as_unscoped` | both branches are refused as "selects rows by no equality" | green |
| `adv_bulk_update_where_on_identity_member` | `instances: {where: at.region == …}` is refused at validate (`unobservable_fact`) | green |
| `adv_go_…` / `adv_typescript_…` | Go and TS give the reference verdicts on the new fixture, for both the healthy and the dropped-member target | green |

Red output, each case run alone first:
```
panicked at …/adversary_463_pass1.rs:502:5:
a target keying shelves by `at.region` alone, newest wins, passes every scenario:
panicked at …/adversary_463_pass1.rs:534:5:
a region-keyed newest-wins target passes every scenario
```
The `sealed` steps open shelves in this order: `(at.region-1, at.shelf-1, sealed)`, `(place.region, at.shelf-2, mode-2)`, `(place.region, at.shelf-3, sealed)`. The `stored` steps open the first two only. The selected row is always opened after the decoy that matches its region, so a newest-wins target answers correctly every time.

**3. Suite run** (after the cases existed): `cargo test -p ess-conformance --locked --offline --no-fail-fast --test row_set_struct_identity --test adversary_w1_1_pass1 --test identity_changing_updates --test adversary_463_pass1` exited 101.

| Target | Passed / failed |
|---|---|
| adversary_463_pass1 | 8 / 2 |
| adversary_w1_1_pass1 | 12 / 0 |
| identity_changing_updates | 9 / 0 |
| row_set_struct_identity | 11 / 0 |

The "before" count of 32 comes from this same run with my file left out.

**4. Finding**
- **`crates/verify/ess-conformance/src/synthesize/row_set.rs:1116`** (`arrange_rows`): every decoy is arranged before the selected rows. A partial-member selector (`at.region` only, `exists`/count 1) therefore never puts a region-matching unsealed row after a sealed one. A target that keys records by the pinned member, newest replacing older, passes both branches. A first-wins target is caught; a newest-wins one is not. The story's `partial_member_selector_selects_many` only covers `count: {gte: 2}`.
  - **Verdict:** CONFIRMED, warning.
  - **Origin:** undecided. It reproduces at `ess/22` on a plain field, through paths every change in this unit gates on `identity_selectors`, but I did not run it against the base.
  - **What reaches it:** the story names the partial selector as a supported composition (story §4 "Composes with").
  - **Fix, not applied:** on branches that need a selected row, also arrange the decoy that refutes the non-key conjunct after the selected rows.

**5. Attacked and could not break**
- A target ignoring one identity member, at both 2 and 3 members, nested included.
- A whole-struct target against a partial selector.
- `Optional` and `Integer` members.
- An `Integer`-only selector, which is refused as unscoped.
- The `instances:` sibling, which validate refuses, so the shared gates are never reached.
- Go and TS parity.
- `ess/22` byte guards (adversary_w1_1_pass1, 12/12).
- `identity_changing_updates` (9/9).

**6. Paths written outside the worktree, and two slips**
- `~/.cache/ess-054-463/adv/{run.sh, case.log, case2.log, case3.log, case4.log, suite.log, tmp/node-compile-cache}`
- `/dev/shm/ess-054/ess-054-463-adv`: cleaned with `cargo clean`, 505.5 MiB.
- `~/.cache/b10x-go-cache/ess-054-463-adv`: removed.
- **Slip 1:** I ran `rustfmt --edition 2024` on my file, and it followed `mod support_go;` and rewrote one import line in `crates/verify/ess-conformance/tests/support_go/mod.rs`. I put the line back by hand about a minute later. `git status` now matches the tree as I received it. Another agent holding this tree could have seen the change during that minute.
- **Slip 2:** I ran `go clean -cache` once without `GOCACHE` set. That emptied the shared default Go build cache (`~/.cache/go-build`), not just mine. It is only a cache, so builds refill it, but it was not mine to clear.

**7.**
```findings
- file: crates/verify/ess-conformance/src/synthesize/row_set.rs
  line: 1116
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: decoys are always arranged before the selected rows, so a target that keys records by a partial selector's member with the newest record winning passes every scenario of an exists/count-1 branch
```
