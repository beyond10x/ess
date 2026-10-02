---
format: aep.planning-md/3
id: review-result:served-store-and-entry-adversary-final-20261003
kind: review-result
status: active
title: Served store and entry final adversary, frozen 4a17f69b
relations:
- reviews: story:served-store-and-entry
revision: 1
---
unit: story:served-store-and-entry; frozen candidate 4a17f69b62f53e80bda3c32cdecd79f06fca1ac0 plus tests-only additions
verdict: CONFIRMED
cases: executed 30→33, red 1
origin: introduced 0 / pre-existing 1 / undecided 0
wrote-outside-worktree: 3 directory roots; exact retained artifacts enumerated below
needs-coordinator: route the confirmed pre-existing Go decoder defect and resolve the retained red case before declaring this target green
Publication normalization: literal user-home prefixes are replaced with $HOME/; raw report SHA256 d7d97ccecd04f94d8eac6ddc89c0a7cddaaf241c48a77cf9172b10468147f057. No findings, counts or quoted outputs are otherwise changed.

```text
 crates/generate/ess-synth/tests/served_entry.rs | 166 ++++++++++++++++++++++++
 1 file changed, 166 insertions(+)
```

Frozen source comparison base: 1ff3056850e52ed3cf5f2a7e1a1d7f4af46cb036. Checkout: $HOME/.local/state/worktree/trees/b10x/ess/ess-serial-318-20261003. Only crates/generate/ess-synth/tests/served_entry.rs changed: 166 insertions, zero deletions. Existing cases and assertions were preserved. No implementation, planning, fixture, manifest or commit changes. The before count of 30 was supplied by the coordinator; no baseline suite was run.

1. Prospective cases, each written before the first test execution and run alone before the suite

crates/generate/ess-synth/tests/served_entry.rs:18 — adversary_ambiguous_actor_aliases_never_choose_a_grant
GREEN: two declared Writer actors cannot resolve through the ambiguous short alias; qualified names select their exact distinct grants; refusals leave no rows; default none rejects a qualified granted header. Both Rust and Go executed.
Command: env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 RUSTC_WRAPPER= GOCACHE=$HOME/.cache/ess-w5-go-cache TMPDIR=$HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2 cargo test -p ess-synth --locked --offline --test served_entry adversary_ambiguous_actor_aliases_never_choose_a_grant -- --exact --nocapture
Exit: 0
Verbatim output:
```text
   Compiling ess-synth v0.51.0 ($HOME/.local/state/worktree/trees/b10x/ess/ess-serial-318-20261003/crates/generate/ess-synth)
    Finished `test` profile [unoptimized] target(s) in 1.67s
     Running tests/served_entry.rs (target/debug/deps/served_entry-a07dcb14d3828c50)

running 1 test
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"none"}
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"none"}
test adversary_ambiguous_actor_aliases_never_choose_a_grant ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 32 filtered out; finished in 6.17s

```

crates/generate/ess-synth/tests/served_entry.rs:88 — adversary_union_identity_orders_tags_then_typed_payloads
RED: declared union tag order, absent/present Optional payload order, numeric 2 before 10 and signed Integer extrema; duplicate key replacement and exact state update. Every assertion passes through Rust HTTP. Go fails while compiling its generated consumer, before its HTTP assertions execute.
Command: env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 RUSTC_WRAPPER= GOCACHE=$HOME/.cache/ess-w5-go-cache TMPDIR=$HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2 cargo test -p ess-synth --locked --offline --test served_entry adversary_union_identity_orders_tags_then_typed_payloads -- --exact --nocapture
Exit: 101
Verbatim output:
```text
    Finished `test` profile [unoptimized] target(s) in 0.20s
     Running tests/served_entry.rs (target/debug/deps/served_entry-a07dcb14d3828c50)

running 1 test
{"format":"ess/1","callers":"actor-header","demonstration":true}

thread 'adversary_union_identity_orders_tags_then_typed_payloads' (164805) panicked at crates/generate/ess-synth/tests/served_entry.rs:276:13:
# example.invalid/notebook/server
server/wire.go:344:4: declared and not used: shape
server/wire.go:346:35: undefined: shape

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test adversary_union_identity_orders_tags_then_typed_payloads ... FAILED

failures:

failures:
    adversary_union_identity_orders_tags_then_typed_payloads

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 32 filtered out; finished in 10.27s

error: test failed, to rerun pass `-p ess-synth --test served_entry`
```

crates/generate/ess-synth/tests/served_entry.rs:145 — adversary_static_head_and_document_routes_keep_their_contracts
GREEN: HEAD has no body, directory index works with a query string, POST refuses, a static openapi.json cannot shadow the generated document, and an index symlink outside the selected root refuses. Both Rust and Go executed.
Command: env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 RUSTC_WRAPPER= GOCACHE=$HOME/.cache/ess-w5-go-cache TMPDIR=$HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2 cargo test -p ess-synth --locked --offline --test served_entry adversary_static_head_and_document_routes_keep_their_contracts -- --exact --nocapture
Exit: 0
Verbatim output:
```text
    Finished `test` profile [unoptimized] target(s) in 0.11s
     Running tests/served_entry.rs (target/debug/deps/served_entry-a07dcb14d3828c50)

running 1 test
{"format":"ess/1","callers":"none"}
{"format":"ess/1","callers":"none"}
test adversary_static_head_and_document_routes_keep_their_contracts ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 32 filtered out; finished in 26.00s

```

