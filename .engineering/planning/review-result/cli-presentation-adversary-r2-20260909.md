---
format: aep.planning-md/1
id: review-result:cli-presentation-adversary-r2-20260909
kind: review-result
status: active
title: CLI presentation final adversarial review
relations:
- reviews: story:cli-presentation-binding
revision: 1
---
unit: story:cli-presentation-binding — ATTACK2 (final attack) of work/cli-binding-latest-20260909 over 113f5925ebb6a0a57a73e661687d9fa5deddb0b8
verdict: NEEDS-CHANGE
cases: executed 32→39, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: route the new integer-token correction; planning, final gate and cleanup remain coordinator-owned

1. git --no-pager diff --stat

Command: git --no-pager diff --no-index --stat /dev/null crates/generate/ess-cli-project/tests/adversarial_r2.rs

```text
 .../ess-cli-project/tests/adversarial_r2.rs        | 406 +++++++++++++++++++++
 1 file changed, 406 insertions(+)
```

This is this adversary's complete source delta. The assigned shared checkout already contained implementation, planning, edge and consumer-coverage changes; its aggregate diff is not an adversary edit. This attack changed no production file or existing test, did not mutate a file under attack even temporarily, ran no AEP command, and made no commit, branch change or publication. The only non-test writes are the report and small logs under the assigned .local/tmp/cli-wave scratch.

Source stamps taken after authoring and before the first test run, then checked unchanged after the final test run with sha256sum -c (all seven OK):

```text
e6bb0a4ecf475afaeb2f476b95503d64f040a4420862a141ce2b8706ab090946  crates/specify/ess-cli-contract/src/lib.rs
36619c847e00666622b846772ebe726fb6bbdbcf596f2fb1f6f5d5ade84e30f2  crates/specify/ess-cli-contract/src/wire.rs
c784b63fd293347e4abd73ca43f82146d8ed4d5a9e00c5d1bfde1e4df911467f  crates/specify/ess-cli-contract/src/resolve.rs
e62949c04402ae40f612784a4dd811ed61337f1b6f749e5759aad37585597c78  crates/generate/ess-cli-project/src/lib.rs
c86234d09e03e9a1c54ae89851dbb7abb1957ceebf45b42a71af1b0d4c5d894d  crates/generate/ess-cli-project/src/runtime.rs
e02f45c3e60ae0c466ac11b655416b9da6387f2891b9e9eefe4e8d618aca4ce2  crates/edge/ess-cli/src/cli_binding.rs
e1a8f0f23d6980affa85b2952a2a92a8561bac2cb4c564390e11fdbf04afc9fb  docs/design/cli-presentation-binding.md
```

Final new-test SHA-256:
b85285ba2d4b10f02a1f55dc16dd237c9cc6f0d08ccb5c5af05dd16e5915669e  crates/generate/ess-cli-project/tests/adversarial_r2.rs

2. Cases added and first isolated execution

All seven cases were authored before any case or suite was executed. The first execution compiled successfully and ran exactly the intended case. Rustfmt was applied only to the new test file before that execution.

| New case in crates/generate/ess-cli-project/tests/adversarial_r2.rs | Assertion | Current result |
|---|---|---|
| :79 exact_json_integer_negative_zero_reaches_scalar_list_and_map_handlers | The documented JSON integer token -0 reaches a handler as integer zero through a scalar option, List element and Map value; all three inputs come from an authored model and binding | RED: all three exit 2, cli_input, zero dispatches |
| :122 optional_protected_omission_and_utf8_byte_limit_hold_at_the_handler_seam | Optional omission performs no source read; exactly 1 MiB UTF-8 is admitted; two bytes beyond the limit are refused without dispatch or output echo | GREEN |
| :160 document_inline_and_acquired_text_use_the_same_utf8_byte_bound | Inline and custom-stdin document paths admit exactly 1 MiB and refuse larger UTF-8 text without echo | GREEN |
| :190 native_regular_document_symlinks_and_protected_parent_symlinks_are_admitted | Document final symlinks and protected parent symlinks are admitted as documented; protected final symlinks, oversized files and invalid UTF-8 are refused | GREEN |
| :226 aliases_preserve_custom_globals_and_double_dash_stops_output_selection | Root/two-token aliases preserve custom global context; -- makes subsequent output-looking text ordinary positional data and stops preliminary mode selection | GREEN |
| :268 non_utf8_argv_is_refused_in_selected_json_mode_without_dispatch | Real Unix OsString bytes invalid as UTF-8 produce stable JSON cli_parse without a handler call | GREEN |
| :326 dynamic_result_and_both_error_replies_preserve_finite_failure_policy | InvalidValue/Unavailable/Interrupted at success, operational-error and usage-error reply validation preserve stable codes, finite exits and empty internal data | GREEN |

