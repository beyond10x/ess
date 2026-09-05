---
format: aep.planning-md/1
id: review-result:review-boundaries-3-delivery-adversary-pass-1
kind: review-result
status: active
title: Wave 3 delivery adversary pass 1
relations:
- reviews: story:review-persisted-delivery-validation
revision: 1
---
unit: story:review-persisted-delivery-validation at f1baa9051be7d6cfc48ec1dcd302d0c87ac21a15 plus adversary test additions
verdict: nothing found
cases: executed 67→73, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: none

1. `git --no-pager diff --stat`

```text
 crates/edge/ess-cli/tests/persisted_delivery.rs    | 101 +++++++++++++++++++
 crates/generate/ess-deployment/tests/deployment.rs | 108 +++++++++++++++++++++
 2 files changed, 209 insertions(+)
```

Both modified tracked paths are assigned integration test files. No untracked test files exist. Production, inline tests, manifests, planning and Git lifecycle were untouched. `git --no-pager diff --check` exited0 with no output. HEAD remained f1baa9051be7d6cfc48ec1dcd302d0c87ac21a15.

2. Added cases and isolated execution

The before count comes from the implementor's reported18 deployment +49 CLI cases. No preemptive baseline suite ran. Six cases were authored, then each selected alone with `--exact` and observed executing exactly one case. All six were green on their first actual test execution; there is no red test output to report. The initial cache startup failure executed zero cases and is retained below separately.

| Test file:line | Added case / measured assertion | Isolated result |
|---|---|---|
| crates/generate/ess-deployment/tests/deployment.rs:1086 | Compiler/runtime-reader differential checks for five invalid local shapes: value on optional config, endpoint/secret environment collision, repeated mount, simultaneous legacy/named storage, repeated volume. Each is refused by the actual compiler and by JSON, JSON Value, YAML and nested JSON/YAML IR decoding; compiler output control round-trips canonically. | 1 passed |
| crates/generate/ess-deployment/tests/deployment.rs:1129 | A build cycle is inserted in persisted canonical build bytes, its digest is recomputed, runtime build_digest is updated, and release build/runtime digests are updated. Standalone runtime admission succeeds, proving the cycle needs the included build; bundle admission refuses every tested route. Valid bundle retains canonical bytes. | 1 passed |
| crates/generate/ess-deployment/tests/deployment.rs:1162 | Three public mutations of an unselected catalog candidate (build, semantic and runtime digest) each block resolve_stack and persisted catalog decoding. Empty-stack resolution with the unchanged catalog is the valid control. | 1 passed |
| crates/edge/ess-cli/tests/persisted_delivery.rs:179 | Raw duplicate desired-release map keys, each containing an individually valid release, are rejected through .json and .yaml CLI reads before fake ORAS/Helm. Each format has a successful valid dry-run control. | 1 passed |
| crates/edge/ess-cli/tests/persisted_delivery.rs:208 | Raw duplicate current-image keys block both actual removal reconciliation and diff for .json and .yaml. Successful valid dry-run controls demonstrate the expected reverse removal set. Neither fake executor is called. | 1 passed |
| crates/edge/ess-cli/tests/persisted_delivery.rs:254 | A dependency-valid rollout ordering that delays a newly ready lexical predecessor is rejected before execution; the compiler's actual dynamic lexical Kahn order has a successful dry-run control. No fake executor calls. | 1 passed |

The existing valid executor control also ran in the package suite and checked exactly `oras\nhelm\nhelm\n`. All CLI commands had PATH restricted to the local compiled Rust fakes. No real external service, executor or credentials were used.

Initial attempted isolated command, before cache restoration:

Command: `env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-deployment --test deployment adversary_runtime_readers_match_compiler_slot_and_volume_refusals -- --exact --nocapture`

Exit: 101. Executed cases: 0. This is a compiler metadata startup failure, not a failing test or a finding.

```text
error: process didn't exit successfully: `/usr/bin/sccache /home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc -vV` (exit status: 2)
--- stderr
sccache: error: path must be shorter than SUN_LEN
```

The coordinator restored the exact assigned socket and reported a foreground server with idle timeout disabled. No client environment override or adversary cache lifecycle action was used. The following are the first actual executions, in order:

Case 1: `adversary_runtime_readers_match_compiler_slot_and_volume_refusals`

Command: `env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-deployment --test deployment adversary_runtime_readers_match_compiler_slot_and_volume_refusals -- --exact --nocapture`

Exit: 0. Executed: 1.

