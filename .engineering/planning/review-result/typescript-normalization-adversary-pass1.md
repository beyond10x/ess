---
format: aep.planning-md/1
id: review-result:typescript-normalization-adversary-pass1
kind: review-result
status: active
title: Independent TypeScript normalization source and native attack
relations:
- reviews: story:typescript-normalization-target
revision: 1
---
unit: story:typescript-normalization-target at 68f0b57dfe01f069c38b070784ea63ed27bf92da plus the three-file test-only patch
verdict: nothing found
cases: executed 317→320, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 86 files plus 2 shared tool-cache roots
needs-coordinator: none

```console
$ git --no-pager diff --stat
$ git diff --check
```

Both exited 0 with empty output: no frozen tracked file changed. The new files remain untracked because this role cannot stage them. The following are the actual per-path `git --no-pager diff --no-index --stat -- /dev/null <new path>` outputs (exit 1 means the new file differs from /dev/null):

```text
 .../tests/normalization_typescript_adversary.rs    | 196 +++++++++++++++++++++
 1 file changed, 196 insertions(+)
 .../normalization_typescript_adversary.ts.txt      | 42 ++++++++++++++++++++++
 1 file changed, 42 insertions(+)
 .../tests/normalization_typescript_adversary.rs    | 84 ++++++++++++++++++++++
 1 file changed, 84 insertions(+)
```

The complete patch has three test paths, 322 insertions and no deletions; `test-only.patch` retains all new contents. No implementation, existing test, frozen fixture, planning, documentation, Git index, or worktree lifecycle was changed. All 1,018 frozen content and file-mode identities match the coordinator manifest. The read-only raw directory comparison also matches all twelve old Rust/Go maps: 218 artifact files plus their canonical map, at actual producer 0.19.0.

This pass used the frozen source, caller/test diff, governed story, adversary charter, and the bound six-document patch SHA256 293adae766c12223d3c943d988d2500c75c6ac8e55c6b1a39bd49cda82948b09. The equality limitation remains explicitly bound; this attack makes no new eligibility or public intersection-shape claim.

Public path substitutions: $WORKTREE is the frozen unit tree; $SCRATCH is the assigned adversary packet; $EVIDENCE is the parent TypeScript packet; $SCCACHE and $GOCACHE are shared tool caches. Exact original paths and verbatim logs are retained in private-report.md and write-inventory.json.

1. New cases and isolated first executions

Every selected test was authored before its first execution, and each ran alone before either related suite. All five final test functions are green; only three are selected in the ordinary feature configuration. The two additional functions require `typescript-typecheck`.

| Test | Independently specified boundary |
|---|---|
| schema-contract/tests/normalization_typescript_adversary.rs: referenced_property_names_stop_at_plan_admission_and_annotation_decoys_stay_inert | The referenced propertyNames bundle is refused at both original input/output root-selection locations even in an unused branch; annotation JSON stays inert in an independently usable branch. |
| same file: compiler_owned_integer_map_key_pattern_is_not_implicitly_qualified | A compiler-minted Map<Integer,String> must refuse TypeScript generation at the exact escaped /models/<projection>/$defs/sample.keys.Row/properties/lookup~1~0/propertyNames/pattern location. |
| same file: native_bounds_compare_the_exact_binary_fraction_not_its_shortest_decimal | Eight literal min/max controls distinguish the exact binary integer 1000000000000000128 from the shortest decimal 1000000000000000100 and neighboring bigint values. Complete native serialized JSON and refusal fields also match the reference. |
| same file plus fixtures/normalization_typescript_adversary.ts.txt: native_retained_reentry_and_escaped_nested_position_authority | Eight public text-retain/base64-reentry cases preserve exact token spans, duplicates and huge tokens only inside selected captures/discarded tails, literal versus escaped astral text, unconditional empty token IiI=, null versus absence, and first-entry Unicode precedence. Two public nested tuple executions use escaped branch/field static keys. One binding assertion and three deliberate private-map corruptions check absent/zero/wrong full arity. |
| ess-cli/tests/normalization_typescript_adversary.rs: generation_and_check_need_no_native_tools_and_module_refusal_preserves_bytes | Native tools absent from PATH and compiler overrides unset: scoped-package publication and two drift checks succeed; a Go-only module request fails while every existing reported output byte stays unchanged. No dist or node_modules directory appears. |

Three initial probes failed because of test-construction assumptions, not implementation defects. Their original exit-101 outputs are retained below: (a) qualified-bundle propertyNames with a ref is outside public Plan admission, so the intended TypeScript qualifier attack never reached the target; the case was corrected into an explicit admission-boundary assertion, (b) the positional refusal spelling was corrected from the invented input_positional to the already bound positional_element_type, cited at docs/design/positional-array-normalization.md:334, and (c) the test read projection_digest from the root instead of its nested ModelIdentity and now matches Root::Model directly. No existing assertion was edited. These are not counted as product-red cases or findings.

All Cargo invocations used this environment (CARGO_TARGET_DIR was removed, never assigned):

```sh
env -u CARGO_TARGET_DIR \
  RUSTC_WRAPPER=/usr/bin/sccache CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 \
  CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  ESS_GO_COMPILER=/usr/bin/go \
  ESS_TYPESCRIPT_COMPILER=/usr/lib/node_modules/typescript/lib/tsc.js \
  TMPDIR=$SCRATCH/tmp cargo <arguments below>
```

Commands ran in $WORKTREE. stdout and stderr were merged through tee, with Bash pipefail preserving the command exit. The logs below preserve the original output with only the declared private-path substitutions.


isolated-property-names.log

```console
$ cargo test --offline --locked -p schema-contract --test normalization_typescript_adversary referenced_property_name_constraints_refuse_unused_branches_with_exact_locations -- --exact --nocapture
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 2.52s
     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-77c8d31bfda31b23)

running 1 test

thread 'referenced_property_name_constraints_refuse_unused_branches_with_exact_locations' (3245594) panicked at crates/generate/schema-contract/tests/normalization_typescript_adversary.rs:39:79:
called `Result::unwrap()` on an `Err` value: Refused([Finding { pointer: "/branches/unused/0/input", rule: "root_selection", detail: "/components/schemas/Map/propertyNames: propertyNames" }, Finding { pointer: "/branches/unused/0/output", rule: "root_selection", detail: "/components/schemas/Map/propertyNames: propertyNames" }])
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test referenced_property_name_constraints_refuse_unused_branches_with_exact_locations ... FAILED

failures:

failures:
    referenced_property_name_constraints_refuse_unused_branches_with_exact_locations

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p schema-contract --test normalization_typescript_adversary`
```

Exit: 101.

isolated-exact-bound.log

```console
$ cargo test --offline --locked -p schema-contract --features typescript-typecheck --test normalization_typescript_adversary native_bounds_compare_the_exact_binary_fraction_not_its_shortest_decimal -- --exact --nocapture
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.33s
     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-305814febf43591c)

running 1 test
qualified engine {"execPath":"/usr/bin/node","node":"v22.23.1","v8":"12.4.254.21-node.56"}; compiler /usr/lib/node_modules/typescript/lib/tsc.js
adversary-exact-bound: executed 8; failed 0; repeated fresh-call checks 8
test native_bounds_compare_the_exact_binary_fraction_not_its_shortest_decimal ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 1.30s

