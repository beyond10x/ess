---
format: aep.planning-md/1
id: review-result:review-boundaries-4-infra-adversary-pass-1
kind: review-result
status: active
title: Wave 4 infrastructure IR adversary pass 1
summary: Five independent regression cases pass; no findings at the reviewed implementation.
relations:
- reviews: story:review-infra-ir-invariants
revision: 1
---
unit: story:review-infra-ir-invariants at b7c3258cf8668cb3d5c8a1f01e362c4092292069 plus this pass's test-only additions
verdict: nothing found
cases: executed 265→270, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: none

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
git --no-pager diff --stat
```

Exit: 0. Raw combined output:

```text
 crates/infra/infra-compiler/tests/read.rs      | 140 +++++++++++++++++
 crates/infra/infra-project/tests/round_trip.rs | 200 +++++++++++++++++++++++++
 2 files changed, 340 insertions(+)
```


## 1. Test-only scope

Base reviewed with three dots: 28e97095d9e06c8b4585876a681a5eda5278c1ab...b7c3258cf8668cb3d5c8a1f01e362c4092292069. The complete implementation diff is the subject of this pass. Its privacy, checked transformation and projector transaction boundaries are specified by docs/design/review-infra-ir-invariants.md and the checked-in story acceptance.

The complete patch for this pass is target/review-boundaries-4/adversary-tests.patch. It only appends cases/fixture text to the two existing integration-test files. No preexisting assertion, source, inline production test, manifest, design or planning document was edited.

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
git --no-pager diff --numstat
```

Exit: 0. Raw combined output:

```text
140	0	crates/infra/infra-compiler/tests/read.rs
200	0	crates/infra/infra-project/tests/round_trip.rs
```

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
git status --short
```

Exit: 0. Raw combined output:

```text
 M crates/infra/infra-compiler/tests/read.rs
 M crates/infra/infra-project/tests/round_trip.rs
```


There are no untracked test files omitted from the diff stat. No SDK tree was written during this pass.

## 2. Cases added and isolated execution before suites

All five cases existed before the first test command. Each ran alone with --exact and a runner count of one before any suite run. They were green on first execution; there is no red behavioral finding.

| Case | Current file:line | What it asserts | Current result |
| --- | --- | --- | --- |
| checked_retargeting_remints_a_new_owner_without_invalidating_the_source_handles | crates/infra/infra-compiler/tests/read.rs:387 | A service can be removed after its surviving site is redirected to another same-owner service; the returned owner resolves the new site, a second dangling edit refuses, and all original handles/document remain usable/unchanged | green |
| indirect_environment_and_volume_sites_cannot_lose_their_resolved_targets | crates/infra/infra-compiler/tests/read.rs:424 | Valid envFrom-only and volume-only configmap/secret references, including optional secret references, survive no-op admission and reject target deletion with IrDanglingHandle rather than digest mismatch; retained handles and bytes remain usable | green |
| captured_detached_edits_and_panics_cannot_mutate_existing_owners | crates/infra/infra-compiler/tests/read.rs:493 | A detached clone captured during a successful callback cannot mutate either owner afterward; a callback panic after candidate mutation leaves the source document/digest and all six source handles intact | green |
| admitted_patches_preserve_other_fields_for_statefulsets_and_daemonsets | crates/infra/infra-project/tests/round_trip.rs:378 | Real strategic patches preserve complete sidecar content, existing requests/env/image/liveness/startup while adding stated TCP readiness and limits; StatefulSet replica/PDB changes and DaemonSet unchanged replica semantics agree with bundle recompilation and simulated outcomes | green |
| same_named_workloads_in_two_namespaces_reach_distinct_budget_fixed_points | crates/infra/infra-project/tests/round_trip.rs:468 | Equal workload names in two namespaces retain distinct patch/object slots; both induced budget gaps close through emitted files, and projecting the rebuilt result emits no further patch/object | green |

Runtime ownership cases use Observation -> compile -> public try_transform -> read_document admission. No unchecked owner or foreign-owner handle is fabricated. Projection cases call public project, apply its actual emitted patch/object files through the preexisting test applier, then rebuild with the actual observation/compiler route. The four Change classes are exercised across the two new projection cases; the independent namespace case checks both fixed-point convergence and idempotence. These are local synthetic observations, with no cluster or executor.

Exact first isolated results:

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-compiler --test read checked_retargeting_remints_a_new_owner_without_invalidating_the_source_handles -- --exact
```

Exit: 0. Raw combined output:

```text
   Compiling infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-compiler)
    Finished `test` profile [unoptimized] target(s) in 0.55s
     Running tests/read.rs (target/debug/deps/read-a2a89aa47a12a0b4)

running 1 test
test checked_retargeting_remints_a_new_owner_without_invalidating_the_source_handles ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s

```

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-compiler --test read indirect_environment_and_volume_sites_cannot_lose_their_resolved_targets -- --exact
```

Exit: 0. Raw combined output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.08s
     Running tests/read.rs (target/debug/deps/read-a2a89aa47a12a0b4)

running 1 test
test indirect_environment_and_volume_sites_cannot_lose_their_resolved_targets ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.01s

```

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-compiler --test read captured_detached_edits_and_panics_cannot_mutate_existing_owners -- --exact
```

Exit: 0. Raw combined output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/read.rs (target/debug/deps/read-a2a89aa47a12a0b4)

running 1 test
test captured_detached_edits_and_panics_cannot_mutate_existing_owners ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.01s

```

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-project --test round_trip admitted_patches_preserve_other_fields_for_statefulsets_and_daemonsets -- --exact
```

Exit: 0. Raw combined output:

```text
   Compiling infra-analyze v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-analyze)
   Compiling infra-spec v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-spec)
   Compiling infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-project)
    Finished `test` profile [unoptimized] target(s) in 2.11s
     Running tests/round_trip.rs (target/debug/deps/round_trip-2ed84414750d9f23)

running 1 test
test admitted_patches_preserve_other_fields_for_statefulsets_and_daemonsets ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.02s

```

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-project --test round_trip same_named_workloads_in_two_namespaces_reach_distinct_budget_fixed_points -- --exact
```

Exit: 0. Raw combined output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/round_trip.rs (target/debug/deps/round_trip-2ed84414750d9f23)

running 1 test
test same_named_workloads_in_two_namespaces_reach_distinct_budget_fixed_points ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s

```


The panic case deliberately catches its own synthetic callback panic and asserts it occurred; it is a successful source-isolation case, not a hidden test failure.

After these isolated runs, rustfmt was restricted to the two assigned test files:
```console
rustfmt --edition 2021 --config skip_children=true crates/infra/infra-compiler/tests/read.rs crates/infra/infra-project/tests/round_trip.rs
```
Exit 0, empty stdout/stderr. No production/module child file was formatted.

## 3. Package suites and checks

Before counts come from the committed implementation report, not a preemptive baseline run: infra-compiler 38 including 16 doctests; infra-project 42; infra-analyze 69; infra-spec 64; ess-cli 52; total 265.

After counts come from the following full five-package runner output:
- infra-compiler: executed 38 -> 41, exit 0; read integration lane 10 -> 13, retained 16 doctests.
- infra-project: executed 42 -> 44, exit 0; round_trip lane 7 -> 9.
- infra-analyze: executed 69 -> 69, exit 0; no case added there.
- infra-spec: executed 64 -> 64, exit 0; no case added there.
- ess-cli: executed 52 -> 52, exit 0; no case added there.
- All five packages: executed 265 -> 270, exit 0.

Unchanged counts in the three unedited packages are expected; both lanes receiving cases increased by their actual additions.

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-compiler -p infra-analyze -p infra-spec -p infra-project -p ess-cli
```

Exit: 0. Raw combined output:

