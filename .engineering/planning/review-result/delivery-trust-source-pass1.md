---
format: aep.planning-md/1
id: review-result:delivery-trust-source-pass1
kind: review-result
status: active
title: Delivery trust source attack, pass 1
relations:
- reviews: story:review-delivery-trust-contract
revision: 1
---
unit: story:review-delivery-trust-contract — handed working tree at base d9c9905546b774c7520755289933c125a005b1d1; source pins 14dd82dc0f980389e210eb223724ab3b0735efc378a86fc6d888b13ebb36d251
verdict: CONFIRMED
cases: executed 49→52, red 2
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3003 retained native paths under /home/timo/.cache/ess-w16-delivery-adversary-1-tmp; exact paths in native-seal.tsv
needs-coordinator: correct the missing initial component/build release-unit qualification; root owns routing and the remaining source pass
git --no-pager diff --stat (inherited uncommitted implementation, as explicitly assigned)

```text
 .github/actions/release-component/action.yml    |  22 +++-
 .github/actions/release-component/release.sh    |  75 +++++++++---
 crates/edge/ess-cli/src/main.rs                 | 149 ++++++++++++++++++++++--
 crates/edge/ess-cli/tests/command_surface.rs    |  31 +++++
 crates/generate/ess-deployment/src/component.rs |   6 +-
 crates/generate/ess-deployment/src/release.rs   |  18 +--
 docs/design/review-delivery-trust.md            |   9 +-
 website/docs/concepts/component-delivery.md     |  75 +++++++++++-
 website/docs/reference/cli.md                   |  40 ++++++-
 website/docs/reference/formats.md               |  15 +++
 10 files changed, 390 insertions(+), 50 deletions(-)
```

The full Git diff above is inherited. Git stat omits the three inherited untracked source files.
The handed-source comparison is the proof of this pass's own change:

```text
 .../edge/ess-cli/tests/delivery_trust.rs           | 86 ++++++++++++++++++++++
 1 file changed, 86 insertions(+)
```

Only crates/edge/ess-cli/tests/delivery_trust.rs changed relative to handed-source: 86 appended
lines, zero deleted or changed existing lines. All other thirteen assigned source pins remain
exactly equal to the handed source. No implementation, action, documentation, existing assertion,
dependency, planning or Git mutation was made. The external-tool fixture is unchanged.
Complete own patch: [test-only.diff](/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/target/review-boundaries-16/adversary-pass-1/test-only.diff), SHA256
90ffc6eb515f7143ed296d9ef9e0c087deb0730d036f0c94c23c50da1bb6d108.
Complete final source mapping: [final-source-pins.json](/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/target/review-boundaries-16/adversary-pass-1/final-source-pins.json).
The two failing tests represent one defect, not two findings.

2. Added cases, focused before the complete suite.

Baseline executed counts come from the handed implementor report: delivery 28 + deployment 21 = 49.
This pass adds three delivery cases; the complete relevant suites execute 31 +21 = 52.

The action case at crates/edge/ess-cli/tests/delivery_trust.rs:1564 asserts that a compiled
component descriptor whose runtime release unit is absent from the supplied build refuses before
the generic check and release tools. It is red. The descriptor is admitted by the real
ComponentSpec parser and compile_component; only its runtime release-unit name is changed.
Actual Bash, ESS, model compilation, canonical IR, suite and report readers run.
The first failure was at test assertion :1579; final formatting moves that assertion to :1585.

Command and environment identity, verbatim:

```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "test", "--locked", "--offline", "-p", "ess-cli", "--test", "delivery_trust", "adversary_component_release_unit_mismatch_stops_action_before_generic_check", "--", "--exact", "--nocapture"]
launch-environment-sha256 cab4228c6e556579428ebd62f41442af05a1e09e338bf724e0996329ba1328cb
```

stdout, verbatim:

```text

running 1 test
test adversary_component_release_unit_mismatch_stops_action_before_generic_check ... FAILED

failures:

failures:
    adversary_component_release_unit_mismatch_stops_action_before_generic_check

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.27s

```

