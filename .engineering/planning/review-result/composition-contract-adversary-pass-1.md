---
format: aep.planning-md/1
id: review-result:composition-contract-adversary-pass-1
kind: review-result
status: active
title: Composition contract source attack pass 1
relations:
- reviews: story:review-composition-contract
revision: 1
---
unit: story:review-composition-contract at 327b968acb0fb7237acf005972aa80f16074cd8a plus additive test SHA256 a2448d93c3a03a374cf29cc83cbb4b290796471beb86bc8ae6e36544e75f7e5b
verdict: nothing found
cases: executed 10→12, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: record this pass, decide test retention, run integration gates and public delivery
git --no-pager diff --stat
```text
 .../specify/ess-composition/tests/composition.rs   | 133 +++++++++++++++++++++
 1 file changed, 133 insertions(+)
```

Read the complete assigned brief, installed aep-drive 0.8.0 adversary charter, repository AGENTS, story acceptance, original unit brief, implementor report, complete change and five changed files. Read the actual compose caller at crates/edge/ess-cli/src/main.rs:1840, selected-surface traversal and emitted runtime. Opening base: e176858010e45bada104ebb10e9b69e2232e0ac0. HEAD remained the frozen subject. The implementor-report hash matched bcfc1563e5909f53ce1c4b6a9adbff4183a67d27fc856199d01beaf291a78b1c; all supplied source and retained-fixture hashes matched before additions.

1. Added cases, written before the first test command

Only crates/specify/ess-composition/tests/composition.rs changed: 133 additive lines, no deletions or rewritten assertions. The first-test-source.sha256 and unchanged-test-check.log establish the same test bytes for focused runs and the subsequent suite. The supplied baseline was 10 Cargo package cases in three summaries plus three distinct emitted-client child cases; no pre-addition package run was made.

At composition.rs:479, downstream_operation_construction_requires_an_emitted_descriptor compiles the actual emission as a separate library. A downstream selected Todo query descriptor compiles. Attempts to create Todo/workbench.usage.RecordUsage through Operation::new or its fields must return exit 1 with E0624/private and E0451/private respectively. These deliberate compiler refusals test the new claim in website/docs/reference/cli.md:80 and src/lib.rs:748. They are not runtime test failures or setup mistakes. The package case is green.

First focused command (exit 0; focused-descriptor.log and focused-descriptor.exit):

```text
cargo test --locked -p ess-composition downstream_operation_construction_requires_an_emitted_descriptor -- --exact --nocapture
```

```text
   Compiling ess-composition v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/crates/specify/ess-composition)
    Finished `test` profile [unoptimized] target(s) in 0.51s
     Running unittests src/lib.rs (target/debug/deps/ess_composition-13563bcd6ceb4522)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/composition.rs (target/debug/deps/composition-0fbadb9df0767612)

running 1 test
command: "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc" "--edition=2021" "--crate-type=lib" "--crate-name=composition_fixture" "-D" "warnings" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-descriptor-2601509/lib.rs" "-o" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-descriptor-2601509/libcomposition_fixture.rlib"
exit: exit status: 0
stdout:

stderr:

command: "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc" "--edition=2021" "--crate-type=lib" "-D" "warnings" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-descriptor-2601509/selected.rs" "--extern" "composition_fixture=/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-descriptor-2601509/libcomposition_fixture.rlib" "--out-dir" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-descriptor-2601509"
exit: exit status: 0
stdout:

stderr:

command: "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc" "--edition=2021" "--crate-type=lib" "-D" "warnings" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-descriptor-2601509/constructor.rs" "--extern" "composition_fixture=/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-descriptor-2601509/libcomposition_fixture.rlib" "--out-dir" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-descriptor-2601509"
exit: exit status: 1
stdout:

stderr:
error[E0624]: associated function `new` is private
  --> /home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-descriptor-2601509/constructor.rs:1:88
   |
 1 | pub fn operation() -> composition_fixture::Operation { composition_fixture::Operation::new("todo", "workbench.usage.RecordUsage", co...
   |                                                                                        ^^^ private associated function
   |
  ::: /home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-descriptor-2601509/lib.rs:61:5
   |
61 |     const fn new(service_key: &'static str, semantic: &'static str, kind: OperationKind) -> Self {
   |     -------------------------------------------------------------------------------------------- private associated function defined here

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0624`.

