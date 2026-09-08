---
format: aep.planning-md/1
id: review-result:consumer-coverage-source-pass1
kind: review-result
status: active
title: Consumer coverage source review, pass 1
relations:
- reviews: story:review-consumer-coverage
revision: 1
---
unit: story:review-consumer-coverage source pass 1 — b37572e410a7b4a4d18e2abb4fd99ed0db5401f5 plus tests-only working tree
verdict: NEEDS-CHANGE
cases: executed 88→92, red 4
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths (assigned TMP root and own lease registry; listed in part 6)
needs-coordinator: route the two reproducible findings; implementation, AEP, integration, publication and cleanup remain coordinator-owned

```text
git --no-pager diff --stat
 .../edge/ess-xtask/src/consumer_coverage/tests.rs  | 88 ++++++++++++++++++++++
 1 file changed, 88 insertions(+)
```

## 1. Tests-only scope

Only the reserved cfg(test) file changed. The complete original file is an unchanged byte prefix; 88 lines were appended, no original assertion was removed, ignored, relaxed or rewritten. The patch is tests-only.patch and the exact complete current 1,213-file source is archived in source.tar.gz, independently read against source-manifest.json. No implementation or AEP/Git lifecycle mutation was made. The source base has no consumer_coverage module; these two defects originate in the new extractor.

## 2. Newly written cases and first actual results

All four cases existed before the first Cargo command. The coordinator supplied the prior full ess-xtask count of 88 (83 main plus 5 layout); no baseline suite was rerun. All four additions failed for their named assertions on first actual execution, not compilation setup. The first command compiled the actual package and selected one exact case; the other three commands selected their exact names from that same retained executable (SHA256 7c871d6dc96d242d77d87b0deb44b670e30b19d2e1a9089445cf3fa28bc23f6d). Each directly exited 101 with 0 passed, 1 failed, 0 ignored and 86 filtered.

- tests.rs:866, absolute external type: adding an unused local std module must not change the graph of ::std::string::String. It currently inventories the local shadow and changes the root/member shapes.
- tests.rs:880, absolute external import: the same external authority must survive a leading-colon use. It currently resolves that use through the local module.
- tests.rs:894, public associated constant: adding Owner::NEW_MODE must require a new finite classification. Existing classifications currently accept it unchanged.
- tests.rs:924, associated output type: changing Iterator::Item from u8 to u16 must change its bound callable contract. The produced profile hashes are identical. This last case is a controlled future-bound-profile fixture; no current 55-entry selected profile binds this Iterator::next entry, so no current production baseline transfer is claimed for that fixture.

### absolute-type-first


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
    "consumer_coverage::tests::adversary_absolute_external_type_is_not_shadowed_by_a_local_module",
    "--",
    "--exact",
    "--test-threads",
    "1"
  ],
  "at": 1788830654.284823,
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18"
}
```

```json
{
  "at": 1788830674.3598409,
  "elapsed_seconds": 20.074951615999453,
  "exit": 101,
  "pid": 3262344
}
```

stdout SHA256 c51d93b65063cd00dbd671f19f4b6b5443f918b23f3a27dbd32cdef49e3dd7d3

```text

running 1 test
test consumer_coverage::tests::adversary_absolute_external_type_is_not_shadowed_by_a_local_module ... FAILED

failures:

---- consumer_coverage::tests::adversary_absolute_external_type_is_not_shadowed_by_a_local_module stdout ----

thread 'consumer_coverage::tests::adversary_absolute_external_type_is_not_shadowed_by_a_local_module' (3265257) panicked at crates/edge/ess-xtask/src/consumer_coverage/tests.rs:873:5:
assertion `left == right` failed: an absolute external path still names std::string::String; an unused local module is outside the reachable model graph
  left: Object {"diagnostic_macros": Object {}, "macro_definitions": Object {}, "obligations": Object {"rust:fixture::Root": String("3583b78a9b40aa6a92b7d0f8190c6d73c364c63515b7fab708a0bbabd615e58a"), "rust:fixture::Root/field/value": String("af29abb3614b6d5449467174f2ab87760fd62aa155f9c12fb8e950c2ce994c4f")}, "references": Object {"fixture::Root": Array []}}
 right: Object {"diagnostic_macros": Object {}, "macro_definitions": Object {}, "obligations": Object {"rust:fixture::Root": String("68f7b6fddd0cf40e6c6ac304d0a64e52d7d0a1f4d1022554d1829361cb3c8fce"), "rust:fixture::Root/field/value": String("e03a8f39f5df179428cf14fd23a673a5db3c2838d61fbca292dca5804d47a1e9"), "rust:fixture::std::string::String": String("ab994e4f57829e95be63c33b47ceb309b203f4b7cf9dc1eaea03e24ec51f5694"), "rust:fixture::std::string::String/field/hidden": String("c35786411487c5aa1dd4906018312268e6f1862fd1a34c4ee9380d20c01e2f7f")}, "references": Object {"fixture::Root": Array [String("fixture::std::string::String")], "fixture::std::string::String": Array []}}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    consumer_coverage::tests::adversary_absolute_external_type_is_not_shadowed_by_a_local_module

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 86 filtered out; finished in 0.00s

