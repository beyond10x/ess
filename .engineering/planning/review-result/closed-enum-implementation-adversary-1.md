---
format: aep.planning-md/1
id: review-result:closed-enum-implementation-adversary-1
kind: review-result
status: active
title: 'Closed enum implementation adversary: invariant witness finding'
relations:
- reviews: story:closed-enum-outcome-coverage
revision: 1
---
unit: story:closed-enum-outcome-coverage at a66279d50e21847f33b01c644a22c3079aba5d1b
verdict: CONFIRMED
cases: executed 70→76, red 2
origin: introduced 0 / pre-existing 0 / undecided 1
wrote-outside-worktree: 5 paths (four report/log files and one compiler target directory)
needs-coordinator: record and route invariant witness finding; commit test-only additions; decide scope of correction before integrating
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 a7af796ae77c73ee4366f4c037909f96a211672380ba27d8b2199e6349774215, retained as local-evidence:runtime-gaps/publication-replay/snapshots/a7af796ae77c73ee4366f4c037909f96a211672380ba27d8b2199e6349774215.md. Source creation recorded at 2026-09-11T03:34:03Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 c8842c2a80eb14bbb602367316424fcbf3d44312fc5467fb7e5f98839949a388, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/c8842c2a80eb14bbb602367316424fcbf3d44312fc5467fb7e5f98839949a388-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->
 crates/verify/ess-conformance/tests/finite_enum.rs | 78 ++++++++++++++++++++++
 1 file changed, 78 insertions(+)

The before count is inherited from the implementor's report: finite_enum 16 plus synthesis 54. The after count is the scoped suite actually executed: finite_enum 22 plus synthesis 54, 74 passing and 2 failing. The implementor's other 407 cases were not rerun and are not counted as newly executed.

Added cases, all in crates/verify/ess-conformance/tests/finite_enum.rs:

- :261 adversary_finite_wrapper_witness_respects_declared_invariant — RED. A declared newtype invariant excludes Offline, so an executable branch witness must not supply Offline.
- :266 adversary_legacy_default_wrapper_witness_respects_declared_invariant — RED. The same check with a genuine default controls whether the failure is isolated to the new no-default path.
- :271 adversary_optional_container_cannot_prove_required_leaf_coverage — GREEN. Optional at an intermediate struct container must preserve the missing-default refusal.
- :281 adversary_raw_named_scalar_deferral_does_not_escape_full_validation — GREEN. Replacing the enum with a named String newtype must not turn raw shape deferral into validated coverage.
- :291 adversary_repeated_same_fact_is_one_correlated_domain — GREEN. Equality and singleton membership on the same fact share the actual domain and retain all twelve unique witnesses.
- :300 adversary_undeclared_variant_inside_membership_is_not_coverage — GREEN. A fictional enum membership literal is rejected through full source assembly.

Both red cases were authored before the first test run. Each ran alone before the suite. Original red output refers to assertion line 243; final rustfmt moved the same assertion to line 249. No test was weakened, skipped or deleted; no implementation file was edited.

