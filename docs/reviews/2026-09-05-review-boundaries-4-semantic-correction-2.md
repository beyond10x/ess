unit: story:review-semantic-diff-coverage — final bounded F01 dependency correction
verdict: green
cases: executed 505→507, red 5
origin: n/a
wrote-outside-worktree: none
needs-coordinator: yes — final personal verification, integration and publication; no outside-scope patch

## 1. Unit, accepted contract and findings

Make component slices include the views actually served over the network, and make a view slice include its complete named reusable row definition, so changed emitted OpenAPI contracts have changed identity and real regeneration obligations.

Base/unchanged HEAD: 0333681dbbbbf79a380e6f95e21536de0c1166c8. This contains production correction 5c009bfc and the nine second-review tests. The full immutable review2 remains unchanged at 86527 bytes, SHA25697cbcbe685c63349071ee4b91059cfcff7a108c28a939aa9afd6a377467c3d9c. The coordinator supplied carried 0/new 2/resolved 2 and 505 executed (502 pass/3 red); this pass did not rerun a redundant full red baseline. No third full adversary was commissioned.

The ESS binding addendum was updated before production changes to record the coordinator's already recorded Atlas ADR 0036/migration story revision 8 decision. ExposesView includes top-level/grouped CLI views and exactly the network/owned-domain selection of ess-gen::http::routes. RowShape/row-shape links a view to its named reusable TypeHandle definition. The unpublished impact/3 vocabulary now contains 26 relations, preserving the original 21 and the five expressly bound additions. No impact/4, frozen format reinterpretation, persisted IR field or source/whole hash change.

Scope was confirmed against actual owners: compiler graph.rs walks components and views; http.rs routes admits network views from ir.domain(owned_handle).views; OpenAPI schemas explicitly includes view.shape and its full type definition. Components own domains, not entities. The graph cannot depend on ess-gen, so it uses the same resolved selection at its lower compiler layer; a public-route comparison test checks that the two producers agree. The existing graph reachability guard now also compiles the reusable-row fixture and checks ALL's new RowShape is minted. All prior examples/guard assertions remain.

| Finding/class | Concrete repair | Actual result |
|---|---|---|
| Network-served view content is consumed by the component's OpenAPI artifact. | In walk_components, add ExposesView for each view in each owned domain only when reached_by is Network. Keep the existing explicit CLI branch. | Served-view summary mutation changes actual OpenAPI content after removing provenance, changes its profiled contract digest, and produces its actual artifact obligation. The unchanged adversary prints true/true. |
| A reusable row type contributes its complete constraints and metadata independently of copied fields. | In walk_views, add the distinct RowShape edge to resolved.shape when present, preserving Projects/FieldType/ParameterType edges. | The row invariant mutation changes the directly view-seeded digest and the actual OpenAPI digest/obligation. The independent direct seed removes component reachability as a possible explanation. |

Class-boundary coverage: a new test compares exact ExposesView edges with actual HTTP route views for both Billing components under InProcess and Network. It observes zero served views for InProcess and exactly the two invoice views for Network, leaving the email component's unowned invoice views excluded. It checks both forward slice inclusion and reverse closure. The second test checks a sole distinct row-shape relation, absence when the same fields are inline, and retention through before/after graph union after shape removal. Both were executed red before production. Existing grouped/top-level CLI/parameter, ownership carrier/old union, termination/determinism and all-relation guards pass unchanged.

## 2. Actual diff and preserved tests

```text
 crates/specify/ess-compiler/src/graph.rs           |  28 ++++-
 crates/verify/ess-diff/tests/graph.rs              | 122 +++++++++++++++++++++
 ...s-semantic-diff-impact-evolution-design-v0.1.md |  21 +++-
 .../go/gatepass/server/pass-service.openapi.json   |   2 +-
 .../gatepass-server/src/pass-service.openapi.json  |   2 +-
 5 files changed, 166 insertions(+), 9 deletions(-)
```

Complete status:

```text
 M crates/specify/ess-compiler/src/graph.rs
 M crates/verify/ess-diff/tests/graph.rs
 M docs/design/ess-semantic-diff-impact-evolution-design-v0.1.md
 M generated/go/gatepass/server/pass-service.openapi.json
 M generated/rust/gatepass/crates/gatepass-server/src/pass-service.openapi.json
```

Five tracked files changed, no untracked source files: graph production, graph tests, binding design, and the two reserved OpenAPI payloads. Every adversary assertion and frozen fixture was preserved; the final read-only diff check against those paths is empty. No stamp parser, diff comparator, CLI/xtask, root manifest/lock, website, conformance source or sibling edit. git diff --check exit 0; raw empty output and exit are retained as correction2-diff-check.log/.exit.

## 3. Red evidence

Five meaningful red cases = three supplied independent review2 regressions plus two new class-boundary cases. The three supplied raw logs are attributed to the adversary, not claimed as new execution by this implementor. Both new cases ran before production and failed for absent network and row dependencies; no setup/compile failure occurred in this correction. The earlier adversary's import and malformed Ranking fixture mistakes remain in its immutable report and are not included in this red count.

### adversary2-isolated-01b

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff --test review_adversary_f01_pass2 served_view_change_reaches_its_openapi_artifact -- --exact --nocapture
```

Exit 101. Full preserved output:

```text
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 1.33s
     Running tests/review_adversary_f01_pass2.rs (target/debug/deps/review_adversary_f01_pass2-c2309f25ac1af007)

running 1 test
delta: {
  "format": "ess-diff/2",
  "before": {
    "system": "probe",
    "specification_version": 1,
    "spec_digest": "3008ddc5eb033c30a5d6f6c4abaef84590aaeb8393672ac6395358e7392ff0d8"
  },
  "after": {
    "system": "probe",
    "specification_version": 1,
    "spec_digest": "2d875058ae8c82bf51cc644774c86a1d8d252507695ba3c328f3a9e1eeea3c1d"
  },
  "changes": [
    {
      "id": "view/probe.core.Items/summary-changed",
      "relation": "changed",
      "change": {
        "category": "view",
        "subject": "probe.core.Items",
        "changed": {
          "kind": "summary-changed",
          "before": "Original rows.",
          "after": "Revised rows."
        }
      }
    }
  ]
}

OpenAPI contract digest moved: false; OpenAPI owed: false; old=ArtifactDigests { source_digest: "3008ddc5eb033c30a5d6f6c4abaef84590aaeb8393672ac6395358e7392ff0d8", contract_digest: "slice-sha256/2:29640c3025ee6ba6a66fabaf64bf487cc3fb285ca481e940012ac544141c1037" }; new=ArtifactDigests { source_digest: "2d875058ae8c82bf51cc644774c86a1d8d252507695ba3c328f3a9e1eeea3c1d", contract_digest: "slice-sha256/2:29640c3025ee6ba6a66fabaf64bf487cc3fb285ca481e940012ac544141c1037" }

thread 'served_view_change_reaches_its_openapi_artifact' (1627197) panicked at crates/verify/ess-diff/tests/review_adversary_f01_pass2.rs:82:5:
assertion `left == right` failed: an observably changed served contract must not retain a current slice claim
  left: [false, false]
 right: [true, true]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test served_view_change_reaches_its_openapi_artifact ... FAILED

failures:

failures:
    served_view_change_reaches_its_openapi_artifact

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.05s

error: test failed, to rerun pass `-p ess-diff --test review_adversary_f01_pass2`

```

### adversary2-isolated-02

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff --test review_adversary_f01_pass2 reusable_row_invariant_change_reaches_its_openapi_artifact -- --exact --nocapture
```

Exit 101. Full preserved output:

```text
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 1.30s
     Running tests/review_adversary_f01_pass2.rs (target/debug/deps/review_adversary_f01_pass2-c2309f25ac1af007)

running 1 test
delta: {
  "format": "ess-diff/2",
  "before": {
    "system": "probe",
    "specification_version": 1,
    "spec_digest": "3008ddc5eb033c30a5d6f6c4abaef84590aaeb8393672ac6395358e7392ff0d8"
  },
  "after": {
    "system": "probe",
    "specification_version": 1,
    "spec_digest": "47ec6afd1f689995b9cd99f393e538d29aa0b8cd10fd1feeb01c7340e24c1ca6"
  },
  "changes": [
    {
      "id": "type/probe.core.Row/invariants-changed",
      "relation": "changed",
      "change": {
        "category": "type",
        "subject": "probe.core.Row",
        "changed": {
          "kind": "invariants-changed",
          "before": [
            "amount >= 0"
          ],
          "after": [
            "amount > 0"
          ]
        }
      }
    }
  ]
}

OpenAPI contract digest moved: false; OpenAPI owed: false; old=ArtifactDigests { source_digest: "3008ddc5eb033c30a5d6f6c4abaef84590aaeb8393672ac6395358e7392ff0d8", contract_digest: "slice-sha256/2:29640c3025ee6ba6a66fabaf64bf487cc3fb285ca481e940012ac544141c1037" }; new=ArtifactDigests { source_digest: "47ec6afd1f689995b9cd99f393e538d29aa0b8cd10fd1feeb01c7340e24c1ca6", contract_digest: "slice-sha256/2:29640c3025ee6ba6a66fabaf64bf487cc3fb285ca481e940012ac544141c1037" }

thread 'reusable_row_invariant_change_reaches_its_openapi_artifact' (1633433) panicked at crates/verify/ess-diff/tests/review_adversary_f01_pass2.rs:82:5:
assertion `left == right` failed: an observably changed served contract must not retain a current slice claim
  left: [false, false]
 right: [true, true]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test reusable_row_invariant_change_reaches_its_openapi_artifact ... FAILED

failures:

failures:
    reusable_row_invariant_change_reaches_its_openapi_artifact

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.05s

error: test failed, to rerun pass `-p ess-diff --test review_adversary_f01_pass2`

```

### adversary2-isolated-09

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff --test review_adversary_f01_pass2 reusable_row_type_belongs_to_the_view_slice_it_supplies -- --exact --nocapture
```

Exit 101. Full preserved output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/review_adversary_f01_pass2.rs (target/debug/deps/review_adversary_f01_pass2-c2309f25ac1af007)

running 1 test
view old digest=slice-sha256/2:357b13b3c14e6b71896099fbb3825de4de0d8037f65d363534566f3c4c0c9745, new digest=slice-sha256/2:357b13b3c14e6b71896099fbb3825de4de0d8037f65d363534566f3c4c0c9745

thread 'reusable_row_type_belongs_to_the_view_slice_it_supplies' (1638010) panicked at crates/verify/ess-diff/tests/review_adversary_f01_pass2.rs:107:5:
assertion `left != right` failed: a reusable row type's changed invariant belongs to the view's supplied schema contract
  left: "slice-sha256/2:357b13b3c14e6b71896099fbb3825de4de0d8037f65d363534566f3c4c0c9745"
 right: "slice-sha256/2:357b13b3c14e6b71896099fbb3825de4de0d8037f65d363534566f3c4c0c9745"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test reusable_row_type_belongs_to_the_view_slice_it_supplies ... FAILED

failures:

failures:
    reusable_row_type_belongs_to_the_view_slice_it_supplies

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-diff --test review_adversary_f01_pass2`

```

### correction2-boundaries-red

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff --test graph correction2_ -- --nocapture
```

Exit 101. Full preserved output:

```text
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 0.34s
     Running tests/graph.rs (target/debug/deps/graph-428d50882447a7e7)

running 2 tests

thread 'correction2_row_shape_is_a_distinct_dependency_and_survives_graph_union' (1770157) panicked at crates/verify/ess-diff/tests/graph.rs:400:5:
assertion `left == right` failed: the named row is its own dependency
  left: 0
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test correction2_row_shape_is_a_distinct_dependency_and_survives_graph_union ... FAILED

thread 'correction2_network_exposure_matches_actual_routes_and_owned_domains' (1770156) panicked at crates/verify/ess-diff/tests/graph.rs:349:13:
assertion `left == right` failed: invoice-service with Network: only actual served views
  left: {}
 right: {View { name: ViewRef(QualifiedName(billing.invoice.InvoiceById)) }, View { name: ViewRef(QualifiedName(billing.invoice.OutstandingInvoices)) }}
test correction2_network_exposure_matches_actual_routes_and_owned_domains ... FAILED

failures:

failures:
    correction2_network_exposure_matches_actual_routes_and_owned_domains
    correction2_row_shape_is_a_distinct_dependency_and_survives_graph_union

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p ess-diff --test graph`

```

## 4. Green verification and actual counts

The exact full package command from review2 now executes 507 passed, zero failed/ignored across 38 runner summaries. Baseline 505 means its printed 502 passed+3 failed. Only the graph lane received new cases, 8→10; all other execution counts remain unchanged because no cases were added there. Four zero-case doc-test summaries remain zero and are not claimed as runtime evidence.

```text
ess-compiler::lib: executed 19 → 19, exit 0; baseline failed 0, final failed 0
ess-compiler::adversarial.rs: executed 2 → 2, exit 0; baseline failed 0, final failed 0
ess-compiler::billing.rs: executed 14 → 14, exit 0; baseline failed 0, final failed 0
ess-compiler::oracle_fixture.rs: executed 11 → 11, exit 0; baseline failed 0, final failed 0
ess-compiler::sealed_state.rs: executed 3 → 3, exit 0; baseline failed 0, final failed 0
ess-compiler::view_shapes.rs: executed 1 → 1, exit 0; baseline failed 0, final failed 0
ess-diff::lib: executed 9 → 9, exit 0; baseline failed 0, final failed 0
ess-diff::artifacts.rs: executed 13 → 13, exit 0; baseline failed 0, final failed 0
ess-diff::canonical.rs: executed 20 → 20, exit 0; baseline failed 0, final failed 0
ess-diff::families.rs: executed 69 → 69, exit 0; baseline failed 0, final failed 0
ess-diff::graph.rs: executed 8 → 10, exit 0; baseline failed 0, final failed 0
ess-diff::impact.rs: executed 15 → 15, exit 0; baseline failed 0, final failed 0
ess-diff::review_adversary_f01.rs: executed 7 → 7, exit 0; baseline failed 0, final failed 0
ess-diff::review_adversary_f01_pass2.rs: executed 5 → 5, exit 0; baseline failed 3, final failed 0
ess-diff::revision_pair.rs: executed 11 → 11, exit 0; baseline failed 0, final failed 0
ess-gen::lib: executed 55 → 55, exit 0; baseline failed 0, final failed 0
ess-gen::agreement.rs: executed 4 → 4, exit 0; baseline failed 0, final failed 0
ess-gen::asyncapi.rs: executed 18 → 18, exit 0; baseline failed 0, final failed 0
ess-gen::corpus.rs: executed 3 → 3, exit 0; baseline failed 0, final failed 0
ess-gen::determinism.rs: executed 2 → 2, exit 0; baseline failed 0, final failed 0
ess-gen::docs.rs: executed 32 → 32, exit 0; baseline failed 0, final failed 0
ess-gen::openapi.rs: executed 35 → 35, exit 0; baseline failed 0, final failed 0
ess-gen::provenance.rs: executed 18 → 18, exit 0; baseline failed 0, final failed 0
ess-gen::relations.rs: executed 4 → 4, exit 0; baseline failed 0, final failed 0
ess-gen::review_adversary_f01.rs: executed 3 → 3, exit 0; baseline failed 0, final failed 0
ess-gen::review_adversary_f01_pass2.rs: executed 3 → 3, exit 0; baseline failed 0, final failed 0
ess-gen::schema.rs: executed 27 → 27, exit 0; baseline failed 0, final failed 0
ess-synth::lib: executed 8 → 8, exit 0; baseline failed 0, final failed 0
ess-synth::clap.rs: executed 10 → 10, exit 0; baseline failed 0, final failed 0
ess-synth::go.rs: executed 19 → 19, exit 0; baseline failed 0, final failed 0
ess-synth::http.rs: executed 9 → 9, exit 0; baseline failed 0, final failed 0
ess-synth::relations.rs: executed 2 → 2, exit 0; baseline failed 0, final failed 0
ess-synth::synthesis.rs: executed 29 → 29, exit 0; baseline failed 0, final failed 0
ess-synth::web.rs: executed 17 → 17, exit 0; baseline failed 0, final failed 0
Doc-tests ess_compiler: executed 0 → 0, exit 0; baseline failed 0, final failed 0
Doc-tests ess_diff: executed 0 → 0, exit 0; baseline failed 0, final failed 0
Doc-tests ess_gen: executed 0 → 0, exit 0; baseline failed 0, final failed 0
Doc-tests ess_synth: executed 0 → 0, exit 0; baseline failed 0, final failed 0
```

Package totals: compiler 50→50, diff 157→159, gen 204→204, synth 94→94. Targeted mechanism run executed 28 cases across graph and both review generations' diff/gen tests, all green. Regression controls executed 139 cases: families 69, artifacts 13, canonical 20, provenance 18, HTTP 9, clap 10. This includes the previous owner/completeness fixes, complete v4 suite fixture, legacy delta, source/whole/index, normalized-equivalence and actual 14 Rust/Web plus one clap manifest controls. All package checks passed on their first attempt in this correction: scoped formatter application, strict Clippy, complete package suites, final formatter check. No check was weakened or ignored.

### adversary2-package-tests

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-compiler -p ess-diff -p ess-gen -p ess-synth --no-fail-fast
```

