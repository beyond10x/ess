unit: story:review-report-reader-validation
verdict: green
cases: executed 229→238, red 6
origin: n/a
wrote-outside-worktree: none
needs-coordinator: no

## 1. Unit and acceptance

story:review-report-reader-validation — Validate standalone conformance report claims on read. Acceptance: The standalone v1 report reader refuses unknown versions and internally contradictory report claims that currently deserialize successfully.

The change places validation in the public Deserialize implementation, backed by a private closed wire shape. It checks the exact report format, supported suite version, non-pass count/list agreement, count bounded by executed scenarios, known non-pass vocabulary, and aggregate status. Existing Serialize code and field order are unchanged. V1 scenarios_failed still counts every non-pass, Rust unsupported still makes a run failed, and Rust errors or Go skips alone make it inconclusive. Empty execution summaries remain passing. Scenario identities remain opaque; the report supplies no authority for asserting source coverage or guessing a producer from a status token.

| Inferred scope line | Observation | Result |
| --- | --- | --- |
| Extend tests beside `src/evidence.rs:180`; exact fixture placement is an implementation choice. | The existing `evidence::tests` module owns the report field, canonical round-trip, and closed-shape checks. The new cases fit that module; the Go report bytes are a fixed inline fixture matching `src/go/runtime.go:581` field order and `writeReport` non-pass accounting. | Confirmed; no new external fixture surface. |
| No separate document change is required. | The assignment validates the existing v1 report and the supported suite-format authority. The writer and persisted fields remain unchanged. | Confirmed. |
| No additional surfaces are established. | `StandaloneConformanceReport`, its Deserialize implementation, Rust report aggregation, Go writer, and `SuiteFormat` all reside in `crates/verify/ess-conformance`. | Confirmed. |
| Work collides with any other unit scoped to `crates/verify/ess-conformance`. | The reader and existing tests share `src/evidence.rs`; the crate-directory scope includes the complete change. | Confirmed; retain the single crate scope. |

The graph node for `story:review-report-reader-validation` contains only `decomposes: epic:review-boundary-remediation` and `serves: vision:O2`; it has no prerequisite or blocker edge. No proposed mechanical fix in Scope needed an independent relocation experiment.

## 2. Observed diff

```text
 crates/verify/ess-conformance/src/evidence.rs | 295 +++++++++++++++++++++++++-
 1 file changed, 292 insertions(+), 3 deletions(-)
```

git diff --check: exit 0. Only the assigned package's existing evidence.rs source/test module changed. No commit, planning mutation, or worktree lifecycle command was performed.

## 3. Red run before implementation

Command: cargo test --locked -p ess-conformance
Exit: 101

All six refusal tests failed because all four public routes accepted invalid reports: the explicit report reader, JSON value deserialization, JSON reader deserialization, and YAML deserialization. The three added compatibility cases passed on the base. The run stopped after the failed library lane; it did not claim the later integration lanes executed.

The following is the test runner's output verbatim. Complete raw Cargo output, including compilation diagnostics, is retained at target/review-boundaries-1/review-report-reader-validation/red.log.

