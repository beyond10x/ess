---
format: aep.planning-md/1
id: review-result:consumer-coverage-source-pass2
kind: review-result
status: active
title: Consumer coverage source adversary pass 2
relations:
- reviews: story:review-consumer-coverage
revision: 1
---
unit: story:review-consumer-coverage source pass 2 — c375e35def175b51a61c259e73b3b00749399539 plus tests-only working tree
verdict: NEEDS-CHANGE
cases: executed 99→103, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths (assigned TMP root and own lease registry; listed in part 6)
needs-coordinator: route these two findings; no third full attack is assigned; implementation, AEP, integration, publication and cleanup remain coordinator-owned

```text
git --no-pager diff --stat
 .../edge/ess-xtask/src/consumer_coverage/tests.rs  | 106 +++++++++++++++++++++
 1 file changed, 106 insertions(+)
```

## 1. Tests-only scope

The only dirty path is crates/edge/ess-xtask/src/consumer_coverage/tests.rs, the reserved cfg(test) owner. Its complete c375 source is an unchanged byte prefix: 106 lines were appended, preserving every existing and prior-adversarial assertion. tests-only.patch records the exact diff. No implementation, classification, baseline, profile, binding, Taskfile, planning or old evidence was edited. The full source archive/readback covers the 1,214-file tested working tree.

## 2. Concrete cases and their first actual outcomes

All four additions existed before any test execution in this pass. Before 99 is the implementor's actual corrected package count (94 main and 5 layout), never a new baseline run. The first command compiled the real package and selected the default-trait case alone. The other three first commands each selected exactly one case from that same actual retained executable. No compilation setup error occurred.

- tests.rs:1114, new default trait method: old finite classifications must refuse a new public API declaration even when its existing parent is an OwnedHelper. First execution:0 passed,1 failed,direct 101; the unchanged classification set admitted the new method.
- tests.rs:1145, signature type macro: either resolve the signature-selected Self::Item dependency or explicitly refuse the unsupported contract. A compiler-checked local Rust implementation using identity_type!(Self::Item) first returned7u8 successfully; the extractor then accepted both u8 and u16 associated declarations with identical c78cbe57… callable hashes. First execution:0 passed,1 failed,direct 101. This is a bounded same-impl fixture, not a current production baseline-transfer incident or cross-impl resolution request.
- tests.rs:1188, expected-panic and surplus results: expected-panic text, a measured benchmark count and a second summary cannot qualify exact ordinary support. First execution:1 passed,direct 0.
- tests.rs:1200, wire literal/reference positions: one escaped definitions reference resolves, schema description changes remain annotations, and literal description/title values still change shape without treating a literal external $ref as a schema reference. First execution:1 passed,direct 0.

Every targeted run selected exactly one case and reported97 filtered,0 ignored. Complete first raw outcomes follow.

### default-trait-first


```json
{
  "argv": [
    "cargo",
    "test",
    "--locked",
    "--offline",
    "-p",
    "ess-xtask",
    "--bin",
    "ess-xtask",
    "consumer_coverage::tests::adversary_pass2_new_default_trait_method_needs_its_own_classification",
    "--",
    "--exact",
    "--test-threads",
    "1"
  ],
  "at": 1788837326.2056863,
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18"
}
```

```json
{
  "at": 1788837346.1220446,
  "elapsed_seconds": 19.916297405026853,
  "exit": 101,
  "pid": 3767780
}
```

stdout SHA256 8b0e01d8a5976acfc7ff6f2e695a7ed85460c32db772293039bc1a0348021326

```text

running 1 test
test consumer_coverage::tests::adversary_pass2_new_default_trait_method_needs_its_own_classification ... FAILED

failures:

---- consumer_coverage::tests::adversary_pass2_new_default_trait_method_needs_its_own_classification stdout ----

thread 'consumer_coverage::tests::adversary_pass2_new_default_trait_method_needs_its_own_classification' (3770048) panicked at crates/edge/ess-xtask/src/consumer_coverage/tests.rs:1138:5:
a new public default trait method is a concrete API addition even when its parent trait is an owned helper
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    consumer_coverage::tests::adversary_pass2_new_default_trait_method_needs_its_own_classification

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 97 filtered out; finished in 0.00s

```

stderr SHA256 8ae96def2270350e2ef39330fdc716078c7460c82c1a530b525f88365d5495d4