Exit 101. Full preserved output:

```text
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-synth)
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 1.36s
     Running unittests src/lib.rs (target/debug/deps/ess_compiler-b4c0bc8f0757838a)

running 19 tests
test graph::tests::a_closure_walks_the_edges_backwards_and_not_forwards ... ok
test graph::tests::a_closure_keeps_the_edges_that_explain_each_construct_it_reached ... ok
test graph::tests::a_command_in_a_slice_brings_its_outcomes_and_what_they_name ... ok
test graph::tests::a_slice_includes_its_seeds_each_with_no_path ... ok
test resolve::tests::a_bridged_refusal_is_located_by_the_declaration_its_path_names ... ok
test resolve::tests::a_needle_that_occurs_twice_is_not_located_because_the_wrong_line_is_worse_than_none ... ok
test resolve::tests::a_declaration_written_once_is_located_at_its_own_line_and_column ... ok
test resolve::tests::a_code_the_bridge_has_no_class_for_still_gets_one ... ok
test resolve::tests::a_needle_in_two_files_is_not_located_because_one_of_them_is_wrong ... ok
test graph::tests::merging_two_graphs_can_only_ever_reach_more ... ok
test graph::tests::the_construct_that_changed_is_in_its_own_closure_with_no_path ... ok
test resolve::tests::a_refusal_is_filed_under_the_layer_its_document_path_names ... ok
test resolve::tests::a_refusal_from_the_domain_crate_keeps_the_code_the_compiler_would_have_given_it ... ok
test resolve::tests::every_code_renders_as_its_family_and_number ... ok
test resolve::tests::every_named_code_is_a_family_paired_with_a_class ... ok
test resolve::tests::the_register_lists_every_code_it_declares ... ok
test resolve::tests::the_second_needle_is_tried_when_the_first_is_ambiguous ... ok
test resolve::tests::with_no_files_named_a_span_still_carries_the_document_path ... ok
test graph::tests::a_slice_reaches_what_a_seed_rests_on_transitively ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/adversarial.rs (target/debug/deps/adversarial-b2f6b2147eadfc2d)

running 2 tests
test the_generator_reaches_both_compilation_and_refusal ... ok
test every_document_is_refused_with_reasons_or_compiled_identically_twice ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/billing.rs (target/debug/deps/billing-127704819ded2aac)

running 14 tests
test a_refusal_from_the_whole_pipeline_carries_a_code_and_the_line_it_belongs_on ... ok
test no_source_file_in_the_compiler_reads_a_clock_or_an_unordered_map ... ok
test every_handle_in_the_ir_names_something_the_ir_holds ... ok
test the_crossing_between_two_contexts_is_recorded_with_the_reason_someone_gave_for_it ... ok
test the_reaction_graph_names_the_binding_that_causes_each_command ... ok
test the_billing_specification_resolves ... ok
test a_binding_that_escalates_carries_the_event_it_emits_as_a_handle ... ok
test every_stable_reference_from_the_compiler_graph_resolves_against_its_ir ... ok
test canonical_json_ends_in_a_newline ... ok
test a_field_keeps_the_shape_of_its_type_rather_than_a_rendering_of_it ... ok
test the_json_orders_its_keys_the_way_a_btreemap_does ... ok
test compiling_without_the_file_list_still_reports_the_document_path ... ok
test the_source_digest_names_exactly_the_canonical_semantic_model ... ok
test compiling_the_billing_example_twice_produces_byte_identical_json ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/oracle_fixture.rs (target/debug/deps/oracle_fixture-58359c9d77095e64)

running 11 tests
test an_outcome_updates_an_entity_without_moving_it_and_that_entity_declares_an_invariant ... ok
test every_on_failure_policy_the_model_has_is_reachable_in_this_fixture ... ok
test the_command_every_binding_invokes_can_be_forced_to_fail ... ok
test the_fixture_compiles_from_the_files_it_lives_in ... ok
test the_eventual_view_converges_on_a_state_the_creating_command_does_not_reach ... ok
test an_illegal_transition_can_be_attempted_from_a_state_a_scenario_can_reach ... ok
test a_binding_maps_an_event_field_that_has_a_same_typed_sibling ... ok
test a_row_reaches_the_read_your_writes_view_after_a_single_command ... ok
test dropping_one_binding_leaves_others_with_scenarios_of_their_own ... ok
test the_fixture_carries_something_the_normative_example_does_not ... ok
test every_input_the_oracle_needs_is_carried_by_one_of_the_examples ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/sealed_state.rs (target/debug/deps/sealed_state-fe1b4fe83ef9373a)

running 3 tests
test validated_and_resolved_state_have_no_public_fields ... ok
test provenance_never_hashes_an_empty_serialization_fallback ... ok
test every_compiler_entrance_validates_before_resolution ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/view_shapes.rs (target/debug/deps/view_shapes-6be70769d1254075)

running 1 test
test a_shape_is_one_handle_with_checked_fields_in_every_view ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/ess_diff-6c940e1b4fa9bdeb)

running 9 tests
test change::tests::a_change_id_names_its_category_subject_subtype_and_member_in_that_order ... ok
test change::tests::a_change_with_no_member_renders_three_parts_rather_than_a_trailing_slash ... ok
test impact::tests::a_whole_answer_absorbs_a_narrowing_whichever_way_round_they_are_joined ... ok
test impact::tests::a_suite_resting_on_a_construct_the_graph_has_no_node_for_owes_the_whole_suite ... ok
test change::tests::only_a_grant_and_a_variant_decide_a_direction ... ok
test impact::tests::a_change_to_the_specification_itself_owes_the_whole_suite ... ok
test delta::tests::a_delta_puts_its_changes_in_canonical_order_however_they_arrive ... ok
test impact::tests::an_unfollowed_file_is_not_an_artifact_that_owes_regeneration ... ok
test change::tests::the_canonical_order_is_the_category_order_and_not_the_alphabet ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/artifacts.rs (target/debug/deps/artifacts-9f64f03ab905b12c)

running 13 tests
test an_owed_artifacts_path_explains_the_membership_hop_by_hop ... ok
test the_artifacts_the_currency_changes_reach_are_owed_and_named ... ok
test an_artifact_whose_slice_nothing_reached_is_absent_from_the_answer ... ok
test a_grant_change_owes_the_documents_that_read_grants_and_not_the_ones_that_do_not ... ok
test a_change_to_the_system_header_owes_every_artifact ... ok
test the_two_predicate_edits_narrow_the_artifacts_differently_and_both_subsets_are_named ... ok
test whole_model_artifacts_are_owed_by_any_change_at_all ... ok
test the_six_change_delta_owes_a_strict_subset_of_the_artifacts ... ok
test a_committed_tree_is_answered_for_fail_closed_file_by_file ... ok
test review_whole_model_hashes_and_index_bytes_remain_frozen ... ok
test the_artifact_answer_is_byte_identical_between_runs ... ok
test a_committed_artifact_with_a_false_contract_digest_is_owed_as_a_false_claim ... ok
test review_legacy_slice_stamps_are_owed_even_when_raw_hashes_match ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s

     Running tests/canonical.rs (target/debug/deps/canonical-d218f8527473a680)

running 20 tests
test a_binding_still_has_one_delivery_a_document_can_write ... ok
test every_change_variant_has_something_to_say_for_itself ... ok
test a_change_is_spelt_the_same_way_in_its_id_and_in_the_document ... ok
test review_freeze_legacy_delta_bytes ... ok
test review_version_admission_refuses_new_vocabulary_in_legacy_envelopes ... ok
test no_source_file_in_the_diff_engine_reads_a_clock_or_an_unordered_map ... ok
test no_source_file_in_the_diff_engine_calls_an_ir_handle_accessor ... ok
test a_system_still_has_no_naming_a_document_can_set ... ok
test a_delta_this_build_wrote_is_read_back_without_complaint ... ok
test canonical_json_ends_in_a_newline ... ok
test a_document_with_six_defects_reports_six ... ok
test the_changes_are_written_in_the_category_order_and_not_the_alphabet ... ok
test a_delta_whose_id_was_edited_is_refused ... ok
test a_delta_whose_relation_was_edited_is_refused ... ok
test a_delta_written_in_a_format_this_build_does_not_read_is_refused ... ok
test every_change_in_a_delta_has_its_own_id ... ok
test a_delta_naming_two_systems_is_refused_on_the_way_in_as_well ... ok
test review_new_default_delta_format_is_version_two ... ok
test a_delta_whose_changes_are_out_of_order_is_refused ... ok
test diffing_the_same_pair_twice_produces_byte_identical_json ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/families.rs (target/debug/deps/families-5b96283fbb857370)

running 69 tests
test a_component_that_no_longer_publishes_an_event_is_reported ... ok
test a_bindings_naming_is_compared_key_by_key ... ok
test a_binding_reacting_to_a_different_event_is_reported ... ok
test a_filter_respaced_is_the_same_predicate_and_no_change ... ok
test a_binding_invoking_a_different_command_moves_its_mapping_with_it ... ok
test a_filter_removed_reads_as_containing_every_instance ... ok
test a_commands_naming_is_compared_key_by_key ... ok
test a_new_transition_arrives_with_the_outcome_that_takes_it ... ok
test a_command_added_is_one_change ... ok
test a_component_accepting_a_new_command_is_changed_and_not_widened ... ok
test a_guard_respaced_is_the_same_predicate_and_no_change ... ok
test a_type_that_became_a_different_kind_of_thing_is_reported_as_that_and_nothing_else ... ok
test a_bindings_failure_policy_is_compared ... ok
test a_mapping_filled_from_somewhere_else_is_reported_with_both_sources ... ok
test a_binding_added_is_one_change ... ok
test a_newtype_that_wraps_something_else_is_reported ... ok
test a_filter_that_contains_different_instances_is_changed_with_no_direction ... ok
test a_construct_moving_between_files_is_not_a_change ... ok
test a_struct_field_that_changed_type_is_reported ... ok
test a_payload_declaration_arriving_is_a_payload_change ... ok
test a_types_own_invariants_are_reported_as_different_and_never_as_stronger ... ok
test a_union_gaining_a_variant_widens_it_just_as_an_enum_does ... ok
test an_events_wire_name_moving_is_not_the_event_moving ... ok
test a_union_that_is_tagged_by_another_field_is_reported ... ok
test a_view_exposing_a_new_field_is_reported_with_the_type_it_carries ... ok
test an_entity_field_that_changed_type_is_reported ... ok
test a_views_consistency_promise_is_compared_and_not_classified ... ok
test an_entity_field_replaced_is_removed_and_added_and_never_a_rename ... ok
test an_actor_declared_with_no_grants_at_all_is_still_a_change_to_report ... ok
test an_entity_added_arrives_with_its_synthesised_state_enum_and_nothing_is_diffed_inside ... ok
test an_error_that_gained_a_field_is_reported_with_the_type_it_carries ... ok
test a_views_naming_is_compared_key_by_key ... ok
test a_union_variant_that_carries_something_else_is_not_a_variant_removed_and_added ... ok
test an_entitys_naming_is_compared_key_by_key ... ok
test an_entity_fields_naming_is_compared_key_by_key ... ok
test a_view_fields_naming_is_compared_key_by_key ... ok
test a_view_added_is_one_change ... ok
test an_identitys_display_name_and_summary_are_compared ... ok
test a_view_projecting_a_different_entity_is_a_source_change ... ok
test an_event_field_that_changed_type_is_reported ... ok
test an_input_added_is_reported_with_the_type_it_carries ... ok
test an_outcome_added_is_one_change_and_claims_no_direction ... ok
test an_input_that_changed_type_is_reported ... ok
test an_event_renamed_is_reported_as_removed_and_added_and_never_as_a_rename ... ok
test reordering_a_commands_input_is_reported_once ... ok
test reordering_an_enums_variants_is_reported_without_claiming_a_direction ... ok
test renaming_an_entitys_identity_is_the_one_rename_this_crate_reports ... ok
test an_outcomes_summary_is_compared ... ok
test reordering_an_event_payload_is_reported_once_and_not_as_a_field_change ... ok
test reordering_an_entitys_fields_is_reported_once ... ok
test reordering_a_views_fields_is_reported_once ... ok
test reordering_a_commands_outcomes_is_a_real_change ... ok
test an_input_fields_naming_is_compared_key_by_key ... ok
test an_invariant_statement_reworded_without_moving_the_predicate_is_still_a_change ... ok
test the_specifications_version_moving_is_reported_and_is_not_the_identity ... ok
test what_an_outcome_emits_is_compared_in_order ... ok
test writing_out_a_naming_default_is_not_a_change ... ok
test the_paragraph_saying_what_the_system_is_is_compared ... ok
test what_an_error_tells_the_caller_is_compared ... ok
test the_error_a_branch_reports_is_compared ... ok
test review_outcome_refusal_is_independent_of_its_error ... ok
test review_reach_is_a_change_without_an_unrelated_surface_edit ... ok
test review_view_parameter_naming_is_compared_without_a_filter_edit ... ok
test review_cli_top_level_grouped_views_and_binary_are_changes ... ok
test review_view_ranking_is_compared_without_a_filter_edit ... ok
test review_outcome_sets_are_independent_of_event_payload ... ok
test review_residual_refs_cannot_hide_beside_a_classified_change ... ok
test review_unclassified_transition_order_cannot_hide_beside_a_classified_edit ... ok
test review_relation_cardinality_name_and_removal_are_changes ... ok

test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s

     Running tests/graph.rs (target/debug/deps/graph-b94d34c7c7fe4967)

running 8 tests
test the_graph_records_the_reference_an_author_wrote_and_not_its_reverse ... ok
test a_component_is_reached_through_what_it_accepts_and_publishes ... ok
test a_type_is_reached_through_the_declarations_that_hold_it_and_not_by_name ... ok
test a_closure_over_the_whole_model_terminates_and_stays_inside_it ... ok
test building_the_same_graph_twice_produces_the_same_edges_in_the_same_order ... ok
test review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union ... ok
test review_cli_views_and_parameter_types_are_forward_slice_dependencies ... ok
test every_relation_in_the_vocabulary_is_minted_by_a_specification_this_repository_ships ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/impact.rs (target/debug/deps/impact-130b9b43fe12ce7e)

running 15 tests
test a_suite_produced_from_the_later_revision_is_refused_rather_than_narrowed ... ok
test a_suite_whose_contract_digest_its_model_does_not_compute_is_refused ... ok
test two_specifications_of_different_systems_are_refused_here_too ... ok
test a_suite_for_another_system_is_refused ... ok
test a_suite_resting_on_a_construct_no_graph_has_a_node_for_owes_the_whole_suite ... ok
test an_edited_entity_invariant_owes_every_scenario_that_rests_on_the_entity_and_no_other ... ok
test an_edited_outcome_guard_owes_every_scenario_because_every_scenario_creates_through_it ... ok
test a_variant_removed_from_an_enum_reaches_the_entity_that_holds_it_transitively ... ok
test taking_a_grant_from_an_actor_owes_only_the_scenarios_that_act_as_that_actor ... ok
test the_suite_the_fixture_obliges_is_ten_scenarios_and_the_delta_is_six_changes ... ok
test every_scenario_resting_directly_on_a_changed_construct_is_owed_again ... ok
test a_narrowed_answer_never_reports_more_scenarios_than_the_suite_holds ... ok
test analysing_the_same_pair_twice_produces_byte_identical_json ... ok
test a_change_in_a_family_the_delta_still_does_not_compare_owes_the_whole_suite ... ok
test a_domains_naming_moving_owes_the_whole_suite_because_no_family_compares_a_domain ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

     Running tests/review_adversary_f01.rs (target/debug/deps/review_adversary_f01-8795d8f802dd31cc)

running 7 tests
test explicit_domain_naming_defaults_remain_semantically_equivalent ... ok
test complete_generated_and_authored_suite_four_bytes_remain_frozen ... ok
test relation_delta_versions_refuse_relabeling_and_public_serialize_bypasses ... ok
test moved_outcome_reference_retains_its_owner ... ok
test moved_outcome_reference_is_independent_of_a_classified_edit ... ok
test ownership_cardinality_invalidates_both_emitted_schema_ends ... ok
test incomplete_schema_stamp_is_owed_by_the_real_impact_reader ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s

     Running tests/review_adversary_f01_pass2.rs (target/debug/deps/review_adversary_f01_pass2-1ffc276d4342fd42)

running 5 tests
test reusable_row_type_belongs_to_the_view_slice_it_supplies ... FAILED
test ranking_precedence_survives_the_checked_delta_roundtrip ... ok
test switching_equal_row_shapes_retains_independent_residual_coverage ... ok
test served_view_change_reaches_its_openapi_artifact ... FAILED
test reusable_row_invariant_change_reaches_its_openapi_artifact ... FAILED

failures:

---- reusable_row_type_belongs_to_the_view_slice_it_supplies stdout ----
view old digest=slice-sha256/2:357b13b3c14e6b71896099fbb3825de4de0d8037f65d363534566f3c4c0c9745, new digest=slice-sha256/2:357b13b3c14e6b71896099fbb3825de4de0d8037f65d363534566f3c4c0c9745

thread 'reusable_row_type_belongs_to_the_view_slice_it_supplies' (1654633) panicked at crates/verify/ess-diff/tests/review_adversary_f01_pass2.rs:149:5:
assertion `left != right` failed: a reusable row type's changed invariant belongs to the view's supplied schema contract
  left: "slice-sha256/2:357b13b3c14e6b71896099fbb3825de4de0d8037f65d363534566f3c4c0c9745"
 right: "slice-sha256/2:357b13b3c14e6b71896099fbb3825de4de0d8037f65d363534566f3c4c0c9745"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- served_view_change_reaches_its_openapi_artifact stdout ----
delta: {
  "format": "ess-diff/2",
  "before": {
    "system": "probe",
    "specification_version": 1,
    "spec_digest": "3008ddc5eb033c30a5d6f6c4abaef84590aaeb8393672ac6395358e7392ff0d8"
  },
  "after": {
    "system": "probe",
    "specification_version": 1,
    "spec_digest": "2d875058ae8c82bf51cc644774c86a1d8d252507695ba3c328f3a9e1eeea3c1d"
  },
  "changes": [
    {
      "id": "view/probe.core.Items/summary-changed",
      "relation": "changed",
      "change": {
        "category": "view",
        "subject": "probe.core.Items",
        "changed": {
          "kind": "summary-changed",
          "before": "Original rows.",
          "after": "Revised rows."
        }
      }
    }
  ]
}

OpenAPI contract digest moved: false; OpenAPI owed: false; old=ArtifactDigests { source_digest: "3008ddc5eb033c30a5d6f6c4abaef84590aaeb8393672ac6395358e7392ff0d8", contract_digest: "slice-sha256/2:29640c3025ee6ba6a66fabaf64bf487cc3fb285ca481e940012ac544141c1037" }; new=ArtifactDigests { source_digest: "2d875058ae8c82bf51cc644774c86a1d8d252507695ba3c328f3a9e1eeea3c1d", contract_digest: "slice-sha256/2:29640c3025ee6ba6a66fabaf64bf487cc3fb285ca481e940012ac544141c1037" }

thread 'served_view_change_reaches_its_openapi_artifact' (1654634) panicked at crates/verify/ess-diff/tests/review_adversary_f01_pass2.rs:107:5:
assertion `left == right` failed: an observably changed served contract must not retain a current slice claim
  left: [false, false]
 right: [true, true]

---- reusable_row_invariant_change_reaches_its_openapi_artifact stdout ----
delta: {
  "format": "ess-diff/2",
  "before": {
    "system": "probe",
    "specification_version": 1,
    "spec_digest": "3008ddc5eb033c30a5d6f6c4abaef84590aaeb8393672ac6395358e7392ff0d8"
  },
  "after": {
    "system": "probe",
    "specification_version": 1,
    "spec_digest": "47ec6afd1f689995b9cd99f393e538d29aa0b8cd10fd1feeb01c7340e24c1ca6"
  },
  "changes": [
    {
      "id": "type/probe.core.Row/invariants-changed",
      "relation": "changed",
      "change": {
        "category": "type",
        "subject": "probe.core.Row",
        "changed": {
          "kind": "invariants-changed",
          "before": [
            "amount >= 0"
          ],
          "after": [
            "amount > 0"
          ]
        }
      }
    }
  ]
}

OpenAPI contract digest moved: false; OpenAPI owed: false; old=ArtifactDigests { source_digest: "3008ddc5eb033c30a5d6f6c4abaef84590aaeb8393672ac6395358e7392ff0d8", contract_digest: "slice-sha256/2:29640c3025ee6ba6a66fabaf64bf487cc3fb285ca481e940012ac544141c1037" }; new=ArtifactDigests { source_digest: "47ec6afd1f689995b9cd99f393e538d29aa0b8cd10fd1feeb01c7340e24c1ca6", contract_digest: "slice-sha256/2:29640c3025ee6ba6a66fabaf64bf487cc3fb285ca481e940012ac544141c1037" }

thread 'reusable_row_invariant_change_reaches_its_openapi_artifact' (1654632) panicked at crates/verify/ess-diff/tests/review_adversary_f01_pass2.rs:107:5:
assertion `left == right` failed: an observably changed served contract must not retain a current slice claim
  left: [false, false]
 right: [true, true]


failures:
    reusable_row_invariant_change_reaches_its_openapi_artifact
    reusable_row_type_belongs_to_the_view_slice_it_supplies
    served_view_change_reaches_its_openapi_artifact

test result: FAILED. 2 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

error: test failed, to rerun pass `-p ess-diff --test review_adversary_f01_pass2`
     Running tests/revision_pair.rs (target/debug/deps/revision_pair-6384e25d4d698c13)

running 11 tests
test two_different_systems_are_refused_rather_than_reported_as_a_rewrite ... ok
test a_revision_compared_with_itself_reports_nothing ... ok
test rewriting_an_entitys_invariant_is_changed_and_quotes_both_statements ... ok
test removing_an_enum_variant_narrows_the_type_that_accepted_it ... ok
test taking_a_command_from_an_actor_narrows_what_the_system_permits ... ok
test adding_an_enum_variant_widens_the_type_that_accepts_it ... ok
test nothing_the_after_revision_only_rewrote_reaches_the_delta ... ok
test rewriting_an_outcomes_when_is_changed_and_renders_both_guards_canonically ... ok
test the_delta_survives_being_written_and_read_back ... ok
test granting_a_command_to_an_actor_widens_what_the_system_permits ... ok
test the_fixture_pair_differs_by_exactly_six_changes ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/lib.rs (target/debug/deps/ess_gen-9f7cfaccfa26d347)

running 55 tests
test artifact::tests::portable_artifacts_refuse_escape_and_platform_aliases ... ok
test artifact::tests::a_destination_set_rejects_duplicates_case_aliases_and_file_parents_in_any_order ... ok
test authored::tests::a_heading_becomes_a_section_with_an_anchor ... ok
test authored::tests::a_fence_keeps_its_language_and_loses_its_trailing_newline ... ok
test authored::tests::a_link_an_adopter_wrote_stays_theirs ... ok
test authored::tests::a_list_becomes_items_and_a_quote_becomes_a_quote ... ok
test authored::tests::a_paragraph_keeps_its_inline_structure ... ok
test authored::tests::a_table_keeps_its_header_apart_from_its_rows ... ok
test authored::tests::a_top_level_heading_is_demoted_because_the_page_title_is_the_first ... ok
test authored::tests::a_leading_title_becomes_the_page_title_and_not_a_second_heading ... ok
test authored::tests::raw_html_is_dropped_rather_than_passed_through ... ok
test docs::tests::a_gap_that_ships_says_which_crate_closes_it ... ok
test docs::tests::a_heading_and_its_anchor_agree ... ok
test docs::tests::a_lifecycle_that_connects_every_pair_says_it_forbids_nothing ... ok
test docs::tests::a_lifecycle_renders_as_a_state_diagram_with_its_initial_and_terminal_states_marked ... ok
test docs::tests::a_list_of_three_reads_as_a_person_would_write_it ... ok
test docs::tests::a_lifecycle_with_one_state_forbids_nothing_rather_than_forbidding_everything ... ok
test docs::tests::a_plural_of_entity_is_entities ... ok
test docs::tests::a_state_no_transition_touches_is_still_drawn ... ok
test document::tests::a_link_names_what_it_points_at_and_never_a_path ... ok
test graph::tests::a_component_group_is_a_dot_cluster_and_graphviz_only_boxes_clusters ... ok
test docs::tests::a_transition_from_two_states_draws_one_arrow_from_each ... ok
test graph::tests::a_dot_label_keeps_its_parts_on_separate_lines ... ok
test graph::tests::a_mermaid_label_cannot_close_the_quoted_string_it_sits_in ... ok
test document::tests::a_page_id_says_how_deep_it_is_so_a_renderer_can_reach_the_root ... ok
test docs::tests::the_page_names_every_transition_the_specification_does_not_permit ... ok
test html::tests::a_code_block_is_a_code_listing_and_carries_its_language ... ok
test document::tests::a_document_round_trips_through_its_own_format ... ok
test html::tests::a_construct_is_addressed_by_the_section_that_documents_it ... ok
test html::tests::a_diagram_is_a_pre_the_renderer_draws_into_and_never_a_code_listing ... ok
test html::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test html::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test html::tests::a_table_is_a_table_with_a_head_and_a_body ... ok
test html::tests::a_page_reaches_its_stylesheet_and_its_renderer_from_wherever_it_sits ... ok
test html::tests::an_adopters_front_page_goes_above_the_index_and_nowhere_else ... ok
test html::tests::markup_in_text_never_reaches_the_browser_as_markup ... ok
test html::tests::the_sidebar_groups_the_nested_pages_and_marks_the_page_the_reader_is_on ... ok
test markdown::tests::a_diagram_is_a_fenced_mermaid_block ... ok
test markdown::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test markdown::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test markdown::tests::a_quotation_marks_every_line_it_covers ... ok
test markdown::tests::a_section_flattens_into_the_stream_and_its_children_follow_it ... ok
test markdown::tests::a_table_is_written_with_the_separator_a_reader_expects ... ok
test schema::types::tests::a_decimal_is_written_as_an_exact_string_because_a_json_number_is_read_as_a_float ... ok
test schema::types::tests::a_reference_is_a_pointer_into_the_defs_of_the_document_holding_it ... ok
test schema::types::tests::a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test schema::types::tests::a_string_keyed_map_publishes_no_property_name_rule_that_checks_nothing ... ok
test schema::types::tests::a_union_branch_pins_its_tag_so_exactly_one_branch_can_match ... ok
test schema::types::tests::a_union_tagged_value_moves_its_payload_aside_rather_than_colliding_with_the_tag ... ok
test schema::types::tests::an_optional_outside_a_field_gains_a_null_branch_because_a_list_element_cannot_be_absent ... ok
test schema::types::tests::an_integer_key_is_constrained_to_the_text_an_integer_is_spelt_with ... ok
test html::tests::the_default_style_is_the_stylesheet_that_is_published ... ok
test html::tests::every_emitted_file_says_what_it_was_generated_from ... ok
test html::tests::checked_rendering_validates_deserialized_page_identities_before_map_collection ... ok
test html::tests::checked_rendering_preserves_valid_parent_and_nested_page_bytes ... ok

test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/agreement.rs (target/debug/deps/agreement-4af69d28643a528e)

running 4 tests
test the_agreement_check_compares_the_constructs_the_defect_was_about_rather_than_nothing ... ok
test every_keyword_the_projections_publish_is_classified_as_an_assertion_or_an_annotation ... ok
test no_projection_collapses_a_newtype_into_the_representation_it_wraps ... ok
test every_projection_publishes_the_same_schema_for_a_construct_more_than_one_of_them_describes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/asyncapi.rs (target/debug/deps/asyncapi-f449a9e3ff824c48)

running 18 tests
test a_payload_refuses_an_undeclared_field_and_spells_absence_by_leaving_it_out_of_required ... ok
test a_union_pins_its_tag_so_exactly_one_branch_matches_rather_than_none_or_both ... ok
test a_binding_no_component_handles_still_states_its_failure_policy ... ok
test a_collection_says_what_it_holds_and_an_absent_element_is_null_because_it_has_no_key_to_omit ... ok
test a_dropped_failure_is_stated_in_prose_and_not_only_in_an_extension ... ok
test a_payload_field_carries_the_grammar_the_model_states_and_not_a_note_naming_it ... ok
test every_event_in_the_billing_example_appears_in_some_document ... ok
test the_publisher_of_an_event_sees_who_reacts_to_it_and_under_what_failure_policy ... ok
test the_channel_and_its_message_say_nothing_about_the_binding ... ok
test a_bindings_mapping_and_the_reason_for_its_type_crossing_reach_the_document ... ok
test a_bindings_delivery_and_failure_reach_the_receiving_operation ... ok
test every_document_carries_the_provenance_of_the_model_it_came_from ... ok
test an_events_channel_address_is_its_declared_wire_name_or_else_its_qualified_name ... ok
test a_document_is_a_valid_asyncapi_three_skeleton ... ok
test every_ref_resolves_inside_the_document_that_holds_it ... ok
test every_component_gets_one_document_named_after_it ... ok
test a_document_shows_what_the_component_publishes_and_what_it_reacts_to ... ok
test regenerating_from_the_same_model_produces_the_same_bytes ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/corpus.rs (target/debug/deps/corpus-b5c508be613e316e)

running 3 tests
test the_gatepass_documentation_is_byte_for_byte_what_is_pinned ... ok
test the_oracle_fixture_documentation_is_byte_for_byte_what_is_pinned ... ok
test the_billing_documentation_is_byte_for_byte_what_is_pinned ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/determinism.rs (target/debug/deps/determinism-82fd72ce337b58c9)

running 2 tests
test the_determinism_scan_sees_code_and_not_prose ... ok
test no_generator_reads_a_clock_or_an_unordered_map ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/docs.rs (target/debug/deps/docs-9d17c089cbb09f3a)

running 32 tests
test a_type_nothing_references_is_flagged_rather_than_left_looking_used ... ok
test a_grant_that_crosses_two_contexts_links_to_the_other_contexts_page ... ok
test an_entitys_lifecycle_transitions_reach_the_page_as_arrows ... ok
test checked_site_preserves_valid_deserialized_nested_pages_and_every_artifact_byte ... ok
test a_views_filter_reaches_the_page_rather_than_being_silently_dropped ... ok
test a_declared_conversion_carries_its_reason_everywhere_a_reader_might_start ... ok
test an_entitys_identity_reaches_the_page_by_name_and_not_only_by_type ... ok
test an_entitys_absent_transition_is_named_as_a_move_the_specification_does_not_permit ... ok
test a_bindings_delivery_and_failure_semantics_are_stated_in_words ... ok
test an_actors_grant_renders_as_an_edge_from_the_actor_to_that_command_in_the_index_graph ... ok
test a_binding_renders_as_a_flow_and_a_lifecycle_as_a_state_diagram ... ok
test an_actor_that_may_invoke_nothing_is_still_on_the_page ... ok
test a_type_reached_only_through_an_entitys_field_is_not_called_unreached ... ok
test an_entitys_invariant_reaches_the_page_as_a_condition_on_every_instance ... ok
test an_empty_gap_allowlist_puts_no_cannot_show_section_on_any_page ... ok
test a_commands_refusal_branch_is_documented_and_not_only_its_name ... ok
test a_wrong_state_branch_is_documented_with_the_states_the_document_never_lists ... ok
test a_views_eventual_consistency_reads_differently_from_an_immediate_one ... ok
test checked_site_rejects_deserialized_collisions_with_late_static_assets ... ok
test an_outcome_the_input_cannot_decide_says_so_rather_than_claiming_it_is_unreachable ... ok
test an_outcome_that_changes_an_entity_says_which_instance_and_where_the_identity_is_read ... ok
test a_components_ownership_and_a_workloads_replica_floor_are_both_documented ... ok
test every_link_between_pages_lands_on_a_page_that_exists_at_the_heading_it_names ... ok
test every_type_kind_reaches_a_page_including_the_tagged_union ... ok
test every_member_of_a_resolved_domain_reaches_the_page_of_the_context_it_belongs_to ... ok
test every_page_says_which_specification_produced_it ... ok
test the_provenance_header_is_a_markdown_comment_a_renderer_can_close ... ok
test the_command_that_takes_each_move_reaches_the_page_beside_the_move_itself ... ok
test an_events_payload_and_an_errors_payload_are_both_documented_field_by_field ... ok
test every_name_the_ir_holds_appears_on_some_page ... ok
test generating_the_documentation_twice_produces_byte_identical_output ... ok
test an_outcome_says_what_it_does_to_an_entity_and_a_refusal_says_it_changes_none ... ok

test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/openapi.rs (target/debug/deps/openapi-84c5c21bb15c13da)

running 35 tests
test a_component_that_accepts_nothing_still_gets_a_document ... ok
test a_command_with_no_input_is_exposed_without_a_body ... ok
test a_command_no_component_accepts_appears_in_no_document ... ok
test a_map_with_a_non_string_key_says_the_key_is_still_a_string ... ok
test a_served_view_declares_its_rows_and_the_consistency_a_caller_gets ... ok
test a_view_is_served_only_where_the_specification_says_something_outside_reads_it ... ok
test every_kind_of_type_the_model_has_projects_into_a_schema ... ok
test a_command_names_the_actors_permitted_to_invoke_it_and_no_authentication_mechanism ... ok
test a_decimal_is_a_string_because_a_json_number_is_a_float ... ok
test a_refusal_the_subjects_state_decides_is_a_conflict_and_not_a_bad_request ... ok
test a_command_is_only_ever_a_post ... ok
test a_command_a_binding_delivers_at_least_once_requires_an_idempotency_key ... ok
test every_component_gets_one_document_named_after_it ... ok
test a_command_no_binding_invokes_carries_no_idempotency_header ... ok
test a_refusal_the_input_decides_carries_the_declared_error_payload ... ok
test a_commands_input_becomes_a_closed_object_over_its_declared_fields ... ok
test a_newtype_stays_a_schema_of_its_own_rather_than_becoming_its_representation ... ok
test a_document_is_valid_yaml_with_a_version_an_info_block_and_paths ... ok
test a_command_no_actor_names_carries_no_grant_rather_than_a_grant_to_everybody ... ok
test several_outcomes_on_one_status_stay_distinguishable ... ok
test an_outcome_that_emits_says_so_without_claiming_to_return_the_events ... ok
test a_command_with_no_wire_name_is_exposed_under_the_name_the_model_gives_it ... ok
test a_command_is_exposed_at_its_wire_name_under_its_domains ... ok
test two_commands_claiming_one_path_both_move_to_their_qualified_names ... ok
test every_document_carries_its_provenance_as_a_comment_and_as_data ... ok
test an_external_outcome_is_an_upstream_failure_and_not_a_validation_refusal ... ok
test each_declared_outcome_is_its_own_response_and_no_status_is_invented ... ok
test the_operation_id_is_the_commands_qualified_name ... ok
test every_document_this_generator_can_produce_is_a_valid_openapi_document ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_schema_the_document_declares_is_pointed_at_by_something ... ok
test regenerating_from_the_same_ir_produces_the_same_bytes ... ok
test the_entities_published_are_exactly_those_of_the_domains_the_component_owns ... ok
test every_schema_a_document_embeds_is_valid_in_the_dialect_openapi_31_declares ... ok
test the_document_a_server_hands_out_is_the_committed_one_in_the_other_dialect ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/provenance.rs (target/debug/deps/provenance-97bf54f63a94402c)

running 18 tests
test a_damaged_digest_reads_as_nothing ... ok
test a_text_without_both_digests_reads_as_nothing ... ok
test a_whole_model_slice_is_stamped_as_one ... ok
test a_generator_that_stamps_nothing_cannot_ship_an_artifact - should panic ... ok
test review_new_reader_refuses_unsupported_profile_without_legacy_fallback ... ok
test review_new_reader_requires_envelopes_and_exact_digest_tokens ... ok
test a_generator_that_pairs_a_stamp_with_the_wrong_slice_cannot_ship_an_artifact - should panic ... ok
test the_reader_reads_back_every_form_the_writer_emits ... ok
test review_every_constructs_digest_has_an_explicit_profile_and_whole_remains_bare ... ok
test the_whole_model_contract_digest_is_not_the_source_digest ... ok
test a_change_no_construct_can_be_named_for_moves_every_contract_digest ... ok
test a_change_outside_an_artifacts_slice_leaves_its_contract_digest_standing ... ok
test review_docs_ir_retains_page_profiles_and_does_not_claim_a_flat_stamp ... ok
test review_profile_is_read_in_all_emissions_and_old_reader_refuses_ordinary_slices ... ok
test review_conflicting_structured_and_comment_stamps_are_unreadable ... ok
test correction_structured_envelopes_require_typed_attribution ... ok
test review_marker_looking_model_content_does_not_override_real_emitted_stamps ... ok
test correction_actual_yaml_requires_complete_matching_paired_attribution ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/relations.rs (target/debug/deps/relations-9e1fbfdeb5e1e1b5)

running 4 tests
test the_committed_openapi_document_is_byte_for_byte_what_the_projection_writes ... ok
test the_openapi_document_states_the_relation_and_links_the_targets_schema ... ok
test the_entity_document_states_the_relation_on_the_property_that_carries_it ... ok
test the_committed_entity_documents_are_byte_for_byte_what_the_schema_projection_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/review_adversary_f01.rs (target/debug/deps/review_adversary_f01-07b39ab4f948a888)

running 3 tests
test complete_comments_do_not_admit_an_unknown_profile_via_body_markers ... ok
test structured_stamp_requires_the_complete_emitted_envelope ... ok
test conflicting_locations_and_every_malformed_profile_are_refused ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/review_adversary_f01_pass2.rs (target/debug/deps/review_adversary_f01_pass2-dba0aa538828ec41)

running 3 tests
test incomplete_authoritative_stamp_cannot_fall_back_to_model_provenance ... ok
test duplicate_digest_aliases_and_duplicate_system_keys_are_unreadable ... ok
test paired_yaml_refuses_mixed_digest_aliases_even_when_hashes_agree ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/schema.rs (target/debug/deps/schema-c872c2e57cbce1cf)

running 27 tests
test a_list_element_may_be_null_where_a_field_may_only_be_absent ... ok
test a_field_carries_its_own_words_beside_the_reference_to_its_type ... ok
test a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test a_field_is_called_what_the_specification_says_it_is_called_on_the_wire ... ok
test a_map_is_an_object_whose_keys_are_the_text_its_key_type_is_spelt_with ... ok
test a_map_key_that_is_not_the_text_its_key_type_is_spelt_with_is_refused ... ok
test a_bytes_field_refuses_a_string_that_is_not_base64 ... ok
test an_optional_field_may_be_absent_and_a_required_field_may_not ... ok
test a_newtype_over_a_string_publishes_no_constraint_the_specification_never_stated ... ok
test an_event_payload_accepts_what_the_specification_says_it_carries ... ok
test a_command_input_accepts_a_filled_instance_and_refuses_a_misspelt_field ... ok
test a_tagged_union_round_trips_because_every_branch_pins_its_tag ... ok
test a_uuid_newtype_carries_the_format_of_what_it_wraps ... ok
test an_error_that_carries_nothing_accepts_an_empty_object_and_nothing_else ... ok
test an_amount_is_written_as_an_exact_decimal_string_and_a_float_is_refused ... ok
test every_artifact_is_a_json_schema_document_declaring_the_dialect_it_is_written_in ... ok
test an_invariant_travels_with_the_type_and_says_it_is_not_a_constraint ... ok
test a_newtype_keeps_its_name_instead_of_collapsing_into_its_representation ... ok
test every_command_input_event_payload_error_payload_and_named_type_gets_a_schema ... ok
test a_uuid_is_refused_unless_it_is_the_canonical_hyphenated_form ... ok
test a_decimal_amount_is_refused_when_it_is_not_written_the_way_the_pattern_says ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_message_accepts_an_instance_of_itself_and_refuses_one_that_is_wrong ... ok
test no_schema_uses_a_keyword_outside_the_set_this_projection_publishes ... ok
test every_published_document_is_a_valid_json_schema_in_the_dialect_it_declares ... ok
test every_schema_says_which_specification_it_came_from ... ok
test generation_is_byte_identical_between_runs ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

     Running unittests src/lib.rs (target/debug/deps/ess_synth-38a9c8cb806bed12)

running 8 tests
test go::name::tests::a_fragment_keeps_every_segment_because_identifiers_are_joined_from_them ... ok
test go::name::tests::a_marker_method_is_unexported_which_is_what_seals_the_interface ... ok
test go::name::tests::a_nested_declaration_becomes_one_identifier ... ok
test rust::name::tests::a_kebab_case_outcome_becomes_a_variant ... ok
test rust::name::tests::a_field_the_specification_may_call_type_is_escaped_rather_than_broken ... ok
test rust::name::tests::a_nested_declaration_becomes_one_identifier ... ok
test go::name::tests::a_package_name_that_would_shadow_a_predeclared_identifier_is_repaired ... ok
test rust::name::tests::a_pascal_case_transition_name_becomes_a_method ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/clap.rs (target/debug/deps/clap-109b221b0c9c3274)

running 10 tests
test a_specification_declaring_no_command_line_emits_no_verbs ... ok
test an_enum_typed_field_carries_its_whole_closed_set ... ok
test the_binary_generates_its_own_completions ... ok
test every_placed_word_is_an_obligation ... ok
test the_manifest_names_the_binary_the_declaration_names ... ok
test the_tree_carries_the_declared_binary_and_its_groups ... ok
test review_second_pass_actual_clap_manifest_retains_complete_comment_admission ... ok
test a_placed_view_becomes_a_verb ... ok
test a_string_field_offers_no_values ... ok
test the_emission_is_deterministic ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/go.rs (target/debug/deps/go-408e408e0985c694)

running 19 tests
test a_map_keyed_by_bytes_is_refused_at_the_target_stage_and_never_emitted ... ok
test an_owed_crossing_gets_its_own_package_because_go_refuses_an_import_cycle ... ok
test an_owed_transformation_and_a_retry_policy_are_emitted_the_way_the_binding_declares_them ... ok
test two_seams_of_one_component_that_derive_one_method_name_are_refused_not_renamed ... ok
test a_newtype_is_a_guarded_struct_and_never_a_defined_string ... ok
test the_generated_transformation_reads_the_event_through_the_declared_crossing ... ok
test every_weakening_is_visible_in_the_generated_source_and_not_only_in_the_report ... ok
test a_command_outcome_keeps_the_refusal_beside_the_success ... ok
test a_closed_set_is_sealed_by_an_unexported_marker_so_no_other_package_can_join_it ... ok
test refinement_answers_ok_because_a_sealed_interfaces_zero_value_names_no_state ... ok
test no_go_source_uses_a_tab_free_indent_or_a_trailing_space ... ok
test the_plans_obligations_and_the_modules_stubs_are_the_same_list ... ok
test an_obligation_is_an_interface_and_a_stub_that_returns_a_value_never_a_panic ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test an_illegal_transition_is_a_method_that_does_not_exist ... ok
test the_transport_is_the_one_the_billing_binding_requires ... ok
test the_plan_is_byte_identical_in_both_targets_trees ... ok
test the_rust_target_reports_nothing_and_the_go_target_reports_its_weakenings ... ok
test emitting_twice_is_byte_identical ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/http.rs (target/debug/deps/http-1f24898b1897b081)

running 9 tests
test a_browser_cannot_bind_a_socket_and_says_so_rather_than_emitting_one ... ok
test the_routes_a_server_answers_are_the_routes_the_contract_declares ... ok
test a_specification_that_says_nothing_about_reach_gets_no_server_at_all ... ok
test the_plan_is_byte_identical_in_both_trees_of_the_demonstration ... ok
test both_applications_carry_the_same_startup_record_outside_the_runtime_they_append ... ok
test review_http_payloads_use_slice_profiles_while_neutral_plans_stay_frozen ... ok
test the_served_contract_is_the_document_the_projection_publishes ... ok
test emitting_a_served_surface_twice_is_byte_identical ... ok
test correction_actual_cargo_manifests_keep_their_comment_provenance ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/relations.rs (target/debug/deps/relations-0da2273dd262509e)

running 2 tests
test the_committed_rust_module_is_byte_for_byte_what_the_projection_writes ... ok
test the_generated_data_struct_says_what_the_field_carrying_a_relation_means ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/synthesis.rs (target/debug/deps/synthesis-f193ff4f35681316)

running 29 tests
test a_domain_named_primitives_cannot_shadow_the_representation_module ... ok
test a_domain_named_obligation_cannot_shadow_the_refusal_module ... ok
test a_component_named_like_a_reserved_package_is_renamed_by_rule ... ok
test colliding_domain_modules_are_renamed_by_rule_not_by_luck ... ok
test a_binding_whose_command_no_component_accepts_is_refused_never_guessed ... ok
test colliding_event_names_become_full_name_variants_by_rule_not_by_luck ... ok
test a_mapping_through_a_non_mechanical_crossing_makes_the_transformation_an_obligation ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test grants_are_refused_rather_than_owed ... ok
test every_construct_of_the_specification_appears_in_the_plan ... ok
test newtypes_stay_distinct_and_the_declared_crossing_is_the_only_bridge ... ok
test a_view_query_obligation_carries_filter_and_consistency ... ok
test two_components_accepting_one_command_is_refused_naming_both ... ok
test the_billing_plan_counts_are_pinned ... ok
test send_email_behaviour_is_owed_with_the_specifications_own_cause ... ok
test a_mechanical_conversion_is_generated_and_any_other_declared_crossing_is_owed ... ok
test a_component_port_is_typed_against_the_generated_types ... ok
test a_command_outcome_enum_keeps_the_refusal_beside_the_success ... ok
test the_billing_binding_is_generated_where_determined_and_owed_where_not ... ok
test the_billing_plan_gives_every_capability_exactly_one_disposition ... ok
test only_the_initial_state_can_be_constructed ... ok
test a_stub_refuses_with_a_value_never_a_panic_and_never_a_todo ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test the_plans_obligations_and_the_workspaces_stubs_are_the_same_list ... ok
test the_transport_is_the_one_the_billing_binding_requires ... ok
test the_transport_records_its_invocations_and_can_deliver_an_occurrence_twice ... ok
test the_plan_never_names_the_emission_language ... ok
test the_legal_transitions_are_the_whole_transition_api ... ok
test emitting_twice_is_byte_identical ... ok

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/web.rs (target/debug/deps/web-1cb70fdbab20ef2a)

running 17 tests
test a_command_no_component_accepts_is_refused_at_the_target_stage_and_gets_no_form ... ok
test an_absent_optional_field_is_omitted_rather_than_sent_as_null ... ok
test a_list_and_a_map_cross_as_the_shapes_json_already_has ... ok
test the_committed_tree_holds_no_compiled_module ... ok
test the_catalogue_carries_every_command_with_its_typed_input_and_every_declared_outcome ... ok
test the_public_browser_catalog_is_the_web_targets_exact_document ... ok
test every_generated_type_crosses_the_boundary_in_both_directions ... ok
test the_page_names_no_construct_of_the_specification_it_was_generated_from ... ok
test a_tagged_union_crosses_where_the_published_schema_says_its_payload_sits ... ok
test the_bridge_names_no_realization_and_installs_none ... ok
test the_web_target_reports_six_weakenings_and_refuses_nothing_of_billing ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test every_weakening_is_visible_in_the_generated_source_and_not_only_in_the_report ... ok
test the_catalogue_carries_the_lifecycle_and_says_where_instances_can_be_observed ... ok
test the_bridge_takes_no_dependency_because_the_gate_reaches_no_network ... ok
test emitting_twice_is_byte_identical ... ok
test the_plan_is_byte_identical_in_all_three_targets_trees ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

   Doc-tests ess_compiler

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_diff

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_gen

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_synth

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p ess-diff --test review_adversary_f01_pass2`