```

stderr SHA256 d2b669a0883b14237dbb77521a4618c19ec3d873d65ae207af11af37d5973cee

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
   Compiling thiserror v2.0.20
   Compiling schemars v0.8.22
   Compiling const-oid v0.10.2
   Compiling schemars_derive v0.8.22
   Compiling digest v0.11.3
   Compiling thiserror-impl v2.0.20
   Compiling hashbrown v0.17.1
   Compiling cfg-if v1.0.4
   Compiling dyn-clone v1.0.20
   Compiling equivalent v1.0.2
   Compiling cpufeatures v0.3.1
   Compiling sha2 v0.11.0
   Compiling indexmap v2.14.1
   Compiling utf8parse v0.2.2
   Compiling unsafe-libyaml v0.2.11
   Compiling ryu v1.0.23
   Compiling serde_yaml v0.9.34+deprecated
   Compiling ess-primitives v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/crates/specify/ess-primitives)
   Compiling anstyle-parse v1.0.0
   Compiling pulldown-cmark v0.13.4
   Compiling anstyle v1.0.14
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle-query v1.1.5
   Compiling colorchoice v1.0.5
   Compiling anstream v1.0.0
   Compiling ess-domain v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/crates/specify/ess-domain)
   Compiling unicase v2.9.0
   Compiling clap_lex v1.1.0
   Compiling heck v0.5.0
   Compiling pulldown-cmark-escape v0.11.0
   Compiling bitflags v2.13.1
   Compiling strsim v0.11.1
   Compiling anyhow v1.0.104
   Compiling clap_builder v4.6.6
   Compiling ess-compiler v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/crates/specify/ess-compiler)
   Compiling clap_derive v4.6.4
   Compiling ess-xtask v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/crates/edge/ess-xtask)
   Compiling clap v4.6.6
   Compiling ess-gen v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/crates/generate/ess-gen)
    Finished `test` profile [unoptimized] target(s) in 20.05s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-7e95320043038dad)
error: test failed, to rerun pass `-p ess-xtask --bin ess-xtask`
```

Completion and count record:
```json
{
  "elapsed_seconds": 20.074951615999453,
  "exit": 101,
  "native_copies": 2,
  "resource_stop": null,
  "resources_ok_after_retention": true,
  "source_unchanged": true,
  "stderr_sha256": "d2b669a0883b14237dbb77521a4618c19ec3d873d65ae207af11af37d5973cee",
  "stdout_sha256": "c51d93b65063cd00dbd671f19f4b6b5443f918b23f3a27dbd32cdef49e3dd7d3",
  "summaries": [
    "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 86 filtered out; finished in 0.00s"
  ]
}
```

### absolute-import-first


```json
{
  "argv": [
    "target/debug/deps/ess_xtask-7e95320043038dad",
    "--exact",
    "consumer_coverage::tests::adversary_absolute_external_import_keeps_its_external_owner",
    "--test-threads",
    "1"
  ],
  "at": 1788830694.9664333,
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18"
}
```

```json
{
  "at": 1788830695.0585349,
  "elapsed_seconds": 0.09204213495831937,
  "exit": 101,
  "pid": 3267316
}
```

stdout SHA256 3a1ca94f7da2eb342441d692610953ae638573e33b31e626883af7b78b4201d1

```text

running 1 test
test consumer_coverage::tests::adversary_absolute_external_import_keeps_its_external_owner ... FAILED

failures:

---- consumer_coverage::tests::adversary_absolute_external_import_keeps_its_external_owner stdout ----

thread 'consumer_coverage::tests::adversary_absolute_external_import_keeps_its_external_owner' (3267317) panicked at crates/edge/ess-xtask/src/consumer_coverage/tests.rs:887:5:
assertion `left == right` failed: a leading-colon use must not bind a local module of the same name
  left: Object {"diagnostic_macros": Object {}, "macro_definitions": Object {}, "obligations": Object {"rust:fixture::Root": String("3583b78a9b40aa6a92b7d0f8190c6d73c364c63515b7fab708a0bbabd615e58a"), "rust:fixture::Root/field/value": String("af29abb3614b6d5449467174f2ab87760fd62aa155f9c12fb8e950c2ce994c4f")}, "references": Object {"fixture::Root": Array []}}
 right: Object {"diagnostic_macros": Object {}, "macro_definitions": Object {}, "obligations": Object {"rust:fixture::Root": String("68f7b6fddd0cf40e6c6ac304d0a64e52d7d0a1f4d1022554d1829361cb3c8fce"), "rust:fixture::Root/field/value": String("e03a8f39f5df179428cf14fd23a673a5db3c2838d61fbca292dca5804d47a1e9"), "rust:fixture::std::string::String": String("ab994e4f57829e95be63c33b47ceb309b203f4b7cf9dc1eaea03e24ec51f5694"), "rust:fixture::std::string::String/field/hidden": String("c35786411487c5aa1dd4906018312268e6f1862fd1a34c4ee9380d20c01e2f7f")}, "references": Object {"fixture::Root": Array [String("fixture::std::string::String")], "fixture::std::string::String": Array []}}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    consumer_coverage::tests::adversary_absolute_external_import_keeps_its_external_owner

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 86 filtered out; finished in 0.00s

```