```text
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.24
   Compiling zmij v1.0.23
   Compiling serde_json v1.0.151
   Compiling serde_core v1.0.229
   Compiling itoa v1.0.18
   Compiling syn v3.0.4
   Compiling typenum v1.20.1
   Compiling hybrid-array v0.4.14
   Compiling serde v1.0.229
   Compiling serde_derive v1.0.229
   Compiling syn v2.0.119
   Compiling memchr v2.8.3
   Compiling serde_derive_internals v0.29.1
   Compiling crypto-common v0.2.2
   Compiling block-buffer v0.12.1
   Compiling schemars v0.8.22
   Compiling const-oid v0.10.2
   Compiling thiserror v2.0.20
   Compiling digest v0.11.3
   Compiling schemars_derive v0.8.22
   Compiling thiserror-impl v2.0.20
   Compiling cfg-if v1.0.4
   Compiling cpufeatures v0.3.1
   Compiling equivalent v1.0.2
   Compiling dyn-clone v1.0.20
   Compiling hashbrown v0.17.1
   Compiling indexmap v2.14.1
   Compiling sha2 v0.11.0
   Compiling ryu v1.0.23
   Compiling utf8parse v0.2.2
   Compiling unsafe-libyaml v0.2.11
   Compiling anstyle-parse v1.0.0
   Compiling serde_yaml v0.9.34+deprecated
   Compiling ess-primitives v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/crates/specify/ess-primitives)
   Compiling anstyle-query v1.1.5
   Compiling anstyle v1.0.14
   Compiling pulldown-cmark v0.13.4
   Compiling is_terminal_polyfill v1.70.2
   Compiling colorchoice v1.0.5
   Compiling anstream v1.0.0
   Compiling ess-domain v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/crates/specify/ess-domain)
   Compiling strsim v0.11.1
   Compiling unicase v2.9.0
   Compiling anyhow v1.0.104
   Compiling clap_lex v1.1.0
   Compiling bitflags v2.13.1
   Compiling heck v0.5.0
   Compiling pulldown-cmark-escape v0.11.0
   Compiling ess-compiler v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/crates/specify/ess-compiler)
   Compiling clap_derive v4.6.4
   Compiling clap_builder v4.6.6
   Compiling ess-xtask v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/crates/edge/ess-xtask)
   Compiling clap v4.6.6
   Compiling ess-gen v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/crates/generate/ess-gen)
    Finished `test` profile [unoptimized] target(s) in 19.88s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-7e95320043038dad)
error: test failed, to rerun pass `-p ess-xtask --bin ess-xtask`
```

Completion and actual-count record:
```json
{
  "elapsed_seconds": 19.916297405026853,
  "exit": 101,
  "native_copies": 2,
  "resource_stop": null,
  "resources_ok_after_retention": true,
  "source_unchanged": true,
  "stderr_sha256": "8ae96def2270350e2ef39330fdc716078c7460c82c1a530b525f88365d5495d4",
  "stdout_sha256": "8b0e01d8a5976acfc7ff6f2e695a7ed85460c32db772293039bc1a0348021326",
  "summaries": [
    "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 97 filtered out; finished in 0.00s"
  ]
}
```

### signature-macro-first


```json
{
  "argv": [
    "target/debug/deps/ess_xtask-7e95320043038dad",
    "--exact",
    "consumer_coverage::tests::adversary_pass2_type_macro_cannot_hide_a_selected_associated_contract",
    "--test-threads",
    "1"
  ],
  "at": 1788837373.3170714,
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18"
}
```

```json
{
  "at": 1788837373.415084,
  "elapsed_seconds": 0.09795236599165946,
  "exit": 101,
  "pid": 3771806
}
```

stdout SHA256 f25a10b5e3bbbb69aac6398e2b98948bc167c77c1342c23dad11ac4913f1dadd

```text

running 1 test
test consumer_coverage::tests::adversary_pass2_type_macro_cannot_hide_a_selected_associated_contract ... FAILED

failures:

---- consumer_coverage::tests::adversary_pass2_type_macro_cannot_hide_a_selected_associated_contract stdout ----

thread 'consumer_coverage::tests::adversary_pass2_type_macro_cannot_hide_a_selected_associated_contract' (3771807) panicked at crates/edge/ess-xtask/src/consumer_coverage/tests.rs:1174:36:
assertion `left != right` failed: an admitted signature macro must not erase its selected Self::Item dependency
  left: String("c78cbe57d9057c518b6c33433ca285e7e9563c1c39e0874d3ee29c62799897fe")
 right: String("c78cbe57d9057c518b6c33433ca285e7e9563c1c39e0874d3ee29c62799897fe")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    consumer_coverage::tests::adversary_pass2_type_macro_cannot_hide_a_selected_associated_contract

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 97 filtered out; finished in 0.00s

```

