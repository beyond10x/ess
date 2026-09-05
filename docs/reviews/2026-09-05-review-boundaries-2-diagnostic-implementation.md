unit:                   story:review-kubectl-diagnostic-sanitization — Keep untrusted kubectl stderr out of ESS diagnostics
verdict:                green
cases:                  executed 8→11, red 2
origin:                 n/a
wrote-outside-worktree: none
needs-coordinator:      no

## 1. Unit and acceptance

story:review-kubectl-diagnostic-sanitization: a failed kubectl process cannot copy untrusted stderr into ESS diagnostics, including both failed Secret collection attempts.

Base: c1c23b24cee8f527784b7f8467c21a609710c65e. Source changes are uncommitted in the assigned managed worktree. The AEP graph has no depends_on edge for this unit. Integration, adversarial review, lifecycle moves, commits, publication and worktree cleanup remain coordinator actions.

The shared private helper now accepts an adapter-owned static operation label. A failed child process contributes only its exit status to the error; stdout, stderr and complete invocation arguments are discarded. The caller-selected context can originate from the CLI or current-context output, so neither route enters diagnostics. Successful stdout and all invocation arguments are unchanged.

The original no-disclosure assertion is restored from the preserved adversarial patch. Existing refusal, destination preservation and empty stdout assertions remain. The temporary known-disclosure expectation is replaced by value-free operation/exit-status checks.

The class is every invocation through this credential-edge kubectl helper. The failure matrix derives resource cases from KINDS, so another kind cannot silently escape that matrix. It covers list-contexts, current-context, every cluster/resource collection (including namespaces, nodes and Secrets), both -A and fallback attempts, synthetic UTF-8 and invalid UTF-8 stderr, failed stdout, and explicit/implicit untrusted contexts. A successful retry case verifies the complete collection order, exactly two resource attempts per kind, context selection and canonical observation byte equality. A separate context-listing case freezes successful stdout bytes.

## 2. Actual diff

```text
 crates/infra/ess-kubernetes/src/lib.rs             |  33 +++--
 .../ess-kubernetes/tests/fixtures/fake_command.rs  |  72 +++++++--
 .../infra/ess-kubernetes/tests/secret_boundary.rs  | 162 +++++++++++++++++++--
 3 files changed, 232 insertions(+), 35 deletions(-)
```

Scope confirmations:

| Scope hypothesis | Actual evidence | Disposition |
|---|---|---|
| Package and three cited implementation/test files | The observed diff touches only crates/infra/ess-kubernetes/src/lib.rs, tests/secret_boundary.rs and tests/fixtures/fake_command.rs. | Confirmed. |
| Static operation/status diagnostics satisfy the inferred mechanism | src/lib.rs:34 centralizes refusal rendering; the four call sites are at lines 49, 61, 72 and 77. Applying exactly the proposed static-label/status mechanism turned the full package from two disclosure failures to 11 passes in mechanism-test.log. | Confirmed by one direct measurement before further source changes; no other production mechanism was added. |
| Resource helper coverage includes cluster metadata | scan uses the same collection loop for namespaces/nodes and all other entries in KINDS. There is no separate cluster-metadata helper call in this checkout. The matrix enumerates KINDS directly. | Confirmed current call surface; no additional operation invented. |
| Existing fixture needs configurable failure modes | tests/fixtures/fake_command.rs:16 identifies contexts/current-context/all resource shapes; lines 38–60 provide per-operation/all-resource failures, first-attempt-only failures and invalid UTF-8 stderr. | Confirmed; original ESS_TEST_SECRET_FAILURE path retained for the preserved red reproduction. |
| No documents or persisted-format changes required | Successful stdout handling, context storage, bundle fields, KINDS, sanitizer and serialization remain unchanged; existing frozen observation test and successful fallback byte comparison pass. | Confirmed. |
| Package-wide reservation is sufficient | git diff --stat names exactly three files under the reserved package; no other source or planning changes. | Confirmed. |

## 3. Test-first red and mechanism measurement

The unchanged base first ran the exact complete package lane, exit 0, 2 library + 6 integration tests:

Command: env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-kubernetes