```text
   Compiling infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-compiler)
   Compiling infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-project)
    Finished `test` profile [unoptimized] target(s) in 0.45s
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
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 19 tests
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-15fb048ddae516a0)

running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running unittests src/lib.rs (target/debug/deps/infra_analyze-0ffed922cf4133dc)

running 19 tests
test code::tests::every_code_renders_in_the_diag_namespace_and_the_generated_list_holds_them_all ... ok
test code::tests::severity_orders_info_below_warning_below_error ... ok
test code::tests::the_required_and_optional_reference_codes_disagree_in_severity_by_design ... ok
test code::tests::wire_strings_are_unique_because_two_rules_sharing_one_code_are_indistinguishable_downstream ... ok
test directions::tests::directions_rank_errors_above_warnings_above_info ... ok
test directions::tests::a_clean_candidate_produces_no_direction_and_an_excepted_one_states_its_counts ... ok
test graph::tests::a_graph_node_reads_its_namespace_off_the_key_and_a_cluster_node_has_none ... ok
test graph::tests::a_replicaset_name_derives_its_deployment_only_when_the_hash_confirms_it ... ok
test directions::tests::findings_sharing_a_root_evidence_value_collapse_into_one_direction ... ok
test html::tests::html_escaping_defuses_every_metacharacter_it_claims_to ... ok
test graph::tests::a_mermaid_label_cannot_close_the_quoted_string_it_sits_in ... ok
test html::tests::the_severity_classes_cover_all_three_severities_and_none ... ok
test invariants::tests::a_minority_is_not_a_majority_and_a_bare_half_is_not_either ... ok
test invariants::tests::every_code_renders_in_the_prop_namespace_and_wire_strings_are_unique ... ok
test properties::tests::a_bare_image_name_has_neither_registry_nor_tag_nor_digest ... ok
test properties::tests::a_digest_pinned_image_reports_the_digest_and_whatever_tag_rides_along ... ok
test properties::tests::a_namespaced_hub_image_has_no_registry_because_team_is_not_a_host ... ok
test properties::tests::a_tagged_image_with_a_registry_port_keeps_both_apart ... ok
test properties::tests::an_image_with_a_registry_port_and_no_tag_is_untagged_not_tagged_5000 ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/analysis.rs (target/debug/deps/analysis-f81ae0c31523e57a)

running 12 tests
test a_cluster_without_majority_uniformity_yields_no_candidate ... ok
test properties_on_an_old_format_bundle_carry_coverage_as_unscanned_not_as_uncovered ... ok
test a_candidate_with_exceptions_reads_as_uniformity_with_exceptions_not_as_violations ... ok
test properties_carry_declared_and_observed_replicas_per_workload ... ok
test the_registry_candidate_names_the_dominant_registry_and_lists_every_exception ... ok
test directions_rank_errors_first_and_lead_with_the_autoscaler_aimed_at_nothing ... ok
test the_directions_text_states_candidate_exceptions_without_prescribing ... ok
test all_three_candidates_are_mined_from_the_committed_observation_in_code_order ... ok
test properties_name_the_budgets_and_autoscalers_covering_each_workload ... ok
test the_html_page_sections_by_namespace_aggregates_pods_and_badges_by_worst_finding ... ok
test the_html_page_writes_out_as_one_self_contained_file ... ok
test the_namespace_filter_scopes_sections_findings_and_directions_alike ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/determinism.rs (target/debug/deps/determinism-22db1f859d880eee)

running 5 tests
test the_scan_sees_a_real_violation_and_ignores_prose_and_substrings ... ok
test the_analysis_uses_no_unordered_map_and_reads_no_clock ... ok
test two_diagnoses_of_one_ir_serialize_byte_identically ... ok
test candidates_directions_and_the_html_page_render_byte_identically_across_two_runs ... ok
test two_graph_constructions_render_byte_identical_documents_and_diagrams ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/diagnosis.rs (target/debug/deps/diagnosis-e32ea7ff2f9732ab)

running 23 tests
test the_new_rules_stay_silent_on_a_bundle_that_did_not_scan_their_kinds ... ok
test a_crashlooping_container_is_an_error_and_a_creating_one_is_not ... ok
test a_container_without_probes_fires_and_the_probed_coredns_container_does_not ... ok
test a_container_without_bounds_fires_and_the_bounded_coredns_container_does_not ... ok
test a_pod_its_workload_expects_ready_fires_and_a_finished_job_pod_does_not ... ok
test a_job_short_of_its_completions_with_failures_fires_and_a_completed_one_does_not ... ok
test a_required_missing_reference_is_an_error_and_an_optional_one_is_info ... ok
test a_multi_replica_workload_without_a_budget_fires_and_a_covered_one_does_not ... ok
test a_pending_claim_fires_and_a_bound_one_does_not ... ok
test a_budget_guarding_nothing_fires_and_the_one_guarding_switchboard_does_not ... ok
test latest_and_untagged_images_fire_and_a_pinned_tag_does_not ... ok
test every_registered_code_fires_at_least_once_on_the_example_observation ... ok
test an_unreferenced_claim_fires_and_the_mounted_one_does_not ... ok
test an_autoscaler_pinned_to_one_size_fires_and_a_real_range_does_not ... ok
test a_selector_matching_nothing_is_diagnosed_and_a_matching_one_is_not ... ok
test a_suspended_cronjob_is_info_and_a_running_one_is_not ... ok
test the_severity_floor_filters_out_exactly_what_is_below_it ... ok
test an_autoscaler_aimed_at_nothing_is_an_error_and_an_aimed_one_is_not ... ok
test one_replica_is_info_and_two_replicas_or_a_daemonset_are_not ... ok
test repeated_restarts_fire_and_a_stable_container_does_not ... ok
test findings_arrive_sorted_and_each_carries_its_codes_registered_severity ... ok
test two_services_selecting_one_workload_set_are_reported_once_together ... ok
test unreferenced_config_fires_and_referenced_or_token_managed_config_does_not ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/graph.rs (target/debug/deps/graph-b921749062e0b541)

running 10 tests
test a_replicaset_whose_deployment_is_gone_and_a_hashless_pod_both_stay_underived ... ok
test a_pod_whose_scanned_replicaset_is_absent_or_deploymentless_is_handled_exactly ... ok
test on_a_bundle_without_replicasets_the_hash_fallback_derives_and_names_itself ... ok
test a_deployment_pod_is_owned_exactly_through_its_observed_replicaset ... ok
test a_job_pod_chains_to_its_job_and_cronjob_and_a_bare_pod_stays_a_typed_fact ... ok
test every_edge_relation_is_minted_from_the_committed_observation ... ok
test the_selector_edge_carries_the_selector_and_the_env_edge_carries_its_site ... ok
test restricting_to_a_namespace_keeps_its_objects_their_edges_and_the_nodes_they_reach ... ok
test the_mermaid_rendering_groups_by_namespace_and_leaves_the_runtime_layer_to_the_json ... ok
test the_json_document_chains_to_the_ir_it_was_built_from ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/lib.rs (target/debug/deps/infra_compiler-f0c886bf2a1891d1)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/determinism.rs (target/debug/deps/determinism-9ec910889020b138)

running 7 tests
test the_scan_sees_a_real_violation_and_ignores_prose_and_substrings ... ok
test the_digest_is_the_full_sha256_all_64_hex_characters ... ok
test editing_scanned_at_changes_provenance_and_not_the_digest ... ok
test a_semantic_change_does_change_the_digest ... ok
test the_compiler_uses_no_unordered_map_and_reads_no_clock ... ok
test compiling_the_same_observation_twice_yields_byte_identical_documents ... ok
test a_bundle_with_reordered_kinds_and_reordered_item_lists_compiles_to_the_identical_ir ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/read.rs (target/debug/deps/read-473bee8fa98a265a)

running 13 tests
test a_foreign_format_is_refused_before_anything_else_is_believed ... ok
test an_edited_document_is_refused_for_its_digest ... ok
test an_edited_document_with_a_dangling_claim_reports_both_defects_in_one_run ... ok
test the_fixture_mints_every_handle_kind_or_the_round_trip_proves_too_little ... ok
test a_document_that_does_not_read_as_the_shape_is_refused_as_malformed ... ok
test a_hand_written_resolved_claim_is_refused_even_when_its_digest_is_freshly_stamped ... ok
test a_persisted_document_reads_back_into_the_identical_ir ... ok
test checked_retargeting_remints_a_new_owner_without_invalidating_the_source_handles ... ok
test captured_detached_edits_and_panics_cannot_mutate_existing_owners ... ok
test indirect_environment_and_volume_sites_cannot_lose_their_resolved_targets ... ok
test every_handle_lookup_stays_total_after_compile_read_clone_and_checked_transform ... ok
test deleting_any_referenced_target_is_refused_without_changing_the_source_owner ... ok
test privacy_and_noop_transform_preserve_the_frozen_base_writer_document ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/resolution.rs (target/debug/deps/resolution-1a08a56169ecfe84)

running 5 tests
test a_dangling_reference_is_carried_as_a_fact_and_never_refuses_compilation ... ok
test the_unresolved_site_keeps_the_declared_name_so_the_ir_reads_on_its_own ... ok
test volume_claims_and_optional_secret_references_resolve_with_their_flags_kept ... ok
test a_resolved_reference_is_a_handle_whose_lookup_is_total ... ok
test an_absent_service_account_name_resolves_as_default_because_that_is_what_the_kubelet_does ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/infra_project-ea4cefede791f1bd)

running 9 tests
test patch::tests::a_slug_is_read_back_as_namespace_kind_and_name_even_when_the_name_holds_dots ... ok
test patch::tests::a_file_holding_a_container_change_is_strategic_however_it_was_reached ... ok
test patch::tests::the_join_of_two_types_is_the_one_that_can_carry_both ... ok
test patch::tests::the_container_list_carries_the_merge_key_it_is_matched_by ... ok
test project::tests::a_manifest_port_keeps_the_type_it_was_written_as ... ok
test project::tests::a_generated_budget_names_no_uid_because_nothing_has_assigned_one ... ok
test render::tests::only_an_induced_gap_is_marked_so_a_reader_can_tell_it_from_the_clusters_own ... ok
test project::tests::the_nearest_bound_of_a_range_is_the_bound_the_count_is_outside_of ... ok
test project::tests::failed_candidate_admission_records_nothing_and_preserves_the_working_owner ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/determinism.rs (target/debug/deps/determinism-893dd4c6438712f7)

running 6 tests
test the_scan_sees_a_real_violation_and_ignores_prose_and_substrings ... ok
test the_projection_crate_uses_no_unordered_map_and_reads_no_clock ... ok
test every_file_in_the_committed_tree_is_one_the_library_still_produces ... ok
test the_committed_projection_tree_is_what_the_library_produces_right_now ... ok
test shuffling_a_bundles_items_changes_no_byte_of_the_tree ... ok
test two_projections_of_one_specification_and_snapshot_are_byte_identical ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/projection.rs (target/debug/deps/projection-d3013a26367d3b62)

running 18 tests
test a_budget_whose_name_is_taken_is_owed_rather_than_written_over ... ok
test a_false_predicate_is_refused_because_a_condition_names_no_field ... ok
test a_remedy_that_states_only_one_missing_half_leaves_the_whole_gap_owed ... ok
test a_replica_count_above_the_range_is_lowered_to_the_ceiling ... ok
test a_stated_probe_is_written_into_the_container_that_lacks_it ... ok
test two_expectations_that_disagree_leave_one_of_them_refused_rather_than_silently_lost ... ok
test the_same_gap_without_a_stated_value_is_an_obligation_that_names_what_is_missing ... ok
test a_gap_this_projections_own_changes_open_is_marked_as_such_and_closed_in_the_same_tree ... ok
test every_gap_kind_that_needs_a_decision_gets_one_with_the_class_that_names_it ... ok
test every_obligation_names_a_decision_rather_than_repeating_the_gap ... ok
test every_gap_the_snapshot_reports_gets_exactly_one_entry_and_no_gap_is_lost ... ok
test a_resource_gap_is_patched_only_because_the_specification_states_the_quantities ... ok
test a_probe_gap_with_nothing_stated_is_owed_and_says_what_to_write_where ... ok
test the_obligations_document_names_every_gap_the_tree_does_not_close_and_no_others ... ok
test a_missing_disruption_budget_becomes_a_manifest_built_from_the_workloads_own_selector ... ok
test one_object_gets_one_patch_file_and_its_type_is_the_one_that_carries_every_change_in_it ... ok
test a_replica_count_below_the_range_is_raised_to_the_floor_and_nothing_more ... ok
test the_tree_holds_a_summary_an_obligations_list_and_nothing_it_did_not_generate ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/round_trip.rs (target/debug/deps/round_trip-9026362e64fa4a24)

running 9 tests
test apply::tests::a_plain_merge_patch_replaces_a_list_whole_which_is_why_a_container_change_is_not_one ... ok
test apply::tests::a_null_deletes_the_key_it_names_as_rfc_7386_says ... ok
test apply::tests::a_keyed_list_merges_the_entry_it_names_and_leaves_the_rest_alone ... ok
test checked_projection_round_trips_all_four_changes_including_probes_and_induced_budget ... ok
test same_named_workloads_in_two_namespaces_reach_distinct_budget_fixed_points ... ok
test admitted_patches_preserve_other_fields_for_statefulsets_and_daemonsets ... ok
test a_container_patch_emitted_as_a_plain_merge_would_delete_the_containers_it_does_not_name ... ok
test a_corrupted_patch_value_is_caught_and_the_regressed_expectation_is_named ... ok
test applying_the_emitted_tree_closes_every_gap_it_claims_and_moves_nothing_else ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/secrets.rs (target/debug/deps/secrets-da37d14b3f2d0428)

running 2 tests
test a_dangling_secret_reference_is_owed_and_the_obligation_says_why_nothing_can_write_it ... ok
test no_emitted_byte_carries_a_secrets_digest_or_key_name ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running unittests src/lib.rs (target/debug/deps/infra_spec-4bca2e012aa4ac5e)

running 14 tests
test drift::tests::membership_reports_each_side_once_and_nothing_for_a_shared_key ... ok
test drift::tests::the_change_ordering_puts_membership_before_the_fields_of_a_surviving_member ... ok
test raw::tests::an_id_is_lowercase_dashed_and_starts_with_a_letter ... ok
test facts::tests::every_documented_fact_path_parses_and_the_membership_check_agrees_with_the_list ... ok
test simulate::tests::an_undecidable_subject_beside_a_gap_still_reads_false ... ok
test simulate::tests::an_undecidable_subject_beside_only_holding_ones_reads_unknown_not_true ... ok
test raw::tests::the_three_workload_kinds_parse_by_their_ir_spelling_and_nothing_else_does ... ok
test spec::tests::a_workload_label_selector_cannot_select_a_service_or_the_cluster ... ok
test spec::tests::only_cluster_scope_selects_an_expectation_that_names_its_own_subject ... ok
test spec::tests::every_kind_declares_its_wire_name_in_the_generated_vocabulary ... ok
test render::tests::a_verdict_marker_is_three_characters_so_the_columns_line_up ... ok
test render::tests::the_missing_pair_names_only_what_is_missing ... ok
test simulate::tests::a_verdict_is_the_outcome_variant_and_not_a_field_beside_it ... ok
test simulate::tests::an_optional_dangling_reference_is_not_required_and_a_plain_one_is ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/determinism.rs (target/debug/deps/determinism-4a1c75fbfcd914d2)

running 7 tests
test the_scan_sees_a_real_violation_and_ignores_prose_and_substrings ... ok
test nothing_in_this_crate_can_read_a_wall_clock_because_no_expectation_names_a_duration ... ok
test the_desired_state_crate_uses_no_unordered_map_and_reads_no_clock ... ok
test two_simulations_of_one_specification_and_snapshot_are_byte_identical ... ok
test two_drift_reports_of_one_pair_are_byte_identical ... ok
test the_committed_documents_are_what_the_library_produces_right_now ... ok
test shuffling_a_bundles_items_changes_no_byte_of_either_document ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/drift.rs (target/debug/deps/drift-bdf907f0243c4c76)

running 8 tests
test two_snapshots_of_different_clusters_are_refused_rather_than_compared ... ok
test comparing_a_snapshot_with_itself_reports_no_change_at_all ... ok
test a_reference_change_is_only_reported_for_a_holder_present_in_both_snapshots ... ok
test every_change_kind_the_pair_was_built_to_exercise_appears_exactly_where_it_should ... ok
test a_workloads_replica_count_image_and_labels_each_arrive_as_their_own_typed_change ... ok
test a_configuration_change_names_the_keys_and_never_a_value ... ok
test a_pods_churn_is_not_drift_because_drift_is_over_declared_state ... ok
test reordering_a_templates_containers_is_not_a_change_because_containers_compare_by_name ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/simulate.rs (target/debug/deps/simulate-038789899bcfb715)

running 14 tests
test a_scope_naming_an_observed_but_empty_namespace_is_undecidable_not_vacuously_satisfied ... ok
test a_service_with_no_selector_is_undecidable_rather_than_failing_a_resolution_it_never_claimed ... ok
test a_bundle_that_never_scanned_disruption_budgets_is_undecidable_and_not_uncovered ... ok
test one_scope_holding_both_a_contradicted_and_an_undecidable_subject_reads_false ... ok
test a_digest_pinned_image_satisfies_the_pin_expectation_and_a_tagged_one_does_not ... ok
test a_predicate_reads_the_projections_facts_and_a_false_one_carries_the_values_it_read ... ok
test a_gap_beside_an_undecidable_subject_still_decides_the_expectation_false ... ok
test each_undecidable_expectation_on_the_fixture_carries_the_reason_its_name_promises ... ok
test an_optional_dangling_reference_holds_and_a_required_one_does_not ... ok
test the_committed_example_reaches_all_three_verdicts_and_the_counts_are_the_documented_ones ... ok
test an_expectation_the_snapshot_cannot_decide_never_becomes_a_gap ... ok
test every_expectation_kind_holds_somewhere_on_the_fixture_and_fails_somewhere_on_it ... ok
test a_report_names_every_subject_the_scope_selected_including_the_ones_that_held ... ok
test workload_exists_holds_for_each_of_the_three_kinds_and_fails_when_the_kind_is_the_wrong_one ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/spec.rs (target/debug/deps/spec-d27b9540362abae1)

running 21 tests
test a_format_this_build_does_not_read_is_refused_with_its_own_code ... ok
test a_remedy_beside_a_kind_that_never_finds_an_empty_field_is_refused_rather_than_carried ... ok
test a_document_that_does_not_deserialize_is_one_coded_refusal_and_not_a_serde_sentence ... ok
test a_scope_selects_exactly_the_subject_classes_its_shape_can_reach ... ok
test a_probes_remedy_for_a_probe_the_expectation_never_asks_for_is_refused ... ok
test a_predicate_reading_a_fact_the_projection_never_states_is_refused_as_a_typo ... ok
test a_probe_remedy_states_exactly_one_handler_and_neither_is_refused_the_same_way_as_both ... ok
test a_quoted_number_is_refused_as_a_port_name_because_it_is_one ... ok
test a_remedy_that_validates_is_carried_on_the_expectation_and_a_document_without_one_carries_none ... ok
test a_remedy_that_states_nothing_is_refused_because_it_leaves_the_gap_where_it_was ... ok
test a_specification_reads_from_json_too_because_json_is_yaml ... ok
test an_id_that_is_not_an_identifier_is_refused_and_a_dashed_lowercase_one_is_not ... ok
test a_scope_that_cannot_select_the_expectations_subject_is_refused_in_both_directions ... ok
test a_specification_with_four_defects_reports_four_refusals_in_one_run ... ok
test a_specification_with_no_expectations_is_refused_rather_than_read_as_satisfied ... ok
test two_expectations_sharing_an_id_are_refused_because_a_report_names_a_verdict_by_it ... ok
test every_kind_whose_parameters_can_decide_nothing_is_refused_with_one_code ... ok
test the_validated_type_is_only_reachable_through_validation ... ok
test the_committed_example_specification_validates_and_declares_every_kind ... ok
test a_remedy_changes_no_verdict_because_nothing_evaluates_one ... ok
test the_committed_example_specification_simulates_identically_with_and_without_its_remedies ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests infra_analyze

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests infra_compiler

running 16 tests
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 598) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr::model (line 659) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 546) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 570) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 576) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 584) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 590) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 558) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 552) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 564) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 623) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 604) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 619) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 615) ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr::model (line 651) ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 537) ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

   Doc-tests infra_project

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests infra_spec

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```


