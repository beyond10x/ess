unit: story:direct-library-return-observations; er-library-conformance working tree after both reviewed corrections, base 472d35fbe3109ad47f89a65df44b60c1f14acd55
verdict: nothing found
cases: executed 16→23, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths for this recheck; 3 evidence files and existing managed ER worker build directory
needs-coordinator: none

```text
 .../tests/ess_direct_return_adversary.rs           | 140 +++++++++++++++++++++
 1 file changed, 140 insertions(+)
```

This is the retained adversary-only test diff (git diff --no-index --stat /dev/null), unchanged during the final recheck. No implementation file was edited by this reviewer; no AEP command was run.

The seven independent tests and their initial failing output are retained in ess-adversary-pass1.md and ess-adversary-pass2.md. The four presence violations originally passed runner/authoring and are now rejected. The two Json resource violations originally passed and now fail the named conformance checks. The valid presence/partial-literal control remains green.

Both fixes were inspected: direct_response_declarations preserves nested field presence only for the new observation; ValueProfile::DirectResponse checks nested presence and recursively traverses opaque Json values for the promised resource limits. Older response/selection profiles keep their previous behavior and canonical declaration serialization.

Final rechecks, in order:

```console
CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo test --manifest-path checks/ess-conformance/Cargo.toml --config LOCAL_ESS_PATCH_CONFIG -j2 --test ess_direct_return_adversary
```

```text
   Compiling ess-conformance v0.37.0 (WORKTREE_ROOT/ess/er-library-conformance/crates/verify/ess-conformance)
   Compiling er-ess-conformance v0.1.0 (WORKTREE_ROOT/entity-runtime/er-store-executor-contracts/checks/ess-conformance)
    Finished `test` profile [unoptimized] target(s) in 16.26s
     Running tests/ess_direct_return_adversary.rs (checks/ess-conformance/target/debug/deps/ess_direct_return_adversary-7a27d50981916623)

running 7 tests
test authored_named_literal_requires_nested_null_when_absent ... ok
test authored_named_literal_rejects_nested_null_when_omitted ... ok
test json_return_depth_limit_is_enforced ... ok
test nested_omitted_when_absent_rejects_explicit_null ... ok
test nested_null_when_absent_requires_explicit_null ... ok
test json_return_collection_limit_is_enforced ... ok
test valid_presence_and_partial_top_level_literals_remain_admitted ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

```

Exit 0.

```console
CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo test -p ess-conformance --test direct_returns --locked -j2
```

```text
running 16 tests
test pure_return_native_binary64_remains_explicitly_unsupported ... ok
test pure_return_correct_literal_compiles_without_events ... ok
test pure_return_source_requires_its_format_and_a_successful_typed_response ... ok
test pure_return_unknown_response_coordinate_refuses_at_compile ... ok
test pure_return_generated_shape_is_checked_without_authored_literals ... ok
test pure_return_generators_refuse_unsupported_execution ... ok
test pure_return_wrong_literal_fails_named_scenario ... ok
test pure_return_correct_literal_passes_without_events ... ok
test pure_return_coverage_and_report_retain_exact_admitted_bytes ... ok
test pure_return_payloads_have_an_independent_lossless_resource_bound ... ok
test pure_return_nested_presence_is_preserved_and_checked ... ok
test pure_return_absent_field_and_extra_field_refuse ... ok
test pure_return_complete_nested_values_preserve_order_and_duplicates ... ok
test pure_return_legacy_suite_bytes_unchanged ... ok
test pure_return_json_obeys_the_same_recursive_resource_bounds ... ok
test pure_return_old_formats_refuse_before_target_effects ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

```

Exit 0. The existing 16 cases include the implementor's regressions for both findings and valid boundary controls. Combined with the independent seven cases, 23 cases executed with zero failures.

No outstanding finding in the reviewed corrections. This is a bounded recheck of the two measured defects and the unit's direct-return regression suite, not a claim about the complete ESS repository gate.

Final source digests:

```text
5b5e611f51c951ef6b92101b02056447c487fe8d7b75a40346f89b8d3d444136  direct_response.rs
616ea596a8a6757fdce2633c1610adcf8c0edf3262f68edecf66ee6b7e2662e5  selection.rs
c33471d42c8cf6257592bcbc5f6daf547a2b3b908c57f4e31f8750af4b302275  typed_fields.rs
```

Outside-write paths for this recheck:

- WORKTREE_ROOT/entity-runtime/er-store-executor-contracts/checks/ess-conformance/target
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-seven-recheck.log
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-existing-direct-final.log
- EVIDENCE_CACHE/store-executor/evidence/ess-adversary-final.md

Separately, the coordinator requested reconstruction of the previously performed ER production mutation patches. These two evidence-only files were written in the same assigned cache directory; git apply --check passed and no production file was edited:

- EVIDENCE_CACHE/store-executor/evidence/store-production-mutation.patch
- EVIDENCE_CACHE/store-executor/evidence/executor-production-mutation.patch

```findings
[]
```

