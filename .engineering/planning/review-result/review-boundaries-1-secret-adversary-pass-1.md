---
format: aep.planning-md/1
id: review-result:review-boundaries-1-secret-adversary-pass-1
kind: review-result
status: active
title: Secret boundary adversary pass 1
relations:
- reviews: story:review-secret-sanitization
revision: 1
---
unit: story:review-secret-sanitization at 4851c9301a06a9557a36b23086aeee48ded26b03 plus test additions
verdict: red
cases: executed 5→8, red 1
origin: introduced 0 / pre-existing 0 / undecided 1
wrote-outside-worktree: none
needs-coordinator: yes

`git --no-pager diff --stat`:

```text
 .../ess-kubernetes/tests/fixtures/fake_command.rs  | 12 ++++
 .../infra/ess-kubernetes/tests/secret_boundary.rs  | 73 ++++++++++++++++++++++
 2 files changed, 85 insertions(+)
```

Only test files changed. All existing assertions remain intact; the fixture gained an opt-in synthetic subprocess failure. No implementation, planning, staging, commit, branch, or worktree lifecycle operation was performed. `git diff --check` exited 0.

## New cases and first execution

All three cases were written before the first Cargo execution. The supplied implementor report established the baseline of five executed tests (two library, three integration); I did not rerun the untouched suite. Cargo used `RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, the assigned scratch as TMPDIR, and the worktree's own target. The coordinator explicitly cleared the exclusive Cargo slot before testing.

1. `crates/infra/ess-kubernetes/tests/secret_boundary.rs:205`, `failed_secret_subprocess_diagnostics_do_not_echo_secret_values`: a fake kubectl fails both Secret invocations and emits a synthetic sentinel on stderr. The scan refuses and preserves the existing destination, but echoes the sentinel to diagnostics. The case is red.
2. `crates/infra/ess-kubernetes/tests/secret_boundary.rs:232`, `invalid_json_after_secret_values_is_refused_without_leaking_or_replacing_output`: truncated JSON, an invalid escape following a secret value, and trailing data all refuse without leaking or replacing the destination. The case is green.
3. `crates/infra/ess-kubernetes/tests/secret_boundary.rs:254`, `malformed_late_annotations_with_sentinel_keys_refuse_before_any_write`: a late invalid annotation containing sentinel keys and nested values refuses after earlier Secret values have been sanitized in memory; no output is created or replaced and no sentinel is emitted. The case is green.

The first case ran alone first:

`cargo test --locked -p ess-kubernetes --test secret_boundary failed_secret_subprocess_diagnostics_do_not_echo_secret_values -- --exact` — exit **101**.

Raw combined output: `target/review-boundaries-1/review-secret-sanitization/adversary-first-stderr.log`. Only the compilation line containing the checkout's machine path is excluded below; runner output is verbatim.

```text
    Finished `test` profile [unoptimized] target(s) in 0.23s
     Running tests/secret_boundary.rs (target/debug/deps/secret_boundary-9f7580c6900df241)

running 1 test
test failed_secret_subprocess_diagnostics_do_not_echo_secret_values ... FAILED

failures:

---- failed_secret_subprocess_diagnostics_do_not_echo_secret_values stdout ----

thread 'failed_secret_subprocess_diagnostics_do_not_echo_secret_values' (3096177) panicked at crates/infra/ess-kubernetes/tests/secret_boundary.rs:221:5:
the failed Secret subprocess diagnostic leaked the synthetic sentinel
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    failed_secret_subprocess_diagnostics_do_not_echo_secret_values

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.08s

error: test failed, to rerun pass `-p ess-kubernetes --test secret_boundary`
```

The formatter later expanded one assertion without altering it, moving the captured failure assertion from line 221 to its current line 224. No test was weakened.

The other newly authored cases then ran individually before the suite:

`cargo test --locked -p ess-kubernetes --test secret_boundary invalid_json_after_secret_values_is_refused_without_leaking_or_replacing_output -- --exact` — exit **0**.

`cargo test --locked -p ess-kubernetes --test secret_boundary malformed_late_annotations_with_sentinel_keys_refuse_before_any_write -- --exact` — exit **0**.

Raw logs: `target/review-boundaries-1/review-secret-sanitization/adversary-first-json.log` and `adversary-first-annotations.log`, in that order. Verbatim output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.05s
     Running tests/secret_boundary.rs (target/debug/deps/secret_boundary-9f7580c6900df241)

running 1 test
test invalid_json_after_secret_values_is_refused_without_leaking_or_replacing_output ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.09s

    Finished `test` profile [unoptimized] target(s) in 0.03s
     Running tests/secret_boundary.rs (target/debug/deps/secret_boundary-9f7580c6900df241)

running 1 test
test malformed_late_annotations_with_sentinel_keys_refuse_before_any_write ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.09s
```

## Suite and package checks

After the targeted runs, `cargo test --locked -p ess-kubernetes` exited **101**. Complete raw output: `target/review-boundaries-1/review-secret-sanitization/adversary-suite.log`. Verbatim output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.03s
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
test failed_secret_subprocess_diagnostics_do_not_echo_secret_values ... FAILED
test absent_optional_secret_fields_and_empty_maps_remain_allowed ... ok
test valid_secret_observation_bytes_remain_compatible ... ok
test malformed_late_annotations_with_sentinel_keys_refuse_before_any_write ... ok
test invalid_json_after_secret_values_is_refused_without_leaking_or_replacing_output ... ok
test malformed_secret_response_corpus_is_refused_without_output_or_diagnostic_values ... ok