```

Exit: 0.

isolated-reentry-position.log

```console
$ cargo test --offline --locked -p schema-contract --features typescript-typecheck --test normalization_typescript_adversary native_retained_reentry_and_escaped_nested_position_authority -- --exact --nocapture
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.39s
     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-305814febf43591c)

running 1 test

thread 'native_retained_reentry_and_escaped_nested_position_authority' (3257114) panicked at crates/generate/schema-contract/tests/normalization_typescript_adversary.rs:133:13:
assertion `left == right` failed: {"input":"{\"provider\":{},\"pair\":[false,\"r\",1e999]}","pointer":"/input/pair/0","rule":"input_positional"}
  left: "positional_element_type"
 right: String("input_positional")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test native_retained_reentry_and_escaped_nested_position_authority ... FAILED

failures:

failures:
    native_retained_reentry_and_escaped_nested_position_authority

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p schema-contract --test normalization_typescript_adversary`
```

Exit: 101.

isolated-reentry-position-corrected.log

```console
$ cargo test --offline --locked -p schema-contract --features typescript-typecheck --test normalization_typescript_adversary native_retained_reentry_and_escaped_nested_position_authority -- --exact --nocapture
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.36s
     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-305814febf43591c)

running 1 test
qualified engine {"execPath":"/usr/bin/node","node":"v22.23.1","v8":"12.4.254.21-node.56"}; compiler /usr/lib/node_modules/typescript/lib/tsc.js
adversary retained reentry and nested arity: executed 14; failed 0 (8 public reentries, 2 public nested executions, 1 binding assertion, 3 private corruptions)
test native_retained_reentry_and_escaped_nested_position_authority ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 1.23s

```

Exit: 0.

isolated-model-property-names.log

```console
$ cargo test --offline --locked -p schema-contract --test normalization_typescript_adversary compiler_owned_integer_map_key_pattern_is_not_implicitly_qualified -- --exact --nocapture
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.41s
     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-77c8d31bfda31b23)

running 1 test

thread 'compiler_owned_integer_map_key_pattern_is_not_implicitly_qualified' (3264317) panicked at crates/generate/schema-contract/tests/normalization_typescript_adversary.rs:70:85:
called `Option::unwrap()` on a `None` value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test compiler_owned_integer_map_key_pattern_is_not_implicitly_qualified ... FAILED

failures:

failures:
    compiler_owned_integer_map_key_pattern_is_not_implicitly_qualified

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p schema-contract --test normalization_typescript_adversary`
```

Exit: 101.

isolated-cli-no-tools.log

```console
$ cargo test --offline --locked -p ess-cli --test normalization_typescript_adversary generation_and_check_need_no_native_tools_and_module_refusal_preserves_bytes -- --exact --nocapture
   Compiling ess-cli v0.19.0 ($WORKTREE/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.32s
     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-961ce397dfd9b4b9)

running 1 test
test generation_and_check_need_no_native_tools_and_module_refusal_preserves_bytes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

```

Exit: 0.

isolated-model-property-names-corrected.log

```console
$ cargo test --offline --locked -p schema-contract --test normalization_typescript_adversary compiler_owned_integer_map_key_pattern_is_not_implicitly_qualified -- --exact --nocapture
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.41s
     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-77c8d31bfda31b23)

running 1 test
test compiler_owned_integer_map_key_pattern_is_not_implicitly_qualified ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

```

Exit: 0.

isolated-property-names-admission-corrected.log

```console
$ cargo test --offline --locked -p schema-contract --test normalization_typescript_adversary referenced_property_names_stop_at_plan_admission_and_annotation_decoys_stay_inert -- --exact --nocapture
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.39s
     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-77c8d31bfda31b23)

running 1 test
test referenced_property_names_stop_at_plan_admission_and_annotation_decoys_stay_inert ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.02s

```

Exit: 0.

2. Related suites and final test-harness checks

The before number 317 comes from the implementor's frozen handoff, not from an adversary suite run before authoring. The same ordinary command now ran 320 cases in 56 runner-summary groups, with zero failed or ignored. The three added ordinary cases account for the increase.

The explicit related native command ran ten runner cases: the six inherited groups, two new native groups, and two non-native admission controls repeated under the feature. Its six inherited groups executed 3,091 shared vectors, 26,966 numeric executions, 31 schema controls, 56 boundary controls, seven private validator conjunctions, and 25 equality controls: 30,176 controls total. The new native groups add 22 checks: 18 public runtime cases, one private binding assertion, and three private corruption probes. These are case counts, not a count of API invocations: a retained-document scenario may call both entries, while two Unicode scenarios stop at the first entry. Eight numeric cases also have separate repeated fresh-call checks, as the raw log states; those repeats are not added again to the 22. The 13 named value-only APIs remain explicitly inapplicable to the text/helper corpus, with no newly skipped applicable vector.

Initial scoped schema Clippy found five test-style lint diagnostics (literal separators, two needless by-value helper arguments, and two numeric-cast lints). Only these new test helpers changed: the stage helper now consumes its value, bundle input is borrowed, and the exact binary integer assertion uses fixed decimal formatting without a cast. Both scoped Clippy commands then passed. After that test-only edit, all four schema adversary functions were rerun together and passed; the unrelated inherited suite was not repeated. Scoped rustfmt check and git diff --check exited 0.

related-default-packages.log

```console
$ cargo test --offline --locked -p schema-contract -p ess-cli
   Compiling ess-cli v0.19.0 ($WORKTREE/crates/edge/ess-cli)
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.44s
     Running unittests src/main.rs (target/debug/deps/ess-bb7ad213e0b4cc6b)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/authored_scenarios.rs (target/debug/deps/authored_scenarios-1c2c3df406105ba6)

running 25 tests
test author_nonmatching_only ... ok
test author_empty ... ok
test author_nested_only ... ok
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test web_nested_only ... ok
test go_nested_only ... ok
test ir_nonmatching_only ... ok
test go_empty ... ok
test go_nonmatching_only ... ok
test ir_empty ... ok
test run_nonmatching_only ... ok
test web_empty ... ok
test run_empty ... ok
test web_nonmatching_only ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test run_nested_only ... ok
test ir_nested_only ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s

     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-1b975346dabc52e1)

running 9 tests
test selected_directory_symlink_reports_the_requested_path_and_stays_shallow ... ok
test empty_selection_cannot_follow_or_replace_an_output_symlink ... ok
test non_utf8_matching_content_is_refused_before_any_output_write ... ok
test matching_broken_links_are_read_errors_even_beside_a_valid_source ... ok
test lexical_selection_order_decides_the_first_read_error ... ok
test committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners ... ok
test omitted_scenarios_ignore_a_poisoned_working_directory_default ... ok
test empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner ... ok
test valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s

     Running tests/authored_site.rs (target/debug/deps/authored_site-ef4cfdbdb2eedafd)

