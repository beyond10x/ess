---
format: aep.planning-md/1
id: review-result:review-boundaries-2-containment-adversary-pass-2
kind: review-result
status: active
title: Output containment second test review
relations:
- reviews: story:review-output-containment
revision: 1
---
unit: story:review-output-containment at dc122aea038cc18757c3a160b1b36b6798ef6df0 plus the test-only additions below
verdict: nothing found
cases: executed 230→235, red 1 fixture setup corrected, 0 remaining
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: /home/timo/.cache/ess-w2-ctmp (temporary fixtures; removed)
needs-coordinator: record dispatch/resource adaptations, integrate and run the full gate
```text
$ git --no-pager diff --stat
 crates/edge/ess-cli/tests/output_containment.rs | 306 ++++++++++++++++++++++++
 1 file changed, 306 insertions(+)
```

1. Review bound and resumption

Only crates/edge/ess-cli/tests/output_containment.rs changed, with 306 inserted lines. No production file changed. Every pre-existing test and assertion remains intact. The five new cases were written by the delegated reviewer before its first attempted executions. Two delegated turns ended with the harness message “This content was flagged for possible cybersecurity risk.” The restricted retry had left five local synthetic-fixture cases and five startup-failure logs, but no completed report or executed test case. Those failed dispatches are not additional completed review passes.

The coordinator resumed this same second pass locally, reading the complete unit/correction diffs, acceptance, new tests and real CLI callers. This is an explicit adaptation from delegated review to coordinator completion; neither human review nor independent-agent approval is claimed. The coordinator had not edited this unit's production implementation. No third full review was started.

The initial attempts could not run rustc because sccache's startup socket exceeded SUN_LEN. Cargo cached that failed rustc -vV result in target/.rustc_info.json, then replayed it despite changed startup conditions; a socket-only strace observed no bind/connect during the replay. The failed metadata is preserved as failed-rustc-info.json and only that disposable compiler-info file was removed. A forced uncached query allowed the first actual test execution. All startup logs and the chronological diagnosis are retained in coordinator-pass-2-resumption.md; startup failures are not test findings.

A separate task-owned sccache server uses target/sccache.sock and the existing shared disk cache. It does not share this unit's target directory with another tree. The first actual case temporarily used the explicitly assigned shorter /home/timo/.cache/ess-w2-ctmp root and CARGO_CACHE_RUSTC_INFO=0. After deleting only the failed metadata, the other cases and full suites used the original assigned scratch TMPDIR. No network-backed test, real user file, credentials, race or permission change was used.

2. New cases and first actual isolated executions

The before count230 is from the completed correction: CLI41 and ess-gen189. No package suite was run before these new cases existed. Each case below was selected alone before the full suites. The first four actual executions passed. The fifth initially failed while compiling its test fixture because a required platform was omitted; that is an invalid fixture, not a product finding. Adding the existing compiler-required Linux/amd64 platform preserved every assertion. Its next isolated execution passed and exercised both real projection workflows.

1. crates/edge/ess-cli/tests/output_containment.rs:578 — Disjoint companion files nested within generated directories retain exact reference bytes and repeat identically.

```sh
env CARGO_CACHE_RUSTC_INFO=0 SCCACHE_SERVER_UDS="$PWD/target/sccache.sock" TMPDIR=/home/timo/.cache/ess-w2-ctmp RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-cli --test output_containment composition_preserves_disjoint_files_inside_generated_directories -- --exact
```

```text
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.40s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 1 test
test composition_preserves_disjoint_files_inside_generated_directories ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.03s


exit: 0
```

2. crates/edge/ess-cli/tests/output_containment.rs:622 — Unix non-UTF8 and native backslash/colon filenames remain distinct caller-selected files.

```sh
env SCCACHE_SERVER_UDS="$PWD/target/sccache.sock" TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-cli --test output_containment composition_keeps_native_non_utf8_and_backslash_filenames_distinct -- --exact
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.08s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 1 test
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.01s


exit: 0
```

3. crates/edge/ess-cli/tests/output_containment.rs:652 — Both companion flags refuse a synthetic encountered parent link before .. can hide it, preserving the whole synthetic fixture.

```sh
env SCCACHE_SERVER_UDS="$PWD/target/sccache.sock" TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-cli --test output_containment composition_refuses_cancelled_parent_links_before_disjoint_companions_change -- --exact
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 1 test
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.02s


exit: 0
```