stderr SHA256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

```text
```

Completion and count record:
```json
{
  "elapsed_seconds": 0.09204213495831937,
  "exit": 101,
  "native_copies": 1,
  "resource_stop": null,
  "resources_ok_after_retention": true,
  "source_unchanged": true,
  "stderr_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "stdout_sha256": "3a1ca94f7da2eb342441d692610953ae638573e33b31e626883af7b78b4201d1",
  "summaries": [
    "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 86 filtered out; finished in 0.00s"
  ]
}
```

### associated-constant-first


```json
{
  "argv": [
    "target/debug/deps/ess_xtask-7e95320043038dad",
    "--exact",
    "consumer_coverage::tests::adversary_new_public_associated_constant_requires_a_concrete_classification",
    "--test-threads",
    "1"
  ],
  "at": 1788830708.5335944,
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18"
}
```

```json
{
  "at": 1788830708.6205065,
  "elapsed_seconds": 0.08685376797802746,
  "exit": 101,
  "pid": 3268543
}
```

stdout SHA256 285099958b6cd780d3ac5b70a4c8b1cdd96c06c47014cdd9f65e8909055c9957

```text

running 1 test
test consumer_coverage::tests::adversary_new_public_associated_constant_requires_a_concrete_classification ... FAILED

failures:

---- consumer_coverage::tests::adversary_new_public_associated_constant_requires_a_concrete_classification stdout ----

thread 'consumer_coverage::tests::adversary_new_public_associated_constant_requires_a_concrete_classification' (3268544) panicked at crates/edge/ess-xtask/src/consumer_coverage/tests.rs:917:5:
adding a public associated constant is a new API declaration and must not inherit the old finite classifications
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    consumer_coverage::tests::adversary_new_public_associated_constant_requires_a_concrete_classification

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 86 filtered out; finished in 0.00s

```

stderr SHA256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

```text
```

Completion and count record:
```json
{
  "elapsed_seconds": 0.08685376797802746,
  "exit": 101,
  "native_copies": 1,
  "resource_stop": null,
  "resources_ok_after_retention": true,
  "source_unchanged": true,
  "stderr_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "stdout_sha256": "285099958b6cd780d3ac5b70a4c8b1cdd96c06c47014cdd9f65e8909055c9957",
  "summaries": [
    "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 86 filtered out; finished in 0.00s"
  ]
}
```

### associated-output-first


```json
{
  "argv": [
    "target/debug/deps/ess_xtask-7e95320043038dad",
    "--exact",
    "consumer_coverage::tests::adversary_changed_associated_output_type_invalidates_its_consumer_contract",
    "--test-threads",
    "1"
  ],
  "at": 1788830709.2390547,
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18"
}
```

```json
{
  "at": 1788830709.3290007,
  "elapsed_seconds": 0.08988432202022523,
  "exit": 101,
  "pid": 3268653
}
```

stdout SHA256 3b8e1dc6982fc23572edbd745f6a5b88786d7c88230a0758091f3b476cac8638

```text

running 1 test
test consumer_coverage::tests::adversary_changed_associated_output_type_invalidates_its_consumer_contract ... FAILED

failures:

---- consumer_coverage::tests::adversary_changed_associated_output_type_invalidates_its_consumer_contract stdout ----

thread 'consumer_coverage::tests::adversary_changed_associated_output_type_invalidates_its_consumer_contract' (3268654) panicked at crates/edge/ess-xtask/src/consumer_coverage/tests.rs:945:5:
assertion `left != right` failed: Iterator::Item changes the callable result contract even when the method spells Self::Item and its body is unchanged
  left: "8028eebfc2040d1708efdf029abd1e5b510020593904769c8d6c015aef8f2e4a"
 right: "8028eebfc2040d1708efdf029abd1e5b510020593904769c8d6c015aef8f2e4a"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    consumer_coverage::tests::adversary_changed_associated_output_type_invalidates_its_consumer_contract

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 86 filtered out; finished in 0.00s

```

stderr SHA256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855

```text
```

Completion and count record:
```json
{
  "elapsed_seconds": 0.08988432202022523,
  "exit": 101,
  "native_copies": 1,
  "resource_stop": null,
  "resources_ok_after_retention": true,
  "source_unchanged": true,
  "stderr_sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
  "stdout_sha256": "3b8e1dc6982fc23572edbd745f6a5b88786d7c88230a0758091f3b476cac8638",
  "summaries": [
    "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 86 filtered out; finished in 0.00s"
  ]
}
```