```

### correction2-mechanism-green-1

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff -p ess-gen --test review_adversary_f01 --test review_adversary_f01_pass2 --test graph --no-fail-fast -- --nocapture
```

Exit 0. Full preserved output:

```text
   Compiling ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-compiler)
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
   Compiling ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-conformance)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 9.75s
     Running tests/graph.rs (target/debug/deps/graph-b94d34c7c7fe4967)

running 10 tests
test correction2_row_shape_is_a_distinct_dependency_and_survives_graph_union ... ok
test a_component_is_reached_through_what_it_accepts_and_publishes ... ok
test the_graph_records_the_reference_an_author_wrote_and_not_its_reverse ... ok
test a_type_is_reached_through_the_declarations_that_hold_it_and_not_by_name ... ok
test a_closure_over_the_whole_model_terminates_and_stays_inside_it ... ok
test building_the_same_graph_twice_produces_the_same_edges_in_the_same_order ... ok
test review_cli_views_and_parameter_types_are_forward_slice_dependencies ... ok
test correction2_network_exposure_matches_actual_routes_and_owned_domains ... ok
test review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union ... ok
test every_relation_in_the_vocabulary_is_minted_by_a_specification_this_repository_ships ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/review_adversary_f01.rs (target/debug/deps/review_adversary_f01-8795d8f802dd31cc)

running 7 tests
test explicit_domain_naming_defaults_remain_semantically_equivalent ... ok
test complete_generated_and_authored_suite_four_bytes_remain_frozen ... ok
test relation_delta_versions_refuse_relabeling_and_public_serialize_bypasses ... ok
delta: {
  "format": "ess-diff/2",
  "before": {
    "system": "billing",
    "specification_version": 3,
    "spec_digest": "68b6804aca957b2c8f5505c34b7cd35d69e018a96886692317a664ee37b52eae"
  },
  "after": {
    "system": "billing",
    "specification_version": 3,
    "spec_digest": "78d5b9e5d397e9a7b87bf313de23d482c40260294a1cf2d42eb59936ef77a51c"
  },
  "changes": [
    {
      "id": "system/billing/unclassified-changed",
      "relation": "changed",
      "change": {
        "category": "system",
        "subject": "billing",
        "changed": {
          "kind": "unclassified-changed"
        }
      }
    }
  ]
}

invalidation: Some(Whole { because: UncomparedFamilyChanged })
test moved_outcome_reference_retains_its_owner ... ok
delta: {
  "format": "ess-diff/2",
  "before": {
    "system": "billing",
    "specification_version": 3,
    "spec_digest": "68b6804aca957b2c8f5505c34b7cd35d69e018a96886692317a664ee37b52eae"
  },
  "after": {
    "system": "billing",
    "specification_version": 3,
    "spec_digest": "54198b00f1d234b83ffd852eb4434f7a6678e585ace7a5d2e9b29bc400092385"
  },
  "changes": [
    {
      "id": "system/billing/unclassified-changed",
      "relation": "changed",
      "change": {
        "category": "system",
        "subject": "billing",
        "changed": {
          "kind": "unclassified-changed"
        }
      }
    },
    {
      "id": "command/billing.email.SendEmail/summary-changed",
      "relation": "changed",
      "change": {
        "category": "command",
        "subject": "billing.email.SendEmail",
        "changed": {
          "kind": "summary-changed",
          "before": null,
          "after": "A classified change beside the moved reference."
        }
      }
    }
  ]
}

invalidation: Some(Whole { because: UncomparedFamilyChanged })
test moved_outcome_reference_is_independent_of_a_classified_edit ... ok
test ownership_cardinality_invalidates_both_emitted_schema_ends ... ok
incomplete schema/types/billing.invoice.InvoiceId.schema.json obligation: Some(ProvenanceUnreadable)
test incomplete_schema_stamp_is_owed_by_the_real_impact_reader ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.66s

     Running tests/review_adversary_f01_pass2.rs (target/debug/deps/review_adversary_f01_pass2-1ffc276d4342fd42)

running 5 tests
test ranking_precedence_survives_the_checked_delta_roundtrip ... ok
view old digest=slice-sha256/2:93e4e45c671679f3540cbce7a374477d62a012d32d51b3599caa41edd9ae3ed6, new digest=slice-sha256/2:c9323e8bc1a51b637c0d33a292e14f490ac7151f3300db2f6c4731f82a3ae3e1
test reusable_row_type_belongs_to_the_view_slice_it_supplies ... ok
test switching_equal_row_shapes_retains_independent_residual_coverage ... ok
delta: {
  "format": "ess-diff/2",
  "before": {
    "system": "probe",
    "specification_version": 1,
    "spec_digest": "3008ddc5eb033c30a5d6f6c4abaef84590aaeb8393672ac6395358e7392ff0d8"
  },
  "after": {
    "system": "probe",
    "specification_version": 1,
    "spec_digest": "47ec6afd1f689995b9cd99f393e538d29aa0b8cd10fd1feeb01c7340e24c1ca6"
  },
  "changes": [
    {
      "id": "type/probe.core.Row/invariants-changed",
      "relation": "changed",
      "change": {
        "category": "type",
        "subject": "probe.core.Row",
        "changed": {
          "kind": "invariants-changed",
          "before": [
            "amount >= 0"
          ],
          "after": [
            "amount > 0"
          ]
        }
      }
    }
  ]
}

OpenAPI contract digest moved: true; OpenAPI owed: true; old=ArtifactDigests { source_digest: "3008ddc5eb033c30a5d6f6c4abaef84590aaeb8393672ac6395358e7392ff0d8", contract_digest: "slice-sha256/2:7cfdfe8c8c39670b6980a73fe47da31be540765c55793e1c51163f22ca4a0636" }; new=ArtifactDigests { source_digest: "47ec6afd1f689995b9cd99f393e538d29aa0b8cd10fd1feeb01c7340e24c1ca6", contract_digest: "slice-sha256/2:d0408e9ea0d5b8b045e3eb14ea217e19ddebbff4c6d07a5738669c48400099e0" }
test reusable_row_invariant_change_reaches_its_openapi_artifact ... ok
delta: {
  "format": "ess-diff/2",
  "before": {
    "system": "probe",
    "specification_version": 1,
    "spec_digest": "3008ddc5eb033c30a5d6f6c4abaef84590aaeb8393672ac6395358e7392ff0d8"
  },
  "after": {
    "system": "probe",
    "specification_version": 1,
    "spec_digest": "2d875058ae8c82bf51cc644774c86a1d8d252507695ba3c328f3a9e1eeea3c1d"
  },
  "changes": [
    {
      "id": "view/probe.core.Items/summary-changed",
      "relation": "changed",
      "change": {
        "category": "view",
        "subject": "probe.core.Items",
        "changed": {
          "kind": "summary-changed",
          "before": "Original rows.",
          "after": "Revised rows."
        }
      }
    }
  ]
}

OpenAPI contract digest moved: true; OpenAPI owed: true; old=ArtifactDigests { source_digest: "3008ddc5eb033c30a5d6f6c4abaef84590aaeb8393672ac6395358e7392ff0d8", contract_digest: "slice-sha256/2:7cfdfe8c8c39670b6980a73fe47da31be540765c55793e1c51163f22ca4a0636" }; new=ArtifactDigests { source_digest: "2d875058ae8c82bf51cc644774c86a1d8d252507695ba3c328f3a9e1eeea3c1d", contract_digest: "slice-sha256/2:4163a9a2affacfeb21dfac5b110eca1d0f985e9ea1d48fde3c5a4192ea2d810f" }
test served_view_change_reaches_its_openapi_artifact ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/review_adversary_f01.rs (target/debug/deps/review_adversary_f01-d8564f620078ebe8)

running 3 tests
test complete_comments_do_not_admit_an_unknown_profile_via_body_markers ... ok
removed system: None
removed specification_version: None
removed regenerate: None
test structured_stamp_requires_the_complete_emitted_envelope ... ok
test conflicting_locations_and_every_malformed_profile_are_refused ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/review_adversary_f01_pass2.rs (target/debug/deps/review_adversary_f01_pass2-f07c90bf79ce56f2)

running 3 tests
test incomplete_authoritative_stamp_cannot_fall_back_to_model_provenance ... ok
test duplicate_digest_aliases_and_duplicate_system_keys_are_unreadable ... ok
test paired_yaml_refuses_mixed_digest_aliases_even_when_hashes_agree ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


```