stderr SHA256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

```text
```

Completion and actual-count record:
```json
{
  "elapsed_seconds": 0.09795236599165946,
  "exit": 101,
  "native_copies": 1,
  "resource_stop": null,
  "resources_ok_after_retention": true,
  "source_unchanged": true,
  "stderr_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "stdout_sha256": "f25a10b5e3bbbb69aac6398e2b98948bc167c77c1342c23dad11ac4913f1dadd",
  "summaries": [
    "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 97 filtered out; finished in 0.00s"
  ]
}
```

### expected-panic-first


```json
{
  "argv": [
    "target/debug/deps/ess_xtask-7e95320043038dad",
    "--exact",
    "consumer_coverage::tests::adversary_pass2_expected_panic_and_surplus_results_do_not_qualify_support",
    "--test-threads",
    "1"
  ],
  "at": 1788837374.0644066,
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18"
}
```

```json
{
  "at": 1788837374.169003,
  "elapsed_seconds": 0.10453622590284795,
  "exit": 0,
  "pid": 3771914
}
```

stdout SHA256 30d2372dbd0b7adb874963ec3474aac9dc70723ccb73fbb2c54cb4159635e8f9

```text

running 1 test
test consumer_coverage::tests::adversary_pass2_expected_panic_and_surplus_results_do_not_qualify_support ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 0.00s

```

stderr SHA256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

```text
```

Completion and actual-count record:
```json
{
  "elapsed_seconds": 0.10453622590284795,
  "exit": 0,
  "native_copies": 1,
  "resource_stop": null,
  "resources_ok_after_retention": true,
  "source_unchanged": true,
  "stderr_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "stdout_sha256": "30d2372dbd0b7adb874963ec3474aac9dc70723ccb73fbb2c54cb4159635e8f9",
  "summaries": [
    "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 0.00s"
  ]
}
```

### wire-literals-first


```json
{
  "argv": [
    "target/debug/deps/ess_xtask-7e95320043038dad",
    "--exact",
    "consumer_coverage::tests::adversary_pass2_wire_literal_refs_and_escaped_definition_names_keep_their_positions",
    "--test-threads",
    "1"
  ],
  "at": 1788837374.8209934,
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18"
}
```

```json
{
  "at": 1788837374.9201062,
  "elapsed_seconds": 0.09905163501389325,
  "exit": 0,
  "pid": 3771998
}
```

stdout SHA256 780039ee4000845a806e2d670e001a984dfcfc4baff83c789e4eec7ab7414360

```text

running 1 test
test consumer_coverage::tests::adversary_pass2_wire_literal_refs_and_escaped_definition_names_keep_their_positions ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 0.00s

```

stderr SHA256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

```text
```

Completion and actual-count record:
```json
{
  "elapsed_seconds": 0.09905163501389325,
  "exit": 0,
  "native_copies": 1,
  "resource_stop": null,
  "resources_ok_after_retention": true,
  "source_unchanged": true,
  "stderr_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "stdout_sha256": "780039ee4000845a806e2d670e001a984dfcfc4baff83c789e4eec7ab7414360",
  "summaries": [
    "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 97 filtered out; finished in 0.00s"
  ]
}
```

## 3. Affected package after the additions

The package command used --no-fail-fast so the layout target executed despite main-target failure. Main actually ran 98 cases:96 passed and the two new defect cases failed. Layout ran 5 cases and all passed. Total 103 executed,101 passed,2 failed,0 ignored. All 99 prior cases and both new boundary controls passed. Only ess-xtask tests changed, so ess-diff's separate166-case suite was not repeated. No full checker, extraction or mutation was launched.

### affected-package


```json
{
  "argv": [
    "cargo",
    "test",
    "--locked",
    "--offline",
    "-p",
    "ess-xtask",
    "--no-fail-fast"
  ],
  "at": 1788837396.1673257,
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18"
}
```

```json
{
  "at": 1788837439.348609,
  "elapsed_seconds": 43.18122204800602,
  "exit": 101,
  "pid": 3773307
}
```

stdout SHA256 d4b8e046d441fc0e0ab64658b400201d925ba659ddeb4cc3d81e4ca3470fcef3

