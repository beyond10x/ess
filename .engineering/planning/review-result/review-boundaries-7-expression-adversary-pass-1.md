---
format: aep.planning-md/1
id: review-result:review-boundaries-7-expression-adversary-pass-1
kind: review-result
status: active
title: Expression typechecking independent review
relations:
- reviews: story:review-expression-typechecking
revision: 1
---
unit: review-expression-typechecking adversary pass 1, frozen f03ecdafda8059562d781cd4c842a0fd936bc8c2 plus the new tests listed below
verdict: nothing found
cases: executed 683→697, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 shared cache paths; pre-existing sccache socket use also disclosed
needs-coordinator: none

git --no-pager diff --stat
```console
```

The command has empty output: no tracked file changed. The only additions are untracked tests and their test fixture, shown completely below. Git index/ref mutation is outside this assignment, so the tests were not staged merely to make ordinary diff --stat include them.

```console
git ls-files --others --exclude-standard
crates/specify/ess-compiler/tests/adversary_expression_pass1.rs
crates/specify/ess-compiler/tests/fixtures/adversary_expression.yaml
crates/verify/ess-conformance/tests/adversary_expression_pass1.rs
```

Supplemental no-index stats for those exact new paths (exit 1 means a new file differs from /dev/null):

```console
 .../tests/adversary_expression_pass1.rs            | 236 +++++++++++++++++++++
 1 file changed, 236 insertions(+)
 .../tests/fixtures/adversary_expression.yaml       | 74 ++++++++++++++++++++++
 1 file changed, 74 insertions(+)
 .../tests/adversary_expression_pass1.rs            | 108 +++++++++++++++++++++
 1 file changed, 108 insertions(+)
```

No production, inherited test, manifest, lockfile, documentation, planning, Git index/ref, or lifecycle write occurred. The full 17-file opening-base 21eac63d347d5d1328712cd59dd9ae5edf41aace..f03ecdafda8059562d781cd4c842a0fd936bc8c2 diff, new tests, shared-checker callers, unit acceptance, full installed adversary 0.8.0 charter, repository instructions, corrected binding, and coordinator-supplied Atlas authority were read. Managed registration was confirmed using worktree repo list; the coordinator retains lease and lifecycle ownership.

Case inventory (all green now; the full exact focused executions follow):

| File | Line | Assertion |
| --- | ---: | --- |
| crates/specify/ess-compiler/tests/adversary_expression_pass1.rs | 45 | Map values bind nested Lists; nearest repeated binder restores the outer List after its body. |
| crates/specify/ess-compiler/tests/adversary_expression_pass1.rs | 61 | Canonical and enormous List ordinals validate; leading-zero, Map ordinal/key, count continuation and wire-alias selectors refuse. |
| crates/specify/ess-compiler/tests/adversary_expression_pass1.rs | 89 | Parameter collection target resolves before pushing a binder named param; nested shadowing and later free uses retain their distinct meanings. |
| crates/specify/ess-compiler/tests/adversary_expression_pass1.rs | 108 | A binder named param does not count as using a declared filter parameter. |
| crates/specify/ess-compiler/tests/adversary_expression_pass1.rs | 119 | Guard, entity, struct and filter enum checks see through optional newtypes and reject non-Text masquerading literals. |
| crates/specify/ess-compiler/tests/adversary_expression_pass1.rs | 147 | Literal-on-left enum checks and same-representation fact comparisons behave consistently through the public IR adapter. |
| crates/specify/ess-compiler/tests/adversary_expression_pass1.rs | 179 | An invalid quantifier target suppresses only its unresolved binder cascade while independent sibling errors keep source order and command ownership. |
| crates/specify/ess-compiler/tests/adversary_expression_pass1.rs | 195 | Ninety consuming recursive field steps validate; missing terminal fields and non-progress transparent cycles refuse. |
| crates/specify/ess-compiler/tests/adversary_expression_pass1.rs | 212 | Empty scalar memberships remain legal; empty aggregate memberships and aggregate presence remain ill-typed. |
| crates/specify/ess-compiler/tests/adversary_expression_pass1.rs | 225 | Every invalid membership element produces its own stable enum/kind diagnostic after a valid element. |
| crates/verify/ess-conformance/tests/adversary_expression_pass1.rs | 42 | Bound operand and scalar quantifier errors are 035; legal collection reads retain projection refusal 026. |
| crates/verify/ess-conformance/tests/adversary_expression_pass1.rs | 54 | Authored scalar controls compile, while aggregate, unprojected, parameter, collection and wire-alias reads refuse as 026. |
| crates/verify/ess-conformance/tests/adversary_expression_pass1.rs | 82 | Optional enum and later membership type errors remain 035, including a short-circuitable child. |
| crates/verify/ess-conformance/tests/adversary_expression_pass1.rs | 95 | A legal deep optional recursive path resolves semantically and separately fails the producer depth capability as 026. |

2. Focused cases, written and individually selected before the complete package suite.

Ten compiler cases and four authored conformance cases were added. Their assertions exercise public RawSpecFile::parse -> Specification::assemble -> compile/compile_locating, then the public resolved checker and authored compiler. No sealed state was manufactured. All fourteen cases are green after correcting the authored document setup. No semantic counterexample was found.

The original authored documents omitted a command timeline. Their four initial focused executions failed due to the existing NothingHappens scenario admission rule; three also returned the intended expression refusal. This is a fixture setup failure, not a typechecking finding. The correction adds a no-input ZObserve command and a single valid timeline step. It changes only this pass's newly added test fixture/helper and no assertion. The original focused output is retained below and in focused-11.log through focused-14.log. Corrected focused executions follow those original outputs.

All commands use the assigned tree's own target with CARGO_TARGET_DIR unset; TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-expression-typechecking/target/review-boundaries-7/adversary-pass-1; RUSTC_WRAPPER=/usr/bin/sccache; SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock; CARGO_INCREMENTAL=0; CARGO_PROFILE_DEV_DEBUG=0; CARGO_PROFILE_TEST_DEBUG=0; CARGO_CACHE_RUSTC_INFO=0; CARGO_BUILD_JOBS=4; CARGO_NET_OFFLINE=true. The socket was checked as listening before use. No new daemon or shared Cargo build directory was created.

```console
cargo test --locked --offline -p ess-compiler --test adversary_expression_pass1 map_values_bind_nested_lists_and_restore_the_outer_binder -- --exact --nocapture
   Compiling ess-domain v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-expression-typechecking/crates/specify/ess-domain)
   Compiling ess-compiler v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-expression-typechecking/crates/specify/ess-compiler)
    Finished `test` profile [unoptimized] target(s) in 4.12s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-fdfc9b2597c5bae3)

running 1 test
test map_values_bind_nested_lists_and_restore_the_outer_binder ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.01s

exit: 0
```

```console
cargo test --locked --offline -p ess-compiler --test adversary_expression_pass1 canonical_list_ordinals_are_legal_but_map_ordinals_and_wire_names_are_not -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-fdfc9b2597c5bae3)

running 1 test
test canonical_list_ordinals_are_legal_but_map_ordinals_and_wire_names_are_not ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.01s

exit: 0
```

```console
cargo test --locked --offline -p ess-compiler --test adversary_expression_pass1 parameter_targets_are_resolved_before_shadowing_and_uses_resume_afterward -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-fdfc9b2597c5bae3)

running 1 test
test parameter_targets_are_resolved_before_shadowing_and_uses_resume_afterward ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.01s

exit: 0
```

```console
cargo test --locked --offline -p ess-compiler --test adversary_expression_pass1 a_parameter_shadowed_everywhere_is_still_unused -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.08s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-fdfc9b2597c5bae3)

running 1 test
test a_parameter_shadowed_everywhere_is_still_unused ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

exit: 0
```

```console
cargo test --locked --offline -p ess-compiler --test adversary_expression_pass1 all_owners_reject_wrong_enum_literals_through_optional_newtypes -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-fdfc9b2597c5bae3)

running 1 test
test all_owners_reject_wrong_enum_literals_through_optional_newtypes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.01s

exit: 0
```

```console
cargo test --locked --offline -p ess-compiler --test adversary_expression_pass1 reverse_enum_literals_and_fact_operands_follow_the_same_representation_rules -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-fdfc9b2597c5bae3)

running 1 test
test reverse_enum_literals_and_fact_operands_follow_the_same_representation_rules ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

exit: 0
```

```console
cargo test --locked --offline -p ess-compiler --test adversary_expression_pass1 unrelated_failures_survive_invalid_quantifier_targets_in_source_order -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.09s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-fdfc9b2597c5bae3)

running 1 test
test unrelated_failures_survive_invalid_quantifier_targets_in_source_order ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

exit: 0
```

```console
cargo test --locked --offline -p ess-compiler --test adversary_expression_pass1 finite_recursive_selectors_consume_segments_but_transparent_cycles_do_not -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-fdfc9b2597c5bae3)

running 1 test
test finite_recursive_selectors_consume_segments_but_transparent_cycles_do_not ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.01s

exit: 0
```

```console
cargo test --locked --offline -p ess-compiler --test adversary_expression_pass1 empty_membership_remains_typed_and_does_not_hide_aggregate_reads -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.08s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-fdfc9b2597c5bae3)

running 1 test
test empty_membership_remains_typed_and_does_not_hide_aggregate_reads ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.01s

exit: 0
```

```console
cargo test --locked --offline -p ess-compiler --test adversary_expression_pass1 membership_reports_every_invalid_member_even_after_a_valid_enum_literal -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.08s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-fdfc9b2597c5bae3)

running 1 test
test membership_reports_every_invalid_member_even_after_a_valid_enum_literal ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

exit: 0
```