```text
running 67 tests
test decision::tests::a_decision_reads_its_two_other_cases_as_neither_satisfied_nor_the_other ... ok
test decision::tests::a_refusal_renders_the_predicate_the_command_and_every_reason ... ok
test decision::tests::exactly_one_reason_says_another_candidate_would_help ... ok
test evidence::tests::a_standalone_report_carries_every_field_an_adapter_needs ... ok
test evidence::tests::report_readers_refuse_malformed_and_unsupported_suite_versions ... FAILED
test evidence::tests::report_readers_do_not_guess_a_producer_from_status_vocabulary ... ok
test evidence::tests::report_readers_refuse_more_nonpasses_than_executed_scenarios ... FAILED
test evidence::tests::the_closed_report_round_trips_with_identical_canonical_bytes ... ok
test evidence::tests::report_readers_refuse_nonpass_count_and_list_disagreement ... FAILED
test faulty::tests::a_fault_is_injected_into_the_system_that_declares_what_it_breaks ... ok
test faulty::tests::every_fault_says_what_it_is_and_where_it_goes ... ok
test evidence::tests::report_readers_refuse_status_claims_that_contradict_the_list ... FAILED
test faulty::tests::no_two_faults_claim_the_same_scenario ... ok
test evidence::tests::report_readers_refuse_unknown_report_formats ... FAILED
test faulty::tests::only_the_two_faults_the_boundary_cannot_express_are_injected_in_the_implementation ... ok
test evidence::tests::report_readers_refuse_nonpass_entries_without_a_known_nonpass_status ... FAILED
test evidence::tests::unknown_report_fields_are_refused ... ok
test input::tests::a_primitive_refuses_a_node_of_the_wrong_shape_rather_than_coercing_it ... ok
test go::tests::every_go_file_is_in_the_package_the_readme_names ... ok
test input::tests::every_primitive_projects_to_the_one_fact_value_that_can_hold_it ... ok
test input::tests::shape_errors_render_one_per_line_and_name_the_input_root_by_name ... ok
test report::tests::a_quoted_input_reads_as_the_call_that_was_made ... ok
test report::tests::a_diagnostic_answers_all_five_of_the_questions_a_failure_has_to_answer ... ok
test report::tests::a_scenario_status_is_the_strongest_of_its_checks_and_a_contradiction_outranks_everything ... ok
test go::tests::the_runner_is_a_constant_and_only_the_suite_moves ... ok
test report::tests::an_unsupported_scenario_makes_the_run_fail_rather_than_look_like_a_pass ... ok
test report::tests::every_check_code_has_a_distinct_name_and_a_rule_sentence ... ok
test runner::tests::a_count_with_neither_bound_is_a_suite_defect_and_not_a_satisfied_assertion ... ok
test runner::tests::a_count_is_the_half_of_an_ordering_claim_that_says_the_rows_were_there ... ok
test evidence::tests::report_readers_preserve_go_producer_bytes_and_historical_nonpass_counts ... ok
test runner::tests::a_declared_order_is_checked_on_adjacent_rows_and_the_next_key_breaks_a_tie ... ok
test runner::tests::a_nested_row_binds_the_paths_a_predicate_spells ... ok
test runner::tests::a_position_in_a_view_that_declares_no_order_is_a_suite_defect ... ok
test runner::tests::a_position_names_both_ends_and_a_row_that_is_not_there_is_not_a_match ... ok
test runner::tests::a_ranking_key_a_row_does_not_publish_is_undecidable_rather_than_out_of_order ... ok
test runner::tests::a_predicate_a_row_cannot_answer_is_reported_rather_than_retried ... ok
test runner::tests::a_view_that_holds_nothing_does_not_satisfy_an_invariant_by_being_empty ... ok
test runner::tests::the_runners_clock_advances_on_every_read_so_a_deadline_can_bound_anything ... ok
test runner::tests::ids_come_from_the_suite_and_from_nothing_ambient ... ok
test runner::tests::an_empty_field_set_means_a_row_exists_and_not_that_anything_will_do ... ok
test runner::tests::an_order_over_fewer_than_two_rows_holds_and_does_not_double_as_a_non_emptiness_claim ... ok
test scenario::tests::a_declared_leaf_admits_what_its_type_admits_and_absence_only_where_the_type_permits_it ... ok
test scenario::tests::a_purpose_is_one_line_and_says_something ... ok
test scenario::tests::a_scenario_id_names_the_construct_it_exercises_rather_than_its_position ... ok
test scenario::tests::a_payload_shape_round_trips_through_the_form_a_suite_is_stored_in ... ok
test scenario::tests::a_semantic_reference_renders_the_way_the_design_writes_one ... ok
test scenario::tests::a_suite_format_from_a_later_build_is_refused_rather_than_guessed ... ok
test scenario::tests::a_scenario_id_that_names_no_construct_is_refused ... ok
test scenario::tests::every_binding_aspect_is_in_the_list_that_is_walked_to_produce_them ... ok
test scenario::tests::an_invariant_scenario_is_keyed_by_the_entity_and_the_branch_and_never_by_a_position ... ok
test scenario::tests::a_suite_refuses_a_second_scenario_under_one_id ... ok
test scenario::tests::a_transition_ref_refuses_a_name_no_lifecycle_can_declare ... ok
test scenario::tests::two_scenarios_about_the_same_thing_in_the_same_way_are_one_id ... ok
test scenario::tests::the_ids_of_a_suite_sort_the_way_a_reader_sorts_the_file ... ok
test synthesize::tests::a_refusal_names_the_construct_the_code_and_the_repair ... ok
test scenario::tests::every_scenario_id_reads_back_from_the_form_a_report_prints ... ok
test synthesize::tests::every_refusal_carries_a_distinct_code_in_one_family ... ok
test web::tests::no_comparison_sits_in_a_text_node_mustache ... ok
test witness::tests::a_text_witness_is_its_own_path_so_two_fields_of_one_type_never_agree ... ok
test witness::tests::an_enum_offers_every_variant_it_declares_and_the_first_one_only_once ... ok
test witness::tests::an_integer_leaf_is_never_offered_a_fractional_candidate ... ok
test witness::tests::the_alternatives_for_a_number_are_the_guards_own_literals_either_side ... ok
test witness::tests::the_candidate_count_is_bounded_however_many_fields_a_guard_reads ... ok
test witness::tests::two_uuid_witnesses_differ_and_neither_moves_when_a_third_field_appears ... ok
test web::tests::the_page_calls_nothing_the_player_does_not_return ... ok
test web::tests::the_page_is_specification_neutral ... ok
test evidence::tests::report_readers_preserve_rust_producer_bytes_for_every_supported_suite ... ok

failures:

---- evidence::tests::report_readers_refuse_malformed_and_unsupported_suite_versions stdout ----

thread 'evidence::tests::report_readers_refuse_malformed_and_unsupported_suite_versions' (3064903) panicked at crates/verify/ess-conformance/src/evidence.rs:213:9:
invalid report accepted: [("report reader", true), ("JSON value", true), ("JSON reader", true), ("YAML", true)]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- evidence::tests::report_readers_refuse_more_nonpasses_than_executed_scenarios stdout ----

thread 'evidence::tests::report_readers_refuse_more_nonpasses_than_executed_scenarios' (3064904) panicked at crates/verify/ess-conformance/src/evidence.rs:213:9:
invalid report accepted: [("report reader", true), ("JSON value", true), ("JSON reader", true), ("YAML", true)]

---- evidence::tests::report_readers_refuse_nonpass_count_and_list_disagreement stdout ----

thread 'evidence::tests::report_readers_refuse_nonpass_count_and_list_disagreement' (3064905) panicked at crates/verify/ess-conformance/src/evidence.rs:213:9:
invalid report accepted: [("report reader", true), ("JSON value", true), ("JSON reader", true), ("YAML", true)]

---- evidence::tests::report_readers_refuse_status_claims_that_contradict_the_list stdout ----

thread 'evidence::tests::report_readers_refuse_status_claims_that_contradict_the_list' (3064907) panicked at crates/verify/ess-conformance/src/evidence.rs:213:9:
invalid report accepted: [("report reader", true), ("JSON value", true), ("JSON reader", true), ("YAML", true)]

---- evidence::tests::report_readers_refuse_unknown_report_formats stdout ----

thread 'evidence::tests::report_readers_refuse_unknown_report_formats' (3064908) panicked at crates/verify/ess-conformance/src/evidence.rs:213:9:
invalid report accepted: [("report reader", true), ("JSON value", true), ("JSON reader", true), ("YAML", true)]

---- evidence::tests::report_readers_refuse_nonpass_entries_without_a_known_nonpass_status stdout ----

thread 'evidence::tests::report_readers_refuse_nonpass_entries_without_a_known_nonpass_status' (3064906) panicked at crates/verify/ess-conformance/src/evidence.rs:213:9:
invalid report accepted: [("report reader", true), ("JSON value", true), ("JSON reader", true), ("YAML", true)]


failures:
    evidence::tests::report_readers_refuse_malformed_and_unsupported_suite_versions
    evidence::tests::report_readers_refuse_more_nonpasses_than_executed_scenarios
    evidence::tests::report_readers_refuse_nonpass_count_and_list_disagreement
    evidence::tests::report_readers_refuse_nonpass_entries_without_a_known_nonpass_status
    evidence::tests::report_readers_refuse_status_claims_that_contradict_the_list
    evidence::tests::report_readers_refuse_unknown_report_formats

test result: FAILED. 61 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p ess-conformance --lib`
```

## 4. Final green runs and actual counts

All Cargo commands used RUSTC_WRAPPER=/usr/bin/sccache, CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, and CARGO_PROFILE_TEST_DEBUG=0. TMPDIR was the assigned scratch's tmp directory. CARGO_TARGET_DIR was never set.

Unchanged-base command: cargo test --locked -p ess-conformance
Exit: 0
Raw log: target/review-boundaries-1/review-report-reader-validation/baseline.log

The base runner's summary lines, in library, authored, elapsed, execution, faults, halt, suite, synthesis, witness, and doctest order:

```text
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Final command: cargo test --locked -p ess-conformance
Exit: 0
Raw log: target/review-boundaries-1/review-report-reader-validation/green-final.log

