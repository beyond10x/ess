unit: story:direct-library-return-observations; er-library-conformance working tree based on 472d35fbe3109ad47f89a65df44b60c1f14acd55
verdict: CONFIRMED
cases: executed 14→19, red 4
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 9 paths; 7 evidence paths under assigned cache plus test and build output in assigned managed ER worker tree
needs-coordinator: route nested presence defect to ESS implementor; retain recheck evidence after correction

```text
 .../tests/ess_direct_return_adversary.rs           | 106 +++++++++++++++++++++
 1 file changed, 106 insertions(+)
```

This is the adversary-only diff, obtained with git diff --no-index --stat /dev/null on the new untracked test file. The ESS implementation tree was read only. The ER worker tree already held the separately frozen store/executor deliverables; none was changed during review. No AEP command or implementation edit was made.

Tests were written before execution. The first two tests were each executed alone before the existing direct-return suite; the two authored cases were added and each executed alone before the final five-case suite. Existing count 14 was supplied by the implementor and then measured after the first probes existed. The combined measured suite after additions runs 19 cases across two binaries.

The public path is ordinary ess/16 source → Specification::assemble → compile → synthesize → AdmittedSuite::from_suite → Runner::run_admitted. The target supplies only its actual response map. No observation schema or report is forged. Authored cases use authored::compile with ordinary ess-scenario/4 documents.

The source declares library.api.Item.note as Optional<String> with an explicit presence policy. Runner cases return item:{} for null_when_absent and item:{note:null} for omitted_when_absent; both must fail shape checking. Authored cases put those same malformed objects in a named response literal; both must be refused because named literals are complete objects. The control confirms the correct encodings pass and partial top-level response:{} remains admitted.

Case nested-null: first execution alone, exit 101.

```text
running 1 test
test nested_null_when_absent_requires_explicit_null ... FAILED

failures:

---- nested_null_when_absent_requires_explicit_null stdout ----

thread 'nested_null_when_absent_requires_explicit_null' (2938500) panicked at tests/ess_direct_return_adversary.rs:57:5:
assertion `left == right` failed
  left: Passed
 right: Failed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    nested_null_when_absent_requires_explicit_null

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test ess_direct_return_adversary`
```

Case nested-omitted: first execution alone, exit 101.

```text
running 1 test
test nested_omitted_when_absent_rejects_explicit_null ... FAILED

failures:

---- nested_omitted_when_absent_rejects_explicit_null stdout ----

thread 'nested_omitted_when_absent_rejects_explicit_null' (2946069) panicked at tests/ess_direct_return_adversary.rs:62:5:
assertion `left == right` failed
  left: Passed
 right: Failed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    nested_omitted_when_absent_rejects_explicit_null

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test ess_direct_return_adversary`
```

Case authored-null: first execution alone, exit 101.

```text
running 1 test
test authored_named_literal_requires_nested_null_when_absent ... FAILED

failures:

---- authored_named_literal_requires_nested_null_when_absent stdout ----

thread 'authored_named_literal_requires_nested_null_when_absent' (2965974) panicked at tests/ess_direct_return_adversary.rs:86:5:
assertion `left == right` failed
  left: 1
 right: 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    authored_named_literal_requires_nested_null_when_absent

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.01s

error: test failed, to rerun pass `--test ess_direct_return_adversary`
```

Case authored-omitted: first execution alone, exit 101.

```text
running 1 test
test authored_named_literal_rejects_nested_null_when_omitted ... FAILED

failures:

---- authored_named_literal_rejects_nested_null_when_omitted stdout ----

thread 'authored_named_literal_rejects_nested_null_when_omitted' (2968603) panicked at tests/ess_direct_return_adversary.rs:93:5:
assertion `left == right` failed
  left: 1
 right: 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    authored_named_literal_rejects_nested_null_when_omitted

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test ess_direct_return_adversary`
```

Existing suite, run after the first added probes existed:

```console
CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo test -p ess-conformance --test direct_returns --locked -j2
```

```text
running 14 tests
test pure_return_native_binary64_remains_explicitly_unsupported ... ok
test pure_return_unknown_response_coordinate_refuses_at_compile ... ok
test pure_return_correct_literal_compiles_without_events ... ok
test pure_return_generated_shape_is_checked_without_authored_literals ... ok
test pure_return_generators_refuse_unsupported_execution ... ok
test pure_return_wrong_literal_fails_named_scenario ... ok
test pure_return_correct_literal_passes_without_events ... ok
test pure_return_coverage_and_report_retain_exact_admitted_bytes ... ok
test pure_return_source_requires_its_format_and_a_successful_typed_response ... ok
test pure_return_payloads_have_an_independent_lossless_resource_bound ... ok
test pure_return_absent_field_and_extra_field_refuse ... ok
test pure_return_complete_nested_values_preserve_order_and_duplicates ... ok
test pure_return_legacy_suite_bytes_unchanged ... ok
test pure_return_old_formats_refuse_before_target_effects ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

