---
format: aep.planning-md/1
id: review-result:review-boundaries-7-openapi-adversary-pass-2
kind: review-result
status: active
title: OpenAPI accounting final independent review
relations:
- reviews: story:review-openapi-semantic-accounting
revision: 1
---
unit: story:review-openapi-semantic-accounting, second and final adversary pass, 42513466e27248c008d39efe5a676d64c3283d7a plus the two new test files
verdict: nothing found
cases: executed 165→172, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: no deliberate paths; incidental Cargo cache effects detailed in part 6
needs-coordinator: none
```console
$ git --no-pager diff --stat
$ git --no-pager diff --no-index --stat -- /dev/null <each of the two added test paths>
 .../generate/ess-openapi/tests/adversary_pass2.rs  | 173 +++++++++++++++++++++
 1 file changed, 173 insertions(+)
 .../edge/ess-cli/tests/openapi_adversary_pass2.rs  | 61 ++++++++++++++++++++++
 1 file changed, 61 insertions(+)
```

1. Tests-only diff and frozen scope

The tracked diff above is empty. The following no-index statistics include both otherwise invisible untracked test files, each compared with /dev/null. Complete additions are preserved in final-tests.patch; final-status.txt names exactly these two additions. No production, existing test, manifest, generated, documentation, planning, index or ref was changed. Each new test was written and formatted before its first execution, and pre-execution-tests.sha256 matches final-tests.sha256 byte for byte.

The 165-case baseline comes from the implementor's correction-pass-1/correction-report.md, which reports all 165 passed with none ignored. No pre-addition baseline suite was run. The accepted binding, original implementation diff from 21eac63d347d5d1328712cd59dd9ae5edf41aace, callers and prior reports were inspected. The correction retains the first-pass assertions; its review-file diff contains rustfmt changes plus document(&Value) and borrowed callers, with no deletion, skip or weakening of an assertion. The first review report still hashes to 4ac12af6456773e2698fd4a5b76e97ecd4acb68e87477c5df2957d17ef6cdda3 and was not edited.

2. Added attempts and focused execution

The six library tests are in crates/generate/ess-openapi/tests/adversary_pass2.rs; the CLI test is in crates/edge/ess-cli/tests/openapi_adversary_pass2.rs. Each focused invocation below selected exactly one case and ran before the package suite. All seven were green on their first execution; there was no red execution or compilation/setup failure to omit. Looped assertions are not counted as separate executed cases.

| Case | Assertion | Current result |
|---|---|---|
| null_empty_and_escaped_unknown_unit_fields_refuse | Each integer/number/boolean schema rejects null, empty-object, empty-array and false unknown fields, including empty and escaped field names, in JSON and YAML. | green |
| yaml_aliases_cannot_hide_unknown_unit_fields | Valid YAML anchored unit maps and aliases reload, but adding a null unknown field to the shared unit is refused at the exact expanded pointer. | green |
| schema_like_annotation_literals_do_not_trigger_wire_preflight | Retained annotation literals shaped like invalid unit schemas remain ordinary source literals, with four exact normalization sites and no fabricated semantic gap. | green |
| duplicate_source_keys_refuse_even_when_values_agree | Root, escaped nested and YAML duplicate source keys refuse before import can erase equal values. | green |
| exact_source_identity_requires_a_consistent_new_envelope | A semantically identical source comment changes the byte hash; changed text with old hash refuses, the fully consistent new declaration admits, and uppercase hash spelling refuses. | green |
| null_schema_nodes_cannot_disappear_behind_optional_carriers | Null component, optional property, array item, request schema and response schema each produce refusal at the schema site. | green |
| same_path_projection_refusal_preserves_the_unadmitted_input | Both CLI spellings and all three terminal formats reject a nested null unknown field without changing the input when --ir and --out name the same file; JSON and YAML inputs covered. | green |

The following focused-evidence.md existed before suite execution:

Focused 01

```console
$ cargo test --locked --offline -p ess-openapi --test adversary_pass2 null_empty_and_escaped_unknown_unit_fields_refuse -- --exact --nocapture
stdout:

running 1 test
test null_empty_and_escaped_unknown_unit_fields_refuse ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.01s

stderr:
   Compiling ess-openapi v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/generate/ess-openapi)
    Finished `test` profile [unoptimized] target(s) in 0.21s
     Running tests/adversary_pass2.rs (target/debug/deps/adversary_pass2-07c047cb8bb2a2e3)
exit: 0
```