```text
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.24
   Compiling typenum v1.20.1
   Compiling serde_core v1.0.229
   Compiling utf8parse v0.2.2
   Compiling zmij v1.0.23
   Compiling anstyle v1.0.14
   Compiling anstyle-query v1.1.5
   Compiling is_terminal_polyfill v1.70.2
   Compiling colorchoice v1.0.5
   Compiling strsim v0.11.1
   Compiling serde_json v1.0.151
   Compiling const-oid v0.10.2
   Compiling clap_lex v1.1.0
   Compiling heck v0.5.0
   Compiling serde v1.0.229
   Compiling itoa v1.0.18
   Compiling cfg-if v1.0.4
   Compiling cpufeatures v0.3.1
   Compiling memchr v2.8.3
   Compiling hybrid-array v0.4.14
   Compiling anstyle-parse v1.0.0
   Compiling block-buffer v0.12.1
   Compiling crypto-common v0.2.2
   Compiling anstream v1.0.0
   Compiling digest v0.11.3
   Compiling clap_builder v4.6.6
   Compiling sha2 v0.11.0
   Compiling syn v3.0.4
   Compiling serde_derive v1.0.229
   Compiling clap_derive v4.6.4
   Compiling clap v4.6.6
   Compiling ess-kubernetes v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-kubectl-diagnostic-sanitization/crates/infra/ess-kubernetes)
    Finished `test` profile [unoptimized] target(s) in 6.84s
     Running unittests src/lib.rs (target/debug/deps/ess_kubernetes-5f388cf73a35bac9)

running 2 tests
test tests::malformed_secret_lists_are_refused_before_any_output_can_be_written ... ok
test tests::secret_values_and_last_applied_configuration_never_survive_sanitization ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/ess_kubernetes-fc6b8b90c0cbb6e8)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/secret_boundary.rs (target/debug/deps/secret_boundary-9f7580c6900df241)

running 6 tests
test known_kubectl_stderr_disclosure_is_confined_to_the_failure_diagnostic ... ok
test absent_optional_secret_fields_and_empty_maps_remain_allowed ... ok
test valid_secret_observation_bytes_remain_compatible ... ok
test malformed_late_annotations_with_sentinel_keys_refuse_before_any_write ... ok
test invalid_json_after_secret_values_is_refused_without_leaking_or_replacing_output ... ok
test malformed_secret_response_corpus_is_refused_without_output_or_diagnostic_values ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.87s

   Doc-tests ess_kubernetes

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

After writing tests and extending only the fake process fixture, before changing src/lib.rs, the same command exited 101:

```text
   Compiling ess-kubernetes v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-kubectl-diagnostic-sanitization/crates/infra/ess-kubernetes)
    Finished `test` profile [unoptimized] target(s) in 0.21s
     Running unittests src/lib.rs (target/debug/deps/ess_kubernetes-5f388cf73a35bac9)

running 2 tests
test tests::malformed_secret_lists_are_refused_before_any_output_can_be_written ... ok
test tests::secret_values_and_last_applied_configuration_never_survive_sanitization ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/ess_kubernetes-fc6b8b90c0cbb6e8)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/secret_boundary.rs (target/debug/deps/secret_boundary-9f7580c6900df241)

running 9 tests
test successful_context_listing_preserves_output_bytes ... ok
test every_kubectl_caller_uses_value_free_failure_diagnostics ... FAILED
test failed_secret_subprocess_diagnostics_do_not_echo_secret_values ... FAILED
test valid_secret_observation_bytes_remain_compatible ... ok
test absent_optional_secret_fields_and_empty_maps_remain_allowed ... ok
test malformed_late_annotations_with_sentinel_keys_refuse_before_any_write ... ok
test invalid_json_after_secret_values_is_refused_without_leaking_or_replacing_output ... ok
test successful_resource_retry_preserves_context_order_and_observation_bytes ... ok
test malformed_secret_response_corpus_is_refused_without_output_or_diagnostic_values ... ok

failures:

---- every_kubectl_caller_uses_value_free_failure_diagnostics stdout ----

