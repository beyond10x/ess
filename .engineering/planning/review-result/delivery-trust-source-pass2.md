---
format: aep.planning-md/1
id: review-result:delivery-trust-source-pass2
kind: review-result
status: active
title: Delivery trust source review, final pass 2
relations:
- reviews: story:review-delivery-trust-contract
revision: 1
---
unit: story:review-delivery-trust-contract — second and final source attack; working tree /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract, branch impl/review-delivery-trust-contract, base d9c9905546b774c7520755289933c125a005b1d1; final-source-pins.json identifies reviewed bytes
verdict: nothing found
cases: executed 56→59, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 6993 retained native paths in assigned /home/timo/.cache/ess-w16-delivery-adversary-2-tmp; exhaustive paths in outside-paths.tsv
needs-coordinator: none
git --no-pager diff --stat
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

1. Scope proof against the handed source snapshot

The stat above is the complete inherited dirty unit diff against its base, as the filled work order requires. Its production, action and documentation paths were handed to this pass; they are not this adversary's edits. Git omits inherited untracked additions from that stat, including delivery_trust.rs; the retained git-status.txt and fourteen-file source pins make those paths explicit. The exact own test-only delta is test-only.diff against handed-source/. It appends three cases and two helpers only to crates/edge/ess-cli/tests/delivery_trust.rs. Every existing assertion and the complete handed test byte prefix remain unchanged, as do the other thirteen handed paths. No production byte, action, document, dependency, planning record, Git ref or index was changed by this pass. Scratch reporting and producer code remain in the assigned target directory. The work order explicitly permits this inherited-diff distinction.

The complete adversary 0.8.0 charter, local AGENTS.md, active story, accepted B01–B07/T01–T18 binding, public CLI/formats/delivery claims, actual action, changed owners/callers, original implementation report, previous finding and complete correction report were read. Review covered the whole handed implementation after correction, not only the prior missing-runtime-unit fix.

Exact own scope check:
```text
13 handed paths unchanged byte-for-byte; complete handed delivery_trust.rs byte prefix unchanged; 299 appended lines; 3 new tests and 2 helpers.
```

2. Newly written cases and focused execution

All cases were justified in cases-before-run.md and written before their first focused execution. All are green now. The first action model variant and its initial red output remain intact as a test-fixture mistake, described below; it is not included as a product red in the header.

- crates/edge/ess-cli/tests/delivery_trust.rs:1786, adversary2_coherent_wrong_model_bundle_refuses_qualification_after_plain_consistency. Measured: each coherently rehashed alternate system, semantic version and semantic digest passes typed bundle validation, YAML roundtrip, and the real plain verify-bundle command with conformance unassessed and exact canonical stdout; qualified publish then exits 1 for explicit deployment/model identity mismatch before any ORAS call. Reachability: an ordinary supplied bundle path plus the documented explicit model/report/selection flags reaches this distinction. No valid cryptographic signature or producer authentication is inferred. First focused execution passed.
- crates/edge/ess-cli/tests/delivery_trust.rs:1892, adversary2_action_rechecks_current_model_and_deployment_before_evidence_upload. Measured: CHECK_COMMAND replaces the current model with an independently compiled same-version semantic change, or supplies a valid changed component chart or runtime semantic digest. The real action completes its first positive gate and thirteen finite external calls, then refuses at the second current-context gate; it makes no evidence/bundle ORAS upload and writes no summary. Reachability: the documented generic CHECK_COMMAND and supplied compiled component/runtime inputs are ordinary action inputs. The initial model fixture merely appended a YAML comment and incorrectly expected a semantic identity change. Its focused exit 101, complete child stdout/stderr and all eighteen successful finite action calls are preserved. EssIr::source_digest (crates/specify/ess-compiler/src/ir.rs:1569–1590) hashes semantic compact model content; SuiteProvenance::of uses it, so comments do not change the bound identity. The correction changes a real invariant predicate from weight_grams >= 0 to >= 1 and explicitly proves equal system/version plus unequal compiled provenance digest before running the action. This was a new fixture assumption error, not a product finding. The corrected focused execution passed before any full suite ran. Root independently raised the same distinction; fixture-correction.md retains the analysis.
- crates/edge/ess-cli/tests/delivery_trust.rs:1962, adversary2_post_admission_model_and_deployment_replacement_keeps_owned_context. Measured: after the existing finite ORAS fixture signals admission, replacing the caller's model, component, build, runtime and bundle with invalid bytes still publishes the original exact report or canonical bundle, emits the admitted identities, and makes exactly one upload. Reachability: publish-conformance and qualified publish are ordinary CLI routes; deterministic post-admission replacement is fixture-synchronized to examine the admitted-buffer boundary. It does not claim that a hostile local writer or execution backend is part of the supported trust contract. First focused execution passed.

`focused-coherent-bundle` — exact command, direct result, stdout, stderr:

command.txt
```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "test", "--locked", "--offline", "-p", "ess-cli", "--test", "delivery_trust", "adversary2_coherent_wrong_model_bundle_refuses_qualification_after_plain_consistency", "--", "--exact", "--nocapture"]
launch-environment-sha256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634
```