Focused 02

```console
$ cargo test --locked --offline -p ess-openapi --test adversary_pass2 yaml_aliases_cannot_hide_unknown_unit_fields -- --exact --nocapture
stdout:

running 1 test
test yaml_aliases_cannot_hide_unknown_unit_fields ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

stderr:
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/adversary_pass2.rs (target/debug/deps/adversary_pass2-07c047cb8bb2a2e3)
exit: 0
```

Focused 03

```console
$ cargo test --locked --offline -p ess-openapi --test adversary_pass2 schema_like_annotation_literals_do_not_trigger_wire_preflight -- --exact --nocapture
stdout:

running 1 test
test schema_like_annotation_literals_do_not_trigger_wire_preflight ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

stderr:
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_pass2.rs (target/debug/deps/adversary_pass2-07c047cb8bb2a2e3)
exit: 0
```

Focused 04

```console
$ cargo test --locked --offline -p ess-openapi --test adversary_pass2 duplicate_source_keys_refuse_even_when_values_agree -- --exact --nocapture
stdout:

running 1 test
test duplicate_source_keys_refuse_even_when_values_agree ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

stderr:
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_pass2.rs (target/debug/deps/adversary_pass2-07c047cb8bb2a2e3)
exit: 0
```

Focused 05

```console
$ cargo test --locked --offline -p ess-openapi --test adversary_pass2 exact_source_identity_requires_a_consistent_new_envelope -- --exact --nocapture
stdout:

running 1 test
test exact_source_identity_requires_a_consistent_new_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

stderr:
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_pass2.rs (target/debug/deps/adversary_pass2-07c047cb8bb2a2e3)
exit: 0
```

Focused 06

```console
$ cargo test --locked --offline -p ess-openapi --test adversary_pass2 null_schema_nodes_cannot_disappear_behind_optional_carriers -- --exact --nocapture
stdout:

running 1 test
test null_schema_nodes_cannot_disappear_behind_optional_carriers ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

stderr:
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_pass2.rs (target/debug/deps/adversary_pass2-07c047cb8bb2a2e3)
exit: 0
```

Focused 07

```console
$ cargo test --locked --offline -p ess-cli --test openapi_adversary_pass2 same_path_projection_refusal_preserves_the_unadmitted_input -- --exact --nocapture
stdout:

running 1 test
test same_path_projection_refusal_preserves_the_unadmitted_input ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

stderr:
   Compiling ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.20s
     Running tests/openapi_adversary_pass2.rs (target/debug/deps/openapi_adversary_pass2-966dc5657edb9f75)
exit: 0
```

3. Full package suite and test-source hygiene