```text

running 98 tests
test consumer_coverage::tests::adversary_pass2_expected_panic_and_surplus_results_do_not_qualify_support ... ok
test consumer_coverage::tests::adversary_absolute_external_type_is_not_shadowed_by_a_local_module ... ok
test consumer_coverage::tests::adversary_pass2_new_default_trait_method_needs_its_own_classification ... FAILED
test consumer_coverage::tests::cfg_test_is_excluded_without_hiding_production ... ok
test consumer_coverage::tests::adversary_absolute_external_import_keeps_its_external_owner ... ok
test consumer_coverage::tests::consumer_unknown_cfg_is_not_a_default_profile ... ok
test consumer_coverage::tests::adversary_new_public_associated_constant_requires_a_concrete_classification ... ok
test consumer_coverage::tests::adversary_pass2_wire_literal_refs_and_escaped_definition_names_keep_their_positions ... ok
test consumer_coverage::tests::comments_and_whitespace_do_not_change_rust_identity_or_shape ... ok
test consumer_coverage::tests::concrete_consumer_entries_and_full_case_names_are_not_package_aliases ... ok
test consumer_coverage::tests::changed_string_backed_alternative_is_not_hidden_by_wire_string ... ok
test consumer_coverage::tests::finite_unaccepted_eligibility_has_exact_complete_accounting ... ok
test consumer_coverage::tests::adversary_changed_associated_output_type_invalidates_its_consumer_contract ... ok
test consumer_coverage::tests::classified_nonmodel_macro_definition_and_invocation_drift_refuse ... ok
test consumer_coverage::tests::generated_consumer_owner_uses_the_checked_invocation_grammar ... ok
test consumer_coverage::tests::new_member_or_consumer_cannot_inherit_eligibility ... ok
test consumer_coverage::tests::correction_unresolved_self_contracts_do_not_borrow_an_unrelated_trait_member ... ok
test consumer_coverage::tests::ignored_case_and_nested_execution_are_visible_candidates_only ... ok
test consumer_coverage::tests::local_aliases_inline_modules_reexports_and_cycles_are_resolved ... ok
test consumer_coverage::tests::correction_trait_associated_declarations_require_their_own_classifications ... ok
test consumer_coverage::tests::adversary_pass2_type_macro_cannot_hide_a_selected_associated_contract ... FAILED
test consumer_coverage::tests::pending_owner_checkpoint_is_complete_but_never_eligible ... ok
test consumer_coverage::tests::local_declarations_shadow_prelude_leaves_and_generics_refuse ... ok
test consumer_coverage::tests::production_cfg_and_alias_ambiguity_fail_closed ... ok
test consumer_coverage::tests::new_callable_and_target_alternatives_change_exact_inventory ... ok
test consumer_coverage::tests::correction_associated_member_profiles_and_unsupported_forms_remain_closed ... ok
test consumer_coverage::tests::measured_build_profile_refuses_unsupported_target_features_and_wrappers ... ok
test consumer_coverage::tests::declaration_order_and_unknown_representation_grammar_are_not_erased ... ok
test consumer_coverage::tests::correction_qualified_associated_constants_participate_in_declared_array_results ... ok
test consumer_coverage::tests::public_reexport_additions_and_alias_changes_have_concrete_identities ... ok
test consumer_coverage::tests::correction_wrapped_self_projection_requires_a_resolved_associated_owner ... ok
test consumer_coverage::tests::private_optional_tuple_and_enum_members_have_separate_identities ... ok
test consumer_coverage::tests::representation_attributes_invalidate_shape_and_unknown_attributes_refuse ... ok
test consumer_coverage::tests::stage2_cargo_artifact_requires_exact_owner_target_profile_and_completed_build ... ok
test consumer_coverage::tests::stage2_case_engine_lists_filters_and_executes_before_qualifying ... ok
test consumer_coverage::tests::stage2_case_engine_refuses_source_drift_or_missing_actual_result ... ok
test consumer_coverage::tests::correction_absolute_external_groups_renames_and_reexports_preserve_the_owner ... ok
test consumer_coverage::tests::stage2_measured_stable_listing_requires_exact_nonignored_case ... ok
test consumer_coverage::tests::stage2_exact_finite_baseline_plus_behavioral_case_covers_two_pairs ... ok
test consumer_coverage::tests::stage2_current_flags_target_tools_and_complete_configuration_set_are_bound ... ok
test consumer_coverage::tests::stage2_claims_need_cases_and_named_refusal_without_contradictions ... ok
test consumer_coverage::tests::stage2_one_actual_pass_requires_successful_direct_exit ... ok
test consumer_coverage::tests::stage2_swallowed_panic_and_skipped_nested_branches_do_not_qualify ... ok
test consumer_coverage::tests::stage2_new_obligation_can_gain_behavior_without_expanding_initial_unknowns ... ok
test consumer_coverage::tests::stage2_zero_selected_ignored_failing_and_forged_results_refuse ... ok
test consumer_coverage::tests::stage2_classification_refusal_preserves_actual_accounting_diagnostics ... ok
test consumer_coverage::tests::stage2_new_consumer_and_changed_profile_cannot_inherit_unknown ... ok
test consumer_coverage::tests::stage2_new_optional_or_string_backed_member_cannot_inherit_unknown ... ok
test consumer_coverage::tests::unknown_production_macro_cannot_hide_a_declaration ... ok
test consumer_coverage::tests::stage2_unknown_requires_independent_owner_and_closed_manifest ... ok
test consumer_coverage::tests::consumer_identity_separates_body_comments_and_unrelated_entries_from_signatures ... ok
test consumer_coverage::tests::stage2_refusal_counts_only_current_eligible_unknown_cells ... ok
test consumer_coverage::tests::stale_duplicate_ownerless_accepted_and_mandatory_unknown_rows_refuse ... ok
test consumer_coverage::tests::unknown_type_owner_is_a_named_refusal ... ok
test support::tests::a_complete_source_block_is_accepted_without_owning_release_prose ... ok
test consumer_coverage::tests::wire_unknown_keyword_dialect_and_external_references_refuse ... ok
test consumer_coverage::tests::wire_boolean_schemas_and_ordered_literals_are_concrete ... ok
test consumer_coverage::tests::stage2_removed_duplicate_and_missing_pairs_refuse ... ok
test consumer_coverage::tests::wire_definitions_references_and_escaped_property_names_are_separate ... ok
test consumer_coverage::tests::wire_annotations_are_excluded_only_at_schema_positions ... ok
test support::tests::cargo_version_changes_invalidate_only_the_source_block ... ok
test support::tests::command_inventory_requires_a_complete_nonempty_unique_section ... ok
test consumer_coverage::tests::wire_reference_cycle_is_finite_and_descendants_invalidate_parents ... ok
test support::tests::emitted_version_markers_are_parsed_and_must_agree_across_the_projection ... ok
test support::tests::adversary_real_source_version_drift_keeps_release_bytes_independent ... ok
test tests::a_named_but_untagged_version_is_not_an_incomplete_release ... ok
test support::tests::every_material_row_change_is_refused_with_its_location ... ok
test support::tests::html_requires_the_actual_output_root_and_both_nonempty_local_assets ... ok
test support::tests::help_inventory_reads_both_clap_layouts_without_swallowing_neighbor_options ... ok
test support::tests::support_check_is_an_available_maintenance_command ... ok
test support::tests::missing_duplicated_or_reordered_block_markers_refuse ... ok
test tests::a_version_tag_whose_commit_never_reached_main_is_refused ... ok
test tests::an_empty_release_is_refused_by_name ... ok
test tests::a_version_tag_with_no_release_behind_it_is_refused ... ok
test tests::an_undated_release_is_refused_by_name ... ok
test tests::exclusions_cover_only_the_named_subtree ... ok
test tests::generated_paths_must_stay_below_the_projection_root ... ok
test tests::only_bare_version_tags_are_release_tags ... ok
test tests::published_release_tags_come_from_the_json_report ... ok
test tests::release_notes_stop_before_the_next_release ... ok
test tests::the_release_record_is_checked_after_every_release_run ... ok
test tests::the_generated_index_includes_static_site_source ... ok
test tests::workspace_version_comes_only_from_the_workspace_package_table ... ok
test tests::sync_checks_and_reconciles_in_both_directions ... ok
test consumer_coverage::tests::correction_associated_contract_tracks_selected_transitive_declarations_only ... ok
test tests::release_publication_is_evaluated_after_every_dependency_finishes ... ok
test consumer_coverage::tests::file_level_production_cfg_cannot_hide_from_the_graph ... ok
test consumer_coverage::tests::selected_root_reaching_guarded_diagnostic_type_is_not_an_opaque_leaf ... ok
test consumer_coverage::tests::external_module_inside_inline_module_uses_its_semantic_directory ... ok
test consumer_coverage::tests::actual_selected_roots_resolve_without_diagnostic_generated_declarations ... ok
test consumer_coverage::tests::diagnostic_macro_definition_and_invocation_changes_refuse ... ok
test support::tests::adversary_real_docs_ir_marker_is_nested_json_not_visible_marker_text ... ok
test support::tests::adversary_actual_help_missing_target_metadata_cannot_borrow_neighbor_values ... ok
test support::tests::adversary_actual_cli_refusal_is_not_a_successful_support_observation ... ok
test support::tests::adversary_adjacent_readme_is_selected_without_authored_flags ... ok
test support::tests::adversary_front_page_override_replaces_adjacent_readme_for_a_specification_file ... ok
test support::tests::adversary_actual_explicit_site_and_combined_maps_keep_distinct_roots ... ok
test support::tests::adversary_all_real_material_rows_refuse_cell_removal_duplicate_and_order_drift ... ok

failures:

---- consumer_coverage::tests::adversary_pass2_new_default_trait_method_needs_its_own_classification stdout ----

thread 'consumer_coverage::tests::adversary_pass2_new_default_trait_method_needs_its_own_classification' (3773735) panicked at crates/edge/ess-xtask/src/consumer_coverage/tests.rs:1138:5:
a new public default trait method is a concrete API addition even when its parent trait is an owned helper
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- consumer_coverage::tests::adversary_pass2_type_macro_cannot_hide_a_selected_associated_contract stdout ----

thread 'consumer_coverage::tests::adversary_pass2_type_macro_cannot_hide_a_selected_associated_contract' (3773736) panicked at crates/edge/ess-xtask/src/consumer_coverage/tests.rs:1174:36:
assertion `left != right` failed: an admitted signature macro must not erase its selected Self::Item dependency
  left: String("c78cbe57d9057c518b6c33433ca285e7e9563c1c39e0874d3ee29c62799897fe")
 right: String("c78cbe57d9057c518b6c33433ca285e7e9563c1c39e0874d3ee29c62799897fe")


failures:
    consumer_coverage::tests::adversary_pass2_new_default_trait_method_needs_its_own_classification
    consumer_coverage::tests::adversary_pass2_type_macro_cannot_hide_a_selected_associated_contract

test result: FAILED. 96 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 37.46s


running 5 tests
test every_workspace_crate_lives_under_an_area_directory ... ok
test the_path_scan_reads_an_area_qualified_path ... ok
test the_path_scan_excludes_by_root_relative_path_and_reads_published_website_source ... ok
test the_path_scan_finds_at_least_one_path_in_this_repository ... ok
test every_literal_path_naming_a_workspace_crate_exists ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

```