exit.txt
```text
exit Some(0)
status exit status: 0
pid 2263221
started_at SystemTime { tv_sec: 1788788319, tv_nsec: 706555164 }
ended_at SystemTime { tv_sec: 1788788321, tv_nsec: 148868446 }
elapsed_seconds 1.442316
```

stdout.raw
```text

running 1 test
test adversary2_coherent_wrong_model_bundle_refuses_qualification_after_plain_consistency ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 37 filtered out; finished in 0.29s

```

stderr.raw
```text
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 1.13s
     Running tests/delivery_trust.rs (target/debug/deps/delivery_trust-8268e192ee8c98d2)
```

`focused-action-recheck` — exact command, direct result, stdout, stderr:

command.txt
```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "test", "--locked", "--offline", "-p", "ess-cli", "--test", "delivery_trust", "adversary2_action_rechecks_current_model_and_deployment_before_evidence_upload", "--", "--exact", "--nocapture"]
launch-environment-sha256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634
```

exit.txt
```text
exit Some(101)
status exit status: 101
pid 2267327
started_at SystemTime { tv_sec: 1788788356, tv_nsec: 363082012 }
ended_at SystemTime { tv_sec: 1788788356, tv_nsec: 738716103 }
elapsed_seconds 0.375641
```

stdout.raw
```text

running 1 test
test adversary2_action_rechecks_current_model_and_deployment_before_evidence_upload ... FAILED

failures:

failures:
    adversary2_action_rechecks_current_model_and_deployment_before_evidence_upload

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 37 filtered out; finished in 0.29s

```

stderr.raw
```text
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running tests/delivery_trust.rs (target/debug/deps/delivery_trust-8268e192ee8c98d2)

thread 'adversary2_action_rechecks_current_model_and_deployment_before_evidence_upload' (2267340) panicked at crates/edge/ess-cli/tests/delivery_trust.rs:1864:9:
assertion `left == right` failed: model: conformance: passed for the supplied exact declared selection
selected suite: sha256:53ff8e14d1af37b36948ee55e15daa39b1a411f5db9289ce285cae83d759b199
selection: {"scope":{"kind":"system"},"origins":"authored","filter":{"kind":"all"}}
model: oracle/v1; spec_digest: 4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee; contract_digest: 9c8f1b65057d7378da54f3072e27e6bb046abd22265bbdf1c1caadb94ecaa1bd
raw report sha256: sha256:fe11c748a821fc7808def18813c6579dab36e5959b5d3eba8b96b10f9886b8bd
component: oracle; system: oracle; semantic_version: v1; digest: sha256:7711933d4e6c20b9b798f5ece75be7eb6f44142fa75cb288d6a2b2a6dfd3b684
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
component: oracle; system: oracle; semantic_version: v1; digest: sha256:7711933d4e6c20b9b798f5ece75be7eb6f44142fa75cb288d6a2b2a6dfd3b684
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
component: oracle; system: oracle; semantic_version: v1; digest: sha256:7711933d4e6c20b9b798f5ece75be7eb6f44142fa75cb288d6a2b2a6dfd3b684
build digest: sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8; runtime digest: sha256:5538a120fc1a22dde331fb39808d1e4e72bf0c69a79068049875886ce18b3910; runtime semantic digest: sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee
attachment binding: unverified
producer origin: unverified
artifact execution: unverified
signature verification: unsupported
release consistency: checked
conformance: not assessed
attachment binding: unverified
producer origin: unverified
artifact execution: unverified
signature verification: unsupported
release consistency: checked
conformance: not assessed
attachment binding: unverified
producer origin: unverified
artifact execution: unverified
signature verification: unsupported
release consistency: checked
conformance: not assessed
attachment binding: unverified
producer origin: unverified
artifact execution: unverified
signature verification: unsupported
release consistency: checked
conformance: passed for the supplied exact declared selection
selected suite: sha256:53ff8e14d1af37b36948ee55e15daa39b1a411f5db9289ce285cae83d759b199
selection: {"scope":{"kind":"system"},"origins":"authored","filter":{"kind":"all"}}
model: oracle/v1; spec_digest: 4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee; contract_digest: 9c8f1b65057d7378da54f3072e27e6bb046abd22265bbdf1c1caadb94ecaa1bd
raw report sha256: sha256:fe11c748a821fc7808def18813c6579dab36e5959b5d3eba8b96b10f9886b8bd
component: oracle; system: oracle; semantic_version: v1; digest: sha256:7711933d4e6c20b9b798f5ece75be7eb6f44142fa75cb288d6a2b2a6dfd3b684
build digest: sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8; runtime digest: sha256:5538a120fc1a22dde331fb39808d1e4e72bf0c69a79068049875886ce18b3910; runtime semantic digest: sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee
release unit: oracle-chart; version: 1.2.3; source_commit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; semantic_digest: sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee; build_digest: sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8; runtime_digest: sha256:5538a120fc1a22dde331fb39808d1e4e72bf0c69a79068049875886ce18b3910
artifact: chart; build_output: chart; kind: "helm_chart"; reference: registry.example/charts/oracle-chart; digest: sha256:3333333333333333333333333333333333333333333333333333333333333333; platforms: {}
release unit: oracle-runtime; version: 1.2.3; source_commit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; semantic_digest: sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee; build_digest: sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8; runtime_digest: sha256:5538a120fc1a22dde331fb39808d1e4e72bf0c69a79068049875886ce18b3910
artifact: app; build_output: app; kind: "oci_image"; reference: registry.example/oracle; digest: sha256:1111111111111111111111111111111111111111111111111111111111111111; platforms: {"linux/amd64":"sha256:2222222222222222222222222222222222222222222222222222222222222222"}
attachment binding: unverified
producer origin: unverified
artifact execution: unverified
signature verification: unsupported

  left: Some(0)
 right: Some(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: test failed, to rerun pass `-p ess-cli --test delivery_trust`
```