Initial package formatting check:
Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo fmt --package infra-compiler --package infra-analyze --package infra-spec --package infra-project --package ess-cli --check
```

Exit: 0. Raw combined output:

```text
```


Initial strict Clippy found only a length lint in one newly authored test:
Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p infra-compiler -p infra-analyze -p infra-spec -p infra-project -p ess-cli --all-targets -- -D warnings
```

Exit: 101. Raw combined output:

```text
    Checking infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-compiler)
    Checking infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-project)
error: this function has too many lines (113/100)
   --> crates/infra/infra-project/tests/round_trip.rs:350:1
    |
350 | fn admitted_patches_preserve_other_fields_for_statefulsets_and_daemonsets() {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_lines
    = note: `-D clippy::too-many-lines` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::too_many_lines)]`

error: could not compile `infra-project` (test "round_trip") due to 1 previous error
```


This was a test-authoring lint, not an implementation finding or red runtime case. I extracted that test's identical expectation text into PARTIAL_CONTAINER_EXPECTATIONS without changing assertions, suppressed no lint, and reran the affected case alone:
Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-project --test round_trip admitted_patches_preserve_other_fields_for_statefulsets_and_daemonsets -- --exact
```

Exit: 0. Raw combined output:

```text
   Compiling infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-project)
    Finished `test` profile [unoptimized] target(s) in 0.36s
     Running tests/round_trip.rs (target/debug/deps/round_trip-2ed84414750d9f23)

running 1 test
test admitted_patches_preserve_other_fields_for_statefulsets_and_daemonsets ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.02s

```


