---
format: aep.planning-md/1
id: review-result:schema-resource-identity-adversary-pass-1
kind: review-result
status: active
title: Schema resource identity independent source adversary
relations:
- reviews: story:review-schema-resource-identity
revision: 1
---
unit: story:review-schema-resource-identity / 50c1001369c90f662b50d162aac97961199c1e72
verdict: nothing found
cases: executed 167→173, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: none

```text
$ git --no-pager diff --stat
```

The tracked diff is empty. The sole new source path is untracked, so the additional tests-only stat is required:

```text
$ git --no-pager diff --no-index --stat /dev/null crates/edge/ess-cli/tests/schema_registry_identity_adversary.rs
 .../tests/schema_registry_identity_adversary.rs    | 396 +++++++++++++++++++++
 1 file changed, 396 insertions(+)
```

The no-index stat exits 1 because the added file differs from `/dev/null`; it is not a test failure. `tests-only.patch` contains the entire additive 396-line target. No inherited test, fixture, production, generated schema, documentation, binding, Cargo.lock, planning, or Git bytes were changed. Git status reports only `?? crates/edge/ess-cli/tests/schema_registry_identity_adversary.rs`.

## 1. Subject and test additions

Tree: `/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity`; branch `impl/review-schema-resource-identity`; base `e283a26cf0176e96e41b13e5e8b79cd1a82ab88d`. The complete adversary charter, own AGENTS, original and adversary briefs, accepted binding, active story revision 11, frozen nine-path diff, new inherited tests and fixtures, relevant callers, and entire implementor report were read before writing probes. The implementor report SHA-256 is `73b736a6f1480333ed568f6a4b6fe5c262728188f05f5d0d9b2782a5c74cdfe8`.

The baseline is the supplied implementor execution: 167 passed, 0 failed, 0 ignored, 26 summaries in 30.009861360 seconds. It was not rerun before the additional case existed. Six cases were written together, and the first selected admission case was executed alone before any integration-target or package suite. All scenarios invoke the actual compiled `ess` binary; none substitute a schema checker.

| Added case (`crates/edge/ess-cli/tests/schema_registry_identity_adversary.rs`) | Measured contract | Final state |
| --- | --- | --- |
| `an_unselected_invalid_resource_blocks_both_documented_pairs` at line 184 | Both documented payload resources are blocked by an unselected unresolved HTTPS reference, invalid schema type, or relative root ID; registry refusals leave `valid` empty. | Green |
| `envelope_definitions_cannot_shadow_either_payload_resource_root` at line 212 | Conflicting envelope-local `definitions` and `$defs` do not shadow the separate generated resource; nested type failures report the payload pointer for both dialects. | Green |
| `the_selected_schema_checks_its_selector_and_nonobject_payload_as_one_instance` at line 237 | String, array, null and boolean payloads validate through the strict envelope; invalid payloads refuse; a selected envelope whose own selector const disagrees refuses at `/schema`. | Green |
| `decoded_duplicate_ids_are_collisions_and_filenames_supply_no_identity` at line 283 | Different JSON escape bytes decoding to one root ID collide; moving a supplied resource into a nested unrelated filename preserves lookup; a filename selector refuses. | Green |
| `inlining_idless_generated_payloads_breaks_their_original_root_references` at line 316 | Inlining each actual ID-less generated schema under payload produces the documented offline root-fragment refusal. | Green |
| `registry_admission_and_selected_typescript_projection_have_distinct_boundaries` at line 336 | An unselected invalid schema blocks registry validation while selected widget projection retains exact expected bytes; syntax projection refuses before touching an existing sentinel or creating an absent output parent, including `--check`. | Green |

The copies read both actual frozen generated files. Removing only their added root `$id` must equal the original parsed schema, and original byte reads must remain equal. All per-call new-case registry/instance inputs are snapshotted beneath the call receipt before execution. Mutations and the resource relocation occur only in newly constructed scratch inputs.