Environment for every Cargo command in this report:

```console
env -u CARGO_TARGET_DIR -u CARGO_ENCODED_RUSTFLAGS RUSTUP_TOOLCHAIN=1.98.1 RUSTFLAGS='-C link-arg=-fuse-ld=lld' RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 TMPDIR="$PWD/.local/tmp/cli-wave"
```

All Cargo activity was serialized under the coordinator's transferred slot. Task target target/cli-gate-integration was used; the existing generated-package fixture created its own disposable build under assigned .local/tmp/cli-wave/ess-cli-fixture-kERRkJ. Disk inspection before execution reported 38 GiB available. No unmanaged build location was selected.

First isolated command (under the environment above):

```console
cargo test --target-dir target/cli-gate-integration --locked --offline -p ess-cli-project --test adversarial_r2 exact_json_integer_negative_zero_reaches_scalar_list_and_map_handlers -- --exact --nocapture
```

Exit status: 101. Actual output, retained verbatim in adversary-ess-r2-negative-zero.log:

```text
   Compiling ess-cli-project v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/crates/generate/ess-cli-project)
    Finished `test` profile [unoptimized] target(s) in 0.28s
     Running tests/adversarial_r2.rs (target/cli-gate-integration/debug/deps/adversarial_r2-ead8f216e60feb5d)

running 1 test
Integer: output=ProcessOutput { exit_code: 2, stdout: "", stderr: "{\"error\":{\"code\":\"cli_input\",\"data\":{}},\"ok\":false}\n" }, dispatches=0
List<Integer>: output=ProcessOutput { exit_code: 2, stdout: "", stderr: "{\"error\":{\"code\":\"cli_input\",\"data\":{}},\"ok\":false}\n" }, dispatches=0
Map<String, Integer>: output=ProcessOutput { exit_code: 2, stdout: "", stderr: "{\"error\":{\"code\":\"cli_input\",\"data\":{}},\"ok\":false}\n" }, dispatches=0

thread 'exact_json_integer_negative_zero_reaches_scalar_list_and_map_handlers' (1004186) panicked at crates/generate/ess-cli-project/tests/adversarial_r2.rs:113:5:
the documented exact integer token -0 must be admitted at each typed argv position
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test exact_json_integer_negative_zero_reaches_scalar_list_and_map_handlers ... FAILED

failures:

failures:
    exact_json_integer_negative_zero_reaches_scalar_list_and_map_handlers

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-cli-project --test adversarial_r2`
```

3. Targeted suite, followed by new-test hygiene verification

Command (under the same environment):

```console
cargo test --target-dir target/cli-gate-integration --locked --offline --no-fail-fast -p ess-cli-contract -p ess-cli-project --test binding --test projection --test adversarial --test adversarial_r2 -- --nocapture
```

Exit status: 101. Actual output, retained verbatim in adversary-ess-r2-suite.log:

```text
   Compiling ess-cli-contract v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/crates/specify/ess-cli-contract)
   Compiling ess-cli-project v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/crates/generate/ess-cli-project)
    Finished `test` profile [unoptimized] target(s) in 0.51s
     Running tests/binding.rs (target/cli-gate-integration/debug/deps/binding-123096e1d911e467)