```console
cargo test --locked --offline -p ess-conformance --test adversary_expression_pass1 malformed_bound_operands_refuse_before_the_collection_projection_gap -- --exact --nocapture
   Compiling ess-conformance v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-expression-typechecking/crates/verify/ess-conformance)
    Finished `test` profile [unoptimized] target(s) in 0.41s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-692cdd3d83e19b32)

running 1 test

thread 'malformed_bound_operands_refuse_before_the_collection_projection_gap' (3371277) panicked at crates/verify/ess-conformance/tests/adversary_expression_pass1.rs:27:5:
assertion `left == right` failed: {forall: {in: groups, as: group, that: {exists: {in: group, as: entry, that: entry.count > true}}}}: [Refusal { origin: "review-scenario.yaml", scenario: Some(Authored { domain: DomainRef(QualifiedName(review.data)), name: AuthoredName("expression-review") }), cause: InvalidPredicate { view: ViewRef(QualifiedName(review.data.Items)), diagnostic: ExpressionError { code: TypeMismatch, owner: "authored row review.data.Items.satisfies", path: None, segment: None, boundary: None, message: "`entry.count > true`: operator `>` does not admit `Boolean` (Bool) and `Bool literal `true`` (Bool)" } } }, Refusal { origin: "review-scenario.yaml", scenario: Some(Authored { domain: DomainRef(QualifiedName(review.data)), name: AuthoredName("expression-review") }), cause: NothingHappens }]
  left: 2
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test malformed_bound_operands_refuse_before_the_collection_projection_gap ... FAILED

failures:

failures:
    malformed_bound_operands_refuse_before_the_collection_projection_gap

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-conformance --test adversary_expression_pass1`
exit: 101
```

```console
cargo test --locked --offline -p ess-conformance --test adversary_expression_pass1 authored_rows_preserve_scalar_controls_and_reject_aggregate_and_unprojected_reads -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-692cdd3d83e19b32)

running 1 test

thread 'authored_rows_preserve_scalar_controls_and_reject_aggregate_and_unprojected_reads' (3371309) panicked at crates/verify/ess-conformance/tests/adversary_expression_pass1.rs:63:9:
entry.amount >= 0: [Refusal { origin: "review-scenario.yaml", scenario: Some(Authored { domain: DomainRef(QualifiedName(review.data)), name: AuthoredName("expression-review") }), cause: NothingHappens }]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test authored_rows_preserve_scalar_controls_and_reject_aggregate_and_unprojected_reads ... FAILED

failures:

failures:
    authored_rows_preserve_scalar_controls_and_reject_aggregate_and_unprojected_reads

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-conformance --test adversary_expression_pass1`
exit: 101
```

```console
cargo test --locked --offline -p ess-conformance --test adversary_expression_pass1 authored_optional_enum_membership_rejects_later_invalid_values -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-692cdd3d83e19b32)

running 1 test

thread 'authored_optional_enum_membership_rejects_later_invalid_values' (3371340) panicked at crates/verify/ess-conformance/tests/adversary_expression_pass1.rs:27:5:
assertion `left == right` failed: phase == false: [Refusal { origin: "review-scenario.yaml", scenario: Some(Authored { domain: DomainRef(QualifiedName(review.data)), name: AuthoredName("expression-review") }), cause: InvalidPredicate { view: ViewRef(QualifiedName(review.data.Items)), diagnostic: ExpressionError { code: TypeMismatch, owner: "authored row review.data.Items.satisfies", path: None, segment: None, boundary: None, message: "`phase == false`: operator `==` does not admit `review.data.WrappedPhase` (Text) and `Bool literal `false`` (Bool)" } } }, Refusal { origin: "review-scenario.yaml", scenario: Some(Authored { domain: DomainRef(QualifiedName(review.data)), name: AuthoredName("expression-review") }), cause: NothingHappens }]
  left: 2
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test authored_optional_enum_membership_rejects_later_invalid_values ... FAILED

failures:

failures:
    authored_optional_enum_membership_rejects_later_invalid_values

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-conformance --test adversary_expression_pass1`
exit: 101
```

```console
cargo test --locked --offline -p ess-conformance --test adversary_expression_pass1 authored_semantic_depth_is_distinct_from_the_projection_limit -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.07s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-692cdd3d83e19b32)

running 1 test

thread 'authored_semantic_depth_is_distinct_from_the_projection_limit' (3371371) panicked at crates/verify/ess-conformance/tests/adversary_expression_pass1.rs:27:5:
assertion `left == right` failed: entry.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.amount > 0: [Refusal { origin: "review-scenario.yaml", scenario: Some(Authored { domain: DomainRef(QualifiedName(review.data)), name: AuthoredName("expression-review") }), cause: UnreadablePredicate { view: ViewRef(QualifiedName(review.data.Items)), path: "entry.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.amount" } }, Refusal { origin: "review-scenario.yaml", scenario: Some(Authored { domain: DomainRef(QualifiedName(review.data)), name: AuthoredName("expression-review") }), cause: NothingHappens }]
  left: 2
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test authored_semantic_depth_is_distinct_from_the_projection_limit ... FAILED

failures:

failures:
    authored_semantic_depth_is_distinct_from_the_projection_limit

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-conformance --test adversary_expression_pass1`
exit: 101
```

```console
cargo test --locked --offline -p ess-conformance --test adversary_expression_pass1 malformed_bound_operands_refuse_before_the_collection_projection_gap -- --exact --nocapture
   Compiling ess-conformance v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-expression-typechecking/crates/verify/ess-conformance)
    Finished `test` profile [unoptimized] target(s) in 0.24s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-692cdd3d83e19b32)

running 1 test
test malformed_bound_operands_refuse_before_the_collection_projection_gap ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

exit: 0
```

```console
cargo test --locked --offline -p ess-conformance --test adversary_expression_pass1 authored_rows_preserve_scalar_controls_and_reject_aggregate_and_unprojected_reads -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-692cdd3d83e19b32)

running 1 test
test authored_rows_preserve_scalar_controls_and_reject_aggregate_and_unprojected_reads ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

exit: 0
```

```console
cargo test --locked --offline -p ess-conformance --test adversary_expression_pass1 authored_optional_enum_membership_rejects_later_invalid_values -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-692cdd3d83e19b32)

running 1 test
test authored_optional_enum_membership_rejects_later_invalid_values ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

exit: 0
```

```console
cargo test --locked --offline -p ess-conformance --test adversary_expression_pass1 authored_semantic_depth_is_distinct_from_the_projection_limit -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-692cdd3d83e19b32)

running 1 test
test authored_semantic_depth_is_distinct_from_the_projection_limit ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

exit: 0
```

3. Complete assigned package suite, executed only after all focused cases existed and the authored setup correction passed its exact selections.

The before count is the frozen implementor's reported 683 cases, not a pre-emptive suite execution. The final runner output contains 26 summaries: ess-compiler 83, ess-conformance 253, ess-domain 361, and three zero-case rustdoc summaries, totaling 697 passed, 0 failed, 0 ignored, 0 measured, 0 filtered out. Both new integration-test binaries were selected, contributing 10 and 4 cases. The four initial setup failures are not semantic red findings and are not included in red 0.