Reached callers: `crates/edge/ess-cli/src/schema.rs:87` reads the actual registry/instances and calls `schema_contract::validate::validate` at line 101; `crates/generate/schema-contract/src/validate.rs:124` indexes root IDs, validates metadata, prepares the offline registry and compiles all supplied resources before line 206 enters instance checks. Lines 225–263 select and validate the whole instance. `crates/edge/ess-cli/src/schema.rs:149` performs the separate selected TypeScript lookup and calls `project` before any output branch at lines 164–182. These are the public CLI paths described by `website/docs/guides/generate-artifacts.md:50` and its worked invocation at line 75. The references and output checks remain bounded by the accepted local registry and restricted projection contracts.

## 2. Selected executions before the package suite

Every command below used the assigned tree as cwd. `commands.json` records exact command strings and `execution-manifest.json` records sequence, exits, counts and measured `time -p` values. The process environment disables wrappers and shared targets; the prepended toolchain PATH retains the ambient executable lookup after the explicit toolchain directory. Each captured log is immutable at handoff.

### First case alone

```sh
env -u CARGO_TARGET_DIR -u RUSTC_WRAPPER -u SCCACHE_SERVER_UDS PATH='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin':"$PATH" RUSTC='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc' RUSTDOC='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc' CARGO_HOME='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/cargo-home' TMPDIR='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/tmp' GOCACHE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/go-cache' GOMODCACHE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/go-mod' CARGO_INCREMENTAL='0' CARGO_PROFILE_DEV_DEBUG='0' CARGO_PROFILE_TEST_DEBUG='0' CARGO_CACHE_RUSTC_INFO='0' CARGO_BUILD_JOBS='2' CARGO_NET_OFFLINE='true' ESS_SCHEMA_IDENTITY_ADVERSARY_EVIDENCE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli' cargo test --locked -p ess-cli --test schema_registry_identity_adversary an_unselected_invalid_resource_blocks_both_documented_pairs -- --exact --nocapture
```

Exit 0. Complete captured output:

```text
   Compiling ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.29s
     Running tests/schema_registry_identity_adversary.rs (target/debug/deps/schema_registry_identity_adversary-13ce93394dad40e9)

running 1 test
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-invoice-unresolved-3501646-0/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-invoice-invalid-3501646-1/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-invoice-relative-3501646-2/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-source-unresolved-3501646-3/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-source-invalid-3501646-4/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-source-relative-3501646-5/calls/00 exit Some(1)
test an_unselected_invalid_resource_blocks_both_documented_pairs ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.12s

real 0.43
user 0.35
sys 0.12
```

### Initial six-case target: one probe-data error, retained in full

The first target run passed five cases and failed one assertion in my new test. This is not a product finding: the actual generated Money.currency schema declares only `{"type":"string"}`; it has no enum. My initial `"BOGUS"` mutation was therefore still valid, and the CLI correctly returned 0. The exact original new target is preserved in `initial-adversary-test-exact.rs`, SHA-256 `11bd8c39b20638da01e79098d348d067d1b564fe298fdbb6a986c45791b7e190`, matching `source-before.sha256`. `initial-adversary-test.rs` also retains the earlier scratch copy with its one trailing blank line omitted; it is not described as byte-identical.

```sh
env -u CARGO_TARGET_DIR -u RUSTC_WRAPPER -u SCCACHE_SERVER_UDS PATH='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin':"$PATH" RUSTC='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc' RUSTDOC='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc' CARGO_HOME='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/cargo-home' TMPDIR='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/tmp' GOCACHE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/go-cache' GOMODCACHE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/go-mod' CARGO_INCREMENTAL='0' CARGO_PROFILE_DEV_DEBUG='0' CARGO_PROFILE_TEST_DEBUG='0' CARGO_CACHE_RUSTC_INFO='0' CARGO_BUILD_JOBS='2' CARGO_NET_OFFLINE='true' ESS_SCHEMA_IDENTITY_ADVERSARY_EVIDENCE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli' cargo test --locked -p ess-cli --test schema_registry_identity_adversary -- --nocapture
```