The following is the final test runner's output verbatim; compilation lines remain in the raw log:

```text
running 67 tests
test decision::tests::a_decision_reads_its_two_other_cases_as_neither_satisfied_nor_the_other ... ok
test decision::tests::a_refusal_renders_the_predicate_the_command_and_every_reason ... ok
test evidence::tests::a_standalone_report_carries_every_field_an_adapter_needs ... ok
test decision::tests::exactly_one_reason_says_another_candidate_would_help ... ok
test evidence::tests::report_readers_refuse_more_nonpasses_than_executed_scenarios ... ok
test evidence::tests::the_closed_report_round_trips_with_identical_canonical_bytes ... ok
test evidence::tests::report_readers_do_not_guess_a_producer_from_status_vocabulary ... ok
test evidence::tests::unknown_report_fields_are_refused ... ok
test faulty::tests::a_fault_is_injected_into_the_system_that_declares_what_it_breaks ... ok
test evidence::tests::report_readers_refuse_nonpass_count_and_list_disagreement ... ok
test faulty::tests::every_fault_says_what_it_is_and_where_it_goes ... ok
test faulty::tests::no_two_faults_claim_the_same_scenario ... ok
test faulty::tests::only_the_two_faults_the_boundary_cannot_express_are_injected_in_the_implementation ... ok
test input::tests::a_primitive_refuses_a_node_of_the_wrong_shape_rather_than_coercing_it ... ok
test input::tests::every_primitive_projects_to_the_one_fact_value_that_can_hold_it ... ok
test go::tests::every_go_file_is_in_the_package_the_readme_names ... ok
test input::tests::shape_errors_render_one_per_line_and_name_the_input_root_by_name ... ok
test report::tests::a_quoted_input_reads_as_the_call_that_was_made ... ok
test go::tests::the_runner_is_a_constant_and_only_the_suite_moves ... ok
test report::tests::a_scenario_status_is_the_strongest_of_its_checks_and_a_contradiction_outranks_everything ... ok
test report::tests::a_diagnostic_answers_all_five_of_the_questions_a_failure_has_to_answer ... ok
test evidence::tests::report_readers_preserve_go_producer_bytes_and_historical_nonpass_counts ... ok
test report::tests::an_unsupported_scenario_makes_the_run_fail_rather_than_look_like_a_pass ... ok
test report::tests::every_check_code_has_a_distinct_name_and_a_rule_sentence ... ok
test runner::tests::a_count_with_neither_bound_is_a_suite_defect_and_not_a_satisfied_assertion ... ok
test runner::tests::a_count_is_the_half_of_an_ordering_claim_that_says_the_rows_were_there ... ok
test runner::tests::a_declared_order_is_checked_on_adjacent_rows_and_the_next_key_breaks_a_tie ... ok
test runner::tests::a_nested_row_binds_the_paths_a_predicate_spells ... ok
test runner::tests::a_position_in_a_view_that_declares_no_order_is_a_suite_defect ... ok
test runner::tests::a_position_names_both_ends_and_a_row_that_is_not_there_is_not_a_match ... ok
test runner::tests::a_view_that_holds_nothing_does_not_satisfy_an_invariant_by_being_empty ... ok
test runner::tests::an_empty_field_set_means_a_row_exists_and_not_that_anything_will_do ... ok
test runner::tests::an_order_over_fewer_than_two_rows_holds_and_does_not_double_as_a_non_emptiness_claim ... ok
test runner::tests::a_predicate_a_row_cannot_answer_is_reported_rather_than_retried ... ok
test runner::tests::a_ranking_key_a_row_does_not_publish_is_undecidable_rather_than_out_of_order ... ok
test evidence::tests::report_readers_refuse_unknown_report_formats ... ok
test runner::tests::the_runners_clock_advances_on_every_read_so_a_deadline_can_bound_anything ... ok
test runner::tests::ids_come_from_the_suite_and_from_nothing_ambient ... ok
test scenario::tests::a_declared_leaf_admits_what_its_type_admits_and_absence_only_where_the_type_permits_it ... ok
test scenario::tests::a_purpose_is_one_line_and_says_something ... ok
test scenario::tests::a_scenario_id_names_the_construct_it_exercises_rather_than_its_position ... ok
test scenario::tests::a_payload_shape_round_trips_through_the_form_a_suite_is_stored_in ... ok
test scenario::tests::a_suite_refuses_a_second_scenario_under_one_id ... ok
test scenario::tests::an_invariant_scenario_is_keyed_by_the_entity_and_the_branch_and_never_by_a_position ... ok
test scenario::tests::a_semantic_reference_renders_the_way_the_design_writes_one ... ok
test scenario::tests::every_binding_aspect_is_in_the_list_that_is_walked_to_produce_them ... ok
test scenario::tests::a_scenario_id_that_names_no_construct_is_refused ... ok
test scenario::tests::two_scenarios_about_the_same_thing_in_the_same_way_are_one_id ... ok
test scenario::tests::every_scenario_id_reads_back_from_the_form_a_report_prints ... ok
test scenario::tests::a_transition_ref_refuses_a_name_no_lifecycle_can_declare ... ok
test scenario::tests::a_suite_format_from_a_later_build_is_refused_rather_than_guessed ... ok
test witness::tests::a_text_witness_is_its_own_path_so_two_fields_of_one_type_never_agree ... ok
test witness::tests::an_enum_offers_every_variant_it_declares_and_the_first_one_only_once ... ok
test synthesize::tests::a_refusal_names_the_construct_the_code_and_the_repair ... ok
test witness::tests::an_integer_leaf_is_never_offered_a_fractional_candidate ... ok
test synthesize::tests::every_refusal_carries_a_distinct_code_in_one_family ... ok
test web::tests::no_comparison_sits_in_a_text_node_mustache ... ok
test scenario::tests::the_ids_of_a_suite_sort_the_way_a_reader_sorts_the_file ... ok
test witness::tests::the_alternatives_for_a_number_are_the_guards_own_literals_either_side ... ok
test evidence::tests::report_readers_refuse_malformed_and_unsupported_suite_versions ... ok
test witness::tests::two_uuid_witnesses_differ_and_neither_moves_when_a_third_field_appears ... ok
test web::tests::the_page_calls_nothing_the_player_does_not_return ... ok
test witness::tests::the_candidate_count_is_bounded_however_many_fields_a_guard_reads ... ok
test evidence::tests::report_readers_refuse_nonpass_entries_without_a_known_nonpass_status ... ok
test web::tests::the_page_is_specification_neutral ... ok
test evidence::tests::report_readers_refuse_status_claims_that_contradict_the_list ... ok
test evidence::tests::report_readers_preserve_rust_producer_bytes_for_every_supported_suite ... ok

test result: ok. 67 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/authored.rs (target/debug/deps/authored-8632a10f4af9b7c5)

running 47 tests
test a_halt_of_a_listing_the_model_calls_eventual_retries_because_the_model_said_so ... ok
test a_name_a_closed_set_does_not_have_is_refused_with_the_set ... ok
test a_domain_the_model_does_not_declare_is_refused_by_name ... ok
test a_halt_after_no_rows_at_all_is_refused_rather_than_compiled ... ok
test a_document_that_is_not_one_is_refused_rather_than_read_as_an_empty_scenario ... ok
test a_format_this_build_does_not_implement_is_refused_before_anything_is_read ... ok
test a_field_the_surface_does_not_declare_is_refused_by_name ... ok
test a_declared_field_nothing_supplies_is_refused_by_name ... ok
test a_command_the_model_does_not_declare_is_refused_by_name ... ok
test a_claim_the_timelines_own_instants_contradict_is_refused ... ok
test a_positional_claim_takes_the_order_from_the_view_rather_than_from_the_author ... ok
test a_bounded_negative_that_forbids_no_event_is_refused ... ok
test a_halt_claimed_of_a_listing_with_no_declared_order_is_refused_by_the_code_that_already_says_so ... ok
test a_halt_stated_beside_another_claim_is_two_assertions_filed_as_one ... ok
test a_predicate_reading_something_the_view_does_not_publish_is_refused ... ok
test a_halt_compiles_to_a_step_of_its_own_and_not_to_a_claim_about_rows ... ok
test a_position_in_a_view_that_declares_no_order_is_refused ... ok
test a_reference_where_the_suite_compares_a_value_it_carries_is_refused ... ok
test a_state_the_lifecycle_does_not_declare_is_refused_as_a_state_and_not_as_a_variant ... ok
test a_scenario_that_runs_nothing_is_refused_rather_than_counted_as_a_check ... ok
test a_value_read_off_an_event_nothing_required_is_refused ... ok
test a_scenario_compiles_to_the_id_the_domain_and_the_name_make ... ok
test a_window_of_no_seconds_is_refused_rather_than_compiled_into_a_check_that_cannot_fail ... ok
test a_timeline_whose_instants_do_not_ascend_is_refused ... ok
test a_value_the_declared_type_does_not_admit_is_refused_where_it_sits ... ok
test a_view_the_model_does_not_declare_is_refused_by_name ... ok
test a_window_measured_from_an_instant_nothing_marked_is_refused_with_the_ones_that_are ... ok
test an_event_the_model_does_not_declare_is_refused_by_name ... ok
test an_elapsed_claim_compiles_to_the_four_steps_that_carry_it_and_they_come_before_the_act ... ok
test an_actor_the_specification_does_not_grant_the_command_is_refused ... ok
test an_entity_the_model_does_not_declare_is_refused_by_name ... ok
test an_instance_bound_to_a_field_that_cannot_hold_an_identity_is_refused ... ok
test an_error_the_model_does_not_declare_is_refused_by_name ... ok
test a_window_that_states_other_than_one_bound_is_refused ... ok
test an_assertion_that_states_other_than_one_claim_is_refused ... ok
test an_actor_the_model_does_not_declare_is_refused_by_name ... ok
test an_act_cannot_open_a_window_at_its_own_instant ... ok
test an_instance_named_before_anything_binds_it_is_refused ... ok
test two_files_naming_one_scenario_are_refused_rather_than_one_displacing_the_other ... ok
test the_order_the_files_are_handed_over_in_does_not_reach_the_result ... ok
test the_steps_are_the_vocabulary_a_generated_scenario_already_uses ... ok
test an_outcome_the_command_does_not_declare_is_refused_with_the_ones_it_does ... ok
test an_instance_the_arrangement_does_not_declare_is_refused_by_name ... ok
test one_name_for_two_instants_is_refused_rather_than_read_as_the_later_one ... ok
test the_committed_billing_suite_holds_the_authored_scenario_beside_the_generated_ones ... ok
test every_cause_is_reachable_from_a_document ... ok
test two_compilations_of_one_file_produce_identical_bytes ... ok

test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/elapsed.rs (target/debug/deps/elapsed-6fd40fe27025c9c1)

running 7 tests
test a_deadline_the_target_ran_past_fails_the_within_claim ... ok
test a_target_that_holds_the_window_and_reports_it_passes ... ok
test an_event_published_inside_the_window_fails_the_bounded_negative_and_nothing_else ... ok
test a_target_with_no_clock_reports_unsupported_and_the_run_fails ... ok
test a_target_whose_clock_never_moves_fails_rather_than_being_read_as_having_waited ... ok
test a_window_opened_at_an_instant_nothing_marked_is_a_suite_defect_and_not_a_failed_implementation ... ok
test two_runs_over_one_window_produce_byte_identical_reports ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/execution.rs (target/debug/deps/execution-113b2a96108d91d1)

running 12 tests
test an_eventual_assertion_asks_again_within_a_deadline_and_never_sleeps ... ok
test a_target_that_cannot_expose_an_observation_fails_the_run_rather_than_skipping_it ... ok
test a_scenario_whose_input_no_longer_reaches_its_branch_fails_with_a_diagnostic_naming_the_defect ... ok
test a_view_answered_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test a_value_of_the_wrong_declared_type_is_caught_by_the_same_check_as_a_missing_one ... ok
test an_event_missing_a_field_it_declares_is_named_leaf_by_leaf_rather_than_reported_as_absent ... ok
test an_eventual_view_is_read_again_and_a_read_your_writes_view_is_not ... ok
test every_scenario_checked_something_and_no_family_of_them_was_silently_empty ... ok
test a_read_your_writes_view_is_not_quietly_read_at_current_when_no_token_came_back ... ok
test every_scenario_the_billing_specification_obliges_passes_against_the_reference_implementation ... ok
test a_view_assertion_names_the_instance_the_scenario_created_rather_than_any_row ... ok
test two_runs_of_one_suite_against_one_target_produce_byte_identical_reports ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/faults.rs (target/debug/deps/faults-6e3f179538f17d56)

running 11 tests
test a_fault_nothing_catches_is_recorded_rather_than_quietly_dropped ... ok
test dropping_one_binding_leaves_the_other_two_green ... ok
test the_diagnostic_of_a_caught_fault_names_the_defect_rather_than_reporting_that_something_broke ... ok
test a_wrong_mapping_is_invisible_to_a_target_that_cannot_show_its_invocations ... ok
test every_fault_that_could_be_a_boundary_perturbation_is_one ... ok
test the_widest_blast_radius_is_scenarios_that_could_not_be_arranged_rather_than_extra_verdicts ... ok
test each_specification_is_passed_in_full_by_the_implementation_written_from_it ... ok
test two_runs_against_one_faulty_target_produce_byte_identical_reports ... ok
test a_fault_does_not_simply_break_everything ... ok
test each_fault_fails_the_scenario_that_exists_to_catch_it ... ok
test a_faults_blast_radius_is_accounted_for ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s

     Running tests/halt.rs (target/debug/deps/halt-d3dad7d92bc1f43c)

running 7 tests
test a_halt_of_an_eventual_listing_is_asked_again_while_the_projection_catches_up ... ok
test a_target_whose_producer_stops_when_the_reader_does_passes ... ok
test a_target_that_cannot_read_a_row_at_a_time_reports_unsupported_and_the_run_fails ... ok
test a_listing_that_ran_out_before_the_reader_stopped_it_is_not_a_halt ... ok
test two_runs_over_one_halt_claim_produce_byte_identical_reports ... ok
test retrying_does_not_rescue_a_producer_that_never_stops ... ok
test a_target_that_reads_the_whole_listing_fails_rather_than_being_read_as_having_stopped ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/suite.rs (target/debug/deps/suite-eb757c7c3ad82a6d)

running 14 tests
test a_suite_naming_something_that_is_not_an_ess_name_is_refused_while_it_is_read ... ok
test the_scan_for_a_clock_finds_one_and_does_not_find_a_word_that_merely_ends_in_a_banned_token ... ok
test a_suite_parses_from_text_alone_without_an_ir ... ok
test a_count_and_a_position_read_back_as_what_a_runner_in_another_language_must_read ... ok
test the_steps_a_binding_and_an_invariant_need_survive_being_read_back_from_text ... ok
test every_scenario_id_the_billing_model_can_produce_reads_back ... ok
test the_scenario_ids_appear_in_the_file_in_the_order_a_sorted_key_list_would_be ... ok
test the_step_vocabulary_expresses_the_worked_example_from_section_ten ... ok
test a_suite_serialised_in_one_process_resolves_in_another ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test the_dependency_set_names_a_type_no_derived_from_would_have_mentioned ... ok
test the_suite_records_the_same_model_digest_the_projections_do ... ok
test inserting_one_outcome_re_keys_nothing_around_it ... ok
test serialising_a_suite_twice_produces_byte_identical_json ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/synthesis.rs (target/debug/deps/synthesis-f2b457a2201a7d7c)

running 49 tests
test a_binding_whose_branch_the_event_decides_refuses_the_flow_and_still_checks_the_mapping ... ok
test a_command_that_declares_no_wrong_state_answer_is_refused_by_name_beside_its_scenario ... ok
test a_guard_no_candidate_can_satisfy_is_refused_with_the_number_tried ... ok
test a_filter_reading_something_no_scenario_knows_refuses_rather_than_guessing ... ok
test a_parameterised_view_is_queried_with_the_value_the_scenario_put_in_the_row ... ok
test a_command_that_accepts_a_wrong_state_is_asserted_as_accepting_rather_than_refusing ... ok
test a_state_reached_only_through_a_branch_no_input_reaches_is_refused_rather_than_arranged ... ok
test a_component_nothing_declares_is_refused_by_name ... ok
test a_binding_that_drops_its_failures_refuses_that_check_and_names_the_reason ... ok
test a_read_your_writes_view_filled_by_the_command_that_ran_is_asserted_to_hold_a_row ... ok
test a_binding_mapping_names_the_source_the_document_wrote_and_not_its_same_typed_sibling ... ok
test a_binding_that_retries_forces_one_failure_and_still_requires_the_consequence ... ok
test an_entity_nothing_creates_cannot_be_acted_on_and_says_so ... ok
test a_value_object_nothing_observable_holds_keeps_a_refusal_naming_what_would_close_it ... ok
test a_binding_flow_is_proved_through_the_event_the_invoked_command_publishes ... ok
test a_move_is_observed_through_the_view_the_state_it_left_is_filtered_on ... ok
test a_synthesised_count_is_a_floor_the_scenario_arranged_and_never_a_ceiling ... ok
test a_scenario_that_moves_an_instance_names_the_one_an_earlier_step_created ... ok
test a_declared_order_is_asserted_against_two_rows_the_scenario_arranged_itself ... ok
test an_undecidable_guard_refuses_and_does_not_spend_the_candidate_budget ... ok
test a_declared_error_is_asserted_by_name_and_never_by_an_invented_payload ... ok
test an_order_the_specification_cannot_put_two_rows_under_is_refused_and_not_asserted ... ok
test a_value_objects_own_invariants_are_read_at_every_field_position_a_view_holds_one ... ok
test a_view_is_asserted_in_the_block_its_own_consistency_decides ... ok
test a_view_that_does_not_hold_the_instance_yet_is_not_asked_about_its_invariants ... ok
test a_view_the_entity_has_not_reached_yet_is_asserted_to_exclude_the_instance_by_name ... ok
test a_move_that_is_illegal_in_a_state_is_attempted_with_the_input_that_would_have_worked ... ok
test an_outcome_that_updates_an_instance_acts_on_one_the_scenario_created ... ok
test an_event_assertion_carries_the_declared_shape_and_exactly_the_values_the_payload_determines ... ok
test a_synthesised_suite_survives_being_written_and_read_back ... ok
test an_invariant_over_a_field_no_view_publishes_refuses_rather_than_being_dropped ... ok
test an_at_least_once_binding_delivers_the_event_twice_and_requires_no_count ... ok
test an_illegal_move_requires_the_branch_and_the_declared_error_rather_than_merely_failing ... ok
test a_binding_that_escalates_requires_the_event_the_escalation_declares ... ok
test every_declared_transition_has_a_scenario_that_proves_it_can_occur ... ok
test every_declared_outcome_is_either_a_scenario_or_a_named_refusal_or_asserted_by_the_state_family ... ok
test every_command_names_an_instance_an_earlier_step_of_the_same_scenario_bound ... ok
test the_failure_control_is_armed_after_the_arrangement_and_before_the_command_that_triggers_it ... ok
test an_actor_is_named_only_where_the_specification_grants_the_command ... ok
test the_dependency_set_names_the_types_the_scenario_is_made_of ... ok
test a_suite_for_one_component_holds_only_what_that_component_realises ... ok
test the_refusal_branch_asserts_that_no_event_the_specification_declares_occurred ... ok
test an_outcome_no_input_decides_is_reached_by_injection_and_by_nothing_else ... ok
test an_invariant_is_asserted_against_every_view_that_publishes_what_it_reads ... ok
test the_input_a_scenario_sends_is_re_decided_against_the_guard_it_claims_to_reach ... ok
test a_whole_system_suite_does_not_mention_a_component ... ok
test each_example_synthesises_the_families_its_specification_declares ... ok
test every_clause_of_every_binding_is_either_a_scenario_or_a_named_refusal ... ok
test synthesising_the_same_specification_twice_produces_byte_identical_output ... ok

test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running tests/witness.rs (target/debug/deps/witness-273dca67f5f69494)

running 24 tests
test a_newtype_is_transparent_so_a_deep_path_reaches_through_it_without_a_segment ... ok
test an_absent_optional_binds_nothing_rather_than_binding_a_default ... ok
test a_refuted_guard_carries_the_leaf_and_the_value_that_refuted_it ... ok
test a_refusal_names_the_predicate_the_command_and_the_path ... ok
test a_candidate_carrying_a_field_no_type_declares_is_refused ... ok
test a_candidate_missing_a_required_field_is_refused_before_any_guard_is_read ... ok
test a_path_landing_on_an_aggregate_is_unevaluable_by_construction ... ok
test a_list_a_map_and_a_union_bind_no_fact_because_no_fact_path_can_name_one ... ok
test a_disjunction_one_of_whose_branches_holds_is_satisfied_despite_an_undecidable_branch ... ok
test a_scalar_of_the_wrong_shape_is_refused_rather_than_coerced ... ok
test a_path_into_a_list_or_a_union_names_the_aggregate_rather_than_the_missing_element ... ok
test an_absent_optional_is_unevaluable_but_says_a_candidate_could_repair_it ... ok
test equality_over_two_texts_is_decided_even_though_ordering_them_is_not ... ok
test ordering_across_two_types_is_unevaluable_not_false ... ok
test a_newtype_is_transparent_when_a_path_is_resolved_as_well_as_when_it_is_projected ... ok
test a_deep_path_no_type_declares_is_unevaluable_and_ess_domain_does_not_refuse_it ... ok
test a_candidate_input_projects_one_fact_per_scalar_leaf ... ok
test a_conjunction_of_two_undecidable_leaves_reports_both ... ok
test otherwise_and_external_are_not_guards_over_the_input ... ok
test the_normative_shape_of_guard_is_decidable_for_both_signs ... ok
test only_an_absent_value_says_another_candidate_would_help ... ok
test the_same_text_ordering_is_decidable_once_a_scale_contains_both_values ... ok
test ordering_two_texts_is_unevaluable_because_an_ess_specification_declares_no_scale ... ok
test unclassified_is_a_drift_alarm_and_no_enumerated_source_trips_it ... ok

test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests ess_conformance

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

| Test lane | Executed before → after | Final exit |
| --- | --- | --- |
| Library | 58 → 67 | 0 |
| authored | 47 → 47 | 0 |
| elapsed | 7 → 7 | 0 |
| execution | 12 → 12 | 0 |
| faults | 11 → 11 | 0 |
| halt | 7 → 7 | 0 |
| suite | 14 → 14 | 0 |
| synthesis | 49 → 49 | 0 |
| witness | 24 → 24 | 0 |
| Doctests | 0 → 0 | 0 |
| Complete package | 229 → 238 | 0 |

The nine added cases belong to the library lane, whose executed count increased by nine. No added case was assigned to an unchanged integration lane. No case was ignored, filtered, or removed.

The intermediate targeted command cargo test --locked -p ess-conformance --lib reran the same unfiltered library lane after implementation, exit 0. Its 58-case baseline comes from the complete package's base library runner; the --lib spelling was not separately executed on the base. Raw log: target/review-boundaries-1/review-report-reader-validation/targeted-green.log.

```text
test result: ok. 67 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Formatter command: cargo fmt -p ess-conformance -- --check
Final exit: 0
Output: empty
Raw log: target/review-boundaries-1/review-report-reader-validation/fmt-final.log

