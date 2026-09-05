---
format: aep.planning-md/1
id: review-result:review-boundaries-1-report-adversary-pass-1
kind: review-result
status: active
title: Report reader adversary pass 1, resource-limited
relations:
- reviews: story:review-report-reader-validation
revision: 1
---
unit: story:review-report-reader-validation at ebe5e9dd7aeb1737fd7d5a2671d56824b5824eb5
verdict: blocked
cases: executed 238→unavailable, red 1; added binary executed 4
origin: introduced 0 / pre-existing 0 / undecided 1
wrote-outside-worktree: none
needs-coordinator: yes

## 1. Test-only diff

`git --no-pager diff --stat` produced no output because the new test file is untracked; this role may not stage files. The corresponding new-file diff, `git --no-pager diff --no-index --stat /dev/null crates/verify/ess-conformance/tests/report_reader_adversary.rs`, was:

```text
 .../tests/report_reader_adversary.rs               | 173 +++++++++++++++++++++
 1 file changed, 173 insertions(+)
```

Only that new test file was authored. No implementation file, existing assertion, planning artifact, branch or commit was changed. The attack covered the complete three-dot diff from opening commit `3d8d6c6b287ce1c462cc50ea74f1ba5c171b827b` to committed head `ebe5e9dd7aeb1737fd7d5a2671d56824b5824eb5`.

## 2. New cases and first execution

All four cases were written before any test command. The existing package's 238 executed cases come from the implementor's final report; no suite was run to collect that number. The tests exercise the standalone JSON reader, JSON value/reader routes, actual YAML mappings, JSON streaming and `Deserialize::deserialize_in_place`. Duplicate fields are tested through text-preserving readers instead of a JSON value that would discard duplicate keys.

| Case in crates/verify/ess-conformance/tests/report_reader_adversary.rs | Assertion | Observed result |
| --- | --- | --- |
| :72 aggregate_status_does_not_depend_on_nonpass_order_or_multiplicity | Enumerate all 85 non-pass sequences of length zero through three, two total counts and all four overall statuses; preserve opaque repeated identities and status precedence. | Green on first execution and added-binary run. |
| :103 count_extremes_refuse_contradictions_without_inventing_coverage | Zero, one and usize::MAX totals; count/list contradictions; negative, fractional, string and null counts. Empty passing summaries remain valid at large totals because the document cannot establish source coverage. | Green on first execution and added-binary run. |
| :124 closed_wire_fields_stay_required_and_nonnullable_on_every_route | Remove/null each required field and add unknown fields; demand consistent refusal across six routes. | Red first time at implementation: null through the YAML mapping route. The case stops there; later entries in its loop were not measured. |
| :142 duplicate_claims_cannot_hide_behind_a_valid_last_value | Duplicate format, suite, status, count and list claims cannot hide an invalid earlier field behind the valid last value. | Green on first execution and added-binary run. |

First command:

`cargo test --locked -p ess-conformance --test report_reader_adversary aggregate_status_does_not_depend_on_nonpass_order_or_multiplicity -- --exact`

Environment: `RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`. Own target directory; no CARGO_TARGET_DIR override. Exit 0. Runner output verbatim; raw combined Cargo log is retained at `target/review-boundaries-1/review-report-reader-validation/adversary-first-aggregate.log`:

```text
running 1 test
test aggregate_status_does_not_depend_on_nonpass_order_or_multiplicity ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.27s
```

The pre-build free-space check was 8,785,264,640 bytes, above the 8,589,934,592-byte reserve but down substantially from the coordinator's last observation. I stopped Cargo and notified the coordinator. The coordinator then explicitly authorized the already-built executable for remaining first executions and its complete four-case run. These invocations do not recompile source.

Command: `target/debug/deps/report_reader_adversary-fe92ac06642c38e7 count_extremes_refuse_contradictions_without_inventing_coverage --exact`

Exit: 0. Raw log: `target/review-boundaries-1/review-report-reader-validation/adversary-first-count_extremes_refuse_contradictions_without_inventing_coverage.log`.

