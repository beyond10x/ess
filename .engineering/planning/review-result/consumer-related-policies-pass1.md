---
format: aep.planning-md/3
id: review-result:consumer-related-policies-pass1
kind: review-result
status: active
title: Independent copied policy and related view review
relations:
- reviews: story:feature-request-307
- reviews: story:feature-request-360
revision: 1
---
approve
unit: story:feature-request-307 + story:feature-request-360, frozen patch c21546dad925ec214e8f99944c51c9d8dfbc64bb5bce63461535f947c8dcf64e
verdict: nothing found
cases: own bounded probe executed 2 → 4, red 0; six cumulative test executions across two successful runs
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 authorized cache/temp paths
needs-coordinator: combined full-package validation and delivery remain pending

Observed implementation diff (the seven frozen paths predate this review; I changed none):

```text
 crates/verify/ess-conformance/src/synthesize.rs    |  81 +++++-
 .../ess-conformance/src/synthesize/related.rs      |  34 ++-
 .../ess-conformance/src/synthesize/subject_fact.rs |  49 +++-
 .../tests/fixtures/subject-guard-copied-field.yaml |  23 +-
 .../tests/subject_guard_copied_field.rs            | 321 +++++++++++++++++----
 5 files changed, 438 insertions(+), 70 deletions(-)
```

All seven source SHA-256 values were checked before and after review and match related-source-sha256.txt. Report SHA bb87e508770456e5415a3c8d08d2a7ef9c028d642767a4e24f27333732659612 and patch SHA c21546dad925ec214e8f99944c51c9d8dfbc64bb5bce63461535f947c8dcf64e match the assigned handoff. Acceptance read: canonical 307/360 revision 7.

The counter-edge source hypothesis at subject_fact.rs:5151,5223,5230,5333 was NOT confirmed. I constructed ordinary single-valued Mode=[Run] input and nested-predicate variants of existing adversary_counter_limit_pass1::a_guarded_sibling_answering_one_step_short_is_witnessed. The models retain the same honest behavior and shifted `over >=4` mutant. Both the original control and new variants reject that mutant. I withdraw the preliminary concern as a finding; source-only reasoning did not establish an observable regression.

The temporary Rust test file was retained verbatim as target/backlog-input/review-counter-probe.rs and removed only from its temporary tests/ location after byte equality verification. No frozen file was touched. To replay, copy that retained Rust file to crates/verify/ess-conformance/tests/temporary_review_related_counter.rs and run the exact command below. It includes the existing fixture/harness plus three clearly named variant cases; the filter selected four tests, not the six unrelated inherited cases.

Initial probe compilation failed because my temporary extraction accidentally duplicated trailing test definitions. That is my harness mistake, not product red evidence; review-counter-probe.log retains it, and it executed zero tests. Corrected probe run (`review-counter-probe-run.log`):

```text
   Compiling ess-conformance v0.51.0 (ess-backlog-related-20261002/crates/verify/ess-conformance)
    Finished `test` profile [unoptimized] target(s) in 2.51s
     Running tests/temporary_review_related_counter.rs (../ess-backlog-synthesis-20261002/target/debug/deps/temporary_review_related_counter-bf65a115d3da4e0b)

running 2 tests
test a_guarded_sibling_answering_one_step_short_is_witnessed ... ok
test a_guarded_sibling_answering_one_step_short_is_witnessed_with_ordinary_input ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.51s

```

Expanded bounded probe (`review-counter-nested.log`), exact command:

```console
CARGO_TARGET_DIR=../ess-backlog-synthesis-20261002/target CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 TMPDIR=<task-specific-external-cache> cargo test -p ess-conformance --test temporary_review_related_counter a_guarded_sibling_answering_one_step_short --locked -- --nocapture
```

```text
   Compiling ess-conformance v0.51.0 (ess-backlog-related-20261002/crates/verify/ess-conformance)
    Finished `test` profile [unoptimized] target(s) in 0.89s
     Running tests/temporary_review_related_counter.rs (../ess-backlog-synthesis-20261002/target/debug/deps/temporary_review_related_counter-bf65a115d3da4e0b)

running 4 tests
test a_guarded_sibling_answering_one_step_short_is_witnessed_with_nested_noop_control ... ok
test a_guarded_sibling_answering_one_step_short_is_witnessed ... ok
test a_guarded_sibling_answering_one_step_short_is_witnessed_with_nested_ordinary_input ... ok
test a_guarded_sibling_answering_one_step_short_is_witnessed_with_ordinary_input ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.28s

```

Both runtime invocations exited 0. This was a control/variant probe on the frozen treatment; I did not rebuild the exact base or rerun the owner's 57-case suite. The owner's identical-test baseline51pass/6fail and treatment57pass/0fail evidence was read, not claimed as my own execution. Full package remains coordinator-owned.

Source and evidence reviewed without finding another counterexample:

- Optional absence goals only apply to resolved top-level Optional fields, remove direct comparisons depending on that same path from the search goal, and keep independent conjuncts. `goal_input` still requires complete `selects` branch evaluation (subject_fact.rs:4938-4977,5333). MAX_BOUNDARIES remains8. Honest tests require Passed; the interpreter limitation is an explicitly separate test.
- Related copied values are settled using the actual supplied symbolic input and captured referenced rows. `invoke_created_with` preserves the incoming chain and merges steps/source facts; payload-only reads keep their previous path (synthesize.rs:4174-4224). Cycle checks are explicit, and existing singleton sources suppress source decoys (related.rs:161-192,269).
- Copied-field exclusion remains typed through `row.shows`; the new companion is sought only when the view identifies rows and the entity is not singleton. No param is dropped or arbitrary source identity invented (synthesize.rs:6066-6080,6116-6120).
- Honest targets assert all relevant creation/later/transition cases pass. Four source/filter mutants require Failed in Pack; nine policy mutants plus CopiesNothing require actual Failed, not Unsupported (related_copied_view_parameter.rs:289-319; subject_guard_copied_field.rs:437-489). Source captures and true/false/absent values are inspected explicitly.

Outside-tree writes are compiler output in ../ess-backlog-synthesis-20261002/target and process temporaries under <task-specific-external-cache>, both explicitly authorized by coordinator. No implementation, AEP, remote or existing-test mutation occurred. No build remains running.

```findings
[]
```