stderr, verbatim:

```text
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.94s
     Running tests/delivery_trust.rs (target/debug/deps/delivery_trust-8268e192ee8c98d2)

thread 'adversary_component_release_unit_mismatch_stops_action_before_generic_check' (1359618) panicked at crates/edge/ess-cli/tests/delivery_trust.rs:1579:5:
incompatible component/build release units must refuse at initial qualification; generic_check_ran=true, external_calls=17, fixture=/home/timo/.cache/ess-w16-delivery-adversary-1-tmp/ess-delivery-trust-1359617-0, stderr=conformance: passed for the supplied exact declared selection
selected suite: sha256:53ff8e14d1af37b36948ee55e15daa39b1a411f5db9289ce285cae83d759b199
selection: {"scope":{"kind":"system"},"origins":"authored","filter":{"kind":"all"}}
model: oracle/v1; spec_digest: 4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee; contract_digest: 9c8f1b65057d7378da54f3072e27e6bb046abd22265bbdf1c1caadb94ecaa1bd
raw report sha256: sha256:fe11c748a821fc7808def18813c6579dab36e5959b5d3eba8b96b10f9886b8bd
component: oracle; system: oracle; semantic_version: v1; digest: sha256:a965dd493e967a7249c1e7b0d41375d4a14d1ab1cabc334a69aece128e13b2d7
build digest: sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8; runtime digest: sha256:5538a120fc1a22dde331fb39808d1e4e72bf0c69a79068049875886ce18b3910; runtime semantic digest: sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee
attachment binding: unverified
producer origin: unverified
artifact execution: unverified
signature verification: unsupported
conformance: passed for the supplied exact declared selection
selected suite: sha256:53ff8e14d1af37b36948ee55e15daa39b1a411f5db9289ce285cae83d759b199
selection: {"scope":{"kind":"system"},"origins":"authored","filter":{"kind":"all"}}
model: oracle/v1; spec_digest: 4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee; contract_digest: 9c8f1b65057d7378da54f3072e27e6bb046abd22265bbdf1c1caadb94ecaa1bd
raw report sha256: sha256:fe11c748a821fc7808def18813c6579dab36e5959b5d3eba8b96b10f9886b8bd
component: oracle; system: oracle; semantic_version: v1; digest: sha256:a965dd493e967a7249c1e7b0d41375d4a14d1ab1cabc334a69aece128e13b2d7
build digest: sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8; runtime digest: sha256:5538a120fc1a22dde331fb39808d1e4e72bf0c69a79068049875886ce18b3910; runtime semantic digest: sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee
attachment binding: unverified
producer origin: unverified
artifact execution: unverified
signature verification: unsupported
conformance: passed for the supplied exact declared selection
selected suite: sha256:53ff8e14d1af37b36948ee55e15daa39b1a411f5db9289ce285cae83d759b199
selection: {"scope":{"kind":"system"},"origins":"authored","filter":{"kind":"all"}}
model: oracle/v1; spec_digest: 4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee; contract_digest: 9c8f1b65057d7378da54f3072e27e6bb046abd22265bbdf1c1caadb94ecaa1bd
raw report sha256: sha256:fe11c748a821fc7808def18813c6579dab36e5959b5d3eba8b96b10f9886b8bd
component: oracle; system: oracle; semantic_version: v1; digest: sha256:a965dd493e967a7249c1e7b0d41375d4a14d1ab1cabc334a69aece128e13b2d7
build digest: sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8; runtime digest: sha256:5538a120fc1a22dde331fb39808d1e4e72bf0c69a79068049875886ce18b3910; runtime semantic digest: sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee
attachment binding: unverified
producer origin: unverified
artifact execution: unverified
signature verification: unsupported
lowering was refused:
[unknown_reference:Release] release includes artifact app not declared by the build

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: test failed, to rerun pass `-p ess-cli --test delivery_trust`
```

