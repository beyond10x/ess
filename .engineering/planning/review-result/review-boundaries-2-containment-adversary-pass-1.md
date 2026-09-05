---
format: aep.planning-md/1
id: review-result:review-boundaries-2-containment-adversary-pass-1
kind: review-result
status: active
title: Output containment adversary pass 1
relations:
- reviews: story:review-output-containment
revision: 1
---
unit: story:review-output-containment at e6803c061b33dfe8d5c9fdfff10d8f1408083b31 plus the test-only working-tree additions below
verdict: CONFIRMED — compose companion outputs can collide with the generated client tree before refusal
cases: executed 220→225, red 1
origin: introduced 0 / pre-existing 0 / undecided 1
wrote-outside-worktree: none
needs-coordinator: route the compose output-set collision correction and acknowledge the extra read-only formatting command scope deviation
```text
$ git --no-pager diff --stat
 crates/edge/ess-cli/tests/output_containment.rs | 114 ++++++++++++++++++++++++
 crates/generate/ess-gen/tests/docs.rs           |  68 ++++++++++++++
 2 files changed, 182 insertions(+)
```

1. Diff bound

Only the two integration-test files shown above changed: 182 insertions, no deletions. Existing assertions remain intact. No production files, inline tests, planning files, Git lifecycle, or AEP commands were changed or run. The captured patch is target/review-boundaries-2/adversary-tests.patch. git diff --check exited 0. The tests were written before any test command in this pass. The baseline 220 is the implementor/coordinator handoff count (ess-cli 33, ess-gen 187), not a preemptive suite run.

Process deviation: after the full package suites, I mistakenly ran cargo fmt --all --check even though the brief explicitly requires package-scoped fmt and forbids --all. It was read-only, exited 1, and reported formatting differences only in untouched generated/rust/billing and generated/rust/gatepass sources. No formatter mutation was performed. The required package-scoped check subsequently passed. This command-scope violation is disclosed rather than represented as a finding about this unit.

2. New cases and their first isolated executions

All commands below ran in /home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment. TMPDIR and build target remained within this worktree. The exact first-run outputs are reproduced below; each selected exactly one case.

1. crates/edge/ess-cli/tests/output_containment.rs:206 — RED

A valid real compose fixture with disjoint outputs succeeds. For each of --out and --client-plan-out, choosing out/Cargo.toml or out/src together with --client-rust-out out must refuse before either the existing client manifest or the source-parent destination changes.

```sh
env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-cli --test output_containment composition_companion_outputs_cannot_collide_with_the_generated_client_tree -- --exact > target/review-boundaries-2/adversary-case-1.log 2>&1
```

Exit status: 101.

```text
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.23s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 1 test
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... FAILED

failures:

---- composition_companion_outputs_cannot_collide_with_the_generated_client_tree stdout ----

thread 'composition_companion_outputs_cannot_collide_with_the_generated_client_tree' (183772) panicked at crates/edge/ess-cli/tests/output_containment.rs:246:5:
companion/generated destination collisions must refuse before writes:
--out out/Cargo.toml: exit Some(0), client manifest preserved false, source parent absent false; stderr: 
--out out/src: exit Some(1), client manifest preserved true, source parent absent false; stderr: error: output path has an incompatible file type or symlink: /home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/target/review-boundaries-2/ess-output-183771-2/out/src
--client-plan-out out/Cargo.toml: exit Some(0), client manifest preserved false, source parent absent false; stderr: 
--client-plan-out out/src: exit Some(1), client manifest preserved true, source parent absent false; stderr: error: output path has an incompatible file type or symlink: /home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/target/review-boundaries-2/ess-output-183771-4/out/src
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    composition_companion_outputs_cannot_collide_with_the_generated_client_tree

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.06s

error: test failed, to rerun pass `-p ess-cli --test output_containment`
```

2. crates/edge/ess-cli/tests/output_containment.rs:265 — GREEN

Legitimate requested roots working/../out and absent/../out preserve generated bytes; an existing file hidden by ../ is refused without writes. Normalization does not create the discarded absent component.