stderr SHA256 796e91cdcc3dc1aa871398e9bc52ddc46a3698d4a0b214364fde8209d8d96a58

```text
   Compiling ess-xtask v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/crates/edge/ess-xtask)
    Finished `test` profile [unoptimized] target(s) in 5.53s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-7e95320043038dad)
error: test failed, to rerun pass `-p ess-xtask --bin ess-xtask`
     Running tests/layout.rs (target/debug/deps/layout-19a7b55d79637f7c)
error: 1 target failed:
    `-p ess-xtask --bin ess-xtask`
```

Completion and actual-count record:
```json
{
  "elapsed_seconds": 43.18122204800602,
  "exit": 101,
  "native_copies": 4,
  "resource_stop": null,
  "resources_ok_after_retention": true,
  "source_unchanged": true,
  "stderr_sha256": "796e91cdcc3dc1aa871398e9bc52ddc46a3698d4a0b214364fde8209d8d96a58",
  "stdout_sha256": "d4b8e046d441fc0e0ab64658b400201d925ba659ddeb4cc3d81e4ca3470fcef3",
  "summaries": [
    "test result: FAILED. 96 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 37.46s",
    "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s"
  ]
}
```

## 4. Findings and reachability

| File:line | Verdict / origin | Finding | What was measured | What reaches it |
|---|---|---|---|---|