running 9 tests
test binary_downloads_are_not_silently_decoded ... ok
test an_explicit_missing_front_page_is_an_error ... ok
test asset_symlink_output_is_refused_before_any_page_changes ... ok
test publication_without_strict_mode_preserves_unpublished_links ... ok
test a_page_identity_can_itself_end_in_html ... ok
test strict_links_check_local_and_cross_page_fragments ... ok
test undeclared_existing_and_escaping_targets_are_refused_with_source_lines ... ok
test assets_cannot_collide_with_generated_output_or_escape_it ... ok
test selected_sources_resolve_after_relocation_with_queries_fragments_and_downloads ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-8cc8a001564682d0)

running 1 test
test cli_composition_obeys_the_independently_authored_vectors ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running tests/binary64_count_adversary.rs (target/debug/deps/binary64_count_adversary-157c78d411c7247c)

running 2 tests
test cli_binary64_model_and_original_suites_preserve_report_destinations_in_both_formats ... ok
test generated_go_binary64_shapes_refuse_before_factory_and_report_publication ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.85s

     Running tests/binary64_publication.rs (target/debug/deps/binary64_publication-2aa6dae635cf5e49)

running 1 test
test binary64_sparse_model_refuses_every_unsupported_publication_route ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-e6799b61d16bc84b)

running 1 test
test binary64_publication_never_replaces_sources_or_partially_updates_a_library ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/command_surface.rs (target/debug/deps/command_surface-820c2588e9f0042b)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-770e0fe494304b6a)

running 4 tests
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/count_reports.rs (target/debug/deps/count_reports-e56e7768c00ccbc3)

running 3 tests
test count_report_opt_in_has_a_distinct_detailed_surface_and_unknown_coverage ... ok
test count_cli_configuration_and_original_suite_refusals_preserve_destinations ... ok
test count_cli_preserves_default_bytes_and_standalone_detailed_pairing ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s

     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-b1bba5ffbcb3cf51)

running 3 tests
test generated_go_skip_counts_preserve_opaque_ids_and_strictness_without_a_destination ... ok
test generated_go_abnormal_teardown_cannot_publish_a_completed_skip ... ok
test generated_go_rejects_closed_predicate_metadata_before_any_target ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.13s

     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-1638f9d800bd41df)

running 2 tests
test generated_go_abnormal_unsupported_error_formatting_cannot_complete ... ok
test generated_go_admits_only_typed_predicate_paths_and_operator_envelopes ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.05s

     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-5f67664c69a669b0)

running 4 tests
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_rust_cli_writes ... ok
test a_binding_function_capture_is_refused_before_web_cli_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-ffe05f2b4a2d9df9)

running 13 tests
test count_go_predicate_admission_matches_rust_leaf_grammar ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test count_report_skip_only_is_inconclusive_without_actual_failures ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test count_retained_runtime_preserves_legacy_behavior_and_does_not_gain_version_checks ... ok
test count_go_empty_selection_is_inconclusive_and_clock_conversion_is_checked ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test count_go_actual_producers_keep_skip_error_and_teardown_categories ... ok
test count_go_refusals_precede_targets_and_incomplete_runs_never_publish ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.48s

     Running tests/model_types.rs (target/debug/deps/model_types-d1ddd7a24ad016b3)

running 3 tests
test all_type_binary64_libraries_publish_finite_codecs_with_atomic_preflight ... ok
test root_selection_and_output_refusals_preserve_existing_files ... ok
test each_target_retains_the_same_model_selection_and_distinct_input_provenance ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/normalization.rs (target/debug/deps/normalization-ac90639e6bf9bab8)

running 17 tests
test unsupported_go_pattern_has_no_successful_partial_artifact ... ok
test incompatible_later_destination_preserves_the_entire_existing_output ... ok
test output_symlinks_and_hardlinks_are_refused_without_mutation ... ok
test generated_paths_cannot_replace_canonical_source_inputs_or_follow_links ... ok
test stale_bundle_missing_source_and_unknown_dispatch_refuse ... ok
test qualified_go_base64_pattern_is_published_without_decoding_the_value ... ok
test positional_cli_refuses_unused_branches_before_publication ... ok
test output_cannot_replace_any_declared_input ... ok
test checked_recipe_and_run_are_deterministic_with_json_only_stdout ... ok
test explicit_binary64_inputs_run_through_the_cli_without_weakening_version_one ... ok
test generation_checks_refuse_before_creating_or_changing_destinations ... ok
test failed_check_or_execution_preserves_output_and_does_not_emit_a_result ... ok
test raw_json_capture_runs_before_schema_and_keeps_failure_output_untouched ... ok
test generated_libraries_match_the_api_and_drift_check_never_repairs_files ... ok
test positional_cli_prepares_text_and_refuses_before_publication ... ok
test model_sources_are_compiled_pinned_and_protected_by_the_cli ... ok
test binary64_cli_keeps_numeric_identity_and_emits_checked_format_five ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.91s

     Running tests/normalization_positional_adversary.rs (target/debug/deps/normalization_positional_adversary-04c8fc26cdea59df)

running 2 tests
test cli_runtime_grammar_controls_preserve_existing_output ... ok
test cli_invalid_policy_blocks_all_publication_and_valid_metadata_is_drift_checked ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

     Running tests/normalization_typescript.rs (target/debug/deps/normalization_typescript-f409a899280f8a84)

running 7 tests
test unqualified_profile_refuses_with_source_pointer_and_no_partial_files ... ok
test model_inputs_are_recompiled_pinned_and_protected_before_typescript_generation ... ok
test retained_recipe_and_bundle_inputs_cannot_be_overwritten ... ok
test typescript_cli_emits_an_accounted_executable_package ... ok
test module_and_late_path_conflicts_refuse_before_any_publication ... ok
test generated_file_and_parent_links_refuse_without_touching_their_destinations ... ok
test every_planned_file_participates_in_read_only_drift_checks ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s

     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-961ce397dfd9b4b9)

running 1 test
test generation_and_check_need_no_native_tools_and_module_refusal_preserves_bytes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/openapi_accounting.rs (target/debug/deps/openapi_accounting-13b1a0abdf6eaad5)

running 6 tests
test complete_projection_preserves_supported_yaml_through_both_spellings ... ok
test both_import_spellings_write_the_accounted_envelope_for_every_presentation ... ok
test a_refused_import_keeps_existing_output_and_creates_no_new_file ... ok
test legacy_projection_refuses_before_destination_mutation ... ok
test partial_projection_reports_the_durable_gap_and_preserves_destinations ... ok
test unresolved_projection_reports_the_exact_reference_and_preserves_destinations ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/openapi_adversary_pass1.rs (target/debug/deps/openapi_adversary_pass1-260b8c6adc541fec)

running 2 tests
test cli_rejects_duplicate_keys_and_tampered_accounting_before_output ... ok
test cli_rejects_unknown_unit_fields_before_any_projection_output ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/openapi_adversary_pass2.rs (target/debug/deps/openapi_adversary_pass2-2edeb93cde446f25)

running 1 test
test same_path_projection_refusal_preserves_the_unadmitted_input ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/output_containment.rs (target/debug/deps/output_containment-8e5d534f5c43715b)

running 19 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-b206c0da2903f754)

running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/schema_bundle.rs (target/debug/deps/schema_bundle-732645e9c358ab76)

