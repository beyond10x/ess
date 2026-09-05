---
format: aep.planning-md/1
id: review-result:review-boundaries-2-diagnostic-adversary-pass-1
kind: review-result
status: active
title: Diagnostic adversary pass 1
relations:
- reviews: story:review-kubectl-diagnostic-sanitization
revision: 1
---
unit: story:review-kubectl-diagnostic-sanitization — b26829a571c0569ba2f63a5da495b987397b43a4 plus this test-only working tree
verdict: nothing found
cases: executed 11→14, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: none
git --no-pager diff --stat

```text
 .../ess-kubernetes/tests/fixtures/fake_command.rs  | 22 ++++++
 .../infra/ess-kubernetes/tests/secret_boundary.rs  | 89 ++++++++++++++++++++++
 2 files changed, 111 insertions(+)
```

## 1. Test-only bound

The two paths above are the existing integration test and its Rust fake-command fixture. No production file, existing assertion, planning artifact or Git lifecycle state was changed. The complete patch is retained in target/review-boundaries-2/adversary-tests.patch.

The attacked implementation is commit b26829a571c0569ba2f63a5da495b987397b43a4; its base is c1c23b24cee8f527784b7f8467c21a609710c65e. The reported before count of 11 comes from the implementor's final package report. No package suite was executed before adding these cases.

## 2. New cases and first isolated executions

All three cases were written before any test ran. Each first execution selected exactly one named case and exited zero. No red output was produced. Only test formatting followed those isolated runs; all assertions were retained.

- crates/infra/ess-kubernetes/tests/secret_boundary.rs:412 — signal_terminated_kubectl_discards_both_streams_before_refusing: contexts, current-context and both Secret collection attempts terminate through SIGTERM after writing synthetic values to both streams. ESS must return failure, retain actionable signal status, emit neither value, and preserve the prior destination. The real callers are the CLI contexts verb and scan with explicit or implicit context through the shared kubectl helper. Green. This added case is Linux-specific because it uses /bin/kill to terminate only its own synthetic child PID.
- crates/infra/ess-kubernetes/tests/secret_boundary.rs:438 — retry_failure_reports_the_final_exit_status_without_child_values: the Secret all-namespaces attempt exits 23 and fallback exits 254. The final error must name 254, discard the earlier error and both child streams, preserve both calls and the existing destination. The real caller is scan's unconditional resource fallback. Green.
- crates/infra/ess-kubernetes/tests/secret_boundary.rs:456 — failed_payloads_larger_than_pipe_capacity_are_not_partially_reported_or_written: contexts and Secret failures emit the 35-byte synthetic marker 8,192 additional times to each stream (286,720 additional bytes per stream), then exit 73. ESS must drain the process, report only a bounded diagnostic and status, keep stdout empty, and create no observation. The real callers are contexts and scan; large child diagnostics need no new production path. Green.

First isolated execution 1:

Command: env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-kubernetes --test secret_boundary signal_terminated_kubectl_discards_both_streams_before_refusing -- --exact > target/review-boundaries-2/adversary-case-1.log 2>&1

Exit status: 0.

```text
   Compiling ess-kubernetes v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-kubectl-diagnostic-sanitization/crates/infra/ess-kubernetes)
    Finished `test` profile [unoptimized] target(s) in 0.19s
     Running tests/secret_boundary.rs (target/debug/deps/secret_boundary-9f7580c6900df241)

running 1 test
test signal_terminated_kubectl_discards_both_streams_before_refusing ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.11s

```

First isolated execution 2:

Command: env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-kubernetes --test secret_boundary retry_failure_reports_the_final_exit_status_without_child_values -- --exact > target/review-boundaries-2/adversary-case-2.log 2>&1

Exit status: 0.

```text
    Finished `test` profile [unoptimized] target(s) in 0.03s
     Running tests/secret_boundary.rs (target/debug/deps/secret_boundary-9f7580c6900df241)

running 1 test
test retry_failure_reports_the_final_exit_status_without_child_values ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.11s

```

First isolated execution 3:

Command: env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-kubernetes --test secret_boundary failed_payloads_larger_than_pipe_capacity_are_not_partially_reported_or_written -- --exact > target/review-boundaries-2/adversary-case-3.log 2>&1

