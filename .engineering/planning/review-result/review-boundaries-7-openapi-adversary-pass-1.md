---
format: aep.planning-md/1
id: review-result:review-boundaries-7-openapi-adversary-pass-1
kind: review-result
status: active
title: OpenAPI accounting first independent review
relations:
- reviews: story:review-openapi-semantic-accounting
revision: 1
---
unit: story:review-openapi-semantic-accounting, adversary pass 1 resumed, 633e1d9839dc0e8dae071e955f8e8af540c8a685 plus the two unchanged new test files
verdict: CONFIRMED
cases: executed 150→163, red 4
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: no deliberate files; incidental shared cache effects at two paths detailed in part 6
needs-coordinator: route the single closed-admission blocker to the implementor; preserve the legacy reader while fixing the new boundary

1. Tests-only diff

```console
$ git --no-pager diff --stat
```

The tracked diff is empty. Both pass-1 files remain untracked, so the read-only no-index comparison below includes their actual changes without touching the Git index:

```console
$ git --no-pager diff --no-index --stat /dev/null crates/generate/ess-openapi/tests/adversary_pass1.rs
 .../generate/ess-openapi/tests/adversary_pass1.rs  | 235 +++++++++++++++++++++
 1 file changed, 235 insertions(+)
$ git --no-pager diff --no-index --stat /dev/null crates/edge/ess-cli/tests/openapi_adversary_pass1.rs
 .../edge/ess-cli/tests/openapi_adversary_pass1.rs  | 101 +++++++++++++++++++++
 1 file changed, 101 insertions(+)
```

Each no-index command exits 1 because an added file differs from /dev/null. No implementation, old test, documentation, manifest, lockfile, planning, Git index/ref or lifecycle write was made. Final status:

```text
?? crates/edge/ess-cli/tests/openapi_adversary_pass1.rs
?? crates/generate/ess-openapi/tests/adversary_pass1.rs
```

2. Cases, written before focused execution

This is the continuation of the interrupted first pass, not another attack. The two test files and focused-01 raw argv/stdout/stderr/status were recovered byte-for-byte and their SHA-256 checks still pass. The original focused-01 run compiled and executed one failing assertion; it was a semantic admission failure, not a setup failure. No test source was changed during resumption. The current installed 0.8.0 adversary charter was read in full; the original unit/pass brief, story, accepted binding, complete implementation diff, its callers, and tests were inspected. The current coordinator-supplied Atlas authority was read at /home/timo/.local/state/worktree/trees/b10x/atlas/wt-b2081c2f6924.

All 13 executable additions received a case-alone focused run before the full suite. The library uses loops for sites and encodings; these assertions are not counted as separate test cases.

| Focus | File and case | Assertion | Now |
|---|---|---|---|
| 01 | crates/generate/ess-openapi/tests/adversary_pass1.rs:78, closed_import_rejects_unknown_integer_variant_fields | Reject an unknown field inside integer variants at component/property/request/response/array-item sites, JSON and YAML | red |
| 02 | crates/generate/ess-openapi/tests/adversary_pass1.rs:83, closed_import_rejects_unknown_number_variant_fields | Same closed-boundary requirement for number variants | red |
| 03 | crates/generate/ess-openapi/tests/adversary_pass1.rs:88, closed_import_rejects_unknown_boolean_variant_fields | Same closed-boundary requirement for boolean variants | red |
| 04 | crates/generate/ess-openapi/tests/adversary_pass1.rs, nonunit_schema_variants_reject_unknown_fields | Unknown fields also refuse on reference/string/array/object variants | green |
| 05 | crates/generate/ess-openapi/tests/adversary_pass1.rs, unconsumed_variant_keywords_remain_gaps_after_replay | Every unconsumed keyword retains a site-specific gap after reload and prevents projection | green |
| 06 | crates/generate/ess-openapi/tests/adversary_pass1.rs, contradictory_string_enum_and_const_survive_both_boundaries | Both string constraints survive reload and projection, even when contradictory | green |
| 07 | crates/generate/ess-openapi/tests/adversary_pass1.rs, unsupported_items_cannot_disappear_at_nested_or_message_sites | Boolean, untyped, or type-array items explicitly refuse at each site | green |
| 08 | crates/generate/ess-openapi/tests/adversary_pass1.rs, schema_resource_and_dialect_features_refuse_but_annotation_literals_do_not | Resource/dialect changes refuse; annotation literal objects are not interpreted as schemas | green |
| 09 | crates/generate/ess-openapi/tests/adversary_pass1.rs, reference_escape_identity_is_not_decoded_twice | Escaped reference targets retain exact identity and separate unresolved sites | green |
| 10 | crates/generate/ess-openapi/tests/adversary_pass1.rs, accounting_order_duplicates_codes_and_missing_arrays_cannot_be_normalized_away | Reordering, duplicate records, changed codes, and omitted arrays refuse at replay | green |
| 11 | crates/generate/ess-openapi/tests/adversary_pass1.rs, duplicate_escaped_map_keys_are_rejected_before_replay | Escaped duplicate JSON keys refuse before typed decoding | green |
| 12 | crates/edge/ess-cli/tests/openapi_adversary_pass1.rs:44, cli_rejects_unknown_unit_fields_before_any_projection_output | An actual persisted import with an unknown integer-variant field refuses before either CLI spelling writes output | red |
| 13 | crates/edge/ess-cli/tests/openapi_adversary_pass1.rs:80, cli_rejects_duplicate_keys_and_tampered_accounting_before_output | Digest tampering and duplicate keys refuse and preserve output through the CLI | green |