2. Target suite after all case-alone runs

Command: env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=2 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 RUSTC_WRAPPER= GOCACHE=$HOME/.cache/ess-w5-go-cache TMPDIR=$HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2 cargo test -p ess-synth --locked --offline --test served_entry -- --test-threads=2 --nocapture
Exit: 101
Verbatim output:
```text
    Finished `test` profile [unoptimized] target(s) in 0.26s
     Running tests/served_entry.rs (target/debug/deps/served_entry-a07dcb14d3828c50)

running 33 tests
test a_non_network_domain_named_memory_remains_available ... ok
test a_specification_without_network_reach_gets_no_memory_or_entry ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"none"}
{"format":"ess/1","callers":"actor-header","demonstration":true}
test a_network_domain_named_memory_remains_available ... ok
{"format":"ess/1","callers":"none"}
test a_stateless_served_component_needs_no_storage ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"none"}
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"none"}
test adversary_ambiguous_actor_aliases_never_choose_a_grant ... ok
{"format":"ess/1","callers":"none"}
{"format":"ess/1","callers":"none"}
test adversary_static_head_and_document_routes_keep_their_contracts ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}

thread 'adversary_union_identity_orders_tags_then_typed_payloads' (224797) panicked at crates/generate/ess-synth/tests/served_entry.rs:276:13:
# example.invalid/notebook/server
server/wire.go:344:4: declared and not used: shape
server/wire.go:346:35: undefined: shape

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test adversary_union_identity_orders_tags_then_typed_payloads ... FAILED
test an_entry_point_needing_caller_attributes_refuses_to_start_naming_them ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
test an_entry_point_with_owed_commands_refuses_to_start_naming_them ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
test decimal_identity_preserves_rendering_equality_and_order_over_http ... ok
test fallible_context_names_do_not_collide_with_authored_types_or_commands ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
test fallible_context_propagates_before_effects_and_keeps_legacy_implementations ... ok
test generated_runtime_dependencies_respect_the_minimum_rust_version ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
test generated_stores_replace_and_remove_only_the_addressed_identity ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
test go_memory_ports_compare_decoded_json_values_without_changing_native_equality ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
test generated_names_do_not_hide_ports_or_runtime_imports ... ok
test memory_ports_preserve_the_single_obligation_stub ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
test json_identity_preserves_number_spelling_and_member_order_over_http ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
test only_reachable_context_needs_prevent_a_component_from_starting ... ok
test other_assigned_values_and_external_answers_remain_named_obligations ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
test structural_identities_use_typed_order_and_ignore_map_insertion_order ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
Rust: 5 HTTP conformance scenarios
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
Go: 5 HTTP conformance scenarios
test the_fixture_suite_passes_against_the_generated_go_and_rust_servers ... ok
test reusable_types_keep_the_default_dependency_graph_empty_and_build_for_wasm ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
test the_generated_context_assigns_distinct_uuids ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
test the_generated_context_reads_the_clock_through_a_newtype ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"none"}
test the_go_entry_point_serves_the_fixture_with_no_hand_written_code ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"none"}
test the_rust_entry_point_serves_the_fixture_with_no_hand_written_code ... ok
test the_served_notes_plan_has_no_obligation ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
test the_single_crate_entry_builds_and_serves_with_the_server_feature ... ok
{"format":"ess/1","callers":"none"}
{"format":"ess/1","callers":"none"}
test the_static_directory_is_served_beside_the_api ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
{"format":"ess/1","callers":"actor-header","demonstration":true}
test the_store_lists_in_identity_order ... ok
test unsupported_memory_context_has_no_panic_or_fabricated_answer ... ok
{"format":"ess/1","callers":"actor-header","demonstration":true}
test the_generated_context_reads_the_system_clock ... ok

failures:

failures:
    adversary_union_identity_orders_tags_then_typed_payloads

test result: FAILED. 32 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 91.52s

error: test failed, to rerun pass `-p ess-synth --test served_entry`
```

The target executed 33 cases: 32 passed, 1 failed, 0 ignored. This is a red target, not a passing review. The existing fixture conformance controls executed five HTTP scenarios per language. No full package, workspace, release, CI or site suite was repeated. rustfmt --edition 2021 --check of the changed test and git diff --check passed. Target-specific strict Clippy passed (exit 0):
```text
    Checking ess-synth v0.51.0 ($HOME/.local/state/worktree/trees/b10x/ess/ess-serial-318-20261003/crates/generate/ess-synth)
    Finished `dev` profile [unoptimized] target(s) in 0.70s
```