## 3. Affected package after the additions

Only ess-xtask was modified. Its full package was run with --no-fail-fast so the integration layout owner executed even after the main binary failed. Actual result: 83 original main cases passed, four new cases failed, five layout cases passed; executed 92, 88 passed, 4 failed, no ignored cases. No ess-diff assertion or source changed, so its separate 166-case suite was not repeated.

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
  "at": 1788830722.1858704,
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18"
}
```

```json
{
  "at": 1788830765.6879618,
  "elapsed_seconds": 43.50202354392968,
  "exit": 101,
  "pid": 3270052
}
```

stdout SHA256 1dbf98d7643220c8b80af4836093d6650fac614236ca006e08dfe35fc6c41f05

```text

running 87 tests
test consumer_coverage::tests::consumer_unknown_cfg_is_not_a_default_profile ... ok
test consumer_coverage::tests::finite_unaccepted_eligibility_has_exact_complete_accounting ... ok
test consumer_coverage::tests::comments_and_whitespace_do_not_change_rust_identity_or_shape ... ok
test consumer_coverage::tests::cfg_test_is_excluded_without_hiding_production ... ok
test consumer_coverage::tests::adversary_absolute_external_type_is_not_shadowed_by_a_local_module ... FAILED
test consumer_coverage::tests::adversary_absolute_external_import_keeps_its_external_owner ... FAILED
test consumer_coverage::tests::concrete_consumer_entries_and_full_case_names_are_not_package_aliases ... ok
test consumer_coverage::tests::changed_string_backed_alternative_is_not_hidden_by_wire_string ... ok
test consumer_coverage::tests::generated_consumer_owner_uses_the_checked_invocation_grammar ... ok
test consumer_coverage::tests::classified_nonmodel_macro_definition_and_invocation_drift_refuse ... ok
test consumer_coverage::tests::adversary_new_public_associated_constant_requires_a_concrete_classification ... FAILED
test consumer_coverage::tests::local_declarations_shadow_prelude_leaves_and_generics_refuse ... ok
test consumer_coverage::tests::adversary_changed_associated_output_type_invalidates_its_consumer_contract ... FAILED
test consumer_coverage::tests::declaration_order_and_unknown_representation_grammar_are_not_erased ... ok
test consumer_coverage::tests::local_aliases_inline_modules_reexports_and_cycles_are_resolved ... ok
test consumer_coverage::tests::new_member_or_consumer_cannot_inherit_eligibility ... ok
test consumer_coverage::tests::pending_owner_checkpoint_is_complete_but_never_eligible ... ok
test consumer_coverage::tests::stage2_case_engine_lists_filters_and_executes_before_qualifying ... ok
test consumer_coverage::tests::stage2_case_engine_refuses_source_drift_or_missing_actual_result ... ok
test consumer_coverage::tests::measured_build_profile_refuses_unsupported_target_features_and_wrappers ... ok
test consumer_coverage::tests::production_cfg_and_alias_ambiguity_fail_closed ... ok
test consumer_coverage::tests::stage2_measured_stable_listing_requires_exact_nonignored_case ... ok
test consumer_coverage::tests::stage2_current_flags_target_tools_and_complete_configuration_set_are_bound ... ok
test consumer_coverage::tests::stage2_exact_finite_baseline_plus_behavioral_case_covers_two_pairs ... ok
test consumer_coverage::tests::ignored_case_and_nested_execution_are_visible_candidates_only ... ok
test consumer_coverage::tests::stage2_new_obligation_can_gain_behavior_without_expanding_initial_unknowns ... ok
test consumer_coverage::tests::stage2_claims_need_cases_and_named_refusal_without_contradictions ... ok
test consumer_coverage::tests::stage2_new_consumer_and_changed_profile_cannot_inherit_unknown ... ok
test consumer_coverage::tests::public_reexport_additions_and_alias_changes_have_concrete_identities ... ok
test consumer_coverage::tests::stage2_swallowed_panic_and_skipped_nested_branches_do_not_qualify ... ok
test consumer_coverage::tests::stage2_classification_refusal_preserves_actual_accounting_diagnostics ... ok
test consumer_coverage::tests::representation_attributes_invalidate_shape_and_unknown_attributes_refuse ... ok
test consumer_coverage::tests::stage2_refusal_counts_only_current_eligible_unknown_cells ... ok
test consumer_coverage::tests::stage2_new_optional_or_string_backed_member_cannot_inherit_unknown ... ok
test consumer_coverage::tests::new_callable_and_target_alternatives_change_exact_inventory ... ok
test consumer_coverage::tests::stage2_one_actual_pass_requires_successful_direct_exit ... ok
test consumer_coverage::tests::stage2_zero_selected_ignored_failing_and_forged_results_refuse ... ok
test consumer_coverage::tests::private_optional_tuple_and_enum_members_have_separate_identities ... ok
test consumer_coverage::tests::stage2_removed_duplicate_and_missing_pairs_refuse ... ok
test consumer_coverage::tests::unknown_production_macro_cannot_hide_a_declaration ... ok
test consumer_coverage::tests::unknown_type_owner_is_a_named_refusal ... ok
test consumer_coverage::tests::stale_duplicate_ownerless_accepted_and_mandatory_unknown_rows_refuse ... ok
test consumer_coverage::tests::stage2_unknown_requires_independent_owner_and_closed_manifest ... ok
test consumer_coverage::tests::stage2_cargo_artifact_requires_exact_owner_target_profile_and_completed_build ... ok
test support::tests::a_complete_source_block_is_accepted_without_owning_release_prose ... ok
test consumer_coverage::tests::wire_annotations_are_excluded_only_at_schema_positions ... ok
test consumer_coverage::tests::wire_unknown_keyword_dialect_and_external_references_refuse ... ok
test consumer_coverage::tests::wire_boolean_schemas_and_ordered_literals_are_concrete ... ok
test consumer_coverage::tests::wire_definitions_references_and_escaped_property_names_are_separate ... ok
test support::tests::cargo_version_changes_invalidate_only_the_source_block ... ok
test consumer_coverage::tests::consumer_identity_separates_body_comments_and_unrelated_entries_from_signatures ... ok
test support::tests::command_inventory_requires_a_complete_nonempty_unique_section ... ok
test support::tests::every_material_row_change_is_refused_with_its_location ... ok
test consumer_coverage::tests::wire_reference_cycle_is_finite_and_descendants_invalidate_parents ... ok
test support::tests::adversary_real_source_version_drift_keeps_release_bytes_independent ... ok
test support::tests::missing_duplicated_or_reordered_block_markers_refuse ... ok
test support::tests::help_inventory_reads_both_clap_layouts_without_swallowing_neighbor_options ... ok
test support::tests::support_check_is_an_available_maintenance_command ... ok
test tests::an_empty_release_is_refused_by_name ... ok
test tests::an_undated_release_is_refused_by_name ... ok
test tests::exclusions_cover_only_the_named_subtree ... ok
test tests::generated_paths_must_stay_below_the_projection_root ... ok
test tests::only_bare_version_tags_are_release_tags ... ok
test tests::published_release_tags_come_from_the_json_report ... ok
test tests::release_notes_stop_before_the_next_release ... ok
test tests::a_named_but_untagged_version_is_not_an_incomplete_release ... ok
test tests::the_generated_index_includes_static_site_source ... ok
test tests::the_release_record_is_checked_after_every_release_run ... ok
test tests::workspace_version_comes_only_from_the_workspace_package_table ... ok
test tests::sync_checks_and_reconciles_in_both_directions ... ok
test tests::a_version_tag_with_no_release_behind_it_is_refused ... ok
test tests::release_publication_is_evaluated_after_every_dependency_finishes ... ok
test support::tests::emitted_version_markers_are_parsed_and_must_agree_across_the_projection ... ok
test tests::a_version_tag_whose_commit_never_reached_main_is_refused ... ok
test support::tests::html_requires_the_actual_output_root_and_both_nonempty_local_assets ... ok
test consumer_coverage::tests::file_level_production_cfg_cannot_hide_from_the_graph ... ok
test consumer_coverage::tests::selected_root_reaching_guarded_diagnostic_type_is_not_an_opaque_leaf ... ok
test consumer_coverage::tests::actual_selected_roots_resolve_without_diagnostic_generated_declarations ... ok
test consumer_coverage::tests::external_module_inside_inline_module_uses_its_semantic_directory ... ok
test consumer_coverage::tests::diagnostic_macro_definition_and_invocation_changes_refuse ... ok
test support::tests::adversary_real_docs_ir_marker_is_nested_json_not_visible_marker_text ... ok
test support::tests::adversary_actual_help_missing_target_metadata_cannot_borrow_neighbor_values ... ok
test support::tests::adversary_actual_cli_refusal_is_not_a_successful_support_observation ... ok
test support::tests::adversary_adjacent_readme_is_selected_without_authored_flags ... ok
test support::tests::adversary_front_page_override_replaces_adjacent_readme_for_a_specification_file ... ok
test support::tests::adversary_actual_explicit_site_and_combined_maps_keep_distinct_roots ... ok
test support::tests::adversary_all_real_material_rows_refuse_cell_removal_duplicate_and_order_drift ... ok

