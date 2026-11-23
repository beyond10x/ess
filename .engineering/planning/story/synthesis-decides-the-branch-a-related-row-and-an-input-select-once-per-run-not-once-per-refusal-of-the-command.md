---
format: aep.planning-md/3
id: story:synthesis-decides-the-branch-a-related-row-and-an-input-select-once-per-run-not-once-per-refusal-of-the-command
kind: story
status: draft
title: Synthesis decides the branch a related row and an input select once per run, not once per refusal of the command
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/made.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/stored_rows_decided_once_work.rs
revision: 6
---
## Outcome

Synthesizing a model whose command has many refusals guarding on one related row (`when_related:`
beside an input guard) decides the branch that a related row and an input select once per
synthesis, not once per refusal of the command. The work of such a synthesis is back below its
0.92.0 level, and the suite, the standard output, the standard error and the exit status are
byte-identical to the released 0.97.3. Shipped by hotfix release 0.97.4; the fix is already on the
branch `perf/synthesis-regression-since-0.93.0` (commit `7ba3b6d85d`).

## Need

A downstream specification reported that `ess verify conform synthesize` became many times slower
from 0.93.0 on. Reproduced with the released binaries of 0.92.0, 0.93.0 and 0.97.3 on a copy of
that specification whose only change is the `requires:` pin line, set to each binary's own version.
The suite is identical on all three (79 scenarios, 34 refusals, sha `5ad75b0e1f6be476`):

| binary | instructions (user) | user CPU |
|---|---|---|
| 0.92.0 | 336.5e9 | 71.6 s |
| 0.93.0 | 4,346.6e9 | 902.5 s |
| 0.97.3 | 2,536.5e9 | 491.9 s |
| the fix | 88.1e9 | 18.3 s |

0.97.3 is 7.5 times 0.92.0 in instructions. The reported lines differ from 0.92.0 only in the
number of candidates the search tried: 31 refusals read `of the 41 tried` where 0.92.0 read `of the
16 tried`; the suite does not move.

Cause, found with a bisect between the tags and a profile:

- The regression enters with commit `a46122ba42` ("witness a list-literal guard beside other
  inputs and negation"), released in 0.93.0. On a reduced copy of the specification, its parent
  `53daf1fcf4` runs 14.70e9 instructions with 8 candidates tried, and `a46122ba42` runs 53.02e9
  with 14 tried. On the generic model `perf2/regress/generic/m` the parent runs 0.968e9 (the
  0.92.0 suite) and `a46122ba42` 2.005e9 (the 0.93.0 suite, which witnesses more on that model).
- The commit appends up to `MAX_CANDIDATES` (64) list-fill candidates to the witness search of a
  list-literal guard. The creator of the related row then leaves more rows (16 to 41 candidates on
  the downstream specification), and the command that reads them is tried with more inputs.
- A profile of 0.97.3 on the reduced copy (line tables, `perf --call-graph=lbr`) puts 86 %
  inclusive in `related_guard::chooses` and `related_guard::selects`, reached from
  `with_row_where` inside the row search of the refusal's goal. A refusal accepts only its own
  branch, and each of the 31 refusals decides the same (row, input) pairs anew. `subject_fact`'s
  own `selects` already keeps its pick per row (`row_picks`); the related-guard pick had no memo.
  The work is refusals times rows times inputs, and `a46122ba42` raised the last two.

Reduced copy of the downstream specification, instructions: 0.92.0 14.66e9, 0.97.3 40.08e9, the
fix 7.91e9 (suite sha `c6d671aead862f93` on 0.92.0 and 0.97.3). Generic model `generic/m`: 0.92.0
0.97e9, 0.93.0 2.01e9, 0.97.3 3.04e9, the fix 2.00e9, suite identical to 0.97.3's.

## Fit review

A defect reported by an adopter and reproduced on the released binary: adopted as a fix, with no
new construct to design. It is a regression, not a shape of the model: on the same specification
0.92.0 needs 336.5e9 instructions and 0.97.3 2,536.5e9 for the same suite. Per the hotfix rule of
`AGENTS.md` it is pending work, shipped in the patch release 0.97.4 and merged to `main` by a merge
commit.

The candidates of `a46122ba42` stay: the 0.93.0 suite of the generic model differs from the 0.92.0
one because they witness more there, and the fix removes the repeated decision, not the candidates.

## Design

- The pick of a related row and an input is kept in `made`'s memory per declared command, entity,
  row (state, settled fields, unwritten fields) and input, under the key prefix `"\0related|"` of
  `made::row_picks`, through a new `RowPicks::answer` that `subject_fact`'s `row_picks` use as
  well. `related_guard::chooses` asks the memo before `selects`.
- The memo is sound because `selects` is pure: it reads only the IR, the command, the entity, the
  row's `settled`, `unwritten` and `state`, and the input.
- A counter, `related_selections_tried` (exported by `ess_conformance::synthesize`), counts the
  picks made, for the work test.
- Format: no new `ess/N`. No meaning, identity, reference, canonical form or persisted field
  changes. The suite bytes do not move: the identity run over 176 suites (generic and billing
  models, the reduced copy, 17 review models, two further downstream specifications and the reported
  one) compared byte for byte with the released 0.97.3 gives 176 of 176
  identical, and the downstream specification's standard output (modulo the `--out` path),
  standard error and exit status are identical too.

## Acceptance

- `a_related_row_many_refusals_read_is_decided_within_a_bound` in
  `crates/verify/ess-conformance/tests/stored_rows_decided_once_work.rs` (target
  `stored_rows_decided_once_work`): on the generic `dock(2, 2)` model (2 tools and 2 marks, refusals guarding on one bay), `related_selections_tried` is at most 450. The memo gives 324; the same code with
  the memo disabled gives 846, so the test fails on the old path. The target's 4 tests pass in
  7.6 to 13 s of step time; the new test took about 5 s on a loaded host.
- The identity run: 176 of 176 suites byte-identical to the released 0.97.3 (log
  `perf/identity-regress.log`), and the downstream specification's suite equal to 0.92.0's
  (`5ad75b0e1f6be476`).
- The byte-pin targets of `AGENTS.md` that the change can move stay green: in `ess-conformance`,
  `synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
  `adversary_related_values_all_bytes`, `external_beside_held_guard`; and beside them
  `adversary_u4_list_literals`, `missing_related_row_beside_input_refusal`,
  `adversary_related_guard_pass1`, `related_guard_ranked_candidates_work`. `cargo clippy -p
  ess-conformance --all-targets` and `task fmt-check` pass.
- On release build of the fix, the downstream specification runs in 88.1e9 instructions (18.3 s
  user CPU), below the 336.5e9 (71.6 s) of 0.92.0.

## Decisions

- None for this fix.
- Left open, outside this story: `synthesis_suite_bytes_table` ran 431 s in the check of this
  change and `related_guard_ranked_candidates_work` 133 s, `external_beside_held_guard` 123 s,
  `adversary_related_values_all_bytes` 76 s. The notes record the first as over the test budget
  before this change; a step over the 60 s per test or 120 s per step budget is a defect with a
  story of its own, not part of this one.