`focused-action-recheck-corrected` — exact command, direct result, stdout, stderr:

command.txt
```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "test", "--locked", "--offline", "-p", "ess-cli", "--test", "delivery_trust", "adversary2_action_rechecks_current_model_and_deployment_before_evidence_upload", "--", "--exact", "--nocapture"]
launch-environment-sha256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634
```

exit.txt
```text
exit Some(0)
status exit status: 0
pid 2281801
started_at SystemTime { tv_sec: 1788788522, tv_nsec: 49184935 }
ended_at SystemTime { tv_sec: 1788788523, tv_nsec: 355191667 }
elapsed_seconds 1.306013
```

stdout.raw
```text

running 1 test
test adversary2_action_rechecks_current_model_and_deployment_before_evidence_upload ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 37 filtered out; finished in 0.44s

```

stderr.raw
```text
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.85s
     Running tests/delivery_trust.rs (target/debug/deps/delivery_trust-8268e192ee8c98d2)
```

`focused-owned-context` — exact command, direct result, stdout, stderr:

command.txt
```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "test", "--locked", "--offline", "-p", "ess-cli", "--test", "delivery_trust", "adversary2_post_admission_model_and_deployment_replacement_keeps_owned_context", "--", "--exact", "--nocapture"]
launch-environment-sha256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634
```

exit.txt
```text
exit Some(0)
status exit status: 0
pid 2285893
started_at SystemTime { tv_sec: 1788788559, tv_nsec: 404106804 }
ended_at SystemTime { tv_sec: 1788788559, tv_nsec: 701793679 }
elapsed_seconds 0.297690
```

stdout.raw
```text

running 1 test
test adversary2_post_admission_model_and_deployment_replacement_keeps_owned_context ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 37 filtered out; finished in 0.21s

```

stderr.raw
```text
    Finished `test` profile [unoptimized] target(s) in 0.08s
     Running tests/delivery_trust.rs (target/debug/deps/delivery_trust-8268e192ee8c98d2)
```

3. Relevant complete suites and checks, after focused execution

The handed correction supplied the actual baseline: delivery_trust 35 plus deployment 21 = 56. This pass ran delivery_trust 38 and deployment 21 = 59, with zero failed, ignored or filtered cases. The inherited 323-case affected-package and 18-control results were read, not rerun or claimed as this pass's execution. The original final-delivery run passed before Clippy flagged only the new action test length; extracting its model preparation with unchanged assertions was followed by final-delivery-refactored and strict Clippy. Both the original lint refusal and subsequent green result are retained below.

`format-own-tests` — exact command, direct result, stdout, stderr:

command.txt
```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-12/preparation/toolchain-snapshot/bin/rustfmt", "--edition", "2021", "--config", "skip_children=true", "crates/edge/ess-cli/tests/delivery_trust.rs"]
launch-environment-sha256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634
```

exit.txt
```text
exit Some(0)
status exit status: 0
pid 2297557
started_at SystemTime { tv_sec: 1788788637, tv_nsec: 135266058 }
ended_at SystemTime { tv_sec: 1788788637, tv_nsec: 160383768 }
elapsed_seconds 0.025121
```

stdout.raw
```text
```

stderr.raw
```text
```

`final-delivery` — exact command, direct result, stdout, stderr:

command.txt
```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "test", "--locked", "--offline", "-p", "ess-cli", "--test", "delivery_trust"]
launch-environment-sha256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634
```

exit.txt
```text
exit Some(0)
status exit status: 0
pid 2300364
started_at SystemTime { tv_sec: 1788788662, tv_nsec: 215607788 }
ended_at SystemTime { tv_sec: 1788788664, tv_nsec: 100592839 }
elapsed_seconds 1.884989
```

