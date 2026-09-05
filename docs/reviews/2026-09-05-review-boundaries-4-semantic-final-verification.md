# Semantic final correction verification

The coordinator inspected the complete five-file correction diff and verified the retained
assertions on clean commit `21f274476c644be35bd9c7905d72211ae5f8682c`. This is bounded
verification after two full adversarial passes, not a third full review.

The compiler now selects network-exposed views using the same reach and owned-domain
selection as the actual HTTP generator. A separate row-shape relation retains the complete
named row type, including constraints that copied fields do not represent. The binding
impact/3 vocabulary has 26 relations, consistent with Atlas decision commit
`7b67e8e2437ec9956135930435875a8a76139c3f`. The correction adds two graph boundary cases;
no earlier assertion or adversary test was removed, weakened, or re-pinned.

The coordinator independently parsed both changed generated OpenAPI JSON documents
against their previous committed bytes. Each has exactly one changed leaf:
`/info/x-ess-provenance/contract_digest`. All other JSON values match. The implementor's
97-file regeneration inventory records exactly these two changed files and 95 unchanged
files, including the neutral plans. The full correction report preserves commands and
hashes; the source and whole-model identity controls remain frozen.

On the exact clean correction commit, the coordinator executed all five pass-2 diff
cases, both new graph boundary cases, and all ten pass-1 adversary cases. All 17 passed,
zero failed, zero ignored. The implementor's separate full four-package run records
507 passed, zero failed, zero ignored; formatter and strict Clippy returned 0. SDK
candidate compatibility and the complete integration gate remain pending.

The two pass-2 findings are corrected. Their original provenance remains undecided
because these witnesses were not executed against the opening base. This verification
does not claim a CLI generated-tree checker, generated clap compilation, SDK pin
publication, or conformance coverage beyond the retained cases.

Completed at: 2026-09-05T17:45:02.284902+00:00

## Exact coordinator results

```json
{
  "subject": "21f274476c644be35bd9c7905d72211ae5f8682c",
  "results": [
    {
      "name": "pass2",
      "exit_code": 0,
      "seconds": 9.345151896995958,
      "summaries": [
        [
          "ok",
          "5",
          "0",
          "0"
        ]
      ]
    },
    {
      "name": "graph",
      "exit_code": 0,
      "seconds": 0.3862055119825527,
      "summaries": [
        [
          "ok",
          "2",
          "0",
          "0"
        ]
      ]
    },
    {
      "name": "pass1",
      "exit_code": 0,
      "seconds": 0.7522172420285642,
      "summaries": [
        [
          "ok",
          "7",
          "0",
          "0"
        ],
        [
          "ok",
          "3",
          "0",
          "0"
        ]
      ]
    }
  ]
}
```

## pass2

```json
{
  "subject": "21f274476c644be35bd9c7905d72211ae5f8682c",
  "argv": [
    "cargo",
    "test",
    "--offline",
    "--locked",
    "-p",
    "ess-diff",
    "--test",
    "review_adversary_f01_pass2"
  ],
  "env": {
    "TMPDIR": "/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/target",
    "RUSTC_WRAPPER": "/usr/bin/sccache",
    "SCCACHE_SERVER_UDS": "/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock",
    "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_DEBUG": "0",
    "CARGO_CACHE_RUSTC_INFO": "0",
    "CARGO_NET_OFFLINE": "true"
  }
}
```

```text
   Compiling ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/specify/ess-compiler)
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/generate/ess-gen)
   Compiling ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-conformance)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 9.28s
     Running tests/review_adversary_f01_pass2.rs (target/debug/deps/review_adversary_f01_pass2-c2309f25ac1af007)

running 5 tests
test reusable_row_type_belongs_to_the_view_slice_it_supplies ... ok
test ranking_precedence_survives_the_checked_delta_roundtrip ... ok
test switching_equal_row_shapes_retains_independent_residual_coverage ... ok
test served_view_change_reaches_its_openapi_artifact ... ok
test reusable_row_invariant_change_reaches_its_openapi_artifact ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

```

## graph

```json
{
  "subject": "21f274476c644be35bd9c7905d72211ae5f8682c",
  "argv": [
    "cargo",
    "test",
    "--offline",
    "--locked",
    "-p",
    "ess-diff",
    "--test",
    "graph",
    "correction2_"
  ],
  "env": {
    "TMPDIR": "/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/target",
    "RUSTC_WRAPPER": "/usr/bin/sccache",
    "SCCACHE_SERVER_UDS": "/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock",
    "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_DEBUG": "0",
    "CARGO_CACHE_RUSTC_INFO": "0",
    "CARGO_NET_OFFLINE": "true"
  }
}
```

```text
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/crates/verify/ess-diff)
    Finished `test` profile [unoptimized] target(s) in 0.36s
     Running tests/graph.rs (target/debug/deps/graph-428d50882447a7e7)

running 2 tests
test correction2_row_shape_is_a_distinct_dependency_and_survives_graph_union ... ok
test correction2_network_exposure_matches_actual_routes_and_owned_domains ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s

```

## pass1

```json
{
  "subject": "21f274476c644be35bd9c7905d72211ae5f8682c",
  "argv": [
    "cargo",
    "test",
    "--offline",
    "--locked",
    "-p",
    "ess-diff",
    "-p",
    "ess-gen",
    "--test",
    "review_adversary_f01"
  ],
  "env": {
    "TMPDIR": "/home/timo/.local/state/worktree/trees/b10x/ess/review-semantic-diff-coverage/target",
    "RUSTC_WRAPPER": "/usr/bin/sccache",
    "SCCACHE_SERVER_UDS": "/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock",
    "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_DEBUG": "0",
    "CARGO_CACHE_RUSTC_INFO": "0",
    "CARGO_NET_OFFLINE": "true"
  }
}
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.10s
     Running tests/review_adversary_f01.rs (target/debug/deps/review_adversary_f01-8795d8f802dd31cc)

running 7 tests
test explicit_domain_naming_defaults_remain_semantically_equivalent ... ok
test complete_generated_and_authored_suite_four_bytes_remain_frozen ... ok
test moved_outcome_reference_is_independent_of_a_classified_edit ... ok
test moved_outcome_reference_retains_its_owner ... ok
test relation_delta_versions_refuse_relabeling_and_public_serialize_bypasses ... ok
test ownership_cardinality_invalidates_both_emitted_schema_ends ... ok
test incomplete_schema_stamp_is_owed_by_the_real_impact_reader ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.63s

     Running tests/review_adversary_f01.rs (target/debug/deps/review_adversary_f01-d8564f620078ebe8)

running 3 tests
test complete_comments_do_not_admit_an_unknown_profile_via_body_markers ... ok
test structured_stamp_requires_the_complete_emitted_envelope ... ok
test conflicting_locations_and_every_malformed_profile_are_refused ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```