running 8 tests
test missing_dialect_and_incomplete_import_leave_existing_output_alone ... ok
test document_import_refusals_preserve_source_and_existing_output ... ok
test corrupted_import_cannot_be_projected_and_output_cannot_replace_source ... ok
test type_planning_and_output_refusals_leave_no_partial_library ... ok
test types_bundle_is_deterministic_and_never_replaces_its_input ... ok
test import_reload_projection_and_instance_validation_keep_original_data ... ok
test document_root_survives_reload_projection_and_each_type_target ... ok
test native_bundle_targets_require_identity_and_emit_build_metadata ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/target_failure.rs (target/debug/deps/target_failure-87adfa1dc447352a)

running 1 test
test fatal_synthesis_preserves_destinations_and_has_a_typed_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

     Running unittests src/lib.rs (target/debug/deps/schema_contract-b829eb3b3c64c47e)

running 24 tests
test typescript::tests::emitted_array_helper_cannot_be_shadowed_by_root_or_definition ... ok
test typescript::tests::an_external_reference_is_refused_instead_of_becoming_unknown ... ok
test realize::pattern_accounting_tests::schema_pattern_metadata_follows_schema_positions_only ... ok
test realize::normalize::numeric::tests::zero_clamps_are_sign_stable_in_either_operand_order ... ok
test realize::normalize::numeric::tests::rounding_matches_independently_observed_binary64_bits ... ok
test realize::normalize::eval::positional_defense::checked_arity_is_required_for_present_values_and_missing_propagates ... ok
test typescript::tests::an_unsupported_structural_keyword_is_refused_at_its_pointer ... ok
test typescript::tests::normalized_definition_collisions_keep_the_existing_deterministic_refusal ... ok
test typescript::tests::module_reserved_aliases_are_refused ... ok
test typescript::tests::reserved_words_and_primitive_type_aliases_are_refused ... ok
test typescript::tests::root_and_normalized_definitions_cannot_claim_the_same_binding ... ok
test typescript::tests::noncolliding_projection_retains_the_complete_baseline_bytes ... ok
test typescript::tests::projects_the_supported_structural_vocabulary ... ok
test typescript::tests::projection_is_deterministic_across_property_insertion_order ... ok
test typescript::tests::unused_array_name_and_keyword_properties_keep_their_valid_bytes ... ok
test typescript::tests::validation_refinements_do_not_become_a_second_runtime_contract ... ok
test validate::tests::duplicate_schema_identities_are_refused_before_instances ... ok
test validate::tests::an_unprovided_reference_is_refused_offline ... ok
test validate::tests::failures_accumulate_across_instances_and_fields ... ok
test validate::tests::a_valid_instance_selects_its_schema_by_identity ... ok
test realize::binary64_codec_tests::marked_non_numeric_or_missing_nodes_refuse_before_either_emission ... ok
test realize::binary64_codec_tests::recursive_map_reachability_and_mixed_open_record_refusal_are_located ... ok
test realize::binary64_codec_tests::exact_tuples_and_conditional_helper_collisions_keep_their_boundaries ... ok
test realize::binary64_codec_tests::native_tuple_union_order_and_known_open_fields_keep_original_tokens ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.80s

     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-6a05dd9728d36e35)

running 4 tests
test lazy_defaults_and_later_stages_cannot_erase_float_identity ... ok
test optional_aliases_require_complete_policies_and_exact_model_pins ... ok
test nested_nullable_floats_raw_capture_and_exact_siblings_survive_two_stages ... ok
test native_rust_composition_preserves_identity_in_both_number_feature_modes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.91s

     Running tests/binary64_structural.rs (target/debug/deps/binary64_structural-600c5e8462df158f)

running 4 tests
test all_selected_finite_model_types_emit_both_native_libraries ... ok
test excluded_float_selection_preserves_plain_representation_and_reservations ... ok
test complete_non_binary64_output_maps_remain_identical ... ok
test rust_original_token_wire_corpus ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.00s

     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-6ac63202542e5540)

running 2 tests
test compiler_owned_helper_names_and_selected_root_reports_remain_conditional ... ok
test rust_recursive_unions_and_nullable_maps_keep_original_tokens ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.07s

     Running tests/bundle.rs (target/debug/deps/bundle-bd0a6ea277e97d8f)

running 12 tests
test unsupported_semantics_are_accumulated_at_source_pointers ... ok
test malformed_schema_constraints_and_unselected_projection_roots_refuse ... ok
test annotations_are_not_walked_as_schemas_and_reference_siblings_survive ... ok
test recursive_schemas_and_escaped_component_names_keep_their_identity ... ok
test document_identity_is_versioned_and_replayed_while_v1_bytes_remain_unchanged ... ok
test one_of_is_exclusive_and_false_roots_accept_no_value ... ok
test document_import_refuses_collisions_dialect_changes_and_nonlocal_references ... ok
test root_closure_keeps_qualification_without_inventing_service_metadata ... ok
test original_byte_changes_remain_visible_even_when_structural_meaning_is_equal ... ok
test document_root_retains_typed_fields_recursive_references_and_original_locations ... ok
test complete_persisted_accounting_is_rechecked_and_unknown_formats_refuse ... ok
test source_and_projection_agree_under_the_explicitly_selected_dialect ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/normalization.rs (target/debug/deps/normalization-065a7657787f1c88)

running 15 tests
test duplicate_keys_cannot_hide_a_dispatch_branch_or_record_expression ... ok
test additional_properties_cannot_bypass_a_named_output_field_type ... ok
test raw_json_execution_refuses_precision_loss_before_even_an_unused_value_is_discarded ... ok
test raw_json_execution_refuses_duplicate_keys_trailing_data_and_excessive_depth ... ok
test chosen_branches_and_conditions_are_lazy_but_all_are_type_checked ... ok
test explicit_requirements_run_before_transformation_and_null_is_present ... ok
test arithmetic_requires_exact_signed_tokens_and_explicit_overflow_policy ... ok
test distinct_count_uses_selected_categories_and_mapping_retains_order ... ok
test strict_envelope_root_identity_and_every_branch_must_check ... ok
test scalar_equality_has_no_target_dependent_structural_or_numeric_coercion ... ok
test nullable_object_and_list_fallbacks_remain_usable_for_field_access_and_mapping ... ok
test prefix_selection_is_case_sensitive_and_does_not_parse_or_normalize_text ... ok
test raw_json_execution_preserves_exact_values_and_integer_lexical_distinctions ... ok
test fallback_preserves_false_zero_empty_null_and_absence_independently ... ok
test stages_check_each_boundary_and_dispatch_has_no_implicit_default ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/normalization_base64_adversary.rs (target/debug/deps/normalization_base64_adversary-6f349d066413908c)

running 1 test
test base64_values_do_not_qualify_unrelated_model_property_name_patterns ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/normalization_binary64.rs (target/debug/deps/normalization_binary64-ffa6f5cee5944e19)