stdout.raw
```text

running 38 tests
test adversary_component_release_unit_mismatch_refuses_positive_cli_routes ... ok
test fix1_mixed_and_additional_build_outputs_preserve_qualification_and_publication ... ok
test t10_complete_but_empty_unfiltered_selection_is_inconclusive ... ok
test fix1_chart_release_unit_requires_an_owned_chart_output ... ok
test fix1_missing_chart_release_unit_refuses_before_checks_or_tools ... ok
test fix1_runtime_release_unit_requires_an_owned_oci_output ... ok
test adversary_component_release_unit_mismatch_stops_action_before_generic_check ... ok
test t01_t02_placeholders_and_rehashed_evidence_remain_only_consistency_checked ... ok
test t02_invalid_nested_release_claims_and_duplicate_maps_refuse_before_qualified_upload ... ok
test adversary2_post_admission_model_and_deployment_replacement_keeps_owned_context ... ok
test t06_complete_rust_go_and_filtered_reports_qualify_only_their_exact_selection ... ok
test t12_consistently_changed_artifact_platform_and_source_context_remains_execution_unverified ... ok
test t08_generated_origin_qualifies_itself_and_refuses_other_origin_expectations ... ok
test t13_report_upload_owns_original_bytes_and_distinguishes_attachment_digest ... ok
test t10_in_scope_refusal_beside_nonempty_all_passes_blocks_positive_qualification ... ok
test t11_neither_expected_variant_and_noncanonical_context_are_refused_without_tools ... ok
test t12_compiled_deployment_and_every_artifact_context_are_checked_and_named ... ok
test adversary2_coherent_wrong_model_bundle_refuses_qualification_after_plain_consistency ... ok
test t13_mutating_caller_paths_after_admission_cannot_change_staged_report_or_bundle ... ok
test t11_usage_groups_and_raw_pin_syntax_refuse_before_effects ... ok
test adversary_action_keeps_original_snapshots_when_generic_check_replaces_caller_files ... ok
test t16_action_input_environment_and_generic_check_failure_contract ... ok
test t09_complete_original_parent_lineage_is_required ... ok
test t03_arbitrary_logs_and_wrong_envelopes_refuse_all_positive_routes ... ok
test t16_chart_digest_absence_preserves_earlier_effects_and_blocks_all_evidence ... ok
test t15_publisher_failure_is_propagated_before_any_later_evidence_or_bundle_call ... ok
test t15_caller_files_beneath_release_output_get_distinct_action_snapshots ... ok
test t04_legacy_readers_remain_readable_but_no_legacy_summary_qualifies ... ok
test t13_oras_bad_digest_non_utf8_and_nonzero_status_fail_both_publishers ... ok
test t17_grouped_flat_streams_and_existing_canonical_outputs_are_identical ... ok
test adversary2_action_rechecks_current_model_and_deployment_before_evidence_upload ... ok
test t10_negative_inconclusive_and_unknown_reports_stop_all_publication_routes ... ok
test t15_missing_invalid_and_nonqualifying_action_inputs_stop_before_generic_check ... ok
test t08_scope_origin_and_subset_cannot_satisfy_a_different_expectation ... ok
test t11_exact_tokens_membership_and_duplicate_keys_reach_the_report_reader ... ok
test t15_snapshot_pins_block_mutation_before_evidence_and_final_bundle_publication ... ok
test t07_original_suite_pairing_and_full_model_identity_refuse_substitution ... ok
test t16_actual_action_build_adopt_platform_chart_and_summary_paths_preserve_limits ... ok

test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.99s

```

stderr.raw
```text
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.87s
     Running tests/delivery_trust.rs (target/debug/deps/delivery_trust-8268e192ee8c98d2)
```

`final-deployment` — exact command, direct result, stdout, stderr:

command.txt
```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "test", "--locked", "--offline", "-p", "ess-deployment", "--test", "deployment"]
launch-environment-sha256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634
```

exit.txt
```text
exit Some(0)
status exit status: 0
pid 2304483
started_at SystemTime { tv_sec: 1788788692, tv_nsec: 563562383 }
ended_at SystemTime { tv_sec: 1788788692, tv_nsec: 756500016 }
elapsed_seconds 0.192948
```

stdout.raw
```text

running 21 tests
test input_order_does_not_change_locked_bytes ... ok
test canonical_build_ir_restores_an_omitted_empty_secret_set ... ok
test build_graph_is_canonical_and_projects_executable_buildkit_inputs ... ok
test undeclared_secrets_and_cycles_are_stage_strict_refusals ... ok
test helm_defaults_materialize_typed_secret_slots_without_secret_bytes ... ok
test realization_runtime_release_stack_and_environment_form_one_exact_chain ... ok
test adversary_rehashed_bundle_cannot_launder_a_build_cycle ... ok
test component_release_bundle_is_canonical_and_revalidates_after_transport ... ok
test persisted_component_and_release_readers_refuse_invalid_local_documents ... ok
test adversary_runtime_readers_match_compiler_slot_and_volume_refusals ... ok
test persisted_documents_preserve_compiler_bytes_across_all_public_reader_routes ... ok
test mutable_public_documents_are_rechecked_at_consuming_entrypoints ... ok
test adversary_unselected_catalog_candidate_mutation_is_revalidated ... ok
test persisted_build_readers_refuse_invalid_graphs_and_compiler_constraints ... ok
test persisted_readers_reject_duplicate_map_keys_before_collection ... ok
test persisted_runtime_readers_refuse_local_relationship_and_slot_defects ... ok
test persisted_convenience_readers_and_catalogs_use_the_checked_boundary ... ok
test persisted_lock_readers_preserve_local_service_identity_and_reject_invariants ... ok
test persisted_deployment_readers_reject_invalid_release_sets_and_canonical_order ... ok
test persisted_bundle_checks_original_keys_and_consistently_rehashed_nested_graphs ... ok
test persisted_duplicate_keys_are_rejected_at_every_populated_nested_map ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

```