Direct status, verbatim:

```text
exit Some(101)
status exit status: 101
pid 1359451
started_at SystemTime { tv_sec: 1788784445, tv_nsec: 994695013 }
ended_at SystemTime { tv_sec: 1788784447, tv_nsec: 222513419 }
elapsed_seconds 1.227822
```

The direct-command case at crates/edge/ess-cli/tests/delivery_trust.rs:1596 asserts that
check-conformance and publish-conformance refuse the same canonical mismatched component context
before ORAS. It is red: both actual commands return 0, and the publisher makes one fixture ORAS
call. This isolates the missing relationship check in the new qualifier from inherited late
release verification. Both command outputs and statuses are retained in its fixture directory.
The first failure was at assertion :1606; final formatting moves that assertion to :1612.

Command and environment identity, verbatim:

```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "test", "--locked", "--offline", "-p", "ess-cli", "--test", "delivery_trust", "adversary_component_release_unit_mismatch_refuses_positive_cli_routes", "--", "--exact", "--nocapture"]
launch-environment-sha256 cab4228c6e556579428ebd62f41442af05a1e09e338bf724e0996329ba1328cb
```

stdout, verbatim:

```text

running 1 test
test adversary_component_release_unit_mismatch_refuses_positive_cli_routes ... FAILED

failures:

failures:
    adversary_component_release_unit_mismatch_refuses_positive_cli_routes

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 30 filtered out; finished in 0.19s

```

stderr, verbatim:

```text
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.87s
     Running tests/delivery_trust.rs (target/debug/deps/delivery_trust-8268e192ee8c98d2)

thread 'adversary_component_release_unit_mismatch_refuses_positive_cli_routes' (1439348) panicked at crates/edge/ess-cli/tests/delivery_trust.rs:1606:5:
component/build release-unit mismatch must refuse positive qualification before ORAS; results=[("check-conformance", Some(0), true), ("publish-conformance", Some(0), false)], external_calls=1, fixture=/home/timo/.cache/ess-w16-delivery-adversary-1-tmp/ess-delivery-trust-1439347-0
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: test failed, to rerun pass `-p ess-cli --test delivery_trust`
```

Direct status, verbatim:

```text
exit Some(101)
status exit status: 101
pid 1439172
started_at SystemTime { tv_sec: 1788784577, tv_nsec: 16820358 }
ended_at SystemTime { tv_sec: 1788784578, tv_nsec: 91696098 }
elapsed_seconds 1.074878
```

The original-path mutation case at crates/edge/ess-cli/tests/delivery_trust.rs:1625 replaces
both caller report.json and expected.json during CHECK_COMMAND. It asserts successful final
publication with the exact pre-check report buffer, while proving both caller files changed.
It is green; no fixture mode or product bypass is needed.

Command and environment identity, verbatim:

```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "test", "--locked", "--offline", "-p", "ess-cli", "--test", "delivery_trust", "adversary_action_keeps_original_snapshots_when_generic_check_replaces_caller_files", "--", "--exact", "--nocapture"]
launch-environment-sha256 cab4228c6e556579428ebd62f41442af05a1e09e338bf724e0996329ba1328cb
```

stdout, verbatim:

```text

running 1 test
test adversary_action_keeps_original_snapshots_when_generic_check_replaces_caller_files ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out; finished in 0.30s

```

stderr, verbatim:

```text
    Finished `test` profile [unoptimized] target(s) in 0.08s
     Running tests/delivery_trust.rs (target/debug/deps/delivery_trust-8268e192ee8c98d2)
```

Direct status, verbatim:

```text
exit Some(0)
status exit status: 0
pid 1445582
started_at SystemTime { tv_sec: 1788784610, tv_nsec: 844036445 }
ended_at SystemTime { tv_sec: 1788784611, tv_nsec: 237581607 }
elapsed_seconds 0.393549
```

