---
format: aep.planning-md/1
id: review-result:cli-presentation-adversary-r1-20260909
kind: review-result
status: active
title: CLI presentation adversary first pass 2026-09-09
relations:
- reviews: story:cli-presentation-binding
revision: 1
---
unit: story:cli-presentation-binding — ATTACK1 of working tree work/cli-binding-latest-20260909 over 113f5925ebb6a0a57a73e661687d9fa5deddb0b8
verdict: NEEDS-CHANGE
cases: executed 27→32, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: route two source fixes; full edge/repository/website gates and planning remain coordinator-owned

1. git --no-pager diff --stat

The adversary-owned delta is a new test file. Command: git --no-pager diff --no-index --stat /dev/null crates/generate/ess-cli-project/tests/adversarial.rs

```text
 .../generate/ess-cli-project/tests/adversarial.rs  | 204 +++++++++++++++++++++
 1 file changed, 204 insertions(+)
```

The assigned shared checkout already contained the implementation, root integration, and planning changes before this attack. Their full working-tree diff is not an adversary edit. No implementation file or existing test was changed by this pass; no AEP command, commit, branch change, or external publication was performed. Report and logs are task scratch under .local/tmp/cli-wave.

Source identities covered at the end of execution:

```text
e6bb0a4ecf475afaeb2f476b95503d64f040a4420862a141ce2b8706ab090946  crates/specify/ess-cli-contract/src/lib.rs
1f59663faedd7b145a3b0228ae24574caf31d416e70b560f1a571186aaf78f9f  crates/specify/ess-cli-contract/src/resolve.rs
36619c847e00666622b846772ebe726fb6bbdbcf596f2fb1f6f5d5ade84e30f2  crates/specify/ess-cli-contract/src/wire.rs
e62949c04402ae40f612784a4dd811ed61337f1b6f749e5759aad37585597c78  crates/generate/ess-cli-project/src/lib.rs
3442a9a8904dec7cf3e17c9d2c38439f99693c6b27eb87d1643619e42d45105c  crates/generate/ess-cli-project/src/runtime.rs
b5e0371cbbdd74498fb2510bb61f051d8c81d0f015fbb8a8e4b41c40ada20d30  crates/generate/ess-cli-project/tests/adversarial.rs
```

2. Cases added and their first isolated executions

All five cases were written before any execution. A static read corrected a flattened field naming fixture before its first run; no compilation failure is reported as a finding. No suite was run before these cases existed.

Every Cargo command used:
TMPDIR="$PWD/.local/tmp/cli-wave" RUSTC_WRAPPER= CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
The existing task target/debug directory was reused; nested generated packages used their own task TempDir targets. Builds were serialized with the coordinator; disk inspection reported 46 GiB available.

File: crates/generate/ess-cli-project/tests/adversarial.rs; case: admitted_binary_names_emit_cargo_accepted_targets
Assertion: A compiled binding must emit a Cargo-accepted standalone package, or compilation must refuse an unsupported binary name.
Current result: RED. First isolated command:
cargo test --offline -p ess-cli-project --test adversarial admitted_binary_names_emit_cargo_accepted_targets -- --exact --nocapture
Exit: 101.

```text
   Compiling ess-cli-project v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/crates/generate/ess-cli-project)
    Finished `test` profile [unoptimized] target(s) in 0.52s
     Running tests/adversarial.rs (target/debug/deps/adversarial-8d43142f2d7f14c2)

running 1 test

thread 'admitted_binary_names_emit_cargo_accepted_targets' (396887) panicked at crates/generate/ess-cli-project/tests/adversarial.rs:70:9:
compiler admitted binary build, but its emitted Cargo package is invalid:
error: failed to parse manifest at `/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/.local/tmp/cli-wave/.tmp3HeTa7/Cargo.toml`

Caused by:
  the binary target name `build` is forbidden, it conflicts with cargo's build directory names

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test admitted_binary_names_emit_cargo_accepted_targets ... FAILED

failures:

failures:
    admitted_binary_names_emit_cargo_accepted_targets

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p ess-cli-project --test adversarial`
```

File: crates/generate/ess-cli-project/tests/adversarial.rs; case: document_file_refuses_a_fifo_without_waiting_for_an_external_writer
Assertion: A forbidden non-regular document source must reach its refusal without another process satisfying a FIFO open handshake.
Current result: RED. First isolated command:
cargo test --offline -p ess-cli-project --test adversarial document_file_refuses_a_fifo_without_waiting_for_an_external_writer -- --exact --nocapture
Exit: 101.

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversarial.rs (target/debug/deps/adversarial-8d43142f2d7f14c2)

running 1 test

thread 'document_file_refuses_a_fifo_without_waiting_for_an_external_writer' (397514) panicked at crates/generate/ess-cli-project/tests/adversarial.rs:110:5:
assertion `left == right` failed: a document source must reject the non-regular file without an external writer releasing open
  left: Err(Timeout)
 right: Ok(Err(Unavailable))
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test document_file_refuses_a_fifo_without_waiting_for_an_external_writer ... FAILED

failures:

failures:
    document_file_refuses_a_fifo_without_waiting_for_an_external_writer

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 1.00s

error: test failed, to rerun pass `-p ess-cli-project --test adversarial`
```