| crates/edge/ess-xtask/src/consumer_coverage/consumer.rs:525 | NEEDS-CHANGE / introduced | A newly added public default trait method keeps the old finite consumer classification set because trait methods are skipped as concrete API entries. | The first exact case failed at tests.rs:1138 with direct 101 because the old finite classifications admitted the added method. The parent trait declaration hash changes, but classification admission checks its identity and reason, not a reviewed shape. | Production `run` calls `consumer::extract`; `Inventory::walk` sends each production trait to `trait_declaration`, which explicitly skips TraitItem::Fn. Existing owned public interfaces use this grammar: `TypeEnvironment` at ess-domain/src/expression.rs:77 has default methods at:90/:94, and `Generator` at ess-gen/src/artifact.rs:124 is classified OwnedHelper. Adding a default method need not change implementing types or bound existing method signatures. This test proves the declaration/classification mechanism; it does not claim a newly added production method was compiled or used during this pass. |

| crates/edge/ess-xtask/src/consumer_coverage/consumer.rs:587 | NEEDS-CHANGE / introduced | An admitted type macro in a callable signature hides its Self-associated dependency, so changing that associated type can preserve the callable declaration fingerprint. | The compiler-checked Rust control succeeds, then the first exact case fails at tests.rs:1174 with direct 101: both accepted declarations hash to c78cbe57d9057c518b6c33433ca285e7e9563c1c39e0874d3ee29c62799897fe. | `Inventory::implementation` calls `associated_contract` for each production impl method. Its syn visitor enters the signature but never examines or refuses a type macro token body; `Self::Item` remains inside those tokens. A reviewed unchanged macro definition does not bind a later changed associated declaration. The corrected contract explicitly includes same-impl signature-selected dependencies and refuses unsupported forms. The constructed fixture shows this mechanism gap; no current 55 selected entry uses the demonstrated macro signature, so no current baseline transfer is claimed. |