The three library failures share the assertion at adversary_pass1.rs:74. Each measures ten admissions: five sites × two encodings. Their raw output shows the unknown field erased and checked projection accepted. The CLI failure at openapi_adversary_pass1.rs:76 measures six calls: two spellings × existing/absent/stdout destinations. All six return 0; existing files are overwritten, absent files created, and stdout receives 196 bytes. These are four executable red cases exposing one defect.

For focused-02 through focused-13 and suite-01, CARGO_TARGET_DIR was unset and this environment was used. The original focused-01 argv already records its complete environment. The existing original sccache socket was present and reused; no daemon was started. All Cargo builds used the pass's own existing target below assigned scratch.

```text
CARGO_BUILD_JOBS=4
CARGO_CACHE_RUSTC_INFO=0
CARGO_INCREMENTAL=0
CARGO_NET_OFFLINE=true
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
GOCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/go-cache
GOMODCACHE=/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/go-mod-cache
GOTMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1
RUSTC_WRAPPER=/usr/bin/sccache
SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock
TMPDIR=/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1
```

Verbatim focused argv and separate streams follow. Stream labels are presentation; captured contents are unchanged. Each status is the actual Cargo exit, not the enclosing recording shell's status.

focused-01 argv:

```text
env -u CARGO_TARGET_DIR TMPDIR='/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1' GOTMPDIR='/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1' GOCACHE='/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/go-cache' GOMODCACHE='/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/go-mod-cache' RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo test --locked --offline --target-dir '/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/cargo-target' -p ess-openapi --test adversary_pass1 closed_import_rejects_unknown_integer_variant_fields -- --exact --nocapture
```

stdout:

```text

running 1 test
ADMITTED json /interface/types/A/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/types/A/properties/p~0~1/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/properties/p~0~1/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/operations/probe/request/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/operations/probe/request/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/operations/probe/responses/200/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/operations/probe/responses/200/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/types/A/items/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/items/unexpected_constraint; erased=true; checked_projection_ok=true
test closed_import_rejects_unknown_integer_variant_fields ... FAILED

failures:

failures:
    closed_import_rejects_unknown_integer_variant_fields

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.01s

```

stderr:

```text
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.24
   Compiling serde_core v1.0.229
   Compiling typenum v1.20.1
   Compiling zmij v1.0.23
   Compiling serde v1.0.229
   Compiling hybrid-array v0.4.14
   Compiling hashbrown v0.17.1
   Compiling crypto-common v0.2.2
   Compiling block-buffer v0.12.1
   Compiling const-oid v0.10.2
   Compiling serde_json v1.0.151
   Compiling itoa v1.0.18
   Compiling equivalent v1.0.2
   Compiling syn v3.0.4
   Compiling digest v0.11.3
   Compiling indexmap v2.14.1
   Compiling ryu v1.0.23
   Compiling serde_derive v1.0.229
   Compiling cpufeatures v0.3.1
   Compiling unsafe-libyaml v0.2.11
   Compiling cfg-if v1.0.4
   Compiling memchr v2.8.3
   Compiling sha2 v0.11.0
   Compiling serde_yaml v0.9.34+deprecated
   Compiling ess-openapi v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/generate/ess-openapi)
    Finished `test` profile [unoptimized] target(s) in 4.19s
     Running tests/adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/adversary_pass1-7267d3dd9e17cfe9)

thread 'closed_import_rejects_unknown_integer_variant_fields' (2678110) panicked at crates/generate/ess-openapi/tests/adversary_pass1.rs:74:5:
closed import envelope admitted unknown fields in integer unit variants: ["json /interface/types/A", "yaml /interface/types/A", "json /interface/types/A/properties/p~0~1", "yaml /interface/types/A/properties/p~0~1", "json /interface/operations/probe/request/schema", "yaml /interface/operations/probe/request/schema", "json /interface/operations/probe/responses/200/schema", "yaml /interface/operations/probe/responses/200/schema", "json /interface/types/A/items", "yaml /interface/types/A/items"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: test failed, to rerun pass `-p ess-openapi --test adversary_pass1`
```

exit status: 101

focused-02 argv:

```text
cargo test --locked --offline --target-dir /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/cargo-target -p ess-openapi --test adversary_pass1 closed_import_rejects_unknown_number_variant_fields -- --exact --nocapture 
```

stdout:

```text

running 1 test
ADMITTED json /interface/types/A/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/types/A/properties/p~0~1/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/properties/p~0~1/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/operations/probe/request/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/operations/probe/request/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/operations/probe/responses/200/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/operations/probe/responses/200/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/types/A/items/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/items/unexpected_constraint; erased=true; checked_projection_ok=true
test closed_import_rejects_unknown_number_variant_fields ... FAILED

failures:

failures:
    closed_import_rejects_unknown_number_variant_fields

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.01s

```

stderr:

```text
    Finished `test` profile [unoptimized] target(s) in 0.27s
     Running tests/adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/adversary_pass1-7267d3dd9e17cfe9)

thread 'closed_import_rejects_unknown_number_variant_fields' (3322290) panicked at crates/generate/ess-openapi/tests/adversary_pass1.rs:74:5:
closed import envelope admitted unknown fields in number unit variants: ["json /interface/types/A", "yaml /interface/types/A", "json /interface/types/A/properties/p~0~1", "yaml /interface/types/A/properties/p~0~1", "json /interface/operations/probe/request/schema", "yaml /interface/operations/probe/request/schema", "json /interface/operations/probe/responses/200/schema", "yaml /interface/operations/probe/responses/200/schema", "json /interface/types/A/items", "yaml /interface/types/A/items"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: test failed, to rerun pass `-p ess-openapi --test adversary_pass1`
```

exit status: 101

focused-03 argv:

```text
cargo test --locked --offline --target-dir /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/cargo-target -p ess-openapi --test adversary_pass1 closed_import_rejects_unknown_boolean_variant_fields -- --exact --nocapture 
```

stdout:

```text

running 1 test
ADMITTED json /interface/types/A/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/types/A/properties/p~0~1/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/properties/p~0~1/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/operations/probe/request/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/operations/probe/request/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/operations/probe/responses/200/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/operations/probe/responses/200/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/types/A/items/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/items/unexpected_constraint; erased=true; checked_projection_ok=true
test closed_import_rejects_unknown_boolean_variant_fields ... FAILED

failures:

failures:
    closed_import_rejects_unknown_boolean_variant_fields

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.01s

```

stderr:

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/adversary_pass1-7267d3dd9e17cfe9)

thread 'closed_import_rejects_unknown_boolean_variant_fields' (3322321) panicked at crates/generate/ess-openapi/tests/adversary_pass1.rs:74:5:
closed import envelope admitted unknown fields in boolean unit variants: ["json /interface/types/A", "yaml /interface/types/A", "json /interface/types/A/properties/p~0~1", "yaml /interface/types/A/properties/p~0~1", "json /interface/operations/probe/request/schema", "yaml /interface/operations/probe/request/schema", "json /interface/operations/probe/responses/200/schema", "yaml /interface/operations/probe/responses/200/schema", "json /interface/types/A/items", "yaml /interface/types/A/items"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: test failed, to rerun pass `-p ess-openapi --test adversary_pass1`
```

exit status: 101

focused-04 argv:

```text
cargo test --locked --offline --target-dir /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/cargo-target -p ess-openapi --test adversary_pass1 nonunit_schema_variants_reject_unknown_fields -- --exact --nocapture 
```

stdout:

```text

running 1 test
test nonunit_schema_variants_reject_unknown_fields ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

```

stderr:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/adversary_pass1-7267d3dd9e17cfe9)
```

exit status: 0

focused-05 argv:

```text
cargo test --locked --offline --target-dir /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/cargo-target -p ess-openapi --test adversary_pass1 unconsumed_variant_keywords_remain_gaps_after_replay -- --exact --nocapture 
```

stdout:

```text

running 1 test
test unconsumed_variant_keywords_remain_gaps_after_replay ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.02s

```

stderr:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/adversary_pass1-7267d3dd9e17cfe9)
```

exit status: 0

focused-06 argv:

```text
cargo test --locked --offline --target-dir /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/cargo-target -p ess-openapi --test adversary_pass1 contradictory_string_enum_and_const_survive_both_boundaries -- --exact --nocapture 
```

stdout:

```text

running 1 test
test contradictory_string_enum_and_const_survive_both_boundaries ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

```

stderr:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/adversary_pass1-7267d3dd9e17cfe9)
```

exit status: 0

focused-07 argv:

```text
cargo test --locked --offline --target-dir /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/cargo-target -p ess-openapi --test adversary_pass1 unsupported_items_cannot_disappear_at_nested_or_message_sites -- --exact --nocapture 
```

stdout:

```text

running 1 test
test unsupported_items_cannot_disappear_at_nested_or_message_sites ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

```

stderr:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/adversary_pass1-7267d3dd9e17cfe9)
```

exit status: 0

focused-08 argv:

```text
cargo test --locked --offline --target-dir /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/cargo-target -p ess-openapi --test adversary_pass1 schema_resource_and_dialect_features_refuse_but_annotation_literals_do_not -- --exact --nocapture 
```

stdout:

```text

running 1 test
test schema_resource_and_dialect_features_refuse_but_annotation_literals_do_not ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.01s

```

stderr:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/adversary_pass1-7267d3dd9e17cfe9)
```

exit status: 0

focused-09 argv:

```text
cargo test --locked --offline --target-dir /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/cargo-target -p ess-openapi --test adversary_pass1 reference_escape_identity_is_not_decoded_twice -- --exact --nocapture 
```

stdout:

```text

running 1 test
test reference_escape_identity_is_not_decoded_twice ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

```

stderr:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/adversary_pass1-7267d3dd9e17cfe9)
```

exit status: 0

focused-10 argv:

```text
cargo test --locked --offline --target-dir /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/cargo-target -p ess-openapi --test adversary_pass1 accounting_order_duplicates_codes_and_missing_arrays_cannot_be_normalized_away -- --exact --nocapture 
```

stdout:

```text

running 1 test
test accounting_order_duplicates_codes_and_missing_arrays_cannot_be_normalized_away ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

```

stderr:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/adversary_pass1-7267d3dd9e17cfe9)
```

exit status: 0

focused-11 argv:

```text
cargo test --locked --offline --target-dir /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/cargo-target -p ess-openapi --test adversary_pass1 duplicate_escaped_map_keys_are_rejected_before_replay -- --exact --nocapture 
```

stdout:

```text

running 1 test
test duplicate_escaped_map_keys_are_rejected_before_replay ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

```

stderr:

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/adversary_pass1-7267d3dd9e17cfe9)
```

exit status: 0

focused-12 argv:

```text
cargo test --locked --offline --target-dir /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/cargo-target -p ess-cli --test openapi_adversary_pass1 cli_rejects_unknown_unit_fields_before_any_projection_output -- --exact --nocapture 
```

stdout:

```text

running 1 test
flat existing: exit=Some(0); stdout_bytes=0; destination_preserved=false
flat absent: exit=Some(0); stdout_bytes=0; destination_preserved=false
flat stdout: exit=Some(0); stdout_bytes=196; destination_preserved=false
area existing: exit=Some(0); stdout_bytes=0; destination_preserved=false
area absent: exit=Some(0); stdout_bytes=0; destination_preserved=false
area stdout: exit=Some(0); stdout_bytes=196; destination_preserved=false
test cli_rejects_unknown_unit_fields_before_any_projection_output ... FAILED

failures:

failures:
    cli_rejects_unknown_unit_fields_before_any_projection_output

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.03s

```

stderr:

```text
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling serde_core v1.0.229
   Compiling serde v1.0.229
   Compiling memchr v2.8.3
   Compiling serde_json v1.0.151
   Compiling foldhash v0.2.0
   Compiling allocator-api2 v0.2.21
   Compiling schemars v0.8.22
   Compiling hashbrown v0.17.1
   Compiling thiserror v2.0.20
   Compiling indexmap v2.14.1
   Compiling syn v3.0.4
   Compiling syn v2.0.119
   Compiling dyn-clone v1.0.20
   Compiling autocfg v1.5.1
   Compiling serde_derive_internals v0.29.1
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.20
   Compiling schemars_derive v0.8.22
   Compiling num-traits v0.2.19
   Compiling libc v0.2.189
   Compiling num-integer v0.1.47
   Compiling getrandom v0.3.4
   Compiling heck v0.5.0
   Compiling zerocopy v0.8.56
   Compiling version_check v0.9.5
   Compiling pulldown-cmark v0.13.4
   Compiling ahash v0.8.12
   Compiling num-bigint v0.4.8
   Compiling parking_lot_core v0.9.12
   Compiling pulldown-cmark-escape v0.11.0
   Compiling ref-cast v1.0.27
   Compiling bitflags v2.13.1
   Compiling regex-syntax v0.8.11
   Compiling unicase v2.9.0
   Compiling num-rational v0.4.2
   Compiling num-iter v0.1.46
   Compiling num-complex v0.4.6
   Compiling ref-cast-impl v1.0.27
   Compiling serde_yaml v0.9.34+deprecated
   Compiling aho-corasick v1.1.5
   Compiling smallvec v1.16.0
   Compiling once_cell v1.21.4
   Compiling scopeguard v1.2.0
   Compiling utf8parse v0.2.2
   Compiling lock_api v0.4.14
   Compiling anstyle-parse v1.0.0
   Compiling regex-automata v0.4.18
   Compiling infra-domain v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/infra/infra-domain)
   Compiling num v0.4.3
   Compiling unicode-general-category v1.1.0
   Compiling borrow-or-share v0.2.4
   Compiling is_terminal_polyfill v1.70.2
   Compiling colorchoice v1.0.5
   Compiling bit-vec v0.8.0
   Compiling anstyle v1.0.14
   Compiling anstyle-query v1.1.5
   Compiling bit-set v0.8.0
   Compiling anstream v1.0.0
   Compiling fluent-uri v0.4.1
   Compiling fraction v0.17.0
   Compiling ess-primitives v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/specify/ess-primitives)
   Compiling parking_lot v0.12.5
   Compiling strum_macros v0.28.0
   Compiling micromap v0.3.0
   Compiling outref v0.5.2
   Compiling vsimd v0.8.0
   Compiling bytecount v0.6.9
   Compiling clap_lex v1.1.0
   Compiling percent-encoding v2.3.2
   Compiling num-cmp v0.1.0
   Compiling strsim v0.11.1
   Compiling clap_builder v4.6.6
   Compiling infra-compiler v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/infra/infra-compiler)
   Compiling ess-domain v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/specify/ess-domain)
   Compiling jsonschema-value v0.52.1
   Compiling strum v0.28.0
   Compiling referencing v0.52.1
   Compiling uuid-simd v0.8.0
   Compiling fancy-regex v0.19.0
   Compiling regex v1.13.1
   Compiling email_address v0.2.9
   Compiling jsonschema-regex v0.52.1
   Compiling clap_derive v4.6.4
   Compiling data-encoding v2.11.1
   Compiling anyhow v1.0.104
   Compiling jsonschema v0.52.1
   Compiling clap v4.6.6
   Compiling semver v1.0.28
   Compiling ess-kubernetes v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/infra/ess-kubernetes)
   Compiling infra-analyze v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/infra/infra-analyze)
   Compiling ess-openapi v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/generate/ess-openapi)
   Compiling infra-spec v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/infra/infra-spec)
   Compiling ess-compiler v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/specify/ess-compiler)
   Compiling infra-project v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/infra/infra-project)
   Compiling ess-gen v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/generate/ess-gen)
   Compiling ess-realization v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/specify/ess-realization)
   Compiling ess-composition v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/specify/ess-composition)
   Compiling ess-deployment v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/generate/ess-deployment)
   Compiling ess-conformance v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/verify/ess-conformance)
   Compiling schema-contract v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/generate/schema-contract)
   Compiling ess-synth v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/generate/ess-synth)
   Compiling ess-diff v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/verify/ess-diff)
   Compiling ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 17.78s
     Running tests/openapi_adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/openapi_adversary_pass1-5b967d6bb3cd60e9)