4. crates/edge/ess-cli/tests/output_containment.rs:777 — Four real local generation sinks first pass valid/repeat controls, then refuse late directory and case-alias conflicts without changing any earlier file.

```sh
env SCCACHE_SERVER_UDS="$PWD/target/sccache.sock" TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-cli --test output_containment local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes -- --exact
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 1 test
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.39s


exit: 0
```

5. crates/edge/ess-cli/tests/output_containment.rs:789 — Local BuildKit and Kubernetes projections first pass valid/repeat controls, then refuse late directory and case-alias conflicts before any output changes.

```sh
env SCCACHE_SERVER_UDS="$PWD/target/sccache.sock" TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-cli --test output_containment local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes -- --exact
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 1 test
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... FAILED

failures:

---- local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes stdout ----

thread 'local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes' (493534) panicked at crates/edge/ess-cli/tests/output_containment.rs:824:5:
local build compilation: Output { status: ExitStatus(unix_wait_status(256)), stdout: "", stderr: "lowering was refused:\n[invalid_value:Build] a build must declare at least one target platform\n" }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-cli --test output_containment`

exit: 101
```

The missing-platform fixture was corrected before the suite. No production code or assertion was changed. Corrected isolated command is the same, with this output:

```text
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.25s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 1 test
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.12s


exit: 0
```

3. Full package suites and required checks

The full suites ran after all five isolated cases and the valid fixture correction. CLI executed46 and ess-gen189, combined235: all passed, no failures or ignored cases. This is 230→235, exactly five selected new cases. Test-only formatting used rustfmt --edition 2021 on the assigned integration-test file; the required package formatter check passed afterward. No workspace formatter was invoked.

```sh
env SCCACHE_SERVER_UDS="$PWD/target/sccache.sock" TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-cli
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/command_surface.rs (target/debug/deps/command_surface-f896f6f697ed70aa)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-14ea054bad6c3502)

running 4 tests
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-ba60d23811c1c6c2)

running 7 tests
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 19 tests
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s


exit: 0
```

```sh
env SCCACHE_SERVER_UDS="$PWD/target/sccache.sock" TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-gen
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.04s
     Running unittests src/lib.rs (target/debug/deps/ess_gen-5cfeec7d828080d8)