stderr.raw
```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/deployment.rs (target/debug/deps/deployment-552190fc5f4292de)
```

`final-clippy` — exact command, direct result, stdout, stderr:

command.txt
```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "clippy", "-p", "ess-cli", "-p", "ess-deployment", "--all-targets", "--locked", "--offline", "--", "-D", "warnings"]
launch-environment-sha256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634
```

exit.txt
```text
exit Some(101)
status exit status: 101
pid 2306356
started_at SystemTime { tv_sec: 1788788708, tv_nsec: 107867091 }
ended_at SystemTime { tv_sec: 1788788708, tv_nsec: 446972753 }
elapsed_seconds 0.339110
```

stdout.raw
```text
```

stderr.raw
```text
    Checking ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/crates/edge/ess-cli)
error: this function has too many lines (104/100)
    --> crates/edge/ess-cli/tests/delivery_trust.rs:1851:1
     |
1851 | fn adversary2_action_rechecks_current_model_and_deployment_before_evidence_upload() {
     | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |
     = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_lines
     = note: `-D clippy::too-many-lines` implied by `-D warnings`
     = help: to override `-D warnings` add `#[allow(clippy::too_many_lines)]`

error: could not compile `ess-cli` (test "delivery_trust") due to 1 previous error
```

`format-own-tests-clippy` — exact command, direct result, stdout, stderr:

command.txt
```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-12/preparation/toolchain-snapshot/bin/rustfmt", "--edition", "2021", "--config", "skip_children=true", "crates/edge/ess-cli/tests/delivery_trust.rs"]
launch-environment-sha256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634
```

exit.txt
```text
exit Some(0)
status exit status: 0
pid 2323243
started_at SystemTime { tv_sec: 1788788910, tv_nsec: 62024011 }
ended_at SystemTime { tv_sec: 1788788910, tv_nsec: 86367319 }
elapsed_seconds 0.024346
```

stdout.raw
```text
```

stderr.raw
```text
```

`final-delivery-refactored` — exact command, direct result, stdout, stderr:

command.txt
```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "test", "--locked", "--offline", "-p", "ess-cli", "--test", "delivery_trust"]
launch-environment-sha256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634
```

exit.txt
```text
exit Some(0)
status exit status: 0
pid 2329538
started_at SystemTime { tv_sec: 1788788949, tv_nsec: 65319249 }
ended_at SystemTime { tv_sec: 1788788951, tv_nsec: 73713901 }
elapsed_seconds 2.008403
```

stdout.raw
```text

running 38 tests
test adversary_component_release_unit_mismatch_stops_action_before_generic_check ... ok
test t01_t02_placeholders_and_rehashed_evidence_remain_only_consistency_checked ... ok
test t10_complete_but_empty_unfiltered_selection_is_inconclusive ... ok
test t02_invalid_nested_release_claims_and_duplicate_maps_refuse_before_qualified_upload ... ok
test adversary_component_release_unit_mismatch_refuses_positive_cli_routes ... ok
test fix1_mixed_and_additional_build_outputs_preserve_qualification_and_publication ... ok
test adversary2_post_admission_model_and_deployment_replacement_keeps_owned_context ... ok
test t06_complete_rust_go_and_filtered_reports_qualify_only_their_exact_selection ... ok
test fix1_runtime_release_unit_requires_an_owned_oci_output ... ok
test fix1_chart_release_unit_requires_an_owned_chart_output ... ok
test fix1_missing_chart_release_unit_refuses_before_checks_or_tools ... ok
test t12_consistently_changed_artifact_platform_and_source_context_remains_execution_unverified ... ok
test t13_report_upload_owns_original_bytes_and_distinguishes_attachment_digest ... ok
test adversary2_coherent_wrong_model_bundle_refuses_qualification_after_plain_consistency ... ok
test t11_neither_expected_variant_and_noncanonical_context_are_refused_without_tools ... ok
test t13_mutating_caller_paths_after_admission_cannot_change_staged_report_or_bundle ... ok
test t08_generated_origin_qualifies_itself_and_refuses_other_origin_expectations ... ok
test t10_in_scope_refusal_beside_nonempty_all_passes_blocks_positive_qualification ... ok
test adversary_action_keeps_original_snapshots_when_generic_check_replaces_caller_files ... ok
test t16_action_input_environment_and_generic_check_failure_contract ... ok
test t12_compiled_deployment_and_every_artifact_context_are_checked_and_named ... ok
test t11_usage_groups_and_raw_pin_syntax_refuse_before_effects ... ok
test t16_chart_digest_absence_preserves_earlier_effects_and_blocks_all_evidence ... ok
test t15_publisher_failure_is_propagated_before_any_later_evidence_or_bundle_call ... ok
test t09_complete_original_parent_lineage_is_required ... ok
test t03_arbitrary_logs_and_wrong_envelopes_refuse_all_positive_routes ... ok
test t04_legacy_readers_remain_readable_but_no_legacy_summary_qualifies ... ok
test t13_oras_bad_digest_non_utf8_and_nonzero_status_fail_both_publishers ... ok
test t15_caller_files_beneath_release_output_get_distinct_action_snapshots ... ok
test t17_grouped_flat_streams_and_existing_canonical_outputs_are_identical ... ok
test adversary2_action_rechecks_current_model_and_deployment_before_evidence_upload ... ok
test t15_missing_invalid_and_nonqualifying_action_inputs_stop_before_generic_check ... ok
test t10_negative_inconclusive_and_unknown_reports_stop_all_publication_routes ... ok
test t11_exact_tokens_membership_and_duplicate_keys_reach_the_report_reader ... ok
test t08_scope_origin_and_subset_cannot_satisfy_a_different_expectation ... ok
test t15_snapshot_pins_block_mutation_before_evidence_and_final_bundle_publication ... ok
test t07_original_suite_pairing_and_full_model_identity_refuse_substitution ... ok
test t16_actual_action_build_adopt_platform_chart_and_summary_paths_preserve_limits ... ok