failures:

---- failed_secret_subprocess_diagnostics_do_not_echo_secret_values stdout ----

thread 'failed_secret_subprocess_diagnostics_do_not_echo_secret_values' (3097754) panicked at crates/infra/ess-kubernetes/tests/secret_boundary.rs:221:5:
the failed Secret subprocess diagnostic leaked the synthetic sentinel
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    failed_secret_subprocess_diagnostics_do_not_echo_secret_values

test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.83s

error: test failed, to rerun pass `-p ess-kubernetes --test secret_boundary`
```

Executed count: library 2, binary 0, integration 6, total 8. Seven passed and one failed; none were ignored. Cargo stopped at the failed integration target, so doc tests did not execute on this pass; the supplied baseline doc-test count was zero.

`cargo fmt -p ess-kubernetes -- --check` first exited 1 for one new assertion's wrapping. I changed only that whitespace; the final check exited **0** with no output. Logs: `adversary-fmt.log` and `adversary-fmt-final.log` under the assigned scratch. `cargo clippy --locked -p ess-kubernetes --all-targets -- -D warnings` exited **0**, with this final output line:

```text
    Finished `dev` profile [unoptimized] target(s) in 0.15s
```

The Clippy raw output is `target/review-boundaries-1/review-secret-sanitization/adversary-clippy.log`. The assertion wrapping was the only subsequent change. Each command's exit is retained beside its raw log in an `.exit` file.

## Measured finding and reachability

Findings cover committed unit head `4851c9301a06a9557a36b23086aeee48ded26b03` plus the test-only working tree shown above.

| File:line | Verdict | Origin | Severity | Finding | What was measured | What reaches it |
|---|---|---|---|---|---|---|
| crates/infra/ess-kubernetes/src/lib.rs:43 | CONFIRMED | undecided | warning | A failed Secret subprocess response can emit its synthetic secret value through the adapter's verbatim stderr diagnostic. | The new test's sentinel-exclusion assertion at tests/secret_boundary.rs:224 failed on its first targeted run, exit 101, and again in the package suite. Refusal and preservation assertions before it passed. | The public `ess-kubernetes scan --context synthetic-context --out …` calls `scan` at src/main.rs:30, which invokes kubectl at src/lib.rs:71–72. The fake kubectl failed both attempts and returned the sentinel on stderr; the fallback error reaches src/main.rs:35 and is printed. |

This finding exercises the existing subprocess error path before successful JSON parsing and the changed sanitizer. It does not show that the new malformed-shape guards themselves fail. The diff leaves this error handler unchanged, but no base execution was performed in this pass, so origin remains **undecided**, not guessed pre-existing. The coordinator requested this exact report before base replay and owns routing. No live cluster or actual credential exposure was tested or claimed.

The fixture source is `crates/infra/ess-kubernetes/tests/fixtures/fake_command.rs`. The exact first-run helper binary remains at `target/tmp/secret-boundary-3096176/bin/kubectl`. For a coordinator replay against another binary, use that helper directory on PATH, set `ESS_TEST_SECRET_FAILURE=SYNTHETIC-MALFORMED-SECRET-SENTINEL`, and invoke the binary with `scan --context synthetic-context --out <existing-synthetic-destination>`. The helper accepts both `--context synthetic-context get secrets -A -o json` and `--context synthetic-context get secrets -o json`; each writes `malformed synthetic Secret response: SYNTHETIC-MALFORMED-SECRET-SENTINEL` to stderr and exits 1. Other resource invocations return empty lists. No source checkout or base mutation is necessary.

A change to subprocess diagnostic handling would address the measured path; I made no such implementation change. Collection scope and retry semantics remain outside this attack's requested change.

## Attacks that did not break it

- Malformed JSON following a secret value was refused with value-free parser diagnostics and the original destination intact.
- Nested malformed annotations with sentinel keys were refused after prior in-memory sanitization, with neither partial output nor leaked keys.
- The existing complete valid observation golden and optional-field cases stayed green in the attacked suite.

## Resources and paths

The pre-command free-space checks remained above the revised 8,589,934,592-byte reserve. Free space was 17,814,290,432 bytes before the first targeted execution and 17,090,940,928 bytes after the final formatter check. The target then occupied 161,720 KiB. I notified the coordinator of the changing free space and released the exclusive Cargo slot immediately after the last command. No cache or build directory was deleted.

Paths written outside the assigned worktree: **none**. Tests, the synthetic process fixture, generated fake commands, synthetic observations, raw logs, exit records, and this report are all inside the assigned worktree. No CARGO_TARGET_DIR was set. Harness token count and tool count are unavailable; no elapsed-time estimate is claimed.

```findings
- file: crates/infra/ess-kubernetes/src/lib.rs
  line: 43
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: A failed Secret subprocess response can emit its synthetic secret value through the adapter's verbatim stderr diagnostic.
```