Exit 101. Complete captured output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.09s
     Running tests/schema_registry_identity_adversary.rs (target/debug/deps/schema_registry_identity_adversary-13ce93394dad40e9)

running 6 tests
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-typescript-boundaries-3502897-3/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-invoice-unresolved-3502897-0/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-invoice-inline-3502897-2/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-text-3502897-5/calls/00 exit Some(0)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-decoded-ids-3502897-4/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-invoice-root-3502897-1/calls/00 exit Some(0)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-typescript-boundaries-3502897-3/calls/01 exit Some(0)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-typescript-source-3502897-9/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-invoice-invalid-3502897-6/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-text-3502897-5/calls/01 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-typescript-source-3502897-9/calls/01 exit Some(1)
test registry_admission_and_selected_typescript_projection_have_distinct_boundaries ... ok
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-renamed-3502897-8/calls/00 exit Some(0)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-source-inline-3502897-7/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-invoice-root-3502897-1/calls/01 exit Some(0)

thread 'envelope_definitions_cannot_shadow_either_payload_resource_root' (3502900) panicked at crates/edge/ess-cli/tests/schema_registry_identity_adversary.rs:143:9:
assertion `left == right` failed: Output { status: ExitStatus(unix_wait_status(0)), stdout: "{\n  \"issues\": [],\n  \"schema_count\": 2,\n  \"valid\": [\n    {\n      \"instance\": \"instances/one.json\",\n      \"schema\": \"registry/z-unrelated.schema.json\",\n      \"schema_id\": \"urn:example:adversary-invoice-envelope:1\"\n    }\n  ]\n}\n", stderr: "" }
  left: Some(0)
 right: Some(1)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test envelope_definitions_cannot_shadow_either_payload_resource_root ... FAILED
test inlining_idless_generated_payloads_breaks_their_original_root_references ... ok
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-invoice-relative-3502897-10/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-text-3502897-5/calls/02 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-renamed-3502897-8/calls/01 exit Some(1)
test decoded_duplicate_ids_are_collisions_and_filenames_supply_no_identity ... ok
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-array-3502897-12/calls/00 exit Some(0)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-source-unresolved-3502897-11/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-array-3502897-12/calls/01 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-array-3502897-12/calls/02 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-source-invalid-3502897-13/calls/00 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-null-3502897-14/calls/00 exit Some(0)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-source-relative-3502897-15/calls/00 exit Some(1)
test an_unselected_invalid_resource_blocks_both_documented_pairs ... ok
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-null-3502897-14/calls/01 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-null-3502897-14/calls/02 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-boolean-3502897-16/calls/00 exit Some(0)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-boolean-3502897-16/calls/01 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-boolean-3502897-16/calls/02 exit Some(1)
test the_selected_schema_checks_its_selector_and_nonobject_payload_as_one_instance ... ok

failures:

failures:
    envelope_definitions_cannot_shadow_either_payload_resource_root

test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s

error: test failed, to rerun pass `-p ess-cli --test schema_registry_identity_adversary`
real 0.29
user 0.45
sys 0.13
```

The coordinator was notified. Only my new probe input was changed from `json!("BOGUS")` to `json!(false)`, retaining the expected refusal and payload pointer; every inherited assertion stayed byte-identical. The new target alone was formatted with:

```sh
/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustfmt --edition 2021 crates/edge/ess-cli/tests/schema_registry_identity_adversary.rs
```

Exit 0, no output. This was the only manual test-data correction; there were no compiler/setup failures and no production repair. The original implementor's separately reported TypeScript diagnostic-substring correction was not changed or reclassified as a defect.

### Corrected root-reference case alone

```sh
env -u CARGO_TARGET_DIR -u RUSTC_WRAPPER -u SCCACHE_SERVER_UDS PATH='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin':"$PATH" RUSTC='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc' RUSTDOC='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc' CARGO_HOME='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/cargo-home' TMPDIR='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/tmp' GOCACHE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/go-cache' GOMODCACHE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/go-mod' CARGO_INCREMENTAL='0' CARGO_PROFILE_DEV_DEBUG='0' CARGO_PROFILE_TEST_DEBUG='0' CARGO_CACHE_RUSTC_INFO='0' CARGO_BUILD_JOBS='2' CARGO_NET_OFFLINE='true' ESS_SCHEMA_IDENTITY_ADVERSARY_EVIDENCE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli' cargo test --locked -p ess-cli --test schema_registry_identity_adversary envelope_definitions_cannot_shadow_either_payload_resource_root -- --exact --nocapture
```

Exit 0. Complete captured output:

```text
   Compiling ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.26s
     Running tests/schema_registry_identity_adversary.rs (target/debug/deps/schema_registry_identity_adversary-13ce93394dad40e9)