running 55 tests
test artifact::tests::portable_artifacts_refuse_escape_and_platform_aliases ... ok
test artifact::tests::a_destination_set_rejects_duplicates_case_aliases_and_file_parents_in_any_order ... ok
test authored::tests::a_heading_becomes_a_section_with_an_anchor ... ok
test authored::tests::a_list_becomes_items_and_a_quote_becomes_a_quote ... ok
test authored::tests::a_link_an_adopter_wrote_stays_theirs ... ok
test authored::tests::a_paragraph_keeps_its_inline_structure ... ok
test authored::tests::a_leading_title_becomes_the_page_title_and_not_a_second_heading ... ok
test authored::tests::a_fence_keeps_its_language_and_loses_its_trailing_newline ... ok
test authored::tests::a_table_keeps_its_header_apart_from_its_rows ... ok
test authored::tests::a_top_level_heading_is_demoted_because_the_page_title_is_the_first ... ok
test authored::tests::raw_html_is_dropped_rather_than_passed_through ... ok
test docs::tests::a_gap_that_ships_says_which_crate_closes_it ... ok
test docs::tests::a_heading_and_its_anchor_agree ... ok
test docs::tests::a_lifecycle_renders_as_a_state_diagram_with_its_initial_and_terminal_states_marked ... ok
test docs::tests::a_lifecycle_that_connects_every_pair_says_it_forbids_nothing ... ok
test docs::tests::a_list_of_three_reads_as_a_person_would_write_it ... ok
test docs::tests::a_lifecycle_with_one_state_forbids_nothing_rather_than_forbidding_everything ... ok
test docs::tests::a_plural_of_entity_is_entities ... ok
test docs::tests::a_state_no_transition_touches_is_still_drawn ... ok
test docs::tests::a_transition_from_two_states_draws_one_arrow_from_each ... ok
test document::tests::a_page_id_says_how_deep_it_is_so_a_renderer_can_reach_the_root ... ok
test docs::tests::the_page_names_every_transition_the_specification_does_not_permit ... ok
test graph::tests::a_dot_label_keeps_its_parts_on_separate_lines ... ok
test document::tests::a_link_names_what_it_points_at_and_never_a_path ... ok
test graph::tests::a_mermaid_label_cannot_close_the_quoted_string_it_sits_in ... ok
test html::tests::a_code_block_is_a_code_listing_and_carries_its_language ... ok
test graph::tests::a_component_group_is_a_dot_cluster_and_graphviz_only_boxes_clusters ... ok
test html::tests::a_diagram_is_a_pre_the_renderer_draws_into_and_never_a_code_listing ... ok
test document::tests::a_document_round_trips_through_its_own_format ... ok
test html::tests::a_construct_is_addressed_by_the_section_that_documents_it ... ok
test html::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test html::tests::a_table_is_a_table_with_a_head_and_a_body ... ok
test html::tests::a_page_reaches_its_stylesheet_and_its_renderer_from_wherever_it_sits ... ok
test html::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test html::tests::the_sidebar_groups_the_nested_pages_and_marks_the_page_the_reader_is_on ... ok
test markdown::tests::a_diagram_is_a_fenced_mermaid_block ... ok
test html::tests::markup_in_text_never_reaches_the_browser_as_markup ... ok
test html::tests::an_adopters_front_page_goes_above_the_index_and_nowhere_else ... ok
test markdown::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test markdown::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test markdown::tests::a_section_flattens_into_the_stream_and_its_children_follow_it ... ok
test markdown::tests::a_table_is_written_with_the_separator_a_reader_expects ... ok
test markdown::tests::a_quotation_marks_every_line_it_covers ... ok
test schema::types::tests::a_decimal_is_written_as_an_exact_string_because_a_json_number_is_read_as_a_float ... ok
test schema::types::tests::a_reference_is_a_pointer_into_the_defs_of_the_document_holding_it ... ok
test schema::types::tests::a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test schema::types::tests::a_string_keyed_map_publishes_no_property_name_rule_that_checks_nothing ... ok
test schema::types::tests::a_union_tagged_value_moves_its_payload_aside_rather_than_colliding_with_the_tag ... ok
test schema::types::tests::an_integer_key_is_constrained_to_the_text_an_integer_is_spelt_with ... ok
test schema::types::tests::a_union_branch_pins_its_tag_so_exactly_one_branch_can_match ... ok
test schema::types::tests::an_optional_outside_a_field_gains_a_null_branch_because_a_list_element_cannot_be_absent ... ok
test html::tests::the_default_style_is_the_stylesheet_that_is_published ... ok
test html::tests::every_emitted_file_says_what_it_was_generated_from ... ok
test html::tests::checked_rendering_validates_deserialized_page_identities_before_map_collection ... ok
test html::tests::checked_rendering_preserves_valid_parent_and_nested_page_bytes ... ok

test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/agreement.rs (target/debug/deps/agreement-a6d7a7ff380699da)

running 4 tests
test every_projection_publishes_the_same_schema_for_a_construct_more_than_one_of_them_describes ... ok
test the_agreement_check_compares_the_constructs_the_defect_was_about_rather_than_nothing ... ok
test every_keyword_the_projections_publish_is_classified_as_an_assertion_or_an_annotation ... ok
test no_projection_collapses_a_newtype_into_the_representation_it_wraps ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/asyncapi.rs (target/debug/deps/asyncapi-9e439fb4245702f8)

running 18 tests
test a_binding_no_component_handles_still_states_its_failure_policy ... ok
test a_collection_says_what_it_holds_and_an_absent_element_is_null_because_it_has_no_key_to_omit ... ok
test a_union_pins_its_tag_so_exactly_one_branch_matches_rather_than_none_or_both ... ok
test a_payload_field_carries_the_grammar_the_model_states_and_not_a_note_naming_it ... ok
test a_payload_refuses_an_undeclared_field_and_spells_absence_by_leaving_it_out_of_required ... ok
test a_dropped_failure_is_stated_in_prose_and_not_only_in_an_extension ... ok
test a_bindings_mapping_and_the_reason_for_its_type_crossing_reach_the_document ... ok
test the_publisher_of_an_event_sees_who_reacts_to_it_and_under_what_failure_policy ... ok
test the_channel_and_its_message_say_nothing_about_the_binding ... ok
test every_document_carries_the_provenance_of_the_model_it_came_from ... ok
test an_events_channel_address_is_its_declared_wire_name_or_else_its_qualified_name ... ok
test every_event_in_the_billing_example_appears_in_some_document ... ok
test a_bindings_delivery_and_failure_reach_the_receiving_operation ... ok
test a_document_shows_what_the_component_publishes_and_what_it_reacts_to ... ok
test a_document_is_a_valid_asyncapi_three_skeleton ... ok
test every_ref_resolves_inside_the_document_that_holds_it ... ok
test every_component_gets_one_document_named_after_it ... ok
test regenerating_from_the_same_model_produces_the_same_bytes ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/corpus.rs (target/debug/deps/corpus-93718f8b1fc63993)