```text
   Compiling ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
    Finished `test` profile [unoptimized] target(s) in 2.88s
     Running tests/deployment.rs (target/debug/deps/deployment-f7d4304711ce6995)

running 1 test
test adversary_runtime_readers_match_compiler_slot_and_volume_refusals ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.02s
```

Case 2: `adversary_rehashed_bundle_cannot_launder_a_build_cycle`

Command: `env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-deployment --test deployment adversary_rehashed_bundle_cannot_launder_a_build_cycle -- --exact --nocapture`

Exit: 0. Executed: 1.

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/deployment.rs (target/debug/deps/deployment-f7d4304711ce6995)

running 1 test
test adversary_rehashed_bundle_cannot_launder_a_build_cycle ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.02s
```

Case 3: `adversary_unselected_catalog_candidate_mutation_is_revalidated`

Command: `env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-deployment --test deployment adversary_unselected_catalog_candidate_mutation_is_revalidated -- --exact --nocapture`

Exit: 0. Executed: 1.

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/deployment.rs (target/debug/deps/deployment-f7d4304711ce6995)

running 1 test
test adversary_unselected_catalog_candidate_mutation_is_revalidated ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.03s
```

Case 4: `adversary_duplicate_desired_keys_are_refused_before_any_executor`

Command: `env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-cli --test persisted_delivery adversary_duplicate_desired_keys_are_refused_before_any_executor -- --exact --nocapture`

Exit: 0. Executed: 1.

```text
   Compiling ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 3.73s
     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-15fb048ddae516a0)

running 1 test
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.09s
```

Case 5: `adversary_duplicate_current_keys_block_removal_and_diff`

Command: `env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-cli --test persisted_delivery adversary_duplicate_current_keys_block_removal_and_diff -- --exact --nocapture`

Exit: 0. Executed: 1.

```text
    Finished `test` profile [unoptimized] target(s) in 0.08s
     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-15fb048ddae516a0)

running 1 test
test adversary_duplicate_current_keys_block_removal_and_diff ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.09s
```

Case 6: `adversary_noncanonical_topological_order_is_refused_before_execution`

Command: `env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-cli --test persisted_delivery adversary_noncanonical_topological_order_is_refused_before_execution -- --exact --nocapture`

Exit: 0. Executed: 1.

```text
    Finished `test` profile [unoptimized] target(s) in 0.08s
     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-15fb048ddae516a0)

running 1 test
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.08s
```

Raw isolated results were saved to `target/review-boundaries-3/adversary-isolated-1.log` before starting the package suites. The resource interruption note `adversary-execution-pending-1.md` describes the earlier pause and is superseded by this completed report; it did not count as a pass.

3. Package suites and gates

All commands below ran after all six isolated cases. Measured lane counts:

| Lane | Before (implementor) | After (this pass) | Red |
|---|---:|---:|---:|
| ess-deployment unit / integration / doc | 0 /18 /0 | 0 /21 /0 | 0 |
| ess-cli unit | 11 | 11 | 0 |
| ess-cli command_surface | 5 | 5 | 0 |
| ess-cli command_surface_adversary | 4 | 4 | 0 |
| ess-cli go_conformance | 7 | 7 | 0 |
| ess-cli output_containment | 19 | 19 | 0 |
| ess-cli persisted_delivery | 3 | 6 | 0 |
| Total | 67 | 73 | 0 |

Command: `env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-deployment`

Exit: 0.

```text
   Compiling ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
    Finished `test` profile [unoptimized] target(s) in 1.05s
     Running unittests src/lib.rs (target/debug/deps/ess_deployment-e5782514a987b31e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/deployment.rs (target/debug/deps/deployment-f7d4304711ce6995)

running 21 tests
test input_order_does_not_change_locked_bytes ... ok
test canonical_build_ir_restores_an_omitted_empty_secret_set ... ok
test build_graph_is_canonical_and_projects_executable_buildkit_inputs ... ok
test undeclared_secrets_and_cycles_are_stage_strict_refusals ... ok
test helm_defaults_materialize_typed_secret_slots_without_secret_bytes ... ok
test persisted_component_and_release_readers_refuse_invalid_local_documents ... ok
test component_release_bundle_is_canonical_and_revalidates_after_transport ... ok
test adversary_rehashed_bundle_cannot_launder_a_build_cycle ... ok
test adversary_runtime_readers_match_compiler_slot_and_volume_refusals ... ok
test persisted_documents_preserve_compiler_bytes_across_all_public_reader_routes ... ok
test persisted_convenience_readers_and_catalogs_use_the_checked_boundary ... ok
test persisted_build_readers_refuse_invalid_graphs_and_compiler_constraints ... ok
test mutable_public_documents_are_rechecked_at_consuming_entrypoints ... ok
test adversary_unselected_catalog_candidate_mutation_is_revalidated ... ok
test persisted_deployment_readers_reject_invalid_release_sets_and_canonical_order ... ok
test persisted_lock_readers_preserve_local_service_identity_and_reject_invariants ... ok
test realization_runtime_release_stack_and_environment_form_one_exact_chain ... ok
test persisted_runtime_readers_refuse_local_relationship_and_slot_defects ... ok
test persisted_readers_reject_duplicate_map_keys_before_collection ... ok
test persisted_bundle_checks_original_keys_and_consistently_rehashed_nested_graphs ... ok
test persisted_duplicate_keys_are_rejected_at_every_populated_nested_map ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

   Doc-tests ess_deployment

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-cli`