running 8 tests
test inaccessible_map_and_union_input_policies_refuse_before_generation ... ok
test new_expressions_refuse_old_formats_without_any_binary64_model_in_the_recipe ... ok
test compiler_metadata_and_finite_codecs_cannot_be_minted_by_schema_annotations ... ok
test modeled_numeric_policy_is_required_even_for_unused_optional_fields ... ok
test floating_integral_results_do_not_become_integer_operands ... ok
test floating_construction_requires_the_new_recipe_format ... ok
test modeled_defaults_and_input_tokens_preserve_finite_bits ... ok
test literal_admission_and_numeric_assignment_are_checked_in_lazy_branches ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/normalization_binary64_targets.rs (target/debug/deps/normalization_binary64_targets-42234919cfef530e)

running 1 test
test native_rust_executes_finite_binary64_and_retained_documents ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.89s

     Running tests/normalization_go.rs (target/debug/deps/normalization_go-0c1193d456608749)

running 6 tests
test qualified_patterns_do_not_hide_other_obligations_or_inspect_annotation_data ... ok
test go_target_refuses_referenced_pattern_semantics_before_emitting_files ... ok
test every_primitive_map_key_pattern_is_qualified_at_its_nested_source_pointer ... ok
test go_target_refuses_invalid_library_identity ... ok
test unqualified_syntax_refuses_even_when_the_reference_matches ... ok
test go_target_retains_source_identity_and_accounts_for_every_file ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.45s

     Running tests/normalization_legacy_bytes.rs (target/debug/deps/normalization_legacy_bytes-93c94abd105fff23)

running 1 test
test complete_legacy_file_maps_are_preserved ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/normalization_model.rs (target/debug/deps/normalization_model-89876fb1c8dbaf30)

running 6 tests
test unevaluated_invariants_cannot_become_a_successful_normalizer ... ok
test old_envelopes_and_old_root_reader_refuse_model_identity ... ok
test every_model_identity_coordinate_is_rechecked ... ok
test checked_model_enums_retain_membership_without_flattening_imported_intersections ... ok
test qualified_sources_can_normalize_into_model_owned_records ... ok
test checked_models_preserve_wire_names_units_and_absent_fields ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/normalization_numeric.rs (target/debug/deps/normalization_numeric-1523a0089515596f)

running 5 tests
test finite_range_and_undeclared_precision_loss_refuse_without_partial_results ... ok
test numeric_recipe_round_trip_preserves_authored_decimal_tokens ... ok
test version_one_refuses_even_an_empty_numeric_declaration_and_conversion ... ok
test rounded_conversion_preserves_pipeline_order_and_exact_unselected_integers ... ok
test decoding_paths_and_tokens_are_checked_not_guessed ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/normalization_positional.rs (target/debug/deps/normalization_positional-9f2f68780e8408ae)

running 12 tests
test tuple_reads_preserve_type_and_do_not_join_homogeneous_operations ... ok
test source_tuple_alternatives_are_not_erased_by_member_or_fallback_typing ... ok
test policy_and_position_require_exact_tuples_without_guessing ... ok
test refinements_and_requiredness_run_after_preparation ... ok
test lexical_preparation_composes_with_three_modeled_binary64_stages ... ok
test old_formats_refuse_even_empty_positional_declarations ... ok
test pure_tuple_mapping_does_not_require_original_text ... ok
test positional_declarations_are_closed ... ok
test format_six_inherits_all_existing_binary64_vectors ... ok
test positional_corpus_has_independent_expected_values_and_findings ... ok
test typed_construction_and_old_readers_cannot_bypass_version_or_path_checks ... ok
test paths_keep_exact_conflict_order ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running tests/normalization_positional_adversary.rs (target/debug/deps/normalization_positional_adversary-b574b88c84aa21f8)

running 5 tests
test discarded_source_order_depth_precedes_later_invalid_key ... ok
test source_refinements_and_stage_identity_remain_checked ... ok
test source_union_proofs_survive_computed_collection_and_nested_position ... ok
test literal_cross_boundary_cases_and_repeated_calls_are_atomic ... ok
test native_rust_preserves_independent_token_cases_and_checked_metadata ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.98s

     Running tests/normalization_positional_targets.rs (target/debug/deps/normalization_positional_targets-147d42612f3cd64e)

running 1 test
test native_rust_executes_position_and_binary64_contracts ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.23s

     Running tests/normalization_raw.rs (target/debug/deps/normalization_raw-6319606f1f23aa8f)

running 8 tests
test capture_is_explicit_and_refinements_apply_to_the_encoded_representation ... ok
test capture_envelope_is_closed_and_legacy_readers_refuse ... ok
test explicit_capture_preserves_tokens_before_the_first_schema ... ok
test old_formats_refuse_even_empty_capture_declarations ... ok
test capture_paths_are_qualified_and_conflicts_are_explicit ... ok
test absent_null_and_wrong_intermediates_reach_the_declared_schema ... ok
test root_paths_leaf_types_and_both_overlap_directions_are_checked ... ok
test lexical_corpus_matches_independent_expected_values_and_findings ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s

     Running tests/normalization_raw_adversary.rs (target/debug/deps/normalization_raw_adversary-f99abf591d2d16c4)

running 3 tests
test selectors_use_declared_empty_and_escaped_wire_names_and_exact_conflict_order ... ok
test captured_lexemes_and_mixed_error_order_obey_the_document ... ok
test generated_rust_runs_adversarial_lexical_cases ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.40s

     Running tests/normalization_raw_targets.rs (target/debug/deps/normalization_raw_targets-74006e087b628173)

running 1 test
test native_rust_captures_and_decodes_retained_documents ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.92s

     Running tests/normalization_rust.rs (target/debug/deps/normalization_rust-027e3e842b9e18eb)

running 6 tests
test rust_target_refuses_invalid_or_reserved_package_names ... ok
test standalone_rust_target_executes_declared_binary64_conversion ... ok
test standalone_rust_target_executes_model_owned_wire_records ... ok
test standalone_rust_target_preserves_reference_behavior_and_feature_unification ... ok
test standalone_rust_target_executes_version_two_ordered_operations ... ok
test standalone_rust_target_matches_the_qualified_base64_language ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.33s

     Running tests/normalization_typescript.rs (target/debug/deps/normalization_typescript-b2f16c11a1d388cb)

running 2 tests
test package_identity_is_bounded_and_never_interpreted_as_a_path_or_identifier ... ok
test every_recipe_emits_a_complete_deterministic_accounted_typescript_package ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.55s

     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-17590cbf92a092aa)

running 2 tests
test compiler_owned_integer_map_key_pattern_is_not_implicitly_qualified ... ok
test referenced_property_names_stop_at_plan_admission_and_annotation_decoys_stay_inert ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/normalization_typescript_legacy_bytes.rs (target/debug/deps/normalization_typescript_legacy_bytes-2e67339d70e0dcb2)

running 1 test
test complete_legacy_file_maps_are_preserved ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

     Running tests/normalization_typescript_schema.rs (target/debug/deps/normalization_typescript_schema-0bf2d79a35340010)

running 4 tests
test source_tuple_alternatives_cannot_mint_a_position_binding ... ok
test ref_closures_and_annotation_decoys_preserve_qualified_source_locations ... ok
test unqualified_constraints_refuse_at_every_nested_source_pointer ... ok
test numeric_and_positional_constraints_are_in_the_fixed_profile ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/normalization_v2.rs (target/debug/deps/normalization_v2-e2bcb33e3a857bed)