Then reran the affected full infra-project package after this fixture-only refactor; it still executed 44 cases:
Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-project
```

Exit: 0. Raw combined output:

```text
   Compiling infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-project)
    Finished `test` profile [unoptimized] target(s) in 0.52s
     Running unittests src/lib.rs (target/debug/deps/infra_project-41d61ffb90a92fc1)

running 9 tests
test patch::tests::the_join_of_two_types_is_the_one_that_can_carry_both ... ok
test patch::tests::a_file_holding_a_container_change_is_strategic_however_it_was_reached ... ok
test patch::tests::a_slug_is_read_back_as_namespace_kind_and_name_even_when_the_name_holds_dots ... ok
test project::tests::a_generated_budget_names_no_uid_because_nothing_has_assigned_one ... ok
test patch::tests::the_container_list_carries_the_merge_key_it_is_matched_by ... ok
test render::tests::only_an_induced_gap_is_marked_so_a_reader_can_tell_it_from_the_clusters_own ... ok
test project::tests::a_manifest_port_keeps_the_type_it_was_written_as ... ok
test project::tests::the_nearest_bound_of_a_range_is_the_bound_the_count_is_outside_of ... ok
test project::tests::failed_candidate_admission_records_nothing_and_preserves_the_working_owner ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/determinism.rs (target/debug/deps/determinism-edb9145cb06310a2)