failures:

---- consumer_coverage::tests::adversary_absolute_external_type_is_not_shadowed_by_a_local_module stdout ----

thread 'consumer_coverage::tests::adversary_absolute_external_type_is_not_shadowed_by_a_local_module' (3270702) panicked at crates/edge/ess-xtask/src/consumer_coverage/tests.rs:873:5:
assertion `left == right` failed: an absolute external path still names std::string::String; an unused local module is outside the reachable model graph
  left: Object {"diagnostic_macros": Object {}, "macro_definitions": Object {}, "obligations": Object {"rust:fixture::Root": String("3583b78a9b40aa6a92b7d0f8190c6d73c364c63515b7fab708a0bbabd615e58a"), "rust:fixture::Root/field/value": String("af29abb3614b6d5449467174f2ab87760fd62aa155f9c12fb8e950c2ce994c4f")}, "references": Object {"fixture::Root": Array []}}
 right: Object {"diagnostic_macros": Object {}, "macro_definitions": Object {}, "obligations": Object {"rust:fixture::Root": String("68f7b6fddd0cf40e6c6ac304d0a64e52d7d0a1f4d1022554d1829361cb3c8fce"), "rust:fixture::Root/field/value": String("e03a8f39f5df179428cf14fd23a673a5db3c2838d61fbca292dca5804d47a1e9"), "rust:fixture::std::string::String": String("ab994e4f57829e95be63c33b47ceb309b203f4b7cf9dc1eaea03e24ec51f5694"), "rust:fixture::std::string::String/field/hidden": String("c35786411487c5aa1dd4906018312268e6f1862fd1a34c4ee9380d20c01e2f7f")}, "references": Object {"fixture::Root": Array [String("fixture::std::string::String")], "fixture::std::string::String": Array []}}