running 6 tests
test selected_values_must_be_present_even_when_predicate_is_always_false ... ok
test version_one_cannot_smuggle_extended_operations_through_nested_expressions ... ok
test first_match_and_filter_do_not_evaluate_unselected_values ... ok
test ordered_operations_preserve_source_indices_and_exact_construction ... ok
test every_extended_operation_requires_version_two ... ok
test extended_operands_and_scopes_check_before_any_execution ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/realize.rs (target/debug/deps/realize-912d961597822f97)

running 7 tests
test requiredness_nullability_defaults_and_unrestricted_json_are_distinct ... ok
test findings_resolve_in_original_source_and_floating_array_bounds_keep_positions ... ok
test references_enums_consts_and_explicit_types_apply_as_intersections ... ok
test unsupported_shapes_are_accumulated_without_implicit_narrowing ... ok
test tuple_positions_open_objects_and_exclusive_unions_are_accounted ... ok
test roots_select_their_exact_closure_and_keep_qualified_provenance ... ok
test allocation_and_recursion_refuse_only_the_unsupported_boundaries ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/rust_realization.rs (target/debug/deps/rust_realization-f0ccef63efb779d3)

running 4 tests
test unsupported_native_shapes_and_names_refuse_before_emission ... ok
test model_wire_mapping_builds_with_native_roundtrips ... ok
test generated_library_preserves_presence_open_data_unions_and_exact_numbers ... ok
test hostile_source_names_remain_comments_and_wire_strings ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.11s

     Running tests/typescript_bindings.rs (target/debug/deps/typescript_bindings-140511e3b07ddea7)

running 3 tests
test array_helpers_in_alternatives_and_unused_definitions_cannot_be_shadowed ... ok
test escaped_and_non_ascii_definition_keys_refuse_root_collisions_deterministically ... ok
test property_names_and_unemitted_items_do_not_reserve_bindings ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests schema_contract

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Exit: 0.

related-native.log

```console
$ cargo test --offline --locked -p schema-contract --features typescript-typecheck --test normalization_typescript_adversary --test normalization_typescript_native -- --nocapture --test-threads=1
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.47s
     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-305814febf43591c)

running 4 tests
test compiler_owned_integer_map_key_pattern_is_not_implicitly_qualified ... ok
test native_bounds_compare_the_exact_binary_fraction_not_its_shortest_decimal ... qualified engine {"execPath":"/usr/bin/node","node":"v22.23.1","v8":"12.4.254.21-node.56"}; compiler /usr/lib/node_modules/typescript/lib/tsc.js
adversary-exact-bound: executed 8; failed 0; repeated fresh-call checks 8
ok
test native_retained_reentry_and_escaped_nested_position_authority ... adversary retained reentry and nested arity: executed 14; failed 0 (8 public reentries, 2 public nested executions, 1 binding assertion, 3 private corruptions)
ok
test referenced_property_names_stop_at_plan_admission_and_annotation_decoys_stay_inert ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.37s

     Running tests/normalization_typescript_native.rs (target/debug/deps/normalization_typescript_native-d8b8cc34408f07a8)

running 6 tests
test native_equality_preserves_the_frozen_integer_float_limitation ... qualified engine {"execPath":"/usr/bin/node","node":"v22.23.1","v8":"12.4.254.21-node.56"}; compiler /usr/lib/node_modules/typescript/lib/tsc.js
equality-v1: executed 7; failed 0; repeated fresh-call checks 7
equality-v5: executed 7; failed 0; repeated fresh-call checks 7
equality-v6: executed 7; failed 0; repeated fresh-call checks 7
equality-checked-binary64: executed 4; failed 0; repeated fresh-call checks 4
ok
test native_numeric_bits_shortest_text_and_exact_midpoints ... numeric-qualification: executed 26966; failed 0; repeated fresh-call checks 26966
numeric qualification: 13455 independent finite bit patterns, native-shortest text through rounded and exact entry; 28 exact rational midpoint/huge-exponent tokens through both entries
ok
test native_private_validator_conjunctions_match_the_pinned_schema_engine ... private schema conjunctions: executed 7; failed 0
ok
test native_schema_numeric_refinements_and_diagnostic_multiplicity ... schema-controls: executed 31; failed 0; repeated fresh-call checks 31
ok
test native_typescript_executes_every_shared_text_and_retained_document_case ... base64 reference corpus: executed 830 strings through bundle values, model values and model keys, 2490 cases
v1: executed 40; failed 0; repeated fresh-call checks 40
v2: executed 14; failed 0; repeated fresh-call checks 14
numeric: executed 35; failed 0; repeated fresh-call checks 35
model: executed 66; failed 0; repeated fresh-call checks 66
base64: executed 2490; failed 0; repeated fresh-call checks 2490
raw: executed 149; failed 0; repeated fresh-call checks 149
raw-adversary: executed 42; failed 0; repeated fresh-call checks 42
binary64-v5: executed 70; failed 0; repeated fresh-call checks 70
positional: executed 93; failed 0; repeated fresh-call checks 93
mixed: executed 3; failed 0; repeated fresh-call checks 3
binary64-v6: executed 70; failed 0; repeated fresh-call checks 70
positional-adversary: executed 19; failed 0; repeated fresh-call checks 19
TypeScript shared runtime corpus: executed 3091; failed 0; skipped 0 (13 named value-only entrypoints are inapplicable)
ok
test native_unicode_prototypes_lexical_precedence_and_private_arity_defense ... typescript-boundaries: executed 0; failed 0; repeated fresh-call checks 0
TypeScript boundary controls: executed 56; failed 0
ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 45.94s

```

Exit: 0.

clippy-schema.log

```console
$ cargo clippy --offline --locked -p schema-contract --features typescript-typecheck --test normalization_typescript_adversary -- -D warnings
    Checking serde v1.0.229
    Checking thiserror v2.0.20
    Checking ref-cast v1.0.27
    Checking schemars v0.8.22
    Checking serde_yaml v0.9.34+deprecated
    Checking ahash v0.8.12
    Checking fluent-uri v0.4.1
    Checking jsonschema-value v0.52.1
    Checking email_address v0.2.9
    Checking referencing v0.52.1
    Checking ess-primitives v0.19.0 ($WORKTREE/crates/specify/ess-primitives)
    Checking jsonschema v0.52.1
    Checking ess-domain v0.19.0 ($WORKTREE/crates/specify/ess-domain)
    Checking ess-compiler v0.19.0 ($WORKTREE/crates/specify/ess-compiler)
    Checking ess-gen v0.19.0 ($WORKTREE/crates/generate/ess-gen)
    Checking schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
error: long literal lacking separators
   --> crates/generate/schema-contract/tests/normalization_typescript_adversary.rs:101:17
    |
101 |     let bound = 1.0000000000000001e18_f64;
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^ help: consider: `1.000_000_000_000_000_1e18_f64`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#unreadable_literal
    = note: `-D clippy::unreadable-literal` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::unreadable_literal)]`

error: this argument is passed by value, but not consumed in the function body
  --> crates/generate/schema-contract/tests/normalization_typescript_adversary.rs:10:20
   |
