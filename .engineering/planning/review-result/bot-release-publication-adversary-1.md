---
format: aep.planning-md/3
id: review-result:bot-release-publication-adversary-1
kind: review-result
status: active
title: Release preparation adversary first pass
relations:
- reviews: story:bot-release-publication
revision: 1
---
unit: ESS bot-release publisher preparation at d09cd00ba0ece15bf043798ac622d32c9f94eb3b with frozen author patch e2dc6b7ee80680d7e591643c7b022af089007d0ad001420b419dbc91eb0a54d9
verdict: CONFIRMED
cases: executed 365→366, red 1
origin: introduced 0 / pre-existing 1 / undecided 0
wrote-outside-worktree: 19 paths
needs-coordinator: return the author's bounded two-file correction for a second pass

```text
 .../tests/adversary_release_preparation.rs         | 174 +++++++++++++++++++++
 1 file changed, 174 insertions(+)
```

This stat is the adversary-owned delta against the pre-applied frozen author patch. The handed-over author diff remains 5 files, 180 insertions and 53 deletions; it is not part of the adversary-owned delta. The only adversary worktree write is the new Rust integration test. Its SHA-256 is `45076b556c696f54a6f6262479224c029a4bc09c83cb12434f7ea8a25b9e78c6`.

## 2. Case added before execution

`crates/edge/ess-xtask/tests/adversary_release_preparation.rs:69` adds `release_record_fetches_historical_tag_objects_before_checking_off_main`. It creates a real bare Git remote with `main` and an annotated `9.9.9` tag on an orphan commit. It then reproduces a shallow, tagless runner checkout. The test proves that `git ls-remote --tags origin` advertises the tag while local `git rev-parse --verify 9.9.9^{commit}` cannot peel it, then requires the release-record checkout to fetch full history and tag objects. It is red now at line 164 because `fetch-depth` is absent.

The first compile referred to an unavailable test-only crate and exited 101 before the case ran. That harness typo was replaced with a `std` scratch guard under the assigned temporary directory; it is not a finding. The exact preliminary compiler output is retained in the raw logs.

Command, executed alone after the harness correction:

```console
CARGO_TARGET_DIR=<assigned-build>/target TMPDIR=<assigned-build>/tmp CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo test -p ess-xtask --test adversary_release_preparation --locked -- --exact release_record_fetches_historical_tag_objects_before_checking_off_main --nocapture
```

Exit status: 101. Current red output, with only private path prefixes replaced by stable aliases:

```text
   Compiling ess-xtask v0.52.0 (<review-tree>/crates/edge/ess-xtask)
    Finished `test` profile [unoptimized] target(s) in 6.75s
     Running tests/adversary_release_preparation.rs (<assigned-build>/target/debug/deps/adversary_release_preparation-1fe6a147f3708751)

running 1 test

thread 'release_record_fetches_historical_tag_objects_before_checking_off_main' (3128575) panicked at crates/edge/ess-xtask/tests/adversary_release_preparation.rs:164:5:
assertion `left == right` failed: release status skips any remote tag whose object is absent locally, so the record job must fetch full history
  left: None
 right: Some(0)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test release_record_fetches_historical_tag_objects_before_checking_off_main ... FAILED

failures:

failures:
    release_record_fetches_historical_tag_objects_before_checking_off_main

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

error: test failed, to rerun pass `-p ess-xtask --test adversary_release_preparation`
```

The exact raw output SHA-256 is `e599ce3232b35f45361d9e5145bfb5f8f7bc33674b6004dca0fcc09e839e76b6`.

## 3. Suite run after the case existed

The frozen author suite was run after the adversary case existed, with exactly that finding case deselected to establish the before count and expose any collateral regression:

```console
CARGO_TARGET_DIR=<assigned-build>/target TMPDIR=<assigned-build>/tmp CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo test -p ess-xtask --locked -- --skip release_record_fetches_historical_tag_objects_before_checking_off_main
```

Exit status: 0. It ran 365 passing tests and retained 3 documented ignores; the new adversary binary reports one filtered test. Verbatim output follows with only private path prefixes replaced by stable aliases:

```text
   Compiling ess-xtask v0.52.0 (<review-tree>/crates/edge/ess-xtask)
    Finished `test` profile [unoptimized] target(s) in 14.33s
     Running unittests src/main.rs (<assigned-build>/target/debug/deps/ess_xtask-d02f7e78a20aceea)

running 225 tests
test consumer_coverage::accounting_v2_tests::acquisition_profile_inventory_refuses_a_ninth_scenario_acquisition_row ... ok
test cli_reference::tests::hidden_commands_and_hidden_aliases_are_not_listed ... ok
test cli_reference::tests::help_text_is_escaped_for_the_site_and_the_table ... ok
test cli_reference::tests::every_leaf_is_listed_with_its_invocation_and_arguments ... ok
test cli_reference::tests::a_page_without_exactly_one_marker_pair_is_refused ... ok
test consumer_coverage::accounting_v2_tests::historical_v1_readers_reject_v2_and_every_new_surface ... ok
test consumer_coverage::accounting_v2_tests::binary_unit_artifact_binds_target_kind_root_and_case_source_separately ... ok
test cli_reference::tests::the_committed_page_names_the_commands_and_flags_adopters_asked_for ... ok
test consumer_coverage::accounting_v2_tests::aggregate_reference_deltas_require_both_local_endpoints_in_the_frontier ... ok
test consumer_coverage::accounting_v2_tests::aggregate_proof_binds_the_exact_reviewed_manifest ... ok
test cli_reference::tests::rendering_is_deterministic ... ok
test consumer_coverage::accounting_v2_tests::aggregate_refused_child_requires_the_exact_qualified_refusal_boundary ... ok
test consumer_coverage::accounting_v2_tests::complete_current_subtree_requires_changed_profile_and_every_current_child ... ok
test consumer_coverage::accounting_v2_tests::aggregate_closure_is_bound_to_the_exact_reconciliation_and_frozen_residual ... ok
test cli_reference::tests::prose_outside_the_markers_is_kept_byte_for_byte ... ok
test consumer_coverage::accounting_v2_tests::v2_readers_are_closed_by_format_stage_disposition_and_fields ... ok
test consumer_coverage::accounting_v2_tests::replacement_binding_requires_the_exact_kind_cases_reason_refusal_or_closure ... ok
test consumer_coverage::accounting_v3_tests::legacy_native_receipt_shape_remains_byte_for_byte_unchanged ... ok
test consumer_coverage::accounting_v3_tests::v3_requires_an_exact_disjoint_legacy_and_model_case_partition ... ok
test consumer_coverage::accounting_v2_tests::aggregate_modes_require_complete_nonoverlapping_same_consumer_frontiers ... ok
test consumer_coverage::accounting_v2_tests::reconciliation_consumes_the_complete_stale_set_once_and_preserves_one_unknown ... ok
test cli_reference::tests::check_accepts_a_current_page_and_refuses_one_missing_a_new_flag ... ok
test consumer_coverage::accounting_v3_tests::historical_accounting_readers_reject_v3_without_changing_v1_or_v2 ... ok
test consumer_coverage::accounting_v3_tests::v3_candidate_reader_is_closed_and_requires_model_behavior_evidence ... ok
test consumer_coverage::accounting_v2_tests::reconciliation_refuses_missing_extra_duplicate_wrong_and_revived_decisions ... ok
test consumer_coverage::accounting_v2_tests::acquisition_refuses_a_discarded_default_probe_in_the_actual_authored_body ... ok
test consumer_coverage::accounting_v2_tests::acquisition_v1_signature_only_family_is_refused ... ok
test consumer_coverage::accounting_v2_tests::acquisition_authority_is_exactly_eight_closed_rows_and_planned_equals_proved ... ok
test consumer_coverage::accounting_v3_tests::v3_plan_reader_consumes_model_claims_once_and_keeps_case_sets_disjoint ... ok
test consumer_coverage::model_behavior_tests::authority_rejects_duplicate_keys_before_a_json_value_exists ... ok
test consumer_coverage::model_behavior_tests::authority_numbers_read_the_same_in_every_serde_json_build ... ok
test consumer_coverage::model_behavior_tests::authority_rejects_unknown_members_and_unclaimed_cases ... ok
test consumer_coverage::model_behavior_tests::authority_rejects_inexact_hashes_and_execution_contracts ... ok
test consumer_coverage::model_behavior_tests::candidate_binds_a_wire_model_to_its_current_shape ... ok
test consumer_coverage::model_behavior_tests::candidate_distinguishes_extractor_and_cargo_no_feature_encodings ... ok
test consumer_coverage::model_behavior_tests::candidate_rejects_a_stale_claim_after_a_current_claim ... ok
test consumer_coverage::model_behavior_tests::candidate_rejects_a_stale_model_shape_claim ... ok
test consumer_coverage::model_behavior_tests::candidate_rejects_an_unknown_model_claim ... ok
test consumer_coverage::model_behavior_tests::closed_authority_accepts_one_exact_case_and_claim ... ok
test consumer_coverage::model_behavior_tests::candidate_plan_and_execution_bind_every_digest_and_exact_receipt ... ok
test consumer_coverage::accounting_v2_tests::acquisition_refuses_ninth_missing_stale_wrong_role_source_or_execution ... ok
test consumer_coverage::preservation::tests::baseline_identity_and_authority_list_cannot_be_silently_replaced ... ok
test consumer_coverage::preservation::tests::actual_mapping_preserves_every_current_consumer_and_reviewed_requirement ... ok
test consumer_coverage::preservation::tests::omitted_duplicate_and_unknown_consumers_refuse ... ok
test consumer_coverage::preservation::tests::entrypoints_execution_and_claim_boundaries_cannot_drift ... ok
test consumer_coverage::accounting_v2_tests::pinned_historical_source_reproduces_all_five_old_aggregate_hashes ... ok
test consumer_coverage::tests::adversary_absolute_external_import_keeps_its_external_owner ... ok
test consumer_coverage::preservation::tests::reviewed_requirements_cases_and_behavior_cannot_be_dropped_or_changed ... ok
test consumer_coverage::tests::adversary_absolute_external_type_is_not_shadowed_by_a_local_module ... ok
test consumer_coverage::tests::adversary_new_public_associated_constant_requires_a_concrete_classification ... ok
test consumer_coverage::tests::adversary_changed_associated_output_type_invalidates_its_consumer_contract ... ok
test consumer_coverage::tests::adversary_pass2_expected_panic_and_surplus_results_do_not_qualify_support ... ok
test consumer_coverage::tests::adversary_pass2_new_default_trait_method_needs_its_own_classification ... ok
test consumer_coverage::tests::adversary_pass2_type_macro_cannot_hide_a_selected_associated_contract ... ok
test consumer_coverage::tests::cfg_test_is_excluded_without_hiding_production ... ok
test consumer_coverage::tests::adversary_pass2_wire_literal_refs_and_escaped_definition_names_keep_their_positions ... ok
test consumer_coverage::tests::changed_string_backed_alternative_is_not_hidden_by_wire_string ... ok
test consumer_coverage::tests::comments_and_whitespace_do_not_change_rust_identity_or_shape ... ok
test consumer_coverage::tests::classified_nonmodel_macro_definition_and_invocation_drift_refuse ... ok
test consumer_coverage::tests::consumer_unknown_cfg_is_not_a_default_profile ... ok
test consumer_coverage::tests::concrete_consumer_entries_and_full_case_names_are_not_package_aliases ... ok
test consumer_coverage::tests::correction2_opaque_macros_refuse_every_callable_signature ... ok
test consumer_coverage::tests::correction2_bodies_and_unselected_associated_macros_stay_outside_callable_identity ... ok
test consumer_coverage::tests::consumer_identity_separates_body_comments_and_unrelated_entries_from_signatures ... ok
test consumer_coverage::tests::correction2_header_constraints_track_associated_dependencies_and_refuse_macros ... ok
test consumer_coverage::preservation::tests::changed_source_authorities_require_the_mapping_to_follow ... ok
test consumer_coverage::tests::correction2_trait_contract_refuses_unresolved_qualified_and_wrapped_owners ... ok
test consumer_coverage::tests::correction_absolute_external_groups_renames_and_reexports_preserve_the_owner ... ok
test consumer_coverage::enforce::metadata_tests::metadata_cells_reject_behavior_fields_and_old_closed_disposition_readers_reject_metadata ... ok
test consumer_coverage::tests::correction_associated_contract_tracks_selected_transitive_declarations_only ... ok
test consumer_coverage::tests::correction2_selected_associated_macros_refuse_transitively ... ok
test consumer_coverage::tests::correction_associated_member_profiles_and_unsupported_forms_remain_closed ... ok
test consumer_coverage::tests::correction_qualified_associated_constants_participate_in_declared_array_results ... ok
test consumer_coverage::tests::correction_unresolved_self_contracts_do_not_borrow_an_unrelated_trait_member ... ok
test consumer_coverage::tests::correction_trait_associated_declarations_require_their_own_classifications ... ok
test consumer_coverage::tests::declaration_order_and_unknown_representation_grammar_are_not_erased ... ok
test consumer_coverage::tests::correction_wrapped_self_projection_requires_a_resolved_associated_owner ... ok
test consumer_coverage::tests::finite_unaccepted_eligibility_has_exact_complete_accounting ... ok
test consumer_coverage::tests::generated_consumer_owner_uses_the_leading_invocation_identifier ... ok
test consumer_coverage::tests::ignored_case_and_nested_execution_are_visible_candidates_only ... ok
test consumer_coverage::tests::local_aliases_inline_modules_reexports_and_cycles_are_resolved ... ok
test consumer_coverage::tests::local_declarations_shadow_prelude_leaves_and_generics_refuse ... ok
test consumer_coverage::tests::measured_build_profile_refuses_unsupported_target_features_and_wrappers ... ok
test consumer_coverage::tests::correction2_trait_contract_tracks_selected_associated_types_and_constants ... ok
test consumer_coverage::tests::new_callable_and_target_alternatives_change_exact_inventory ... ok
test consumer_coverage::tests::new_member_or_consumer_cannot_inherit_eligibility ... ok
test consumer_coverage::tests::pending_owner_checkpoint_is_complete_but_never_eligible ... ok
test consumer_coverage::enforce::metadata_tests::accounting_readers_reject_legacy_unknown_envelopes_and_wrong_stages ... ok
test consumer_coverage::tests::production_cfg_and_alias_ambiguity_fail_closed ... ok
test consumer_coverage::tests::one_macro_invoked_twice_in_one_module_keeps_two_identities ... ok
test consumer_coverage::tests::private_optional_tuple_and_enum_members_have_separate_identities ... ok
test consumer_coverage::tests::public_reexport_additions_and_alias_changes_have_concrete_identities ... ok
test consumer_coverage::tests::representation_attributes_invalidate_shape_and_unknown_attributes_refuse ... ok
test consumer_coverage::tests::stage2_case_engine_lists_filters_and_executes_before_qualifying ... ok
test consumer_coverage::tests::stage2_case_engine_refuses_source_drift_or_missing_actual_result ... ok
test consumer_coverage::enforce::metadata_tests::metadata_planning_and_qualified_counts_conserve_every_cell_and_keep_cases_separate ... ok
test consumer_coverage::tests::stage2_classification_refusal_preserves_actual_accounting_diagnostics ... ok
test consumer_coverage::tests::stage2_cargo_artifact_requires_exact_owner_target_profile_and_completed_build ... ok
test consumer_coverage::tests::stage2_exact_finite_baseline_plus_behavioral_case_covers_two_pairs ... ok
test consumer_coverage::tests::stage2_measured_stable_listing_requires_exact_nonignored_case ... ok
test consumer_coverage::enforce::metadata_tests::a_metadata_container_cannot_absorb_a_new_descendant_obligation ... ok
test consumer_coverage::tests::stage2_claims_need_cases_and_named_refusal_without_contradictions ... ok
test consumer_coverage::tests::stage2_new_consumer_and_changed_profile_cannot_inherit_unknown ... ok
test consumer_coverage::tests::stage2_new_obligation_can_gain_behavior_without_expanding_initial_unknowns ... ok
test consumer_coverage::tests::stage2_one_actual_pass_requires_successful_direct_exit ... ok
test consumer_coverage::tests::stage2_refusal_counts_only_current_eligible_unknown_cells ... ok
test consumer_coverage::tests::stage2_removed_duplicate_and_missing_pairs_refuse ... ok
test consumer_coverage::tests::stage2_new_optional_or_string_backed_member_cannot_inherit_unknown ... ok
test consumer_coverage::tests::stage2_swallowed_panic_and_skipped_nested_branches_do_not_qualify ... ok
test consumer_coverage::tests::stage2_zero_selected_ignored_failing_and_forged_results_refuse ... ok
test consumer_coverage::tests::stage2_unknown_requires_independent_owner_and_closed_manifest ... ok
test consumer_coverage::tests::unknown_production_macro_cannot_hide_a_declaration ... ok
test consumer_coverage::tests::stale_duplicate_ownerless_accepted_and_mandatory_unknown_rows_refuse ... ok
test consumer_coverage::tests::unknown_type_owner_is_a_named_refusal ... ok
test consumer_coverage::tests::wire_boolean_schemas_and_ordered_literals_are_concrete ... ok
test consumer_coverage::tests::wire_annotations_are_excluded_only_at_schema_positions ... ok
test consumer_coverage::enforce::metadata_tests::execution_plan_rejects_missing_duplicate_changed_or_premature_guard_claims ... ok
test consumer_coverage::tests::correction2_trait_callable_entries_keep_signature_and_cfg_boundaries ... ok
test consumer_coverage::tests::wire_reference_cycle_is_finite_and_descendants_invalidate_parents ... ok
test consumer_coverage::tests::wire_unknown_keyword_dialect_and_external_references_refuse ... ok
test consumer_coverage::tests::wire_definitions_references_and_escaped_property_names_are_separate ... ok
test diagnostics::tests::a_cell_escapes_what_mdx_and_a_table_would_read ... ok
test docs::tests::a_family_the_version_history_gives_a_release_and_this_lane_does_not_track_is_refused ... ok
test docs::tests::a_family_a_page_names_and_this_lane_does_not_track_is_refused ... ok
test docs::tests::a_format_version_no_release_ships_may_still_be_called_unreleased ... ok
test docs::tests::a_readme_may_name_a_historical_release_outside_its_install_instructions ... ok
test docs::tests::a_readme_that_installs_an_older_release_is_refused ... ok
test consumer_coverage::tests::stage2_current_flags_target_tools_and_complete_configuration_set_are_bound ... ok
test diagnostics::tests::an_empty_summary_is_refused_by_name ... ok
test docs::tests::a_shipped_format_still_called_unreleased_is_refused ... ok
test docs::tests::a_bare_version_belongs_to_the_family_named_before_it_on_the_line ... ok
test diagnostics::tests::the_page_has_a_row_for_every_code_the_story_names ... ok
test docs::tests::a_wave_spelling_and_a_plain_version_both_order_by_minor ... ok
test docs::tests::a_tracked_version_no_reference_page_names_is_reported ... ok
test docs::tests::a_version_run_is_read_whole ... ok
test docs::tests::a_supported_list_is_read_whether_or_not_rustfmt_wrapped_it ... ok
test docs::tests::an_install_walkthrough_pinned_to_an_older_release_is_refused ... ok
test docs::tests::one_family_does_not_match_another_whose_name_ends_in_it ... ok
test docs::tests::the_committed_readme_installs_the_newest_release ... ok
test docs::tests::every_recorded_release_is_a_version_and_every_supported_version_is_recorded ... ok
test docs::tests::every_family_the_committed_version_history_gives_a_release_is_tracked ... ok
test git_checkout::tests::a_git_directory_git_cannot_open_is_no_checkout ... ok
test docs::tests::the_committed_release_notes_do_not_trail_the_newest_release ... ok
test git_checkout::tests::a_gitfile_and_a_git_symlink_mark_a_checkout ... ok
test git_checkout::tests::an_unreadable_git_directory_is_an_error_not_an_absence ... ok
test infra_acceptance::tests::a_cluster_that_already_exists_is_never_taken_over ... ok
test docs::tests::the_newest_dated_heading_is_the_newest_release ... ok
test infra_acceptance::tests::a_program_outside_the_allowlist_is_refused_however_it_is_wrapped ... ok
test infra_acceptance::tests::a_command_naming_a_projection_output_or_a_foreign_cluster_is_refused_before_it_runs ... ok
test git_checkout::tests::each_entry_git_opens_a_repository_by_marks_a_checkout ... ok
test infra_acceptance::tests::a_run_name_outside_the_disposable_prefix_is_refused ... ok
test infra_acceptance::tests::a_scratch_below_a_git_directory_holding_only_an_exclude_file_is_admitted ... ok
test infra_acceptance::tests::a_scratch_below_an_unreadable_git_directory_is_refused ... ok
test infra_acceptance::tests::a_secret_value_is_found_in_plain_and_base64_form_and_nowhere_else ... ok
test infra_acceptance::tests::a_scratch_below_a_repository_a_gitfile_or_a_git_symlink_is_refused ... ok
test infra_acceptance::tests::case_b_every_spelling_that_reaches_a_projection_or_a_foreign_target_is_refused ... ok
test docs::tests::every_phrasing_the_docs_have_used_for_an_unreleased_format_is_read ... ok
test infra_acceptance::tests::the_placement_intent_is_a_valid_infra_spec_naming_the_harness_workload ... ok
test infra_acceptance::tests::every_manifest_renders_without_an_unresolved_placeholder ... ok
test consumer_coverage::metadata::tests::opaque_proof_checks_reject_different_runs_pairs_guard_sources_and_manifests ... ok
test support::tests::a_complete_source_block_is_accepted_without_owning_release_prose ... ok
test docs::tests::every_tracked_format_version_is_named_in_a_reference_page ... ok
test docs::tests::every_family_a_published_page_names_is_tracked ... ok
test consumer_coverage::metadata::tests::profile_declaration_boundary_is_exact_and_does_not_follow_a_manifest_allowlist ... ok
test consumer_coverage::metadata::tests::manifest_cannot_drop_duplicate_add_descendants_or_add_a_fourth_profile ... ok
test consumer_coverage::metadata::tests::manifest_rows_reject_missing_stale_hashes_wrong_roles_and_behavior_fields ... ok
test consumer_coverage::metadata::tests::changed_definition_and_forged_model_dictionary_cannot_reuse_the_container_row ... ok
test support::tests::cargo_version_changes_invalidate_only_the_source_block ... ok
test support::tests::command_inventory_requires_a_complete_nonempty_unique_section ... ok
test support::tests::emitted_version_markers_are_parsed_and_must_agree_across_the_projection ... ok
test support::tests::every_material_row_change_is_refused_with_its_location ... ok
test support::tests::help_inventory_reads_both_clap_layouts_without_swallowing_neighbor_options ... ok
test support::tests::html_requires_the_actual_output_root_and_both_nonempty_local_assets ... ok
test support::tests::missing_duplicated_or_reordered_block_markers_refuse ... ok
test support::tests::support_check_is_an_available_maintenance_command ... ok
test support::tests::adversary_real_source_version_drift_keeps_release_bytes_independent ... ok
test tests::a_named_but_untagged_version_is_not_an_incomplete_release ... ok
test tests::a_minor_release_with_no_change_fragment_is_refused ... ok
test tests::a_version_tag_whose_commit_never_reached_main_is_refused ... ok
test tests::a_tag_that_cannot_be_gated_is_exempt_with_its_reason ... ok
test tests::a_version_tag_with_no_changelog_section_is_refused ... ok
test tests::an_empty_release_is_refused_by_name ... ok
test tests::an_undated_release_is_refused_by_name ... ok
test tests::exclusions_cover_only_the_named_subtree ... ok
test tests::generated_paths_must_stay_below_the_projection_root ... ok
test tests::only_bare_version_tags_are_release_tags ... ok
test tests::published_release_tags_come_from_the_json_report ... ok
test tests::release_notes_stop_before_the_next_release ... ok
test tests::release_preparation_is_evaluated_after_every_dependency_finishes ... ok
test tests::a_version_tag_with_no_release_behind_it_is_refused ... ok
test tests::successful_release_preparation_is_not_treated_as_publication ... ok
test tests::sync_locks_existing_missing_and_nested_roots_before_any_mutation ... ok
test tests::sync_checks_and_reconciles_in_both_directions ... ok
test consumer_coverage::metadata::tests::fresh_provider_and_exact_six_relationships_share_the_actual_wire_inventory ... ok
test tests::sync_preserves_explicit_exclusions_without_excluding_all_hidden_files ... ok
test tests::sync_refuses_destination_alias_into_an_enrolled_tree ... ok
test tests::sync_refuses_enrolled_ancestor_and_reserved_planned_paths ... ok
test tests::the_generated_index_includes_static_site_source ... ok
test tests::the_release_record_is_checked_after_publication_and_failed_preparation ... ok
test tests::workspace_version_comes_only_from_the_workspace_package_table ... ok
test whats_changed::tests::a_summary_over_the_public_limit_is_refused ... ok
test whats_changed::tests::a_version_sorts_by_number_and_not_by_text ... ok
test whats_changed::tests::an_anchor_matches_the_heading_github_would_mint ... ok
test tests::sync_refuses_reserved_state_before_writing_or_pruning ... ok
test whats_changed::tests::the_check_fails_when_the_site_page_is_stale ... ok
test whats_changed::tests::the_site_page_is_written_with_links_to_the_release_posts ... ok
test whats_changed::tests::an_exempt_minor_is_named_with_its_reason ... ok
test whats_changed::tests::every_committed_fragment_is_valid_and_renders ... ok
test infra_acceptance::tests::only_harness_manifests_reach_apply ... ok
test consumer_coverage::metadata::tests::fresh_provider_refuses_retained_drift_dialect_container_and_reference_inventory_changes ... ok
test infra_acceptance::tests::the_scanner_finds_every_known_bypass ... ok
test consumer_coverage::tests::file_level_production_cfg_cannot_hide_from_the_graph ... ok
test consumer_coverage::tests::actual_selected_roots_resolve_without_diagnostic_generated_declarations ... ok
test consumer_coverage::tests::selected_root_reaching_guarded_diagnostic_type_is_not_an_opaque_leaf ... ok
test consumer_coverage::tests::external_module_inside_inline_module_uses_its_semantic_directory ... ok
test consumer_coverage::tests::diagnostic_macro_definition_and_invocation_changes_refuse ... ok
test consumer_coverage::metadata::tests::current_compiled_provider_executes_one_guard_and_binds_its_opaque_proof_to_this_run ... ok
test consumer_coverage::accounting_v3_tests::v3_qualification_consumes_exact_model_and_retained_v2_proofs ... ok
test support::tests::adversary_actual_cli_refusal_is_not_a_successful_support_observation has been running for over 60 seconds
test support::tests::adversary_actual_explicit_site_and_combined_maps_keep_distinct_roots has been running for over 60 seconds
test support::tests::adversary_actual_help_missing_target_metadata_cannot_borrow_neighbor_values has been running for over 60 seconds
test support::tests::adversary_adjacent_readme_is_selected_without_authored_flags has been running for over 60 seconds
test support::tests::adversary_all_real_material_rows_refuse_cell_removal_duplicate_and_order_drift has been running for over 60 seconds
test support::tests::adversary_front_page_override_replaces_adjacent_readme_for_a_specification_file has been running for over 60 seconds
test support::tests::adversary_real_docs_ir_marker_is_nested_json_not_visible_marker_text has been running for over 60 seconds
test support::tests::adversary_actual_help_missing_target_metadata_cannot_borrow_neighbor_values ... ok
test support::tests::adversary_real_docs_ir_marker_is_nested_json_not_visible_marker_text ... ok
test support::tests::adversary_actual_cli_refusal_is_not_a_successful_support_observation ... ok
test support::tests::adversary_adjacent_readme_is_selected_without_authored_flags ... ok
test support::tests::adversary_front_page_override_replaces_adjacent_readme_for_a_specification_file ... ok
test support::tests::adversary_actual_explicit_site_and_combined_maps_keep_distinct_roots ... ok
test support::tests::adversary_all_real_material_rows_refuse_cell_removal_duplicate_and_order_drift ... ok

test result: ok. 225 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 119.93s

     Running tests/adapter_model.rs (<assigned-build>/target/debug/deps/adapter_model-4b38f3ca8cecd70a)

running 21 tests
test the_schema_declares_exactly_the_modelled_fields_and_values ... ok
test the_redeclared_completion_is_the_history_models ... ok
test schema_drift_is_caught ... ok
test a_source_without_its_hand_written_reader_is_caught ... ok
test unknown_fields_admitted_are_caught ... ok
test a_gained_format_value_is_caught ... ok
test the_rust_type_carries_exactly_the_modelled_fields_types_and_values ... ok
test a_gained_completion_value_is_caught ... ok
test a_third_way_to_write_a_source_is_caught ... ok
test every_declaration_this_scan_reads_is_non_empty ... ok
test a_lost_field_is_caught ... ok
test a_gained_field_is_caught ... ok
test a_source_read_by_a_derived_deserialize_is_caught ... ok
test a_dropped_prefix_check_is_caught ... ok
test a_required_source_that_may_be_left_out_is_caught ... ok
test an_omissible_source_that_must_be_written_is_caught ... ok
test a_map_of_the_wrong_value_type_is_caught ... ok
test a_prefix_check_outside_the_reader_is_caught ... ok
test an_untagged_source_is_caught ... ok
test a_source_left_out_of_the_prefix_check_is_caught ... ok
test an_unmodelled_serde_key_is_caught ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/adversary_release_preparation.rs (<assigned-build>/target/debug/deps/adversary_release_preparation-1fe6a147f3708751)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

     Running tests/ci_lanes.rs (<assigned-build>/target/debug/deps/ci_lanes-d664f90432a219b1)

running 25 tests
test release_preparation_has_no_publication_authority_or_command ... ok
test a_queue_run_prebuilds_the_archives_the_tag_publishes_and_the_release_builds_them_otherwise ... ok
test the_archive_jobs_build_the_test_binaries_without_debug_information ... ok
test the_prune_scan_recognises_a_once_per_process_prune ... ok
test release_preparation_retains_an_exact_tag_and_commit_artifact ... ok
test a_release_reuses_a_green_gate_on_the_exact_tagged_commit_and_otherwise_runs_it ... ok
test the_gate_check_aggregates_every_lane_and_cannot_be_skipped ... ok
test every_step_of_task_check_runs_in_some_pull_request_lane ... ok
test every_lockfile_a_lane_builds_is_fetched_before_the_lane_runs ... ok
test consumer_check_runs_in_task_check_only_when_opted_in ... ok
test test_builds_are_cheap_to_compile_link_and_run ... ok
test every_job_that_runs_task_installs_the_pinned_release_by_checksum ... ok
test nextest_and_every_action_are_pinned_by_commit ... ok
test compile_caches_are_saved_only_where_a_later_run_restores_them ... ok
test the_release_builds_intel_macos_on_apple_silicon_and_keeps_its_four_archives ... ok
test the_workspace_shards_are_one_complete_partition ... ok
test a_release_backfill_dispatched_from_main_saves_no_compile_cache ... ok
test the_test_shards_run_the_archives_two_jobs_build_side_by_side_and_compile_nothing ... ok
test every_browser_binary_takes_the_whole_shard_while_it_runs ... ok
test pull_requests_run_feature_off_on_number_semantics_and_every_other_run_runs_all_of_it ... ok
test a_release_publishes_no_pull_request_or_unsuccessful_package_run_of_the_tagged_commit ... ok
test nextest_serialises_exactly_the_binaries_that_coordinate_through_process_state ... ok
test a_freshly_installed_gh_stub_runs_while_other_cases_spawn ... ok
test a_release_publishes_only_a_queue_run_of_the_tagged_commit_that_holds_all_four_archives ... ok
test a_release_reuses_the_gate_of_a_merged_pull_request_only_when_it_tested_the_tagged_tree ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s

     Running tests/d2_constraint_home.rs (<assigned-build>/target/debug/deps/d2_constraint_home-027c262c2294a12d)

running 6 tests
test the_clause_scan_survives_a_line_break_inside_a_clause ... ok
test in_full_is_not_tied_to_one_spelling_of_either_clause ... ok
test the_reference_scan_reads_files_that_are_neither_markdown_nor_under_docs ... ok
test the_scan_reads_no_ignored_build_output_whether_or_not_it_exists ... ok
test no_reference_to_the_deleted_register_omits_that_it_is_gone ... ok
test d2_is_stated_once_in_the_record_and_not_in_a_plan_file ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s

     Running tests/d2_constraint_home_adversary.rs (<assigned-build>/target/debug/deps/d2_constraint_home_adversary-bf66c437c6bf5027)

running 6 tests
test the_home_page_carries_no_completeness_claim_about_copies_of_d2 ... ok
test the_home_page_names_only_linkers_that_ship_the_tests_it_claims ... ok
test the_go_realizations_named_test_exists_and_its_obligations_match_the_plan ... ok
test d2_is_stated_once_under_docs_however_the_second_clause_is_worded ... ok
test docs_pages_link_to_the_home_and_pages_outside_docs_at_least_name_it ... ok
test every_markdown_page_stating_d2_in_full_links_to_its_home ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s

     Running tests/history_model.rs (<assigned-build>/target/debug/deps/history_model-570d4b6259855e85)

running 16 tests
test the_schema_declares_exactly_the_modelled_fields_and_values ... ok
test adversary_a_schema_property_of_the_wrong_type_is_caught ... ok
test the_modelled_spec_digest_is_the_one_the_reader_admits ... ok
test adversary_a_rename_all_inside_cfg_attr_is_caught ... ok
test adversary_a_variant_the_reader_cannot_read_is_caught ... ok
test the_rust_type_carries_exactly_the_modelled_fields_types_and_values ... ok
test a_field_of_the_wrong_type_is_caught ... ok
test an_undeclared_alias_is_caught ... ok
test the_integer_bound_is_the_readers ... ok
test a_container_rename_all_is_caught ... ok
test adversary_a_defaulted_required_field_is_caught ... ok
test adversary_a_catch_all_variant_is_caught ... ok
test a_split_rename_is_caught ... ok
test every_declaration_this_scan_reads_is_non_empty ... ok
test adversary_a_skipped_field_is_caught ... ok
test a_missing_field_and_an_extra_value_are_caught ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/host_paths.rs (<assigned-build>/target/debug/deps/host_paths-73ce4427fc16b41e)

running 12 tests
test a_home_directory_whose_account_name_is_not_ascii_is_still_collected ... ok
test a_marker_that_ends_a_sentence_is_not_a_home_directory_path ... ok
test a_marker_preceded_by_a_path_component_is_not_an_absolute_home_path ... ok
test an_escaped_or_encoded_home_directory_path_is_still_collected ... ok
test the_detector_finds_each_home_directory_spelling_and_no_portable_one ... ok
test the_detector_finds_the_home_directory_this_process_runs_under ... ok
test the_markers_cover_the_superuser_home_this_host_records ... ok
test the_markers_cover_the_home_root_of_every_platform_ci_runs_on ... ok
test the_workflow_parse_reads_every_runner_the_workflow_names ... ok
test the_scan_reads_each_named_tree_and_holds_this_file_to_the_same_rule ... ok
test every_unread_tree_that_carries_the_class_is_named_in_the_module_documentation ... ok
test no_tracked_file_under_the_published_trees_names_a_home_directory_path ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.38s

     Running tests/host_paths_adversary.rs (<assigned-build>/target/debug/deps/host_paths_adversary-02a80b36916f60bd)

running 7 tests
test the_transcribed_detector_is_byte_identical_to_the_lane_s ... ok
test the_markers_cover_the_superuser_home_this_host_records ... ok
test an_escaped_or_encoded_home_directory_path_is_still_refused ... ok
test a_marker_that_ends_a_sentence_is_not_an_absolute_home_directory_path ... ok
test the_selection_keeps_a_tracked_path_that_git_quotes ... ok
test every_tracked_file_the_lane_selects_is_examined_rather_than_silently_dropped ... ok
test normalising_the_separator_invents_no_finding_in_the_tracked_tree ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.53s

     Running tests/host_paths_adversary_2.rs (<assigned-build>/target/debug/deps/host_paths_adversary_2-35ee4ca3c636b689)

running 7 tests
test a_runner_family_the_table_does_not_know_is_still_read_from_the_workflow ... ok
test every_home_root_the_runner_table_names_is_one_the_detector_can_see ... ok
test a_home_directory_whose_account_name_is_not_ascii_is_still_refused ... ok
test at_least_one_login_account_has_a_home_the_markers_do_not_describe ... ok
test one_selected_file_the_working_tree_lacks_does_not_abort_the_scan ... ok
test a_relative_path_named_like_a_marker_is_not_collected ... ok
test the_lane_is_itself_in_the_selection_these_cases_read ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/host_paths_adversary_3.rs (<assigned-build>/target/debug/deps/host_paths_adversary_3-b51ea7dac38288d2)

running 5 tests
test a_runs_on_written_as_a_block_sequence_is_not_silence ... ok
test every_runs_on_line_in_the_repository_workflow_contributes_a_label ... ok
test an_absolute_home_path_on_a_diff_line_is_still_collected ... ok
test a_runs_on_spelling_the_repository_could_write_tomorrow_does_not_invent_a_platform ... ok
test every_marker_in_a_scanned_file_survives_the_narrowing ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.22s

     Running tests/host_paths_adversary_4.rs (<assigned-build>/target/debug/deps/host_paths_adversary_4-fc890bd76b7535ec)

running 5 tests
test a_block_sequence_is_read_however_yaml_allows_it_to_be_written ... ok
test a_matrix_key_another_job_defines_is_not_a_runner_this_workflow_runs_on ... ok
test an_expression_the_parse_cannot_resolve_contributes_no_label_wherever_it_is_written ... ok
test a_runner_group_mapping_is_a_runs_on_value_the_parse_can_see ... ok
test the_repository_control_measures_the_sentence_it_is_written_to_mean ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/host_paths_adversary_5.rs (<assigned-build>/target/debug/deps/host_paths_adversary_5-f892cbd18d2a0c1e)

running 5 tests
test a_second_documented_tree_survives_a_continuation_line_the_parse_does_not_recognise ... ok
test the_documentation_parse_distinguishes_a_tree_called_unread_from_one_called_read ... ok
test the_module_summary_line_is_true_of_the_repository_or_names_the_trees_it_is_true_of ... ok
test the_journal_holds_the_largest_share_of_the_defect_the_bullet_says_it_holds ... ok
test tracked_files_is_git_ls_files_unfiltered_and_partitions_against_the_scanned_selection ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/host_paths_adversary_6.rs (<assigned-build>/target/debug/deps/host_paths_adversary_6-d5dedff264a172b3)

running 3 tests
test a_bullet_between_the_anchors_is_dropped_unless_it_is_spelt_with_an_asterisk ... ignored, story:the-unread-tree-bullet-is-read-whole — UNREAD_TREE_SECTION's doc claims every bullet between the anchors is read; a dash-marked one is dropped without a word
test a_closing_anchor_inside_a_bullet_truncates_the_list_without_a_panic ... ignored, story:the-unread-tree-bullet-is-read-whole — a second closing anchor truncates the list silently and neither guard floor notices
test the_parse_cannot_tell_the_corrected_bullet_from_the_one_the_correction_removed ... ignored, story:the-unread-tree-bullet-is-read-whole — the widened parse stops at the bullet's first colon, so the corrected bullet and the falsified one parse alike

test result: ok. 0 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/internal_names.rs (<assigned-build>/target/debug/deps/internal_names-ee6c69484f433e37)

running 6 tests
test the_detector_does_not_fire_on_a_file_that_names_nobody ... ok
test the_detector_finds_every_spelling_a_name_is_written_in ... ok
test the_separator_run_is_bounded_so_two_sentences_do_not_join_into_a_finding ... ok
test this_lane_is_held_to_its_own_rule ... ok
test the_scan_reads_the_whole_repository_and_not_a_list_of_trees ... ok
test no_tracked_file_names_the_organisation_this_repository_was_written_inside ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s

     Running tests/layout.rs (<assigned-build>/target/debug/deps/layout-d6146172a03056c2)

running 5 tests
test the_path_scan_reads_an_area_qualified_path ... ok
test every_workspace_crate_lives_under_an_area_directory ... ok
test the_path_scan_excludes_by_root_relative_path_and_reads_published_website_source ... ok
test every_literal_path_naming_a_workspace_crate_exists ... ok
test the_path_scan_finds_at_least_one_path_in_this_repository ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running tests/model_enums.rs (<assigned-build>/target/debug/deps/model_enums-2f508281547ce8e4)

running 6 tests
test the_model_lists_every_target_failure_code ... ok
test the_model_lists_every_conformance_check_code ... ok
test the_model_lists_every_admitted_specification_format ... ok
test the_model_lists_every_diagnostic_class ... ok
test the_model_lists_every_diagnostic_family ... ok
test every_source_declaration_this_scan_reads_is_non_empty ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s

     Running tests/site_navigation.rs (<assigned-build>/target/debug/deps/site_navigation-3c2feebf8b05ae8b)

running 8 tests
test every_anchor_the_long_guides_published_still_lands_on_their_old_page ... ok
test the_sidebar_has_the_adopter_categories_in_order_collapsed ... ok
test the_glossary_defines_30_to_60_terms_each_linking_its_page ... ok
test the_slug_matches_what_docusaurus_renders ... ok
test every_page_is_in_the_sidebar_and_every_entry_is_a_page ... ok
test a_split_guide_page_is_at_most_400_lines ... ok
test every_page_has_front_matter_that_parses ... ok
test every_relative_link_lands_on_a_page_and_an_anchor ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s

```