```sh
env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-cli --test output_containment requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files -- --exact > target/review-boundaries-2/adversary-case-2.log 2>&1
```

Exit status: 0.

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 1 test
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.10s

```

3. crates/edge/ess-cli/tests/output_containment.rs:292 — GREEN

Site includes colliding with late static assets or spelling a shared directory with a different case refuse before modifying sentinels or creating any output directories.

```sh
env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-cli --test output_containment late_site_asset_aliases_refuse_before_even_creating_output_directories -- --exact > target/review-boundaries-2/adversary-case-3.log 2>&1
```

Exit status: 0.

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 1 test
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.07s

```

4. crates/generate/ess-gen/tests/docs.rs:57 — GREEN

The public checked Site API rejects JSON-deserialized page IDs that collide with late static assets, a generated filename's descendant, or a case-aliased parent directory.

```sh
env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-gen --test docs checked_site_rejects_deserialized_collisions_with_late_static_assets -- --exact > target/review-boundaries-2/adversary-case-4.log 2>&1
```

Exit status: 0.

```text
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/generate/ess-gen)
    Finished `test` profile [unoptimized] target(s) in 0.81s
     Running tests/docs.rs (target/debug/deps/docs-530499ed1f64a736)

running 1 test
test checked_site_rejects_deserialized_collisions_with_late_static_assets ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 31 filtered out; finished in 0.00s

```

5. crates/generate/ess-gen/tests/docs.rs:79 — GREEN

Valid JSON- and YAML-deserialized nested pages preserve every byte and artifact path from the legacy renderer, including all three static assets.

```sh
env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-gen --test docs checked_site_preserves_valid_deserialized_nested_pages_and_every_artifact_byte -- --exact > target/review-boundaries-2/adversary-case-5.log 2>&1
```

Exit status: 0.

```text
    Finished `test` profile [unoptimized] target(s) in 0.04s
     Running tests/docs.rs (target/debug/deps/docs-530499ed1f64a736)

running 1 test
test checked_site_preserves_valid_deserialized_nested_pages_and_every_artifact_byte ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 31 filtered out; finished in 0.01s

```

3. Full package suites and checks

The full suites ran after all five isolated cases. ess-cli executed 36 cases (35 passed, one failed); ess-gen executed 189 cases (all passed). Combined, the runner count increased from 220 to 225 with no ignored cases. The red suite is the reproducible attack result.

ess-cli:

```sh
env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-cli > target/review-boundaries-2/adversary-ess-cli-suite.log 2>&1
```

Exit status: 101.

```text
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.20s
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/command_surface.rs (target/debug/deps/command_surface-f896f6f697ed70aa)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-14ea054bad6c3502)

running 4 tests
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-ba60d23811c1c6c2)

running 7 tests
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s

     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 9 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... FAILED
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok

failures:

---- composition_companion_outputs_cannot_collide_with_the_generated_client_tree stdout ----

thread 'composition_companion_outputs_cannot_collide_with_the_generated_client_tree' (191165) panicked at crates/edge/ess-cli/tests/output_containment.rs:246:5:
companion/generated destination collisions must refuse before writes:
--out out/Cargo.toml: exit Some(0), client manifest preserved false, source parent absent false; stderr: 
--out out/src: exit Some(1), client manifest preserved true, source parent absent false; stderr: error: output path has an incompatible file type or symlink: /home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/target/review-boundaries-2/ess-output-191161-11/out/src
--client-plan-out out/Cargo.toml: exit Some(0), client manifest preserved false, source parent absent false; stderr: 
--client-plan-out out/src: exit Some(1), client manifest preserved true, source parent absent false; stderr: error: output path has an incompatible file type or symlink: /home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/target/review-boundaries-2/ess-output-191161-18/out/src
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    composition_companion_outputs_cannot_collide_with_the_generated_client_tree

test result: FAILED. 8 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s

error: test failed, to rerun pass `-p ess-cli --test output_containment`
```

ess-gen:

```sh
env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-gen > target/review-boundaries-2/adversary-ess-gen-suite.log 2>&1
```