### correction2-controls-green-1

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-diff -p ess-gen -p ess-synth --test families --test artifacts --test canonical --test provenance --test http --test clap --no-fail-fast
```

Exit 0. Full preserved output:

```text
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-synth)
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 1.87s
     Running tests/artifacts.rs (target/debug/deps/artifacts-9f64f03ab905b12c)

running 13 tests
test an_artifact_whose_slice_nothing_reached_is_absent_from_the_answer ... ok
test an_owed_artifacts_path_explains_the_membership_hop_by_hop ... ok
test a_change_to_the_system_header_owes_every_artifact ... ok
test the_artifacts_the_currency_changes_reach_are_owed_and_named ... ok
test whole_model_artifacts_are_owed_by_any_change_at_all ... ok
test the_six_change_delta_owes_a_strict_subset_of_the_artifacts ... ok
test a_grant_change_owes_the_documents_that_read_grants_and_not_the_ones_that_do_not ... ok
test the_two_predicate_edits_narrow_the_artifacts_differently_and_both_subsets_are_named ... ok
test the_artifact_answer_is_byte_identical_between_runs ... ok
test review_whole_model_hashes_and_index_bytes_remain_frozen ... ok
test a_committed_tree_is_answered_for_fail_closed_file_by_file ... ok
test a_committed_artifact_with_a_false_contract_digest_is_owed_as_a_false_claim ... ok
test review_legacy_slice_stamps_are_owed_even_when_raw_hashes_match ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

     Running tests/canonical.rs (target/debug/deps/canonical-d218f8527473a680)