running 3 tests
test the_gatepass_documentation_is_byte_for_byte_what_is_pinned ... ok
test the_oracle_fixture_documentation_is_byte_for_byte_what_is_pinned ... ok
test the_billing_documentation_is_byte_for_byte_what_is_pinned ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/determinism.rs (target/debug/deps/determinism-9942695ed2e87dec)

running 2 tests
test the_determinism_scan_sees_code_and_not_prose ... ok
test no_generator_reads_a_clock_or_an_unordered_map ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/docs.rs (target/debug/deps/docs-530499ed1f64a736)

running 32 tests
test a_type_nothing_references_is_flagged_rather_than_left_looking_used ... ok
test a_grant_that_crosses_two_contexts_links_to_the_other_contexts_page ... ok
test a_bindings_delivery_and_failure_semantics_are_stated_in_words ... ok
test a_declared_conversion_carries_its_reason_everywhere_a_reader_might_start ... ok
test a_commands_refusal_branch_is_documented_and_not_only_its_name ... ok
test an_entitys_identity_reaches_the_page_by_name_and_not_only_by_type ... ok
test a_binding_renders_as_a_flow_and_a_lifecycle_as_a_state_diagram ... ok
test checked_site_rejects_deserialized_collisions_with_late_static_assets ... ok
test an_entitys_absent_transition_is_named_as_a_move_the_specification_does_not_permit ... ok
test an_empty_gap_allowlist_puts_no_cannot_show_section_on_any_page ... ok
test an_entitys_invariant_reaches_the_page_as_a_condition_on_every_instance ... ok
test an_actors_grant_renders_as_an_edge_from_the_actor_to_that_command_in_the_index_graph ... ok
test a_wrong_state_branch_is_documented_with_the_states_the_document_never_lists ... ok
test a_views_filter_reaches_the_page_rather_than_being_silently_dropped ... ok
test checked_site_preserves_valid_deserialized_nested_pages_and_every_artifact_byte ... ok
test a_type_reached_only_through_an_entitys_field_is_not_called_unreached ... ok
test an_actor_that_may_invoke_nothing_is_still_on_the_page ... ok
test an_entitys_lifecycle_transitions_reach_the_page_as_arrows ... ok
test a_views_eventual_consistency_reads_differently_from_an_immediate_one ... ok
test an_events_payload_and_an_errors_payload_are_both_documented_field_by_field ... ok
test an_outcome_that_changes_an_entity_says_which_instance_and_where_the_identity_is_read ... ok
test a_components_ownership_and_a_workloads_replica_floor_are_both_documented ... ok
test an_outcome_the_input_cannot_decide_says_so_rather_than_claiming_it_is_unreachable ... ok
test the_provenance_header_is_a_markdown_comment_a_renderer_can_close ... ok
test an_outcome_says_what_it_does_to_an_entity_and_a_refusal_says_it_changes_none ... ok
test the_command_that_takes_each_move_reaches_the_page_beside_the_move_itself ... ok
test every_page_says_which_specification_produced_it ... ok
test every_link_between_pages_lands_on_a_page_that_exists_at_the_heading_it_names ... ok
test every_member_of_a_resolved_domain_reaches_the_page_of_the_context_it_belongs_to ... ok
test generating_the_documentation_twice_produces_byte_identical_output ... ok
test every_type_kind_reaches_a_page_including_the_tagged_union ... ok
test every_name_the_ir_holds_appears_on_some_page ... ok

test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/openapi.rs (target/debug/deps/openapi-cbc5ba4392fca057)