running 1 test
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-invoice-root-3505422-0/calls/00 exit Some(0)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-invoice-root-3505422-0/calls/01 exit Some(1)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-source-root-3505422-1/calls/00 exit Some(0)
CLI receipt /home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/selected-cli/schema-id-attack-source-root-3505422-1/calls/01 exit Some(1)
test envelope_definitions_cannot_shadow_either_payload_resource_root ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.14s

real 0.42
user 0.41
sys 0.09
```

The corrected isolated execution reached all four CLI calls, including the source-syntax branch that the earlier erroneous assertion prevented from running. Both payload resources accepted their positive control and refused their boolean type mutation.

## 3. Full package suite and final source checks

The complete package suite ran only after those selected executions. It executed 173 cases: all 167 inherited cases plus all six added cases. No ignored or filtered cases are included in the count. There were 27 summaries, 173 passed, 0 failed and 0 ignored; wall time was 21.48 seconds. No test or production behavior changed after this run.

```sh
env -u CARGO_TARGET_DIR -u RUSTC_WRAPPER -u SCCACHE_SERVER_UDS PATH='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin':"$PATH" RUSTC='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc' RUSTDOC='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc' CARGO_HOME='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/cargo-home' TMPDIR='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/tmp' GOCACHE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/go-cache' GOMODCACHE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/go-mod' CARGO_INCREMENTAL='0' CARGO_PROFILE_DEV_DEBUG='0' CARGO_PROFILE_TEST_DEBUG='0' CARGO_CACHE_RUSTC_INFO='0' CARGO_BUILD_JOBS='2' CARGO_NET_OFFLINE='true' ESS_SCHEMA_IDENTITY_ADVERSARY_EVIDENCE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/full-cli' ESS_SCHEMA_IDENTITY_EVIDENCE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/inherited-cli' cargo test --locked -p ess-cli --no-fail-fast
```

Exit 0. Complete captured output:

```text
    Finished `test` profile [unoptimized] target(s) in 0.09s
     Running unittests src/main.rs (target/debug/deps/ess-bb7ad213e0b4cc6b)

running 11 tests
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/authored_scenarios.rs (target/debug/deps/authored_scenarios-1c2c3df406105ba6)

running 25 tests
test author_nested_only ... ok
test author_nonmatching_only ... ok
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test author_empty ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test go_nested_only ... ok
test ir_empty ... ok
test ir_nonmatching_only ... ok
test go_empty ... ok
test web_empty ... ok
test ir_nested_only ... ok
test web_nested_only ... ok
test run_nonmatching_only ... ok
test go_nonmatching_only ... ok
test web_nonmatching_only ... ok
test run_nested_only ... ok
test run_empty ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-1b975346dabc52e1)

running 9 tests
test empty_selection_cannot_follow_or_replace_an_output_symlink ... ok
test non_utf8_matching_content_is_refused_before_any_output_write ... ok
test selected_directory_symlink_reports_the_requested_path_and_stays_shallow ... ok
test matching_broken_links_are_read_errors_even_beside_a_valid_source ... ok
test lexical_selection_order_decides_the_first_read_error ... ok
test committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners ... ok
test omitted_scenarios_ignore_a_poisoned_working_directory_default ... ok
test empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner ... ok
test valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s

     Running tests/authored_site.rs (target/debug/deps/authored_site-ef4cfdbdb2eedafd)