running 6 tests
test the_scan_sees_a_real_violation_and_ignores_prose_and_substrings ... ok
test the_projection_crate_uses_no_unordered_map_and_reads_no_clock ... ok
test the_committed_projection_tree_is_what_the_library_produces_right_now ... ok
test every_file_in_the_committed_tree_is_one_the_library_still_produces ... ok
test two_projections_of_one_specification_and_snapshot_are_byte_identical ... ok
test shuffling_a_bundles_items_changes_no_byte_of_the_tree ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/projection.rs (target/debug/deps/projection-22acbfaec3b7e6ae)

running 18 tests
test a_false_predicate_is_refused_because_a_condition_names_no_field ... ok
test a_budget_whose_name_is_taken_is_owed_rather_than_written_over ... ok
test a_remedy_that_states_only_one_missing_half_leaves_the_whole_gap_owed ... ok
test a_replica_count_above_the_range_is_lowered_to_the_ceiling ... ok
test two_expectations_that_disagree_leave_one_of_them_refused_rather_than_silently_lost ... ok
test a_stated_probe_is_written_into_the_container_that_lacks_it ... ok
test the_same_gap_without_a_stated_value_is_an_obligation_that_names_what_is_missing ... ok
test a_missing_disruption_budget_becomes_a_manifest_built_from_the_workloads_own_selector ... ok
test a_replica_count_below_the_range_is_raised_to_the_floor_and_nothing_more ... ok
test a_probe_gap_with_nothing_stated_is_owed_and_says_what_to_write_where ... ok
test the_obligations_document_names_every_gap_the_tree_does_not_close_and_no_others ... ok
test a_gap_this_projections_own_changes_open_is_marked_as_such_and_closed_in_the_same_tree ... ok
test a_resource_gap_is_patched_only_because_the_specification_states_the_quantities ... ok
test every_obligation_names_a_decision_rather_than_repeating_the_gap ... ok
test every_gap_the_snapshot_reports_gets_exactly_one_entry_and_no_gap_is_lost ... ok
test one_object_gets_one_patch_file_and_its_type_is_the_one_that_carries_every_change_in_it ... ok
test every_gap_kind_that_needs_a_decision_gets_one_with_the_class_that_names_it ... ok
test the_tree_holds_a_summary_an_obligations_list_and_nothing_it_did_not_generate ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/round_trip.rs (target/debug/deps/round_trip-2ed84414750d9f23)

