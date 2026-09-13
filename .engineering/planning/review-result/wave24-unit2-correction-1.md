---
format: aep.planning-md/1
id: review-result:wave24-unit2-correction-1
kind: review-result
status: active
title: 'Unit 2 correction round 1: the falsified sentence removed, the adversary file patched'
relations:
- reviews: story:planning-store-carries-workstation-paths
revision: 1
---
# Unit 2, correction round 1: what changed and what I decided

Worktree `wt-5cf844fd8563`, branch `impl/planning-store-carries-workstation-paths`, uncommitted over
base `bd722fa964bd225b9755f272e22b45fab334449f`.

```
 crates/edge/ess-xtask/tests/host_paths.rs          | 344 +++++++++++++++++++--
 crates/edge/ess-xtask/tests/host_paths_lane/mod.rs | 226 +++++++++++++-
 2 files changed, 567 insertions(+), 3 deletions(-)
```

## The falsified sentence is gone

The bullet no longer claims `journal.jsonl` holds the largest single share. It states the shares as
measured — 87 of 32,806 carrying lines (0.27%), one file; `authored-discovery-source-pass1.md` at
28,730; a carve-out covering 59 of 60 files and 99.7% of lines; 32,163 of 65,838 by findings, where
the journal is first — and says the two units answer the decision oppositely and neither settles it.
The open question is held by `story:scrub-the-planning-store-or-say-why-not`.

## The parse now reads the claim

`documented_unread_trees(source) -> Vec<(String, bool, usize, usize)>`. Bullets are
`` * `tree` — unread, N files, M lines: ``. The lane checks the verdict against `SCANNED_PREFIXES`
and both counts against the detector. Three probes the unit ran on the final code:

| probe | result |
|---|---|
| continuation wrapped with a tab | 1 passed — the section is anchor-delimited, nothing dropped |
| verdict flipped `unread` → `read` | FAILED — the parse reads the claim, not the tree name |
| line count 32806 → 32807 | FAILED, naming both the bullet's number and the detector's |

`UNREAD_TREE_SECTION` is a two-element array (open and close anchor), so indentation cannot end the
section. `transcription_drift` gained a converse check: every column-zero non-`#[test]` fn must be
in `TRANSCRIBED` or in a new `UNTRANSCRIBED` list with a written reason. It found the missing class
by itself, naming all three helpers, and classified three pre-existing untranscribed functions.

## The judgement I was asked for: the `layout.rs` appeal was wrong

`SCANNED_PREFIXES`' own doc stands. The two scans refuse different things — a stale
repository-relative citation is not a defect, a workstation path is — so `layout.rs`'s verdict is
not authority to borrow. What they share is a fact, not a verdict: nothing under the store can be
corrected after the fact. The bullet now says that, names the earlier appeal as the thing that was
wrong, and cites `SCANNED_PREFIXES` as the reason.

## The coordinator decision: I patched the pass-1 adversary file

Three of `host_paths_adversary_5.rs`'s five cases were unsatisfiable by any edit to files the unit
owns. The unit left them standing, wrote the fix into scratch unapplied, and handed it back. It was
right to. I read it, applied it, and ran it.

| case | why no edit of the unit's could reach it | what I did |
|---|---|---|
| `the_journal_holds_the_largest_share_…` | `assert!(bullet.contains("largest single share"))` at `:256` fires when the falsified claim is removed; leaving the claim in fires the `assert_eq!` at `:277`. Both branches red. The case demands a sentence **and** that the sentence be true, and the repository says it is not | precondition `assert!` → `return`. The claim is still refused when made; a bullet that drops it leaves nothing to refuse |
| `the_documentation_parse_distinguishes_…` | drove the adversary's own frozen hand copy on a frozen literal. `:322` asserts `real == {".engineering/"}` and `:333` asserts `parsed != real`, where `parsed` is that copy's constant answer. The two contradict for every possible lane source | re-pointed the copy at `host_paths_lane::documented_unread_trees`, now in `TRANSCRIBED` so `assert_current()` guards it |
| `a_second_documented_tree_survives_…` | same frozen copy on synthetic fixtures; its answer did not depend on the lane at all | same, plus the fixtures rewritten into the grammar the lane parses |

The `assert!` → `return` is a weakening and is recorded as one. It is acceptable only because the
bullet's counts are pinned by a live check — the third probe above goes red on a one-line change.
Adversary pass 2 was asked to establish whether that case now catches anything at all.

## Gate, run by me under the Taskfile's pinned profile

`RUSTUP_TOOLCHAIN=1.98.1`, wrappers unset, debug and incremental zeroed.

```
cargo test --package ess-xtask --locked --no-fail-fast      exit 0
  src/main.rs               129 passed; 0 failed
  host_paths                 12 passed; 0 failed
  host_paths_adversary        7 passed; 0 failed
  host_paths_adversary_2      7 passed; 0 failed
  host_paths_adversary_3      5 passed; 0 failed
  host_paths_adversary_4      5 passed; 0 failed
  host_paths_adversary_5      5 passed; 0 failed
  layout                      5 passed; 0 failed
```

175 cases, 8 targets, 0 failed.

## One report-quality finding against the unit

Correction round 1 reported `--bin ess-xtask test result: FAILED. 128 passed; 1 failed` and labelled
it "pre-existing, left alone" without naming the failing case. I ran that target twice — inside the
full suite and as `--bin ess-xtask` alone — and got `129 passed; 0 failed`, exit 0, both times.

A red reported without its name cannot be checked, and this one did not reproduce. Either it never
existed on this tree or it is order-dependent, and the unit's own report cannot tell those apart.
Pass 2 was asked to establish which.