thread 'cli_rejects_unknown_unit_fields_before_any_projection_output' (3326776) panicked at crates/edge/ess-cli/tests/openapi_adversary_pass1.rs:76:5:
unknown wire fields bypassed checked CLI admission and wrote output: ["flat existing", "flat absent", "flat stdout", "area existing", "area absent", "area stdout"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: test failed, to rerun pass `-p ess-cli --test openapi_adversary_pass1`
```

exit status: 101

focused-13 argv:

```text
cargo test --locked --offline --target-dir /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/cargo-target -p ess-cli --test openapi_adversary_pass1 cli_rejects_duplicate_keys_and_tampered_accounting_before_output -- --exact --nocapture 
```

stdout:

```text

running 1 test
test cli_rejects_duplicate_keys_and_tampered_accounting_before_output ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.01s

```

stderr:

```text
    Finished `test` profile [unoptimized] target(s) in 0.08s
     Running tests/openapi_adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/openapi_adversary_pass1-5b967d6bb3cd60e9)
```

exit status: 0

3. Full two-package suite after focused cases

The before count of 150 is the candidate count supplied by the original adversary brief and implementor's green handoff, not a suite executed before these additions. The full resumed run selected every target in ess-cli and ess-openapi with --no-fail-fast so a red target could not suppress later targets. It executed 163 Rust test cases: ess-cli 129 and ess-openapi 34, of which 159 passed and 4 failed. There were zero ignored, measured, or filtered cases. The 150 pre-existing candidate cases remained green; nine new cases are green and four new cases are red. Doc-tests selected zero cases. Embedded CLI subprocesses and Go checks are not added again to Rust executable counts.

Verbatim suite argv:

```text
cargo test --locked --offline --target-dir /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1/cargo-target -p ess-openapi -p ess-cli --no-fail-fast 
```

stdout:

```text

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 25 tests
test author_nested_only ... ok
test author_nonmatching_only ... ok
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test author_empty ... ok
test go_nonmatching_only ... ok
test ir_nested_only ... ok
test run_empty ... ok
test run_nested_only ... ok
test run_nonmatching_only ... ok
test ir_nonmatching_only ... ok
test web_empty ... ok
test web_nested_only ... ok
test ir_empty ... ok
test web_nonmatching_only ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test go_empty ... ok
test go_nested_only ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s


running 9 tests
test non_utf8_matching_content_is_refused_before_any_output_write ... ok
test selected_directory_symlink_reports_the_requested_path_and_stays_shallow ... ok
test empty_selection_cannot_follow_or_replace_an_output_symlink ... ok
test committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners ... ok
test lexical_selection_order_decides_the_first_read_error ... ok
test matching_broken_links_are_read_errors_even_beside_a_valid_source ... ok
test omitted_scenarios_ignore_a_poisoned_working_directory_default ... ok
test valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links ... ok
test empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s


running 9 tests
test an_explicit_missing_front_page_is_an_error ... ok
test a_page_identity_can_itself_end_in_html ... ok
test binary_downloads_are_not_silently_decoded ... ok
test asset_symlink_output_is_refused_before_any_page_changes ... ok
test publication_without_strict_mode_preserves_unpublished_links ... ok
test strict_links_check_local_and_cross_page_fragments ... ok
test selected_sources_resolve_after_relocation_with_queries_fragments_and_downloads ... ok
test assets_cannot_collide_with_generated_output_or_escape_it ... ok
test undeclared_existing_and_escaping_targets_are_refused_with_source_lines ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s


running 4 tests
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s


running 4 tests
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_rust_cli_writes ... ok
test a_binding_function_capture_is_refused_before_web_cli_writes ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 7 tests
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.97s


running 2 tests
test root_selection_and_output_refusals_preserve_existing_files ... ok
test each_target_retains_the_same_model_selection_and_distinct_input_provenance ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s


running 11 tests
test unsupported_go_pattern_has_no_successful_partial_artifact ... ok
test incompatible_later_destination_preserves_the_entire_existing_output ... ok
test output_symlinks_and_hardlinks_are_refused_without_mutation ... ok
test generated_paths_cannot_replace_canonical_source_inputs_or_follow_links ... ok
test stale_bundle_missing_source_and_unknown_dispatch_refuse ... ok
test explicit_binary64_inputs_run_through_the_cli_without_weakening_version_one ... ok
test checked_recipe_and_run_are_deterministic_with_json_only_stdout ... ok
test output_cannot_replace_any_declared_input ... ok
test generation_checks_refuse_before_creating_or_changing_destinations ... ok
test failed_check_or_execution_preserves_output_and_does_not_emit_a_result ... ok
test generated_libraries_match_the_api_and_drift_check_never_repairs_files ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s


running 6 tests
test complete_projection_preserves_supported_yaml_through_both_spellings ... ok
test both_import_spellings_write_the_accounted_envelope_for_every_presentation ... ok
test a_refused_import_keeps_existing_output_and_creates_no_new_file ... ok
test legacy_projection_refuses_before_destination_mutation ... ok
test partial_projection_reports_the_durable_gap_and_preserves_destinations ... ok
test unresolved_projection_reports_the_exact_reference_and_preserves_destinations ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s