Exit status: 0.

```text
    Finished `test` profile [unoptimized] target(s) in 0.04s
     Running unittests src/lib.rs (target/debug/deps/ess_gen-5cfeec7d828080d8)

running 55 tests
test artifact::tests::portable_artifacts_refuse_escape_and_platform_aliases ... ok
test artifact::tests::a_destination_set_rejects_duplicates_case_aliases_and_file_parents_in_any_order ... ok
test authored::tests::a_fence_keeps_its_language_and_loses_its_trailing_newline ... ok
test authored::tests::a_heading_becomes_a_section_with_an_anchor ... ok
test authored::tests::a_link_an_adopter_wrote_stays_theirs ... ok
test authored::tests::a_leading_title_becomes_the_page_title_and_not_a_second_heading ... ok
test authored::tests::a_paragraph_keeps_its_inline_structure ... ok
test authored::tests::a_list_becomes_items_and_a_quote_becomes_a_quote ... ok
test authored::tests::a_table_keeps_its_header_apart_from_its_rows ... ok
test authored::tests::raw_html_is_dropped_rather_than_passed_through ... ok
test docs::tests::a_gap_that_ships_says_which_crate_closes_it ... ok
test authored::tests::a_top_level_heading_is_demoted_because_the_page_title_is_the_first ... ok
test docs::tests::a_heading_and_its_anchor_agree ... ok
test docs::tests::a_lifecycle_renders_as_a_state_diagram_with_its_initial_and_terminal_states_marked ... ok
test docs::tests::a_lifecycle_that_connects_every_pair_says_it_forbids_nothing ... ok
test docs::tests::a_lifecycle_with_one_state_forbids_nothing_rather_than_forbidding_everything ... ok
test docs::tests::a_list_of_three_reads_as_a_person_would_write_it ... ok
test docs::tests::a_plural_of_entity_is_entities ... ok
test docs::tests::a_state_no_transition_touches_is_still_drawn ... ok
test docs::tests::a_transition_from_two_states_draws_one_arrow_from_each ... ok
test document::tests::a_link_names_what_it_points_at_and_never_a_path ... ok
test docs::tests::the_page_names_every_transition_the_specification_does_not_permit ... ok
test graph::tests::a_dot_label_keeps_its_parts_on_separate_lines ... ok
test graph::tests::a_mermaid_label_cannot_close_the_quoted_string_it_sits_in ... ok
test document::tests::a_page_id_says_how_deep_it_is_so_a_renderer_can_reach_the_root ... ok
test graph::tests::a_component_group_is_a_dot_cluster_and_graphviz_only_boxes_clusters ... ok
test html::tests::a_construct_is_addressed_by_the_section_that_documents_it ... ok
test html::tests::a_code_block_is_a_code_listing_and_carries_its_language ... ok
test html::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test document::tests::a_document_round_trips_through_its_own_format ... ok
test html::tests::a_table_is_a_table_with_a_head_and_a_body ... ok
test html::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test html::tests::a_diagram_is_a_pre_the_renderer_draws_into_and_never_a_code_listing ... ok
test html::tests::a_page_reaches_its_stylesheet_and_its_renderer_from_wherever_it_sits ... ok
test schema::types::tests::a_reference_is_a_pointer_into_the_defs_of_the_document_holding_it ... ok
test html::tests::an_adopters_front_page_goes_above_the_index_and_nowhere_else ... ok
test schema::types::tests::a_string_keyed_map_publishes_no_property_name_rule_that_checks_nothing ... ok
test html::tests::markup_in_text_never_reaches_the_browser_as_markup ... ok
test markdown::tests::a_quotation_marks_every_line_it_covers ... ok
test markdown::tests::a_section_flattens_into_the_stream_and_its_children_follow_it ... ok
test markdown::tests::a_table_is_written_with_the_separator_a_reader_expects ... ok
test html::tests::the_sidebar_groups_the_nested_pages_and_marks_the_page_the_reader_is_on ... ok
test markdown::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test schema::types::tests::a_decimal_is_written_as_an_exact_string_because_a_json_number_is_read_as_a_float ... ok
test schema::types::tests::an_integer_key_is_constrained_to_the_text_an_integer_is_spelt_with ... ok
test markdown::tests::a_diagram_is_a_fenced_mermaid_block ... ok
test schema::types::tests::an_optional_outside_a_field_gains_a_null_branch_because_a_list_element_cannot_be_absent ... ok
test markdown::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test schema::types::tests::a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test schema::types::tests::a_union_branch_pins_its_tag_so_exactly_one_branch_can_match ... ok
test schema::types::tests::a_union_tagged_value_moves_its_payload_aside_rather_than_colliding_with_the_tag ... ok
test html::tests::every_emitted_file_says_what_it_was_generated_from ... ok
test html::tests::the_default_style_is_the_stylesheet_that_is_published ... ok
test html::tests::checked_rendering_validates_deserialized_page_identities_before_map_collection ... ok
test html::tests::checked_rendering_preserves_valid_parent_and_nested_page_bytes ... ok

test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/agreement.rs (target/debug/deps/agreement-a6d7a7ff380699da)

running 4 tests
test no_projection_collapses_a_newtype_into_the_representation_it_wraps ... ok
test every_projection_publishes_the_same_schema_for_a_construct_more_than_one_of_them_describes ... ok
test every_keyword_the_projections_publish_is_classified_as_an_assertion_or_an_annotation ... ok
test the_agreement_check_compares_the_constructs_the_defect_was_about_rather_than_nothing ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/asyncapi.rs (target/debug/deps/asyncapi-9e439fb4245702f8)

running 18 tests
test a_collection_says_what_it_holds_and_an_absent_element_is_null_because_it_has_no_key_to_omit ... ok
test a_payload_refuses_an_undeclared_field_and_spells_absence_by_leaving_it_out_of_required ... ok
test a_dropped_failure_is_stated_in_prose_and_not_only_in_an_extension ... ok
test a_payload_field_carries_the_grammar_the_model_states_and_not_a_note_naming_it ... ok
test a_union_pins_its_tag_so_exactly_one_branch_matches_rather_than_none_or_both ... ok
test a_binding_no_component_handles_still_states_its_failure_policy ... ok
test the_publisher_of_an_event_sees_who_reacts_to_it_and_under_what_failure_policy ... ok
test every_event_in_the_billing_example_appears_in_some_document ... ok
test a_bindings_mapping_and_the_reason_for_its_type_crossing_reach_the_document ... ok
test a_bindings_delivery_and_failure_reach_the_receiving_operation ... ok
test the_channel_and_its_message_say_nothing_about_the_binding ... ok
test every_document_carries_the_provenance_of_the_model_it_came_from ... ok
test an_events_channel_address_is_its_declared_wire_name_or_else_its_qualified_name ... ok
test a_document_shows_what_the_component_publishes_and_what_it_reacts_to ... ok
test a_document_is_a_valid_asyncapi_three_skeleton ... ok
test every_ref_resolves_inside_the_document_that_holds_it ... ok
test every_component_gets_one_document_named_after_it ... ok
test regenerating_from_the_same_model_produces_the_same_bytes ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

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
test an_entitys_identity_reaches_the_page_by_name_and_not_only_by_type ... ok
test a_bindings_delivery_and_failure_semantics_are_stated_in_words ... ok
test a_commands_refusal_branch_is_documented_and_not_only_its_name ... ok
test a_type_reached_only_through_an_entitys_field_is_not_called_unreached ... ok
test a_declared_conversion_carries_its_reason_everywhere_a_reader_might_start ... ok
test an_actor_that_may_invoke_nothing_is_still_on_the_page ... ok
test a_views_eventual_consistency_reads_differently_from_an_immediate_one ... ok
test a_binding_renders_as_a_flow_and_a_lifecycle_as_a_state_diagram ... ok
test an_entitys_invariant_reaches_the_page_as_a_condition_on_every_instance ... ok
test an_entitys_absent_transition_is_named_as_a_move_the_specification_does_not_permit ... ok
test an_empty_gap_allowlist_puts_no_cannot_show_section_on_any_page ... ok
test an_outcome_the_input_cannot_decide_says_so_rather_than_claiming_it_is_unreachable ... ok
test an_actors_grant_renders_as_an_edge_from_the_actor_to_that_command_in_the_index_graph ... ok
test checked_site_preserves_valid_deserialized_nested_pages_and_every_artifact_byte ... ok
test a_wrong_state_branch_is_documented_with_the_states_the_document_never_lists ... ok
test an_entitys_lifecycle_transitions_reach_the_page_as_arrows ... ok
test an_outcome_that_changes_an_entity_says_which_instance_and_where_the_identity_is_read ... ok
test checked_site_rejects_deserialized_collisions_with_late_static_assets ... ok
test a_components_ownership_and_a_workloads_replica_floor_are_both_documented ... ok
test a_views_filter_reaches_the_page_rather_than_being_silently_dropped ... ok
test an_events_payload_and_an_errors_payload_are_both_documented_field_by_field ... ok
test the_command_that_takes_each_move_reaches_the_page_beside_the_move_itself ... ok
test every_member_of_a_resolved_domain_reaches_the_page_of_the_context_it_belongs_to ... ok
test every_name_the_ir_holds_appears_on_some_page ... ok
test an_outcome_says_what_it_does_to_an_entity_and_a_refusal_says_it_changes_none ... ok
test every_link_between_pages_lands_on_a_page_that_exists_at_the_heading_it_names ... ok
test the_provenance_header_is_a_markdown_comment_a_renderer_can_close ... ok
test every_page_says_which_specification_produced_it ... ok
test every_type_kind_reaches_a_page_including_the_tagged_union ... ok
test generating_the_documentation_twice_produces_byte_identical_output ... ok

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
test a_document_is_valid_yaml_with_a_version_an_info_block_and_paths ... ok
test a_command_no_binding_invokes_carries_no_idempotency_header ... ok
test a_command_names_the_actors_permitted_to_invoke_it_and_no_authentication_mechanism ... ok
test a_commands_input_becomes_a_closed_object_over_its_declared_fields ... ok
test a_command_with_no_wire_name_is_exposed_under_the_name_the_model_gives_it ... ok
test a_command_is_exposed_at_its_wire_name_under_its_domains ... ok
test a_command_no_actor_names_carries_no_grant_rather_than_a_grant_to_everybody ... ok
test a_refusal_the_input_decides_carries_the_declared_error_payload ... ok
test a_decimal_is_a_string_because_a_json_number_is_a_float ... ok
test every_component_gets_one_document_named_after_it ... ok
test a_newtype_stays_a_schema_of_its_own_rather_than_becoming_its_representation ... ok
test several_outcomes_on_one_status_stay_distinguishable ... ok
test a_command_is_only_ever_a_post ... ok
test two_commands_claiming_one_path_both_move_to_their_qualified_names ... ok
test an_external_outcome_is_an_upstream_failure_and_not_a_validation_refusal ... ok
test an_outcome_that_emits_says_so_without_claiming_to_return_the_events ... ok
test each_declared_outcome_is_its_own_response_and_no_status_is_invented ... ok
test a_command_a_binding_delivers_at_least_once_requires_an_idempotency_key ... ok
test every_document_carries_its_provenance_as_a_comment_and_as_data ... ok
test a_refusal_the_subjects_state_decides_is_a_conflict_and_not_a_bad_request ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test the_operation_id_is_the_commands_qualified_name ... ok
test every_schema_the_document_declares_is_pointed_at_by_something ... ok
test every_document_this_generator_can_produce_is_a_valid_openapi_document ... ok
test the_entities_published_are_exactly_those_of_the_domains_the_component_owns ... ok
test regenerating_from_the_same_ir_produces_the_same_bytes ... ok
test the_document_a_server_hands_out_is_the_committed_one_in_the_other_dialect ... ok
test every_schema_a_document_embeds_is_valid_in_the_dialect_openapi_31_declares ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/provenance.rs (target/debug/deps/provenance-9ec9789f25e84015)

running 9 tests
test a_damaged_digest_reads_as_nothing ... ok
test a_whole_model_slice_is_stamped_as_one ... ok
test a_generator_that_stamps_nothing_cannot_ship_an_artifact - should panic ... ok
test a_text_without_both_digests_reads_as_nothing ... ok
test the_reader_reads_back_every_form_the_writer_emits ... ok
test the_whole_model_contract_digest_is_not_the_source_digest ... ok
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
test a_list_element_may_be_null_where_a_field_may_only_be_absent ... ok
test a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test a_field_is_called_what_the_specification_says_it_is_called_on_the_wire ... ok
test a_field_carries_its_own_words_beside_the_reference_to_its_type ... ok
test an_optional_field_may_be_absent_and_a_required_field_may_not ... ok
test a_map_key_that_is_not_the_text_its_key_type_is_spelt_with_is_refused ... ok
test a_map_is_an_object_whose_keys_are_the_text_its_key_type_is_spelt_with ... ok
test a_bytes_field_refuses_a_string_that_is_not_base64 ... ok
test a_command_input_accepts_a_filled_instance_and_refuses_a_misspelt_field ... ok
test an_error_that_carries_nothing_accepts_an_empty_object_and_nothing_else ... ok
test a_newtype_keeps_its_name_instead_of_collapsing_into_its_representation ... ok
test a_tagged_union_round_trips_because_every_branch_pins_its_tag ... ok
test a_newtype_over_a_string_publishes_no_constraint_the_specification_never_stated ... ok
test a_uuid_newtype_carries_the_format_of_what_it_wraps ... ok
test an_invariant_travels_with_the_type_and_says_it_is_not_a_constraint ... ok
test a_decimal_amount_is_refused_when_it_is_not_written_the_way_the_pattern_says ... ok
test a_uuid_is_refused_unless_it_is_the_canonical_hyphenated_form ... ok
test every_command_input_event_payload_error_payload_and_named_type_gets_a_schema ... ok
test an_event_payload_accepts_what_the_specification_says_it_carries ... ok
test every_artifact_is_a_json_schema_document_declaring_the_dialect_it_is_written_in ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_message_accepts_an_instance_of_itself_and_refuses_one_that_is_wrong ... ok
test an_amount_is_written_as_an_exact_decimal_string_and_a_float_is_refused ... ok
test no_schema_uses_a_keyword_outside_the_set_this_projection_publishes ... ok
test every_published_document_is_a_valid_json_schema_in_the_dialect_it_declares ... ok
test every_schema_says_which_specification_it_came_from ... ok
test generation_is_byte_identical_between_runs ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

   Doc-tests ess_gen

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Required package formatting and strict Clippy:

```sh
env TMPDIR="$PWD/target/review-boundaries-2" cargo fmt -p ess-cli -p ess-gen --check
```

Exit status: 0.

```text
(no output)
```

```sh
env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo clippy --locked -p ess-cli -p ess-gen --all-targets -- -D warnings
```

Exit status: 0.

```text
    Checking ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/generate/ess-gen)
    Checking ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 0.33s