Both findings are introduced relative to published a0cf3ca8681ce06f6fbdbc988d457b23f2136c04, which has no consumer-coverage implementation. The trait-method omission was also present during pass1; the current correction makes the skip explicit. The macro hole is in the new associated-contract visitor. No old-base build or checkout was performed. These are different signatures from pass1's rust.rs:445 and consumer.rs:471 records; root owns the AEP comparison. The original four pass1 cases and all seven correction cases passed in this package run.

A bounded repair can inventory public trait method identities and explicitly refuse unresolved type macros in callable/selected-associated declarations. This report requests no macro expansion engine or cross-impl Rust type resolution. No repair was applied and no approval is issued.

## 5. Scope checked, retained authority and limits

- Reused only unchanged prior reads of the complete accepted binding, story acceptance, repository instructions, adversary/worktree charter and original extractor/accounting/executor/build source. Read the complete correction delta in rust.rs and consumer.rs, all seven correction tests and every 135 new classification row; the only removed row is conditions_fn and every retained classification is unchanged. Read the new story/wave routing and complete correction-report narrative. Large historical streams are retained inputs from root's complete readback, not replayed or described as fresh pass2 execution.
- The correction report SHA256 0b55a51528c2d791dadf4757ad1cdb3c55c893579e423f028deb861d95b318e7 matches dispatch. Its actual 99-case green, 22-case literal Task checker and exact 93 profiles and 157,122 cells are prior corrected-source observations; this pass does not substitute them for final integration after further correction.
- The fixed absolute path/import controls and associated const/type/qualified/wrapped-Self controls all remain green. Existing accounting, cfg/macro/schema, artifact identity, exact listing/results and source/tool/config refusal tests also remain green. The two new executor/wire controls passed on first execution and again in the affected suite.
- The root-owned baseline remains exactly e005a2e74e067ad51315e594151643b702381dc174bb9af5c355c3eeeb17ad51. No unknown was added or promoted; no new consumer support/refusal claim, F01 attribution, runtime execution or downstream semantic change was made. The separate initial-baseline qualification epic remains outside this selected work.
- Frozen toolchain 1.98.1, exact flags, four empty wrappers, locked/offline Cargo, two jobs and own default target were used. Per-lane resource samples remained below the granted 9,739,689,984 aggregate limit and above 6,442,450,944free; no producer was stopped. The six GiB bounded review floor changes no full-integration floor.
- The first four current executable aliases were independently copied before replacement (61,597,152 logical bytes). Every actual selected executable was copied and read back per lane; complete original stdout/stderr, direct results, launch/environment, source before/after and resource samples remain at the named lane paths. All producer PIDs are absent.
- `execution-readback.json` freshly verifies all five test lanes, every retained independent native copy and the complete source archive. Source-archive production is separately recorded as lane source-archive, with actual 0 and complete raw streams. The source archive contains 10,050,941 bytes.
- Final seal inventory/reader cover the full unit target and all five named TMP roots, using native byte names, literal links, full regular payload hashes, metadata and device/inode hardlink identity. Only finite precreated self-written seal bookkeeping is excluded and separately pinned. Reading/census is retention, not an agent independence claim. Root holds the unit unchanged for this readback. Only this pass's own lease is released.

