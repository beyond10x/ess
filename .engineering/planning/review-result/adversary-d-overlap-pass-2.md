---
format: aep.planning-md/3
id: review-result:adversary-d-overlap-pass-2
kind: review-result
status: active
title: Adversary pass 2, 0.42 unit overlap
relations:
- reviews: story:overlapping-accepting-guards-have-declared-precedence
revision: 1
---
unit: overlap-precedence (beyond10x/ess#217) after correction 1, tree ess-d-overlap: uncommitted diff on 4e7c3867e
verdict: INFEASIBLE
cases: executed 1630→1637, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 7 files plus the shared build dir (part 6)
needs-coordinator: none

## 1. Diff stat

```
 crates/generate/ess-entity-runtime/src/lib.rs      |   5 +-
 .../tests/input_guard_overlap.rs                   | 191 ++++++-
 .../ess-conformance/src/interpret/execute.rs       | 166 ++++--
 crates/verify/ess-conformance/src/synthesize.rs    | 614 ++++++++++++++++++---
 .../src/synthesize/bounded_retry.rs                |  10 +-
 crates/verify/ess-conformance/src/witness.rs       | 271 +++++++--
 .../ess-conformance/tests/guarded_external.rs      |   5 +-
 docs/design/input-guard-overlap-precedence.md      | 114 +++-
 website/docs/guides/write-a-specification.md       |   6 +
 website/docs/reference/predicates.md               |  25 +-
 10 files changed, 1211 insertions(+), 196 deletions(-)
?? crates/verify/ess-conformance/tests/adversary_overlap_precedence_pass2.rs   (mine, untracked, test only)
```
The tracked diff is the implementor's. I added one untracked test file and nothing else.

## 2. Cases (crates/verify/ess-conformance/tests/adversary_overlap_precedence_pass2.rs)

| line | case | now |
|---|---|---|
| 175 | overlap needs two Decimal leaves at their midpoints together | green |
| 192 | overlap needs three Integer leaves off base together | green |
| 205 | same overlap written as `not any[!=]`, which `exhausts` accepts as plain | green |
| 222 | overlap needs three Text leaves together | green |
| 241 | Decimal newtype `invariants: ['value < 11.5']`: overlap `11<amount<11.5` is non-empty, but the midpoint 11.5 is refused by the type | **RED** |
| 260 | committed examples billing, gatepass and oracle-fixture: no shadow-by-earlier refusal, no `PrecededExternalEligibility`, no `UnwitnessedOverlap` note | green |
| 315 | overlap on a required leaf inside an `Optional` struct | green |

Red output, verbatim, from running the file alone before the suite:
```
---- an_overlap_whose_midpoint_the_newtype_refuses_is_sent_or_noted stdout ----
capped decimal: the overlap of `small` and `flagged` is neither sent nor noted.
sent: ["refused <- [\"amount=Number(Number(Exact { units: 1, scale: 0, binary: 1.0 }))\"]", "refused <- [\"amount=Number(Number(Exact { units: 10, scale: 0, binary: 10.0 }))\"]", "small <- [\"amount=Number(Number(Exact { units: 11, scale: 0, binary: 11.0 }))\"]"]
refusals: ["refusal[ESS-SYNTH-003]: outcome demo.orders.PlaceOrder/flagged has no scenario `demo.orders.PlaceOrder/outcome/flagged`\n  no candidate of the 7 tried satisfies `(amount > 11 and amount < 13)`\n  help: ...", "refusal[ESS-SYNTH-013]: type demo.orders.Capped has no scenario ..."]
notes: []
test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```
(The refusal help texts are shortened. The full text is in case-run.log.)

## 3. Suite (run after the cases existed)

`cargo test -p ess-conformance --no-fail-fast`: 230 result lines, passed 1636, failed 1, ignored 3, `EXIT=101`. The only failing target is `--test adversary_overlap_precedence_pass2`: `test result: FAILED. 6 passed; 1 failed; ...`. Last result line: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.27s`.
`<before>` = 1637 minus my 7 cases. It was not a separate run.
All 6 pass-1 adversary cases are green now.
`cargo xtask generate --check`: `projections are up to date`, `EXIT=0`.
Clippy `-D warnings` on the new file is clean, and rustfmt is clean.

## Findings

| file:line | severity | verdict | origin | finding | what reaches it |
|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/witness.rs:338 | note | INFEASIBLE | introduced | `exhausts` counts each midpoint as a region it tried, and ignores the newtype invariants that `admitted_inputs` applies to candidates. When an invariant refuses the midpoint, the overlap is taken as shown empty (`synthesize.rs:9029` `continue`). No row is sent and no note is recorded, although inputs the type admits lie there (brief class 2). Fix: in `exhausts`, return false for a leaf with invariants, or count only the candidates that were admitted | I built the invariant myself. Nothing I found shows anyone writes a Decimal invariant between two adjacent guard literals. In this model `flagged` is also refused with ESS-SYNTH-003, so the synthesis is not clean |

## 4. Judgement findings
None beyond the table. The findings cover the working tree on 4e7c3867e.

## 5. Attacked and could not break
- Adopter regressions: `generate --check` is clean, and the three committed examples gain no new refusals and no overlap notes. `PrecededExternalEligibility` refuses whenever any accepting `when:` is declared earlier, which is coarse. It matches the documented Changed entry and no committed example reaches it.
- Midpoint byte stability: the pass runs only after the ladder finds nothing, so a model without overlaps can only change from refused to witnessed. Negative, mixed-scale and zero-crossing midpoints were checked by reading `midpoint`. Decimal has no declared scale, so no precision contract can be broken.
- `exhausts` with multi-leaf overlaps (Decimal×2, Integer×3, Text×3, `not any`, a required leaf under an `Optional` parent): each overlap is sent or noted.
- Interpreter `select` against Entity Runtime order (read, plus the correction-1 tests): refusal+refusal, refusal+accepting, accepting+external (Forced/Open/Withheld), external first, and Undecidable only before the answer all agree with the ER sort (category 0 for `when`+error, category 1 by source index).
- Classes 1 and 3 are covered by pass 1 and still green.

## 6. Paths written outside the worktree
- ~/.cache/ess-wave-n2/overlap/adv2/review.md
- ~/.cache/ess-wave-n2/overlap/adv2/generate-check.log
- ~/.cache/ess-wave-n2/overlap/adv2/case-run.log
- ~/.cache/ess-wave-n2/overlap/adv2/probe-examples.log
- ~/.cache/ess-wave-n2/overlap/adv2/probe-optional.log
- ~/.cache/ess-wave-n2/overlap/adv2/suite-conformance.log
- ~/.cache/claude-tmp/adv2-overlap-synth.diff (a scratch copy of the diff)
- Build dir: ~/.cache/b10x-target/ess-d-overlap (shared, not deleted)

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/src/witness.rs
  line: 338
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: exhausts counts a midpoint that a newtype invariant refuses as a tried region, so a non-empty overlap is skipped as empty with no row and no note
```