---- consumer_coverage::tests::adversary_absolute_external_import_keeps_its_external_owner stdout ----

thread 'consumer_coverage::tests::adversary_absolute_external_import_keeps_its_external_owner' (3270701) panicked at crates/edge/ess-xtask/src/consumer_coverage/tests.rs:887:5:
assertion `left == right` failed: a leading-colon use must not bind a local module of the same name
  left: Object {"diagnostic_macros": Object {}, "macro_definitions": Object {}, "obligations": Object {"rust:fixture::Root": String("3583b78a9b40aa6a92b7d0f8190c6d73c364c63515b7fab708a0bbabd615e58a"), "rust:fixture::Root/field/value": String("af29abb3614b6d5449467174f2ab87760fd62aa155f9c12fb8e950c2ce994c4f")}, "references": Object {"fixture::Root": Array []}}
 right: Object {"diagnostic_macros": Object {}, "macro_definitions": Object {}, "obligations": Object {"rust:fixture::Root": String("68f7b6fddd0cf40e6c6ac304d0a64e52d7d0a1f4d1022554d1829361cb3c8fce"), "rust:fixture::Root/field/value": String("e03a8f39f5df179428cf14fd23a673a5db3c2838d61fbca292dca5804d47a1e9"), "rust:fixture::std::string::String": String("ab994e4f57829e95be63c33b47ceb309b203f4b7cf9dc1eaea03e24ec51f5694"), "rust:fixture::std::string::String/field/hidden": String("c35786411487c5aa1dd4906018312268e6f1862fd1a34c4ee9380d20c01e2f7f")}, "references": Object {"fixture::Root": Array [String("fixture::std::string::String")], "fixture::std::string::String": Array []}}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- consumer_coverage::tests::adversary_new_public_associated_constant_requires_a_concrete_classification stdout ----

thread 'consumer_coverage::tests::adversary_new_public_associated_constant_requires_a_concrete_classification' (3270704) panicked at crates/edge/ess-xtask/src/consumer_coverage/tests.rs:917:5:
adding a public associated constant is a new API declaration and must not inherit the old finite classifications

---- consumer_coverage::tests::adversary_changed_associated_output_type_invalidates_its_consumer_contract stdout ----

thread 'consumer_coverage::tests::adversary_changed_associated_output_type_invalidates_its_consumer_contract' (3270703) panicked at crates/edge/ess-xtask/src/consumer_coverage/tests.rs:945:5:
assertion `left != right` failed: Iterator::Item changes the callable result contract even when the method spells Self::Item and its body is unchanged
  left: "8028eebfc2040d1708efdf029abd1e5b510020593904769c8d6c015aef8f2e4a"
 right: "8028eebfc2040d1708efdf029abd1e5b510020593904769c8d6c015aef8f2e4a"


failures:
    consumer_coverage::tests::adversary_absolute_external_import_keeps_its_external_owner
    consumer_coverage::tests::adversary_absolute_external_type_is_not_shadowed_by_a_local_module
    consumer_coverage::tests::adversary_changed_associated_output_type_invalidates_its_consumer_contract
    consumer_coverage::tests::adversary_new_public_associated_constant_requires_a_concrete_classification

test result: FAILED. 83 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 37.91s


running 5 tests
test the_path_scan_reads_an_area_qualified_path ... ok
test every_workspace_crate_lives_under_an_area_directory ... ok
test the_path_scan_excludes_by_root_relative_path_and_reads_published_website_source ... ok
test the_path_scan_finds_at_least_one_path_in_this_repository ... ok
test every_literal_path_naming_a_workspace_crate_exists ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