File: crates/generate/ess-cli-project/tests/adversarial.rs; case: protected_hardlinks_are_refused_while_regular_documents_remain_admitted
Assertion: Protected hard links are refused, while a regular nonsecret document remains admitted.
Current result: GREEN. First isolated command:
cargo test --offline -p ess-cli-project --test adversarial protected_hardlinks_are_refused_while_regular_documents_remain_admitted -- --exact --nocapture
Exit: 0.

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversarial.rs (target/debug/deps/adversarial-8d43142f2d7f14c2)

running 1 test
test protected_hardlinks_are_refused_while_regular_documents_remain_admitted ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

```

File: crates/generate/ess-cli-project/tests/adversarial.rs; case: invalid_ordinary_values_are_refused_before_protected_acquisition
Assertion: Invalid ordinary typed input prevents protected acquisition and handler dispatch.
Current result: GREEN. First isolated command:
cargo test --offline -p ess-cli-project --test adversarial invalid_ordinary_values_are_refused_before_protected_acquisition -- --exact --nocapture
Exit: 0.

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversarial.rs (target/debug/deps/adversarial-8d43142f2d7f14c2)

running 1 test
test invalid_ordinary_values_are_refused_before_protected_acquisition ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

```

File: crates/generate/ess-cli-project/tests/adversarial.rs; case: authored_field_names_dispatch_using_resolved_wire_names
Assertion: A declaration maps authored field identity to the compiled wire identity, including alias dispatch.
Current result: GREEN. First isolated command:
cargo test --offline -p ess-cli-project --test adversarial authored_field_names_dispatch_using_resolved_wire_names -- --exact --nocapture
Exit: 0.

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/adversarial.rs (target/debug/deps/adversarial-8d43142f2d7f14c2)

running 1 test
test authored_field_names_dispatch_using_resolved_wire_names ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

```

3. Suites after the isolated cases

The 27 before count is the implementor's current-base report: 11 binding + 4 projection + 12 generated process cases. This pass selected those same cases plus the 5 new adversarial cases, for 32 executed and 2 red. The concurrently added consumer coverage witnesses were intentionally outside these selected suites; this count does not claim the entire repository gate ran.

Command: cargo test --offline -p ess-cli-project --test adversarial -- --nocapture
Exit: 101.

```text
    Finished `test` profile [unoptimized] target(s) in 0.09s
     Running tests/adversarial.rs (target/debug/deps/adversarial-8d43142f2d7f14c2)

running 5 tests
test protected_hardlinks_are_refused_while_regular_documents_remain_admitted ... ok
test authored_field_names_dispatch_using_resolved_wire_names ... ok
test invalid_ordinary_values_are_refused_before_protected_acquisition ... ok

thread 'admitted_binary_names_emit_cargo_accepted_targets' (399731) panicked at crates/generate/ess-cli-project/tests/adversarial.rs:70:9:
compiler admitted binary build, but its emitted Cargo package is invalid:
error: failed to parse manifest at `/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/.local/tmp/cli-wave/.tmpvYfjq1/Cargo.toml`

Caused by:
  the binary target name `build` is forbidden, it conflicts with cargo's build directory names

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test admitted_binary_names_emit_cargo_accepted_targets ... FAILED

thread 'document_file_refuses_a_fifo_without_waiting_for_an_external_writer' (399733) panicked at crates/generate/ess-cli-project/tests/adversarial.rs:110:5:
assertion `left == right` failed: a document source must reject the non-regular file without an external writer releasing open
  left: Err(Timeout)
 right: Ok(Err(Unavailable))