test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s

```

stderr.raw
```text
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.89s
     Running tests/delivery_trust.rs (target/debug/deps/delivery_trust-8268e192ee8c98d2)
```

`final-clippy-refactored` — exact command, direct result, stdout, stderr:

command.txt
```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "clippy", "-p", "ess-cli", "-p", "ess-deployment", "--all-targets", "--locked", "--offline", "--", "-D", "warnings"]
launch-environment-sha256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634
```

exit.txt
```text
exit Some(0)
status exit status: 0
pid 2337746
started_at SystemTime { tv_sec: 1788788999, tv_nsec: 867896958 }
ended_at SystemTime { tv_sec: 1788789000, tv_nsec: 264403457 }
elapsed_seconds 0.396512
```

stdout.raw
```text
```

stderr.raw
```text
    Checking ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 0.35s
```

`final-format` — exact command, direct result, stdout, stderr:

command.txt
```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["cargo", "fmt", "-p", "ess-cli", "-p", "ess-deployment", "--check"]
launch-environment-sha256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634
```

exit.txt
```text
exit Some(0)
status exit status: 0
pid 2347139
started_at SystemTime { tv_sec: 1788789034, tv_nsec: 754730354 }
ended_at SystemTime { tv_sec: 1788789035, tv_nsec: 75947665 }
elapsed_seconds 0.321222
```

stdout.raw
```text
```

stderr.raw
```text
```

`final-fixture-format` — exact command, direct result, stdout, stderr:

command.txt
```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/review-boundaries-12/preparation/toolchain-snapshot/bin/rustfmt", "--edition", "2021", "--check", "crates/edge/ess-cli/tests/support/fake_release_component.rs"]
launch-environment-sha256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634
```

exit.txt
```text
exit Some(0)
status exit status: 0
pid 2384380
started_at SystemTime { tv_sec: 1788789217, tv_nsec: 550844024 }
ended_at SystemTime { tv_sec: 1788789217, tv_nsec: 558878706 }
elapsed_seconds 0.008036
```

stdout.raw
```text
```

stderr.raw
```text
```

4. Judgement findings

No new product defect or judgement finding was established in this second source attack.

5. Attacked boundaries and limits

- B01/B03: coherent consistency and recomputed digests cannot substitute a different model for explicit positive conformance qualification.
- B04: both publishing routes keep the admitted model/deployment and exact payload despite later caller-file replacement; private staging bytes were never attacked.
- B06: actual action rechecks current semantic model and component/runtime context after the generic check and before the first evidence upload.
- B07/T01–T18: all 35 inherited delivery cases and 21 deployment cases still execute, including the previous runtime-unit refusal, required-kind existence, mixed/additional outputs and caller-report snapshot controls.
- Public qualifiers remain limited to supplied report, exact declared selection, modeled deployment compatibility and bundle consistency; these runs do not authenticate producers, assess signatures or establish artifact-execution identity.

6. Retained paths, execution controls and sealing

The assigned external write root is /home/timo/.cache/ess-w16-delivery-adversary-2-tmp. Every retained native descendant and literal symlink target is listed losslessly in outside-paths.tsv; native-seal.tsv supplies the final complete hashes and modes. Original implementation /home/timo/.cache/ess-w16-delivery-tmp, first review /home/timo/.cache/ess-w16-delivery-adversary-1-tmp and correction /home/timo/.cache/ess-w16-delivery-fix1-tmp were read and counted, not written or cleaned by this pass. The fifth counted root is /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract/target. Own scratch and all raw records are under its review-boundaries-16/adversary-pass-2 directory. No /tmp, target override, shared build directory, package installation, network, integration, AEP operation, Git mutation, publication, delegation or cleanup was used.

The exact launch-environment.json has SHA256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634. All product producers use the default unit target, existing private Cargo/Go inputs, frozen Rust, two Rust jobs, one producer at a time, all four wrappers empty, preserved HOME, own private TMP/XDG and a distinct explicit never-used actual-producers/<label> coverage-export leaf. No historical counters or exports were reused. Real Bash and jq run the actual action; external tool effects are finite offline fixture calls, with raw requests/payloads/status retained. The model change is independently compiled before use. No browser or whole integration/package repetition was needed for these test-only additions.

The full fresh root dispatch receipt was read before bootstrap. It verified the complete 378-entry frozen Rust and 50-entry Firefox rosters, modes, regular sizes/hashes, literal links and three real local tools. Every retained producer then repeats the complete check before and after execution, stores exact fourteen-file source copies and producer.rs, and retains direct PID/start/end/exit and raw streams before postprocessing. The pre-bootstrap tool manifests are pinned in tool-manifests.json: Rust 387c58849c86996b2aa20d63de19d6e4d925c91552682ed5eb49a1b0a1ce731d and Firefox eaac01c1bafaf70d1392d46914f0fd69ff247f7bd9ad0895334add758543bd9f. Producer, sealer and report helpers are Rust source under assigned scratch; bootstrap-command.txt preserves the exact bootstrap launch recipe and its coverage override.

Complete direct producer lifecycle receipts (all `/proc/<pid>` absent at report assembly):

| Producer | PID | Direct exit | Start (Unix sec/ns) | End (Unix sec/ns) |
|---|---:|---|---|---|
| compile-native-seal | 2391029 | Some(0) | SystemTime { tv_sec: 1788789258, tv_nsec: 592387756 } | SystemTime { tv_sec: 1788789258, tv_nsec: 715183444 } |
| compile-report-helper | 2397705 | Some(0) | SystemTime { tv_sec: 1788789298, tv_nsec: 143601367 } | SystemTime { tv_sec: 1788789298, tv_nsec: 281861693 } |
| final-clippy | 2306356 | Some(101) | SystemTime { tv_sec: 1788788708, tv_nsec: 107867091 } | SystemTime { tv_sec: 1788788708, tv_nsec: 446972753 } |
| final-clippy-refactored | 2337746 | Some(0) | SystemTime { tv_sec: 1788788999, tv_nsec: 867896958 } | SystemTime { tv_sec: 1788789000, tv_nsec: 264403457 } |
| final-delivery | 2300364 | Some(0) | SystemTime { tv_sec: 1788788662, tv_nsec: 215607788 } | SystemTime { tv_sec: 1788788664, tv_nsec: 100592839 } |
| final-delivery-refactored | 2329538 | Some(0) | SystemTime { tv_sec: 1788788949, tv_nsec: 65319249 } | SystemTime { tv_sec: 1788788951, tv_nsec: 73713901 } |
| final-deployment | 2304483 | Some(0) | SystemTime { tv_sec: 1788788692, tv_nsec: 563562383 } | SystemTime { tv_sec: 1788788692, tv_nsec: 756500016 } |
| final-fixture-format | 2384380 | Some(0) | SystemTime { tv_sec: 1788789217, tv_nsec: 550844024 } | SystemTime { tv_sec: 1788789217, tv_nsec: 558878706 } |
| final-format | 2347139 | Some(0) | SystemTime { tv_sec: 1788789034, tv_nsec: 754730354 } | SystemTime { tv_sec: 1788789035, tv_nsec: 75947665 } |
| focused-action-recheck | 2267327 | Some(101) | SystemTime { tv_sec: 1788788356, tv_nsec: 363082012 } | SystemTime { tv_sec: 1788788356, tv_nsec: 738716103 } |
| focused-action-recheck-corrected | 2281801 | Some(0) | SystemTime { tv_sec: 1788788522, tv_nsec: 49184935 } | SystemTime { tv_sec: 1788788523, tv_nsec: 355191667 } |
| focused-coherent-bundle | 2263221 | Some(0) | SystemTime { tv_sec: 1788788319, tv_nsec: 706555164 } | SystemTime { tv_sec: 1788788321, tv_nsec: 148868446 } |
| focused-owned-context | 2285893 | Some(0) | SystemTime { tv_sec: 1788788559, tv_nsec: 404106804 } | SystemTime { tv_sec: 1788788559, tv_nsec: 701793679 } |
| format-own-tests | 2297557 | Some(0) | SystemTime { tv_sec: 1788788637, tv_nsec: 135266058 } | SystemTime { tv_sec: 1788788637, tv_nsec: 160383768 } |
| format-own-tests-clippy | 2323243 | Some(0) | SystemTime { tv_sec: 1788788910, tv_nsec: 62024011 } | SystemTime { tv_sec: 1788788910, tv_nsec: 86367319 } |
| native-seal-final | 2416385 | Some(0) | SystemTime { tv_sec: 1788789357, tv_nsec: 962346002 } | SystemTime { tv_sec: 1788789409, tv_nsec: 24288177 } |
| prepare-handoff | 2410835 | Some(0) | SystemTime { tv_sec: 1788789327, tv_nsec: 775389897 } | SystemTime { tv_sec: 1788789327, tv_nsec: 907332800 } |

The 34 producer endpoint observations measured maximum combined allocated bytes 11083665408 and minimum free bytes 25497640960. Limits were 12,884,901,888 allocated and 8,589,934,592 free. These are endpoint observations, not continuous monitoring. Dispatch and bootstrap resource receipts are separately retained; no resource refusal occurred.

Bootstrap direct receipts, with root's complete fresh pre-bootstrap check copied into bootstrap/:

bootstrap/started.txt
```text
pid 2260954
started_at 2026-09-07T13:38:22.361310734Z
```

bootstrap/exit.txt
```text
exit 0
pid 2260954
started_at 2026-09-07T13:38:22.361310734Z
ended_at 2026-09-07T13:38:22.512050443Z
```

bootstrap/stdout.raw
```text
```

bootstrap/stderr.raw
```text
```

Final native sealer direct command and result:

`native-seal-final` — exact command, direct result, stdout, stderr:

command.txt
```text
cwd /home/timo/.local/state/worktree/trees/b10x/ess/ess-delivery-trust-contract
argv ["target/review-boundaries-16/adversary-pass-2/seal"]
launch-environment-sha256 a5d888b8b5a3966bc00a9569da15f80921fb9fec5b7b8438629b03303bd09634
```

exit.txt
```text
exit Some(0)
status exit status: 0
pid 2416385
started_at SystemTime { tv_sec: 1788789357, tv_nsec: 962346002 }
ended_at SystemTime { tv_sec: 1788789409, tv_nsec: 24288177 }
elapsed_seconds 51.061950
```

stdout.raw
```text
Sealed 79043 regular files (11210035796 logical bytes), 109515 native entries. Paths and literal symlink targets are encoded as native-byte hexadecimal without following links. Excludes this manifest, its owning producer record, report.md, quiescence.json and later handoff.sha256; these are separately bound by handoff.sha256.
```

stderr.raw
```text
```

Final quiescence observer, run from outside the owned roots after all producers returned:
```json
{"observer_pid":2430158,"observed_at_unix_ns":1788789421879673499,"owned_processes":[]}
```

Exact source pins:
```json
{
  ".github/actions/release-component/action.yml": "41dd4fdcf5553a3bdfb078bb302640d845f1e9721946082143b855bd9e46e457",
  ".github/actions/release-component/release.sh": "f60e1784f5dd4876cbe764e0328b259615fa2cf7bc9ac454b918a54288a5a181",
  "crates/edge/ess-cli/src/main.rs": "fa6a3ed8b85d960af2afcf2f6d4401f1f5de75f0fe0dd6bdffe1d498c1f9d5c2",
  "crates/edge/ess-cli/src/release_evidence.rs": "15093a0690b7d0e182711057fe3286833da4bebccb84288eab7153a03ce37fea",
  "crates/edge/ess-cli/tests/command_surface.rs": "7a8769fbdd16f71a11956d44698adc759576d946a62ccde35b388672b52d7939",
  "crates/edge/ess-cli/tests/delivery_trust.rs": "ebb513a227f6f12b293469c43fafe056449a7d66bc6d46c73971509a3dbaea84",
  "crates/edge/ess-cli/tests/support/fake_release_component.rs": "761eed83aa6c31f9364240380fc6a838bc30a5976121b7965e787319f7eb234c",
  "crates/generate/ess-deployment/src/component.rs": "bcfbd5b0980f5829542ae6e589239a88a6f811c2252a2cc78176cc46283de48d",
  "crates/generate/ess-deployment/src/release.rs": "df904ff9dab1a76b312946764adac84f3ebd722958d7780c6800c42f33334fc6",
  "crates/generate/ess-deployment/tests/deployment.rs": "189c6584925154335c7c2c7b1777f79222c5e2391202ab115ec2d8ef8f4cbbc3",
  "docs/design/review-delivery-trust.md": "3baac1dcded6440f0c3a75bd3e1ff4ff27a30c319006859179e19c6b2a39dbe8",
  "website/docs/concepts/component-delivery.md": "4507e3811e5f4769068b53859c20c027b495ed52351b703b69ea47c580ccf247",
  "website/docs/reference/cli.md": "e7d2f839333969574dd248f49c9d60f231df7e5eff13eb36024b0a2361d7f54a",
  "website/docs/reference/formats.md": "cecacef24aeaae88f22b7d3ccccc0d49af8e10cc48b6cd1b7f79ba31649ae622"
}
```

The native manifest covers all five counted roots without following symlinks and retains special-node metadata without opening sockets. Native path and literal link bytes are hexadecimal. Its explicit self-reference exclusions are native-seal.tsv, report.md, quiescence.json, handoff.sha256 and runs/native-seal-final/. The handoff manifest separately hashes every regular file in this scratch tree except itself, including all exclusions, all final-sealer source/tool/resource copies and raw outputs, exact bootstrap and dispatch inputs, report helper source/binary, per-producer receipts/seals, source snapshots, final pins and own test delta. `handoff.sha256` is a byte-binding inventory, not an attestation of producer authenticity. All retained original failures remain bound.

```findings
[]
```