All three cases existed before their focused execution. Each focused filter executed exactly one
case. A first format check later reported formatting in only the added lines; its exit1 and
original diff remain in runs/format-check-1. Only those added lines were formatted before the full
suite. No compilation or fixture failure occurred in the behavioral runs. An earlier apply_patch
tool invocation for bootstrap-command.txt was rejected before any file or compiler action; the
corrected invocation and exact actual bootstrap command are retained.

3. Final relevant complete suites.

Delivery 28→31: 29 pass, 2 red; no ignored or filtered cases. All 28 inherited cases pass.

Command and environment identity, verbatim:

```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "test", "--locked", "--offline", "-p", "ess-cli", "--test", "delivery_trust"]
launch-environment-sha256 cab4228c6e556579428ebd62f41442af05a1e09e338bf724e0996329ba1328cb
```

stdout, verbatim:

```text

running 31 tests
test t12_consistently_changed_artifact_platform_and_source_context_remains_execution_unverified ... ok
test adversary_component_release_unit_mismatch_refuses_positive_cli_routes ... FAILED
test t01_t02_placeholders_and_rehashed_evidence_remain_only_consistency_checked ... ok
test t10_complete_but_empty_unfiltered_selection_is_inconclusive ... ok
test t11_neither_expected_variant_and_noncanonical_context_are_refused_without_tools ... ok
test t02_invalid_nested_release_claims_and_duplicate_maps_refuse_before_qualified_upload ... ok
test t06_complete_rust_go_and_filtered_reports_qualify_only_their_exact_selection ... ok
test t13_report_upload_owns_original_bytes_and_distinguishes_attachment_digest ... ok
test t10_in_scope_refusal_beside_nonempty_all_passes_blocks_positive_qualification ... ok
test t11_usage_groups_and_raw_pin_syntax_refuse_before_effects ... ok
test t13_mutating_caller_paths_after_admission_cannot_change_staged_report_or_bundle ... ok
test t12_compiled_deployment_and_every_artifact_context_are_checked_and_named ... ok
test t16_action_input_environment_and_generic_check_failure_contract ... ok
test adversary_component_release_unit_mismatch_stops_action_before_generic_check ... FAILED
test t16_chart_digest_absence_preserves_earlier_effects_and_blocks_all_evidence ... ok
test t08_generated_origin_qualifies_itself_and_refuses_other_origin_expectations ... ok
test adversary_action_keeps_original_snapshots_when_generic_check_replaces_caller_files ... ok
test t15_publisher_failure_is_propagated_before_any_later_evidence_or_bundle_call ... ok
test t15_caller_files_beneath_release_output_get_distinct_action_snapshots ... ok
test t13_oras_bad_digest_non_utf8_and_nonzero_status_fail_both_publishers ... ok
test t17_grouped_flat_streams_and_existing_canonical_outputs_are_identical ... ok
test t03_arbitrary_logs_and_wrong_envelopes_refuse_all_positive_routes ... ok
test t04_legacy_readers_remain_readable_but_no_legacy_summary_qualifies ... ok
test t15_missing_invalid_and_nonqualifying_action_inputs_stop_before_generic_check ... ok
test t09_complete_original_parent_lineage_is_required ... ok
test t15_snapshot_pins_block_mutation_before_evidence_and_final_bundle_publication ... ok
test t10_negative_inconclusive_and_unknown_reports_stop_all_publication_routes ... ok
test t11_exact_tokens_membership_and_duplicate_keys_reach_the_report_reader ... ok
test t16_actual_action_build_adopt_platform_chart_and_summary_paths_preserve_limits ... ok
test t08_scope_origin_and_subset_cannot_satisfy_a_different_expectation ... ok
test t07_original_suite_pairing_and_full_model_identity_refuse_substitution ... ok

failures:

---- adversary_component_release_unit_mismatch_refuses_positive_cli_routes stdout ----

thread 'adversary_component_release_unit_mismatch_refuses_positive_cli_routes' (1467586) panicked at crates/edge/ess-cli/tests/delivery_trust.rs:1612:5:
component/build release-unit mismatch must refuse positive qualification before ORAS; results=[("check-conformance", Some(0), true), ("publish-conformance", Some(0), false)], external_calls=1, fixture=/home/timo/.cache/ess-w16-delivery-adversary-1-tmp/ess-delivery-trust-1467584-1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- adversary_component_release_unit_mismatch_stops_action_before_generic_check stdout ----

thread 'adversary_component_release_unit_mismatch_stops_action_before_generic_check' (1467587) panicked at crates/edge/ess-cli/tests/delivery_trust.rs:1585:5:
incompatible component/build release units must refuse at initial qualification; generic_check_ran=true, external_calls=17, fixture=/home/timo/.cache/ess-w16-delivery-adversary-1-tmp/ess-delivery-trust-1467584-3, stderr=conformance: passed for the supplied exact declared selection
selected suite: sha256:53ff8e14d1af37b36948ee55e15daa39b1a411f5db9289ce285cae83d759b199
selection: {"scope":{"kind":"system"},"origins":"authored","filter":{"kind":"all"}}
model: oracle/v1; spec_digest: 4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee; contract_digest: 9c8f1b65057d7378da54f3072e27e6bb046abd22265bbdf1c1caadb94ecaa1bd
raw report sha256: sha256:fe11c748a821fc7808def18813c6579dab36e5959b5d3eba8b96b10f9886b8bd
component: oracle; system: oracle; semantic_version: v1; digest: sha256:a965dd493e967a7249c1e7b0d41375d4a14d1ab1cabc334a69aece128e13b2d7
build digest: sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8; runtime digest: sha256:5538a120fc1a22dde331fb39808d1e4e72bf0c69a79068049875886ce18b3910; runtime semantic digest: sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee
attachment binding: unverified
producer origin: unverified
artifact execution: unverified
signature verification: unsupported
conformance: passed for the supplied exact declared selection
selected suite: sha256:53ff8e14d1af37b36948ee55e15daa39b1a411f5db9289ce285cae83d759b199
selection: {"scope":{"kind":"system"},"origins":"authored","filter":{"kind":"all"}}
model: oracle/v1; spec_digest: 4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee; contract_digest: 9c8f1b65057d7378da54f3072e27e6bb046abd22265bbdf1c1caadb94ecaa1bd
raw report sha256: sha256:fe11c748a821fc7808def18813c6579dab36e5959b5d3eba8b96b10f9886b8bd
component: oracle; system: oracle; semantic_version: v1; digest: sha256:a965dd493e967a7249c1e7b0d41375d4a14d1ab1cabc334a69aece128e13b2d7
build digest: sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8; runtime digest: sha256:5538a120fc1a22dde331fb39808d1e4e72bf0c69a79068049875886ce18b3910; runtime semantic digest: sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee
attachment binding: unverified
producer origin: unverified
artifact execution: unverified
signature verification: unsupported
conformance: passed for the supplied exact declared selection
selected suite: sha256:53ff8e14d1af37b36948ee55e15daa39b1a411f5db9289ce285cae83d759b199
selection: {"scope":{"kind":"system"},"origins":"authored","filter":{"kind":"all"}}
model: oracle/v1; spec_digest: 4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee; contract_digest: 9c8f1b65057d7378da54f3072e27e6bb046abd22265bbdf1c1caadb94ecaa1bd
raw report sha256: sha256:fe11c748a821fc7808def18813c6579dab36e5959b5d3eba8b96b10f9886b8bd
component: oracle; system: oracle; semantic_version: v1; digest: sha256:a965dd493e967a7249c1e7b0d41375d4a14d1ab1cabc334a69aece128e13b2d7
build digest: sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8; runtime digest: sha256:5538a120fc1a22dde331fb39808d1e4e72bf0c69a79068049875886ce18b3910; runtime semantic digest: sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee
attachment binding: unverified
producer origin: unverified
artifact execution: unverified
signature verification: unsupported
lowering was refused:
[unknown_reference:Release] release includes artifact app not declared by the build



failures:
    adversary_component_release_unit_mismatch_refuses_positive_cli_routes
    adversary_component_release_unit_mismatch_stops_action_before_generic_check

test result: FAILED. 29 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.21s

```