10 | fn bundle(schemas: Value) -> Bundle {
   |                    ^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#needless_pass_by_value
   = note: `-D clippy::needless-pass-by-value` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::needless_pass_by_value)]`
help: consider taking a reference instead
   |
10 | fn bundle(schemas: &Value) -> Bundle {
   |                    +

error: this argument is passed by value, but not consumed in the function body
  --> crates/generate/schema-contract/tests/normalization_typescript_adversary.rs:20:61
   |
20 | fn stage(bundle: &Bundle, input: &str, output: &str, value: Value) -> Value {
   |                                                             ^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#needless_pass_by_value
help: consider taking a reference instead
   |
20 | fn stage(bundle: &Bundle, input: &str, output: &str, value: &Value) -> Value {
   |                                                             +

error: casting `f64` to `u64` may truncate the value
   --> crates/generate/schema-contract/tests/normalization_typescript_adversary.rs:102:16
    |
102 |     assert_eq!(bound as u64, 1_000_000_000_000_000_128);
    |                ^^^^^^^^^^^^
    |
    = help: if this is intentional allow the lint with `#[allow(clippy::cast_possible_truncation)]` ...
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#cast_possible_truncation
    = note: `-D clippy::cast-possible-truncation` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::cast_possible_truncation)]`

error: casting `f64` to `u64` may lose the sign of the value
   --> crates/generate/schema-contract/tests/normalization_typescript_adversary.rs:102:16
    |
102 |     assert_eq!(bound as u64, 1_000_000_000_000_000_128);
    |                ^^^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#cast_sign_loss
    = note: `-D clippy::cast-sign-loss` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::cast_sign_loss)]`

error: could not compile `schema-contract` (test "normalization_typescript_adversary") due to 5 previous errors
```

Exit: 101.

clippy-schema-corrected.log

```console
$ cargo clippy --offline --locked -p schema-contract --features typescript-typecheck --test normalization_typescript_adversary -- -D warnings
    Checking schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `dev` profile [unoptimized] target(s) in 0.15s
```

Exit: 0.

clippy-cli.log

```console
$ cargo clippy --offline --locked -p ess-cli --test normalization_typescript_adversary -- -D warnings
    Checking schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Checking ess-cli v0.19.0 ($WORKTREE/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 1.28s
```

Exit: 0.

focused-final.log

```console
$ cargo test --offline --locked -p schema-contract --features typescript-typecheck --test normalization_typescript_adversary -- --nocapture --test-threads=1
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.60s
     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-305814febf43591c)

running 4 tests
test compiler_owned_integer_map_key_pattern_is_not_implicitly_qualified ... ok
test native_bounds_compare_the_exact_binary_fraction_not_its_shortest_decimal ... qualified engine {"execPath":"/usr/bin/node","node":"v22.23.1","v8":"12.4.254.21-node.56"}; compiler /usr/lib/node_modules/typescript/lib/tsc.js
adversary-exact-bound: executed 8; failed 0; repeated fresh-call checks 8
ok
test native_retained_reentry_and_escaped_nested_position_authority ... adversary retained reentry and nested arity: executed 14; failed 0 (8 public reentries, 2 public nested executions, 1 binding assertion, 3 private corruptions)
ok
test referenced_property_names_stop_at_plan_admission_and_annotation_decoys_stay_inert ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.55s

```

Exit: 0.

3. Findings and reachability

Nothing found.

The checked-arity corruption probe deliberately mutates only a private Map in a generated test package. It confirms the defensive refusal and restoration behavior; no public route was found that can remove compiler-owned metadata, and this is not reported as a public exploit. The propertyNames reference candidate never reaches TypeScript generation because Plan refuses it first. The separate compiler-owned integer-map case does reach Plan::typescript and measures the exact prepublication schema refusal. The CLI probe reaches the actual binary with the documented target/package arguments.

No origin classification is assigned to corrected test assumptions. No failing product case was reproduced, so there is no introduced/pre-existing/undecided defect claim.

4. Attacks that did not break

- Exact binary schema bounds retained the complete integer value beyond the safe Number integer range.
- Two explicit text entries preserved raw provenance, empty/null/absence distinctions, and fixed-array preparation only at the entry boundary.
- Nested collection evaluation used full escaped static arity keys and rejected missing, zero, and inconsistent private authority.
- Compiler-owned integer key regexes remained unqualified; annotation values did not acquire schema authority.
- The CLI generated and checked complete packages without a native compiler, and refused --module without changing any reported file.
- The inherited Unicode/prototype, numeric, source-order, schema, helper, positional, and equality cases remained green.
- Every old Rust/Go generated file stayed byte-identical, including producer headers and reports. Release-version qualification remains separate coordinator work.

5. Source identity, tools, and inventory

| New path | Mode | Bytes | SHA256 |
|---|---|---:|---|
| crates/generate/schema-contract/tests/normalization_typescript_adversary.rs | 0644 | 9759 | 25ac6e46f0f1b62eccf405cd25868ff3146bedcb1b5153c8f5c88e3538022741 |
| crates/generate/schema-contract/tests/fixtures/normalization_typescript_adversary.ts.txt | 0644 | 2330 | 1e501c69f844213bdbdcf6c5e38402846a248b7df045a7d574013f3effd912f9 |
| crates/edge/ess-cli/tests/normalization_typescript_adversary.rs | 0644 | 3070 | 73766e90740e35b0f79479a7ce21cf1faf79c6e23ec90b761c1543812017181f |

Tool observations:

```text
rustc 1.98.0 (88d9e12ae 2026-08-18)
cargo 1.98.0 (797e8a9bc 2026-08-05)
rustfmt 1.9.0-stable (88d9e12ae1 2026-08-18)
go version go1.26.5-X:nodwarf5 linux/amd64
$GOCACHE
sccache 0.16.0
      Avail