First focused command (exit 101):
```sh
env -u CARGO_ENCODED_RUSTFLAGS RUSTUP_TOOLCHAIN=1.98.1 CARGO_TARGET_DIR=local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/enum CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_INCREMENTAL=false CARGO_PROFILE_TEST_INCREMENTAL=false RUSTFLAGS='-C link-arg=-fuse-ld=lld' RUSTC_WRAPPER=sccache cargo test -p ess-conformance --test finite_enum adversary_finite_wrapper_witness_respects_declared_invariant --offline --locked -- --exact
```
```text
   Compiling ess-conformance v0.22.2 (worktree-state:/trees/b10x/ess/wt-2420def710d2/crates/verify/ess-conformance)
    Finished `test` profile [unoptimized] target(s) in 0.36s
     Running tests/finite_enum.rs (local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/enum/debug/deps/finite_enum-296cbe8b2de94d13)

running 1 test
test adversary_finite_wrapper_witness_respects_declared_invariant ... FAILED

failures:

---- adversary_finite_wrapper_witness_respects_declared_invariant stdout ----

thread 'adversary_finite_wrapper_witness_respects_declared_invariant' (2514774) panicked at crates/verify/ess-conformance/tests/finite_enum.rs:243:17:
assertion `left != right` failed: reporting.core.Refresh/outcome/state-0 claims a reachable branch using a value excluded by Wrapped's invariant; refusals: [Refusal { subject: Type { name: DeclaredTypeRef(QualifiedName(reporting.core.Wrapped)) }, scenario: None, cause: ValueInvariantUnwitnessed { value: DeclaredTypeRef(QualifiedName(reporting.core.Wrapped)), invariants: ["value != Offline"], at: None } }]
  left: Some(Literal { value: Text("Offline") })
 right: Some(Literal { value: Text("Offline") })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    adversary_finite_wrapper_witness_respects_declared_invariant

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-conformance --test finite_enum`
```

First focused command (exit 101):
```sh
env -u CARGO_ENCODED_RUSTFLAGS RUSTUP_TOOLCHAIN=1.98.1 CARGO_TARGET_DIR=local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/enum CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_INCREMENTAL=false CARGO_PROFILE_TEST_INCREMENTAL=false RUSTFLAGS='-C link-arg=-fuse-ld=lld' RUSTC_WRAPPER=sccache cargo test -p ess-conformance --test finite_enum adversary_legacy_default_wrapper_witness_respects_declared_invariant --offline --locked -- --exact
```
```text
    Finished `test` profile [unoptimized] target(s) in 0.04s
     Running tests/finite_enum.rs (local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/enum/debug/deps/finite_enum-296cbe8b2de94d13)

running 1 test
test adversary_legacy_default_wrapper_witness_respects_declared_invariant ... FAILED

failures:

---- adversary_legacy_default_wrapper_witness_respects_declared_invariant stdout ----

thread 'adversary_legacy_default_wrapper_witness_respects_declared_invariant' (2517955) panicked at crates/verify/ess-conformance/tests/finite_enum.rs:243:17:
assertion `left != right` failed: reporting.core.Refresh/outcome/state-0 claims a reachable branch using a value excluded by Wrapped's invariant; refusals: [Refusal { subject: Type { name: DeclaredTypeRef(QualifiedName(reporting.core.Wrapped)) }, scenario: None, cause: ValueInvariantUnwitnessed { value: DeclaredTypeRef(QualifiedName(reporting.core.Wrapped)), invariants: ["value != Offline"], at: None } }]
  left: Some(Literal { value: Text("Offline") })
 right: Some(Literal { value: Text("Offline") })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    adversary_legacy_default_wrapper_witness_respects_declared_invariant

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 21 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-conformance --test finite_enum`
```