running 9 tests
test an_explicit_missing_front_page_is_an_error ... ok
test binary_downloads_are_not_silently_decoded ... ok
test asset_symlink_output_is_refused_before_any_page_changes ... ok
test a_page_identity_can_itself_end_in_html ... ok
test publication_without_strict_mode_preserves_unpublished_links ... ok
test strict_links_check_local_and_cross_page_fragments ... ok
test selected_sources_resolve_after_relocation_with_queries_fragments_and_downloads ... ok
test assets_cannot_collide_with_generated_output_or_escape_it ... ok
test undeclared_existing_and_escaping_targets_are_refused_with_source_lines ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-8cc8a001564682d0)

running 1 test
test cli_composition_obeys_the_independently_authored_vectors ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/binary64_count_adversary.rs (target/debug/deps/binary64_count_adversary-157c78d411c7247c)

running 2 tests
test cli_binary64_model_and_original_suites_preserve_report_destinations_in_both_formats ... ok
test generated_go_binary64_shapes_refuse_before_factory_and_report_publication ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.73s

     Running tests/binary64_publication.rs (target/debug/deps/binary64_publication-2aa6dae635cf5e49)

running 1 test
test binary64_sparse_model_refuses_every_unsupported_publication_route ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-e6799b61d16bc84b)

running 1 test
test binary64_publication_never_replaces_sources_or_partially_updates_a_library ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/command_surface.rs (target/debug/deps/command_surface-820c2588e9f0042b)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-770e0fe494304b6a)

running 4 tests
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/count_reports.rs (target/debug/deps/count_reports-e56e7768c00ccbc3)

running 3 tests
test count_report_opt_in_has_a_distinct_detailed_surface_and_unknown_coverage ... ok
test count_cli_configuration_and_original_suite_refusals_preserve_destinations ... ok
test count_cli_preserves_default_bytes_and_standalone_detailed_pairing ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s

     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-b1bba5ffbcb3cf51)

running 3 tests
test generated_go_skip_counts_preserve_opaque_ids_and_strictness_without_a_destination ... ok
test generated_go_abnormal_teardown_cannot_publish_a_completed_skip ... ok
test generated_go_rejects_closed_predicate_metadata_before_any_target ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.44s

     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-1638f9d800bd41df)

running 2 tests
test generated_go_abnormal_unsupported_error_formatting_cannot_complete ... ok
test generated_go_admits_only_typed_predicate_paths_and_operator_envelopes ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s

     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-5f67664c69a669b0)

running 4 tests
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_web_cli_writes ... ok
test a_binding_function_capture_is_refused_before_rust_cli_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-ffe05f2b4a2d9df9)

running 13 tests
test count_go_predicate_admission_matches_rust_leaf_grammar ... ok
test count_report_skip_only_is_inconclusive_without_actual_failures ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test count_retained_runtime_preserves_legacy_behavior_and_does_not_gain_version_checks ... ok
test count_go_empty_selection_is_inconclusive_and_clock_conversion_is_checked ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test count_go_actual_producers_keep_skip_error_and_teardown_categories ... ok
test count_go_refusals_precede_targets_and_incomplete_runs_never_publish ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.26s

     Running tests/model_types.rs (target/debug/deps/model_types-d1ddd7a24ad016b3)

running 3 tests
test all_type_binary64_libraries_publish_finite_codecs_with_atomic_preflight ... ok
test root_selection_and_output_refusals_preserve_existing_files ... ok
test each_target_retains_the_same_model_selection_and_distinct_input_provenance ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/normalization.rs (target/debug/deps/normalization-ac90639e6bf9bab8)