stderr, verbatim:

```text
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.88s
     Running tests/delivery_trust.rs (target/debug/deps/delivery_trust-8268e192ee8c98d2)
error: test failed, to rerun pass `-p ess-cli --test delivery_trust`
```

Direct status, verbatim:

```text
exit Some(101)
status exit status: 101
pid 1467492
started_at SystemTime { tv_sec: 1788784696, tv_nsec: 216952524 }
ended_at SystemTime { tv_sec: 1788784698, tv_nsec: 328578853 }
elapsed_seconds 2.111630
```

Deployment 21→21: 21 pass; no ignored or filtered cases. This target is relevant because the
finding crosses the component/build/runtime context relationship; no deployment test was edited.

Command and environment identity, verbatim:

```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "test", "--locked", "--offline", "-p", "ess-deployment", "--test", "deployment"]
launch-environment-sha256 cab4228c6e556579428ebd62f41442af05a1e09e338bf724e0996329ba1328cb
```

stdout, verbatim:

```text

running 21 tests
test input_order_does_not_change_locked_bytes ... ok
test canonical_build_ir_restores_an_omitted_empty_secret_set ... ok
test build_graph_is_canonical_and_projects_executable_buildkit_inputs ... ok
test undeclared_secrets_and_cycles_are_stage_strict_refusals ... ok
test helm_defaults_materialize_typed_secret_slots_without_secret_bytes ... ok
test component_release_bundle_is_canonical_and_revalidates_after_transport ... ok
test realization_runtime_release_stack_and_environment_form_one_exact_chain ... ok
test persisted_component_and_release_readers_refuse_invalid_local_documents ... ok
test adversary_rehashed_bundle_cannot_launder_a_build_cycle ... ok
test persisted_build_readers_refuse_invalid_graphs_and_compiler_constraints ... ok
test adversary_runtime_readers_match_compiler_slot_and_volume_refusals ... ok
test persisted_readers_reject_duplicate_map_keys_before_collection ... ok
test mutable_public_documents_are_rechecked_at_consuming_entrypoints ... ok
test adversary_unselected_catalog_candidate_mutation_is_revalidated ... ok
test persisted_deployment_readers_reject_invalid_release_sets_and_canonical_order ... ok
test persisted_convenience_readers_and_catalogs_use_the_checked_boundary ... ok
test persisted_documents_preserve_compiler_bytes_across_all_public_reader_routes ... ok
test persisted_runtime_readers_refuse_local_relationship_and_slot_defects ... ok
test persisted_lock_readers_preserve_local_service_identity_and_reject_invariants ... ok
test persisted_bundle_checks_original_keys_and_consistently_rehashed_nested_graphs ... ok
test persisted_duplicate_keys_are_rejected_at_every_populated_nested_map ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

```