thread 'every_kubectl_caller_uses_value_free_failure_diagnostics' (4176009) panicked at crates/infra/ess-kubernetes/tests/secret_boundary.rs:277:17:
contexts leaked process output
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- failed_secret_subprocess_diagnostics_do_not_echo_secret_values stdout ----

thread 'failed_secret_subprocess_diagnostics_do_not_echo_secret_values' (4176010) panicked at crates/infra/ess-kubernetes/tests/secret_boundary.rs:224:5:
the failed Secret subprocess diagnostic leaked the synthetic sentinel


failures:
    every_kubectl_caller_uses_value_free_failure_diagnostics
    failed_secret_subprocess_diagnostics_do_not_echo_secret_values

test result: FAILED. 7 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.86s

error: test failed, to rerun pass `-p ess-kubernetes --test secret_boundary`
```

The preserved Secret assertion and the new shared-helper matrix failed specifically on process-output disclosure. The positive retry and context-list tests already passed on the baseline behavior.

Then the proposed mechanism alone (static operation label + process status, no stderr or arguments) was applied. The same complete package command exited 0:

```text
   Compiling ess-kubernetes v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-kubectl-diagnostic-sanitization/crates/infra/ess-kubernetes)
    Finished `test` profile [unoptimized] target(s) in 0.46s
     Running unittests src/lib.rs (target/debug/deps/ess_kubernetes-5f388cf73a35bac9)

running 2 tests
test tests::malformed_secret_lists_are_refused_before_any_output_can_be_written ... ok
test tests::secret_values_and_last_applied_configuration_never_survive_sanitization ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/ess_kubernetes-fc6b8b90c0cbb6e8)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/secret_boundary.rs (target/debug/deps/secret_boundary-9f7580c6900df241)

running 9 tests
test successful_context_listing_preserves_output_bytes ... ok
test failed_secret_subprocess_diagnostics_do_not_echo_secret_values ... ok
test absent_optional_secret_fields_and_empty_maps_remain_allowed ... ok
test valid_secret_observation_bytes_remain_compatible ... ok
test malformed_late_annotations_with_sentinel_keys_refuse_before_any_write ... ok
test invalid_json_after_secret_values_is_refused_without_leaking_or_replacing_output ... ok
test successful_resource_retry_preserves_context_order_and_observation_bytes ... ok
test every_kubectl_caller_uses_value_free_failure_diagnostics ... ok
test malformed_secret_response_corpus_is_refused_without_output_or_diagnostic_values ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s

   Doc-tests ess_kubernetes

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 4. Final checks and observed counts

Final command: env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-kubernetes

Exit status: 0.

```text
   Compiling ess-kubernetes v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-kubectl-diagnostic-sanitization/crates/infra/ess-kubernetes)
    Finished `test` profile [unoptimized] target(s) in 0.22s
     Running unittests src/lib.rs (target/debug/deps/ess_kubernetes-5f388cf73a35bac9)

running 2 tests
test tests::malformed_secret_lists_are_refused_before_any_output_can_be_written ... ok
test tests::secret_values_and_last_applied_configuration_never_survive_sanitization ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/ess_kubernetes-fc6b8b90c0cbb6e8)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/secret_boundary.rs (target/debug/deps/secret_boundary-9f7580c6900df241)

running 9 tests
test successful_context_listing_preserves_output_bytes ... ok
test failed_secret_subprocess_diagnostics_do_not_echo_secret_values ... ok
test absent_optional_secret_fields_and_empty_maps_remain_allowed ... ok
test valid_secret_observation_bytes_remain_compatible ... ok
test malformed_late_annotations_with_sentinel_keys_refuse_before_any_write ... ok
test invalid_json_after_secret_values_is_refused_without_leaking_or_replacing_output ... ok
test successful_resource_retry_preserves_context_order_and_observation_bytes ... ok
test every_kubectl_caller_uses_value_free_failure_diagnostics ... ok
test malformed_secret_response_corpus_is_refused_without_output_or_diagnostic_values ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.86s

   Doc-tests ess_kubernetes

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Runner lane counts, read from each command's printed summary:

- Complete package aggregate: executed 8 → 11, exit 0.
- Library unit lane: executed 2 → 2, exit 0. No cases added to this lane.
- Binary unit lane: executed 0 → 0, exit 0. No cases added to this lane.
- secret_boundary integration lane: executed 6 → 9, exit 0.
- Doc-test lane: executed 0 → 0, exit 0. No cases added to this lane.

Formatter command: env TMPDIR="$PWD/target/review-boundaries-2" cargo fmt -p ess-kubernetes --check

Output: empty. Exit status: 0. The standalone fixture was also formatted with rustfmt --edition 2021.

Strict lint command: env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo clippy --locked -p ess-kubernetes --all-targets -- -D warnings

Exit status: 0.

```text
    Checking ess-kubernetes v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-kubectl-diagnostic-sanitization/crates/infra/ess-kubernetes)
    Finished `dev` profile [unoptimized] target(s) in 0.11s