Selected effective environment:
```json
{
  "set": {
    "CARGO_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-2/cargo-home",
    "CARGO": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-coverage/rust-toolchain/bin/cargo",
    "RUSTC": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-coverage/rust-toolchain/bin/rustc",
    "RUSTDOC": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-coverage/rust-toolchain/bin/rustdoc",
    "RUSTFLAGS": "-C link-arg=-fuse-ld=lld",
    "CARGO_BUILD_JOBS": "2",
    "CARGO_NET_OFFLINE": "true",
    "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_DEBUG": "0",
    "CARGO_PROFILE_DEV_INCREMENTAL": "false",
    "CARGO_PROFILE_TEST_INCREMENTAL": "false",
    "RUSTC_WRAPPER": "",
    "RUSTC_WORKSPACE_WRAPPER": "",
    "CARGO_BUILD_RUSTC_WRAPPER": "",
    "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER": "",
    "TMPDIR": "/home/timo/.cache/ess-w18-consumer-source-pass2-tmp",
    "GOCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-2/go-cache",
    "GOMODCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-2/go-mod-cache",
    "XDG_CACHE_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-2/xdg-cache",
    "XDG_CONFIG_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-2/xdg-config",
    "XDG_DATA_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-2/xdg-data",
    "XDG_RUNTIME_DIR": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-2/xdg-runtime",
    "XDG_STATE_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-2/xdg-state",
    "PATH": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-coverage/rust-toolchain/bin:/home/timo/.local/bin:/home/timo/.deno/bin:/home/timo/.codex/packages/standalone/releases/0.153.4-x86_64-unknown-linux-musl/codex-path:/home/timo/.codex/tmp/arg0/codex-arg0EIPBn9:/home/linuxbrew/.linuxbrew/Caskroom/codex/0.153.4/codex-path:/home/timo/.local/bin:/home/timo/.deno/bin:/home/timo/.codex/tmp/arg0/codex-arg0dAS0R8:/home/linuxbrew/.linuxbrew/Caskroom/codex/0.153.4/codex-path:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.opencode/bin:/home/timo/.fly/bin:/home/timo/.cargo/bin:/home/timo:/home/timo/anaconda/bin:/usr/bin:/home/timo/.rbenv:/usr/local/go/bin:/home/timo/go/bin:/home/timo/go:/home/timo/.local/share/gem/ruby/3.0.0/bin:/home/timo/.deno/bin:/home/timo/.yarn/bin:/home/timo/.pulumi/bin:/opt/rocm/bin:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.sdkman/candidates/scala/current/bin:/home/timo/.sdkman/candidates/maven/current/bin:/home/timo/.sdkman/candidates/java/current/bin:/home/timo/.sdkman/candidates/groovy/current/bin:/home/timo/.sdkman/candidates/grails/current/bin:/home/timo/.sdkman/candidates/gradle/current/bin:/home/timo/.local/bin:/usr/local/bin:/bin:/usr/local/sbin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems:/usr/lib/jvm/default/bin:/usr/bin/site_perl:/usr/bin/vendor_perl:/usr/bin/core_perl:/usr/lib/rustup/bin:/var/lib/snapd/snap/bin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems"
  },
  "unset": [
    "CARGO_TARGET_DIR",
    "CARGO_BUILD_TARGET_DIR",
    "CARGO_ENCODED_RUSTFLAGS"
  ],
  "temporary_root": "/home/timo/.cache/ess-w18-consumer-source-pass2-tmp",
  "retained_roots": [
    "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target",
    "/home/timo/.cache/ess-w18-consumer-tmp",
    "/home/timo/.cache/ess-w18-consumer-stage2-tmp",
    "/home/timo/.cache/ess-w18-consumer-source-pass1-tmp",
    "/home/timo/.cache/ess-w18-consumer-correction1-tmp",
    "/home/timo/.cache/ess-w18-consumer-source-pass2-tmp"
  ],
  "allowance_bytes": 9739689984,
  "floor_bytes": 6442450944
}
```

## 6. Every external path written

- /home/timo/.cache/ess-w18-consumer-source-pass2-tmp — sole new assigned external TMP; every descendant is retained and named in the full native census.
- /home/timo/.local/state/worktree/registry.sqlite3 — only ess-consumer-coverage-source-pass2 lease lifecycle through normal manager hooks.

Private Cargo/Go/XDG homes are inside the assigned consumer-source-pass-2 scratch. Original Stage1, Stage2, pass1, correction scratch/TMP and the default corrected checker output were read-only. The retired mutation target remains historical archive members and was not recreated. No cleanup, integration, download/install, Atlas/Website work, Git mutation, AEP command or delegation occurred. This is the second and final full source attack. Root owns correction, integration/publication and eventual cleanup.

```findings

- file: crates/edge/ess-xtask/src/consumer_coverage/consumer.rs
  line: 525
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: A newly added public default trait method keeps the old finite consumer classification set because trait methods are skipped as concrete API entries.

- file: crates/edge/ess-xtask/src/consumer_coverage/consumer.rs
  line: 587
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: An admitted type macro in a callable signature hides its Self-associated dependency, so changing that associated type can preserve the callable declaration fingerprint.

```