The exact raw suite-output SHA-256 is `46de1eb759cbe468b2140729600c5f2981866dc3a0fb0f37ed259101dfaf3b6e`.

Strict scoped clippy also passed after the case existed:

```console
CARGO_TARGET_DIR=<assigned-build>/target TMPDIR=<assigned-build>/tmp CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 cargo clippy -p ess-xtask --all-targets --locked -- -D warnings
```

```text
    Checking serde_core v1.0.229
    Checking memchr v2.8.3
    Checking itoa v1.0.18
    Checking typenum v1.20.1
    Checking hybrid-array v0.4.15
    Checking serde v1.0.229
    Checking zmij v1.0.23
    Checking equivalent v1.0.2
    Checking hashbrown v0.17.1
    Checking indexmap v2.14.2
    Checking serde_json v1.0.151
    Checking dyn-clone v1.0.20
    Checking ryu v1.0.23
    Checking unsafe-libyaml v0.2.11
    Checking serde_yaml v0.9.34+deprecated
    Checking schemars v0.8.22
    Checking thiserror v2.0.21
    Checking block-buffer v0.12.1
    Checking crypto-common v0.2.2
    Checking const-oid v0.10.2
    Checking digest v0.11.3
    Checking ess-primitives v0.52.0 (<review-tree>/crates/specify/ess-primitives)
    Checking cpufeatures v0.3.1
    Checking cfg-if v1.0.5
    Checking sha2 v0.11.0
    Checking ess-domain v0.52.0 (<review-tree>/crates/specify/ess-domain)
    Checking ess-compiler v0.52.0 (<review-tree>/crates/specify/ess-compiler)
    Checking bitflags v2.13.2
    Checking utf8parse v0.2.2
    Checking anstyle-parse v1.0.0
    Checking pulldown-cmark-escape v0.11.0
    Checking anstyle v1.0.14
    Checking anstyle-query v1.1.5
    Checking colorchoice v1.0.5
    Checking is_terminal_polyfill v1.70.2
    Checking unicase v2.9.0
    Checking unicode-ident v1.0.26
    Checking proc-macro2 v1.0.107
    Checking pulldown-cmark v0.13.4
    Checking anstream v1.0.0
    Checking ess-transport v0.52.0 (<review-tree>/crates/specify/ess-transport)
    Checking clap_lex v1.1.1
    Checking strsim v0.11.1
    Checking clap_builder v4.6.7
    Checking ess-gen v0.52.0 (<review-tree>/crates/generate/ess-gen)
   Compiling ess-xtask v0.52.0 (<review-tree>/crates/edge/ess-xtask)
    Checking quote v1.0.47
    Checking ess-realization v0.52.0 (<review-tree>/crates/specify/ess-realization)
    Checking semver v1.0.28
    Checking linux-raw-sys v0.12.1
    Checking rustix v1.1.4
    Checking ess-deployment v0.52.0 (<review-tree>/crates/generate/ess-deployment)
    Checking syn v2.0.119
    Checking ess-conformance v0.52.0 (<review-tree>/crates/verify/ess-conformance)
    Checking anyhow v1.0.104
    Checking clap v4.6.7
    Checking ess-composition v0.52.0 (<review-tree>/crates/specify/ess-composition)
    Finished `dev` profile [unoptimized] target(s) in 38.04s
```