Subsequent scoped suite command (exit 101):
```sh
env -u CARGO_ENCODED_RUSTFLAGS RUSTUP_TOOLCHAIN=1.98.1 CARGO_TARGET_DIR=local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/enum CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_INCREMENTAL=false CARGO_PROFILE_TEST_INCREMENTAL=false RUSTFLAGS='-C link-arg=-fuse-ld=lld' RUSTC_WRAPPER=sccache cargo test -p ess-conformance --test finite_enum --test synthesis --offline --locked --no-fail-fast
```
```text
    Finished `test` profile [unoptimized] target(s) in 0.04s
     Running tests/finite_enum.rs (local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/enum/debug/deps/finite_enum-296cbe8b2de94d13)

running 22 tests
test predicate_node_budget_is_checked_independently_of_domain_size ... ok
test adversary_optional_container_cannot_prove_required_leaf_coverage ... ok
test adversary_raw_named_scalar_deferral_does_not_escape_full_validation ... ok
test an_extra_open_input_conjunct_does_not_prove_coverage ... ok
test optional_domain_does_not_hide_absence ... ok
test missing_variant_reports_a_real_uncovered_value ... ok
test adversary_undeclared_variant_inside_membership_is_not_coverage ... ok
test absence_and_null_stay_unknown_in_the_existing_evaluator ... ok
test duplicate_guards_report_both_overlap_and_the_omitted_value ... ok
test an_extra_enum_conjunct_has_a_concrete_uncovered_assignment ... ok
test overlap_cannot_prove_unique_coverage ... ok
test a_real_default_retains_its_reachable_witness ... ok
test oversized_joint_domain_is_not_mistaken_for_complete_search ... ok
test adversary_legacy_default_wrapper_witness_respects_declared_invariant ... FAILED
test adversary_finite_wrapper_witness_respects_declared_invariant ... FAILED
test external_branch_does_not_replace_or_interfere_with_enum_coverage ... ok
test struct_paths_are_populated_by_the_same_proof_assignments ... ok
test retained_unreachable_branch_is_still_refused ... ok
test transparent_wrappers_preserve_the_real_domain ... ok
test two_six_value_shapes_have_unique_executable_branch_witnesses ... ok
test adversary_repeated_same_fact_is_one_correlated_domain ... ok
test complete_joint_enum_domain_yields_real_witnesses ... ok

failures:

---- adversary_legacy_default_wrapper_witness_respects_declared_invariant stdout ----

thread 'adversary_legacy_default_wrapper_witness_respects_declared_invariant' (2518048) panicked at crates/verify/ess-conformance/tests/finite_enum.rs:243:17:
assertion `left != right` failed: reporting.core.Refresh/outcome/state-0 claims a reachable branch using a value excluded by Wrapped's invariant; refusals: [Refusal { subject: Type { name: DeclaredTypeRef(QualifiedName(reporting.core.Wrapped)) }, scenario: None, cause: ValueInvariantUnwitnessed { value: DeclaredTypeRef(QualifiedName(reporting.core.Wrapped)), invariants: ["value != Offline"], at: None } }]
  left: Some(Literal { value: Text("Offline") })
 right: Some(Literal { value: Text("Offline") })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- adversary_finite_wrapper_witness_respects_declared_invariant stdout ----

thread 'adversary_finite_wrapper_witness_respects_declared_invariant' (2518047) panicked at crates/verify/ess-conformance/tests/finite_enum.rs:243:17:
assertion `left != right` failed: reporting.core.Refresh/outcome/state-0 claims a reachable branch using a value excluded by Wrapped's invariant; refusals: [Refusal { subject: Type { name: DeclaredTypeRef(QualifiedName(reporting.core.Wrapped)) }, scenario: None, cause: ValueInvariantUnwitnessed { value: DeclaredTypeRef(QualifiedName(reporting.core.Wrapped)), invariants: ["value != Offline"], at: None } }]
  left: Some(Literal { value: Text("Offline") })
 right: Some(Literal { value: Text("Offline") })


failures:
    adversary_finite_wrapper_witness_respects_declared_invariant
    adversary_legacy_default_wrapper_witness_respects_declared_invariant

test result: FAILED. 20 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p ess-conformance --test finite_enum`
     Running tests/synthesis.rs (local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/enum/debug/deps/synthesis-16858c3fdad8fcfc)