```text

running 1 test
test count_extremes_refuse_contradictions_without_inventing_coverage ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

Command: `target/debug/deps/report_reader_adversary-fe92ac06642c38e7 closed_wire_fields_stay_required_and_nonnullable_on_every_route --exact`

Exit: 101. Raw log: `target/review-boundaries-1/review-report-reader-validation/adversary-first-closed_wire_fields_stay_required_and_nonnullable_on_every_route.log`.

```text

running 1 test
test closed_wire_fields_stay_required_and_nonnullable_on_every_route ... FAILED

failures:

---- closed_wire_fields_stay_required_and_nonnullable_on_every_route stdout ----

thread 'closed_wire_fields_stay_required_and_nonnullable_on_every_route' (3208069) panicked at crates/verify/ess-conformance/tests/report_reader_adversary.rs:64:9:
assertion `left == right` failed: YAML mapping disagrees for {"completed_at":1700000001000,"failed_scenarios":[],"format":"ess-conformance-report/1","implementation":null,"scenarios_failed":0,"scenarios_total":0,"spec_digest":"13577b3ce695932e980d418d5863bcde07f4c362516d53147870d31eaf2ed861","specification":"billing/v3","status":"passed","suite_version":"ess-conformance/4"}
  left: true
 right: false
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    closed_wire_fields_stay_required_and_nonnullable_on_every_route

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

Command: `target/debug/deps/report_reader_adversary-fe92ac06642c38e7 duplicate_claims_cannot_hide_behind_a_valid_last_value --exact`

Exit: 0. Raw log: `target/review-boundaries-1/review-report-reader-validation/adversary-first-duplicate_claims_cannot_hide_behind_a_valid_last_value.log`.

```text

running 1 test
test duplicate_claims_cannot_hide_behind_a_valid_last_value ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

The first failure output points to :64 in the pre-format helper. Standalone rustfmt subsequently moved the same assertion to :61 without changing it. No assertion was deleted, skipped or weakened, and no code correction was applied.

## 3. Suite execution and remaining gates

After all four first executions, the complete added binary ran:

`target/debug/deps/report_reader_adversary-fe92ac06642c38e7`

Exit 101. Raw log: `target/review-boundaries-1/review-report-reader-validation/adversary-new-binary.log`.

```text

running 4 tests
test duplicate_claims_cannot_hide_behind_a_valid_last_value ... ok
test closed_wire_fields_stay_required_and_nonnullable_on_every_route ... FAILED
test count_extremes_refuse_contradictions_without_inventing_coverage ... ok
test aggregate_status_does_not_depend_on_nonpass_order_or_multiplicity ... ok

failures:

---- closed_wire_fields_stay_required_and_nonnullable_on_every_route stdout ----

thread 'closed_wire_fields_stay_required_and_nonnullable_on_every_route' (3208078) panicked at crates/verify/ess-conformance/tests/report_reader_adversary.rs:64:9:
assertion `left == right` failed: YAML mapping disagrees for {"completed_at":1700000001000,"failed_scenarios":[],"format":"ess-conformance-report/1","implementation":null,"scenarios_failed":0,"scenarios_total":0,"spec_digest":"13577b3ce695932e980d418d5863bcde07f4c362516d53147870d31eaf2ed861","specification":"billing/v3","status":"passed","suite_version":"ess-conformance/4"}
  left: true
 right: false
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    closed_wire_fields_stay_required_and_nonnullable_on_every_route