Linter command: cargo clippy --locked -p ess-conformance --all-targets -- -D warnings
Final exit: 0
Raw log: target/review-boundaries-1/review-report-reader-validation/clippy-final.log

```text
    Finished `dev` profile [unoptimized] target(s) in 1.75s
```

The first Clippy invocation exited 101 for needless_pass_by_value in the new assert_reader_preserves test helper. The correction changed the helper and its callers to borrow the report, without changing assertions. Strict Clippy, the complete package suite, and the formatter check were rerun after that correction. The initial diagnostic remains at target/review-boundaries-1/review-report-reader-validation/clippy.log; the initial complete green run remains at target/review-boundaries-1/review-report-reader-validation/green.log.

Resource measurements: the original 20 GiB launch floor blocked initial builds. Under the coordinator's recorded resource-revision.md, Cargo ran exclusively with an 8 GiB hard reserve. Free bytes after baseline: 11,348,975,616; after red: 11,310,317,568; after the final Cargo command: 11,155,525,632. Final target allocation: 309,280 KiB. No build directories or caches were removed. The exclusive Cargo slot was released after the last formatter process exited.

Whole-task wall duration, token usage, and tool count are unavailable; no estimates are claimed.

## 5. Exclusions and boundaries

- No new report or suite version, new persisted field, count reinterpretation, or producer change: those belong to the coordinated conformance-format migration stories.
- No coverage guarantee, duplicate-scenario ownership assertion, or specification lookup: a closed execution summary cannot establish those facts.
- No source or test edits outside crates/verify/ess-conformance; no coordinator patch was needed.
- No full workspace or site gate: the coordinator owns integration validation. Package source and tests are the assigned scope.

## 6. Outside-worktree writes

None. All raw logs, exit-status files, prepared test text, scope confirmation, graph capture, and this report are within the assigned worktree's target/review-boundaries-1/review-report-reader-validation directory. The already-open implementor lease was heartbeated and remains for coordinator-managed handoff.