running 35 tests
test a_component_that_accepts_nothing_still_gets_a_document ... ok
test a_command_with_no_input_is_exposed_without_a_body ... ok
test a_command_no_component_accepts_appears_in_no_document ... ok
test a_served_view_declares_its_rows_and_the_consistency_a_caller_gets ... ok
test a_map_with_a_non_string_key_says_the_key_is_still_a_string ... ok
test a_view_is_served_only_where_the_specification_says_something_outside_reads_it ... ok
test every_kind_of_type_the_model_has_projects_into_a_schema ... ok
test a_command_a_binding_delivers_at_least_once_requires_an_idempotency_key ... ok
test a_refusal_the_subjects_state_decides_is_a_conflict_and_not_a_bad_request ... ok
test a_command_with_no_wire_name_is_exposed_under_the_name_the_model_gives_it ... ok
test a_command_names_the_actors_permitted_to_invoke_it_and_no_authentication_mechanism ... ok
test every_component_gets_one_document_named_after_it ... ok
test a_command_no_binding_invokes_carries_no_idempotency_header ... ok
test a_commands_input_becomes_a_closed_object_over_its_declared_fields ... ok
test a_command_no_actor_names_carries_no_grant_rather_than_a_grant_to_everybody ... ok
test a_document_is_valid_yaml_with_a_version_an_info_block_and_paths ... ok
test a_refusal_the_input_decides_carries_the_declared_error_payload ... ok
test a_command_is_exposed_at_its_wire_name_under_its_domains ... ok
test a_newtype_stays_a_schema_of_its_own_rather_than_becoming_its_representation ... ok
test an_external_outcome_is_an_upstream_failure_and_not_a_validation_refusal ... ok
test a_command_is_only_ever_a_post ... ok
test a_decimal_is_a_string_because_a_json_number_is_a_float ... ok
test several_outcomes_on_one_status_stay_distinguishable ... ok
test an_outcome_that_emits_says_so_without_claiming_to_return_the_events ... ok
test two_commands_claiming_one_path_both_move_to_their_qualified_names ... ok
test each_declared_outcome_is_its_own_response_and_no_status_is_invented ... ok
test every_document_carries_its_provenance_as_a_comment_and_as_data ... ok
test every_document_this_generator_can_produce_is_a_valid_openapi_document ... ok
test every_schema_the_document_declares_is_pointed_at_by_something ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test the_operation_id_is_the_commands_qualified_name ... ok
test regenerating_from_the_same_ir_produces_the_same_bytes ... ok
test the_entities_published_are_exactly_those_of_the_domains_the_component_owns ... ok
test every_schema_a_document_embeds_is_valid_in_the_dialect_openapi_31_declares ... ok
test the_document_a_server_hands_out_is_the_committed_one_in_the_other_dialect ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/provenance.rs (target/debug/deps/provenance-9ec9789f25e84015)

running 9 tests
test a_text_without_both_digests_reads_as_nothing ... ok
test the_reader_reads_back_every_form_the_writer_emits ... ok
test a_generator_that_stamps_nothing_cannot_ship_an_artifact - should panic ... ok
test a_damaged_digest_reads_as_nothing ... ok
test the_whole_model_contract_digest_is_not_the_source_digest ... ok
test a_whole_model_slice_is_stamped_as_one ... ok
test a_generator_that_pairs_a_stamp_with_the_wrong_slice_cannot_ship_an_artifact - should panic ... ok
test a_change_no_construct_can_be_named_for_moves_every_contract_digest ... ok
test a_change_outside_an_artifacts_slice_leaves_its_contract_digest_standing ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/relations.rs (target/debug/deps/relations-7cdc743b0b26371d)

running 4 tests
test the_committed_openapi_document_is_byte_for_byte_what_the_projection_writes ... ok
test the_openapi_document_states_the_relation_and_links_the_targets_schema ... ok
test the_entity_document_states_the_relation_on_the_property_that_carries_it ... ok
test the_committed_entity_documents_are_byte_for_byte_what_the_schema_projection_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/schema.rs (target/debug/deps/schema-e0945bde8d462715)