test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
```

This is four executed added cases, three passed and one failed. It is **not** a complete package-suite run. The before package count is 238 (library 67; integration lanes 47, 7, 12, 11, 7, 14, 49, 24; doctests 0), from the implementing state's report. The after package count is unavailable.

The complete `cargo test --locked -p ess-conformance`, package formatter command and package Clippy were not run by this adversary after the additions. Direct execution was followed by a free-space observation of 5,726,199,808 bytes, below the reserve; the coordinator explicitly prohibited further Cargo/Clippy. A later observation fell to 3,398,778,880 bytes. No build directory or cache was cleaned.

The coordinator allowed standalone formatting. Final command `rustfmt --edition 2021 --check crates/verify/ess-conformance/tests/report_reader_adversary.rs` exited 0 with empty output; its log and exit file are `target/review-boundaries-1/review-report-reader-validation/adversary-fmt.log` and `target/review-boundaries-1/review-report-reader-validation/adversary-fmt.exit`. `git diff --check` exited 0. The test binary predates whitespace formatting only.

## 4. Finding, origin and reachability

No introduced defect has been established. There is one measured boundary discrepancy whose production reachability and applicability to this story remain limited:

| File:line | Verdict | Origin | Severity | Finding |
| --- | --- | --- | --- | --- |
| crates/verify/ess-conformance/tests/report_reader_adversary.rs:61 | INFEASIBLE | undecided | warning | A YAML report mapping accepts a null implementation identity while the JSON report routes reject the same fixture; no repository YAML report consumer was found. |

**What was measured:** the first execution of `closed_wire_fields_stay_required_and_nonnullable_on_every_route` failed with exit 101, specifically because the YAML route returned success for the fixture containing `implementation: null`. The assertion checks acceptance, not the returned identity string. The added binary reproduced that exact failure.

**What reaches it:** the direct public Serde API `serde_yaml::from_str::<StandaloneConformanceReport>` is callable, and the new test invokes it. The concrete repository report consumer found at `crates/edge/ess-cli/tests/go_conformance.rs:109` uses `StandaloneConformanceReport::from_json`, which rejected this fixture. Both actual producers write JSON. No production YAML-report consumer was located, so the finding is INFEASIBLE rather than a demonstrated production regression.

**Origin:** the base source uses a derived String field for implementation, and the new private wire shape uses the same type. The local serde_yaml String reader accepts scalar text through its string visitor; this provides an explanation for the observation but is not a replay against the base. Disk restrictions prevented baseline execution. Origin therefore remains undecided. I did not move the worktree, substitute the base implementation or claim an unexecuted replay.

**Acceptance limit:** the story promises unknown-version and internally contradictory-claim refusal. It does not explicitly define YAML scalar coercion for an otherwise opaque identity string. This test's broader non-nullability expectation is not by itself proof that the unit fails its stated acceptance. The coordinator must assess that contract question and route the case; no acceptance or release approval is asserted by this pass.

## 5. Attacks that did not break the reader

- Aggregate status remained correct across orderings and repetitions of failed, unsupported, error and skipped entries, including mixed vocabularies and opaque duplicate scenario identities.
- Empty summaries and maximum-sized totals remained readable without inventing a coverage guarantee; impossible list/count combinations and malformed numeric counts were refused.
- Streaming and in-place deserialization enforced the same claim validation as direct JSON reading.
- Duplicate earlier invalid claims could not hide behind the later valid format, suite version, count, list or status.
- The source inspection found no new contract document or fixture file added by this unit; its writer and serialized fields remain unchanged.

## 6. Outside-worktree paths and outstanding work

None. New test source, this report and all direct logs were written inside the assigned worktree; scratch stayed under `target/review-boundaries-1/review-report-reader-validation`. The existing adversary lease was heartbeated. No worktree lifecycle cleanup, planning command, staging, commit or implementation edit was performed. Whole-task wall duration, tokens and tool count are unavailable.

Coordinator action is needed for the resource block, full package/gate execution, baseline replay if this discrepancy needs origin classification, and the contract/routing assessment of the YAML case.

```findings
- file: crates/verify/ess-conformance/tests/report_reader_adversary.rs
  line: 61
  category: boundary
  severity: warning
  verdict: INFEASIBLE
  origin: undecided
  message: A YAML report mapping accepts a null implementation identity while the JSON report routes reject the same fixture; no repository YAML report consumer was found.
```