running 11 tests
test integer_values_use_the_models_exact_signed_64_bit_range ... ok
test refuses_unsupported_constraints_and_primitive_promises ... ok
test supported_nested_value_shapes_are_closed_and_validate_every_element ... ok
test inputless_local_calls_require_explicit_null_and_no_fictional_struct ... ok
test closes_unknown_format_and_fields_at_every_reader_layer ... ok
test resolves_value_types_without_inventing_a_component_or_entity ... ok
test refuses_unresolved_types_fields_and_ambiguous_cli_sources ... ok
test service_forward_command_preserves_the_model_owner_and_input_identity ... ok
test service_forward_parameterized_view_resolves_rows_without_changing_ownership ... ok
test service_forward_refuses_foreign_owners_and_mismatched_input_or_view_shapes ... ok
test refuses_recursive_nested_unresolved_and_constrained_types ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/adversarial.rs (target/cli-gate-integration/debug/deps/adversarial-a91306568c9d8858)

running 5 tests
test protected_hardlinks_are_refused_while_regular_documents_remain_admitted ... ok
test document_file_refuses_a_fifo_without_waiting_for_an_external_writer ... ok
test authored_field_names_dispatch_using_resolved_wire_names ... ok
test invalid_ordinary_values_are_refused_before_protected_acquisition ... ok
test admitted_binary_names_emit_cargo_accepted_targets ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/adversarial_r2.rs (target/cli-gate-integration/debug/deps/adversarial_r2-ead8f216e60feb5d)

running 7 tests
test non_utf8_argv_is_refused_in_selected_json_mode_without_dispatch ... ok
test aliases_preserve_custom_globals_and_double_dash_stops_output_selection ... ok
test native_regular_document_symlinks_and_protected_parent_symlinks_are_admitted ... ok
Integer: output=ProcessOutput { exit_code: 2, stdout: "", stderr: "{\"error\":{\"code\":\"cli_input\",\"data\":{}},\"ok\":false}\n" }, dispatches=0
List<Integer>: output=ProcessOutput { exit_code: 2, stdout: "", stderr: "{\"error\":{\"code\":\"cli_input\",\"data\":{}},\"ok\":false}\n" }, dispatches=0
Map<String, Integer>: output=ProcessOutput { exit_code: 2, stdout: "", stderr: "{\"error\":{\"code\":\"cli_input\",\"data\":{}},\"ok\":false}\n" }, dispatches=0

thread 'exact_json_integer_negative_zero_reaches_scalar_list_and_map_handlers' (1008286) panicked at crates/generate/ess-cli-project/tests/adversarial_r2.rs:113:5:
the documented exact integer token -0 must be admitted at each typed argv position
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test exact_json_integer_negative_zero_reaches_scalar_list_and_map_handlers ... FAILED
test optional_protected_omission_and_utf8_byte_limit_hold_at_the_handler_seam ... ok
test dynamic_result_and_both_error_replies_preserve_finite_failure_policy ... ok
test document_inline_and_acquired_text_use_the_same_utf8_byte_bound ... ok

failures:

failures:
    exact_json_integer_negative_zero_reaches_scalar_list_and_map_handlers

test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p ess-cli-project --test adversarial_r2`
     Running tests/projection.rs (target/cli-gate-integration/debug/deps/projection-803f2fd596653bb8)

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
test acquisition_failure_and_interruption_are_safe_and_do_not_dispatch ... ok
test dynamic_refusals_distinguish_input_resolution_and_interruption ... ok
test parser_refusals_follow_json_selection_without_echoing_values ... ok
test dynamic_input_result_and_errors_require_the_native_validator ... ok
test typed_application_usage_errors_preserve_the_declared_error_contract ... ok
test handler_results_errors_and_interruptions_follow_process_contract ... ok
test help_and_completions_are_executable_and_do_not_dispatch ... ok
test typed_values_optional_omission_and_unit_payload_are_enforced ... ok
test integer_input_preserves_signed_boundaries_and_refuses_unsigned_overflow ... ok
test native_file_acquisition_checks_permissions_symlinks_size_and_utf8 ... ok
test all_protected_channels_are_exclusive_and_never_accept_secret_argv ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Locking 34 packages to latest compatible versions
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.24
   Compiling utf8parse v0.2.2
   Compiling quote v1.0.47
   Compiling anstyle-parse v1.0.0
   Compiling libc v0.2.189
   Compiling anstyle-query v1.1.5
   Compiling serde_core v1.0.229
   Compiling colorchoice v1.0.5
   Compiling is_terminal_polyfill v1.70.2
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
   Compiling linux-raw-sys v0.12.1
   Compiling bitflags v2.13.1
   Compiling itoa v1.0.18
   Compiling memchr v2.8.3
   Compiling rpassword v7.5.4
   Compiling clap_complete v4.6.9
   Compiling demo-cli-contract v0.0.0 (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/.local/tmp/cli-wave/ess-cli-fixture-kERRkJ)
    Finished `test` profile [unoptimized] target(s) in 6.42s
     Running unittests src/lib.rs (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/.local/tmp/cli-wave/ess-cli-fixture-kERRkJ/target/debug/deps/cli_contract-0fbaf74542ccaea9)
     Running unittests src/main.rs (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/.local/tmp/cli-wave/ess-cli-fixture-kERRkJ/target/debug/deps/demo-19ab20e5520cf4fa)
     Running tests/process.rs (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/.local/tmp/cli-wave/ess-cli-fixture-kERRkJ/target/debug/deps/process-ccf40e85d9866734)
   Doc-tests cli_contract

test generated_package_compiles_offline_and_executes_process_fixtures ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.55s

error: 1 target failed:
    `-p ess-cli-project --test adversarial_r2`
```