stderr, verbatim:

```text
   Compiling serde_json v1.0.151
   Compiling memchr v2.8.3
   Compiling semver v1.0.28
   Compiling schemars v0.8.22
   Compiling ess-primitives v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/crates/specify/ess-primitives)
   Compiling ess-domain v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/crates/specify/ess-domain)
   Compiling ess-compiler v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/crates/specify/ess-compiler)
   Compiling ess-realization v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/crates/specify/ess-realization)
   Compiling ess-deployment v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/crates/generate/ess-deployment)
    Finished `test` profile [unoptimized] target(s) in 9.52s
     Running tests/deployment.rs (target/debug/deps/deployment-552190fc5f4292de)
```

Direct status, verbatim:

```text
exit Some(0)
status exit status: 0
pid 1475190
started_at SystemTime { tv_sec: 1788784765, tv_nsec: 77556106 }
ended_at SystemTime { tv_sec: 1788784774, tv_nsec: 743061638 }
elapsed_seconds 9.665509
```

Affected-package strict Clippy passed:
`cargo clippy -p ess-cli -p ess-deployment --all-targets --locked --offline -- -D warnings`.
Final formatting passed:
`cargo fmt -p ess-cli -p ess-deployment --check`.
Their original streams, child PIDs/times/direct exits, source/tool snapshots and seals are retained
under [runs/final-clippy](/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/target/review-boundaries-16/adversary-pass-1/runs/final-clippy) and [runs/final-format](/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/target/review-boundaries-16/adversary-pass-1/runs/final-format).
No complete package, browser, cache or conformance lane was rerun; root owns the full gate, as assigned.