```

The initial lint run exited 101 on two test-only style findings (single-character pattern and format-collect); both were corrected, with no assertion relaxation. Its exact output remains in clippy-initial.log. The final suite was rerun after those edits.

Credential guard mutation: temporarily changed sanitizer iteration from ["data", "stringData"] to ["data"], then ran:

env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-kubernetes --lib secret_values_and_last_applied_configuration_never_survive_sanitization

Exit status: 101; executed 1 test, 1 failed.

```text
   Compiling ess-kubernetes v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-kubectl-diagnostic-sanitization/crates/infra/ess-kubernetes)
    Finished `test` profile [unoptimized] target(s) in 0.15s
     Running unittests src/lib.rs (target/debug/deps/ess_kubernetes-5f388cf73a35bac9)

running 1 test
test tests::secret_values_and_last_applied_configuration_never_survive_sanitization ... FAILED

failures:

---- tests::secret_values_and_last_applied_configuration_never_survive_sanitization stdout ----

thread 'tests::secret_values_and_last_applied_configuration_never_survive_sanitization' (4184449) panicked at crates/infra/ess-kubernetes/src/lib.rs:206:13:
raw secret survived sanitization: RAW-STRING-SECRET
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::secret_values_and_last_applied_configuration_never_survive_sanitization

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-kubernetes --lib`
```

The original two-field sanitizer iteration was restored before final checks. The final full package lane executes that named guard successfully; git diff confirms the sanitizer has no net change.

git diff --check exited 0 with no output.

Disk measurements: before baseline 112,219,242,496 bytes free; after baseline 112,137,928,704; after mechanism 111,179,202,560; after final checks 125,687,275,520. The 8,589,934,592-byte reserve remained satisfied. Final target measurement: 145,944 KiB. No build directory was removed.

## 5. Deliberate boundaries and coordinator decisions

No downstream library, persisted type, format version, context-selection rule, retry policy, collection kind, observation byte contract, public documentation or live system was changed. The central helper uses a generic resource-operation label; the existing progress output identifies the particular fixed kind. Failed subprocess details are intentionally reduced to value-free operation and process exit status.

No live cluster was used. All fixtures carry synthetic sentinels and use the Rust fake kubectl/date executable. No planning-store mutation, Git staging, commit, branch or worktree lifecycle operation was performed. The full workspace offline gate remains an integration check owned by the coordinator.

No coordinator patch is needed.

## 6. Outside paths and retained evidence

None. All deliberately written source, fixtures, generated fake executables, temporary outputs, logs and this report are inside the assigned worktree. Cargo used the brief's required sccache wrapper and worktree-local target, without CARGO_TARGET_DIR.

Evidence directory: /home/timo/.local/state/worktree/trees/b10x/ess/review-kubectl-diagnostic-sanitization/target/review-boundaries-2/

Retained files: baseline-test.log/.exit, red-test.log/.exit, mechanism-test.log/.exit, mutation-test.log/.exit, final-test.log/.exit, fmt-check.log/.exit, clippy.log/.exit, clippy-initial.log/.exit, format.exit, fixture-format.exit and implementation-report.md. Synthetic invocation/output fixtures remain under the worktree's Cargo target temporary directory for coordinator inspection.