running 17 tests
test unsupported_go_pattern_has_no_successful_partial_artifact ... ok
test incompatible_later_destination_preserves_the_entire_existing_output ... ok
test output_symlinks_and_hardlinks_are_refused_without_mutation ... ok
test generated_paths_cannot_replace_canonical_source_inputs_or_follow_links ... ok
test stale_bundle_missing_source_and_unknown_dispatch_refuse ... ok
test qualified_go_base64_pattern_is_published_without_decoding_the_value ... ok
test explicit_binary64_inputs_run_through_the_cli_without_weakening_version_one ... ok
test output_cannot_replace_any_declared_input ... ok
test checked_recipe_and_run_are_deterministic_with_json_only_stdout ... ok
test positional_cli_refuses_unused_branches_before_publication ... ok
test generation_checks_refuse_before_creating_or_changing_destinations ... ok
test failed_check_or_execution_preserves_output_and_does_not_emit_a_result ... ok
test raw_json_capture_runs_before_schema_and_keeps_failure_output_untouched ... ok
test generated_libraries_match_the_api_and_drift_check_never_repairs_files ... ok
test positional_cli_prepares_text_and_refuses_before_publication ... ok
test model_sources_are_compiled_pinned_and_protected_by_the_cli ... ok
test binary64_cli_keeps_numeric_identity_and_emits_checked_format_five ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.88s

     Running tests/normalization_positional_adversary.rs (target/debug/deps/normalization_positional_adversary-04c8fc26cdea59df)

running 2 tests
test cli_runtime_grammar_controls_preserve_existing_output ... ok
test cli_invalid_policy_blocks_all_publication_and_valid_metadata_is_drift_checked ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

     Running tests/openapi_accounting.rs (target/debug/deps/openapi_accounting-13b1a0abdf6eaad5)

running 6 tests
test complete_projection_preserves_supported_yaml_through_both_spellings ... ok
test both_import_spellings_write_the_accounted_envelope_for_every_presentation ... ok
test a_refused_import_keeps_existing_output_and_creates_no_new_file ... ok
test legacy_projection_refuses_before_destination_mutation ... ok
test partial_projection_reports_the_durable_gap_and_preserves_destinations ... ok
test unresolved_projection_reports_the_exact_reference_and_preserves_destinations ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/openapi_adversary_pass1.rs (target/debug/deps/openapi_adversary_pass1-260b8c6adc541fec)

running 2 tests
test cli_rejects_duplicate_keys_and_tampered_accounting_before_output ... ok
test cli_rejects_unknown_unit_fields_before_any_projection_output ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/openapi_adversary_pass2.rs (target/debug/deps/openapi_adversary_pass2-2edeb93cde446f25)

running 1 test
test same_path_projection_refusal_preserves_the_unadmitted_input ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/output_containment.rs (target/debug/deps/output_containment-8e5d534f5c43715b)

running 19 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-b206c0da2903f754)

running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/schema_bundle.rs (target/debug/deps/schema_bundle-732645e9c358ab76)

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

     Running tests/schema_registry_identity.rs (target/debug/deps/schema_registry_identity-3079094f8ad8073b)

running 9 tests
test idless_generated_schemas_are_not_registry_resources ... ok
test typescript_uses_root_id_and_preserves_equal_and_stale_check_behavior ... ok
test accepted_instances_can_coexist_with_later_selector_failures ... ok
test typescript_id_and_projection_refusals_happen_before_output_writes ... ok
test offline_missing_references_and_exact_duplicate_ids_refuse_the_registry ... ok
test syntax_acceptance_is_distinct_from_domain_roster_assembly ... ok
test both_dialects_accept_separate_adopter_resources_and_report_the_envelope ... ok
test selectors_and_strict_envelopes_do_not_modify_domain_payloads ... ok
test nested_payload_constraints_are_checked_through_both_resource_roots ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s

     Running tests/schema_registry_identity_adversary.rs (target/debug/deps/schema_registry_identity_adversary-13ce93394dad40e9)