4. Finding and reachability.

| Source | Verdict | Origin | Measured result and reachable route |
|---|---|---|---|
| crates/edge/ess-cli/src/release_evidence.rs:138 | CONFIRMED | introduced | The new conformance qualifier accepts a component runtime release unit absent from the supplied build, so both positive CLI routes succeed and the action performs generic checks and four evidence uploads before its later release verifier refuses. |

What was measured: the action assertion at delivery_trust.rs:1585 fails after
CHECK_COMMAND ran and 17 external fixture calls completed. Calls 013–016 are the provenance,
SBOM, signature and conformance uploads. The actual action exits 1 only when existing release
verification rejects app as not declared by the build. The CLI assertion at :1612 fails with
check-conformance 0, publish-conformance 0 and one ORAS call. Both focused Rust test producers and
the final delivery suite exit 101. There is no bundle upload in the mismatched action route.

What reaches it: the documented action's component-path input can name this ordinary valid
ess-component/1 descriptor. Component compilation writes its canonical IR; release.sh invokes
check-conformance with that IR and the supplied build/runtime at :78, then CHECK_COMMAND and the
build/adoption/evidence work. The direct documented --component-ir option supplies the same
canonical bytes to both new commands; main.rs:1608–1609 calls deployment then Inputs::qualify.
B03.3 requires checking existing context relationships, B06.1 requires initial qualification
before the generic check/release work, and T12 requires incompatible supplied context to fail
early. The missing comparison is between existing declared release units and actual build outputs;
it needs no modeled-component inference or new persisted relation.

Attribution: release_evidence.rs and the two positive commands are new in this working-tree
implementation and absent from base. Their positive qualification of the incompatible context is
introduced. The action's final release validation and its existing ability to discover this
mismatch late are inherited; I do not attribute that old behavior to this change or claim that
an invalid final bundle was published. This finding concerns the newly promised early
qualification boundary. The original report's own exact selection remains valid.

Suggested correction for root to route: have the new qualifier reject incompatible declared
component release-unit/build-output relationships using the existing typed values before
positive qualification or effects. Preserve the report reader and conservative execution limits.
No correction was applied here.

Root separately supplied a documentation formatting observation: website/docs/reference/formats.md
places the ess-deployment-diff/1 row after the new paragraphs. Root owns that mechanical correction;
it is excluded from this measured behavioral finding and the findings block.

5. Attacked without breaking.

The new original-path replacement case preserves the action's captured report/input snapshots and
exact checked upload bytes through every later gate.
The 28 inherited delivery cases remain green across exact report/selection/model admission,
negative and malformed inputs, private staging, caller mutation, child failures and conservative claims.
The unchanged 21 deployment cases remain green across canonical bytes, nested revalidation,
duplicate maps and consistently rehashed invalid graphs.
This is a bounded source attack; it authenticates no producer, signature, attachment relation
or runtime/chart execution, and supplies no approval.

6. Retention, resources, outside paths and quiescence.

