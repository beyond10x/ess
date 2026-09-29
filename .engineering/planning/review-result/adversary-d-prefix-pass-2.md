---
format: aep.planning-md/3
id: review-result:adversary-d-prefix-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.42 unit prefix
relations:
- reviews: story:diff-classifies-a-newtype-prefix-change
revision: 1
---
unit: diff-prefix (beyond10x/ess#219) pass 2, working tree ess-d-prefix on 4f7398154 (uncommitted implementor diff after correction 1, plus the adversary files adv1_text_prefix.rs and adv2_diff11.rs)
verdict: nothing found
cases: executed 228→232, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths (see part 6)
needs-coordinator: none

## 1. Diff stat

```
 crates/edge/ess-xtask/src/docs.rs             |  1 +
 crates/verify/ess-diff/src/change.rs          | 68 +++++++++++++++++++++++++--
 crates/verify/ess-diff/src/delta.rs           |  2 +-
 crates/verify/ess-diff/src/diff.rs            | 36 +++++++++++++-
 crates/verify/ess-diff/tests/clock_reading.rs | 11 +++++
 docs/design/wire-presence-json-prefix.md      |  9 +++-
 website/docs/guides/track-change.md           |  2 +-
 website/docs/reference/formats.md             |  1 +
 website/docs/reference/spec-versions.md       |  1 +
 9 files changed, 122 insertions(+), 9 deletions(-)
```

The implementor changed all nine tracked paths. This pass added one untracked file,
`crates/verify/ess-diff/tests/adv2_diff11.rs`. It changed no other file.

## 2. Cases added (`crates/verify/ess-diff/tests/adv2_diff11.rs`, 4 cases, all green)

The model is the `ess/18` delivery-context fixture. `AccountId` gets or loses a `prefix:`, and the channel authority is moved.

| case | asserts | now |
|---|---|---|
| `a_binding_context_change_beside_a_prefix_change_is_ess_diff_11` | A mixed delta (external `cause-changed` plus `prefix-added`) is written as `ess-diff/11` with no unclassified entry. The `/10` writer refuses it with `UnrepresentableChange` naming prefix-added. `/11` round-trips byte for byte. The `/10` reader refuses the prefix change and does not refuse the cause change. | green |
| `a_binding_context_change_alone_keeps_ess_diff_10_and_may_be_written_as_11` | With the prefix unchanged, a context-only delta stays `/10` and does not mention `/11` or `prefix-`. An explicit `/11` write differs only in the format string and reads back. | green |
| `every_writer_below_11_refuses_a_prefix_change_and_12_is_unsupported` | Writers `/1` to `/10` all refuse `prefix-changed` (`acc_`→`acc_x`, narrowed). `/12` is unsupported for both the writer and the reader, and the reader's refusal lists `ess-diff/11`. | green |
| `the_text_rendering_of_a_mixed_delta_names_both` | The text rendering shows `prefix `acc_` → (none)` beside the cause change. Exactly one change is expanded. | green |

There is no red output. Run alone (`cargo test -p ess-diff --test adv2_diff11 --no-fail-fast`):

```
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
EXIT=0
```

## 3. Suite run (after the cases existed)

`cargo test -p ess-diff --no-fail-fast` (log `adv2-suite.Y872.log`). Summed over all 32 binaries: passed 232, failed 0. The before count is 228, summed from the implementor's `c1-ess-diff.AXBu.log`. Final lines:

```
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

EXIT=0
```

| command | final line |
|---|---|
| `cargo clippy -p ess-diff --all-targets -- -D warnings` | ``Finished `dev` profile [unoptimized] target(s) in 0.17s`` (after adding a file-level allow for `match_wildcard_for_single_variants` to the adversary file) |
| `cargo fmt -p ess-diff --check` | exit 0 |

## 4. Findings

None.

Residue that is not a finding of this unit:
- `website/docs/reference/formats.md:289` says "Supported delta majors are 1 to 7". It says the same at base 4f7398154. This is stale text that predates the unit, in a file the coordinator owns. It does not claim `/10` is the newest format.
- The changelog draft says "an `ess-diff/3`–`/10` writer refuses it". The `/1` and `/2` writers refuse it too (case 3). The draft is incomplete, not false.

## 5. Attacked and not broken

- Writers: `EssDelta::new` takes the maximum `minimum_format`. The CLI `ess verify diff --format json` prints `to_canonical_json` (main.rs:2548), and text uses `render::text`. The checked `to_canonical_json_for` accepts `/11` and refuses `/1` to `/10` and `/12`. No other `DeltaFormat` or `to_canonical_json_for` writer exists outside the crate.
- Readers: `raw.rs` refuses a prefix kind under `/3` to `/10` with `unsupported_format_version`, and it does not refuse a `/10` kind beside it. `/11` round-trips.
- Stale "newest is /10" claims: none in the ess-diff sources, ess-cli, the schemas or the website. `track-change.md:36` says "up to `ess-diff/11`".
- Format kept: a pair with neither kind stays `/10`, `/8` or `/2`. `generate --check` was clean in the c1 gates.
- Relation after correction 1: added narrows, removed expands, and extended, shortened and unrelated replacements behave correctly. The adv1 cases are still green.
- Removing `reading` from the residual: the reading change is reported once, prefix plus reading gives both changes, and nothing is dropped.
- Classes: distinct witnesses (`acc_`/`acc_x`, `t/`/`u/`, `ord_`/`inv_`) (1); prefix changes beside a kind change or a representation change are not dropped (2); two newtypes that both move each get their own change (3).

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/prefix/adv2/review.md
- ~/.cache/ess-wave-n2/prefix/adv2-suite.Y872.log
- ~/.cache/b10x-target/ess-d-prefix (the assigned shared build dir; test and clippy outputs added)

## 7. Findings block

```findings
[]
```