```

stderr SHA256 2fbd29836ca6ce4692b28bf785f8934cc1b2537b793a0e06f7c0e0189e35dfb3

```text
   Compiling ess-xtask v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/crates/edge/ess-xtask)
    Finished `test` profile [unoptimized] target(s) in 5.41s
     Running unittests src/main.rs (target/debug/deps/ess_xtask-7e95320043038dad)
error: test failed, to rerun pass `-p ess-xtask --bin ess-xtask`
     Running tests/layout.rs (target/debug/deps/layout-19a7b55d79637f7c)
error: 1 target failed:
    `-p ess-xtask --bin ess-xtask`
```

Completion and count record:
```json
{
  "elapsed_seconds": 43.50202354392968,
  "exit": 101,
  "native_copies": 4,
  "resource_stop": null,
  "resources_ok_after_retention": true,
  "source_unchanged": true,
  "stderr_sha256": "2fbd29836ca6ce4692b28bf785f8934cc1b2537b793a0e06f7c0e0189e35dfb3",
  "stdout_sha256": "1dbf98d7643220c8b80af4836093d6650fac614236ca006e08dfe35fc6c41f05",
  "summaries": [
    "test result: FAILED. 83 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 37.91s",
    "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s"
  ]
}
```

## 4. Findings and reachability

| File:line | Verdict / origin | What was measured | What reaches it |
|---|---|---|---|
| crates/edge/ess-xtask/src/consumer_coverage/rust.rs:445 | NEEDS-CHANGE / introduced | The two actual exact path/import cases fail their equal-graph assertions at tests.rs:873 and :888, direct 101. `Type::Path.leading_colon` and `ItemUse.leading_colon` are discarded before `Graph::resolve` applies local shadowing. | Production `run` invokes `rust::extract`, whose selected-root traversal shares this same `Graph::ty`/imports/resolve path. A valid absolute external type or import under a same-named local module reaches the defect. The fixtures establish this bounded discovery mechanism, not compiled production schema/behavior under a new mutated model; no current source is claimed to contain that shadow. |
| crates/edge/ess-xtask/src/consumer_coverage/consumer.rs:471 | NEEDS-CHANGE / introduced | The public-constant case fails at tests.rs:917 because old finite classifications accept the new declaration; the associated-output case fails at :945 with the same 8028eebf… fingerprint. Only ImplItem::Fn is scanned. | Production `consumer::extract` always passes production impls through `Inventory::implementation`; the accepted policy requires each new public API declaration to be explicitly classified. Existing source already contains the ignored form at Timestamp::EPOCH (ess-primitives/src/time.rs:45), Horizon::MAX_DAYS (:378), and EntityLocator::PATTERN (entity.rs:131), so this is a reachable declaration grammar. The associated-output subcase uses a constructed bound Iterator::next profile; none of the currently selected 55 entries uses it, and no actual current baseline transfer is claimed for that subcase. |

Origin is introduced because both failing implementation owners were added after a0cf3ca8681ce06f6fbdbc988d457b23f2136c04; the existing associated constants are evidence of the source grammar, not a claim that the old base had this gate defect. No base checkout/build was performed. Preserve absolute external qualification (or explicitly refuse an unsupported form) before local resolution. Give associated declarations a closed inventory/fingerprint treatment; do not silently skip them. These are proposed repair directions, not applied changes or approval.

## 5. Scope checked, retained authority and limits

- Read the complete accepted binding, story acceptance, adversary charter, worktree instructions and all new extractor/accounting/executor/build-stamp code; inspected the full 30-file diff inventory, all finite JSON payloads through parsing, all 93 profile boundaries and 22 case-attribution records. The 34,818,675 bytes of changed source were hashed; this is not a claim of line-by-line semantic review of the large planning journal or each of 157,068 repeated baseline rows.
- Read the Stage 2 report claims, causal summaries and actual owner assertions; its complete 2,602,434-byte report SHA256 fc92ce3134ead2695c8a06cfbe6da62f73516bfdcd393e706aeb7d67196b1cd8 matches dispatch. Root supplies the independently read 7caf original full checker/mutation streams and seals. This pass did not repeat those producers or present 7caf results as b375 execution.
- The frozen baseline remains exactly e005a2e74e067ad51315e594151643b702381dc174bb9af5c355c3eeeb17ad51: 1,806 model shapes, 87 groups and 157,068 enumerated unknown pairs; 54 mandatory pairs stay excluded. No baseline or eligibility edit occurred.
- Existing package cases covering missing/stale/duplicate accounting, cfg/macro/schema guard refusal, artifact identity, exact listing/results, source/tool/config drift and nested-runtime refusal all remained green. This is the new affected-suite execution; it is not a full production matrix pass.
- Direct Rust F01 claims stay bounded to their actual output/delta assertions; the Struct mutation proves annotation preservation/invalidation, not runtime invariants. No newly attributed Go/browser/TypeScript execution is claimed.
- Root's three-file b375 synchronization explicitly adds locked/offline provider launch. The actual full checker and full integration on final synchronized/fixed source remain owed by root.
- Original stdout/stderr, direct exits, before/after source maps, resource samples, selected environment and independent executable copies remain under each named lane. execution-readback.json freshly verifies all five lanes, every retained native copy and the full source archive. Seven prebuild executable paths were independently retained before alias replacement; source and runtime images are not replaced by hashes alone.
- All producers completed and were absent before the final census. seal/native-inventory.jsonl inventories the complete unit target and all three assigned/retained TMP roots; a separate fresh native reader checks all payloads, native byte names, literal symlink targets, metadata and hardlink identities. Only finite precreated seal bookkeeping paths are excluded and individually pinned. This is retention/readback, not an agent independence claim or AEP evidence issuance.

The selected effective environment is retained verbatim:
```json
{
  "set": {
    "CARGO_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-1/cargo-home",
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
    "TMPDIR": "/home/timo/.cache/ess-w18-consumer-source-pass1-tmp",
    "GOCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-1/go-cache",
    "GOMODCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-1/go-mod-cache",
    "XDG_CACHE_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-1/xdg-cache",
    "XDG_CONFIG_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-1/xdg-config",
    "XDG_DATA_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-1/xdg-data",
    "XDG_RUNTIME_DIR": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-1/xdg-runtime",
    "XDG_STATE_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-source-pass-1/xdg-state",
    "PATH": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target/review-boundaries-18/consumer-coverage/rust-toolchain/bin:/home/timo/.local/bin:/home/timo/.deno/bin:/home/timo/.codex/packages/standalone/releases/0.153.4-x86_64-unknown-linux-musl/codex-path:/home/timo/.codex/tmp/arg0/codex-arg0EIPBn9:/home/linuxbrew/.linuxbrew/Caskroom/codex/0.153.4/codex-path:/home/timo/.local/bin:/home/timo/.deno/bin:/home/timo/.codex/tmp/arg0/codex-arg0dAS0R8:/home/linuxbrew/.linuxbrew/Caskroom/codex/0.153.4/codex-path:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.opencode/bin:/home/timo/.fly/bin:/home/timo/.cargo/bin:/home/timo:/home/timo/anaconda/bin:/usr/bin:/home/timo/.rbenv:/usr/local/go/bin:/home/timo/go/bin:/home/timo/go:/home/timo/.local/share/gem/ruby/3.0.0/bin:/home/timo/.deno/bin:/home/timo/.yarn/bin:/home/timo/.pulumi/bin:/opt/rocm/bin:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.sdkman/candidates/scala/current/bin:/home/timo/.sdkman/candidates/maven/current/bin:/home/timo/.sdkman/candidates/java/current/bin:/home/timo/.sdkman/candidates/groovy/current/bin:/home/timo/.sdkman/candidates/grails/current/bin:/home/timo/.sdkman/candidates/gradle/current/bin:/home/timo/.local/bin:/usr/local/bin:/bin:/usr/local/sbin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems:/usr/lib/jvm/default/bin:/usr/bin/site_perl:/usr/bin/vendor_perl:/usr/bin/core_perl:/usr/lib/rustup/bin:/var/lib/snapd/snap/bin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems"
  },
  "unset": [
    "CARGO_TARGET_DIR",
    "CARGO_ENCODED_RUSTFLAGS"
  ],
  "temporary_root": "/home/timo/.cache/ess-w18-consumer-source-pass1-tmp",
  "retained_roots": [
    "/home/timo/.local/state/worktree/trees/b10x/ess/ess-consumer-coverage-wave18/target",
    "/home/timo/.cache/ess-w18-consumer-tmp",
    "/home/timo/.cache/ess-w18-consumer-stage2-tmp",
    "/home/timo/.cache/ess-w18-consumer-source-pass1-tmp"
  ],
  "allowance_bytes": 8501440512,
  "floor_bytes": 8589934592
}
```

## 6. Every external path written

- /home/timo/.cache/ess-w18-consumer-source-pass1-tmp — sole assigned external TMP root; every descendant is retained and named by the full native census.
- /home/timo/.local/state/worktree/registry.sqlite3 — only the assigned ess-consumer-coverage-source-pass1 lease lifecycle, using the normal manager environment.

No other external write root was selected. The original Stage 1 and Stage 2 scratch/TMP and mutation tree were read-only. No cleanup, download, installation, integration, Atlas/Website operation, branch switch, commit or push was performed. Root owns final cleanup and any changed-scope decision.

```findings

- file: crates/edge/ess-xtask/src/consumer_coverage/rust.rs
  line: 445
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The model resolver discards absolute path qualification in types and imports, so a local module can replace the actual external type owner in the authoritative model graph.

- file: crates/edge/ess-xtask/src/consumer_coverage/consumer.rs
  line: 471
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Implementation scanning omits associated constants and types, allowing a new public constant to retain old finite classifications and an associated result type to escape its bound callable fingerprint.

```
