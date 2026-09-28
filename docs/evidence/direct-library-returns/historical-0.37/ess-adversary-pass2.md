unit: story:direct-library-return-observations; er-library-conformance working tree after nested-presence correction, base 472d35fbe3109ad47f89a65df44b60c1f14acd55
verdict: CONFIRMED
cases: executed 15→22, red 2
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 8 paths this pass; 6 evidence files plus test and build output in assigned managed ER worker tree
needs-coordinator: retain bounded Json resource correction and recheck evidence

```text
 .../tests/ess_direct_return_adversary.rs           | 140 +++++++++++++++++++++
 1 file changed, 140 insertions(+)
```

Adversary-only diff from git diff --no-index --stat /dev/null. The existing 106-line test file gained 34 lines; no ESS implementation edits were made. The previously frozen ER deliverables were untouched.

Pass 1 nested-presence finding is resolved: the five original independent tests now pass (ess-adversary-five-recheck.log). Inspection confirms the direct-return authority preserves nested field presence and the new validation profile checks it, while legacy profiles retain old serialization and checking behavior.

Two new cases drive the unit's resource contract in docs/design/direct-library-returns.md:40 through normal ess/16 source, with a response field of type Json. The actual target returns 65,537 null array members or 130 singleton-array levels, both far below the serialized 1 MiB limit. Each should fail the promised collection/depth bound. The cases were written before either was run and each was run alone before the combined suite.

First execution of json-collection-red alone, exit 101:

```text
running 1 test
test json_return_collection_limit_is_enforced ... FAILED

failures:

---- json_return_collection_limit_is_enforced stdout ----

thread 'json_return_collection_limit_is_enforced' (3002677) panicked at tests/ess_direct_return_adversary.rs:132:5:
assertion `left == right` failed
  left: Passed
 right: Failed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    json_return_collection_limit_is_enforced

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.02s

error: test failed, to rerun pass `--test ess_direct_return_adversary`
```

First execution of json-depth-red alone, exit 101:

```text
running 1 test
test json_return_depth_limit_is_enforced ... FAILED

failures:

---- json_return_depth_limit_is_enforced stdout ----

thread 'json_return_depth_limit_is_enforced' (3004062) panicked at tests/ess_direct_return_adversary.rs:139:5:
assertion `left == right` failed
  left: Passed
 right: Failed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    json_return_depth_limit_is_enforced

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test ess_direct_return_adversary`
```

Suite runs after the new cases existed:

```console
CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo test -p ess-conformance --test direct_returns --locked -j2
```

```text
running 15 tests
test pure_return_native_binary64_remains_explicitly_unsupported ... ok
test pure_return_correct_literal_compiles_without_events ... ok
test pure_return_source_requires_its_format_and_a_successful_typed_response ... ok
test pure_return_unknown_response_coordinate_refuses_at_compile ... ok
test pure_return_generated_shape_is_checked_without_authored_literals ... ok
test pure_return_generators_refuse_unsupported_execution ... ok
test pure_return_correct_literal_passes_without_events ... ok
test pure_return_wrong_literal_fails_named_scenario ... ok
test pure_return_coverage_and_report_retain_exact_admitted_bytes ... ok
test pure_return_payloads_have_an_independent_lossless_resource_bound ... ok
test pure_return_absent_field_and_extra_field_refuse ... ok
test pure_return_nested_presence_is_preserved_and_checked ... ok
test pure_return_complete_nested_values_preserve_order_and_duplicates ... ok
test pure_return_legacy_suite_bytes_unchanged ... ok
test pure_return_old_formats_refuse_before_target_effects ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

```

Exit 0; the existing direct-return binary excludes the independent tests.

```console
CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo test --manifest-path checks/ess-conformance/Cargo.toml --config LOCAL_ESS_PATCH_CONFIG -j2 --test ess_direct_return_adversary
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/ess_direct_return_adversary.rs (checks/ess-conformance/target/debug/deps/ess_direct_return_adversary-7a27d50981916623)

running 7 tests
test authored_named_literal_requires_nested_null_when_absent ... ok
test authored_named_literal_rejects_nested_null_when_omitted ... ok
test json_return_depth_limit_is_enforced ... FAILED
test nested_null_when_absent_requires_explicit_null ... ok
test nested_omitted_when_absent_rejects_explicit_null ... ok
test valid_presence_and_partial_top_level_literals_remain_admitted ... ok
test json_return_collection_limit_is_enforced ... FAILED

failures:

---- json_return_depth_limit_is_enforced stdout ----

thread 'json_return_depth_limit_is_enforced' (3042162) panicked at tests/ess_direct_return_adversary.rs:139:5:
assertion `left == right` failed
  left: Passed
 right: Failed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- json_return_collection_limit_is_enforced stdout ----

thread 'json_return_collection_limit_is_enforced' (3042161) panicked at tests/ess_direct_return_adversary.rs:132:5:
assertion `left == right` failed
  left: Passed
 right: Failed


failures:
    json_return_collection_limit_is_enforced
    json_return_depth_limit_is_enforced

test result: FAILED. 5 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `--test ess_direct_return_adversary`
```

Exit 101; 22 cases executed across the two binaries, 2 red.

| File:line | Verdict | Origin | Finding and reachability |
|---|---|---|---|
| crates/verify/ess-conformance/src/selection.rs:683 | CONFIRMED | introduced | Direct response Json values bypass the promised collection and nesting limits, so oversized collections and deep values pass conformance. A normal synthesized response of primitive type Json reaches this branch; primitive_admits returns true without recursive traversal. Tests measure Passed where the documented bounded observer requires Failed. |

Suggested correction: recursively enforce the direct-return resource profile on opaque Json nodes for both actual values and authored literals, retaining prior profiles unchanged. The new contract introduces this bounded observer; no claim is made that a previous observer enforced these new limits.

Attacked and could not break: the nested-presence correction passes all five independent probes; the 15 existing direct-return cases retain legacy bytes, exact numeric/list values, format admission and report association.

Reviewed source digests before Json correction:

```text
5b5e611f51c951ef6b92101b02056447c487fe8d7b75a40346f89b8d3d444136  direct_response.rs
5e4363fa7070ce51fc19edf65f2d67517075bb106d9bd1e7ec9b199ea07d33dc  selection.rs
c33471d42c8cf6257592bcbc5f6daf547a2b3b908c57f4e31f8750af4b302275  typed_fields.rs
```

Paths written outside the ESS review tree in this pass:

- WORKTREE_ROOT/entity-runtime/er-store-executor-contracts/checks/ess-conformance/tests/ess_direct_return_adversary.rs
- WORKTREE_ROOT/entity-runtime/er-store-executor-contracts/checks/ess-conformance/target
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-json-collection-red.log
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-json-depth-red.log
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-existing-direct-recheck.log
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-seven-red.log
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-pass2.md

The five-case correction log (EVIDENCE_CACHE/store-executor/evidence/ess-adversary-five-recheck.log) was written between passes and is retained as additional recheck evidence.

```findings
- file: crates/verify/ess-conformance/src/selection.rs
  line: 683
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Direct response Json values bypass the promised collection and nesting limits, so oversized collections and deep values pass conformance.
```