14672576512
3174475026	target
```

Native package execution recorded Node v22.23.1, V8 12.4.254.21-node.56, /usr/bin/node and TypeScript Version 6.0.3. RUSTC_WRAPPER was /usr/bin/sccache, version 0.16.0; shared cache root $SCCACHE. The last own target size was 3,174,475,026 bytes, under the 6 GiB allowance, and free space was 14,672,576,512 bytes, above the 8 GiB reserve. Shared cache counters are not attributed to this pass.

All commands launched by this adversary are terminal, and the Cargo/native lane was explicitly released before report assembly. Other coordinators had unrelated Cargo processes in different trees; none was changed or stopped. No full workspace/site gate, integration, release, AEP, Git mutation, or cleanup was performed.

The source-integrity.json record retains each exact read-only comparison command, exit 0 and empty output: all frozen SHA256 values, all 1,018 modes, raw legacy directory equality, scoped format, tracked diff and diff whitespace. Complete test-only diff is test-only.patch. command-exits.json retains all fourteen Cargo commands and their output paths. counts.json retains all 56 ordinary summary rows.

Worktree writes are exactly the three new test files above plus compiler and generated-test products under $WORKTREE/target. Tests that use std::env::temp_dir wrote the following assigned scratch files. Preparation contracts, the prepared test plan and the previous raw-settings packet were left unchanged. Every direct outside-worktree file from this pass, including this packet, follows:

```text
$SCRATCH/clippy-cli.log
$SCRATCH/clippy-schema-corrected.log
$SCRATCH/clippy-schema.log
$SCRATCH/command-exits.json
$SCRATCH/counts.json
$SCRATCH/focused-final.log
$SCRATCH/isolated-cli-no-tools.log
$SCRATCH/isolated-exact-bound.log
$SCRATCH/isolated-model-property-names-corrected.log
$SCRATCH/isolated-model-property-names.log
$SCRATCH/isolated-property-names-admission-corrected.log
$SCRATCH/isolated-property-names.log
$SCRATCH/isolated-reentry-position-corrected.log
$SCRATCH/isolated-reentry-position.log
$SCRATCH/new-test-manifest.json
$SCRATCH/packet-manifest.json
$SCRATCH/private-report.md
$SCRATCH/public-report.md
$SCRATCH/related-default-packages.log
$SCRATCH/related-native.log
$SCRATCH/source-integrity.json
$SCRATCH/test-only.patch
$SCRATCH/tmp/ess-count-cli-refusals-3276846/report.json
$SCRATCH/tmp/ess-count-cli-refusals-3276846/suite.json
$SCRATCH/tmp/ess-count-cli-surfaces-3276846/report-json.json
$SCRATCH/tmp/ess-count-cli-surfaces-3276846/report-yaml.json
$SCRATCH/tmp/ess-count-cli-surfaces-3276846/suite.json
$SCRATCH/tmp/ess-delivery-test-3284198-6/helm
$SCRATCH/tmp/ess-delivery-test-3284198-6/oras
$SCRATCH/tmp/ess-go-count-empty-and-clock-3278790/essconform/README.md
$SCRATCH/tmp/ess-go-count-empty-and-clock-3278790/essconform/count_test.go
$SCRATCH/tmp/ess-go-count-empty-and-clock-3278790/essconform/predicate.go
$SCRATCH/tmp/ess-go-count-empty-and-clock-3278790/essconform/runtime.go
$SCRATCH/tmp/ess-go-count-empty-and-clock-3278790/essconform/suite.go
$SCRATCH/tmp/ess-go-count-empty-and-clock-3278790/essconform/suite.json
$SCRATCH/tmp/ess-go-count-empty-and-clock-3278790/go.mod
$SCRATCH/tmp/ess-go-count-empty-and-clock-3278790/report.json
$SCRATCH/tmp/ess-go-count-matrix-3278790/begin-error.json
$SCRATCH/tmp/ess-go-count-matrix-3278790/begin-skip.json
$SCRATCH/tmp/ess-go-count-matrix-3278790/essconform/README.md
$SCRATCH/tmp/ess-go-count-matrix-3278790/essconform/count_test.go
$SCRATCH/tmp/ess-go-count-matrix-3278790/essconform/predicate.go
$SCRATCH/tmp/ess-go-count-matrix-3278790/essconform/runtime.go
$SCRATCH/tmp/ess-go-count-matrix-3278790/essconform/suite.go
$SCRATCH/tmp/ess-go-count-matrix-3278790/essconform/suite.json
$SCRATCH/tmp/ess-go-count-matrix-3278790/failure.json
$SCRATCH/tmp/ess-go-count-matrix-3278790/go.mod
$SCRATCH/tmp/ess-go-count-matrix-3278790/passed.json
$SCRATCH/tmp/ess-go-count-matrix-3278790/skip.json
$SCRATCH/tmp/ess-go-count-matrix-3278790/teardown.json
$SCRATCH/tmp/ess-go-count-refusals-3278790/essconform/README.md
$SCRATCH/tmp/ess-go-count-refusals-3278790/essconform/count_test.go
$SCRATCH/tmp/ess-go-count-refusals-3278790/essconform/predicate.go
$SCRATCH/tmp/ess-go-count-refusals-3278790/essconform/runtime.go
$SCRATCH/tmp/ess-go-count-refusals-3278790/essconform/suite.go
$SCRATCH/tmp/ess-go-count-refusals-3278790/essconform/suite.json
$SCRATCH/tmp/ess-go-count-refusals-3278790/go.mod
$SCRATCH/tmp/ess-go-count-retained-runtime-3278790/essconform/README.md
$SCRATCH/tmp/ess-go-count-retained-runtime-3278790/essconform/count_test.go
$SCRATCH/tmp/ess-go-count-retained-runtime-3278790/essconform/predicate.go
$SCRATCH/tmp/ess-go-count-retained-runtime-3278790/essconform/runtime.go
$SCRATCH/tmp/ess-go-count-retained-runtime-3278790/essconform/suite.go
$SCRATCH/tmp/ess-go-count-retained-runtime-3278790/essconform/suite.json
$SCRATCH/tmp/ess-go-count-retained-runtime-3278790/go.mod
$SCRATCH/tmp/ess-go-count-retained-runtime-3278790/legacy.json
$SCRATCH/tmp/ess-go-count-skip-only-3278790/essconform/README.md
$SCRATCH/tmp/ess-go-count-skip-only-3278790/essconform/predicate.go
$SCRATCH/tmp/ess-go-count-skip-only-3278790/essconform/runtime.go
$SCRATCH/tmp/ess-go-count-skip-only-3278790/essconform/suite.go
$SCRATCH/tmp/ess-go-count-skip-only-3278790/essconform/suite.json
$SCRATCH/tmp/ess-go-count-skip-only-3278790/go.mod
$SCRATCH/tmp/ess-go-count-skip-only-3278790/report.json
$SCRATCH/tmp/ess-go-count-skip-only-3278790/target.go
$SCRATCH/tmp/ess-go-count-skip-only-3278790/target_test.go
$SCRATCH/tmp/ess-go-predicate-leaves-3278790/essconform/README.md
$SCRATCH/tmp/ess-go-predicate-leaves-3278790/essconform/count_test.go
$SCRATCH/tmp/ess-go-predicate-leaves-3278790/essconform/predicate.go
$SCRATCH/tmp/ess-go-predicate-leaves-3278790/essconform/predicate_admission_test.go
$SCRATCH/tmp/ess-go-predicate-leaves-3278790/essconform/predicate_vectors.json
$SCRATCH/tmp/ess-go-predicate-leaves-3278790/essconform/runtime.go
$SCRATCH/tmp/ess-go-predicate-leaves-3278790/essconform/suite.go
$SCRATCH/tmp/ess-go-predicate-leaves-3278790/essconform/suite.json
$SCRATCH/tmp/ess-go-predicate-leaves-3278790/go.mod
$SCRATCH/tmp/node-compile-cache/v22.23.1-x64-9de703df-1000/026b6062
$SCRATCH/tool-versions.log
$SCRATCH/write-inventory.json
```

Tool-managed shared cache roots also used by the required commands:

```text
$SCCACHE
$GOCACHE
```

Per-object writes inside those shared caches cannot be attributed to this pass because other coordinators also use them. The explicit assigned scratch inventory is complete; no per-object cache ownership or cleanup is claimed. write-inventory.json preserves this distinction, plus the absolute worktree source and target roots. packet-manifest.json hashes the retained packet and temporary files, excluding itself to avoid a self-reference.

```findings
[]
```