Counting: binding 11 + previous adversarial 5 + new adversarial_r2 7 + projection 4 + generated-package process 12 = 39 selected executable cases. Results: 38 passed, 1 failed. The before count 32 comes from the correction state's reported green 11 + 5 + 4 + 12 cases in ess-fix-r1.md, not from a pre-attack run. The separate evolving consumer suites and edge suite are not selected or counted here. The historical 8-case consumer lane in ess-fix-r1.md is likewise excluded. No full ESS or integration-gate result is claimed.

New-test strict Clippy initially found a 107-line function in the new test file. Actual output from adversary-ess-r2-clippy.log:

```text
    Checking ess-cli-project v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/crates/generate/ess-cli-project)
error: this function has too many lines (107/100)
   --> crates/generate/ess-cli-project/tests/adversarial_r2.rs:298:1
    |
298 | fn dynamic_result_and_both_error_replies_preserve_finite_failure_policy() {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_lines
    = note: `-D clippy::too-many-lines` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::too_many_lines)]`

error: could not compile `ess-cli-project` (test "adversarial_r2") due to 1 previous error
```

This was a test-hygiene issue, not a production finding. Only the new test's local RejectReply declaration/implementation moved to module scope; no assertion, input, case name or production source changed. The final command was:

```console
cargo clippy --target-dir target/cli-gate-integration --locked --offline -p ess-cli-project --test adversarial_r2 -- -D warnings
```

Exit status: 0. Final output:

```text
    Checking ess-cli-project v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/crates/generate/ess-cli-project)
    Finished `dev` profile [unoptimized] target(s) in 0.17s
```

After that test-only refactor, all seven fresh cases were rerun:

```console
cargo test --target-dir target/cli-gate-integration --locked --offline -p ess-cli-project --test adversarial_r2 -- --nocapture
```

Exit status: 101, same one red and six green; actual final output:

```text
   Compiling ess-cli-project v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909/crates/generate/ess-cli-project)
    Finished `test` profile [unoptimized] target(s) in 0.28s
     Running tests/adversarial_r2.rs (target/cli-gate-integration/debug/deps/adversarial_r2-ead8f216e60feb5d)

running 7 tests
test non_utf8_argv_is_refused_in_selected_json_mode_without_dispatch ... ok
test aliases_preserve_custom_globals_and_double_dash_stops_output_selection ... ok
test native_regular_document_symlinks_and_protected_parent_symlinks_are_admitted ... ok
Integer: output=ProcessOutput { exit_code: 2, stdout: "", stderr: "{\"error\":{\"code\":\"cli_input\",\"data\":{}},\"ok\":false}\n" }, dispatches=0
List<Integer>: output=ProcessOutput { exit_code: 2, stdout: "", stderr: "{\"error\":{\"code\":\"cli_input\",\"data\":{}},\"ok\":false}\n" }, dispatches=0
Map<String, Integer>: output=ProcessOutput { exit_code: 2, stdout: "", stderr: "{\"error\":{\"code\":\"cli_input\",\"data\":{}},\"ok\":false}\n" }, dispatches=0