command: "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc" "--edition=2021" "--crate-type=lib" "-D" "warnings" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-descriptor-2601509/fields.rs" "--extern" "composition_fixture=/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-descriptor-2601509/libcomposition_fixture.rlib" "--out-dir" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-descriptor-2601509"
exit: exit status: 1
stdout:

stderr:
error[E0451]: fields `service_key`, `semantic` and `kind` of struct `Operation` are private
 --> /home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-descriptor-2601509/fields.rs:1:89
  |
1 | ...ion_fixture::Operation { service_key: "todo", semantic: "workbench.usage.RecordUsage", kind: composition_fixture::OperationKind::C...
  |                             ^^^^^^^^^^^          ^^^^^^^^ private field                   ^^^^ private field
  |                             |
  |                             private field

error: aborting due to 1 previous error

For more information about this error, try `rustc --explain E0451`.

test downstream_operation_construction_requires_an_emitted_descriptor ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.09s

elapsed_seconds: 0.616 user_seconds: 0.419 system_seconds: 0.207
```

At composition.rs:526, recording_example_detects_a_payload_dropping_client_mutant compiles the original emission and a scratch copy differing only at the actual forwarding call: payload becomes &payload[..0]. It compiles the unchanged downstream client_boundary.rs against each library and runs exactly its compatible/incompatible-title case. Original emission passes; the compiled mutant exits 101 at client_boundary.rs:139 with left: []. The outer test requires that exact observed failure shape and is green. The controlled mutation is not in the subject source and is not a finding. This attacks the executed example linked at website/docs/reference/cli.md:85 and its byte-preservation claim at src/lib.rs:751.

Second focused command (exit 0; focused-mutant.log and focused-mutant.exit), still before the package run:

```text
cargo test --locked -p ess-composition recording_example_detects_a_payload_dropping_client_mutant -- --exact --nocapture
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running unittests src/lib.rs (target/debug/deps/ess_composition-13563bcd6ceb4522)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/composition.rs (target/debug/deps/composition-0fbadb9df0767612)

running 1 test
command: "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc" "--edition=2021" "--crate-type=lib" "--crate-name=composition_fixture" "-D" "warnings" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-forwarding-control-2604310/lib.rs" "-o" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-forwarding-control-2604310/libcomposition_fixture.rlib"
exit: exit status: 0
stdout:

stderr:

command: "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc" "--edition=2021" "--test" "-D" "warnings" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/crates/specify/ess-composition/tests/fixtures/client_boundary.rs" "--extern" "composition_fixture=/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-forwarding-control-2604310/libcomposition_fixture.rlib" "-o" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-forwarding-control-2604310/client_boundary_tests"
exit: exit status: 0
stdout:

stderr:

command: "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-forwarding-control-2604310/client_boundary_tests" "compatible_and_incompatible_titles_reach_the_same_selected_operation_unchanged" "--exact" "--nocapture" "--test-threads=1"
exit: exit status: 0
stdout:

running 1 test
test compatible_and_incompatible_titles_reach_the_same_selected_operation_unchanged ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s


stderr:

command: "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc" "--edition=2021" "--crate-type=lib" "--crate-name=composition_fixture" "-D" "warnings" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-forwarding-mutant-2604310/lib.rs" "-o" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-forwarding-mutant-2604310/libcomposition_fixture.rlib"
exit: exit status: 0
stdout:

stderr:

command: "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc" "--edition=2021" "--test" "-D" "warnings" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/crates/specify/ess-composition/tests/fixtures/client_boundary.rs" "--extern" "composition_fixture=/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-forwarding-mutant-2604310/libcomposition_fixture.rlib" "-o" "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-forwarding-mutant-2604310/client_boundary_tests"
exit: exit status: 0
stdout:

stderr:

command: "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-adversary-forwarding-mutant-2604310/client_boundary_tests" "compatible_and_incompatible_titles_reach_the_same_selected_operation_unchanged" "--exact" "--nocapture" "--test-threads=1"
exit: exit status: 101
stdout:

running 1 test
test compatible_and_incompatible_titles_reach_the_same_selected_operation_unchanged ... FAILED

failures:

failures:
    compatible_and_incompatible_titles_reach_the_same_selected_operation_unchanged

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s


stderr:

thread 'compatible_and_incompatible_titles_reach_the_same_selected_operation_unchanged' (2604437) panicked at /home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/crates/specify/ess-composition/tests/fixtures/client_boundary.rs:139:5:
assertion `left == right` failed
  left: []
 right: [123, 34, 100, 101, 116, 97, 105, 108, 115, 34, 58, 123, 34, 116, 105, 116, 108, 101, 34, 58, 34, 73, 110, 98, 111, 120, 34, 125, 125]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

test recording_example_detects_a_payload_dropping_client_mutant ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.27s

elapsed_seconds: 0.344 user_seconds: 0.251 system_seconds: 0.134
```

2. Subsequent package run

Command and complete raw output (exit 0; package-test.log and package-test.exit):

```text
cargo test --locked -p ess-composition
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running unittests src/lib.rs (target/debug/deps/ess_composition-13563bcd6ceb4522)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/composition.rs (target/debug/deps/composition-0fbadb9df0767612)

running 12 tests
test selecting_todo_excludes_usage_and_keeps_the_recursive_contract_closure ... ok
test exact_component_identity_digest_and_closed_registry_are_enforced ... ok
test service_keys_and_digests_have_one_canonical_spelling ... ok
test v1_is_strict_and_requires_an_explicit_component_selection ... ok
test a_reference_that_exists_but_is_outside_the_selected_component_is_refused ... ok
test selecting_usage_excludes_todo_even_though_both_share_one_compiled_model ... ok
test identical_component_imports_are_duplicates_but_two_components_of_one_model_are_not ... ok
test canonical_ir_plan_and_generated_clients_ignore_all_input_order ... ok
test generated_rust_client_matches_the_committed_corpus_and_compiles ... ok
test downstream_operation_construction_requires_an_emitted_descriptor ... ok
test generated_rust_client_executes_the_byte_transport_boundary ... ok
test recording_example_detects_a_payload_dropping_client_mutant ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s

   Doc-tests ess_composition

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

elapsed_seconds: 0.410 user_seconds: 0.529 system_seconds: 0.263
```

The final package run executed 12 cases, 0 failed/ignored, in three summaries. The original generated-client child executed its same three distinct cases, all passing:

```text
command: "/home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1/tmp/ess-composition-client-runtime-2607470/client_boundary_tests" "--nocapture" "--test-threads=1"
exit: exit status: 0
stdout:

running 3 tests
test compatible_and_incompatible_titles_reach_the_same_selected_operation_unchanged ... ok
test missing_endpoint_prevents_authority_lookup_and_transport ... ok
test transport_error_is_returned_without_reinterpretation ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


stderr:
```

The mutant attack also executed the title case once against another unchanged emission and once against the mutant. Thus this package invocation caused five child test executions: the original three, one repeated passing control, and one expected mutant failure. There remain three distinct behavior cases in the child fixture. Neither that repetition nor compiler invocations are counted as added package cases, and the expected mutant failure is not a defect in the frozen subject. Child logs for the final run are under tmp/ess-composition-client-runtime-2607470, tmp/ess-composition-adversary-forwarding-control-2607470 and tmp/ess-composition-adversary-forwarding-mutant-2607470. Focused logs retain the earlier process directories separately.

After the suite, cargo fmt -p ess-composition -- --check exited 0 (fmt-check.log, 0.039 s), cargo clippy --locked -p ess-composition --all-targets -- -D warnings exited 0 (clippy.log, 0.200 s), and git diff --check exited 0 (diff-check.log). No source edits followed the first focused run. There were no unexpected setup failures in this pass; the intentional compiler and mutant failures are preserved in full. No workspace/site/browser/network gate ran here.

All Rust commands used the direct stable toolchain, CARGO_BUILD_JOBS=2, offline private Cargo home target/review-boundaries-9/cargo-home, the worktree target, CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0 and CARGO_CACHE_RUSTC_INFO=0. CARGO_TARGET_DIR, RUSTC_WRAPPER and SCCACHE_SERVER_UDS were unset. TMPDIR was the assigned adversary-pass-1/tmp. Each build had a recorded disk check above 8589934592 bytes; final available space was 15905951744 bytes. Logs retain un-piped command statuses and Bash timing.

3. Findings

No finding against the frozen source was measured. There are no judgement findings or origin assignments.

4. Attacks that did not break the subject

- Ordinary downstream Operation construction refused the unselected semantic name through both private API routes; a selected public descriptor compiled.
- The actual emitted-client recording example failed on a scratch payload-dropping mutation at its exact-byte assertion; the same case passed against unchanged emission.
- The package retained wrong-component, digest, deterministic corpus, byte response, separate authority, missing-endpoint and transport-error controls. The inherited generated golden bytes still matched.

These results are bounded to the two attacks and existing package cases. No typed-client, authority verifier or remote handshake was selected or claimed. The generated template's pre-existing prose was frozen by the brief; this pass did not rewrite it or infer a codec requirement. The public CLI emission route was traced in source but no separate CLI invocation or public URL fetch was made.

5. Outside writes and final manifest

Outside-worktree writes: none. All new test, generated probe/copy, compiler, registry metadata and log writes stayed in the assigned worktree, build target and scratch. No production/documentation/planning/Git/sibling/AEP-helper writes or cleanup commands were performed. The existing golden-compilation test kept its inherited owned-temp cleanup under assigned TMPDIR.

final-source-test.sha256:

```text
db71d32fd00681c3047b1efd41759898ee4e3923ba3373b0580430a81c309afb  crates/specify/ess-composition/src/lib.rs
a2448d93c3a03a374cf29cc83cbb4b290796471beb86bc8ae6e36544e75f7e5b  crates/specify/ess-composition/tests/composition.rs
6a1cebf423bbebfae6164471d5c3817510caabb3e77a7632237e5c3ad90c5b61  crates/specify/ess-composition/tests/fixtures/client_boundary.rs
25d59e365638d549a8bee177004dbe4c86a2172d438d6e352507bc8f2fb26f71  website/docs/reference/cli.md
cee8bed7df2bcf936a482ad8c4c79c7045b803ece402590ac73c9308e25860f3  website/docs/reference/formats.md
48292c59c73e812efabbd03dee3fb4bf2edcbc145cecc1c0c90fd724ea27969e  Cargo.lock
```

frozen-source-check.log/exit and retained-fixtures-check.log/exit both passed: production/API docs, public pages, every existing fixture/golden covered by the supplied manifest, and Cargo.lock remain unchanged. final-status.txt contains only the additive composition.rs modification. additive-tests.patch preserves the complete test-only patch. execution.sha256 identifies all retained top-level logs/statuses/disk records and generated probe source/command logs, including each control/mutant copy. The first and final source manifests are included in that evidence manifest. Every artifact path is relative to /home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-1 unless stated otherwise.

This report is final and will not be overwritten. I relinquish all writes to source, build and scratch on return. The coordinator owns retention, planning records, integration and delivery; this report grants no approval.

```findings
[]
```