test document_file_refuses_a_fifo_without_waiting_for_an_external_writer ... FAILED

failures:

failures:
    admitted_binary_names_emit_cargo_accepted_targets
    document_file_refuses_a_fifo_without_waiting_for_an_external_writer

test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s

error: test failed, to rerun pass `-p ess-cli-project --test adversarial`
```

Command: cargo test --offline -p ess-cli-contract --test binding -- --nocapture
Exit: 0.

```text
   Compiling ess-cli-contract v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/crates/specify/ess-cli-contract)
    Finished `test` profile [unoptimized] target(s) in 0.77s
     Running tests/binding.rs (target/debug/deps/binding-fa9bcb50f98c6097)

running 11 tests
test integer_values_use_the_models_exact_signed_64_bit_range ... ok
test supported_nested_value_shapes_are_closed_and_validate_every_element ... ok
test refuses_unsupported_constraints_and_primitive_promises ... ok
test resolves_value_types_without_inventing_a_component_or_entity ... ok
test inputless_local_calls_require_explicit_null_and_no_fictional_struct ... ok
test closes_unknown_format_and_fields_at_every_reader_layer ... ok
test refuses_unresolved_types_fields_and_ambiguous_cli_sources ... ok
test service_forward_command_preserves_the_model_owner_and_input_identity ... ok
test service_forward_parameterized_view_resolves_rows_without_changing_ownership ... ok
test refuses_recursive_nested_unresolved_and_constrained_types ... ok
test service_forward_refuses_foreign_owners_and_mismatched_input_or_view_shapes ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Command: cargo test --offline -p ess-cli-project --test projection -- --nocapture
Exit: 0.

```text
   Compiling ess-cli-project v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/crates/generate/ess-cli-project)
    Finished `test` profile [unoptimized] target(s) in 0.31s
     Running tests/projection.rs (target/debug/deps/projection-1fc3a4893afef73f)

running 4 tests
test service_commands_and_parameterized_views_dispatch_to_the_declared_owner ... ok
test emits_a_deterministic_standalone_parser_process_package ... ok
test emits_help_and_reference_from_the_resolved_binding ... ok

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 12 tests
test aliases_dispatch_once_and_globals_never_enter_payload ... ok
test dynamic_refusals_distinguish_input_resolution_and_interruption ... ok
test acquisition_failure_and_interruption_are_safe_and_do_not_dispatch ... ok
test dynamic_input_result_and_errors_require_the_native_validator ... ok
test typed_application_usage_errors_preserve_the_declared_error_contract ... ok
test parser_refusals_follow_json_selection_without_echoing_values ... ok
test help_and_completions_are_executable_and_do_not_dispatch ... ok
test handler_results_errors_and_interruptions_follow_process_contract ... ok
test typed_values_optional_omission_and_unit_payload_are_enforced ... ok
test integer_input_preserves_signed_boundaries_and_refuses_unsigned_overflow ... ok
test all_protected_channels_are_exclusive_and_never_accept_secret_argv ... ok
test native_file_acquisition_checks_permissions_symlinks_size_and_utf8 ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Locking 34 packages to latest compatible versions
   Compiling proc-macro2 v1.0.107
   Compiling utf8parse v0.2.2
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.24
   Compiling anstyle-parse v1.0.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling libc v0.2.189
   Compiling anstyle-query v1.1.5
   Compiling colorchoice v1.0.5
   Compiling serde_core v1.0.229
   Compiling anstyle v1.0.14
   Compiling anstream v1.0.0
   Compiling strsim v0.11.1
   Compiling clap_lex v1.1.0
   Compiling zmij v1.0.23
   Compiling clap_builder v4.6.6
   Compiling syn v3.0.5
   Compiling rustix v1.1.4
   Compiling serde_json v1.0.151
   Compiling serde v1.0.229
   Compiling serde_derive v1.0.229
   Compiling clap v4.6.6
   Compiling rtoolbox v0.0.6
   Compiling memchr v2.8.3
   Compiling itoa v1.0.18
   Compiling bitflags v2.13.1
   Compiling linux-raw-sys v0.12.1
   Compiling rpassword v7.5.4
   Compiling clap_complete v4.6.9
   Compiling demo-cli-contract v0.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/.local/tmp/cli-wave/ess-cli-fixture-AbwOLI)
    Finished `test` profile [unoptimized] target(s) in 6.14s
     Running unittests src/lib.rs (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/.local/tmp/cli-wave/ess-cli-fixture-AbwOLI/target/debug/deps/cli_contract-e34ff935d4081ec7)
     Running unittests src/main.rs (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/.local/tmp/cli-wave/ess-cli-fixture-AbwOLI/target/debug/deps/demo-a25d39a27ed562d2)
     Running tests/process.rs (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/.local/tmp/cli-wave/ess-cli-fixture-AbwOLI/target/debug/deps/process-31189b33f2bbb430)
   Doc-tests cli_contract

test generated_package_compiles_offline_and_executes_process_fixtures ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.23s

```