running 2 tests
test cli_rejects_duplicate_keys_and_tampered_accounting_before_output ... ok
test cli_rejects_unknown_unit_fields_before_any_projection_output ... FAILED

failures:

---- cli_rejects_unknown_unit_fields_before_any_projection_output stdout ----
flat existing: exit=Some(0); stdout_bytes=0; destination_preserved=false
flat absent: exit=Some(0); stdout_bytes=0; destination_preserved=false
flat stdout: exit=Some(0); stdout_bytes=196; destination_preserved=false
area existing: exit=Some(0); stdout_bytes=0; destination_preserved=false
area absent: exit=Some(0); stdout_bytes=0; destination_preserved=false
area stdout: exit=Some(0); stdout_bytes=196; destination_preserved=false

thread 'cli_rejects_unknown_unit_fields_before_any_projection_output' (3352907) panicked at crates/edge/ess-cli/tests/openapi_adversary_pass1.rs:76:5:
unknown wire fields bypassed checked CLI admission and wrote output: ["flat existing", "flat absent", "flat stdout", "area existing", "area absent", "area stdout"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    cli_rejects_unknown_unit_fields_before_any_projection_output

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 19 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s


running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s


running 8 tests
test missing_dialect_and_incomplete_import_leave_existing_output_alone ... ok
test document_import_refusals_preserve_source_and_existing_output ... ok
test corrupted_import_cannot_be_projected_and_output_cannot_replace_source ... ok
test type_planning_and_output_refusals_leave_no_partial_library ... ok
test types_bundle_is_deterministic_and_never_replaces_its_input ... ok
test import_reload_projection_and_instance_validation_keep_original_data ... ok
test document_root_survives_reload_projection_and_each_type_target ... ok
test native_bundle_targets_require_identity_and_emit_build_metadata ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s


running 1 test
test fatal_synthesis_preserves_destinations_and_has_a_typed_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s


running 17 tests
test tests::review_document_dialect_is_checked_before_schema_interpretation ... ok
test tests::review_annotation_names_only_normalize_at_admitted_positions ... ok
test tests::an_external_reference_is_refused_instead_of_fetched ... ok
test tests::review_schema_annotation_literals_are_not_walked_as_schemas ... ok
test tests::review_empty_enum_refuses_at_every_site ... ok
test tests::review_reference_sibling_constraints_are_accounted_at_every_site ... ok
test tests::projection_is_byte_deterministic ... ok
test tests::unresolved_local_references_are_reported_without_being_guessed ... ok
test tests::review_missing_array_items_refuses_at_every_site ... ok
test tests::review_schema_dialect_override_refuses_at_every_site ... ok
test tests::review_type_array_is_not_an_absent_type ... ok
test tests::review_invalid_optional_string_constraints_refuse ... ok
test tests::review_unsupported_reference_fragments_are_not_dangling_names ... ok
test tests::supported_ir_survives_projection_and_reimport_semantically ... ok
test tests::review_string_enum_and_const_are_both_retained ... ok
test tests::review_version_prefix_is_not_a_valid_patch_version ... ok
test tests::review_nonstring_enum_and_const_are_accounted_at_every_site ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 6 tests
test unresolved_references_keep_distinct_sites_and_escaped_target_identity ... ok
test partial_accounting_survives_reload_and_blocks_projection ... ok
test raw_source_identity_includes_comments_and_line_endings ... ok
test repeated_fields_and_nested_map_keys_refuse_before_erasure ... ok
test checked_envelope_preserves_exact_source_and_legacy_bytes ... ok
test replay_refuses_tampered_identity_interface_and_accounting ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 11 tests
test duplicate_escaped_map_keys_are_rejected_before_replay ... ok
test reference_escape_identity_is_not_decoded_twice ... ok
test nonunit_schema_variants_reject_unknown_fields ... ok
test accounting_order_duplicates_codes_and_missing_arrays_cannot_be_normalized_away ... ok
test contradictory_string_enum_and_const_survive_both_boundaries ... ok
test unsupported_items_cannot_disappear_at_nested_or_message_sites ... ok
test schema_resource_and_dialect_features_refuse_but_annotation_literals_do_not ... ok
test closed_import_rejects_unknown_number_variant_fields ... FAILED
test closed_import_rejects_unknown_integer_variant_fields ... FAILED
test closed_import_rejects_unknown_boolean_variant_fields ... FAILED
test unconsumed_variant_keywords_remain_gaps_after_replay ... ok

failures:

---- closed_import_rejects_unknown_number_variant_fields stdout ----
ADMITTED json /interface/types/A/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/types/A/properties/p~0~1/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/properties/p~0~1/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/operations/probe/request/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/operations/probe/request/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/operations/probe/responses/200/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/operations/probe/responses/200/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/types/A/items/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/items/unexpected_constraint; erased=true; checked_projection_ok=true

thread 'closed_import_rejects_unknown_number_variant_fields' (3353227) panicked at crates/generate/ess-openapi/tests/adversary_pass1.rs:74:5:
closed import envelope admitted unknown fields in number unit variants: ["json /interface/types/A", "yaml /interface/types/A", "json /interface/types/A/properties/p~0~1", "yaml /interface/types/A/properties/p~0~1", "json /interface/operations/probe/request/schema", "yaml /interface/operations/probe/request/schema", "json /interface/operations/probe/responses/200/schema", "yaml /interface/operations/probe/responses/200/schema", "json /interface/types/A/items", "yaml /interface/types/A/items"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- closed_import_rejects_unknown_integer_variant_fields stdout ----
ADMITTED json /interface/types/A/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/types/A/properties/p~0~1/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/properties/p~0~1/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/operations/probe/request/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/operations/probe/request/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/operations/probe/responses/200/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/operations/probe/responses/200/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/types/A/items/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/items/unexpected_constraint; erased=true; checked_projection_ok=true

thread 'closed_import_rejects_unknown_integer_variant_fields' (3353226) panicked at crates/generate/ess-openapi/tests/adversary_pass1.rs:74:5:
closed import envelope admitted unknown fields in integer unit variants: ["json /interface/types/A", "yaml /interface/types/A", "json /interface/types/A/properties/p~0~1", "yaml /interface/types/A/properties/p~0~1", "json /interface/operations/probe/request/schema", "yaml /interface/operations/probe/request/schema", "json /interface/operations/probe/responses/200/schema", "yaml /interface/operations/probe/responses/200/schema", "json /interface/types/A/items", "yaml /interface/types/A/items"]

---- closed_import_rejects_unknown_boolean_variant_fields stdout ----
ADMITTED json /interface/types/A/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/types/A/properties/p~0~1/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/properties/p~0~1/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/operations/probe/request/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/operations/probe/request/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/operations/probe/responses/200/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/operations/probe/responses/200/schema/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED json /interface/types/A/items/unexpected_constraint; erased=true; checked_projection_ok=true
ADMITTED yaml /interface/types/A/items/unexpected_constraint; erased=true; checked_projection_ok=true

thread 'closed_import_rejects_unknown_boolean_variant_fields' (3353225) panicked at crates/generate/ess-openapi/tests/adversary_pass1.rs:74:5:
closed import envelope admitted unknown fields in boolean unit variants: ["json /interface/types/A", "yaml /interface/types/A", "json /interface/types/A/properties/p~0~1", "yaml /interface/types/A/properties/p~0~1", "json /interface/operations/probe/request/schema", "yaml /interface/operations/probe/request/schema", "json /interface/operations/probe/responses/200/schema", "yaml /interface/operations/probe/responses/200/schema", "json /interface/types/A/items", "yaml /interface/types/A/items"]


failures:
    closed_import_rejects_unknown_boolean_variant_fields
    closed_import_rejects_unknown_integer_variant_fields
    closed_import_rejects_unknown_number_variant_fields

test result: FAILED. 8 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

stderr:

```text
   Compiling ess-openapi v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/generate/ess-openapi)
   Compiling ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 2.31s
     Running unittests src/main.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/ess-bdf16dcf99d99666)
     Running tests/authored_scenarios.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/authored_scenarios-2a52915173d4466e)
     Running tests/authored_scenarios_adversary.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/authored_scenarios_adversary-79337df623536839)
     Running tests/authored_site.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/authored_site-5371c61490a18107)
     Running tests/command_surface.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/command_surface-d726dee0cc47da60)
     Running tests/command_surface_adversary.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/command_surface_adversary-827d8ff65a3e5bad)
     Running tests/feasibility_adversary.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/feasibility_adversary-4129aeb509bf2158)
     Running tests/go_conformance.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/go_conformance-ba1d9e6377cf10a2)
     Running tests/model_types.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/model_types-7811eadb88011dcb)
     Running tests/normalization.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/normalization-5d5bb3cb824dfb8a)
     Running tests/openapi_accounting.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/openapi_accounting-5ee227b993dfc475)
     Running tests/openapi_adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/openapi_adversary_pass1-5b967d6bb3cd60e9)
error: test failed, to rerun pass `-p ess-cli --test openapi_adversary_pass1`
     Running tests/output_containment.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/output_containment-a7a441a678c9d27b)
     Running tests/persisted_delivery.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/persisted_delivery-0c666e794f84efd7)
     Running tests/schema_bundle.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/schema_bundle-e4dfccfa3fd0e18d)
     Running tests/target_failure.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/target_failure-5fcd519645fe8065)
     Running unittests src/lib.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/ess_openapi-d6e5f099f8add75c)
     Running tests/accounting.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/accounting-4be464add4945ce7)
     Running tests/adversary_pass1.rs (target/review-boundaries-7/adversary-pass-1/cargo-target/debug/deps/adversary_pass1-a8c97e6079c0b8d7)
error: test failed, to rerun pass `-p ess-openapi --test adversary_pass1`
   Doc-tests ess_openapi
error: 2 targets failed:
    `-p ess-cli --test openapi_adversary_pass1`
    `-p ess-openapi --test adversary_pass1`
```

exit status: 101

Formatting/Clippy were not run in this adversary continuation: no source or test edit was made, and the assignment makes those conditional. No whole-workspace or site gate was run. This report returns red tests for the implementor; it does not claim delivery readiness.

4. Finding and reachability

Findings cover frozen production commit 633e1d9839dc0e8dae071e955f8e8af540c8a685 plus the two pass-1 test files listed above.

| File:line | Category / severity | Verdict | Origin | Finding |
|---|---|---|---|---|
| crates/generate/ess-openapi/src/accounting.rs:152 | acceptance / blocker | CONFIRMED | introduced | The new checked import reader accepts and erases unknown fields on integer, number, and boolean schema variants, so both CLI projection spellings write output from a wire document the closed-envelope contract requires rejecting. |

What was measured: focused-01/02/03 each execute one failing case at crates/generate/ess-openapi/tests/adversary_pass1.rs:74 and exit 101; unknown nested fields are admitted and erased before replay equality at accounting.rs:179. Focused-12 executes the CLI case, fails its assertion at crates/edge/ess-cli/tests/openapi_adversary_pass1.rs:76 and exits 101. Every actual projection subprocess exits 0 and changes its destination. The full suite reproduces these same four failures.

What reaches it: the test creates original OpenAPI source, invokes the real `ess infra import openapi --path source.json --out import.json --format json` writer, adds an unknown wire field to that persisted file, and invokes both `ess project openapi --ir import.json` and `ess generate project openapi --ir import.json` with --out or stdout. Production routing is main.rs:2962 → project_openapi_interface at :3176 → read_import at :3183 → project_import at :3193 → filesystem write at :3204. The command reference at website/docs/reference/cli.md:128 documents this --ir workflow. Arbitrary persisted bytes are the admission boundary's actual input; no private constructors or unreachable internal state are needed.

Origin: introduced at the new read_import conversion in accounting.rs:152 and its new CLI call at main.rs:3183. `git show 21eac63d347d5d1328712cd59dd9ae5edf41aace:crates/generate/ess-openapi/src/lib.rs` confirms that the old unit variants existed, and `git ls-tree` at that base confirms accounting.rs did not exist. The base had no read_import entry point or claimed replay-checked import envelope. The origin claim is limited to this newly introduced checked-admission path; it does not claim this unit introduced Serde's unit-variant behavior or changed the intentionally retained legacy reader. No tree was moved to the base and no base execution is claimed.

The accepted binding requires unknown wire fields to refuse before returning the public checked value (docs/design/review-openapi-accounting.md, Checked admission). Duplicate preflight succeeds here because each inserted field is unique. The nested unit variant then discards the field, and comparing the reduced typed wire with replay cannot recover it. Existing tamper tests cover the wrapper, service and accounting records, but omit these unit variants. A fix belongs at the new wire boundary, using strict nested admission that retains the legacy reader/writer and its bytes. No fix was applied. There are no separate judgement-only findings.

5. Attacked and not broken

- The unit's supported source/interface/projected fixtures and frozen old-reader compatibility remain green in the full suite.
- Variant-specific unsupported constraints remain durable across replay at components, properties, messages and array items.
- Both contradictory string enum/const constraints survive both boundaries without weakening either one.
- Unsupported items and interpretation-changing dialect/resource features refuse at all tested sites.
- Annotation literals remain annotations; their contents are not recursively interpreted as schemas.
- Escaped reference identity and distinct unresolved reference sites survive reload.
- Missing accounting arrays, reordered or duplicate entries, changed codes, source digest tampering and escaped duplicate keys refuse.
- Nonunit schema variants reject the same unknown-field mutation, and the CLI's digest/duplicate controls preserve existing output.

6. Paths and process closure

No deliberate file was written outside the assigned worktree. Shared dependency/compiler infrastructure can incur incidental cache bookkeeping/content effects at these full paths; parallel users prevent attributing individual cache entries to this pass:

- /home/timo/.cache/sccache — existing shared compiler cache, confirmed by the recovered sccache-before.txt; no explicit pruning or daemon lifecycle operation was performed.
- /home/timo/.cargo/.global-cache — Cargo's existing global cache bookkeeping. Registry/git dependencies were used in locked offline mode; no intentional dependency-cache population or edit was requested.

The existing /home/timo/.cargo/.package-cache lock file and /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock were reused as endpoints; neither is a newly created scratch artifact. No outside Cargo target, /tmp scratch, credentials, network integration, or live system was selected. Existing suite tests create their usual repository-local target fixtures; Go TMPDIR/GOTMPDIR/GOCACHE/GOMODCACHE were all within the assigned scratch.

All deliberate pass output is under /home/timo/.local/state/worktree/trees/b10x/ess/review-openapi-semantic-accounting/target/review-boundaries-7/adversary-pass-1: preserved focused-01.*, focused-02.* through focused-13.*, suite-01.*, resumed-run.environment, recovered-inputs.sha256 and recovered-inputs-verified.txt, final-status.txt, final-tracked-diff.stat, final-untracked-test-diff.stat, final-disk.txt, process-snapshot.txt, this report, CLI probe fixture directories and their exact per-call records, go-cache, go-mod-cache, and the original own cargo-target build directory. Existing original brief, pre-execution hashes/status and sccache record remain intact.

The focused recording process (tool session 24359) and full suite process (tool session 9058) both completed and were observed closed. Their recorded Cargo statuses, rather than the recording-shell exit, are used above. The existing shared sccache daemon is coordinator-owned and was left running. Final disk observation remained above the 8 GiB floor:

```text
Filesystem        1B-blocks         Used   Available Use% Mounted on
/dev/nvme0n1p2 910126964736 831433768960 32385843200  97% /
```

This completes the bounded first attack. No additional attack or lifecycle action was started.

```findings
- file: crates/generate/ess-openapi/src/accounting.rs
  line: 152
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: The new checked import reader accepts and erases unknown fields on integer, number, and boolean schema variants, so both CLI projection spellings write output from a wire document the closed-envelope contract requires rejecting.
```