running 6 tests
test registry_admission_and_selected_typescript_projection_have_distinct_boundaries ... ok
test inlining_idless_generated_payloads_breaks_their_original_root_references ... ok
test decoded_duplicate_ids_are_collisions_and_filenames_supply_no_identity ... ok
test an_unselected_invalid_resource_blocks_both_documented_pairs ... ok
test envelope_definitions_cannot_shadow_either_payload_resource_root ... ok
test the_selected_schema_checks_its_selector_and_nonobject_payload_as_one_instance ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

     Running tests/target_failure.rs (target/debug/deps/target_failure-87adfa1dc447352a)

running 1 test
test fatal_synthesis_preserves_destinations_and_has_a_typed_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

real 21.48
user 103.41
sys 16.19
```

### Formatting and strict Clippy

```sh
env -u CARGO_TARGET_DIR -u RUSTC_WRAPPER -u SCCACHE_SERVER_UDS PATH='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin':"$PATH" RUSTC='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc' RUSTDOC='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc' CARGO_HOME='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/cargo-home' TMPDIR='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/tmp' GOCACHE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/go-cache' GOMODCACHE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/go-mod' CARGO_INCREMENTAL='0' CARGO_PROFILE_DEV_DEBUG='0' CARGO_PROFILE_TEST_DEBUG='0' CARGO_CACHE_RUSTC_INFO='0' CARGO_BUILD_JOBS='2' CARGO_NET_OFFLINE='true' cargo fmt -p ess-cli -- --check
```

Exit 0. Complete captured output:

```text
real 0.14
user 0.11
sys 0.03
```

```sh
env -u CARGO_TARGET_DIR -u RUSTC_WRAPPER -u SCCACHE_SERVER_UDS PATH='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin':"$PATH" RUSTC='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc' RUSTDOC='/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc' CARGO_HOME='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/cargo-home' TMPDIR='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/tmp' GOCACHE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/go-cache' GOMODCACHE='/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1/go-mod' CARGO_INCREMENTAL='0' CARGO_PROFILE_DEV_DEBUG='0' CARGO_PROFILE_TEST_DEBUG='0' CARGO_CACHE_RUSTC_INFO='0' CARGO_BUILD_JOBS='2' CARGO_NET_OFFLINE='true' cargo clippy --locked -p ess-cli --all-targets -- -D warnings
```

Exit 0. Complete captured output:

```text
    Checking ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 0.15s