running 20 tests
test a_binding_still_has_one_delivery_a_document_can_write ... ok
test every_change_variant_has_something_to_say_for_itself ... ok
test review_freeze_legacy_delta_bytes ... ok
test review_version_admission_refuses_new_vocabulary_in_legacy_envelopes ... ok
test a_change_is_spelt_the_same_way_in_its_id_and_in_the_document ... ok
test no_source_file_in_the_diff_engine_reads_a_clock_or_an_unordered_map ... ok
test no_source_file_in_the_diff_engine_calls_an_ir_handle_accessor ... ok
test a_system_still_has_no_naming_a_document_can_set ... ok
test a_delta_whose_changes_are_out_of_order_is_refused ... ok
test a_delta_naming_two_systems_is_refused_on_the_way_in_as_well ... ok
test a_delta_whose_id_was_edited_is_refused ... ok
test every_change_in_a_delta_has_its_own_id ... ok
test canonical_json_ends_in_a_newline ... ok
test review_new_default_delta_format_is_version_two ... ok
test the_changes_are_written_in_the_category_order_and_not_the_alphabet ... ok
test a_delta_written_in_a_format_this_build_does_not_read_is_refused ... ok
test a_delta_this_build_wrote_is_read_back_without_complaint ... ok
test a_delta_whose_relation_was_edited_is_refused ... ok
test a_document_with_six_defects_reports_six ... ok
test diffing_the_same_pair_twice_produces_byte_identical_json ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/families.rs (target/debug/deps/families-5b96283fbb857370)

running 69 tests
test a_filter_respaced_is_the_same_predicate_and_no_change ... ok
test a_construct_moving_between_files_is_not_a_change ... ok
test a_newtype_that_wraps_something_else_is_reported ... ok
test a_commands_naming_is_compared_key_by_key ... ok
test a_payload_declaration_arriving_is_a_payload_change ... ok
test a_filter_removed_reads_as_containing_every_instance ... ok
test a_filter_that_contains_different_instances_is_changed_with_no_direction ... ok
test a_binding_invoking_a_different_command_moves_its_mapping_with_it ... ok
test a_guard_respaced_is_the_same_predicate_and_no_change ... ok
test a_bindings_failure_policy_is_compared ... ok
test a_bindings_naming_is_compared_key_by_key ... ok
test a_mapping_filled_from_somewhere_else_is_reported_with_both_sources ... ok
test a_binding_reacting_to_a_different_event_is_reported ... ok
test a_binding_added_is_one_change ... ok
test a_new_transition_arrives_with_the_outcome_that_takes_it ... ok
test a_command_added_is_one_change ... ok
test a_component_that_no_longer_publishes_an_event_is_reported ... ok
test a_component_accepting_a_new_command_is_changed_and_not_widened ... ok
test a_type_that_became_a_different_kind_of_thing_is_reported_as_that_and_nothing_else ... ok
test a_union_variant_that_carries_something_else_is_not_a_variant_removed_and_added ... ok
test a_types_own_invariants_are_reported_as_different_and_never_as_stronger ... ok
test a_union_that_is_tagged_by_another_field_is_reported ... ok
test a_struct_field_that_changed_type_is_reported ... ok
test a_union_gaining_a_variant_widens_it_just_as_an_enum_does ... ok
test an_entity_fields_naming_is_compared_key_by_key ... ok
test an_entitys_naming_is_compared_key_by_key ... ok
test a_views_consistency_promise_is_compared_and_not_classified ... ok
test a_view_fields_naming_is_compared_key_by_key ... ok
test a_view_projecting_a_different_entity_is_a_source_change ... ok
test an_event_field_that_changed_type_is_reported ... ok
test a_view_exposing_a_new_field_is_reported_with_the_type_it_carries ... ok
test an_entity_added_arrives_with_its_synthesised_state_enum_and_nothing_is_diffed_inside ... ok
test a_view_added_is_one_change ... ok
test a_views_naming_is_compared_key_by_key ... ok
test an_entity_field_replaced_is_removed_and_added_and_never_a_rename ... ok
test an_entity_field_that_changed_type_is_reported ... ok
test an_actor_declared_with_no_grants_at_all_is_still_a_change_to_report ... ok
test an_error_that_gained_a_field_is_reported_with_the_type_it_carries ... ok
test an_events_wire_name_moving_is_not_the_event_moving ... ok
test an_invariant_statement_reworded_without_moving_the_predicate_is_still_a_change ... ok
test an_input_fields_naming_is_compared_key_by_key ... ok
test an_identitys_display_name_and_summary_are_compared ... ok
test an_input_that_changed_type_is_reported ... ok
test an_event_renamed_is_reported_as_removed_and_added_and_never_as_a_rename ... ok
test an_input_added_is_reported_with_the_type_it_carries ... ok
test an_outcome_added_is_one_change_and_claims_no_direction ... ok
test reordering_an_entitys_fields_is_reported_once ... ok
test reordering_a_commands_input_is_reported_once ... ok
test reordering_a_commands_outcomes_is_a_real_change ... ok
test reordering_an_event_payload_is_reported_once_and_not_as_a_field_change ... ok
test renaming_an_entitys_identity_is_the_one_rename_this_crate_reports ... ok
test an_outcomes_summary_is_compared ... ok
test reordering_an_enums_variants_is_reported_without_claiming_a_direction ... ok
test reordering_a_views_fields_is_reported_once ... ok
test what_an_outcome_emits_is_compared_in_order ... ok
test what_an_error_tells_the_caller_is_compared ... ok
test writing_out_a_naming_default_is_not_a_change ... ok
test the_paragraph_saying_what_the_system_is_is_compared ... ok
test the_specifications_version_moving_is_reported_and_is_not_the_identity ... ok
test the_error_a_branch_reports_is_compared ... ok
test review_reach_is_a_change_without_an_unrelated_surface_edit ... ok
test review_outcome_refusal_is_independent_of_its_error ... ok
test review_view_parameter_naming_is_compared_without_a_filter_edit ... ok
test review_cli_top_level_grouped_views_and_binary_are_changes ... ok
test review_outcome_sets_are_independent_of_event_payload ... ok
test review_view_ranking_is_compared_without_a_filter_edit ... ok
test review_residual_refs_cannot_hide_beside_a_classified_change ... ok
test review_unclassified_transition_order_cannot_hide_beside_a_classified_edit ... ok
test review_relation_cardinality_name_and_removal_are_changes ... ok

test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s

     Running tests/provenance.rs (target/debug/deps/provenance-9ec9789f25e84015)

running 18 tests
test a_damaged_digest_reads_as_nothing ... ok
test a_text_without_both_digests_reads_as_nothing ... ok
test a_generator_that_stamps_nothing_cannot_ship_an_artifact - should panic ... ok
test a_whole_model_slice_is_stamped_as_one ... ok
test a_generator_that_pairs_a_stamp_with_the_wrong_slice_cannot_ship_an_artifact - should panic ... ok
test review_new_reader_refuses_unsupported_profile_without_legacy_fallback ... ok
test review_new_reader_requires_envelopes_and_exact_digest_tokens ... ok
test the_reader_reads_back_every_form_the_writer_emits ... ok
test review_every_constructs_digest_has_an_explicit_profile_and_whole_remains_bare ... ok
test the_whole_model_contract_digest_is_not_the_source_digest ... ok
test a_change_no_construct_can_be_named_for_moves_every_contract_digest ... ok
test a_change_outside_an_artifacts_slice_leaves_its_contract_digest_standing ... ok
test review_docs_ir_retains_page_profiles_and_does_not_claim_a_flat_stamp ... ok
test review_conflicting_structured_and_comment_stamps_are_unreadable ... ok
test review_marker_looking_model_content_does_not_override_real_emitted_stamps ... ok
test review_profile_is_read_in_all_emissions_and_old_reader_refuses_ordinary_slices ... ok
test correction_structured_envelopes_require_typed_attribution ... ok
test correction_actual_yaml_requires_complete_matching_paired_attribution ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/clap.rs (target/debug/deps/clap-109b221b0c9c3274)

running 10 tests
test a_specification_declaring_no_command_line_emits_no_verbs ... ok
test a_placed_view_becomes_a_verb ... ok
test an_enum_typed_field_carries_its_whole_closed_set ... ok
test a_string_field_offers_no_values ... ok
test the_tree_carries_the_declared_binary_and_its_groups ... ok
test review_second_pass_actual_clap_manifest_retains_complete_comment_admission ... ok
test every_placed_word_is_an_obligation ... ok
test the_manifest_names_the_binary_the_declaration_names ... ok
test the_binary_generates_its_own_completions ... ok
test the_emission_is_deterministic ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/http.rs (target/debug/deps/http-1f24898b1897b081)

running 9 tests
test a_browser_cannot_bind_a_socket_and_says_so_rather_than_emitting_one ... ok
test the_routes_a_server_answers_are_the_routes_the_contract_declares ... ok
test a_specification_that_says_nothing_about_reach_gets_no_server_at_all ... ok
test the_plan_is_byte_identical_in_both_trees_of_the_demonstration ... ok
test both_applications_carry_the_same_startup_record_outside_the_runtime_they_append ... ok
test review_http_payloads_use_slice_profiles_while_neutral_plans_stay_frozen ... ok
test the_served_contract_is_the_document_the_projection_publishes ... ok
test emitting_a_served_surface_twice_is_byte_identical ... ok
test correction_actual_cargo_manifests_keep_their_comment_provenance ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s


```

### correction2-clippy-1

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo clippy --offline --locked -p ess-compiler -p ess-diff -p ess-gen -p ess-synth --all-targets -- -D warnings
```

Exit 0. Full preserved output:

```text
    Checking ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-compiler)
    Checking ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
    Checking ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-conformance)
    Checking ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-synth)
    Checking ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `dev` profile [unoptimized] target(s) in 5.08s

```

### correction2-packages-1

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --offline --locked -p ess-compiler -p ess-diff -p ess-gen -p ess-synth --no-fail-fast
```

Exit 0. Full preserved output:

```text
   Compiling ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-compiler)
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-synth)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
    Finished `test` profile [unoptimized] target(s) in 3.25s
     Running unittests src/lib.rs (target/debug/deps/ess_compiler-b4c0bc8f0757838a)

