---
format: aep.planning-md/3
id: review-result:adversary-d-prefix-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.42 unit prefix
relations:
- reviews: story:diff-classifies-a-newtype-prefix-change
revision: 1
---
unit: diff-prefix (beyond10x/ess#219), working tree ess-d-prefix on 4f7398154 (uncommitted implementor diff + adversary file)
verdict: NEEDS-CHANGE
cases: executed 220→228, red 1
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths (see part 6)
needs-coordinator: none

## 1. Diff stat

```
 crates/verify/ess-diff/src/change.rs          | 67 +++++++++++++++++++++++++--
 crates/verify/ess-diff/src/diff.rs            | 36 +++++++++++++-
 crates/verify/ess-diff/tests/clock_reading.rs | 11 +++++
 docs/design/wire-presence-json-prefix.md      |  3 +-
 website/docs/reference/formats.md             |  2 +-
 website/docs/reference/spec-versions.md       |  2 +-
 6 files changed, 112 insertions(+), 9 deletions(-)
```

All six tracked paths are the implementor's. The adversary added one untracked test file,
`crates/verify/ess-diff/tests/adv1_text_prefix.rs`, and changed no other file.

## 2. Cases added (`crates/verify/ess-diff/tests/adv1_text_prefix.rs`, 8 cases)

| case | asserts | now |
|---|---|---|
| `a_prefix_change_is_ess_diff_11_vocabulary_and_ess_diff_10_refuses_it` | added/removed/changed: `minimum_format()==11`, delta format `ess-diff/11`, the `/10` writer refuses, the `/10` reader returns `UnsupportedFormatVersion` | RED |
| `case_only_unrelated_and_one_character_replacements_take_the_comparison_relation` | `Ord_→ord_` changed, `ab→ba` changed, `o→ord_` narrowed, `ord_→o` expanded, `ord_a→ord_b` changed | green |
| `a_prefix_moved_beside_an_alphabet_reports_both_each_with_its_own_relation` | alphabet-changed and prefix-changed (narrowed) are both reported, with no unclassified | green |
| `a_prefix_moved_beside_a_representation_reports_both` | prefix-removed and representation-changed are both reported | green |
| `a_prefix_dropped_by_a_kind_change_is_the_kind_change_alone` | newtype with a prefix changed to a struct gives only kind-changed, with no residual | green |
| `a_reading_change_beside_a_prefix_change_reports_both_and_nothing_else` | prefix-added and reading-contract-changed are both reported; nothing else is | green |
| `a_prefix_change_seeds_an_attributed_closure_through_a_wrapping_newtype` | `impact()` closure of `type/Inner/prefix-added` reaches `Outer` (newtype of Inner) and the event using Outer | green |
| `an_outer_prefix_equal_to_the_inner_one_is_called_narrowed_though_nothing_narrows` | evidence for note F2: reported as narrowed even though the effective prefix is unchanged | green (documents F2) |

Red output, the case alone (`cargo test -p ess-diff --test adv1_text_prefix --no-fail-fast`), verbatim:

```
thread 'a_prefix_change_is_ess_diff_11_vocabulary_and_ess_diff_10_refuses_it' (2284253) panicked at crates/verify/ess-diff/tests/adv1_text_prefix.rs:67:9:
assertion `left == right` failed: a prefix kind is not in the released ess-diff/10: {
  "format": "ess-diff/10",
  ...
      "id": "type/demo.msgs.Code/prefix-added",
      "relation": "narrowed",
  ...
  left: 10
 right: 11

test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## 3. Suite run (after the cases above existed)

`cargo test -p ess-diff --no-fail-fast` gave EXIT=101. Totals: 227 passed, 1 failed. The
before-count is 220, taken from the implementor's gate.log step test-ess-diff. Final lines:

```
test result: FAILED. 7 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
error: 1 target failed:
    `-p ess-diff --test adv1_text_prefix`
```

The only failure is the red case above. Other checks:

| command | final line |
|---|---|
| `cargo clippy -p ess-diff --all-targets -- -D warnings` | `Finished dev profile` (clean) |
| `cargo fmt -p ess-diff --check` | exit 0, after rustfmt on the adversary file |
| `cargo xtask generate --check` | `projections are up to date` |

## 4. Findings

| file:line | severity | verdict | origin | finding | fix |
|---|---|---|---|---|---|
| crates/verify/ess-diff/src/change.rs:404 | blocker | NEEDS-CHANGE | introduced | Prefix kinds are assigned `ess-diff/10`, which shipped in 0.41.0 without them. An `ess-diff/10` writer and reader accept them, so a released format's meaning changes. | `=> 11`; add 11 to `SUPPORTED_DELTA_FORMATS` (delta.rs:13) and register `ess-diff/11` in FORMAT_RELEASES as coordinator patch; move the text out of the `/10` rows into new `/11` rows (formats.md:297, spec-versions.md:200); update wire-presence-json-prefix.md:128, the changelog draft, and text_prefix.rs:6/83/157 plus its `3..=9` loop (should be `3..=10`) |
| crates/verify/ess-diff/src/change.rs:369 | note | INFEASIBLE | introduced | The relation is computed from the declared prefix, not the effective one. An outer newtype that declares or drops a prefix its inner layer already imposes is called narrowed/expanded, although the admitted values are unchanged. Reachable: types.rs:1225 accepts equal nested prefixes. The direction is never inverted. | None here. raw.rs:136 re-derives the relation from the change's own content, so an effective-prefix relation would need the change to carry the effective prefix. That is a design choice; the #219 decision chose comparison. |

What reaches F1: every `ess verify diff` over a pair whose newtype prefix moved. The /11 rule is
the coordinator's decision (decisions.md, #219 row, 2026-09-29).

## 5. Attacked and not broken

- Relation for added, removed, extended, shortened, unrelated, case-only, one-character and shared-stem replacements: correct.
- Prefix plus alphabet, plus representation, plus reading, plus kind change: every change is reported, with no unclassified and no drop.
- Removing `reading` from the residual: `ReadingContractChanged` compares the full derived-`PartialEq` `ReadingContract` (reading.rs:67). Nothing is silently dropped.
- Byte stability: `generate --check` is clean. The unit's same-prefix/alphabet pair keeps `ess-diff/8`. An empty delta stays `ess-diff/2`.
- Text and JSON renderers, raw read-back, and derived relation and id checks: covered by the unit's tests and pass.
- Impact: a prefix change seeds a type closure through a wrapping newtype to the event. It no longer triggers the whole-invalidation (`UncomparedFamilyChanged`).
- Classes: distinct witness values in the unit's tests and in these cases (1); no unreached transition (2); two newtypes moving at once each get their own change (3).
- Mutants: dropping `prefix` or `reading` from the residual keys, and swapping narrowed/expanded, are caught by the unit's tests.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/prefix/adv1/review.md
- ~/.cache/ess-wave-n2/prefix/adv1-suite.KoOk.log
- ~/.cache/b10x-target/ess-d-prefix (the assigned shared build dir; incremental outputs added)

## 7. Findings block

```findings
- file: crates/verify/ess-diff/src/change.rs
  line: 404
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: prefix-added/removed/changed are assigned the already released ess-diff/10 instead of ess-diff/11, so a 0.41.0 /10 reader and writer accept vocabulary that format never had
- file: crates/verify/ess-diff/src/change.rs
  line: 369
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the prefix relation reads the declared prefix, so an outer newtype restating its inner layer's prefix is called narrowed or expanded although its effective prefix and admitted values are unchanged
```