running 9 tests
test apply::tests::a_null_deletes_the_key_it_names_as_rfc_7386_says ... ok
test apply::tests::a_keyed_list_merges_the_entry_it_names_and_leaves_the_rest_alone ... ok
test apply::tests::a_plain_merge_patch_replaces_a_list_whole_which_is_why_a_container_change_is_not_one ... ok
test checked_projection_round_trips_all_four_changes_including_probes_and_induced_budget ... ok
test same_named_workloads_in_two_namespaces_reach_distinct_budget_fixed_points ... ok
test admitted_patches_preserve_other_fields_for_statefulsets_and_daemonsets ... ok
test a_container_patch_emitted_as_a_plain_merge_would_delete_the_containers_it_does_not_name ... ok
test a_corrupted_patch_value_is_caught_and_the_regressed_expectation_is_named ... ok
test applying_the_emitted_tree_closes_every_gap_it_claims_and_moves_nothing_else ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/secrets.rs (target/debug/deps/secrets-699e04afe41f118f)

running 2 tests
test a_dangling_secret_reference_is_owed_and_the_obligation_says_why_nothing_can_write_it ... ok
test no_emitted_byte_carries_a_secrets_digest_or_key_name ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

   Doc-tests infra_project

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```


Final package-scoped formatting and strict Clippy:
Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo fmt --package infra-compiler --package infra-analyze --package infra-spec --package infra-project --package ess-cli --check
```