```

Exit 0. This explicitly excludes the separately housed adversary test file.

Added-case suite, run after every red case was executed alone:

```console
CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo test --manifest-path checks/ess-conformance/Cargo.toml --config LOCAL_ESS_PATCH_CONFIG -j2 --test ess_direct_return_adversary
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.03s
     Running tests/ess_direct_return_adversary.rs (checks/ess-conformance/target/debug/deps/ess_direct_return_adversary-7a27d50981916623)

running 5 tests
test authored_named_literal_requires_nested_null_when_absent ... FAILED
test authored_named_literal_rejects_nested_null_when_omitted ... FAILED
test nested_null_when_absent_requires_explicit_null ... FAILED
test nested_omitted_when_absent_rejects_explicit_null ... FAILED
test valid_presence_and_partial_top_level_literals_remain_admitted ... ok

failures:

---- authored_named_literal_requires_nested_null_when_absent stdout ----

thread 'authored_named_literal_requires_nested_null_when_absent' (2970586) panicked at tests/ess_direct_return_adversary.rs:86:5:
assertion `left == right` failed
  left: 1
 right: 0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- authored_named_literal_rejects_nested_null_when_omitted stdout ----

thread 'authored_named_literal_rejects_nested_null_when_omitted' (2970585) panicked at tests/ess_direct_return_adversary.rs:93:5:
assertion `left == right` failed
  left: 1
 right: 0

---- nested_null_when_absent_requires_explicit_null stdout ----

thread 'nested_null_when_absent_requires_explicit_null' (2970587) panicked at tests/ess_direct_return_adversary.rs:61:5:
assertion `left == right` failed
  left: Passed
 right: Failed

---- nested_omitted_when_absent_rejects_explicit_null stdout ----

thread 'nested_omitted_when_absent_rejects_explicit_null' (2970588) panicked at tests/ess_direct_return_adversary.rs:66:5:
assertion `left == right` failed
  left: Passed
 right: Failed


failures:
    authored_named_literal_rejects_nested_null_when_omitted
    authored_named_literal_requires_nested_null_when_absent
    nested_null_when_absent_requires_explicit_null
    nested_omitted_when_absent_rejects_explicit_null

test result: FAILED. 1 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `--test ess_direct_return_adversary`
```

Exit 101.

| File:line | Verdict | Origin | Finding and reachability |
|---|---|---|---|
| crates/verify/ess-conformance/src/direct_response.rs:76 | CONFIRMED | introduced | Direct response authority drops nested struct presence policies, so malformed returns and malformed authored named-object literals pass. Ordinary source compilation and synthesis reach the new caller at this line; typed_fields::declarations drops child naming.presence at typed_fields.rs:38 and selection.rs:640 does not enforce it during nested validation. The new direct-return path exposes this behavior under its promise of complete response type/presence checking. |

Suggested correction: preserve nested presence in the new direct-return declaration authority and enforce it recursively for the DirectResponse validation profile. Preserve legacy suite bytes and behavior by keeping this policy scoped to the new format/profile. Add runner/admission twins and retain the valid controls.

Attacked and could not break: the supplied 14 direct-return tests retain exact numeric/list literals, closed response shape, Binary64 refusal, format admission, old canonical bytes, independent value bounds, generator refusals, and exact suite/report association. Both valid nested encodings and partial top-level authored expectations pass the independent control.

Reviewed source digests before correction:

```text
8595714dd1d4b729fad161d6a590dd63d70530a6b800ed2991e3da49d4d182a0  direct_response.rs
cc92440eb0616dda4fde1e7bf6b62d231df510e764fa81592731cf47dfa75839  selection.rs
1ac408a685b31f3377da851ac2c88b78eefe39e9b85b315e7dadcd0c8c515c8a  typed_fields.rs
```

Every path written outside the ESS review worktree during this review:

- WORKTREE_ROOT/entity-runtime/er-store-executor-contracts/checks/ess-conformance/tests/ess_direct_return_adversary.rs
- WORKTREE_ROOT/entity-runtime/er-store-executor-contracts/checks/ess-conformance/target (Cargo test build output in that existing managed worker tree)
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-nested-null-red.log
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-nested-omitted-red.log
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-authored-null-red.log
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-authored-omitted-red.log
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-existing-direct-suite.log
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-five-red.log
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-pass1.md

```findings
- file: crates/verify/ess-conformance/src/direct_response.rs
  line: 76
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Direct response authority drops nested struct presence policies, so malformed returns and malformed authored named-object literals pass.
```