```

Extra read-only formatting command, already disclosed in part 1:

```sh
env TMPDIR="$PWD/target/review-boundaries-2" cargo fmt --all --check
```

Exit status: 1. Its complete, unmodified 83,016-character output is retained at target/review-boundaries-2/adversary-fmt.log; it reports existing formatting differences in 14 generated source files outside these packages. These differences were not edited or classified as an output-containment finding.

Disk available before this attack was 134,344,421,376 bytes and after the final checks was 137,631,903,744 bytes, above the 8,589,934,592-byte reserve. Cargo used the prescribed sccache wrapper, no incremental compilation, and zero dev/test debug info. No caches or build directories were removed.

4. Findings and reachability

Findings cover e6803c061b33dfe8d5c9fdfff10d8f1408083b31 with only the test additions shown in part 1. The base is c1c23b24cee8f527784b7f8467c21a609710c65e; no base execution occurred, so origin remains undecided. The single finding is a residual acceptance gap in the assigned compose generated-tree sink.

| File:line | Category | Severity | Verdict | Origin | Finding |
|---|---|---|---|---|---|
| crates/edge/ess-cli/src/main.rs:1872 | acceptance | blocker | CONFIRMED | undecided | Compose does not preflight companion JSON destinations together with generated client destinations, so file/file collisions overwrite output and file/directory collisions mutate output before refusal. |

What was measured: the assertion at crates/edge/ess-cli/tests/output_containment.rs:246 fails in its first isolated execution and in the full ess-cli suite, both exit 101. Both companion flags were tested against both collision kinds. With --client-rust-out out and either --out out/Cargo.toml or --client-plan-out out/Cargo.toml, the command exits 0, the existing out/Cargo.toml sentinel changes, and out/src is created. The same invocation claims one path for two different outputs; the generated client manifest overwrites the companion JSON. With either companion destination set to out/src, the command exits 1 after the companion write creates out/src as a file; the existing out/Cargo.toml sentinel remains unchanged. Thus the directory variant measures a new companion-only file inside the selected generated tree before preflight refusal, not an already generated manifest being changed. A valid disjoint-output control succeeds first, proving that fixture compilation and ordinary output generation work.

What reaches it: the real ess compose command, using committed fixture crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml and --service todo=<workspace>/crates/specify/ess-composition/tests/fixtures/two-components --service usage=<same>. Each failing invocation includes --client-rust-out <fixture>/out plus exactly one of --out or --client-plan-out directed at <fixture>/out/Cargo.toml or <fixture>/out/src. The command builder is at tests/output_containment.rs:191; the exact flag matrix is at :222. No test-only implementation entry point, live system, symlink race, or mount race is needed.

The implementation first validates only client_artifacts at main.rs:1862, then writes the composition and client-plan companions at :1872 and :1876, and finally preflights/writes the client tree at :1879. The omitted relationships are therefore discoverable from the command's entire output set before any write. This is not ordinary later I/O failure or a request for atomic rollback: the same invocation declares the mutually conflicting destinations. The correction should compare all declared output paths, including file/directory relationships, before either companion write while preserving valid disjoint destinations and permitted requested roots containing ../. Coordinator owns the routing and implementation; no fix was applied here.

5. Attacks that did not break

- Requested-root normalization accepts legitimate parent roots, avoids creating discarded missing components, and refuses an encountered file hidden by ../ without changing valid output bytes.
- Checked Site rendering validates the full sequence including static assets before map collection can erase an identity collision.
- Real site includes with case-aliased shared directories or descendants of static asset files refuse before creating output directories or modifying sentinels.
- Checked rendering preserves every artifact byte for valid nested Document pages decoded from JSON or YAML.
- Existing package tests for portable path refusal, duplicate includes, symlink roots/ancestors/destinations, hardlinked destinations, ordinary generation, and valid layout remain green.

6. Writes and retained scratch

Authored paths outside the worktree: none. All added test files, fixture temporary directories, logs, and compiler target output stayed inside /home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment. Cargo used the prescribed shared sccache service; its ordinary existing cache activity was not independently inventoried or assigned a new output location.

Retained scratch directory: /home/timo/.local/state/worktree/trees/b10x/ess/review-output-containment/target/review-boundaries-2.
Retained pass files: adversary-pass-1.md, adversary-tests.patch, adversary-case-1.log through adversary-case-5.log, adversary-ess-cli-suite.log, adversary-ess-gen-suite.log, adversary-package-fmt.log, adversary-clippy.log, and the extra-check log adversary-fmt.log. Existing fixture Drop implementations cleaned only their synthetic per-test directories after execution. The coordinator owns all worktree and build-directory cleanup.

```findings
- file: crates/edge/ess-cli/src/main.rs
  line: 1872
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: undecided
  message: Compose does not preflight companion JSON destinations together with generated client destinations, so file/file collisions overwrite output and file/directory collisions mutate output before refusal.
```

