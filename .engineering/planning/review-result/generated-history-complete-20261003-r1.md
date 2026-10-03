---
format: aep.planning-md/3
id: review-result:generated-history-complete-20261003-r1
kind: review-result
status: active
title: Independent review of the complete generated-history candidate
relations:
- reviews: story:feature-request-292
revision: 1
---
approve
unit: story:feature-request-292 complete generated-history candidate cae6187ecf1d3a5c98fed636beb7a2174e1d1e80
verdict: nothing found
cases: 3 executed, red 0, green 3; generated-history baseline 53 -> 56
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths
needs-coordinator: nested-increment target location remains a separate pending unit; no candidate-review action

# Independent tests-only review

Raw report SHA-256:
`3d9885c7c59f5a5d98c9667538cae17f901d03ddb8d1740f410c0b3a390caaa8`.

## Frozen boundary

I reviewed candidate `cae6187ecf1d3a5c98fed636beb7a2174e1d1e80` against base
`86b4a994481f4391f7f38cc2677cdc493ca20404` in `<assigned-tree>`. `HEAD` was the exact candidate
before testing, and the retained source manifest checked all 18 candidate paths successfully.

```text
18 files changed, 5365 insertions(+), 315 deletions(-)
```

The only review-tree change is a prospective test addition:

```text
154  0  crates/verify/ess-conformance/tests/generated_history_values.rs
```

No production source, generated projection, planning artifact, commit, ref, or other checkout was
changed.

## Prospective cases

All three cases were persisted before their first Cargo invocation and preserve existing
assertions.

1. `definite_false_suffix_dominates_unknown_optional_prefix_in_generated_domain` proves that a
   later definite false constraint cannot be masked by an earlier unknown optional member.
2. `complete_increment_domains_distinguish_partial_from_total_overflow` checks the exact
   `i64::MAX` boundary: a partly overflowing complete domain remains unresolved, while an entirely
   overflowing complete domain proves there is no successor.
3. `a_generated_identity_is_not_observation_authority_for_a_cross_row_read` proves that a
   generated relation address cannot authorize a related-row read; the input-supplied-address
   control remains linearizable.

```text
false dominance: 1 passed; 0 failed; 55 filtered out; exit 0
partial/total overflow: 1 passed; 0 failed; 55 filtered out; exit 0
cross-row generated identity: 1 passed; 0 failed; 55 filtered out; exit 0
```

The third fixture needed two corrections before it was admissible: a constrained generated string
could not establish bounded feasibility, then a draft command was refused as
`non_exhaustive_branches`. Neither run reached the asserted behavior; neither is a red case or a
product finding.

## Source judgement

The complete diff, active story, design, callers, and tests preserve the reviewed invariants:

- Unknown presence is explicit in `ess-primitives` and forwarded through ordinary and nested-binder
  predicate adapters. History facts do not turn an unobserved generated leaf into absence.
- Abstract values carry private origins. Validation samples are discarded rather than becoming
  history authority. Copies retain origin, while operations, rows, and effect occurrences remain
  distinct.
- Domain proof has bounded work and depth, retains exhaustion as unresolved, and caches only
  completed proof states. Complete finite enumeration is capped and false-dominance folding still
  inspects a definitely invalid suffix after an unknown prefix.
- Complete-domain transfer and increment retain all admitted members. Increment uses exact checked
  addition and distinguishes partial overflow from total absence of a successor.
- Determined values read the original store and original subject. Set selection reads the
  pre-outcome store and validates staged writes before publication.
- Commands that may read another row are assigned to the shared search partition, preserving
  cross-row ordering and read-your-writes authority.
- Search reports a violation only when no unresolved alternative remains. The CLI keeps refusal,
  violation, and budget exhaustion distinct.

The nested-increment target-location concern overlaps the increment machinery but is a separate
pending unit. This review does not claim to close it and found no candidate regression in that
area.

## Commands and exact results

Every compiler start used Rust 1.98.1, locked/offline dependencies, one job, debug information off,
incremental compilation off, no `RUSTC_WRAPPER`, `<assigned-tree>/target`, and
`<assigned-scratch>/tmp`. Every expensive start passed the 12,884,901,888-byte disk floor; the
lowest observed start was 15,532,220,416 bytes during fixture authoring.

The final generated-history output was:

```text
running 56 tests
test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
EXIT: 0
```

The other exact runner results were:

```text
ess-conformance --lib:
test result: ok. 109 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
exit: 0

19 named neighboring binaries:
5 + 1 + 3 + 6 + 5 + 9 + 7 + 13 + 9 + 3 + 10 + 5 + 12 + 7 + 6 + 11 + 3 + 5 + 12 = 132 passed
exit: 0

primitive observed presence:
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 76 filtered out; finished in 0.00s
exit: 0

primitive nested binders:
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 76 filtered out; finished in 0.00s
exit: 0

CLI generated-history matrix:
known=false: inert 0, early 2, late 2, budget 3
known=true: inert 0, early 0, late 1, budget 3
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.08s
exit: 0
```

Static checks:

```text
cargo clippy -p ess-primitives --all-targets -- -D warnings: exit 0
cargo clippy -p ess-conformance --lib --test generated_history_values -- -D warnings: exit 0
cargo clippy -p ess-cli --test check_history -- -D warnings: exit 0
cargo fmt -p ess-conformance -- --check: exit 0
git diff --check: exit 0
```

The first conformance Clippy attempt found an unnecessary raw-string hash in the prospective test.
That test-only spelling was corrected before the successful Clippy and 56-case rerun. A
repository-wide formatting check reported existing drift under `generated/rust/**`, outside the
candidate and review diffs; it modified nothing, and the scoped package formatting check passes.

The exact post-correction output is retained at `<assigned-scratch>/runner-output-final.log` and the
earlier exact command/result records at `<assigned-scratch>/runner-transcript-from-tool.md`.

## Writes outside the worktree

1. `<assigned-scratch>/tmp` — assigned compiler TMPDIR, empty at handoff.
2. `<assigned-scratch>/runner-output-final.log` — post-correction command/output/exit capture.
3. `<assigned-scratch>/runner-transcript-from-tool.md` — earlier command/result records copied from
   the tool transcript.
4. `<assigned-scratch>/report.md` — raw report.
5. `<assigned-scratch>/report-publication.md` — this path-normalized report.

The compiler build directory was `<assigned-tree>/target`, inside the assigned tree.

```findings
[]
```
