---
format: aep.planning-md/3
id: review-result:adversary-d-digest-pass-1
kind: review-result
status: active
title: Adversary pass 1, 0.42 unit digest
relations:
- reviews: story:caller-sources-keep-the-interpreted-digest
revision: 1
---
unit: caller-digest (beyond10x/ess#216), working tree ~/.local/state/worktree/trees/b10x/ess/ess-d-digest (uncommitted diff on 4e7c3867e)
verdict: CONFIRMED (one note-level finding; no failing case)
cases: executed 2436→2443, red 0
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths (~/.cache/ess-wave-n2/digest/adv1/{review.md,suite-conformance.log,suite-cli.log})
needs-coordinator: none

## 1. git --no-pager diff --stat (tracked; the unit's own diff, untouched by this pass)

```
 crates/edge/ess-cli/tests/interpreted_target.rs    | 182 +++++++++++++++++++++
 .../ess-conformance/src/synthesize/caller.rs       |   6 +
 .../verify/ess-conformance/tests/caller_values.rs  |  32 +++-
 3 files changed, 217 insertions(+), 3 deletions(-)
```
Untracked, added by this pass (test files only):
- crates/verify/ess-conformance/tests/adversary_caller_digest_pass1.rs
- crates/edge/ess-cli/tests/adversary_caller_digest_pass1.rs

## 2. Cases added (all green now; none red when first run)

| case | asserts | now |
|---|---|---|
| conformance `every_entry_point_stamps_the_model_digests_on_a_caller_suite` | `synthesize`, `coverage_build::build` (input/1 / `--suite-format 5`), `web_replay::AdmittedReplay::new`, and `mutate::emit` baseline + manifest all carry `SuiteProvenance::of(model)` spec and contract digests | green |
| conformance `models_differing_only_in_caller_data_carry_different_suite_digests` | same-typed caller source swapped, or one unread actor attribute added: three distinct suite digests, each the model's own | green |
| conformance `a_different_same_typed_caller_source_changes_the_suite_contents` | class 1: reading `billing_account_id` in place of the same-typed `account_id` changes scenario content | green |
| conformance `a_caller_suite_is_synthesized_to_the_same_bytes_twice` | determinism of the canonical bytes | green |
| cli `the_declared_coverage_run_interprets_a_caller_specification` | `run --target interpreted --suite-format 5 --report-format 2` renders a report, no digest refusal | green |
| cli `a_written_declared_coverage_suite_runs_against_its_caller_model` | suite/5 written by `synthesize` then run with `--suite` | green |
| cli `a_suite_is_refused_against_a_model_differing_only_in_caller_data` | refusal fires and exits 1 for a model differing only in a caller source, and only in an actor attribute | green |

First run of those cases alone (not red, verbatim tails):
```
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.38s
```
One probe assertion of mine failed first (the model's only mutants, emit-drop, are stillborn, so no mutant dir existed). That was my fixture's fault, not a finding; I removed the count assertion. Live mutants are still checked when present.

## 3. Suite runs (after the cases existed)

`cargo test -p ess-conformance --no-fail-fast`: EXIT=0, 227 binaries, passed 1614 failed 0 ignored 3. Last line:
```
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.56s
```
`cargo test -p ess-cli --no-fail-fast`: EXIT=0, passed 829 failed 0 ignored 4. Last line:
```
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
`cargo clippy -p ess-conformance -p ess-cli --all-targets -- -D warnings`: `Finished`, no warnings. `cargo fmt -p ess-cli -p ess-conformance -- --check`: exit 0. The unit's gates.log shows `fmt: exit 201`, but that run came before the file was formatted; it is clean now.
Executed before = 2443 − 7 (this pass's two files). That number is derived and was not measured in its own run. After those runs I only reformatted the conformance adversary file with rustfmt (whitespace).

## 4. Findings

| file:line | severity | verdict | origin | finding | fix |
|---|---|---|---|---|---|
| crates/verify/ess-conformance/tests/caller_values.rs:303 | note | CONFIRMED | introduced | The `without_callers()` half of `issue_216_…` compares `synthesize_plain`'s stamp with `SuiteProvenance::of`, which is the same call, so it cannot fail. Acceptance bullet 2 ("a test pins that a model without caller sources keeps its digest") is met only through the committed billing suites under `cargo xtask generate --check`, not by this test. | Pin a literal `spec_digest` for `without_callers()`, or say in the test that the committed suites carry the pin |

What reaches it: nothing at runtime. It is only a question of test strength.

## 5. Attacked and could not break

- Honesty of the overwrite: `source_digest` is SHA-256 over the whole IR's compact JSON. That includes `ResolvedActor.attributes` and every caller source, so two different caller-reading models cannot share a suite digest. The fix also removes a collision that existed before it (two models with equal `rewrite_First` copies).
- Nothing else in the suite carries a digest. Scenarios, refusals and notes hold none (grep over `synthesize*`).
- Derived-IR paths: `synthesize_for` (component), `coverage_build` (input/1, report/2), `mutate::emit`/`audit`, `web_replay`, `sessions`, `record`/`recorded`, `release_evidence` and `fresh_legacy_run_suite` all go through `synthesize` or the model's own `SuiteProvenance::of`. No other `with_commands_rewritten` caller exists.
- Refusal for a different model: it fires for a model differing only in a caller source or only in an actor attribute (cli case), and for one with an extra entity field (the unit's own case).
- Byte stability: the stamp runs only when `caller::uses` holds, and no committed example reads the caller. `generate --check` is clean (unit gate log).
- Determinism: the same bytes twice.
- Classes: 1 (same-typed caller attributes give different content) holds. 2 and 3 do not apply to a provenance stamp.

## 6. Paths written outside the worktree

- ~/.cache/ess-wave-n2/digest/adv1/review.md
- ~/.cache/ess-wave-n2/digest/adv1/suite-conformance.log
- ~/.cache/ess-wave-n2/digest/adv1/suite-cli.log
- Build output went into the assigned ~/.cache/b10x-target/ess-d-digest, which the unit already shares.

## 7. Findings block

```findings
- file: crates/verify/ess-conformance/tests/caller_values.rs
  line: 303
  category: acceptance
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the no-caller half of issue_216 compares SuiteProvenance::of with itself and cannot fail, so acceptance bullet 2's digest pin rests on the committed billing suites, not on this test
```