All scratch written by this pass is beneath /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/target/review-boundaries-16/adversary-pass-1. The only
outside-worktree write root is /home/timo/.cache/ess-w16-delivery-adversary-1-tmp.
The final native manifest enumerates all 3003 retained native entries under that root, in full
native-byte hexadecimal; literal symlink targets are also encoded without traversal. Actual
publication-stage paths that the product subsequently retired appear in the corresponding
fixture calls/*/cwd receipts. Existing compiler/product temporary lifetimes remain owned by those
processes; no manual cleanup was performed. The older implementation TMP root was only counted and
read for the final census. HOME was preserved.

The copied bounded Rust runner used the exact launch-environment.json, emptied all four wrappers,
unset shared target/sccache variables, used the default unit target, offline private Cargo/Go
inputs, two build jobs and assigned TMP/XDG paths. Every recorded producer has a never-reused
explicit coverage export override in its own coverage-override.txt. Source snapshots include
copies of every assigned file. Its source.tsv recorder-hash prefix contains literal backslash-t
and backslash-n escapes; that metadata formatting is retained unchanged, while the source copies
and final-source-pins.json provide an unambiguous complete source mapping.

Every runner-managed producer completed full before/after frozen Rust/Firefox/local-tool checks,
resource measurements and native path inventories. These are endpoint checks, not continuous
resource polling. The measured combined maximum was 7,216,877,568 bytes against 12,884,901,888;
minimum available space was 52,255,653,888 bytes against the 8,589,934,592 floor. All three
assigned counted roots were included. Each direct child status was saved before postprocessing;
every started compiler/test/tool process was awaited before handoff.

The std-only runner bootstrap completed at 2026-09-07T12:33:33.841056381Z with direct status 0,
PID 1352495; its exact command, stdout/stderr and PID/start/end/status files are under bootstrap/.
The root-supplied full pre-bootstrap manifest check completed earlier at 12:32:07.840971Z:
[bootstrap-tool-readback.json](/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-16/preparation/delivery-source-pass-1/bootstrap-tool-readback.json),
SHA256 07225ebb709ed1be86b6c9466d01ac441c1598ef130630ede97cb3d77b6b1494.
The dispatch resource record and first native runner precheck bracket the small bootstrap;
subsequent producers have their own exact before/after resource records.

Final native sealing completed with direct status 0:

```text
Sealed 52421 regular files (7385709191 logical bytes), 72467 native entries. Paths and literal symlink targets are encoded as native-byte hexadecimal without following links. Excludes this manifest, its owning producer record, report.md, quiescence.json and later handoff.sha256; these are separately bound by handoff.sha256.
exit Some(0)
status exit status: 0
pid 1581599
started_at SystemTime { tv_sec: 1788785006, tv_nsec: 913723799 }
ended_at SystemTime { tv_sec: 1788785032, tv_nsec: 699340402 }
elapsed_seconds 25.785620
```

Native manifest SHA256:e04e123379299b7287ecdff43b09cc57f9c4111f689bab29f200d2b04a037dee.
Final seal-producer receipt SHA256:31b5fbce27499976748111326229b518ce6e96c6ef16e5a981b88786c711b0aa.
The native census explicitly excludes only its own manifest, the owning final seal run directory,
this later report, quiescence.json and handoff.sha256; the final handoff binds those separately.
It retains 52,421 regular files / 7,385,709,191 logical bytes and 72,467 native entries across all 3 roots.
Every original focused, suite, formatting and compiler output remains retained.

After final sealing, the native /proc observer exited 0 with no other owned cwd/executable process:

```json
{"observer_pid":1585778,"observed_at_unix_ns":1788785036773327656,"owned_processes":[]}
```

The worktree is quiescent. Source and tool identities plus all run seals, this complete report,
the native manifest and the final observer record are bound by [handoff.sha256](/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/target/review-boundaries-16/adversary-pass-1/handoff.sha256).
Root owns correction routing, the remaining source review pass, full gates, AEP/Git/publication
and all managed cleanup.

```findings
- file: crates/edge/ess-cli/src/release_evidence.rs
  line: 138
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: The new conformance qualifier accepts a component runtime release unit absent from the supplied build, so both positive CLI routes succeed and the action performs generic checks and four evidence uploads before its later release verifier refuses.
```