real 0.19
user 0.12
sys 0.07
```

### Diff checks

```sh
git diff --check
```

Exit 0. Complete captured output:

```text
real 0.00
user 0.00
sys 0.00
```

```sh
git --no-pager diff --no-index --check /dev/null crates/edge/ess-cli/tests/schema_registry_identity_adversary.rs
```

Exit 1. Complete captured output:

```text
real 0.00
user 0.00
sys 0.00
```

The no-index check printed no whitespace diagnostic; its 1 is the differing-file status. The tracked diff check passed. The package formatter also checked the new target. No full-workspace, website, integration or live-service gate was run in this bounded unit.

### Runtime and command counts

```text
rustc 1.98.0 (88d9e12ae 2026-08-18)
binary: rustc
commit-hash: 88d9e12ae178fab0fb5cc050a94da85685d449ea
commit-date: 2026-08-18
host: x86_64-unknown-linux-gnu
release: 1.98.0
LLVM version: 22.1.8
cargo 1.98.0 (797e8a9bc 2026-08-05)
6be28cfac2f930e2ebd966458b02a9bb37669241cf00e78e3c539c14e0d849af  target/debug/ess
```

| Execution | Rust executed / passed / failed | Actual schema-identity CLI calls / exit 0 / exit 1 |
| --- | --- | --- |
| First case alone | 1 / 1 / 0 | 6 / 0 / 6 |
| Initial added target | 6 / 5 / 1 (probe-data mistake) | 29 / 8 / 21 |
| Corrected case alone | 1 / 1 / 0 | 4 / 2 / 2 |
| Full package: added target | 6 / 6 / 0 | 31 / 8 / 23 |
| Full package: inherited schema target | 9 / 9 / 0 | 34 / 6 / 28 |

There are 104 retained schema-identity subprocess receipts in total: 24 successes and 80 refusals. These counts are not Rust case counts, and do not claim to count unrelated subprocesses from the other package tests. `cli-execution-manifest.json` indexes every receipt with exact binary, cwd, argv, exit, elapsed seconds and raw stdout/stderr paths. New-case receipts include their pre-command input snapshots. The inherited case outputs are retained under their existing producer's layout without modifying that test.

## 4. Judgement findings

Nothing found. The initial invalid-currency assertion was my test-input error and is reported above rather than assigned a product severity or an origin. No base execution was needed or performed to classify a defect, and no unmeasured pre-existing classification is asserted. This report covers the frozen subject plus the sole additive test target; it is not an approval or a claim of agent independence.

## 5. Attacks that did not break the contract

- Both separate draft-07 and draft-2020-12 roots survived envelope-local fragment-name collisions and enforced nested generated constraints.
- Registry admission rejected unselected invalid resources, unprovided HTTPS references, relative IDs and duplicate decoded exact strings before accepting instances.
- Root-ID selection survived nested unrelated filenames; selected envelope const and nonobject payload constraints applied to the complete instance.
- Actual generated-schema inlining failed where the public guide warns its root fragments would move.
- The restricted selected widget TypeScript projection retained expected bytes, while generated syntax projection preserved existing and absent output destinations on refusal.
- All inherited syntax-versus-semantic admission, strict envelope, mixed valid/refused instance, generated-byte/provenance and old TypeScript refusal cases passed unchanged.

No URI-alias equivalence, historical immutability enforcement, new namespace, hosted resolution, new format/flag, original numeric-text admission, Binary64 conformance, or coverage behavior is inferred from these results.

## 6. Preservation, paths and relinquishment

All 15 coordinator-supplied final-source entries match their before/after SHA-256 values, including both generated originals, all original nine changed paths, the accepted binding and both production schema-contract owners. Cargo.lock is also unchanged. `final-source.json` lists these 16 preserved inputs plus the new target (17 entries). The final added target SHA-256 is `e63647b37bea4d95e80376d0d8d092a70005d0b6274ab1ac45c9c9b5ae1ae0e0`. The entire final tests-only patch is `tests-only.patch`, SHA-256 `070214f055f28eaa2ffce8a228c0b97c42174d0926eaa9d8f7744426949757a7`.

All scratch paths are beneath `/home/timo/.local/state/worktree/trees/b10x/ess/ess-schema-resource-identity/target/review-boundaries-10/schema-resource-identity/adversary-pass-1`. Build artifacts use this tree's target; Cargo uses the assigned unit Cargo home with its existing read-only cache inputs. TMPDIR, GOCACHE and GOMODCACHE point beneath this scratch. No cleanup, Git mutation, planning/store mutation, external integration call or extra agent was invoked. Read-only toolchain and cache inputs are not claimed as writes. No process syscall trace was run; the outside-path statement reports configured/observed writes, not a fabricated filesystem attestation.

Every path written outside this worktree: **none**.

The final free-space measurement was 46,999,818,240 bytes, above the 8,589,934,592-byte floor. No resource-floor stop was needed.

`evidence.sha256` catalogs all report, command, raw log, source manifest, source snapshot, input and receipt files beneath this scratch, excluding itself, `SHA256SUMS`, `seal-verification.log` and mutable `tmp/`, `go-cache/`, `go-mod/` build/temp contents. Those excluded build/temp directories are retained, not deleted. `SHA256SUMS` seals the report, catalog, final source/command/execution manifests, tests-only patch and catalog verification log. The original implementor records remain untouched outside this attack scratch.

All source, test, build and scratch writes are relinquished when this report and its seal are handed off. The coordinator owns subsequent recording, freezing, routing and gates.

```findings
[]
```