Exit: 0.

```text
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 1.28s
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
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
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 19 tests
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
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
```

Command: `env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo fmt -p ess-deployment --check`

Exit: 0.

```text

```

Command: `env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo fmt -p ess-cli --check`

Exit: 0.

```text

```

Command: `env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo clippy --locked -p ess-deployment --all-targets -- -D warnings`

Exit: 0.

```text
    Checking ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
    Finished `dev` profile [unoptimized] target(s) in 1.34s
```

Command: `env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo clippy --locked -p ess-cli --all-targets -- -D warnings`

Exit: 0.

```text
    Checking ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
    Checking ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 2.01s
```

The raw package commands/output/exits are also preserved in `target/review-boundaries-3/adversary-package-gates-1.log`. No full-workspace gate was run; integration gates remain the coordinator's responsibility. Cargo output retains build/test durations. No claim is made about aggregate elapsed runtime from asynchronous polling wall times.

4. Findings

Nothing found.

| File:line | Category | Severity | Verdict | Origin | What was measured / what reaches it |
|---|---|---|---|---|---|
| — | — | — | — | — | No findings |

No base executable was assigned and no base execution occurred. If a defect had reproduced, its origin would have remained undecided; no guessed pre-existing attribution was made.

5. Examined ground and limits

- Read the complete implementation diff against45832cc885377b2d61845ee33af14f0293d99e67, the acceptance and independent scoping constraints, F02, changed tests, public entrypoints and their CLI consumers.
- BuildIr and ComponentIr reconstruct the existing authored input for existing compiler checks; RuntimeIr checks recoverable local references, slots, volumes and endpoints. Their invariant-bearing fields remain private; no public mutation bypass was found.
- ReleaseManifest, ReleaseBundle, ReleaseCatalog, StackLock and DeploymentIr route generic/nested Serde admission through checked decoding, and public mutable models have consuming-boundary checks; the new unselected-candidate case exercises catalog mutation beyond the original evidence-clearing example.
- Streaming map visitors are checked by the existing recursive map matrix and the new actual CLI duplicate-key paths. Duplicate source keys are supplied as raw text, never constructed through a map that already lost them.
- Canonical order is compared with the actual lexical Kahn implementation, which inserts newly ready nodes into the ready set after each pop. The new dependency-valid alternate-order case does not assume batch order.
- Composition-local StackLock service identities remain distinct from LockedSystem.system; the original positive/negative cases and byte round-trips ran in the package suite.
- CLI desired/current admission precedes affected/removed analysis and external execution; invalid desired, invalid current removal, duplicate keys and alternate ordering refused without fake calls. The valid executor control prevents zero-call assertions from standing alone.
- No claims about omitted original semantic/realization inputs, component completeness, semantic replica bounds/statefulness, absent lock-selection inputs, authenticity, cache origin, conformance truth, recovery, live services or races/rollback were tested or inferred.
- Only assigned tests and scratch were written; no production mutation probe or manufactured compatibility execution claim was used.

6. Authored paths outside the worktree

None. Tests, logs, reports and temporary fixtures stayed under this managed tree, using its own target and prescribed TMPDIR. The assigned coordinator sccache socket was used for authorized compiler-cache access; no adversary cache lifecycle or configuration mutation occurred. Available bytes:140294836224 before first attempt;140288991232 on resumption;140238409728 at return. All exceed the8589934592-byte reserve.

Tracked tests remain uncommitted for coordinator review. No further adversary writes are pending.

```findings
[]
```