thread 'exact_json_integer_negative_zero_reaches_scalar_list_and_map_handlers' (1041426) panicked at crates/generate/ess-cli-project/tests/adversarial_r2.rs:113:5:
the documented exact integer token -0 must be admitted at each typed argv position
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test exact_json_integer_negative_zero_reaches_scalar_list_and_map_handlers ... FAILED
test optional_protected_omission_and_utf8_byte_limit_hold_at_the_handler_seam ... ok
test dynamic_result_and_both_error_replies_preserve_finite_failure_policy ... ok
test document_inline_and_acquired_text_use_the_same_utf8_byte_bound ... ok

failures:

failures:
    exact_json_integer_negative_zero_reaches_scalar_list_and_map_handlers

test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p ess-cli-project --test adversarial_r2`
```

4. Findings and reachable origin

| File:line | Verdict | Origin | Finding / what was measured | What reaches it |
|---|---|---|---|---|
| crates/generate/ess-cli-project/src/runtime.rs:525 | NEEDS-CHANGE | introduced | Exact JSON integer token -0 is refused as cli_input for scalar Integer, List<Integer> and Map<String, Integer> argv inputs, contradicting docs/design/cli-presentation-binding.md:84-86. The first isolated test at tests/adversarial_r2.rs:113 and suite both exited 101; all three process runs exited 2 with zero handler calls. | An ordinary authored ESS Input struct plus an admitted ess-cli/1 local callable and option mapping, executed as attack run --value -0 --render=json (or list/map JSON). Binding::from_yaml → compile → runtime::run → payload reaches the refusal. The projector emits the same runtime.rs verbatim and generated src/main.rs passes env::args_os() into it. No forged Plan or impossible model is involved. |

The measured discrepancy covers the exact source hashes above in the working tree based on 113f5925ebb6a0a57a73e661687d9fa5deddb0b8. The new runtime and wire crates are absent at that base (git ls-tree returned no entries for their paths); this unit introduces both admission and execution, so origin is introduced. The base tree was never checked out or mutated.

The owning document explicitly accepts JSON integer tokens in the signed 64-bit range while excluding decimal/exponent spellings; -0 is an integer token denoting zero. Current payload parsing uses serde_json::from_str and then Shape::Integer's is_i64 check. Correction should preserve lexical integer admission through nested values while retaining refusal of decimal/exponent tokens and out-of-range integers; broad acceptance of integer-valued floating-point values would not satisfy the existing contract. No implementation correction was attempted here.

No additional judgement findings.

5. Attacks that did not break

- The four Cargo-reserved binary names and Unix document FIFO fix remain green in the unchanged ATTACK1 suite.
- Optional protected input omission and the exact UTF-8 byte boundary hold through custom acquisition and dispatch.
- Native document and protected file symlink distinctions, oversized reads and malformed UTF-8 match their documented admission.
- Root/two-token aliases preserve custom global context, and -- prevents output-looking positional data from selecting parser JSON mode.
- Invalid Unix argv encoding receives a safe JSON parser refusal without dispatch.
- All nine finite dynamic result/error validation combinations preserve their documented code/exit/data policy.
- Existing binding and emitted-package process cases remain green; this finite attack makes no full-gate or production-handler claim.

6. Outside-worktree paths and handoff

None. Task-authored files are the new test and .local/tmp/cli-wave/adversary-ess-r2.md, adversary-ess-r2-source-before.sha256, adversary-ess-r2-negative-zero.log, adversary-ess-r2-suite.log, adversary-ess-r2-clippy.log, adversary-ess-r2-clippy-final.log and adversary-ess-r2-final.log. Compiler output stayed in target/cli-gate-integration or the existing fixture's task TempDir; no build tree was deleted manually.

Cargo was returned to root after final execution. The adversary releases only session ess-cli-final-adversary-20260909; root owns staging, correction routing, planning records and cleanup of cli-binding-ess-20260909 at /home/timo/.local/state/worktree/trees/b10x/ess/cli-binding-ess-20260909. No third attack is requested or started.

```findings
- file: crates/generate/ess-cli-project/src/runtime.rs
  line: 525
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Exact JSON integer token -0 is refused as cli_input for scalar Integer, List<Integer> and Map<String, Integer> argv inputs, contradicting docs/design/cli-presentation-binding.md:84-86.
```