running 19 tests
test graph::tests::a_closure_keeps_the_edges_that_explain_each_construct_it_reached ... ok
test graph::tests::a_slice_includes_its_seeds_each_with_no_path ... ok
test graph::tests::a_closure_walks_the_edges_backwards_and_not_forwards ... ok
test graph::tests::merging_two_graphs_can_only_ever_reach_more ... ok
test graph::tests::a_command_in_a_slice_brings_its_outcomes_and_what_they_name ... ok
test graph::tests::the_construct_that_changed_is_in_its_own_closure_with_no_path ... ok
test graph::tests::a_slice_reaches_what_a_seed_rests_on_transitively ... ok
test resolve::tests::a_code_the_bridge_has_no_class_for_still_gets_one ... ok
test resolve::tests::a_declaration_written_once_is_located_at_its_own_line_and_column ... ok
test resolve::tests::a_needle_in_two_files_is_not_located_because_one_of_them_is_wrong ... ok
test resolve::tests::a_refusal_from_the_domain_crate_keeps_the_code_the_compiler_would_have_given_it ... ok
test resolve::tests::every_code_renders_as_its_family_and_number ... ok
test resolve::tests::a_needle_that_occurs_twice_is_not_located_because_the_wrong_line_is_worse_than_none ... ok
test resolve::tests::a_bridged_refusal_is_located_by_the_declaration_its_path_names ... ok
test resolve::tests::the_second_needle_is_tried_when_the_first_is_ambiguous ... ok
test resolve::tests::with_no_files_named_a_span_still_carries_the_document_path ... ok
test resolve::tests::the_register_lists_every_code_it_declares ... ok
test resolve::tests::every_named_code_is_a_family_paired_with_a_class ... ok
test resolve::tests::a_refusal_is_filed_under_the_layer_its_document_path_names ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/adversarial.rs (target/debug/deps/adversarial-b2f6b2147eadfc2d)

running 2 tests
test the_generator_reaches_both_compilation_and_refusal ... ok
test every_document_is_refused_with_reasons_or_compiled_identically_twice ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running tests/billing.rs (target/debug/deps/billing-127704819ded2aac)

running 14 tests
test a_refusal_from_the_whole_pipeline_carries_a_code_and_the_line_it_belongs_on ... ok
test no_source_file_in_the_compiler_reads_a_clock_or_an_unordered_map ... ok
test a_binding_that_escalates_carries_the_event_it_emits_as_a_handle ... ok
test the_billing_specification_resolves ... ok
test every_stable_reference_from_the_compiler_graph_resolves_against_its_ir ... ok
test every_handle_in_the_ir_names_something_the_ir_holds ... ok
test a_field_keeps_the_shape_of_its_type_rather_than_a_rendering_of_it ... ok
test the_crossing_between_two_contexts_is_recorded_with_the_reason_someone_gave_for_it ... ok
test the_reaction_graph_names_the_binding_that_causes_each_command ... ok
test canonical_json_ends_in_a_newline ... ok
test the_json_orders_its_keys_the_way_a_btreemap_does ... ok
test the_source_digest_names_exactly_the_canonical_semantic_model ... ok
test compiling_without_the_file_list_still_reports_the_document_path ... ok
test compiling_the_billing_example_twice_produces_byte_identical_json ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/oracle_fixture.rs (target/debug/deps/oracle_fixture-58359c9d77095e64)

running 11 tests
test the_command_every_binding_invokes_can_be_forced_to_fail ... ok
test a_binding_maps_an_event_field_that_has_a_same_typed_sibling ... ok
test an_outcome_updates_an_entity_without_moving_it_and_that_entity_declares_an_invariant ... ok
test dropping_one_binding_leaves_others_with_scenarios_of_their_own ... ok
test a_row_reaches_the_read_your_writes_view_after_a_single_command ... ok
test the_fixture_compiles_from_the_files_it_lives_in ... ok
test every_on_failure_policy_the_model_has_is_reachable_in_this_fixture ... ok
test the_eventual_view_converges_on_a_state_the_creating_command_does_not_reach ... ok
test an_illegal_transition_can_be_attempted_from_a_state_a_scenario_can_reach ... ok
test the_fixture_carries_something_the_normative_example_does_not ... ok
test every_input_the_oracle_needs_is_carried_by_one_of_the_examples ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/sealed_state.rs (target/debug/deps/sealed_state-fe1b4fe83ef9373a)

running 3 tests
test provenance_never_hashes_an_empty_serialization_fallback ... ok
test every_compiler_entrance_validates_before_resolution ... ok
test validated_and_resolved_state_have_no_public_fields ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/view_shapes.rs (target/debug/deps/view_shapes-6be70769d1254075)

running 1 test
test a_shape_is_one_handle_with_checked_fields_in_every_view ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/ess_diff-6c940e1b4fa9bdeb)

running 9 tests
test change::tests::only_a_grant_and_a_variant_decide_a_direction ... ok
test change::tests::a_change_with_no_member_renders_three_parts_rather_than_a_trailing_slash ... ok
test change::tests::a_change_id_names_its_category_subject_subtype_and_member_in_that_order ... ok
test change::tests::the_canonical_order_is_the_category_order_and_not_the_alphabet ... ok
test impact::tests::a_whole_answer_absorbs_a_narrowing_whichever_way_round_they_are_joined ... ok
test delta::tests::a_delta_puts_its_changes_in_canonical_order_however_they_arrive ... ok
test impact::tests::an_unfollowed_file_is_not_an_artifact_that_owes_regeneration ... ok
test impact::tests::a_change_to_the_specification_itself_owes_the_whole_suite ... ok
test impact::tests::a_suite_resting_on_a_construct_the_graph_has_no_node_for_owes_the_whole_suite ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/artifacts.rs (target/debug/deps/artifacts-9f64f03ab905b12c)

running 13 tests
test the_six_change_delta_owes_a_strict_subset_of_the_artifacts ... ok
test a_change_to_the_system_header_owes_every_artifact ... ok
test the_artifacts_the_currency_changes_reach_are_owed_and_named ... ok
test an_artifact_whose_slice_nothing_reached_is_absent_from_the_answer ... ok
test an_owed_artifacts_path_explains_the_membership_hop_by_hop ... ok
test the_two_predicate_edits_narrow_the_artifacts_differently_and_both_subsets_are_named ... ok
test whole_model_artifacts_are_owed_by_any_change_at_all ... ok
test a_grant_change_owes_the_documents_that_read_grants_and_not_the_ones_that_do_not ... ok
test the_artifact_answer_is_byte_identical_between_runs ... ok
test a_committed_tree_is_answered_for_fail_closed_file_by_file ... ok
test a_committed_artifact_with_a_false_contract_digest_is_owed_as_a_false_claim ... ok
test review_whole_model_hashes_and_index_bytes_remain_frozen ... ok
test review_legacy_slice_stamps_are_owed_even_when_raw_hashes_match ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

     Running tests/canonical.rs (target/debug/deps/canonical-d218f8527473a680)

running 20 tests
test a_binding_still_has_one_delivery_a_document_can_write ... ok
test every_change_variant_has_something_to_say_for_itself ... ok
test a_change_is_spelt_the_same_way_in_its_id_and_in_the_document ... ok
test review_freeze_legacy_delta_bytes ... ok
test review_version_admission_refuses_new_vocabulary_in_legacy_envelopes ... ok
test no_source_file_in_the_diff_engine_reads_a_clock_or_an_unordered_map ... ok
test no_source_file_in_the_diff_engine_calls_an_ir_handle_accessor ... ok
test a_system_still_has_no_naming_a_document_can_set ... ok
test every_change_in_a_delta_has_its_own_id ... ok
test a_delta_whose_changes_are_out_of_order_is_refused ... ok
test a_delta_this_build_wrote_is_read_back_without_complaint ... ok
test a_delta_naming_two_systems_is_refused_on_the_way_in_as_well ... ok
test a_delta_whose_id_was_edited_is_refused ... ok
test canonical_json_ends_in_a_newline ... ok
test review_new_default_delta_format_is_version_two ... ok
test a_delta_whose_relation_was_edited_is_refused ... ok
test the_changes_are_written_in_the_category_order_and_not_the_alphabet ... ok
test a_delta_written_in_a_format_this_build_does_not_read_is_refused ... ok
test a_document_with_six_defects_reports_six ... ok
test diffing_the_same_pair_twice_produces_byte_identical_json ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/families.rs (target/debug/deps/families-5b96283fbb857370)

running 69 tests
test a_filter_that_contains_different_instances_is_changed_with_no_direction ... ok
test a_bindings_naming_is_compared_key_by_key ... ok
test a_filter_respaced_is_the_same_predicate_and_no_change ... ok
test a_component_accepting_a_new_command_is_changed_and_not_widened ... ok
test a_component_that_no_longer_publishes_an_event_is_reported ... ok
test a_filter_removed_reads_as_containing_every_instance ... ok
test a_guard_respaced_is_the_same_predicate_and_no_change ... ok
test a_binding_added_is_one_change ... ok
test a_construct_moving_between_files_is_not_a_change ... ok
test a_commands_naming_is_compared_key_by_key ... ok
test a_bindings_failure_policy_is_compared ... ok
test a_binding_invoking_a_different_command_moves_its_mapping_with_it ... ok
test a_command_added_is_one_change ... ok
test a_newtype_that_wraps_something_else_is_reported ... ok
test a_new_transition_arrives_with_the_outcome_that_takes_it ... ok
test a_mapping_filled_from_somewhere_else_is_reported_with_both_sources ... ok
test a_binding_reacting_to_a_different_event_is_reported ... ok
test a_payload_declaration_arriving_is_a_payload_change ... ok
test a_struct_field_that_changed_type_is_reported ... ok
test a_types_own_invariants_are_reported_as_different_and_never_as_stronger ... ok
test a_type_that_became_a_different_kind_of_thing_is_reported_as_that_and_nothing_else ... ok
test a_union_that_is_tagged_by_another_field_is_reported ... ok
test a_union_gaining_a_variant_widens_it_just_as_an_enum_does ... ok
test a_view_exposing_a_new_field_is_reported_with_the_type_it_carries ... ok
test a_view_added_is_one_change ... ok
test a_union_variant_that_carries_something_else_is_not_a_variant_removed_and_added ... ok
test a_view_fields_naming_is_compared_key_by_key ... ok
test a_views_naming_is_compared_key_by_key ... ok
test an_entity_field_replaced_is_removed_and_added_and_never_a_rename ... ok
test a_view_projecting_a_different_entity_is_a_source_change ... ok
test an_actor_declared_with_no_grants_at_all_is_still_a_change_to_report ... ok
test an_entity_added_arrives_with_its_synthesised_state_enum_and_nothing_is_diffed_inside ... ok
test an_event_field_that_changed_type_is_reported ... ok
test an_entity_fields_naming_is_compared_key_by_key ... ok
test an_entity_field_that_changed_type_is_reported ... ok
test an_event_renamed_is_reported_as_removed_and_added_and_never_as_a_rename ... ok
test an_events_wire_name_moving_is_not_the_event_moving ... ok
test a_views_consistency_promise_is_compared_and_not_classified ... ok
test an_entitys_naming_is_compared_key_by_key ... ok
test an_error_that_gained_a_field_is_reported_with_the_type_it_carries ... ok
test an_input_fields_naming_is_compared_key_by_key ... ok
test an_input_added_is_reported_with_the_type_it_carries ... ok
test an_identitys_display_name_and_summary_are_compared ... ok
test an_invariant_statement_reworded_without_moving_the_predicate_is_still_a_change ... ok
test an_input_that_changed_type_is_reported ... ok
test renaming_an_entitys_identity_is_the_one_rename_this_crate_reports ... ok
test reordering_an_enums_variants_is_reported_without_claiming_a_direction ... ok
test an_outcome_added_is_one_change_and_claims_no_direction ... ok
test reordering_a_views_fields_is_reported_once ... ok
test an_outcomes_summary_is_compared ... ok
test reordering_an_event_payload_is_reported_once_and_not_as_a_field_change ... ok
test reordering_a_commands_outcomes_is_a_real_change ... ok
test reordering_an_entitys_fields_is_reported_once ... ok
test reordering_a_commands_input_is_reported_once ... ok
test the_error_a_branch_reports_is_compared ... ok
test what_an_error_tells_the_caller_is_compared ... ok
test what_an_outcome_emits_is_compared_in_order ... ok
test writing_out_a_naming_default_is_not_a_change ... ok
test the_specifications_version_moving_is_reported_and_is_not_the_identity ... ok
test the_paragraph_saying_what_the_system_is_is_compared ... ok
test review_view_parameter_naming_is_compared_without_a_filter_edit ... ok
test review_reach_is_a_change_without_an_unrelated_surface_edit ... ok
test review_outcome_refusal_is_independent_of_its_error ... ok
test review_cli_top_level_grouped_views_and_binary_are_changes ... ok
test review_view_ranking_is_compared_without_a_filter_edit ... ok
test review_outcome_sets_are_independent_of_event_payload ... ok
test review_residual_refs_cannot_hide_beside_a_classified_change ... ok
test review_unclassified_transition_order_cannot_hide_beside_a_classified_edit ... ok
test review_relation_cardinality_name_and_removal_are_changes ... ok

test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s

     Running tests/graph.rs (target/debug/deps/graph-b94d34c7c7fe4967)

running 10 tests
test correction2_row_shape_is_a_distinct_dependency_and_survives_graph_union ... ok
test a_type_is_reached_through_the_declarations_that_hold_it_and_not_by_name ... ok
test a_closure_over_the_whole_model_terminates_and_stays_inside_it ... ok
test the_graph_records_the_reference_an_author_wrote_and_not_its_reverse ... ok
test a_component_is_reached_through_what_it_accepts_and_publishes ... ok
test building_the_same_graph_twice_produces_the_same_edges_in_the_same_order ... ok
test correction2_network_exposure_matches_actual_routes_and_owned_domains ... ok
test review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union ... ok
test review_cli_views_and_parameter_types_are_forward_slice_dependencies ... ok
test every_relation_in_the_vocabulary_is_minted_by_a_specification_this_repository_ships ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/impact.rs (target/debug/deps/impact-130b9b43fe12ce7e)

running 15 tests
test a_suite_produced_from_the_later_revision_is_refused_rather_than_narrowed ... ok
test two_specifications_of_different_systems_are_refused_here_too ... ok
test a_suite_whose_contract_digest_its_model_does_not_compute_is_refused ... ok
test a_suite_for_another_system_is_refused ... ok
test a_suite_resting_on_a_construct_no_graph_has_a_node_for_owes_the_whole_suite ... ok
test an_edited_entity_invariant_owes_every_scenario_that_rests_on_the_entity_and_no_other ... ok
test an_edited_outcome_guard_owes_every_scenario_because_every_scenario_creates_through_it ... ok
test a_variant_removed_from_an_enum_reaches_the_entity_that_holds_it_transitively ... ok
test every_scenario_resting_directly_on_a_changed_construct_is_owed_again ... ok
test the_suite_the_fixture_obliges_is_ten_scenarios_and_the_delta_is_six_changes ... ok
test taking_a_grant_from_an_actor_owes_only_the_scenarios_that_act_as_that_actor ... ok
test a_narrowed_answer_never_reports_more_scenarios_than_the_suite_holds ... ok
test a_domains_naming_moving_owes_the_whole_suite_because_no_family_compares_a_domain ... ok
test analysing_the_same_pair_twice_produces_byte_identical_json ... ok
test a_change_in_a_family_the_delta_still_does_not_compare_owes_the_whole_suite ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s

     Running tests/review_adversary_f01.rs (target/debug/deps/review_adversary_f01-8795d8f802dd31cc)

running 7 tests
test explicit_domain_naming_defaults_remain_semantically_equivalent ... ok
test complete_generated_and_authored_suite_four_bytes_remain_frozen ... ok
test relation_delta_versions_refuse_relabeling_and_public_serialize_bypasses ... ok
test moved_outcome_reference_retains_its_owner ... ok
test moved_outcome_reference_is_independent_of_a_classified_edit ... ok
test ownership_cardinality_invalidates_both_emitted_schema_ends ... ok
test incomplete_schema_stamp_is_owed_by_the_real_impact_reader ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s

     Running tests/review_adversary_f01_pass2.rs (target/debug/deps/review_adversary_f01_pass2-1ffc276d4342fd42)

running 5 tests
test ranking_precedence_survives_the_checked_delta_roundtrip ... ok
test reusable_row_type_belongs_to_the_view_slice_it_supplies ... ok
test switching_equal_row_shapes_retains_independent_residual_coverage ... ok
test served_view_change_reaches_its_openapi_artifact ... ok
test reusable_row_invariant_change_reaches_its_openapi_artifact ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/revision_pair.rs (target/debug/deps/revision_pair-6384e25d4d698c13)

running 11 tests
test a_revision_compared_with_itself_reports_nothing ... ok
test removing_an_enum_variant_narrows_the_type_that_accepted_it ... ok
test nothing_the_after_revision_only_rewrote_reaches_the_delta ... ok
test taking_a_command_from_an_actor_narrows_what_the_system_permits ... ok
test rewriting_an_entitys_invariant_is_changed_and_quotes_both_statements ... ok
test granting_a_command_to_an_actor_widens_what_the_system_permits ... ok
test the_delta_survives_being_written_and_read_back ... ok
test two_different_systems_are_refused_rather_than_reported_as_a_rewrite ... ok
test adding_an_enum_variant_widens_the_type_that_accepts_it ... ok
test the_fixture_pair_differs_by_exactly_six_changes ... ok
test rewriting_an_outcomes_when_is_changed_and_renders_both_guards_canonically ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/lib.rs (target/debug/deps/ess_gen-9f7cfaccfa26d347)