```console
$ cargo test --locked --offline --no-fail-fast -p ess-openapi -p ess-cli
stdout:

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 25 tests
test author_empty ... ok
test author_nonmatching_only ... ok
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test author_nested_only ... ok
test web_empty ... ok
test ir_nonmatching_only ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test ir_nested_only ... ok
test run_empty ... ok
test run_nested_only ... ok
test run_nonmatching_only ... ok
test web_nested_only ... ok
test go_nonmatching_only ... ok
test go_nested_only ... ok
test ir_empty ... ok
test go_empty ... ok
test web_nonmatching_only ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s


running 9 tests
test empty_selection_cannot_follow_or_replace_an_output_symlink ... ok
test non_utf8_matching_content_is_refused_before_any_output_write ... ok
test selected_directory_symlink_reports_the_requested_path_and_stays_shallow ... ok
test committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners ... ok
test matching_broken_links_are_read_errors_even_beside_a_valid_source ... ok
test lexical_selection_order_decides_the_first_read_error ... ok
test omitted_scenarios_ignore_a_poisoned_working_directory_default ... ok
test valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links ... ok
test empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s


running 9 tests
test binary_downloads_are_not_silently_decoded ... ok
test an_explicit_missing_front_page_is_an_error ... ok
test asset_symlink_output_is_refused_before_any_page_changes ... ok
test a_page_identity_can_itself_end_in_html ... ok
test publication_without_strict_mode_preserves_unpublished_links ... ok
test strict_links_check_local_and_cross_page_fragments ... ok
test selected_sources_resolve_after_relocation_with_queries_fragments_and_downloads ... ok
test undeclared_existing_and_escaping_targets_are_refused_with_source_lines ... ok
test assets_cannot_collide_with_generated_output_or_escape_it ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s


running 4 tests
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s


running 4 tests
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_rust_cli_writes ... ok
test a_binding_function_capture_is_refused_before_web_cli_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 7 tests
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.70s


running 2 tests
test root_selection_and_output_refusals_preserve_existing_files ... ok
test each_target_retains_the_same_model_selection_and_distinct_input_provenance ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s


running 11 tests
test unsupported_go_pattern_has_no_successful_partial_artifact ... ok
test incompatible_later_destination_preserves_the_entire_existing_output ... ok
test output_symlinks_and_hardlinks_are_refused_without_mutation ... ok
test generated_paths_cannot_replace_canonical_source_inputs_or_follow_links ... ok
test stale_bundle_missing_source_and_unknown_dispatch_refuse ... ok
test explicit_binary64_inputs_run_through_the_cli_without_weakening_version_one ... ok
test checked_recipe_and_run_are_deterministic_with_json_only_stdout ... ok
test output_cannot_replace_any_declared_input ... ok
test generation_checks_refuse_before_creating_or_changing_destinations ... ok
test failed_check_or_execution_preserves_output_and_does_not_emit_a_result ... ok
test generated_libraries_match_the_api_and_drift_check_never_repairs_files ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s


running 6 tests
test complete_projection_preserves_supported_yaml_through_both_spellings ... ok
test both_import_spellings_write_the_accounted_envelope_for_every_presentation ... ok
test a_refused_import_keeps_existing_output_and_creates_no_new_file ... ok
test legacy_projection_refuses_before_destination_mutation ... ok
test unresolved_projection_reports_the_exact_reference_and_preserves_destinations ... ok
test partial_projection_reports_the_durable_gap_and_preserves_destinations ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s


running 2 tests
test cli_rejects_duplicate_keys_and_tampered_accounting_before_output ... ok
test cli_rejects_unknown_unit_fields_before_any_projection_output ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 1 test
test same_path_projection_refusal_preserves_the_unadmitted_input ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s


running 19 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s


running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s


running 8 tests
test missing_dialect_and_incomplete_import_leave_existing_output_alone ... ok
test document_import_refusals_preserve_source_and_existing_output ... ok
test corrupted_import_cannot_be_projected_and_output_cannot_replace_source ... ok
test type_planning_and_output_refusals_leave_no_partial_library ... ok
test types_bundle_is_deterministic_and_never_replaces_its_input ... ok
test import_reload_projection_and_instance_validation_keep_original_data ... ok
test document_root_survives_reload_projection_and_each_type_target ... ok
test native_bundle_targets_require_identity_and_emit_build_metadata ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s


running 1 test
test fatal_synthesis_preserves_destinations_and_has_a_typed_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s


running 17 tests
test tests::review_document_dialect_is_checked_before_schema_interpretation ... ok
test tests::an_external_reference_is_refused_instead_of_fetched ... ok
test tests::review_annotation_names_only_normalize_at_admitted_positions ... ok
test tests::review_schema_annotation_literals_are_not_walked_as_schemas ... ok
test tests::projection_is_byte_deterministic ... ok
test tests::review_empty_enum_refuses_at_every_site ... ok
test tests::unresolved_local_references_are_reported_without_being_guessed ... ok
test tests::review_missing_array_items_refuses_at_every_site ... ok
test tests::review_reference_sibling_constraints_are_accounted_at_every_site ... ok
test tests::review_schema_dialect_override_refuses_at_every_site ... ok
test tests::review_type_array_is_not_an_absent_type ... ok
test tests::review_invalid_optional_string_constraints_refuse ... ok
test tests::review_version_prefix_is_not_a_valid_patch_version ... ok
test tests::supported_ir_survives_projection_and_reimport_semantically ... ok
test tests::review_unsupported_reference_fragments_are_not_dangling_names ... ok
test tests::review_string_enum_and_const_are_both_retained ... ok
test tests::review_nonstring_enum_and_const_are_accounted_at_every_site ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 8 tests
test unresolved_references_keep_distinct_sites_and_escaped_target_identity ... ok
test raw_source_identity_includes_comments_and_line_endings ... ok
test partial_accounting_survives_reload_and_blocks_projection ... ok
test repeated_fields_and_nested_map_keys_refuse_before_erasure ... ok
test checked_envelope_preserves_exact_source_and_legacy_bytes ... ok
test checked_import_rejects_unknown_fields_through_mixed_schema_nesting ... ok
test nested_units_reload_and_legacy_unit_admission_remains_unchanged ... ok
test replay_refuses_tampered_identity_interface_and_accounting ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 11 tests
test duplicate_escaped_map_keys_are_rejected_before_replay ... ok
test reference_escape_identity_is_not_decoded_twice ... ok
test nonunit_schema_variants_reject_unknown_fields ... ok
test accounting_order_duplicates_codes_and_missing_arrays_cannot_be_normalized_away ... ok
test contradictory_string_enum_and_const_survive_both_boundaries ... ok
test unsupported_items_cannot_disappear_at_nested_or_message_sites ... ok
test closed_import_rejects_unknown_boolean_variant_fields ... ok
test closed_import_rejects_unknown_number_variant_fields ... ok
test closed_import_rejects_unknown_integer_variant_fields ... ok
test schema_resource_and_dialect_features_refuse_but_annotation_literals_do_not ... ok
test unconsumed_variant_keywords_remain_gaps_after_replay ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 6 tests
test duplicate_source_keys_refuse_even_when_values_agree ... ok
test null_schema_nodes_cannot_disappear_behind_optional_carriers ... ok
test schema_like_annotation_literals_do_not_trigger_wire_preflight ... ok
test exact_source_identity_requires_a_consistent_new_envelope ... ok
test yaml_aliases_cannot_hide_unknown_unit_fields ... ok
test null_empty_and_escaped_unknown_unit_fields_refuse ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

stderr:
   Compiling ess-openapi v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/generate/ess-openapi)
    Finished `test` profile [unoptimized] target(s) in 0.28s
     Running unittests src/main.rs (target/debug/deps/ess-bdf16dcf99d99666)
     Running tests/authored_scenarios.rs (target/debug/deps/authored_scenarios-2a52915173d4466e)
     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-79337df623536839)
     Running tests/authored_site.rs (target/debug/deps/authored_site-5371c61490a18107)
     Running tests/command_surface.rs (target/debug/deps/command_surface-d726dee0cc47da60)
     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-827d8ff65a3e5bad)
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-4129aeb509bf2158)
     Running tests/go_conformance.rs (target/debug/deps/go_conformance-ba1d9e6377cf10a2)
     Running tests/model_types.rs (target/debug/deps/model_types-7811eadb88011dcb)
     Running tests/normalization.rs (target/debug/deps/normalization-5d5bb3cb824dfb8a)
     Running tests/openapi_accounting.rs (target/debug/deps/openapi_accounting-5ee227b993dfc475)
     Running tests/openapi_adversary_pass1.rs (target/debug/deps/openapi_adversary_pass1-5b967d6bb3cd60e9)
     Running tests/openapi_adversary_pass2.rs (target/debug/deps/openapi_adversary_pass2-966dc5657edb9f75)
     Running tests/output_containment.rs (target/debug/deps/output_containment-a7a441a678c9d27b)
     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-0c666e794f84efd7)
     Running tests/schema_bundle.rs (target/debug/deps/schema_bundle-e4dfccfa3fd0e18d)
     Running tests/target_failure.rs (target/debug/deps/target_failure-5fcd519645fe8065)
     Running unittests src/lib.rs (target/debug/deps/ess_openapi-d6e5f099f8add75c)
     Running tests/accounting.rs (target/debug/deps/accounting-4be464add4945ce7)
     Running tests/adversary_pass1.rs (target/debug/deps/adversary_pass1-a8c97e6079c0b8d7)
     Running tests/adversary_pass2.rs (target/debug/deps/adversary_pass2-46879ae163d68c93)
   Doc-tests ess_openapi
exit: 0
```