Exit status: 0.

```text
    Finished `test` profile [unoptimized] target(s) in 0.03s
     Running tests/secret_boundary.rs (target/debug/deps/secret_boundary-9f7580c6900df241)

running 1 test
test failed_payloads_larger_than_pipe_capacity_are_not_partially_reported_or_written ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.12s

```

## 3. Package run and checks

Command: env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo test --locked -p ess-kubernetes > target/review-boundaries-2/adversary-suite.log 2>&1

Exit status: 0.

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

running 12 tests
test successful_context_listing_preserves_output_bytes ... ok
test failed_secret_subprocess_diagnostics_do_not_echo_secret_values ... ok
test retry_failure_reports_the_final_exit_status_without_child_values ... ok
test absent_optional_secret_fields_and_empty_maps_remain_allowed ... ok
test valid_secret_observation_bytes_remain_compatible ... ok
test signal_terminated_kubectl_discards_both_streams_before_refusing ... ok
test malformed_late_annotations_with_sentinel_keys_refuse_before_any_write ... ok
test failed_payloads_larger_than_pipe_capacity_are_not_partially_reported_or_written ... ok
test invalid_json_after_secret_values_is_refused_without_leaking_or_replacing_output ... ok
test successful_resource_retry_preserves_context_order_and_observation_bytes ... ok
test every_kubectl_caller_uses_value_free_failure_diagnostics ... ok
test malformed_secret_response_corpus_is_refused_without_output_or_diagnostic_values ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.91s

   Doc-tests ess_kubernetes

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

The full package executed 14 cases: 2 library, 0 binary, 12 integration and 0 doc tests; all passed, none failed or were ignored. The integration lane increased 9→12. The three added cases all executed on this Linux checkout.

Formatter command: env TMPDIR="$PWD/target/review-boundaries-2" cargo fmt -p ess-kubernetes --check > target/review-boundaries-2/adversary-fmt.log 2>&1

Output: empty. Exit status: 0.

Strict lint command: env TMPDIR="$PWD/target/review-boundaries-2" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 cargo clippy --locked -p ess-kubernetes --all-targets -- -D warnings > target/review-boundaries-2/adversary-clippy.log 2>&1

Exit status: 0.

```text
    Checking ess-kubernetes v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-kubectl-diagnostic-sanitization/crates/infra/ess-kubernetes)
    Finished `dev` profile [unoptimized] target(s) in 0.15s
```

Both edited test files were formatted with rustfmt --edition 2021 before the package run. git diff --check exited zero with empty output. The existing fake-command compiler also ran with -Dwarnings during every integration process.

Available storage was 125,606,780,928 bytes before testing, 125,179,068,416 before the complete package run, and 124,929,929,216 after checks; all exceeded the 8,589,934,592-byte reserve. Cargo used the prescribed sccache wrapper and compact profiles in this worktree's target. No build directory was removed.

## 4. Judgement findings

Nothing found.

## 5. Attacked without a break

- Signal-terminated context/resource subprocesses kept both captured streams out of diagnostics and preserved retry/output behavior.
- Different first/fallback exit codes retained the final status without restoring argument or child-output disclosure.
- Child output exceeding pipe capacity was drained without observable truncation leakage, observation creation or unbounded ESS diagnostics.
- The full existing malformed-Secret, redaction, valid-byte, caller-matrix and successful-fallback assertions remained green.
- This pass used synthetic offline processes; it did not contact a cluster or mutate the implementation to measure a sanitizer guard.

## 6. Written paths

Outside the assigned worktree: none. Deliberate source writes are exactly the two test paths in the leading diff.

Assigned scratch: /home/timo/.local/state/worktree/trees/b10x/ess/review-kubectl-diagnostic-sanitization/target/review-boundaries-2/. Retained files are adversary-case-1.log, adversary-case-2.log, adversary-case-3.log, adversary-suite.log, adversary-fmt.log, adversary-clippy.log, adversary-tests.patch and adversary-pass-1.md. The existing test fixture_root helper also retains its generated Rust fake binaries, synthetic invocation logs and observation fixtures beneath this worktree's Cargo target temporary directory, as it did for the implementation's tests. No external target directory was selected.

```findings
[]
```