3. Findings and measured origin

| File:line | Category | Severity | Verdict | Origin | Finding |
|---|---|---|---|---|---|
| crates/generate/ess-synth/src/go/http.rs:730 | contract-drift | blocker | CONFIRMED | pre-existing | An admitted union with an Optional payload generates a Go decoder that declares shape inside the presence branch and references it outside, so the generated server does not compile. |

What was measured: the new Rust integration harness compiled, then go vet of its emitted valid Go consumer failed with declared-and-not-used shape at generated server/wire.go:344 and undefined shape at :346; the case-alone exit was 101. This is a generated-consumer compilation defect, not a new harness compilation error.

What reaches it: the existing served-notes authored model, changing only NoteId from an Integer newtype to a declared union with zulu: Integer and alpha: Optional<Integer>. AddNote accepts this identity and ArchiveNote addresses it. The compiler and target admit the model with zero obligations/refusals. Ordinary synthesis plus Go build/vet reaches the invalid decoder. The newly documented union/optional memory identity behavior cannot be exercised by this Go consumer because it fails before startup.

Origin proof: the coordinator's retained CLI built from exact main 1ff3056850e52ed3cf5f2a7e1a1d7f4af46cb036 synthesized the identical authored input under $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/union-model. Command: $HOME/.cache/uilab-todo/serial-20261003/baseline/ess-main generate synthesize --path $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/union-model --target go --out $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/base-go. Exit 0:
```text
16 capabilities: 16 generated, 0 obligation(s), 0 refused
16 artifact(s), written to $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/base-go
```
Then, in $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/base-go, GOWORK=off GOPROXY=off GOTOOLCHAIN=local GOCACHE=$HOME/.cache/ess-w5-go-cache TMPDIR=$HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2 go vet ./... exited 1:
```text
# example.invalid/notebook/server
vet: server/wire.go:346:35: undefined: shape
```
cmp of base-go/server/wire.go and candidate target/tmp/served-entry-go-adversary-union-identity-164804/server/wire.go exited 0: byte-identical files. git show of the exact base also retains the same generator declaration and branch scope. Thus origin is measured pre-existing, not inferred from blame. The report does not assign ownership of its correction to this story. Suggested bounded correction: declare the optional union payload temporary in the case scope and assign it within the presence branch; preserve missing/null as the absent value. No correction was applied.

4. Attacks that did not break, and omissions

- Ambiguous actor aliases and distinct qualified grants passed through real Rust and Go executables, including denial under default caller mode.
- Static HEAD, query-bearing directory indexes, document-route priority, method refusal and index-symlink confinement passed in both targets.
- Union tag/Optional payload/numeric-extreme ordering, replacement and addressed state update passed in Rust. Go union memory identity was not reached because of the measured decoder error.
- All existing 30 target cases passed, including context refusal before effects, component reachability, structural/Json identities, generated context, offline consumers, single-crate/default-library/WASM and HTTP conformance.
- No mutation testing, race detector, arbitrary filesystem-race probe, additional name-allocation matrix, production authentication, durable storage or CORS testing was performed. Prior adversary findings on an older candidate were not treated as review coverage of this candidate. No third adversarial pass was attempted or recommended.
- Disk observations remained above the assigned 10 GiB floor; minimum observed available bytes were 20,345,577,472 before target Clippy. No cache cleanup or foreign process control occurred.

5. Every outside-worktree write root and exact retained artifacts

Explicit scratch root: $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/actor-alone.exit
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/actor-alone.log
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/base-go/ (complete generated subtree)
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/base-go-vet.exit
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/base-go-vet.log
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/base-synthesis.exit
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/base-synthesis.log
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/clippy.exit
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/clippy.log
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/static-alone.exit
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/static-alone.log
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/suite.exit
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/suite.log
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/union-alone.exit
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/union-alone.log
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/union-model/ (complete generated subtree)
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/wire-comparison.exit
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/report.md
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/report-public.md
- $HOME/.cache/uilab-todo/serial-20261003/318/adversary-pass-2/report.sha256

Admitted Go build cache updated/reused by go vet/build/tests: $HOME/.cache/ess-w5-go-cache/ (tool-managed cache subtree).
Managed lease state updated only through worktree hooks: $HOME/.local/state/worktree/ (session codex-318-final-adversary; only this lease acquired, renewed and released).
All generated consumer build trees used the assigned checkout's target/ tree. Transient tool files used the assigned TMPDIR. Existing Cargo registry/toolchain caches and the exact-main CLI were consumed, not relocated or deliberately rewritten. No files were written under /tmp, and no generated probe was committed.

```findings
[
  {
    "file": "crates/generate/ess-synth/src/go/http.rs",
    "line": 730,
    "category": "contract-drift",
    "severity": "blocker",
    "verdict": "CONFIRMED",
    "origin": "pre-existing",
    "message": "An admitted union with an Optional payload generates a Go decoder that declares shape inside the presence branch and references it outside, so the generated server does not compile."
  }
]
```
