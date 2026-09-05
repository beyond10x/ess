unit: story:review-secret-sanitization
verdict: green
cases: executed 2→5, red 1
origin: n/a
wrote-outside-worktree: none
needs-coordinator: no

## 1. Unit and scope

story:review-secret-sanitization — Refuse malformed Secret shapes before serialization. Acceptance: every synthetic malformed Secret response in the boundary corpus is refused without emitting its sentinel value to observation bytes or diagnostics.

Read the entire active story, root and package AGENTS.md, implementation charter, unit brief and resource revision. The read-only graph has only decomposes:epic:review-boundary-remediation and serves:vision:O2 edges for this story; there are no prerequisite or blocker edges.

| Inferred scope line | Confirmation |
|---|---|
| Extend package-local malformed corpus and fake-kubectl coverage | Confirmed: the public scan entry point and binary are in this package. Added a Rust subprocess fixture and integration tests here. |
| No documentation edit required | Confirmed: existing package credential contract and the story already define the behavior; no format or kind-list change was necessary. |
| Would collide with any change in this package | Confirmed: the implementation changes its sanitizer and the fixture drives its public scan entry point. No files outside the assigned package were changed. |

The change refuses non-object items; present non-object data/stringData, metadata or annotations; non-string data/stringData entries; and non-string annotation entries. Errors use fixed field names and never interpolate response values or keys. Missing optional fields stay allowed. Valid string hashing, UTF-8 byte lengths, last-applied removal and the complete observation bytes are preserved by a golden that also passed before implementation.

The corpus enumerates string, number, boolean, null, array and nested-object shapes at list, items, item, secret map, secret entry, metadata, annotations and annotation-entry positions where each is malformed. It also puts a valid item before a malformed item. Each response is tested with both absent and existing destinations. These are cases inside one runner test; they are not inflated into the suite's test count.

## 2. Observed diff

`git --no-pager diff --stat` (new files remain untracked, as instructed):

```text
 crates/infra/ess-kubernetes/src/lib.rs | 34 ++++++++++++++++++++++++----------
 1 file changed, 24 insertions(+), 10 deletions(-)
```

Additional observed `git --no-pager diff --no-index --stat -- /dev/null <new-file>` outputs:

```text
 .../infra/ess-kubernetes/tests/secret_boundary.rs  | 202 +++++++++++++++++++++
 1 file changed, 202 insertions(+)
 .../ess-kubernetes/tests/fixtures/fake_command.rs  | 26 ++++++++++++++++++++++
 1 file changed, 26 insertions(+)
 .../tests/fixtures/valid-observation.json          | 88 ++++++++++++++++++++++
 1 file changed, 88 insertions(+)
```

No source changes are committed or staged. `git diff --check` exited 0. The restored sanitizer matches the scratch pre-mutation copy byte for byte.

## 3. Red run before implementation

All Cargo commands used `RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, with TMPDIR inside assigned scratch. CARGO_TARGET_DIR was never set.

Command: `cargo test --locked -p ess-kubernetes`; exit **101**. Tests and golden existed before the sanitizer edit. The two compatibility cases already passed; the malformed corpus exposed acceptance and destination replacement failures.

Complete raw combined output is retained privately at `target/review-boundaries-1/review-secret-sanitization/red.log`. To keep this report portable, compilation-driver lines containing the machine checkout path are left in that raw log. The following test-runner output and failure output are verbatim:

```text
running 3 tests
test absent_optional_secret_fields_and_empty_maps_remain_allowed ... ok
test valid_secret_observation_bytes_remain_compatible ... ok
test malformed_secret_response_corpus_is_refused_without_output_or_diagnostic_values ... FAILED

failures:

---- malformed_secret_response_corpus_is_refused_without_output_or_diagnostic_values stdout ----

thread 'malformed_secret_response_corpus_is_refused_without_output_or_diagnostic_values' (3042808) panicked at crates/infra/ess-kubernetes/tests/secret_boundary.rs:121:5:
item-string, existing=false: accepted malformed response
item-string: created an observation on refusal
item-string, existing=true: accepted malformed response
item-string: replaced prior observation
data-string, existing=false: accepted malformed response
data-string: created an observation on refusal
data-string, existing=true: accepted malformed response
data-string: replaced prior observation
stringData-string, existing=false: accepted malformed response
stringData-string: created an observation on refusal
stringData-string, existing=true: accepted malformed response
stringData-string: replaced prior observation
```

The omitted repetitive lines are retained in the raw log: 92 accepted malformed executions, 46 created destinations, and 46 replaced destinations. These are corpus iterations inside one failing runner test. Exact runner summaries:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s
```

Required guard mutation: changed the sanitizer lookup from `item.get_mut(field)` to `item.get_mut("intentionally-disabled-secret-field")`, then ran `cargo test --locked -p ess-kubernetes --lib tests::secret_values_and_last_applied_configuration_never_survive_sanitization -- --exact`; exit **101**. The mutation was restored before final gates; no assertion was weakened.

Raw combined output: `target/review-boundaries-1/review-secret-sanitization/mutation-red.log`. Verbatim runner output:

```text
     Running unittests src/lib.rs (target/debug/deps/ess_kubernetes-5f388cf73a35bac9)

running 1 test
test tests::secret_values_and_last_applied_configuration_never_survive_sanitization ... FAILED

failures:

---- tests::secret_values_and_last_applied_configuration_never_survive_sanitization stdout ----

thread 'tests::secret_values_and_last_applied_configuration_never_survive_sanitization' (3051301) panicked at crates/infra/ess-kubernetes/src/lib.rs:194:13:
raw secret survived sanitization: RAW-BASE64-SECRET
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::secret_values_and_last_applied_configuration_never_survive_sanitization

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-kubernetes --lib`
```

## 4. Green runs and gates

Baseline command: `cargo test --locked -p ess-kubernetes`; exit **0**. Raw output: `target/review-boundaries-1/review-secret-sanitization/baseline.log`.

```text
     Running unittests src/lib.rs (target/debug/deps/ess_kubernetes-5f388cf73a35bac9)

running 2 tests
test tests::malformed_secret_lists_are_refused_before_any_output_can_be_written ... ok
test tests::secret_values_and_last_applied_configuration_never_survive_sanitization ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/ess_kubernetes-fc6b8b90c0cbb6e8)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_kubernetes

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Targeted green command: `cargo test --locked -p ess-kubernetes --test secret_boundary`; exit **0**. Raw output: `target/review-boundaries-1/review-secret-sanitization/targeted-green.log`.

```text
     Running tests/secret_boundary.rs (target/debug/deps/secret_boundary-9f7580c6900df241)

running 3 tests
test absent_optional_secret_fields_and_empty_maps_remain_allowed ... ok
test valid_secret_observation_bytes_remain_compatible ... ok
test malformed_secret_response_corpus_is_refused_without_output_or_diagnostic_values ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s
```

Final whole-package command, after restoring the mutation: `cargo test --locked -p ess-kubernetes`; exit **0**. Raw output: `target/review-boundaries-1/review-secret-sanitization/green.log`.

```text
     Running unittests src/lib.rs (target/debug/deps/ess_kubernetes-5f388cf73a35bac9)

running 2 tests
test tests::malformed_secret_lists_are_refused_before_any_output_can_be_written ... ok
test tests::secret_values_and_last_applied_configuration_never_survive_sanitization ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/ess_kubernetes-fc6b8b90c0cbb6e8)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/secret_boundary.rs (target/debug/deps/secret_boundary-9f7580c6900df241)

running 3 tests
test valid_secret_observation_bytes_remain_compatible ... ok
test absent_optional_secret_fields_and_empty_maps_remain_allowed ... ok
test malformed_secret_response_corpus_is_refused_without_output_or_diagnostic_values ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.83s

   Doc-tests ess_kubernetes

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Runner-derived lane counts:

- Whole package: executed **2 → 5**, exit **0**. Sum of the runner summaries below.
- Library lane: executed **2 → 2**, exit **0**; no tests were added to this lane.
- Binary lane: executed **0 → 0**, exit **0**; no tests were added to this lane.
- Documentation lane: executed **0 → 0**, exit **0**; no tests were added to this lane.
- New integration lane: absent from the untouched base, so it has no base runner summary to quote. With tests installed before implementation, it executed **3** (2 passed, 1 failed); after implementation it executed **3** (3 passed), exit **0**. The package's 2 → 5 increase demonstrates that the added lane ran.
- Named mutation lane: executed **1**, with 1 failure and exit **101**; that same named test is green in the restored whole-package output above.

`cargo fmt -p ess-kubernetes -- --check`: exit **0**, no output. Raw log and status: `target/review-boundaries-1/review-secret-sanitization/fmt-check.log`, `target/review-boundaries-1/review-secret-sanitization/fmt-check.exit`.

`cargo clippy --locked -p ess-kubernetes --all-targets -- -D warnings`: exit **0**. Raw combined output: `target/review-boundaries-1/review-secret-sanitization/clippy.log`. Its path-independent final output line, verbatim:

```text
    Finished `dev` profile [unoptimized] target(s) in 1.91s
```

Resource observations were sent to the coordinator after builds. The initial 20 GiB floor prevented starting a build; the coordinator's recorded resource revision authorized serialized Cargo with an 8 GiB reserve. Baseline used 78,380 KiB of target with 11,790,979,072 free bytes afterward. After final Clippy, target was 125,552 KiB and free space was 11,682,316,288 bytes. Exclusive Cargo clearance was returned immediately then. No caches or worktrees were removed.

Measured interval from the initial persisted timestamp through final source review: 2026-09-05T09:40:10Z → 2026-09-05T09:48:21Z (8m 11s). This interval starts after instruction reading; whole-turn duration, token count and tool-count measurements are unavailable.

## 5. Deliberate exclusions

- Collection scope and retry behavior remain with story:review-observation-completeness, as assigned.
- No kinds, versions, envelope fields, canonical valid representation or credential authority were changed.
- No new framework or reusable generic schema layer was introduced; the fix validates the exact sanitizer boundary.
- The complete workspace gate and integration review are coordinator-owned; this unit ran the complete package gate.
- No planning mutations, commits, pushes or worktree lifecycle actions were performed. The existing implementor lease was only heartbeated.

## 6. Paths outside the worktree

None. Logs, reports, scratch copies and fixture compilation output are inside this worktree. The Rust process fixture compiles under Cargo's worktree-local `target/tmp/secret-boundary-<pid>/`; its synthetic observation files remain there for coordinator review. Raw logs and supplemental scratch are under `target/review-boundaries-1/review-secret-sanitization/`.