```console
cargo test --locked --offline -p ess-domain -p ess-compiler -p ess-conformance
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
    Blocking waiting for file lock on package cache
   Compiling ess-compiler v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-expression-typechecking/crates/specify/ess-compiler)
    Finished `test` profile [unoptimized] target(s) in 1.22s
     Running unittests src/lib.rs (target/debug/deps/ess_compiler-c9815f0625d45c0f)

running 19 tests
test graph::tests::a_closure_keeps_the_edges_that_explain_each_construct_it_reached ... ok
test graph::tests::a_command_in_a_slice_brings_its_outcomes_and_what_they_name ... ok
test resolve::tests::a_code_the_bridge_has_no_class_for_still_gets_one ... ok
test graph::tests::the_construct_that_changed_is_in_its_own_closure_with_no_path ... ok
test resolve::tests::every_code_renders_as_its_family_and_number ... ok
test resolve::tests::every_named_code_is_a_family_paired_with_a_class ... ok
test graph::tests::a_closure_walks_the_edges_backwards_and_not_forwards ... ok
test graph::tests::a_slice_reaches_what_a_seed_rests_on_transitively ... ok
test resolve::tests::a_needle_that_occurs_twice_is_not_located_because_the_wrong_line_is_worse_than_none ... ok
test resolve::tests::a_bridged_refusal_is_located_by_the_declaration_its_path_names ... ok
test resolve::tests::a_refusal_is_filed_under_the_layer_its_document_path_names ... ok
test graph::tests::merging_two_graphs_can_only_ever_reach_more ... ok
test resolve::tests::the_second_needle_is_tried_when_the_first_is_ambiguous ... ok
test resolve::tests::the_register_lists_every_code_it_declares ... ok
test resolve::tests::a_declaration_written_once_is_located_at_its_own_line_and_column ... ok
test resolve::tests::with_no_files_named_a_span_still_carries_the_document_path ... ok
test resolve::tests::a_needle_in_two_files_is_not_located_because_one_of_them_is_wrong ... ok
test graph::tests::a_slice_includes_its_seeds_each_with_no_path ... ok
test resolve::tests::a_refusal_from_the_domain_crate_keeps_the_code_the_compiler_would_have_given_it ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/adversarial.rs (target/debug/deps/adversarial-24ed790bc6501187)

running 2 tests
test the_generator_reaches_both_compilation_and_refusal ... ok
test every_document_is_refused_with_reasons_or_compiled_identically_twice ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-88a619af65bbea2f)

running 10 tests
test membership_reports_every_invalid_member_even_after_a_valid_enum_literal ... ok
test a_parameter_shadowed_everywhere_is_still_unused ... ok
test unrelated_failures_survive_invalid_quantifier_targets_in_source_order ... ok
test reverse_enum_literals_and_fact_operands_follow_the_same_representation_rules ... ok
test parameter_targets_are_resolved_before_shadowing_and_uses_resume_afterward ... ok
test map_values_bind_nested_lists_and_restore_the_outer_binder ... ok
test finite_recursive_selectors_consume_segments_but_transparent_cycles_do_not ... ok
test all_owners_reject_wrong_enum_literals_through_optional_newtypes ... ok
test empty_membership_remains_typed_and_does_not_hide_aggregate_reads ... ok
test canonical_list_ordinals_are_legal_but_map_ordinals_and_wire_names_are_not ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/billing.rs (target/debug/deps/billing-327608e80009c922)

running 14 tests
test a_refusal_from_the_whole_pipeline_carries_a_code_and_the_line_it_belongs_on ... ok
test no_source_file_in_the_compiler_reads_a_clock_or_an_unordered_map ... ok
test the_billing_specification_resolves ... ok
test every_stable_reference_from_the_compiler_graph_resolves_against_its_ir ... ok
test every_handle_in_the_ir_names_something_the_ir_holds ... ok
test a_binding_that_escalates_carries_the_event_it_emits_as_a_handle ... ok
test the_json_orders_its_keys_the_way_a_btreemap_does ... ok
test a_field_keeps_the_shape_of_its_type_rather_than_a_rendering_of_it ... ok
test the_source_digest_names_exactly_the_canonical_semantic_model ... ok
test the_reaction_graph_names_the_binding_that_causes_each_command ... ok
test the_crossing_between_two_contexts_is_recorded_with_the_reason_someone_gave_for_it ... ok
test compiling_the_billing_example_twice_produces_byte_identical_json ... ok
test compiling_without_the_file_list_still_reports_the_document_path ... ok
test canonical_json_ends_in_a_newline ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/expression_validation.rs (target/debug/deps/expression_validation-22eb9896f95b8ce8)

running 19 tests
test a_bad_membership_member_is_checked_after_a_valid_one ... ok
test entity_scalar_continuation_fails_at_admission ... ok
test enum_literals_are_checked_in_command_guards ... ok
test entity_state_has_its_closed_enum_type_and_no_selectors ... ok
test both_compiler_entries_preserve_valid_predicates_and_canonical_bytes ... ok
test guard_scalar_continuation_fails_at_admission ... ok
test diagnostics_keep_real_owners_and_never_invent_predicate_leaf_spans ... ok
test command_subject_identity_stays_forbidden_as_a_free_read_inside_quantifiers ... ok
test declared_field_names_remain_expression_names_when_wire_names_differ ... ok
test optional_newtypes_keep_their_representation_without_a_value_selector ... ok
test ordering_boolean_is_a_type_error ... ok
test filter_scalar_continuation_fails_at_admission ... ok
test newtype_scalar_continuation_fails_at_admission ... ok
test quantified_bodies_are_checked_in_command_guards ... ok
test struct_scalar_continuation_fails_at_admission ... ok
test the_control_assembles_and_compiles ... ok
test both_fact_operands_are_resolved ... ok
test integer_fractional_comparison_and_decimal_interval_both_validate ... ok
test nested_view_parameters_count_as_used_and_source_only_fields_are_readable ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/oracle_fixture.rs (target/debug/deps/oracle_fixture-bc712a3de7549642)

running 11 tests
test the_eventual_view_converges_on_a_state_the_creating_command_does_not_reach ... ok
test a_row_reaches_the_read_your_writes_view_after_a_single_command ... ok
test the_command_every_binding_invokes_can_be_forced_to_fail ... ok
test an_illegal_transition_can_be_attempted_from_a_state_a_scenario_can_reach ... ok
test every_on_failure_policy_the_model_has_is_reachable_in_this_fixture ... ok
test dropping_one_binding_leaves_others_with_scenarios_of_their_own ... ok
test the_fixture_compiles_from_the_files_it_lives_in ... ok
test an_outcome_updates_an_entity_without_moving_it_and_that_entity_declares_an_invariant ... ok
test a_binding_maps_an_event_field_that_has_a_same_typed_sibling ... ok
test the_fixture_carries_something_the_normative_example_does_not ... ok
test every_input_the_oracle_needs_is_carried_by_one_of_the_examples ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/sealed_state.rs (target/debug/deps/sealed_state-80695927ae9c94ab)

running 3 tests
test every_compiler_entrance_validates_before_resolution ... ok
test validated_and_resolved_state_have_no_public_fields ... ok
test provenance_never_hashes_an_empty_serialization_fallback ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/view_shapes.rs (target/debug/deps/view_shapes-0787cbb244e82b63)

running 1 test
test a_shape_is_one_handle_with_checked_fields_in_every_view ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/wire_fields.rs (target/debug/deps/wire_fields-d180c2f2955c34be)

running 4 tests
test collisions_accumulate_with_semantic_source_locations_before_ir_exists ... ok
test separate_namespaces_special_keys_and_display_names_do_not_collide ... ok
test entity_identity_and_synthetic_state_share_the_field_namespace ... ok
test every_object_namespace_refuses_explicit_and_default_key_collisions ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running unittests src/lib.rs (target/debug/deps/ess_conformance-55cacd48a28793b4)

running 67 tests
test decision::tests::a_decision_reads_its_two_other_cases_as_neither_satisfied_nor_the_other ... ok
test faulty::tests::only_the_two_faults_the_boundary_cannot_express_are_injected_in_the_implementation ... ok
test go::tests::every_go_file_is_in_the_package_the_readme_names ... ok
test evidence::tests::a_standalone_report_carries_every_field_an_adapter_needs ... ok
test decision::tests::exactly_one_reason_says_another_candidate_would_help ... ok
test evidence::tests::report_readers_refuse_more_nonpasses_than_executed_scenarios ... ok
test evidence::tests::report_readers_refuse_malformed_and_unsupported_suite_versions ... ok
test evidence::tests::report_readers_do_not_guess_a_producer_from_status_vocabulary ... ok
test evidence::tests::report_readers_preserve_go_producer_bytes_and_historical_nonpass_counts ... ok
test evidence::tests::unknown_report_fields_are_refused ... ok
test evidence::tests::report_readers_refuse_nonpass_count_and_list_disagreement ... ok
test evidence::tests::report_readers_refuse_status_claims_that_contradict_the_list ... ok
test faulty::tests::every_fault_says_what_it_is_and_where_it_goes ... ok
test faulty::tests::no_two_faults_claim_the_same_scenario ... ok
test input::tests::a_primitive_refuses_a_node_of_the_wrong_shape_rather_than_coercing_it ... ok
test evidence::tests::report_readers_refuse_unknown_report_formats ... ok
test evidence::tests::the_closed_report_round_trips_with_identical_canonical_bytes ... ok
test faulty::tests::a_fault_is_injected_into_the_system_that_declares_what_it_breaks ... ok
test decision::tests::a_refusal_renders_the_predicate_the_command_and_every_reason ... ok
test report::tests::a_quoted_input_reads_as_the_call_that_was_made ... ok
test go::tests::the_runner_is_a_constant_and_only_the_suite_moves ... ok
test report::tests::an_unsupported_scenario_makes_the_run_fail_rather_than_look_like_a_pass ... ok
test report::tests::every_check_code_has_a_distinct_name_and_a_rule_sentence ... ok
test runner::tests::a_count_with_neither_bound_is_a_suite_defect_and_not_a_satisfied_assertion ... ok
test evidence::tests::report_readers_refuse_nonpass_entries_without_a_known_nonpass_status ... ok
test runner::tests::a_count_is_the_half_of_an_ordering_claim_that_says_the_rows_were_there ... ok
test report::tests::a_scenario_status_is_the_strongest_of_its_checks_and_a_contradiction_outranks_everything ... ok
test input::tests::every_primitive_projects_to_the_one_fact_value_that_can_hold_it ... ok
test input::tests::shape_errors_render_one_per_line_and_name_the_input_root_by_name ... ok
test report::tests::a_diagnostic_answers_all_five_of_the_questions_a_failure_has_to_answer ... ok
test runner::tests::a_view_that_holds_nothing_does_not_satisfy_an_invariant_by_being_empty ... ok
test scenario::tests::a_scenario_id_that_names_no_construct_is_refused ... ok
test scenario::tests::a_suite_format_from_a_later_build_is_refused_rather_than_guessed ... ok
test runner::tests::a_position_names_both_ends_and_a_row_that_is_not_there_is_not_a_match ... ok
test scenario::tests::a_declared_leaf_admits_what_its_type_admits_and_absence_only_where_the_type_permits_it ... ok
test runner::tests::a_position_in_a_view_that_declares_no_order_is_a_suite_defect ... ok
test scenario::tests::a_transition_ref_refuses_a_name_no_lifecycle_can_declare ... ok
test runner::tests::a_ranking_key_a_row_does_not_publish_is_undecidable_rather_than_out_of_order ... ok
test synthesize::tests::a_refusal_names_the_construct_the_code_and_the_repair ... ok
test runner::tests::an_order_over_fewer_than_two_rows_holds_and_does_not_double_as_a_non_emptiness_claim ... ok
test web::tests::no_comparison_sits_in_a_text_node_mustache ... ok
test runner::tests::a_declared_order_is_checked_on_adjacent_rows_and_the_next_key_breaks_a_tie ... ok
test runner::tests::an_empty_field_set_means_a_row_exists_and_not_that_anything_will_do ... ok
test witness::tests::a_text_witness_is_its_own_path_so_two_fields_of_one_type_never_agree ... ok
test runner::tests::the_runners_clock_advances_on_every_read_so_a_deadline_can_bound_anything ... ok
test scenario::tests::a_scenario_id_names_the_construct_it_exercises_rather_than_its_position ... ok
test scenario::tests::a_payload_shape_round_trips_through_the_form_a_suite_is_stored_in ... ok
test runner::tests::a_nested_row_binds_the_paths_a_predicate_spells ... ok
test scenario::tests::a_suite_refuses_a_second_scenario_under_one_id ... ok
test scenario::tests::an_invariant_scenario_is_keyed_by_the_entity_and_the_branch_and_never_by_a_position ... ok
test witness::tests::an_enum_offers_every_variant_it_declares_and_the_first_one_only_once ... ok
test scenario::tests::two_scenarios_about_the_same_thing_in_the_same_way_are_one_id ... ok
test witness::tests::an_integer_leaf_is_never_offered_a_fractional_candidate ... ok
test evidence::tests::report_readers_preserve_rust_producer_bytes_for_every_supported_suite ... ok
test scenario::tests::every_binding_aspect_is_in_the_list_that_is_walked_to_produce_them ... ok
test synthesize::tests::every_refusal_carries_a_distinct_code_in_one_family ... ok
test scenario::tests::the_ids_of_a_suite_sort_the_way_a_reader_sorts_the_file ... ok
test scenario::tests::every_scenario_id_reads_back_from_the_form_a_report_prints ... ok
test web::tests::the_page_calls_nothing_the_player_does_not_return ... ok
test web::tests::the_page_is_specification_neutral ... ok
test runner::tests::ids_come_from_the_suite_and_from_nothing_ambient ... ok
test scenario::tests::a_semantic_reference_renders_the_way_the_design_writes_one ... ok
test runner::tests::a_predicate_a_row_cannot_answer_is_reported_rather_than_retried ... ok
test witness::tests::two_uuid_witnesses_differ_and_neither_moves_when_a_third_field_appears ... ok
test witness::tests::the_alternatives_for_a_number_are_the_guards_own_literals_either_side ... ok
test scenario::tests::a_purpose_is_one_line_and_says_something ... ok
test witness::tests::the_candidate_count_is_bounded_however_many_fields_a_guard_reads ... ok

test result: ok. 67 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-692cdd3d83e19b32)

running 4 tests
test malformed_bound_operands_refuse_before_the_collection_projection_gap ... ok
test authored_optional_enum_membership_rejects_later_invalid_values ... ok
test authored_rows_preserve_scalar_controls_and_reject_aggregate_and_unprojected_reads ... ok
test authored_semantic_depth_is_distinct_from_the_projection_limit ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/authored.rs (target/debug/deps/authored-58ae8c531cef1982)

running 49 tests
test a_bounded_negative_that_forbids_no_event_is_refused ... ok
test a_field_the_surface_does_not_declare_is_refused_by_name ... ok
test a_positional_claim_takes_the_order_from_the_view_rather_than_from_the_author ... ok
test a_halt_of_a_listing_the_model_calls_eventual_retries_because_the_model_said_so ... ok
test a_document_that_is_not_one_is_refused_rather_than_read_as_an_empty_scenario ... ok
test a_halt_after_no_rows_at_all_is_refused_rather_than_compiled ... ok
test a_scenario_that_runs_nothing_is_refused_rather_than_counted_as_a_check ... ok
test a_domain_the_model_does_not_declare_is_refused_by_name ... ok
test a_halt_compiles_to_a_step_of_its_own_and_not_to_a_claim_about_rows ... ok
test a_state_the_lifecycle_does_not_declare_is_refused_as_a_state_and_not_as_a_variant ... ok
test a_value_read_off_an_event_nothing_required_is_refused ... ok
test a_position_in_a_view_that_declares_no_order_is_refused ... ok
test a_name_a_closed_set_does_not_have_is_refused_with_the_set ... ok
test a_view_the_model_does_not_declare_is_refused_by_name ... ok
test a_timeline_whose_instants_do_not_ascend_is_refused ... ok
test a_format_this_build_does_not_implement_is_refused_before_anything_is_read ... ok
test a_claim_the_timelines_own_instants_contradict_is_refused ... ok
test a_value_the_declared_type_does_not_admit_is_refused_where_it_sits ... ok
test a_command_the_model_does_not_declare_is_refused_by_name ... ok
test a_halt_claimed_of_a_listing_with_no_declared_order_is_refused_by_the_code_that_already_says_so ... ok
test a_scenario_compiles_to_the_id_the_domain_and_the_name_make ... ok
test a_reference_where_the_suite_compares_a_value_it_carries_is_refused ... ok
test a_halt_stated_beside_another_claim_is_two_assertions_filed_as_one ... ok
test a_predicate_reading_something_the_view_does_not_publish_is_refused ... ok
test a_declared_field_nothing_supplies_is_refused_by_name ... ok
test a_window_of_no_seconds_is_refused_rather_than_compiled_into_a_check_that_cannot_fail ... ok
test an_elapsed_claim_compiles_to_the_four_steps_that_carry_it_and_they_come_before_the_act ... ok
test an_event_the_model_does_not_declare_is_refused_by_name ... ok
test an_entity_the_model_does_not_declare_is_refused_by_name ... ok
test an_actor_the_model_does_not_declare_is_refused_by_name ... ok
test a_window_measured_from_an_instant_nothing_marked_is_refused_with_the_ones_that_are ... ok
test an_error_the_model_does_not_declare_is_refused_by_name ... ok
test an_actor_the_specification_does_not_grant_the_command_is_refused ... ok
test an_instance_bound_to_a_field_that_cannot_hold_an_identity_is_refused ... ok
test a_window_that_states_other_than_one_bound_is_refused ... ok
test an_instance_the_arrangement_does_not_declare_is_refused_by_name ... ok
test authored_predicate_operand_errors_have_their_own_refusal ... ok
test an_instance_named_before_anything_binds_it_is_refused ... ok
test one_name_for_two_instants_is_refused_rather_than_read_as_the_later_one ... ok
test an_assertion_that_states_other_than_one_claim_is_refused ... ok
test the_committed_billing_suite_holds_the_authored_scenario_beside_the_generated_ones ... ok
test two_files_naming_one_scenario_are_refused_rather_than_one_displacing_the_other ... ok
test an_outcome_the_command_does_not_declare_is_refused_with_the_ones_it_does ... ok
test the_steps_are_the_vocabulary_a_generated_scenario_already_uses ... ok
test the_order_the_files_are_handed_over_in_does_not_reach_the_result ... ok
test authored_aggregate_presence_keeps_026_and_valid_scalar_reads_keep_the_predicate ... ok
test an_act_cannot_open_a_window_at_its_own_instant ... ok
test every_cause_is_reachable_from_a_document ... ok
test two_compilations_of_one_file_produce_identical_bytes ... ok

test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

     Running tests/elapsed.rs (target/debug/deps/elapsed-344e830da50565e2)

running 7 tests
test two_runs_over_one_window_produce_byte_identical_reports ... ok
test a_target_with_no_clock_reports_unsupported_and_the_run_fails ... ok
test a_target_that_holds_the_window_and_reports_it_passes ... ok
test a_window_opened_at_an_instant_nothing_marked_is_a_suite_defect_and_not_a_failed_implementation ... ok
test a_deadline_the_target_ran_past_fails_the_within_claim ... ok
test a_target_whose_clock_never_moves_fails_rather_than_being_read_as_having_waited ... ok
test an_event_published_inside_the_window_fails_the_bounded_negative_and_nothing_else ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/execution.rs (target/debug/deps/execution-14c86f734d8ebad7)

running 12 tests
test an_eventual_assertion_asks_again_within_a_deadline_and_never_sleeps ... ok
test a_target_that_cannot_expose_an_observation_fails_the_run_rather_than_skipping_it ... ok
test a_read_your_writes_view_is_not_quietly_read_at_current_when_no_token_came_back ... ok
test an_event_missing_a_field_it_declares_is_named_leaf_by_leaf_rather_than_reported_as_absent ... ok
test a_scenario_whose_input_no_longer_reaches_its_branch_fails_with_a_diagnostic_naming_the_defect ... ok
test a_view_answered_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test every_scenario_the_billing_specification_obliges_passes_against_the_reference_implementation ... ok
test every_scenario_checked_something_and_no_family_of_them_was_silently_empty ... ok
test a_view_assertion_names_the_instance_the_scenario_created_rather_than_any_row ... ok
test a_value_of_the_wrong_declared_type_is_caught_by_the_same_check_as_a_missing_one ... ok
test an_eventual_view_is_read_again_and_a_read_your_writes_view_is_not ... ok
test two_runs_of_one_suite_against_one_target_produce_byte_identical_reports ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

     Running tests/faults.rs (target/debug/deps/faults-81ca2f3021f73626)

running 11 tests
test a_fault_nothing_catches_is_recorded_rather_than_quietly_dropped ... ok
test every_fault_that_could_be_a_boundary_perturbation_is_one ... ok
test dropping_one_binding_leaves_the_other_two_green ... ok
test a_wrong_mapping_is_invisible_to_a_target_that_cannot_show_its_invocations ... ok
test the_diagnostic_of_a_caught_fault_names_the_defect_rather_than_reporting_that_something_broke ... ok
test the_widest_blast_radius_is_scenarios_that_could_not_be_arranged_rather_than_extra_verdicts ... ok
test each_specification_is_passed_in_full_by_the_implementation_written_from_it ... ok
test two_runs_against_one_faulty_target_produce_byte_identical_reports ... ok
test a_faults_blast_radius_is_accounted_for ... ok
test a_fault_does_not_simply_break_everything ... ok
test each_fault_fails_the_scenario_that_exists_to_catch_it ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s

     Running tests/halt.rs (target/debug/deps/halt-d27e9825da93e11f)

running 7 tests
test a_target_whose_producer_stops_when_the_reader_does_passes ... ok
test two_runs_over_one_halt_claim_produce_byte_identical_reports ... ok
test retrying_does_not_rescue_a_producer_that_never_stops ... ok
test a_halt_of_an_eventual_listing_is_asked_again_while_the_projection_catches_up ... ok
test a_listing_that_ran_out_before_the_reader_stopped_it_is_not_a_halt ... ok
test a_target_that_reads_the_whole_listing_fails_rather_than_being_read_as_having_stopped ... ok
test a_target_that_cannot_read_a_row_at_a_time_reports_unsupported_and_the_run_fails ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/report_reader_adversary.rs (target/debug/deps/report_reader_adversary-bdcf01bf6f8c5d8b)

running 4 tests
test duplicate_claims_cannot_hide_behind_a_valid_last_value ... ok
test count_extremes_refuse_contradictions_without_inventing_coverage ... ok
test closed_wire_fields_preserve_their_formats_scalar_contracts ... ok
test aggregate_status_does_not_depend_on_nonpass_order_or_multiplicity ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s

     Running tests/suite.rs (target/debug/deps/suite-a606f28467d04f39)

running 14 tests
test the_steps_a_binding_and_an_invariant_need_survive_being_read_back_from_text ... ok
test a_suite_naming_something_that_is_not_an_ess_name_is_refused_while_it_is_read ... ok
test a_suite_parses_from_text_alone_without_an_ir ... ok
test a_count_and_a_position_read_back_as_what_a_runner_in_another_language_must_read ... ok
test the_scan_for_a_clock_finds_one_and_does_not_find_a_word_that_merely_ends_in_a_banned_token ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test every_scenario_id_the_billing_model_can_produce_reads_back ... ok
test the_step_vocabulary_expresses_the_worked_example_from_section_ten ... ok
test inserting_one_outcome_re_keys_nothing_around_it ... ok
test the_scenario_ids_appear_in_the_file_in_the_order_a_sorted_key_list_would_be ... ok
test the_dependency_set_names_a_type_no_derived_from_would_have_mentioned ... ok
test a_suite_serialised_in_one_process_resolves_in_another ... ok
test the_suite_records_the_same_model_digest_the_projections_do ... ok
test serialising_a_suite_twice_produces_byte_identical_json ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/synthesis.rs (target/debug/deps/synthesis-e2b2d44b3e57f53d)

running 50 tests
test a_command_that_accepts_a_wrong_state_is_asserted_as_accepting_rather_than_refusing ... ok
test a_binding_whose_branch_the_event_decides_refuses_the_flow_and_still_checks_the_mapping ... ok
test a_command_that_declares_no_wrong_state_answer_is_refused_by_name_beside_its_scenario ... ok
test a_guard_no_candidate_can_satisfy_is_refused_with_the_number_tried ... ok
test a_component_nothing_declares_is_refused_by_name ... ok
test a_binding_mapping_names_the_source_the_document_wrote_and_not_its_same_typed_sibling ... ok
test a_move_is_observed_through_the_view_the_state_it_left_is_filtered_on ... ok
test a_declared_error_is_asserted_by_name_and_never_by_an_invented_payload ... ok
test a_filter_reading_something_no_scenario_knows_refuses_rather_than_guessing ... ok
test a_state_reached_only_through_a_branch_no_input_reaches_is_refused_rather_than_arranged ... ok
test a_parameterised_view_is_queried_with_the_value_the_scenario_put_in_the_row ... ok
test an_entity_nothing_creates_cannot_be_acted_on_and_says_so ... ok
test a_binding_flow_is_proved_through_the_event_the_invoked_command_publishes ... ok
test a_declared_order_is_asserted_against_two_rows_the_scenario_arranged_itself ... ok
test a_read_your_writes_view_filled_by_the_command_that_ran_is_asserted_to_hold_a_row ... ok
test a_view_is_asserted_in_the_block_its_own_consistency_decides ... ok
test a_scenario_that_moves_an_instance_names_the_one_an_earlier_step_created ... ok
test a_binding_that_retries_forces_one_failure_and_still_requires_the_consequence ... ok
test a_value_object_nothing_observable_holds_keeps_a_refusal_naming_what_would_close_it ... ok
test a_binding_that_drops_its_failures_refuses_that_check_and_names_the_reason ... ok
test a_move_that_is_illegal_in_a_state_is_attempted_with_the_input_that_would_have_worked ... ok
test a_value_objects_own_invariants_are_read_at_every_field_position_a_view_holds_one ... ok
test an_order_the_specification_cannot_put_two_rows_under_is_refused_and_not_asserted ... ok
test an_undecidable_guard_refuses_and_does_not_spend_the_candidate_budget ... ok
test a_synthesised_count_is_a_floor_the_scenario_arranged_and_never_a_ceiling ... ok
test a_suite_for_one_component_holds_only_what_that_component_realises ... ok
test an_actor_is_named_only_where_the_specification_grants_the_command ... ok
test a_view_that_does_not_hold_the_instance_yet_is_not_asked_about_its_invariants ... ok
test a_view_the_entity_has_not_reached_yet_is_asserted_to_exclude_the_instance_by_name ... ok
test an_invariant_is_asserted_against_every_view_that_publishes_what_it_reads ... ok
test an_invariant_over_a_field_no_view_publishes_refuses_rather_than_being_dropped ... ok
test an_at_least_once_binding_delivers_the_event_twice_and_requires_no_count ... ok
test a_binding_that_escalates_requires_the_event_the_escalation_declares ... ok
test a_synthesised_suite_survives_being_written_and_read_back ... ok
test an_event_assertion_carries_the_declared_shape_and_exactly_the_values_the_payload_determines ... ok
test an_outcome_that_updates_an_instance_acts_on_one_the_scenario_created ... ok
test a_whole_system_suite_does_not_mention_a_component ... ok
test an_illegal_move_requires_the_branch_and_the_declared_error_rather_than_merely_failing ... ok
test the_failure_control_is_armed_after_the_arrangement_and_before_the_command_that_triggers_it ... ok
test each_example_synthesises_the_families_its_specification_declares ... ok
test every_declared_outcome_is_either_a_scenario_or_a_named_refusal_or_asserted_by_the_state_family ... ok
test every_declared_transition_has_a_scenario_that_proves_it_can_occur ... ok
test every_command_names_an_instance_an_earlier_step_of_the_same_scenario_bound ... ok
test the_dependency_set_names_the_types_the_scenario_is_made_of ... ok
test the_refusal_branch_asserts_that_no_event_the_specification_declares_occurred ... ok
test the_input_a_scenario_sends_is_re_decided_against_the_guard_it_claims_to_reach ... ok
test an_outcome_no_input_decides_is_reached_by_injection_and_by_nothing_else ... ok
test every_clause_of_every_binding_is_either_a_scenario_or_a_named_refusal ... ok
test synthesising_the_same_specification_twice_produces_byte_identical_output ... ok
test canonical_expression_compatibility_fixtures ... ok

test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.78s

     Running tests/witness.rs (target/debug/deps/witness-f22e8916a8b41aea)

running 28 tests
test a_disjunction_one_of_whose_branches_holds_is_satisfied_despite_an_undecidable_branch ... ok
test a_candidate_input_projects_one_fact_per_scalar_leaf ... ok
test a_candidate_missing_a_required_field_is_refused_before_any_guard_is_read ... ok
test a_path_into_a_list_or_a_union_names_the_aggregate_rather_than_the_missing_element ... ok
test a_list_a_map_and_a_union_bind_no_fact_in_the_current_projection ... ok
test a_newtype_is_transparent_so_a_deep_path_reaches_through_it_without_a_segment ... ok
test a_newtype_is_transparent_when_a_path_is_resolved_as_well_as_when_it_is_projected ... ok
test a_candidate_carrying_a_field_no_type_declares_is_refused ... ok
test a_refuted_guard_carries_the_leaf_and_the_value_that_refuted_it ... ok
test an_absent_optional_is_unevaluable_but_says_a_candidate_could_repair_it ... ok
test equality_over_two_texts_is_decided_even_though_ordering_them_is_not ... ok
test a_path_landing_on_an_aggregate_is_unevaluable_by_construction ... ok
test ordering_two_texts_is_unevaluable_because_an_ess_specification_declares_no_scale ... ok
test a_conjunction_of_two_undecidable_leaves_reports_both ... ok
test a_long_recursive_read_validates_beyond_the_projection_limit ... ok
test a_scalar_of_the_wrong_shape_is_refused_rather_than_coerced ... ok
test unclassified_is_a_drift_alarm_and_no_enumerated_source_trips_it ... ok
test malformed_declarations_refuse_early_and_direct_bad_reads_remain_unknown ... ok
test otherwise_and_external_are_not_guards_over_the_input ... ok
test the_same_text_ordering_is_decidable_once_a_scale_contains_both_values ... ok
test a_refusal_names_the_predicate_the_command_and_the_path ... ok
test the_normative_shape_of_guard_is_decidable_for_both_signs ... ok
test resolved_adapter_keeps_semantics_separate_from_collection_projection ... ok
test ordering_across_two_types_is_unevaluable_not_false ... ok
test an_absent_optional_binds_nothing_rather_than_binding_a_default ... ok
test expression_search_limits_do_not_define_type_correctness ... ok
test only_an_absent_value_says_another_candidate_would_help ... ok
test legal_collection_cardinality_is_not_currently_projected ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running unittests src/lib.rs (target/debug/deps/ess_domain-0f5a87ed42c0f62e)

running 334 tests
test binding::tests::a_binding_name_is_lower_kebab_because_it_becomes_a_generated_name ... ok
test actor::tests::a_malformed_command_name_is_refused_when_the_document_is_read ... ok
test binding::tests::a_binding_that_does_not_say_what_happens_when_it_fails_is_refused ... ok
test binding::tests::a_binding_that_escalates_into_an_event_nothing_declares_is_refused ... ok
test actor::tests::a_key_the_model_does_not_know_is_refused ... ok
test actor::tests::every_grant_that_names_nothing_is_reported_not_just_the_first ... ok
test actor::tests::a_grant_follows_the_command_identity_and_not_its_wire_name ... ok
test binding::tests::a_binding_names_the_records_that_explain_it_and_writes_them_back ... ok
test actor::tests::an_actor_round_trips_through_yaml ... ok
test binding::tests::a_binding_that_invokes_a_command_nothing_declares_is_refused ... ok
test actor::tests::the_actors_from_the_design_document_validate ... ok
test binding::tests::a_delivery_guarantee_this_build_does_not_implement_is_refused_while_the_document_is_read ... ok
test binding::tests::a_declared_conversion_is_what_lets_two_distinct_types_meet ... ok
test actor::tests::an_actor_permitted_a_command_that_does_not_exist_is_refused ... ok
test binding::tests::a_binding_that_names_no_record_writes_no_key ... ok
test binding::tests::a_mapping_that_reads_a_field_the_event_does_not_have_is_refused ... ok
test binding::tests::a_literal_that_has_structure_underneath_it_is_refused ... ok
test binding::tests::a_binding_assembled_in_code_is_checked_like_a_parsed_one ... ok
test binding::tests::a_binding_that_reacts_to_an_event_nothing_declares_is_refused ... ok
test binding::tests::a_literal_that_is_not_a_variant_of_the_input_it_fills_is_refused ... ok
test binding::tests::a_mapping_between_two_types_with_no_declared_conversion_is_refused ... ok
test binding::tests::a_misspelt_event_prefix_is_refused_rather_than_sent_as_text ... ok
test binding::tests::a_nested_path_is_refused_as_unsupported_rather_than_as_a_missing_field ... ok
test binding::tests::a_mapping_that_writes_an_input_the_command_does_not_take_is_refused ... ok
test binding::tests::a_literal_that_names_a_field_of_the_event_is_refused ... ok
test binding::tests::a_source_that_names_no_field_after_the_prefix_is_refused ... ok
test binding::tests::a_literal_cannot_fill_an_input_that_is_not_text ... ok
test actor::tests::an_actor_that_may_invoke_nothing_is_not_an_error ... ok
test binding::tests::a_failure_policy_that_is_none_of_the_three_words_is_told_which_three_exist ... ok
test binding::tests::only_escalate_takes_a_block_because_it_is_the_only_policy_that_publishes_anything ... ok
test command::tests::a_command_whose_every_outcome_is_external_specifies_no_change_rather_than_nothing_at_all ... ok
test binding::tests::an_escalation_event_on_a_binding_that_does_not_escalate_is_refused ... ok
test binding::tests::an_escalation_names_the_event_it_emits_and_that_event_reaches_the_binding ... ok
test binding::tests::a_literal_that_merely_contains_a_dot_is_still_a_literal ... ok
test binding::tests::an_input_mapped_twice_is_refused ... ok
test binding::tests::one_on_failure_says_one_thing ... ok
test binding::tests::a_mapping_source_renders_as_the_document_wrote_it ... ok
test command::tests::a_command_with_an_accepted_and_a_rejected_outcome_round_trips_through_yaml ... ok
test binding::tests::an_input_the_command_requires_and_the_mapping_omits_is_refused ... ok
test binding::tests::an_escalation_that_names_no_event_is_refused_because_nothing_could_prove_it_happened ... ok
test binding::tests::the_published_schema_offers_both_spellings_of_on_failure ... ok
test binding::tests::the_published_schema_still_describes_a_mapping_as_an_object_of_strings ... ok
test binding::tests::an_optional_input_the_mapping_omits_is_accepted ... ok
test binding::tests::the_binding_the_example_ships_typechecks ... ok
test binding::tests::every_problem_in_a_binding_is_reported_at_once ... ok
test command::tests::a_command_declares_at_most_one_wrong_state_branch ... ok
test command::tests::a_branch_that_accepts_a_wrong_state_needs_no_error_and_may_not_name_one ... ok
test command::tests::a_branch_decided_by_the_field_that_names_its_instance_is_refused ... ok
test command::tests::a_command_with_no_catch_all_branch_is_not_refused_as_a_wedged_state_machine ... ok
test command::tests::a_command_with_no_outcomes_is_refused ... ok
test command::tests::a_payload_whose_two_types_disagree_needs_a_declared_conversion ... ok
test command::tests::a_repeated_payload_line_is_refused_rather_than_silently_last_wins ... ok
test command::tests::an_event_records_a_fact_and_nothing_about_how_it_travels ... ok
test command::tests::a_wrong_state_branch_is_neither_the_catch_all_nor_reachable_by_choosing_an_input ... ok
test command::tests::an_event_field_with_no_declared_source_is_undetermined_and_not_an_error ... ok
test command::tests::an_external_outcome_must_say_what_fails ... ok
test command::tests::a_payload_reading_an_undeclared_input_accumulates_beside_its_neighbours ... ok
test command::tests::a_wrong_state_branch_survives_a_trip_through_the_document_form ... ok
test command::tests::a_payload_literal_cannot_fill_a_field_with_structure ... ok
test command::tests::a_wrong_state_branch_that_names_no_error_is_refused ... ok
test command::tests::an_event_field_declared_twice_is_refused ... ok
test command::tests::a_when_predicate_may_only_read_declared_input_fields ... ok
test command::tests::an_error_payload_field_must_name_a_declared_type ... ok
test command::tests::an_input_field_declared_twice_is_refused ... ok
test command::tests::a_payload_filling_a_field_the_event_does_not_carry_is_refused ... ok
test command::tests::a_move_written_without_an_entity_is_refused_rather_than_read_as_one ... ok
test command::tests::an_instance_named_by_a_branch_that_changes_nothing_is_refused ... ok
test command::tests::an_input_field_must_name_a_declared_type ... ok
test command::tests::a_payload_literal_that_names_an_input_is_a_misspelled_reference ... ok
test command::tests::an_outcome_that_both_creates_and_moves_is_refused_naming_both_keys ... ok
test command::tests::outcome_names_are_lower_kebab ... ok
test command::tests::an_outcome_that_moves_an_entity_without_naming_an_instance_is_refused ... ok
test command::tests::a_payload_literal_with_a_misspelt_prefix_is_a_misspelled_reference ... ok
test command::tests::a_test_strategy_is_decided_by_the_model_and_not_by_a_generator ... ok
test command::tests::a_when_that_is_trivially_true_is_the_default_branch_written_the_long_way ... ok
test command::tests::setting_a_field_from_something_the_command_does_not_take_is_refused ... ok
test command::tests::a_payload_mapping_parses_validates_and_round_trips ... ok
test command::tests::an_outcome_cannot_be_both_conditional_and_external ... ok
test command::tests::setting_a_field_on_no_entity_is_refused ... ok
test command::tests::a_payload_for_an_event_the_branch_does_not_emit_is_refused ... ok
test command::tests::an_outcome_that_neither_emits_nor_errors_is_refused ... ok
test command::tests::an_undeclared_event_and_an_undeclared_error_are_both_reported ... ok
test command::tests::setting_one_field_twice_is_refused_rather_than_silently_keeping_the_last ... ok
test command::tests::writing_wrong_state_false_is_the_same_document_as_leaving_the_key_out ... ok
test command::tests::only_a_refusing_wrong_state_branch_writes_the_key_back ... ok
test command::tests::an_error_is_part_of_a_vocabulary_an_outcome_can_name ... ok
test command::tests::an_event_field_must_name_a_declared_type ... ok
test command::tests::two_unconditional_outcomes_are_refused_as_non_deterministic ... ok
test command::tests::an_outcome_declared_twice_is_refused ... ok
test component::tests::a_component_reached_over_a_network_that_only_projects_a_view_is_accepted ... ok
test component::tests::a_command_placed_twice_is_refused ... ok
test command::tests::an_outcome_that_names_an_error_and_also_emits_is_refused ... ok
test command::tests::an_outcome_that_reports_an_error_and_also_changes_an_entity_is_refused ... ok
test command::tests::an_outcome_cannot_be_both_wrong_state_and_decided_by_the_input ... ok
test component::tests::a_component_name_spelt_like_a_type_is_refused ... ok
test command::tests::an_outcome_names_the_entity_it_changes_and_the_move_it_takes ... ok
test command::tests::an_outcome_says_which_entity_fields_it_sets_and_from_where ... ok
test command::tests::refuses_is_refused_on_a_branch_that_has_no_refusal_to_describe ... ok
test component::tests::a_command_is_handled_by_the_owner_of_its_innermost_domain ... ok
test component::tests::a_command_tree_places_every_accepted_command_exactly_once ... ok
test component::tests::a_command_line_surface_without_a_tree_is_refused ... ok
test command::tests::an_outcome_that_changes_no_entity_says_so_by_saying_nothing ... ok
test component::tests::a_component_accepting_a_command_from_a_domain_no_component_owns_is_allowed ... ok
test component::tests::a_component_accepting_a_command_nothing_declares_is_refused ... ok
test component::tests::a_component_publishing_an_event_from_a_domain_it_does_not_own_is_allowed ... ok
test component::tests::a_component_publishing_an_event_nothing_declares_is_refused ... ok
test component::tests::a_command_tree_on_a_component_reached_another_way_is_refused ... ok
test component::tests::a_component_owning_a_domain_nothing_declares_is_refused ... ok
test component::tests::a_component_accepting_a_command_another_component_owns_the_domain_of_is_refused ... ok
test component::tests::a_component_reached_over_a_network_that_serves_nothing_is_refused ... ok
test component::tests::a_components_one_line_summary_is_read_from_either_spelling ... ok
test component::tests::a_declared_domain_no_component_owns_is_not_an_error ... ok
test component::tests::a_component_that_owns_nothing_accepts_nothing_and_publishes_nothing_is_refused ... ok
test component::tests::a_component_saying_nothing_about_a_command_line_serialises_as_it_did_before ... ok
test component::tests::a_component_says_nothing_about_reach_unless_it_says_something_about_reach ... ok
test component::tests::a_key_the_model_does_not_know_is_refused ... ok
test component::tests::two_components_claiming_a_domain_nothing_declares_report_the_reference_and_not_a_conflict ... ok
test domain::tests::a_qualified_member_from_another_domain_is_refused_when_the_document_is_read ... ok
test component::tests::two_components_owning_the_same_domain_is_refused ... ok
test domain::tests::a_name_declared_twice_in_one_domain_is_refused ... ok
test component::tests::a_tree_placing_a_command_the_component_does_not_accept_is_refused ... ok
test component::tests::a_view_the_command_tree_gives_no_place_is_refused ... ok
test domain::tests::a_domain_nested_inside_another_is_refused_as_a_conflict_and_not_as_a_repeat ... ok
test domain::tests::a_domain_owns_names_rather_than_holding_declarations ... ok
test domain::tests::a_bare_actor_name_is_qualified_and_owned_like_every_other_member ... ok
test domain::tests::every_broken_member_is_reported_not_only_the_first ... ok
test component::tests::a_reach_the_model_does_not_declare_is_refused_by_the_word ... ok
test component::tests::a_group_that_is_not_a_typed_word_is_refused ... ok
test component::tests::a_view_the_command_tree_places_is_accepted ... ok
test component::tests::the_components_from_the_design_document_validate ... ok
test domain::tests::an_actor_and_a_command_may_not_share_a_name ... ok
test domain::tests::a_name_declared_in_two_domains_is_refused ... ok
test domain::tests::a_domain_local_type_is_owned_by_that_domain ... ok
test component::tests::a_tree_reading_a_view_from_a_domain_it_does_not_own_is_refused ... ok
test domain::tests::a_member_outside_the_domains_namespace_is_refused ... ok
test domain::tests::a_bare_member_name_means_a_member_of_this_domain ... ok
test domain::tests::the_same_domain_declared_twice_is_refused ... ok
test entity::tests::a_key_the_relation_shape_does_not_have_is_refused_while_the_document_is_read ... ok
test component::tests::an_accepted_command_the_tree_places_nowhere_is_refused ... ok
test component::tests::a_misspelt_command_is_one_fault_and_reports_one_error ... ok
test component::tests::every_reference_that_names_nothing_is_reported_not_just_the_first ... ok
test domain::tests::an_actor_outside_the_domains_namespace_is_refused ... ok
test entity::tests::a_quantifier_over_a_field_the_entity_does_not_have_is_still_refused ... ok
test entity::tests::a_state_with_no_way_out_must_say_it_is_terminal ... ok
test entity::tests::a_quantifier_reads_its_collection_and_never_its_binder ... ok
test entity::tests::a_references_relation_to_many_is_refused_unless_its_field_holds_many ... ok
test entity::tests::a_state_nothing_reaches_is_refused ... ok
test entity::tests::a_move_that_is_not_declared_is_forbidden_by_its_absence ... ok
test entity::tests::a_relation_parses_from_the_shape_the_design_page_writes ... ok
test entity::tests::a_state_whose_only_transition_returns_to_it_is_a_dead_end ... ok
test entity::tests::a_key_the_model_does_not_know_is_refused ... ok
test entity::tests::a_field_that_shadows_the_identity_is_refused ... ok
test entity::tests::a_creation_is_not_read_as_taking_any_move ... ok
test entity::tests::a_creation_reads_its_new_identity_from_an_event_it_emits_and_never_from_the_input ... ok
test entity::tests::a_malformed_transition_name_is_refused_when_the_document_is_read ... ok
test entity::tests::a_state_name_is_one_upper_camel_case_word ... ok
test entity::tests::a_terminal_state_that_is_not_declared_is_refused ... ok
test entity::tests::a_target_no_entity_declares_is_refused_as_an_undeclared_reference ... ok
test entity::tests::a_refusals_subject_does_not_count_as_taking_the_move_it_claims ... ok
test entity::tests::a_transition_no_command_outcome_takes_is_refused_as_uncaused ... ok
test entity::tests::a_transition_to_a_state_that_does_not_exist_is_refused ... ok
test entity::tests::a_well_formed_ownership_is_accepted ... ok
test entity::tests::an_entity_nobody_owns_is_not_refused ... ok
test entity::tests::a_wrong_state_branch_whose_moves_start_everywhere_is_refused_as_unreachable ... ok
test entity::tests::a_transition_from_a_state_that_does_not_exist_is_refused ... ok
test entity::tests::a_via_field_the_carrier_does_not_declare_is_refused_as_a_missing_declaration ... ok
test entity::tests::a_wrong_state_branch_on_a_command_that_moves_nothing_is_refused_as_unreachable ... ok
test entity::tests::an_invariant_may_read_the_lifecycle_as_state ... ok
test entity::tests::a_via_field_typed_as_something_else_is_refused_as_a_type_mismatch ... ok
test entity::tests::an_entity_with_no_states_is_refused ... ok
test entity::tests::an_entity_that_declares_the_same_field_twice_is_refused ... ok
test entity::tests::an_entity_with_no_transitions_needs_no_command_at_all ... ok
test entity::tests::an_entity_two_entities_claim_to_own_is_refused_as_a_conflicting_declaration ... ok
test entity::tests::a_wrong_state_branch_with_a_state_to_be_wrong_in_is_accepted ... ok
test entity::tests::every_transition_with_a_command_that_takes_it_is_accepted ... ok
test entity::tests::an_invariant_that_reads_a_field_the_entity_does_not_have_is_refused ... ok
test entity::tests::an_outcome_that_moves_an_entity_nobody_declares_is_refused_as_a_dangling_reference ... ok
test entity::tests::an_unresolved_identity_or_field_type_is_refused ... ok
test entity::tests::every_undeclared_state_in_one_lifecycle_is_reported_under_the_same_code ... ok
test entity::tests::an_implication_is_written_in_the_structured_form ... ok
test entity::tests::an_instance_named_by_a_field_the_command_does_not_take_is_refused ... ok
test entity::tests::an_entity_round_trips_through_yaml ... ok
test locate::tests::the_same_specification_in_two_organisations_is_two_sets_of_entities ... ok
test entity::tests::an_outcome_that_takes_a_move_the_entity_does_not_declare_is_refused ... ok
test entity::tests::an_initial_state_that_is_not_declared_is_refused ... ok
test entity::tests::quantifying_over_something_that_is_not_a_collection_is_refused ... ok
test entity::tests::an_instance_named_by_a_field_of_the_wrong_type_is_refused ... ok
test entity::tests::transition_legality_is_not_written_as_an_invariant ... ok
test locate::tests::a_declaration_is_addressed_with_the_protocols_own_scheme ... ok
test entity::tests::an_invariant_may_read_the_identity ... ok
test entity::tests::an_invariant_that_misspells_a_nested_field_is_refused ... ok
test entity::tests::two_relations_carried_by_one_field_are_refused_as_a_duplicate_declaration ... ok
test locate::tests::an_empty_organisation_is_refused_rather_than_producing_a_broken_locator ... ok
test locate::tests::an_unknown_kind_is_refused_with_the_known_ones_listed ... ok
test name::tests::a_qualified_name_knows_its_namespace_and_its_own_name ... ok
test name::tests::a_numeric_version_out_of_range_is_refused_rather_than_saturated ... ok
test entity::tests::two_transitions_with_the_same_name_are_refused ... ok
test locate::tests::a_locator_resolves_back_to_what_it_addresses ... ok
test entity::tests::the_billing_invoice_from_the_design_document_validates ... ok
test locate::tests::every_kind_has_a_distinct_spelling_and_a_valid_entity_type ... ok
test entity::tests::the_states_a_command_refuses_in_are_the_ones_its_moves_do_not_start_from ... ok
test spec::tests::a_file_of_actors_with_no_domain_leaves_them_owned_by_nobody_and_is_refused ... ok
test name::tests::naming_defaults_to_the_concepts_own_name ... ok
test refs::tests::a_reference_is_a_provider_and_a_key_and_round_trips ... ok
test name::tests::a_wire_name_can_change_without_the_identity_changing ... ok
test name::tests::the_schema_bounds_the_numeric_version_where_the_parser_does ... ok
test name::tests::versions_start_at_one_and_read_both_ways ... ok
test spec::parse_tests::a_key_written_twice_is_refused_with_the_key_and_the_line ... ok
test name::tests::the_two_spellings_of_a_version_mean_the_same_thing ... ok
test spec::tests::a_domain_summary_is_not_a_system_level_field ... ok
test spec::tests::a_domain_the_header_lists_and_nobody_declares_is_refused ... ok
test name::tests::malformed_names_are_refused ... ok
test spec::parse_tests::a_top_level_key_written_twice_is_refused_too ... ok
test refs::tests::a_provider_that_is_not_a_key_into_a_map_is_refused ... ok
test spec::tests::a_header_that_keeps_no_roster_is_not_checked_against_one ... ok
test name::tests::the_version_pattern_accepts_exactly_what_the_parser_accepts ... ok
test refs::tests::a_url_is_refused_where_a_reference_belongs ... ok
test spec::parse_tests::the_two_stage_parse_reads_what_the_one_stage_parse_read ... ok
test spec::tests::a_fragment_that_sets_system_level_fields_without_naming_the_system_is_refused ... ok
test name::tests::a_version_wider_than_the_model_is_refused_by_the_version_bound ... ok
test spec::tests::a_member_without_a_domain_is_refused_with_what_to_add ... ok
test refs::tests::a_key_may_hold_a_colon_because_only_the_first_one_separates ... ok
test spec::tests::a_header_this_build_cannot_read_no_longer_hides_the_references_under_it ... ok
test spec::tests::a_domain_the_header_does_not_list_is_refused ... ok
test spec::tests::an_actor_inside_no_declared_domain_is_refused ... ok
test spec::tests::the_same_name_declared_in_two_files_names_both_files ... ok
test spec::tests::a_specification_may_be_one_file ... ok
test spec::tests::an_actor_may_not_take_a_name_a_command_already_has ... ok
test system::tests::a_document_in_a_later_format_is_refused_rather_than_guessed_at ... ok
test system::tests::a_name_declared_in_two_domains_is_refused ... ok
test system::tests::a_member_outside_its_domains_namespace_is_refused ... ok
test spec::tests::nothing_can_be_checked_against_a_specification_that_names_no_system ... ok
test system::format_version_tests::the_format_pattern_accepts_exactly_what_the_parser_accepts ... ok
test spec::tests::an_actor_belongs_to_a_domain_like_every_other_member ... ok
test spec::tests::an_errors_payload_is_resolved_against_the_types_the_system_declares ... ok
test system::tests::a_domain_outside_the_system_is_refused ... ok
test system::tests::a_specification_reports_every_problem_in_one_run ... ok
test system::tests::a_domain_that_is_the_system_itself_is_refused ... ok
test spec::tests::an_event_that_records_one_name_twice_is_refused ... ok
test system::tests::a_fragment_that_sets_system_level_fields_without_naming_the_system_is_refused ... ok
test system::tests::an_actor_declared_by_a_domain_is_claimed_like_every_other_member ... ok
test spec::tests::revalidation_rejects_an_expression_mutated_inside_the_sealed_module ... ok
test spec::tests::a_reusable_view_shape_may_carry_its_entity_lifecycle_state ... ok
test system::tests::an_expression_tree_is_not_a_forbidden_cycle ... ok
test system::tests::a_specification_with_no_header_is_refused ... ok
test system::tests::a_type_that_wraps_itself_is_refused_once_and_not_followed ... ok
test system::tests::a_type_declared_by_a_domain_and_by_the_system_has_one_owner ... ok
test system::tests::a_system_is_assembled_from_one_document ... ok
test system::tests::owner_of_answers_for_a_name_in_each_domain_and_for_one_that_is_owned_by_nobody ... ok
test system::tests::a_union_whose_every_variant_recurses_is_refused ... ok
test system::tests::a_wire_name_change_does_not_move_the_system ... ok
test system::tests::a_system_level_type_declared_by_two_sources_is_refused_naming_both ... ok
test system::tests::every_domains_types_land_in_one_registry ... ok
test system::tests::a_format_version_round_trips_and_nothing_else_is_read_as_one ... ok
test system::tests::a_type_that_reaches_itself_only_through_a_base_case_is_allowed ... ok
test system::tests::a_type_reference_that_resolves_to_nothing_is_refused ... ok
test spec::tests::every_problem_is_reported_at_once ... ok
test topology::tests::a_declared_component_with_no_workload_is_not_refused ... ok
test topology::tests::a_workload_indexed_under_a_name_it_does_not_claim_is_refused ... ok
test topology::tests::a_replica_floor_of_zero_is_refused ... ok
test system::tests::two_types_that_cannot_be_built_without_each_other_are_refused ... ok
test topology::tests::an_exact_instance_count_is_not_a_contradiction ... ok
test topology::tests::the_cross_cutting_pass_does_not_repeat_what_conversion_already_reported ... ok
test topology::tests::a_requirement_that_names_nothing_is_refused ... ok
test topology::tests::requirements_are_ordered_by_content_so_the_same_topology_serialises_the_same_way ... ok
test topology::tests::every_workload_is_refused_when_nothing_declares_a_component_at_all ... ok
test topology::tests::the_topology_from_section_eight_is_accepted_against_the_components_it_names ... ok
test types::tests::a_declaration_reaches_the_model_only_through_the_conversion ... ok
test system::tests::two_headers_are_refused_naming_both_sources ... ok
test topology::tests::a_requirement_that_does_not_say_which_instance_is_refused ... ok
test types::tests::a_field_name_must_be_spellable_as_an_identifier ... ok
test types::tests::a_newtype_invariant_reads_the_value_it_wraps ... ok
test types::tests::a_key_the_model_does_not_know_is_refused_in_a_type_declaration ... ok
test types::tests::a_refused_type_is_not_built_so_dropping_it_cannot_overflow_either ... ok
test types::tests::a_required_value_satisfies_an_optional_target_and_not_the_reverse ... ok
test topology::tests::a_specification_that_states_no_topology_is_refused_for_nothing ... ok
test topology::tests::a_requirement_that_does_not_say_what_kind_of_thing_it_is_is_refused ... ok
test topology::tests::a_topology_with_several_problems_reports_all_of_them ... ok
test topology::tests::a_workload_key_that_is_not_a_component_name_is_refused ... ok
test types::tests::a_type_declaration_reads_as_one_object ... ok
test topology::tests::a_stateless_workload_that_needs_two_instances_is_the_ordinary_case ... ok
test topology::tests::a_single_instance_may_hold_state ... ok
test system::tests::the_same_command_declared_by_two_sources_is_refused_naming_both ... ok
test system::tests::two_sources_may_contribute_to_the_same_domain ... ok
test types::tests::a_type_declared_twice_is_refused ... ok
test system::tests::two_partial_specifications_merge_into_one_graph ... ok
test topology::tests::a_workload_naming_a_component_nobody_declared_is_refused ... ok
test topology::tests::a_workload_that_states_nothing_runs_once_and_is_taken_to_hold_no_state ... ok
test topology::tests::a_stateful_workload_that_needs_two_instances_is_refused ... ok
test types::tests::a_type_invariant_that_is_not_a_predicate_is_refused ... ok
test topology::tests::two_pairs_in_one_requirement_are_refused ... ok
test types::tests::a_type_invariant_that_reads_a_field_the_type_does_not_have_is_refused ... ok
test topology::tests::two_workloads_claiming_one_component_are_refused ... ok
test types::tests::a_named_type_is_not_its_representation ... ok
test types::tests::a_structured_map_key_is_refused ... ok
test topology::tests::a_replica_ceiling_below_the_floor_is_refused ... ok
test types::tests::an_unknown_generic_is_refused_rather_than_read_as_a_name ... ok
test types::tests::an_unresolved_type_is_reported_with_what_is_available ... ok
test view::tests::a_filter_reading_a_field_the_source_does_not_have_is_refused ... ok
test view::tests::a_direction_that_is_not_a_direction_is_refused_by_name ... ok
test view::tests::a_filter_may_read_a_field_the_view_does_not_project ... ok
test types::tests::the_deepest_type_a_real_specification_writes_is_still_accepted ... ok
test view::tests::a_parameter_and_the_filter_that_reads_it_must_be_the_same_set ... ok
test view::tests::a_filter_comparing_a_state_to_a_name_the_lifecycle_does_not_have_is_refused ... ok
test types::tests::a_type_nested_past_the_limit_is_refused_rather_than_overflowing_the_stack ... ok
test view::tests::a_filter_on_a_field_whose_type_lists_no_values_is_left_to_the_ir ... ok
test types::tests::dependencies_are_reported_through_composites ... ok
test types::tests::type_references_parse_and_round_trip ... ok
test types::tests::a_union_carries_its_tag ... ok
test types::tests::a_union_that_names_no_tag_field_is_refused ... ok
test types::tests::a_type_no_value_can_be_is_refused ... ok
test view::tests::a_named_struct_supplies_the_projected_fields_and_round_trips_as_a_reference ... ok
test types::tests::a_type_that_declares_something_is_accepted ... ok
test view::tests::a_field_projected_twice_is_refused ... ok
test types::tests::the_published_schema_accepts_what_the_parser_accepts ... ok
test view::tests::ranking_by_one_field_twice_is_refused ... ok
test view::tests::a_view_round_trips_through_yaml ... ok
test view::tests::a_shape_must_name_a_declared_struct ... ok
test view::tests::a_view_says_nothing_about_order_unless_it_declares_one ... ok
test view::tests::a_view_that_does_not_declare_its_consistency_is_eventual ... ok
test view::tests::a_read_your_writes_view_is_asserted_immediately ... ok
test view::tests::every_field_in_a_named_shape_is_checked_against_the_source_entity ... ok
test view::tests::a_view_cannot_declare_both_a_shape_and_inline_fields ... ok
test view::tests::a_view_field_whose_type_disagrees_with_the_entity_is_refused ... ok
test view::tests::a_projected_field_must_name_a_declared_type ... ok
test view::tests::a_view_of_an_entity_that_does_not_exist_is_refused ... ok
test view::tests::a_view_field_the_source_does_not_have_is_refused ... ok
test view::tests::a_projection_makes_a_scenario_assert_eventually_rather_than_immediately ... ok
test view::tests::a_view_that_projects_nothing_is_refused ... ok
test view::tests::a_view_declares_the_order_its_rows_come_back_in ... ok
test view::tests::a_view_may_be_asked_about_one_value_the_caller_supplies ... ok
test view::tests::ranking_by_a_field_the_view_does_not_project_is_refused ... ok
test view::tests::a_view_field_may_widen_a_required_entity_field_to_an_optional_one ... ok
test view::tests::every_branch_of_a_composite_filter_is_checked_against_the_lifecycle ... ok

test result: ok. 334 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running tests/billing.rs (target/debug/deps/billing-58258f0bf30c5289)

running 18 tests
test the_example_states_a_runtime_shape_without_deploying_anything ... ok
test the_example_shows_a_filter_a_payload_and_an_overridden_wire_name ... ok
test every_move_the_invoice_can_make_names_the_command_that_makes_it ... ok
test a_view_may_project_the_identity_and_the_state_as_well_as_the_fields ... ok
test the_example_shows_a_mapping_that_reads_the_event_and_one_that_does_not ... ok
test the_example_declares_the_one_type_crossing_it_needs_and_says_why ... ok
test the_example_declares_a_type_of_every_kind ... ok
test an_outcome_the_input_cannot_decide_says_that_too ... ok
test an_illegal_move_is_illegal_because_nobody_wrote_it ... ok
test the_example_uses_every_primitive_and_every_composite ... ok
test the_billing_specification_parses_and_validates ... ok
test the_example_shows_both_kinds_of_actor_and_both_consistency_levels ... ok
test a_reference_to_something_nobody_declared_is_refused_with_what_was_available ... ok
test the_example_decomposes_into_components_that_own_the_domains ... ok
test a_projection_declares_how_it_must_be_asserted ... ok
test the_example_binds_one_context_to_the_other_and_says_what_happens_when_it_fails ... ok
test the_examples_escalation_names_the_event_that_proves_it_happened ... ok
test a_command_that_can_be_refused_says_so ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/expression.rs (target/debug/deps/expression-3679b4b65e8ada63)

running 9 tests
test parameters_are_typed_at_depth_and_shadowed_lexically ... ok
test recursive_reads_need_progress_but_have_no_projection_depth_limit ... ok
test complete_paths_keep_nominal_types_optionality_and_access_requirements ... ok
test scalar_enum_collection_and_union_selector_errors_name_the_first_bad_segment ... ok
test scalar_operand_contract_is_distinct_from_nominal_assignment_and_satisfiability ... ok
test every_operand_membership_item_and_dead_ast_child_is_checked ... ok
test quantifiers_resolve_targets_before_pushing_lexical_binders ... ok
test enum_literal_checks_apply_through_wrappers_membership_and_bound_paths ... ok
test failed_operands_suppress_cascades_and_keep_stable_independent_errors ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests ess_compiler

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_conformance

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_domain

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

exit: 0
```