running 55 tests
test artifact::tests::portable_artifacts_refuse_escape_and_platform_aliases ... ok
test artifact::tests::a_destination_set_rejects_duplicates_case_aliases_and_file_parents_in_any_order ... ok
test authored::tests::a_fence_keeps_its_language_and_loses_its_trailing_newline ... ok
test authored::tests::a_link_an_adopter_wrote_stays_theirs ... ok
test authored::tests::a_heading_becomes_a_section_with_an_anchor ... ok
test authored::tests::a_leading_title_becomes_the_page_title_and_not_a_second_heading ... ok
test authored::tests::a_list_becomes_items_and_a_quote_becomes_a_quote ... ok
test authored::tests::a_paragraph_keeps_its_inline_structure ... ok
test authored::tests::a_table_keeps_its_header_apart_from_its_rows ... ok
test authored::tests::a_top_level_heading_is_demoted_because_the_page_title_is_the_first ... ok
test docs::tests::a_gap_that_ships_says_which_crate_closes_it ... ok
test authored::tests::raw_html_is_dropped_rather_than_passed_through ... ok
test docs::tests::a_heading_and_its_anchor_agree ... ok
test docs::tests::a_lifecycle_renders_as_a_state_diagram_with_its_initial_and_terminal_states_marked ... ok
test docs::tests::a_lifecycle_that_connects_every_pair_says_it_forbids_nothing ... ok
test docs::tests::a_lifecycle_with_one_state_forbids_nothing_rather_than_forbidding_everything ... ok
test docs::tests::a_list_of_three_reads_as_a_person_would_write_it ... ok
test docs::tests::a_plural_of_entity_is_entities ... ok
test docs::tests::a_state_no_transition_touches_is_still_drawn ... ok
test docs::tests::a_transition_from_two_states_draws_one_arrow_from_each ... ok
test document::tests::a_link_names_what_it_points_at_and_never_a_path ... ok
test document::tests::a_page_id_says_how_deep_it_is_so_a_renderer_can_reach_the_root ... ok
test docs::tests::the_page_names_every_transition_the_specification_does_not_permit ... ok
test graph::tests::a_component_group_is_a_dot_cluster_and_graphviz_only_boxes_clusters ... ok
test graph::tests::a_dot_label_keeps_its_parts_on_separate_lines ... ok
test graph::tests::a_mermaid_label_cannot_close_the_quoted_string_it_sits_in ... ok
test html::tests::a_code_block_is_a_code_listing_and_carries_its_language ... ok
test html::tests::a_diagram_is_a_pre_the_renderer_draws_into_and_never_a_code_listing ... ok
test html::tests::a_construct_is_addressed_by_the_section_that_documents_it ... ok
test html::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test document::tests::a_document_round_trips_through_its_own_format ... ok
test html::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test html::tests::a_page_reaches_its_stylesheet_and_its_renderer_from_wherever_it_sits ... ok
test html::tests::a_table_is_a_table_with_a_head_and_a_body ... ok
test html::tests::an_adopters_front_page_goes_above_the_index_and_nowhere_else ... ok
test markdown::tests::a_diagram_is_a_fenced_mermaid_block ... ok
test markdown::tests::a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip ... ok
test markdown::tests::a_link_is_addressed_from_the_page_it_is_written_on ... ok
test html::tests::markup_in_text_never_reaches_the_browser_as_markup ... ok
test markdown::tests::a_quotation_marks_every_line_it_covers ... ok
test schema::types::tests::a_union_branch_pins_its_tag_so_exactly_one_branch_can_match ... ok
test schema::types::tests::an_optional_outside_a_field_gains_a_null_branch_because_a_list_element_cannot_be_absent ... ok
test markdown::tests::a_table_is_written_with_the_separator_a_reader_expects ... ok
test markdown::tests::a_section_flattens_into_the_stream_and_its_children_follow_it ... ok
test schema::types::tests::a_decimal_is_written_as_an_exact_string_because_a_json_number_is_read_as_a_float ... ok
test schema::types::tests::a_union_tagged_value_moves_its_payload_aside_rather_than_colliding_with_the_tag ... ok
test schema::types::tests::an_integer_key_is_constrained_to_the_text_an_integer_is_spelt_with ... ok
test schema::types::tests::a_reference_is_a_pointer_into_the_defs_of_the_document_holding_it ... ok
test schema::types::tests::a_string_keyed_map_publishes_no_property_name_rule_that_checks_nothing ... ok
test schema::types::tests::a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test html::tests::the_sidebar_groups_the_nested_pages_and_marks_the_page_the_reader_is_on ... ok
test html::tests::the_default_style_is_the_stylesheet_that_is_published ... ok
test html::tests::every_emitted_file_says_what_it_was_generated_from ... ok
test html::tests::checked_rendering_validates_deserialized_page_identities_before_map_collection ... ok
test html::tests::checked_rendering_preserves_valid_parent_and_nested_page_bytes ... ok

test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/agreement.rs (target/debug/deps/agreement-4af69d28643a528e)

running 4 tests
test the_agreement_check_compares_the_constructs_the_defect_was_about_rather_than_nothing ... ok
test every_keyword_the_projections_publish_is_classified_as_an_assertion_or_an_annotation ... ok
test no_projection_collapses_a_newtype_into_the_representation_it_wraps ... ok
test every_projection_publishes_the_same_schema_for_a_construct_more_than_one_of_them_describes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/asyncapi.rs (target/debug/deps/asyncapi-f449a9e3ff824c48)

running 18 tests
test a_payload_refuses_an_undeclared_field_and_spells_absence_by_leaving_it_out_of_required ... ok
test a_collection_says_what_it_holds_and_an_absent_element_is_null_because_it_has_no_key_to_omit ... ok
test a_binding_no_component_handles_still_states_its_failure_policy ... ok
test a_dropped_failure_is_stated_in_prose_and_not_only_in_an_extension ... ok
test a_payload_field_carries_the_grammar_the_model_states_and_not_a_note_naming_it ... ok
test a_union_pins_its_tag_so_exactly_one_branch_matches_rather_than_none_or_both ... ok
test every_event_in_the_billing_example_appears_in_some_document ... ok
test the_channel_and_its_message_say_nothing_about_the_binding ... ok
test an_events_channel_address_is_its_declared_wire_name_or_else_its_qualified_name ... ok
test the_publisher_of_an_event_sees_who_reacts_to_it_and_under_what_failure_policy ... ok
test a_bindings_mapping_and_the_reason_for_its_type_crossing_reach_the_document ... ok
test a_bindings_delivery_and_failure_reach_the_receiving_operation ... ok
test every_document_carries_the_provenance_of_the_model_it_came_from ... ok
test a_document_is_a_valid_asyncapi_three_skeleton ... ok
test every_ref_resolves_inside_the_document_that_holds_it ... ok
test a_document_shows_what_the_component_publishes_and_what_it_reacts_to ... ok
test every_component_gets_one_document_named_after_it ... ok
test regenerating_from_the_same_model_produces_the_same_bytes ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/corpus.rs (target/debug/deps/corpus-b5c508be613e316e)

running 3 tests
test the_gatepass_documentation_is_byte_for_byte_what_is_pinned ... ok
test the_oracle_fixture_documentation_is_byte_for_byte_what_is_pinned ... ok
test the_billing_documentation_is_byte_for_byte_what_is_pinned ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/determinism.rs (target/debug/deps/determinism-82fd72ce337b58c9)

running 2 tests
test the_determinism_scan_sees_code_and_not_prose ... ok
test no_generator_reads_a_clock_or_an_unordered_map ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/docs.rs (target/debug/deps/docs-9d17c089cbb09f3a)

running 32 tests
test a_type_nothing_references_is_flagged_rather_than_left_looking_used ... ok
test a_grant_that_crosses_two_contexts_links_to_the_other_contexts_page ... ok
test a_views_eventual_consistency_reads_differently_from_an_immediate_one ... ok
test an_entitys_identity_reaches_the_page_by_name_and_not_only_by_type ... ok
test an_entitys_absent_transition_is_named_as_a_move_the_specification_does_not_permit ... ok
test a_commands_refusal_branch_is_documented_and_not_only_its_name ... ok
test a_wrong_state_branch_is_documented_with_the_states_the_document_never_lists ... ok
test an_actors_grant_renders_as_an_edge_from_the_actor_to_that_command_in_the_index_graph ... ok
test a_bindings_delivery_and_failure_semantics_are_stated_in_words ... ok
test an_actor_that_may_invoke_nothing_is_still_on_the_page ... ok
test a_declared_conversion_carries_its_reason_everywhere_a_reader_might_start ... ok
test an_empty_gap_allowlist_puts_no_cannot_show_section_on_any_page ... ok
test a_views_filter_reaches_the_page_rather_than_being_silently_dropped ... ok
test a_binding_renders_as_a_flow_and_a_lifecycle_as_a_state_diagram ... ok
test an_outcome_that_changes_an_entity_says_which_instance_and_where_the_identity_is_read ... ok
test an_entitys_invariant_reaches_the_page_as_a_condition_on_every_instance ... ok
test an_outcome_the_input_cannot_decide_says_so_rather_than_claiming_it_is_unreachable ... ok
test checked_site_rejects_deserialized_collisions_with_late_static_assets ... ok
test checked_site_preserves_valid_deserialized_nested_pages_and_every_artifact_byte ... ok
test an_entitys_lifecycle_transitions_reach_the_page_as_arrows ... ok
test a_components_ownership_and_a_workloads_replica_floor_are_both_documented ... ok
test a_type_reached_only_through_an_entitys_field_is_not_called_unreached ... ok
test the_provenance_header_is_a_markdown_comment_a_renderer_can_close ... ok
test an_events_payload_and_an_errors_payload_are_both_documented_field_by_field ... ok
test every_member_of_a_resolved_domain_reaches_the_page_of_the_context_it_belongs_to ... ok
test every_type_kind_reaches_a_page_including_the_tagged_union ... ok
test the_command_that_takes_each_move_reaches_the_page_beside_the_move_itself ... ok
test every_link_between_pages_lands_on_a_page_that_exists_at_the_heading_it_names ... ok
test every_page_says_which_specification_produced_it ... ok
test every_name_the_ir_holds_appears_on_some_page ... ok
test an_outcome_says_what_it_does_to_an_entity_and_a_refusal_says_it_changes_none ... ok
test generating_the_documentation_twice_produces_byte_identical_output ... ok

test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/openapi.rs (target/debug/deps/openapi-84c5c21bb15c13da)

running 35 tests
test a_component_that_accepts_nothing_still_gets_a_document ... ok
test a_command_with_no_input_is_exposed_without_a_body ... ok
test a_command_no_component_accepts_appears_in_no_document ... ok
test a_map_with_a_non_string_key_says_the_key_is_still_a_string ... ok
test a_served_view_declares_its_rows_and_the_consistency_a_caller_gets ... ok
test a_view_is_served_only_where_the_specification_says_something_outside_reads_it ... ok
test every_kind_of_type_the_model_has_projects_into_a_schema ... ok
test a_command_no_actor_names_carries_no_grant_rather_than_a_grant_to_everybody ... ok
test a_command_names_the_actors_permitted_to_invoke_it_and_no_authentication_mechanism ... ok
test a_newtype_stays_a_schema_of_its_own_rather_than_becoming_its_representation ... ok
test a_command_is_exposed_at_its_wire_name_under_its_domains ... ok
test a_command_is_only_ever_a_post ... ok
test a_decimal_is_a_string_because_a_json_number_is_a_float ... ok
test a_refusal_the_input_decides_carries_the_declared_error_payload ... ok
test a_document_is_valid_yaml_with_a_version_an_info_block_and_paths ... ok
test an_outcome_that_emits_says_so_without_claiming_to_return_the_events ... ok
test a_command_with_no_wire_name_is_exposed_under_the_name_the_model_gives_it ... ok
test a_command_no_binding_invokes_carries_no_idempotency_header ... ok
test each_declared_outcome_is_its_own_response_and_no_status_is_invented ... ok
test a_commands_input_becomes_a_closed_object_over_its_declared_fields ... ok
test a_command_a_binding_delivers_at_least_once_requires_an_idempotency_key ... ok
test several_outcomes_on_one_status_stay_distinguishable ... ok
test two_commands_claiming_one_path_both_move_to_their_qualified_names ... ok
test a_refusal_the_subjects_state_decides_is_a_conflict_and_not_a_bad_request ... ok
test every_document_carries_its_provenance_as_a_comment_and_as_data ... ok
test every_component_gets_one_document_named_after_it ... ok
test the_operation_id_is_the_commands_qualified_name ... ok
test an_external_outcome_is_an_upstream_failure_and_not_a_validation_refusal ... ok
test every_document_this_generator_can_produce_is_a_valid_openapi_document ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_schema_the_document_declares_is_pointed_at_by_something ... ok
test regenerating_from_the_same_ir_produces_the_same_bytes ... ok
test the_entities_published_are_exactly_those_of_the_domains_the_component_owns ... ok
test every_schema_a_document_embeds_is_valid_in_the_dialect_openapi_31_declares ... ok
test the_document_a_server_hands_out_is_the_committed_one_in_the_other_dialect ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/provenance.rs (target/debug/deps/provenance-97bf54f63a94402c)

running 18 tests
test a_text_without_both_digests_reads_as_nothing ... ok
test a_damaged_digest_reads_as_nothing ... ok
test a_generator_that_stamps_nothing_cannot_ship_an_artifact - should panic ... ok
test a_whole_model_slice_is_stamped_as_one ... ok
test review_new_reader_refuses_unsupported_profile_without_legacy_fallback ... ok
test a_generator_that_pairs_a_stamp_with_the_wrong_slice_cannot_ship_an_artifact - should panic ... ok
test the_reader_reads_back_every_form_the_writer_emits ... ok
test review_new_reader_requires_envelopes_and_exact_digest_tokens ... ok
test review_every_constructs_digest_has_an_explicit_profile_and_whole_remains_bare ... ok
test the_whole_model_contract_digest_is_not_the_source_digest ... ok
test a_change_no_construct_can_be_named_for_moves_every_contract_digest ... ok
test a_change_outside_an_artifacts_slice_leaves_its_contract_digest_standing ... ok
test review_docs_ir_retains_page_profiles_and_does_not_claim_a_flat_stamp ... ok
test review_conflicting_structured_and_comment_stamps_are_unreadable ... ok
test review_marker_looking_model_content_does_not_override_real_emitted_stamps ... ok
test review_profile_is_read_in_all_emissions_and_old_reader_refuses_ordinary_slices ... ok
test correction_structured_envelopes_require_typed_attribution ... ok
test correction_actual_yaml_requires_complete_matching_paired_attribution ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/relations.rs (target/debug/deps/relations-9e1fbfdeb5e1e1b5)

running 4 tests
test the_committed_openapi_document_is_byte_for_byte_what_the_projection_writes ... ok
test the_openapi_document_states_the_relation_and_links_the_targets_schema ... ok
test the_entity_document_states_the_relation_on_the_property_that_carries_it ... ok
test the_committed_entity_documents_are_byte_for_byte_what_the_schema_projection_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/review_adversary_f01.rs (target/debug/deps/review_adversary_f01-07b39ab4f948a888)

running 3 tests
test complete_comments_do_not_admit_an_unknown_profile_via_body_markers ... ok
test structured_stamp_requires_the_complete_emitted_envelope ... ok
test conflicting_locations_and_every_malformed_profile_are_refused ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/review_adversary_f01_pass2.rs (target/debug/deps/review_adversary_f01_pass2-dba0aa538828ec41)

running 3 tests
test incomplete_authoritative_stamp_cannot_fall_back_to_model_provenance ... ok
test duplicate_digest_aliases_and_duplicate_system_keys_are_unreadable ... ok
test paired_yaml_refuses_mixed_digest_aliases_even_when_hashes_agree ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/schema.rs (target/debug/deps/schema-c872c2e57cbce1cf)