running 27 tests
test a_field_carries_its_own_words_beside_the_reference_to_its_type ... ok
test a_list_element_may_be_null_where_a_field_may_only_be_absent ... ok
test a_field_is_called_what_the_specification_says_it_is_called_on_the_wire ... ok
test a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test a_map_key_that_is_not_the_text_its_key_type_is_spelt_with_is_refused ... ok
test a_bytes_field_refuses_a_string_that_is_not_base64 ... ok
test an_optional_field_may_be_absent_and_a_required_field_may_not ... ok
test a_map_is_an_object_whose_keys_are_the_text_its_key_type_is_spelt_with ... ok
test a_uuid_newtype_carries_the_format_of_what_it_wraps ... ok
test a_tagged_union_round_trips_because_every_branch_pins_its_tag ... ok
test an_amount_is_written_as_an_exact_decimal_string_and_a_float_is_refused ... ok
test an_error_that_carries_nothing_accepts_an_empty_object_and_nothing_else ... ok
test a_command_input_accepts_a_filled_instance_and_refuses_a_misspelt_field ... ok
test an_invariant_travels_with_the_type_and_says_it_is_not_a_constraint ... ok
test an_event_payload_accepts_what_the_specification_says_it_carries ... ok
test a_newtype_keeps_its_name_instead_of_collapsing_into_its_representation ... ok
test a_newtype_over_a_string_publishes_no_constraint_the_specification_never_stated ... ok
test a_uuid_is_refused_unless_it_is_the_canonical_hyphenated_form ... ok
test a_decimal_amount_is_refused_when_it_is_not_written_the_way_the_pattern_says ... ok
test every_artifact_is_a_json_schema_document_declaring_the_dialect_it_is_written_in ... ok
test every_command_input_event_payload_error_payload_and_named_type_gets_a_schema ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_message_accepts_an_instance_of_itself_and_refuses_one_that_is_wrong ... ok
test no_schema_uses_a_keyword_outside_the_set_this_projection_publishes ... ok
test every_schema_says_which_specification_it_came_from ... ok
test every_published_document_is_a_valid_json_schema_in_the_dialect_it_declares ... ok
test generation_is_byte_identical_between_runs ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

   Doc-tests ess_gen

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


exit: 0
```

```sh
env TMPDIR="$PWD/target/review-boundaries-2" cargo fmt -p ess-cli -p ess-gen --check
```

```text
(no output)

exit: 0
```

```sh
env SCCACHE_SERVER_UDS="$PWD/target/sccache.sock" TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo clippy --locked -p ess-cli -p ess-gen --all-targets -- -D warnings
```

```text
    Checking ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 0.27s

exit: 0
```

4. Findings and reachability

No remaining finding was established. The corrected first-pass collision case remains in the selected suite and passed. The only new red actual execution was the incomplete BuildSpec fixture, which failed before projection; it did not establish an output-containment defect and was repaired without weakening the test.

The new cases invoke the real local ESS binary with compose, generate, synthesize, conform synthesize --target go, conform web, project buildkit and project kubernetes. Their inputs are synthetic or local copies of committed offline fixtures, their valid controls succeed, and refusal checks compare all synthetic destination bytes and entry kinds. No external runner or live infrastructure is invoked.

5. Properties checked without a remaining failure

- Valid disjoint companions inside generated directories preserve canonical companion bytes and repeat unchanged.
- Native Unix non-UTF8/backslash/colon filenames retain their meaning without generated-path alphabet restrictions.
- Encountered parent links cannot be hidden by normalized .. in either compose companion.
- Four generation workflows and two projection workflows inspect late conflicting destinations before changing earlier outputs; valid repeated output is stable.
- All prior traversal, uniqueness, link, root-normalization, companion-set and native-directory-spelling assertions remain selected and passing.

6. Writes, process and cleanup ownership

Production writes: none. Repository additions are confined to the assigned integration-test file. Logs, patch, report, failed Cargo metadata copy and socket-only trace are under target/review-boundaries-2. A temporary target/t directory was tried during diagnosis, remained empty and was removed with rmdir.

Outside-worktree authored root: /home/timo/.cache/ess-w2-ctmp. It held only the first actual test's generated synthetic fixture descendants; their individual names were not inventoried. The fixture Drop removed them, then rmdir of the empty root succeeded. This outside-worktree scratch adaptation and its cleanup are explicit, not an assertion that nothing was written there. Existing shared sccache cache activity remains ordinary configured cache activity, not separately inventoried output.

The task-specific server at <unit>/target/sccache.sock was stopped with its exact SCCACHE_SERVER_UDS: stop-server exited0 and a subsequent connection no longer succeeded. The stop output is preserved in coordinator-pass-2-cache-stop.log. The coordinator owns retaining wanted evidence, publishing the green integrated wave, and exact-id managed worktree cleanup. The unrelated default cache server and other sessions' worktrees were not stopped or modified.

```findings
[]
```