Exit: 0. Raw combined output:

```text
```

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p infra-compiler -p infra-analyze -p infra-spec -p infra-project -p ess-cli --all-targets -- -D warnings
```

Exit: 0. Raw combined output:

```text
    Checking infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-project)
    Finished `dev` profile [unoptimized] target(s) in 0.19s
```


Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
git --no-pager diff --check
```

Exit: 0. Raw combined output:

```text
```


No full workspace gate, production mutation, external compile probe or network fetch ran in this pass. All Cargo commands used the assigned environment shown verbatim above. No target override, cache restart/stop or cleanup occurred.

## 4. Findings

Nothing found.

| File:line | Verdict | Origin | What was measured | What reaches it |
| --- | --- | --- | --- | --- |
| None | — | — | All added runtime cases passed | The public compile/transform/project and emitted patch-to-bundle routes described above |

There is no base reproduction or origin claim for a defect, because no finding reproduced. The implementor's nine old privacy failures and two production-mutation failures are its retained records, not findings or mutations from this pass.

## 5. Attacked boundaries and limits

- Same-owner service retargeting, removal and subsequent refusal retained both owners' proper handles.
- Indirect configmap and secret references in envFrom/volumes could not become dangling through checked transformation, including optional references.
- Deeply detached/captured data and a panicking edit callback left the source owner intact.
- StatefulSet and DaemonSet patches preserved untouched containers/fields; numeric and named probe data survived actual patch application and recompilation.
- Same workload names in different namespaces retained distinct output slots and reached a stable second projection.
- Existing sixteen public API doctests, all six-handle tests and frozen infra-ir/1 byte tests ran in the package suite.
- CLI project_kubernetes at src/main.rs:3236 propagates project errors before artifact extraction/output. I found no valid public input that makes one of the four current reference-preserving Change variants fail candidate membership admission. No new CLI test falsely claims to exercise that internal failure branch. The retained inline workbench transaction test ran in the suite; I did not edit it or force production mutations.
- This does not prove cross-owner handle generativity, full original domain-value admission, unresolved-fact derivation, F06 observation completeness, external Rust consumer compatibility or live deployment. Those are outside the brief.
- Canonical compatibility is the retained byte/round-trip package coverage; no separately built historical compiler ran in this adversary pass.
- Two full attacks are the maximum for this unit; this is pass 1 only. This report makes no approval claim.

## 6. Paths, resources and handoff

Authored outside the assigned worktree: none. Tests are confined to the two paths in the leading stat. Report, test patch and verbatim pass logs are under target/review-boundaries-4. Cargo/test temporary output uses this tree's target and its assigned TMPDIR. The coordinator's shared cache endpoint was used as instructed; no cache lifecycle or configuration change was made.

Before builds, df -B1 --output=avail . printed 130067779584 available bytes. Final pre-report observations:

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
df -B1 --output=avail .
```

Exit: 0. Raw combined output:

```text
       Avail
128716374016
```

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
du -sk target
```

Exit: 0. Raw combined output:

```text
1097424	target
```

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
du -sk target/review-boundaries-4
```

Exit: 0. Raw combined output:

```text
436	target/review-boundaries-4
```


Target size includes scratch. These observations precede adding this report and raw-log copies; all are above the 8589934592-byte reserve. No total token/elapsed accounting is available and none is invented.

Writes are relinquished after saving the report/patch/logs and final read-only hashes. The coordinator owns recording, any follow-up, integration and cleanup.

```findings
[]
```