running 27 tests
test a_list_element_may_be_null_where_a_field_may_only_be_absent ... ok
test a_timestamp_and_a_duration_publish_a_format_and_no_pattern_they_could_be_wrong_about ... ok
test a_field_carries_its_own_words_beside_the_reference_to_its_type ... ok
test a_field_is_called_what_the_specification_says_it_is_called_on_the_wire ... ok
test a_bytes_field_refuses_a_string_that_is_not_base64 ... ok
test a_map_is_an_object_whose_keys_are_the_text_its_key_type_is_spelt_with ... ok
test a_map_key_that_is_not_the_text_its_key_type_is_spelt_with_is_refused ... ok
test an_optional_field_may_be_absent_and_a_required_field_may_not ... ok
test a_newtype_keeps_its_name_instead_of_collapsing_into_its_representation ... ok
test a_uuid_newtype_carries_the_format_of_what_it_wraps ... ok
test a_newtype_over_a_string_publishes_no_constraint_the_specification_never_stated ... ok
test an_error_that_carries_nothing_accepts_an_empty_object_and_nothing_else ... ok
test an_invariant_travels_with_the_type_and_says_it_is_not_a_constraint ... ok
test an_amount_is_written_as_an_exact_decimal_string_and_a_float_is_refused ... ok
test an_event_payload_accepts_what_the_specification_says_it_carries ... ok
test a_tagged_union_round_trips_because_every_branch_pins_its_tag ... ok
test every_command_input_event_payload_error_payload_and_named_type_gets_a_schema ... ok
test a_uuid_is_refused_unless_it_is_the_canonical_hyphenated_form ... ok
test a_decimal_amount_is_refused_when_it_is_not_written_the_way_the_pattern_says ... ok
test a_command_input_accepts_a_filled_instance_and_refuses_a_misspelt_field ... ok
test every_reference_resolves_inside_the_document_that_makes_it ... ok
test every_message_accepts_an_instance_of_itself_and_refuses_one_that_is_wrong ... ok
test every_artifact_is_a_json_schema_document_declaring_the_dialect_it_is_written_in ... ok
test every_schema_says_which_specification_it_came_from ... ok
test every_published_document_is_a_valid_json_schema_in_the_dialect_it_declares ... ok
test no_schema_uses_a_keyword_outside_the_set_this_projection_publishes ... ok
test generation_is_byte_identical_between_runs ... ok

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s

     Running unittests src/lib.rs (target/debug/deps/ess_synth-38a9c8cb806bed12)

running 8 tests
test go::name::tests::a_fragment_keeps_every_segment_because_identifiers_are_joined_from_them ... ok
test go::name::tests::a_marker_method_is_unexported_which_is_what_seals_the_interface ... ok
test go::name::tests::a_nested_declaration_becomes_one_identifier ... ok
test go::name::tests::a_package_name_that_would_shadow_a_predeclared_identifier_is_repaired ... ok
test rust::name::tests::a_field_the_specification_may_call_type_is_escaped_rather_than_broken ... ok
test rust::name::tests::a_pascal_case_transition_name_becomes_a_method ... ok
test rust::name::tests::a_kebab_case_outcome_becomes_a_variant ... ok
test rust::name::tests::a_nested_declaration_becomes_one_identifier ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/clap.rs (target/debug/deps/clap-109b221b0c9c3274)

running 10 tests
test a_specification_declaring_no_command_line_emits_no_verbs ... ok
test a_placed_view_becomes_a_verb ... ok
test a_string_field_offers_no_values ... ok
test the_tree_carries_the_declared_binary_and_its_groups ... ok
test every_placed_word_is_an_obligation ... ok
test an_enum_typed_field_carries_its_whole_closed_set ... ok
test the_binary_generates_its_own_completions ... ok
test the_manifest_names_the_binary_the_declaration_names ... ok
test review_second_pass_actual_clap_manifest_retains_complete_comment_admission ... ok
test the_emission_is_deterministic ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/go.rs (target/debug/deps/go-408e408e0985c694)

running 19 tests
test a_map_keyed_by_bytes_is_refused_at_the_target_stage_and_never_emitted ... ok
test an_owed_crossing_gets_its_own_package_because_go_refuses_an_import_cycle ... ok
test an_owed_transformation_and_a_retry_policy_are_emitted_the_way_the_binding_declares_them ... ok
test two_seams_of_one_component_that_derive_one_method_name_are_refused_not_renamed ... ok
test no_go_source_uses_a_tab_free_indent_or_a_trailing_space ... ok
test a_newtype_is_a_guarded_struct_and_never_a_defined_string ... ok
test a_closed_set_is_sealed_by_an_unexported_marker_so_no_other_package_can_join_it ... ok
test an_obligation_is_an_interface_and_a_stub_that_returns_a_value_never_a_panic ... ok
test the_generated_transformation_reads_the_event_through_the_declared_crossing ... ok
test every_weakening_is_visible_in_the_generated_source_and_not_only_in_the_report ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test a_command_outcome_keeps_the_refusal_beside_the_success ... ok
test the_plans_obligations_and_the_modules_stubs_are_the_same_list ... ok
test an_illegal_transition_is_a_method_that_does_not_exist ... ok
test the_transport_is_the_one_the_billing_binding_requires ... ok
test refinement_answers_ok_because_a_sealed_interfaces_zero_value_names_no_state ... ok
test emitting_twice_is_byte_identical ... ok
test the_plan_is_byte_identical_in_both_targets_trees ... ok
test the_rust_target_reports_nothing_and_the_go_target_reports_its_weakenings ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/http.rs (target/debug/deps/http-1f24898b1897b081)

running 9 tests
test a_browser_cannot_bind_a_socket_and_says_so_rather_than_emitting_one ... ok
test the_routes_a_server_answers_are_the_routes_the_contract_declares ... ok
test a_specification_that_says_nothing_about_reach_gets_no_server_at_all ... ok
test both_applications_carry_the_same_startup_record_outside_the_runtime_they_append ... ok
test the_plan_is_byte_identical_in_both_trees_of_the_demonstration ... ok
test review_http_payloads_use_slice_profiles_while_neutral_plans_stay_frozen ... ok
test the_served_contract_is_the_document_the_projection_publishes ... ok
test emitting_a_served_surface_twice_is_byte_identical ... ok
test correction_actual_cargo_manifests_keep_their_comment_provenance ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/relations.rs (target/debug/deps/relations-0da2273dd262509e)

running 2 tests
test the_committed_rust_module_is_byte_for_byte_what_the_projection_writes ... ok
test the_generated_data_struct_says_what_the_field_carrying_a_relation_means ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/synthesis.rs (target/debug/deps/synthesis-f193ff4f35681316)

running 29 tests
test a_domain_named_primitives_cannot_shadow_the_representation_module ... ok
test a_domain_named_obligation_cannot_shadow_the_refusal_module ... ok
test a_component_named_like_a_reserved_package_is_renamed_by_rule ... ok
test colliding_domain_modules_are_renamed_by_rule_not_by_luck ... ok
test colliding_event_names_become_full_name_variants_by_rule_not_by_luck ... ok
test a_binding_whose_command_no_component_accepts_is_refused_never_guessed ... ok
test a_mapping_through_a_non_mechanical_crossing_makes_the_transformation_an_obligation ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test a_view_query_obligation_carries_filter_and_consistency ... ok
test every_construct_of_the_specification_appears_in_the_plan ... ok
test grants_are_refused_rather_than_owed ... ok
test the_billing_plan_counts_are_pinned ... ok
test the_billing_binding_is_generated_where_determined_and_owed_where_not ... ok
test a_mechanical_conversion_is_generated_and_any_other_declared_crossing_is_owed ... ok
test the_billing_plan_gives_every_capability_exactly_one_disposition ... ok
test a_component_port_is_typed_against_the_generated_types ... ok
test send_email_behaviour_is_owed_with_the_specifications_own_cause ... ok
test a_command_outcome_enum_keeps_the_refusal_beside_the_success ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test two_components_accepting_one_command_is_refused_naming_both ... ok
test only_the_initial_state_can_be_constructed ... ok
test a_stub_refuses_with_a_value_never_a_panic_and_never_a_todo ... ok
test newtypes_stay_distinct_and_the_declared_crossing_is_the_only_bridge ... ok
test the_plans_obligations_and_the_workspaces_stubs_are_the_same_list ... ok
test the_legal_transitions_are_the_whole_transition_api ... ok
test the_plan_never_names_the_emission_language ... ok
test the_transport_is_the_one_the_billing_binding_requires ... ok
test the_transport_records_its_invocations_and_can_deliver_an_occurrence_twice ... ok
test emitting_twice_is_byte_identical ... ok

test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/web.rs (target/debug/deps/web-1cb70fdbab20ef2a)

running 17 tests
test a_command_no_component_accepts_is_refused_at_the_target_stage_and_gets_no_form ... ok
test a_list_and_a_map_cross_as_the_shapes_json_already_has ... ok
test an_absent_optional_field_is_omitted_rather_than_sent_as_null ... ok
test the_catalogue_carries_the_lifecycle_and_says_where_instances_can_be_observed ... ok
test the_page_names_no_construct_of_the_specification_it_was_generated_from ... ok
test a_tagged_union_crosses_where_the_published_schema_says_its_payload_sits ... ok
test the_committed_tree_holds_no_compiled_module ... ok
test every_weakening_is_visible_in_the_generated_source_and_not_only_in_the_report ... ok
test the_web_target_reports_six_weakenings_and_refuses_nothing_of_billing ... ok
test the_bridge_names_no_realization_and_installs_none ... ok
test every_artifact_names_its_specification_and_the_verb_that_rewrites_it ... ok
test the_catalogue_carries_every_command_with_its_typed_input_and_every_declared_outcome ... ok
test the_public_browser_catalog_is_the_web_targets_exact_document ... ok
test every_generated_type_crosses_the_boundary_in_both_directions ... ok
test emitting_twice_is_byte_identical ... ok
test the_plan_is_byte_identical_in_all_three_targets_trees ... ok
test the_bridge_takes_no_dependency_because_the_gate_reaches_no_network ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

   Doc-tests ess_compiler

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_diff

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_gen

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_synth

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


```

### correction2-fmt-1

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo fmt -p ess-compiler -p ess-diff -p ess-gen -p ess-synth -- --check
```

Exit 0. Full preserved output:

```text

```

## 5. Measured regeneration, frozen contracts and limits

The exact owning generator commands were rerun into NEW correction2 stdout maps; no old implementation/review/correction/regeneration record was overwritten. The complete selected maps were compared with current committed counterparts before retaining anything. The existing projection lane excludes its three site/assets constant assets (the same owning xtask exclusion established in the original measurement); corpus lanes compare their docs only. HTTP comparison covers every emitted artifact, with only the four preauthorized payload paths permitted to differ.

97 committed counterparts were compared; 95 are byte-identical and two differ. The unchanged counts are 45 Billing projections, 17 corpus documents and 33 HTTP artifacts. All neutral plans and both served docs payloads are among those unchanged files. No new/deleted generated path and no unexpected scope. Only two files were retained, and parsed before/after JSON differs solely at /info/x-ess-provenance/contract_digest. Each file remains 28821 bytes. The suffix now equals the raw whole-model hash for Gatepass because this corrected component slice covers that input, but the explicit slice-sha256/2 profile remains part of identity and is not removed.

Inventory summary:

```text
projections-billing -> generated: emitted 48, compared 45, changed 0, excluded 3
projections-billing -> crates/generate/ess-gen/tests/corpus/billing: emitted 48, compared 6, changed 0, excluded 42
docs-gatepass -> crates/generate/ess-gen/tests/corpus/gatepass: emitted 5, compared 5, changed 0, excluded 0
docs-oracle -> crates/generate/ess-gen/tests/corpus/oracle-fixture: emitted 6, compared 6, changed 0, excluded 0
http-rust -> generated/rust/gatepass: emitted 20, compared 20, changed 1, excluded 0
http-go -> generated/go/gatepass: emitted 15, compared 15, changed 1, excluded 0
Total compared 97
Total changed 2
Unexpected []
generated/rust/gatepass/crates/gatepass-server/src/pass-service.openapi.json
generated/go/gatepass/server/pass-service.openapi.json
```

Exact retained path/hash/size manifest:

```json
[
  {
    "path": "generated/rust/gatepass/crates/gatepass-server/src/pass-service.openapi.json",
    "source": "correction2-http-rust.json",
    "key": "crates/gatepass-server/src/pass-service.openapi.json",
    "before_sha256": "e21d2825fecb42311f77adee5bbefe35d185f93122c6b1dc38d8afafae461dff",
    "after_sha256": "d4b07f0af6ce720c3fa87e22a62b51dc26288041f57afd42879a1112b526ef19",
    "before_bytes": 28821,
    "after_bytes": 28821,
    "changed": true
  },
  {
    "path": "generated/go/gatepass/server/pass-service.openapi.json",
    "source": "correction2-http-go.json",
    "key": "server/pass-service.openapi.json",
    "before_sha256": "e21d2825fecb42311f77adee5bbefe35d185f93122c6b1dc38d8afafae461dff",
    "after_sha256": "d4b07f0af6ce720c3fa87e22a62b51dc26288041f57afd42879a1112b526ef19",
    "before_bytes": 28821,
    "after_bytes": 28821,
    "changed": true
  }
]
```

Measured JSON difference:

```text
generated/rust/gatepass/crates/gatepass-server/src/pass-service.openapi.json: only /info/x-ess-provenance/contract_digest changes from slice-sha256/2:20853cb8a89aae494e4e0482cf050830b187d6ec3fe16e1df0ad24fc27112701 to slice-sha256/2:e6e58e055d24f8f494dcff274f55e723d967f9d1f9aea16641bb8dacbb71171e
generated/go/gatepass/server/pass-service.openapi.json: only /info/x-ess-provenance/contract_digest changes from slice-sha256/2:20853cb8a89aae494e4e0482cf050830b187d6ec3fe16e1df0ad24fc27112701 to slice-sha256/2:e6e58e055d24f8f494dcff274f55e723d967f9d1f9aea16641bb8dacbb71171e
```

Complete pre-write contents/hashes: correction2-regeneration-before.json. All 97 before/after path/hash/size rows: correction2-regeneration-inventory.json. Group counts/exclusions: correction2-regeneration-summary.json. Exact argv: correction2-generator-commands.json. Complete emitted post-change maps: correction2-projections-billing.json, correction2-docs-gatepass.json, correction2-docs-oracle.json, correction2-http-rust.json and correction2-http-go.json. Final verification confirmed every retained/current counterpart matches its inventoried after hash.

Frozen controls: the unchanged 154846-byte suite/4 fixture (SHA256508b6a3d75d6dabd6fa686b67dcb6c7c881374375aaed9dd8921445a5edc894e) regenerates all 30 scenarios including one authored with zero refusals and byte-identical reader roundtrip. Frozen diff/1 bytes and checked legacy writer/version admission pass. source_digest and WholeModel algorithms/fields remain unchanged; existing exact source/whole/index and neutral-plan assertions pass. The generated differences intentionally change the two profiled slice suffixes only; no all-sliced-bytes-preserved claim is made.

Execution limits: actual library generation, stamp reading, directly seeded slice identities and real GeneratedTree impact were executed. No live service, generated application runtime, deployed reader or new SDK candidate was run by this implementor. The current CLI impact route supplies no GeneratedTree; this report does not claim a CLI committed-tree verification lane. There is still no new impact reader. Public String/Serde, docs-ir per-page attribution and old substring-reader limitations remain unchanged. The binding Atlas decision/revision and publication sequencing are coordinator-provided state, not a sibling action or rollout performed here.

At handoff the coordinator reports the clarified Atlas decision published at 7b67e8e2437ec9956135930435875a8a76139c3f, with its managed authority at that exact advertised main; no new ESS default is published. The coordinator reports 115 Atlas Rust cases and catalog/docs/Pages/projection/brand checks passed, with the same three baseline workspace-fence failures retained. These are attributed coordination facts, not checks executed in this ESS correction.

The coordinator owns final personal verification, integration task check/site build, remaining Atlas coordination, SDK coordination, ESS publication and cleanup. No third full attack, new broad feature/fuzzer, primitive or conformance migration, AEP dependency, source_digest reinterpretation, manifest pin or outside-scope patch. All own Cargo sessions have completed. Required cache wrapper/socket, offline compact profiles and own target/TMPDIR remained in force; no CARGO_TARGET_DIR or cache lifecycle action. Token/duration accounting is unavailable and not invented.

Disk: 115597156352 available bytes before builds; 109786206208 at report assembly; required reserve 8589934592.

## 6. Every path written outside the assigned worktree

None. Production/tests/design/generated changes are the five assigned paths above. Maps, logs, exit files, measurement snapshots and report remain under target/review-boundaries-4; all build/temp files use this tree's target. The coordinator's existing compiler-cache socket was used as required, without lifecycle/configuration commands. No Git mutation, AEP/store write, sibling file, external message, worktree cleanup or outside-tree patch. Writes are relinquished with this handoff.

## Appendix: owning generator and formatter commands

### correction2-projections-billing

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo run --offline --quiet --locked -p ess-cli --bin ess -- generate --format json --path examples/billing
```

Exit 0. Full preserved output (stderr; full stdout map is correction2-projections-billing.json):

```text

```

### correction2-docs-gatepass

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true target/debug/ess generate --kind docs --format json --path examples/gatepass
```

Exit 0. Full preserved output (stderr; full stdout map is correction2-docs-gatepass.json):

```text

```

### correction2-docs-oracle

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true target/debug/ess generate --kind docs --format json --path examples/oracle-fixture
```

Exit 0. Full preserved output (stderr; full stdout map is correction2-docs-oracle.json):

```text

```

### correction2-http-rust

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true target/debug/ess synthesize --target rust --format json --path examples/gatepass
```

Exit 0. Full preserved output (stderr; full stdout map is correction2-http-rust.json):

```text

```

### correction2-http-go

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true target/debug/ess synthesize --target go --format json --path examples/gatepass
```

Exit 0. Full preserved output (stderr; full stdout map is correction2-http-go.json):

```text

```

### correction2-format-apply-1

```sh
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo fmt -p ess-compiler -p ess-diff -p ess-gen -p ess-synth
```

Exit 0. Full preserved output:

```text

```