Additional verification: rustfmt --edition 2021 --check over the two new Rust test files exited 0; git diff --exit-code exited 0. No further suite or exploratory case was started after the bounded pass. All processes started for the focused and package runs have completed. The pre-existing shared sccache service was left running. Free space measured 30 GiB after the suite, above the 8 GiB floor.

4. Judgement findings

Nothing found. There is no findings table row or unmeasured theory to route. The initial authored NothingHappens failures were fixture setup errors and were corrected without changing their assertions. No origin claim requires an unperformed base run.

5. Boundaries exercised without a counterexample

Public specification assembly and both real compiler entrypoints preserve valid recursive, optional, collection and lexical parameter expressions while refusing the tested malformed expressions.
The public resolved adapter agrees on scalar representation and enum literal direction; conformance retains 026 projection and 035 semantic refusal separation.
Quantifier target-before-binding, nearest shadowing, Map value types, canonical List ordinals and independent diagnostics survived the focused cases.
The full package suite also selected the pre-existing sealed-state fence, module-local revalidation, Integer-versus-0.5 and Decimal0.15 witness controls; no unsafe or impossible public Specification mutation was manufactured.
The supplied canonical-byte comparison was read as implementor/coordinator context; this pass ran the existing deterministic fixture case in the package suite and makes no new historical before/after byte-equivalence claim.

6. Paths outside the assigned worktree

- /home/timo/.cache/sccache — normal shared compilation-cache side effects through the already running authorized cache; cache location confirmed by sccache --show-stats. No cleanup was attempted.
- /home/timo/.cargo/.global-cache — normal shared Cargo cache metadata side effects; the file modification time advanced during the runs. Concurrent builds also use this metadata, so this pass does not attribute every observed byte to itself.
- /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock — connection to the pre-existing listening shared cache socket; no new socket or cache daemon was created by this pass.

Cargo's existing /home/timo/.cargo/.package-cache and /home/timo/.cargo/.package-cache-mutate were used as shared dependency lock files; their observed file modification times remain 2021-07-13 and 2024-08-15. No dependency source downloads or new external scratch/build directory were requested. Every report, raw log, argv/exit record and temporary fixture produced by this pass is inside /home/timo/.local/state/worktree/trees/b10x/ess/review-expression-typechecking/target/review-boundaries-7/adversary-pass-1; compiled artifacts are inside this worktree's own target. No /tmp, Go cache, live service, integration or publication operation was used.

```findings
[]
```