Command: cargo clippy --offline -p ess-cli-project --test adversarial -- -D warnings
Exit: 0.

```text
    Checking ess-cli-project v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/crates/generate/ess-cli-project)
    Finished `dev` profile [unoptimized] target(s) in 0.19s
```

rustfmt --edition 2021 --check crates/generate/ess-cli-project/tests/adversarial.rs also passed with exit 0 and no output.

4. Findings

These findings cover the source hashes above, based on 113f5925ebb6a0a57a73e661687d9fa5deddb0b8. Both implementation paths are new in this unit; the base contains neither new crate. No base checkout was moved or mutated.

| File:line | Verdict | Origin | Finding | What was measured | What reaches it |
|---|---|---|---|---|---|
| crates/specify/ess-cli-contract/src/resolve.rs:337 | NEEDS-CHANGE | introduced | The binding compiler admits Cargo-reserved binary names, so a validated binding can emit an unusable standalone package. | adversarial.rs:70: compiler accepted binary build; emitted Cargo metadata failed with exit 101 and Cargo's reserved-target diagnostic. The case itself exited 101. | Authored binary: build follows the documented lowercase-token grammar; ess generate cli compiles it and projects the resulting Cargo.toml. The first reserved name is enough to fail the loop; other candidate names are not claimed as measured here. |
| crates/generate/ess-cli-project/src/runtime.rs:121 | NEEDS-CHANGE | introduced | Document-file acquisition performs a blocking open before regular-file admission, so a FIFO stalls the generated process until an external writer connects. | adversarial.rs:110: the result channel timed out after one second; opening a writer released the call, which then returned Unavailable. The probe joined its thread before asserting and exited 101. | Any declared document source using its file flag selects ProtectedSource::DocumentFile through argument_text. For the shipped fixture, invoke --operation lookup --schema lookup/v1 --input-file input.fifo reaches this acquisition before native validation or handler dispatch. |

The FIFO measurement is a specific regular-file admission failure, not a claim that every filesystem operation has a latency bound. A native nonblocking open followed by metadata admission on that same descriptor can reject this case without weakening the existing document/protected distinction. The reserved-name fix can refuse unsupported target names before projection; the test permits that refusal.

No additional judgement-only findings.

5. Attacks that did not break the selected contract

Protected owner-only files with multiple hard links are refused; the nonsecret regular document path remains admitted.
Ordinary type validation occurs before protected acquisition and does not dispatch invalid input.
Authored field names normalize to declared wire identities during alias dispatch.
Existing selected ownership, source selection, exact integer, output/error, dynamic validator, help, deterministic projection, and emitted-package cases remained green (27/27).
Edge routing and output ownership were read against their new callers and tests; this pass did not execute the edge binary or claim its full gate.

6. Outside-worktree writes and handoff

None. The only new tracked-source candidate is crates/generate/ess-cli-project/tests/adversarial.rs. Logs/report are .local/tmp/cli-wave/adversary-ess-r1*. Existing source-fixture tests created only task-local temporary packages, which their TempDir owners retired. No build directory or managed tree was manually removed.

The Cargo slot is released to root after these commands. Root owns the two fixes, review-result recording, final gates, commits, recovery and cleanup. This pass releases only lease cli-ess-adversary-r1-20260909. The retained test file remains red until the implementor applies fixes; source authoring may resume after this report.

7. Machine-readable findings

```findings
- file: crates/specify/ess-cli-contract/src/resolve.rs
  line: 337
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The binding compiler admits Cargo-reserved binary names, so a validated binding can emit an unusable standalone package.
- file: crates/generate/ess-cli-project/src/runtime.rs
  line: 121
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Document-file acquisition performs a blocking open before regular-file admission, so a FIFO stalls the generated process until an external writer connects.
```