Measured from the actual runner summaries: {
  "summaries": 22,
  "passed": 172,
  "failed": 0,
  "ignored": 0,
  "measured": 0,
  "executed": 172,
  "exit": 0
}. The 22 summaries include zero-case runners; passed plus failed is 172 executed cases, exactly seven above the supplied baseline. This includes every first-review regression and both correction controls. The one CLI case additionally records 12 subprocess invocations; these are assertions within that case, not 12 Rust test cases.

```console
$ cargo fmt -p ess-openapi -p ess-cli -- --check
stdout:
stderr:
exit: 0
```

```console
$ cargo clippy --locked --offline -p ess-openapi -p ess-cli --all-targets -- -D warnings
stdout:
stderr:
    Checking ess-openapi v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/generate/ess-openapi)
    Checking ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 0.18s
exit: 0
```

4. Findings and reachability

Nothing found in this bounded second review. No judgement-only findings are returned. The public library tests use import/read_import/project_import on serialized input. The CLI test invokes the real --ir file reader and --out writer through both documented spellings; it does not construct private admitted values. Existing first-review unknown-field regressions are now green in the complete suite. This report does not approve the unit or establish whole-workspace, site-build or publication readiness.

5. Attacked and not broken

- Raw unit-schema closure survives empty/null values, escaped field names and YAML alias expansion.
- Exact retained-source replay rejects inconsistent identity while admitting a completely consistent new source declaration.
- Schema-like annotation values are not confused with persisted interface nodes.
- Equal duplicate source keys and null unsupported schemas refuse before a successful import can discard them.
- The checked CLI refuses before modifying a destination even when it aliases the input by the same pathname.
- All retained first-review, compatibility, variant-accounting and projection cases remain green.