running 54 tests
test a_guard_no_candidate_can_satisfy_is_refused_with_the_number_tried ... ok
test a_binding_whose_branch_the_event_decides_refuses_the_flow_and_still_checks_the_mapping ... ok
test a_command_that_declares_no_wrong_state_answer_is_refused_by_name_beside_its_scenario ... ok
test a_command_that_accepts_a_wrong_state_is_asserted_as_accepting_rather_than_refusing ... ok
test a_filter_reading_something_no_scenario_knows_refuses_rather_than_guessing ... ok
test a_parameterised_view_is_queried_with_the_value_the_scenario_put_in_the_row ... ok
test a_component_nothing_declares_is_refused_by_name ... ok
test a_state_reached_only_through_a_branch_no_input_reaches_is_refused_rather_than_arranged ... ok
test a_binding_mapping_names_the_source_the_document_wrote_and_not_its_same_typed_sibling ... ok
test a_read_your_writes_view_filled_by_the_command_that_ran_is_asserted_to_hold_a_row ... ok
test a_declared_order_is_asserted_against_two_rows_the_scenario_arranged_itself ... ok
test a_declared_error_is_asserted_by_name_and_never_by_an_invented_payload ... ok
test a_move_is_observed_through_the_view_the_state_it_left_is_filtered_on ... ok
test a_value_object_nothing_observable_holds_keeps_a_refusal_naming_what_would_close_it ... ok
test a_binding_flow_is_proved_through_the_event_the_invoked_command_publishes ... ok
test a_binding_that_retries_forces_one_failure_and_still_requires_the_consequence ... ok
test a_scenario_that_moves_an_instance_names_the_one_an_earlier_step_created ... ok
test an_entity_nothing_creates_cannot_be_acted_on_and_says_so ... ok
test a_synthesised_count_is_a_floor_the_scenario_arranged_and_never_a_ceiling ... ok
test an_order_the_specification_cannot_put_two_rows_under_is_refused_and_not_asserted ... ok
test a_view_is_asserted_in_the_block_its_own_consistency_decides ... ok
test a_move_that_is_illegal_in_a_state_is_attempted_with_the_input_that_would_have_worked ... ok
test a_view_the_entity_has_not_reached_yet_is_asserted_to_exclude_the_instance_by_name ... ok
test an_undecidable_guard_refuses_and_does_not_spend_the_candidate_budget ... ok
test a_value_objects_own_invariants_are_read_at_every_field_position_a_view_holds_one ... ok
test a_binding_that_drops_its_failures_refuses_that_check_and_names_the_reason ... ok
test a_view_that_does_not_hold_the_instance_yet_is_not_asked_about_its_invariants ... ok
test an_actor_is_named_only_where_the_specification_grants_the_command ... ok
test an_at_least_once_binding_delivers_the_event_twice_and_requires_no_count ... ok
test an_invariant_is_asserted_against_every_view_that_publishes_what_it_reads ... ok
test a_synthesised_suite_survives_being_written_and_read_back ... ok
test an_outcome_that_updates_an_instance_acts_on_one_the_scenario_created ... ok
test an_event_assertion_carries_the_declared_shape_and_exactly_the_values_the_payload_determines ... ok
test an_illegal_move_requires_the_branch_and_the_declared_error_rather_than_merely_failing ... ok
test a_binding_that_escalates_requires_the_event_the_escalation_declares ... ok
test a_suite_for_one_component_holds_only_what_that_component_realises ... ok
test a_whole_system_suite_does_not_mention_a_component ... ok
test an_invariant_over_a_field_no_view_publishes_refuses_rather_than_being_dropped ... ok
test every_command_names_an_instance_an_earlier_step_of_the_same_scenario_bound ... ok
test the_failure_control_is_armed_after_the_arrangement_and_before_the_command_that_triggers_it ... ok
test every_declared_outcome_is_either_a_scenario_or_a_named_refusal_or_asserted_by_the_state_family ... ok
test every_declared_transition_has_a_scenario_that_proves_it_can_occur ... ok
test the_input_a_scenario_sends_is_re_decided_against_the_guard_it_claims_to_reach ... ok
test the_refusal_branch_asserts_that_no_event_the_specification_declares_occurred ... ok
test the_dependency_set_names_the_types_the_scenario_is_made_of ... ok
test an_at_most_once_binding_synthesises_no_redelivery_anywhere_in_its_suite ... ok
test each_example_synthesises_the_families_its_specification_declares ... ok
test an_outcome_no_input_decides_is_reached_by_injection_and_by_nothing_else ... ok
test every_clause_of_every_binding_is_either_a_scenario_or_a_named_refusal ... ok
test synthesising_the_same_specification_twice_produces_byte_identical_output ... ok
test coverage_builder_records_the_complete_generated_inventory_and_component_omissions ... ok
test coverage_all_missing_invariants_keep_null_survivors_and_component_proofs_stay_conservative ... ok
test canonical_expression_compatibility_fixtures ... ok
test coverage_missing_lifecycle_and_view_checks_remain_beside_actual_passing_results ... ok