Exit status: 0. Exact raw clippy-output SHA-256: `115908e5fe60751477359ab0bc3853dcad91dcf490c5bfb62e5873d161cc2628`.

`rustfmt --edition 2021 --check crates/edge/ess-xtask/tests/adversary_release_preparation.rs` passed. The repository-wide formatter check is outside this tests-only pass; its generated projections also differ under the installed formatter, so no green repository-wide fmt claim is made here.

## 4. Finding

| Location | Verdict | Origin | Measured | What reaches it |
|---|---|---|---|---|
| `.github/workflows/release-record.yml:45` | CONFIRMED | pre-existing | The case at `crates/edge/ess-xtask/tests/adversary_release_preparation.rs:164` exited 101: `fetch-depth` was `None`, and the real Git fixture showed a remote-advertised off-main tag could not be resolved locally. `crates/edge/ess-xtask/src/main.rs:442-453` silently continues when that local resolution fails. | `release.published`, failed `workflow_run`, schedule, and manual events enter `jobs.record`; line 63 runs `cargo xtask release status`, whose dispatch calls `tags_off_main` at `src/main.rs:224`. The base workflow at d09cd00 also omitted history and tags, so the defect reproduces at the base. The new published trigger expands the events that encounter the existing blind checker. |

A shallow tagless record job can report success while a historical version tag points to a commit that never reached `main`. The release-record workflow claims to check every pushed version tag, so this is a blocker for that contract.