6. Paths, environment and process closure

No deliberate file was written outside the assigned worktree. Offline Cargo can update existing bookkeeping at /home/timo/.cargo/.global-cache and reuse /home/timo/.cargo/.package-cache and dependency caches; individual incidental entries were not traced. No dependency or compiler cache was pruned.

The assigned shared sccache socket was checked by a local connection and returned ECONNREFUSED before any Cargo execution. Per the coordinator's explicit environment correction, RUSTC_WRAPPER and SCCACHE_SERVER_UDS were unset, CARGO_TARGET_DIR was unset, and all compilation used this unit's existing target. No replacement daemon was started and this pass deliberately used no shared sccache content. The failed socket probe is retained verbatim:

```text
socket unavailable: [Errno 111] Connection refused; direct rustc in own target per coordinator
```

Exact selected environment (unlisted inherited variables were not copied into evidence):

```json
{
  "TMPDIR": "/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-2/tmp",
  "CARGO_INCREMENTAL": "0",
  "CARGO_PROFILE_DEV_DEBUG": "0",
  "CARGO_PROFILE_TEST_DEBUG": "0",
  "CARGO_CACHE_RUSTC_INFO": "0",
  "CARGO_BUILD_JOBS": "4",
  "CARGO_NET_OFFLINE": "true",
  "GOCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-2/go-cache",
  "GOMODCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-2/go-mod-cache",
  "GOTMPDIR": "/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-2/tmp",
  "GOPROXY": "off",
  "GOSUMDB": "off"
}
```

All deliberate new test sources are the two files listed above. Assigned pass evidence and fixtures live under /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-2: brief.md, environment.json, sccache-probe.txt, pre-execution-tests.sha256, final-tests.sha256, focused-01 through focused-07 argv/stdout/stderr/status, focused-evidence.md, suite-01 argv/stdout/stderr/status/counts, fmt-final and clippy-final argv/stdout/stderr/status, final-tracked-diff.stat, final-untracked-test-diff.stat, final-tests.patch, final-status.txt, final-disk.txt, preserved-first-report.sha256, this report, tmp, go-cache, go-mod-cache and same-path-* CLI per-call input/argv/stdout/stderr/status files. Existing suite tests also create ordinary worktree-local target fixtures, including first-review CLI scratch; they did not rewrite the immutable first-review report.

The focused recording process, full suite session 39603 and formatting/Clippy process all completed and their child Cargo statuses are retained. There are no pending agent-owned subprocesses or daemon lifecycles. The final disk check remained above the 8 GiB floor:

```text
Filesystem        1B-blocks         Used   Available Use% Mounted on
/dev/nvme0n1p2 910126964736 829099675648 34719936512  96% /
```

This is the second and final full attack for this unit. All writes are relinquished to the coordinator after this report.

```findings
[]
```