test result: ok. 54 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s

error: 1 target failed:
    `-p ess-conformance --test finite_enum`
```

Findings cover the exact candidate above plus these test-only additions.

| Location | Verdict | Origin | Severity | Finding | What reaches it |
|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/witness.rs:471 | CONFIRMED | undecided | warning | Synthesis emits an Offline branch witness for Wrapped(Status) despite its declared value != Offline invariant; the same candidate failure occurs with a real default. | Ordinary ESS/1 source declares Wrapped as a newtype of Status with invariants: [value != Offline], uses Wrapped as the command input, and enumerates the six equality branches. RawSpecFile::parse → Specification::assemble → compile → synthesize admits the source and emits reporting.core.Refresh/outcome/state-0 with literal Offline. Both focused tests fail at the exact witness assertion (:249 after formatting). |

The underlying newtype builder at :471 transparently recurses without using the invariants. The new finite assignment path at :209 invokes that builder. The result also contains a global ValueInvariantUnwitnessed refusal for Wrapped; this review does not claim a refusal-free suite or a falsely successful complete conformance report. It identifies the narrower branch-witness limitation: an invariant-excluded input is still used in a generated outcome scenario. The finite truth table remains conservative over the full declared enum; this is not evidence of an uncovered assignment being silently admitted.

Origin remains undecided. The default control reproduces on the candidate, and read-only base diff confirms that the builder and default candidate path were unchanged, but no base worktree was assigned and no test ran against the exact base 4c03e4fc89153323d79a7639a08679b15c246d28. I did not switch, stash, create another tree or claim an exact-base reproduction. Coordinator can route the existing witness limitation separately or require explicit conservative refusal for invariant-constrained wrappers in this new fragment. Do not infer a need for a general invariant solver from this report.

What I attacked and could not break:

- Optional traversal retains absence conservatism even when the terminal enum itself is required.
- Named open scalar roots cannot escape full typed validation through raw shape deferral.
- Repeated references to one fact remain correlated and produce uniquely selected actual witnesses.
- An invalid enum literal inside membership is rejected.
- The existing six-value, joint-domain, resource-bound, external, default, unreachable and synthesis regression cases selected in this scoped run all remained green.

Formatting the test file with rustfmt and git diff --check succeeded. Formatting happened after the recorded runs; it changed only layout and did not justify rerunning passing cases. No full gate, ownership run, generated-file action, planning action, external integration, commit or push was performed.

Outside-worktree output paths (full):

- local-evidence:/cache/ess-evolution-20260910/priority-wave/enum/adversary/first-finite-invariant.log
- local-evidence:/cache/ess-evolution-20260910/priority-wave/enum/adversary/first-legacy-invariant.log
- local-evidence:/cache/ess-evolution-20260910/priority-wave/enum/adversary/focused-suite.log
- local-evidence:/cache/ess-evolution-20260910/priority-wave/enum/adversary/report.md
- local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/enum (compiler-managed artifacts; existing assigned cache reused)

The worktree CLI additionally maintains its own registry lease metadata; reviewer lease ess-gap-enum-adversary-01a089ee is released on handback. No source or scratch was written in another checkout or /tmp. Coordinator owns commit and final worktree cleanup.

**Publication path-redaction notice.** This public copy replaces only exact local home-path prefixes: the managed-worktree state root is `worktree-state:/`, and the local cache root is `local-evidence:/cache/`. Remaining path suffixes are preserved. These aliases are evidence labels, not executable filesystem paths. Original logs and the original report remain unmodified; quoted output here differs only in those path prefixes. Review findings, verdicts, test counts, and test outcomes are unchanged. This copy records no additional review or execution.

Original report SHA256: `8cf1dbc874e5823a4871dda5b5f14e3e18a55c566a4f001febce8fb3e7c40e2d`.

```findings
- file: crates/verify/ess-conformance/src/witness.rs
  line: 471
  category: property
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: Synthesis emits an Offline branch witness for Wrapped(Status) despite its declared value != Offline invariant; the same candidate failure occurs with a real default.
```