## 5. Attacked and not broken

- Preparation authority stayed read-only: repository and job permissions are read-only, checkout does not persist credentials, and no release-create, release-edit, or release-upload command remains.
- The retained artifact name binds the resolved tag and commit, and the upload paths retain the archive glob, `SHA256SUMS`, and release notes with missing files refused.
- The preparation step requires exactly the four expected target archives, recomputes all checksums, and checks the Linux x86 binary version before retention.
- Successful preparation is not treated as publication; the record job skips a successful preparation and now observes published-release events.
- Exact tag resolution, exact prior-gate reuse, and the existing fallback gate were inspected and their author tests remained green.

Untested boundaries: no GitHub-hosted workflow ran; no bot publication, release creation, artifact download, GitHub Release asset inventory, live permissions, or remote API response was exercised. The exact four-archive and checksum steps were inspected and covered by existing workflow tests, but were not executed on hosted runner outputs. No credentials were read and no public write occurred.

## 6. Paths written outside the worktree

Publication-safe aliases are used here. The exact absolute path inventory is retained in `<assigned-scratch>/publisher-review/external-paths.raw.txt`.

- `<assigned-scratch>/publisher-review`
- `<assigned-scratch>/publisher-review/tmp`
- `<assigned-scratch>/publisher-review/static-snapshot.raw.txt`
- `<assigned-scratch>/publisher-review/static-review.raw.txt`
- `<assigned-scratch>/publisher-review/red-preflight.raw.txt`
- `<assigned-scratch>/publisher-review/preliminary-compile.raw.txt`
- `<assigned-scratch>/publisher-review/red-behavior-preflight.raw.txt`
- `<assigned-scratch>/publisher-review/red-case-first.raw.txt`
- `<assigned-scratch>/publisher-review/suite-preflight.raw.txt`
- `<assigned-scratch>/publisher-review/suite-with-finding-skipped.raw.txt`
- `<assigned-scratch>/publisher-review/clippy-preflight.raw.txt`
- `<assigned-scratch>/publisher-review/clippy.raw.txt`
- `<assigned-scratch>/publisher-review/red-current-preflight.raw.txt`
- `<assigned-scratch>/publisher-review/red-case.raw.txt`
- `<assigned-scratch>/publisher-review/external-paths.raw.txt`
- `<assigned-scratch>/publisher-review/report.public.pass1.md`
- `<assigned-build>`
- `<assigned-build>/target`
- `<assigned-build>/tmp`

```findings
- file: .github/workflows/release-record.yml
  line: 45
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: pre-existing
  message: the release-record checkout does not fetch historical tag objects, so release status silently omits a remote version tag whose commit is unavailable in the shallow local checkout
```
