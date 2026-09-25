---
format: aep.planning-md/2
id: review-result:output-ownership-adversary-wave19-pass1
kind: review-result
status: active
title: Output ownership adversary, pass 1
relations:
- reviews: story:review-output-ownership
revision: 1
---
unit: story:review-output-ownership, pass1, 050c416650119852d213d464dfb76c51673121c6 plus one additive test target
verdict: NEEDS-CHANGE
cases: executed 0→4, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 write surfaces; complete roots and retained fixture paths below
needs-coordinator: correct both concrete CLI findings, execute native macOS branch, integrate and run the full gate

`git --no-pager diff --stat` actual output is empty: the only addition is an untracked new test target, and the adversary does not stage files. Its exact supplemental `git --no-pager diff --no-index --stat /dev/null crates/edge/ess-cli/tests/output_ownership_adversary.rs` output is:

```text
 .../ess-cli/tests/output_ownership_adversary.rs    | 230 +++++++++++++++++++++
 1 file changed, 230 insertions(+)
```

The complete tests-only patch is `tests.patch`, SHA256 `2d94e528a45c8332f5f2a8dc7e0a0b356ae6544d436432dae9c8b18a4501a3a4`. The production/design/planning diff against the held candidate is empty. No existing case or assertion was modified. No implementation, Git index/ref, AEP, integration or cleanup mutation was performed.

1. Added cases and their first actual outcomes

All four cases were written before the first test execution. Each was selected alone before the target suite; there was no initial baseline rerun. The fresh target's prior executed count is 0. The prior 372 CLI cases/43 targets and 29 ownership cases are implementor observations, not execution in this pass. This pass did not repeat the old cut matrices or an unrelated package gate.

- `standalone_generation_refuses_directory_spelling_before_enrollment`: red at test line125, actual first command exit101, 0 passed/1 failed. Real CLI exits0 for `--out out/record.ts/` and publishes `out/record.ts` with enrollment. Expected refusal precedes any enrollment or file mutation.
- `composition_replaces_its_owned_companion_with_a_client_directory`: red at test line145, actual first command exit101, 0 passed/1 failed. Real compose first owns a companion at `anchor/client`; switching its complete selected output to a client directory at that same path is rejected by old preflight.
- `unicode_companions_follow_actual_native_alias_behavior_before_any_write`: green, actual first command exit0, 1 passed. This host's measured É.json/é.json lookup is distinct; both different companions publish and recovery is idempotent. The test also has an actual-alias branch requiring refusal before mutation. That branch has NOT executed on this Linux filesystem and is NOT macOS proof or a third finding.
- `rollback_preserves_an_unselected_owner_and_actual_readonly_file_modes`: green, actual first command exit0, 1 passed. The exact source engine snapshots an edited0400 owned preimage, shares its parent with another owner and an authored file, receives an injected error immediately after the first installation rename, recovers twice, then retires only its selected file. This is an injected callback error, not a killed child process.

After the first case's original red, formatting was applied to the new target and its still-unselected compose success postcondition was corrected to the actual third generated artifact, ess-client-plan.json. The directory-spelling assertion remains unchanged; the original first output remains retained. There were no compile/setup failures. All final source cases were subsequently exercised by the suite.

Every Cargo command used the assigned frozen Rust tools, own default target, `--offline --locked`, jobs2, test threads2, debug0 and incremental0. PATH begins `home-path:sha256:5d15c814f03498bd39d18210ab7d0c2059927041d2b3d5e622849b3754c82785`; RUSTC and RUSTDOC name its exact rustc/rustdoc binaries. The four RUSTC_WRAPPER/RUSTC_WORKSPACE_WRAPPER/CARGO_BUILD_RUSTC_WRAPPER/CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER variables were blank; RUSTFLAGS, CARGO_ENCODED_RUSTFLAGS, CARGO_TARGET_DIR and any CARGO_TARGET_*_RUSTFLAGS variables were unset. The existing `home-path:sha256:acd2952e2d81c785408e0d83690ab14a7df5bd58ff66a00731881d76f8fe98af` supplies the single native lld flag. TMPDIR was `home-path:sha256:2da94aea5297cd04f430e98c24a8087790a4b3ebef6cba9f751df7531036a000`. No dependency install/change or foreign mutable CLI execution occurred.

Actual first command: `cargo test --offline --locked -p ess-cli --test output_ownership_adversary standalone_generation_refuses_directory_spelling_before_enrollment -- --exact --nocapture`

stdout, verbatim:
```text

running 1 test
retained fixture: home-path:sha256:80a79386311eee61fe48e7a9236680a79224f8297ff023f8d6abaf1e3fa5343a
CLI ["schema", "typescript", "--schemas", "record.schema.json", "urn:record", "--root", "Record", "--out", "out/record.ts/"]: status=exit status: 0
stdout:
wrote out/record.ts/ from record.schema.json

stderr:

test standalone_generation_refuses_directory_spelling_before_enrollment ... FAILED

failures:

failures:
    standalone_generation_refuses_directory_spelling_before_enrollment

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.35s

```
stderr, verbatim:
```text
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.24
   Compiling serde_core v1.0.229
   Compiling syn v3.0.4
   Compiling memchr v2.8.3
   Compiling serde v1.0.229
   Compiling zmij v1.0.23
   Compiling itoa v1.0.18
   Compiling serde_json v1.0.151
   Compiling cfg-if v1.0.4
   Compiling serde_derive v1.0.229
   Compiling typenum v1.20.1
   Compiling hybrid-array v0.4.14
   Compiling syn v2.0.119
   Compiling block-buffer v0.12.1
   Compiling crypto-common v0.2.2
   Compiling const-oid v0.10.2
   Compiling foldhash v0.2.0
   Compiling allocator-api2 v0.2.21
   Compiling equivalent v1.0.2
   Compiling digest v0.11.3
   Compiling cpufeatures v0.3.1
   Compiling sha2 v0.11.0
   Compiling hashbrown v0.17.1
   Compiling serde_derive_internals v0.29.1
   Compiling schemars v0.8.22
   Compiling thiserror v2.0.20
   Compiling indexmap v2.14.1
   Compiling schemars_derive v0.8.22
   Compiling thiserror-impl v2.0.20
   Compiling dyn-clone v1.0.20
   Compiling ryu v1.0.23
   Compiling unsafe-libyaml v0.2.11
   Compiling serde_yaml v0.9.34+deprecated
   Compiling autocfg v1.5.1
   Compiling num-traits v0.2.19
   Compiling ess-primitives v0.20.0 (home-path:sha256:0448cef79d54a6c10bcaf54d8dcf7a6c2d80d3817aa7b1712cd9e8d15e4fafde)
   Compiling libc v0.2.189
   Compiling ess-domain v0.20.0 (home-path:sha256:fb0d3cfdbec5922fae5811232c5329d911b9bb04b8054c2b40c1cce2d8c1c290)
   Compiling num-integer v0.1.47
   Compiling heck v0.5.0
   Compiling version_check v0.9.5
   Compiling pulldown-cmark v0.13.4
   Compiling zerocopy v0.8.56
   Compiling bitflags v2.13.1
   Compiling getrandom v0.3.4
   Compiling ahash v0.8.12
   Compiling num-bigint v0.4.8
   Compiling ess-compiler v0.20.0 (home-path:sha256:61fbbc74da230123e24de6038806e1bed552dcff45eac2b7def9b6a26c949405)
   Compiling regex-syntax v0.8.11
   Compiling parking_lot_core v0.9.12
   Compiling ref-cast v1.0.27
   Compiling pulldown-cmark-escape v0.11.0
   Compiling unicase v2.9.0
   Compiling num-rational v0.4.2
   Compiling num-iter v0.1.46
   Compiling num-complex v0.4.6
   Compiling infra-domain v0.20.0 (home-path:sha256:cdc5105fbba6c2ce25f0503cb99971ea18af02f15fa0986d5a91c3e4b27b9843)
   Compiling ref-cast-impl v1.0.27
   Compiling aho-corasick v1.1.5
   Compiling scopeguard v1.2.0
   Compiling utf8parse v0.2.2
   Compiling smallvec v1.16.0
   Compiling once_cell v1.21.4
   Compiling regex-automata v0.4.18
   Compiling anstyle-parse v1.0.0
   Compiling lock_api v0.4.14
   Compiling num v0.4.3
   Compiling ess-gen v0.20.0 (home-path:sha256:e28bad7e5269af6ef6c6569fa082620d3e1c4bc4e556fb42a16dd33e9acfdc57)
   Compiling anstyle v1.0.14
   Compiling anstyle-query v1.1.5
   Compiling bit-vec v0.8.0
   Compiling colorchoice v1.0.5
   Compiling is_terminal_polyfill v1.70.2
   Compiling borrow-or-share v0.2.4
   Compiling unicode-general-category v1.1.0
   Compiling fluent-uri v0.4.1
   Compiling anstream v1.0.0
   Compiling bit-set v0.8.0
   Compiling fraction v0.17.0
   Compiling parking_lot v0.12.5
   Compiling infra-compiler v0.20.0 (home-path:sha256:e04b531a0094d3b54b4bcaaa21d4acae83ce37dfcecb0701a56c93e98799c772)
   Compiling strum_macros v0.28.0
   Compiling bytecount v0.6.9
   Compiling outref v0.5.2
   Compiling num-cmp v0.1.0
   Compiling clap_lex v1.1.0
   Compiling strsim v0.11.1
   Compiling percent-encoding v2.3.2
   Compiling micromap v0.3.0
   Compiling vsimd v0.8.0
   Compiling uuid-simd v0.8.0
   Compiling referencing v0.52.1
   Compiling clap_builder v4.6.6
   Compiling jsonschema-value v0.52.1
   Compiling strum v0.28.0
   Compiling infra-analyze v0.20.0 (home-path:sha256:5ccd58e348fa6ca9efd8298472b14474573f094eb83f23fd1154242561b4654e)
   Compiling fancy-regex v0.19.0
   Compiling regex v1.13.1
   Compiling jsonschema-regex v0.52.1
   Compiling clap_derive v4.6.4
   Compiling email_address v0.2.9
   Compiling data-encoding v2.11.1
   Compiling anyhow v1.0.104
   Compiling rustix v1.1.4
   Compiling jsonschema v0.52.1
   Compiling clap v4.6.6
   Compiling infra-spec v0.20.0 (home-path:sha256:b55cfa3f0407f62821a9277ab7fc635b2e907e13ffcec54ae8f305c22b00dcc7)
   Compiling ess-conformance v0.20.0 (home-path:sha256:f097b96d83e1a9fec02cea3b8a99e1d1faf82ee3687ee6d25dbfdb16efdb33f8)
   Compiling ess-realization v0.20.0 (home-path:sha256:2d2973089ed58e6cc811c0cd52395a30a94beea8f03b01ea989d60b1a5f4b752)
   Compiling semver v1.0.28
   Compiling base64 v0.22.1
   Compiling linux-raw-sys v0.12.1
   Compiling schema-contract v0.20.0 (home-path:sha256:06988eb364730e44f76826ebcf68c99c2dd4a2d45aac0c0b0c4fd10ece850064)
   Compiling ess-deployment v0.20.0 (home-path:sha256:8b83081769883a82bca50fac696c2f23bfbfab7c5e5f1d4e06f5109b72abb95a)
   Compiling ess-diff v0.20.0 (home-path:sha256:34c3b64f77795d1ad154fa147f85cef43760664b454fb05d71675d148adfd081)
   Compiling infra-project v0.20.0 (home-path:sha256:868215a4fb5191ce2bdc43b88829471862a8bdf2f0d8d418e1c2ab42fb9de105)
   Compiling ess-kubernetes v0.20.0 (home-path:sha256:6aba4809a32f090b27726e0f4a0e149ca6c01560b2c23e6416d3518cc2acf59a)
   Compiling ess-synth v0.20.0 (home-path:sha256:01094cf26bc1cb96b26550aae32b4f45de113fff2c823bec9b8189d9e8d5d85e)
   Compiling ess-composition v0.20.0 (home-path:sha256:f23422a3b440ddaa3581a7350f85bcabc8ed85f9e24a388f7575f2d2939390a9)
   Compiling ess-openapi v0.20.0 (home-path:sha256:947e30d2a82363124d8836dc6d5c61e02ceba4235b9b09ac6b608ff62622c6f7)
   Compiling ess-cli v0.20.0 (home-path:sha256:08c13a55a65df8e41c60fefd209a5bd424473fd419a0dffb531e715a1a406ddc)
    Finished `test` profile [unoptimized] target(s) in 1m 25s
     Running tests/output_ownership_adversary.rs (target/debug/deps/output_ownership_adversary-f917312615a5063e)

thread 'standalone_generation_refuses_directory_spelling_before_enrollment' (2257496) panicked at crates/edge/ess-cli/tests/output_ownership_adversary.rs:125:5:
a directory-spelled destination was silently published as a file
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: test failed, to rerun pass `-p ess-cli --test output_ownership_adversary`
```
Direct result:
```json
{"exit":101,"start":1788859373.786642108,"end":1788859459.328859761}

```

Actual first command: `cargo test --offline --locked -p ess-cli --test output_ownership_adversary composition_replaces_its_owned_companion_with_a_client_directory -- --exact --nocapture`

stdout, verbatim:
```text

running 1 test
retained fixture: home-path:sha256:c1a0300582093c6ef6b04d4161aec8f5af6b4f066cbd6ee464024474246d85c3
CLI ["compose", "--path", "home-path:sha256:e9e4883795b8a84c719b9576de5be5a49d1584b66b051ec9e38f1a241ecebea0", "--service", "todo=home-path:sha256:947a342c29e3a64764aceeaaededb486f83d42cd41103aa0a29ebd4d83a6e9ba", "--service", "usage=home-path:sha256:947a342c29e3a64764aceeaaededb486f83d42cd41103aa0a29ebd4d83a6e9ba", "--ownership-root", "anchor", "--out", "anchor/client"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/client

stderr:

CLI ["compose", "--path", "home-path:sha256:e9e4883795b8a84c719b9576de5be5a49d1584b66b051ec9e38f1a241ecebea0", "--service", "todo=home-path:sha256:947a342c29e3a64764aceeaaededb486f83d42cd41103aa0a29ebd4d83a6e9ba", "--service", "usage=home-path:sha256:947a342c29e3a64764aceeaaededb486f83d42cd41103aa0a29ebd4d83a6e9ba", "--ownership-root", "anchor", "--client-rust-out", "anchor/client"]: status=exit status: 1
stdout:

stderr:
error: output path has an incompatible file type or symlink: home-path:sha256:d9f0ae635c79c418e76e1c24b2ee8c255e17bb928d37214ed2e71d9a303a884a

test composition_replaces_its_owned_companion_with_a_client_directory ... FAILED

failures:

failures:
    composition_replaces_its_owned_companion_with_a_client_directory

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.35s


```
stderr, verbatim:
```text
   Compiling ess-cli v0.20.0 (home-path:sha256:08c13a55a65df8e41c60fefd209a5bd424473fd419a0dffb531e715a1a406ddc)
    Finished `test` profile [unoptimized] target(s) in 3.01s
     Running tests/output_ownership_adversary.rs (target/debug/deps/output_ownership_adversary-f917312615a5063e)

thread 'composition_replaces_its_owned_companion_with_a_client_directory' (2294467) panicked at crates/edge/ess-cli/tests/output_ownership_adversary.rs:145:5:
the selected compose owner's file must be replaceable by its complete client tree
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: test failed, to rerun pass `-p ess-cli --test output_ownership_adversary`

```
Direct result:
```json
{"case":"composition_replaces_its_owned_companion_with_a_client_directory","exit":101,"start":1788859570.010899518,"end":1788859573.400342799}

```

Actual first command: `cargo test --offline --locked -p ess-cli --test output_ownership_adversary unicode_companions_follow_actual_native_alias_behavior_before_any_write -- --exact --nocapture`

stdout, verbatim:
```text

running 1 test
retained fixture: home-path:sha256:e88cd3edc0473c3a1c51edc1d04bc0300bcad527e5d24c07f792a2448b216453
native É.json/é.json alias lookup: false
CLI ["compose", "--path", "home-path:sha256:e9e4883795b8a84c719b9576de5be5a49d1584b66b051ec9e38f1a241ecebea0", "--service", "todo=home-path:sha256:947a342c29e3a64764aceeaaededb486f83d42cd41103aa0a29ebd4d83a6e9ba", "--service", "usage=home-path:sha256:947a342c29e3a64764aceeaaededb486f83d42cd41103aa0a29ebd4d83a6e9ba", "--ownership-root", "anchor", "--out", "anchor/É.json", "--client-plan-out", "anchor/é.json"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/É.json; client plan written to anchor/é.json

stderr:

test unicode_companions_follow_actual_native_alias_behavior_before_any_write ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.45s


```
stderr, verbatim:
```text
    Finished `test` profile [unoptimized] target(s) in 0.16s
     Running tests/output_ownership_adversary.rs (target/debug/deps/output_ownership_adversary-f917312615a5063e)

```
Direct result:
```json
{"case":"unicode_companions_follow_actual_native_alias_behavior_before_any_write","exit":0,"start":1788859573.403335307,"end":1788859574.031173139}

```

Actual first command: `cargo test --offline --locked -p ess-cli --test output_ownership_adversary rollback_preserves_an_unselected_owner_and_actual_readonly_file_modes -- --exact --nocapture`

stdout, verbatim:
```text

running 1 test
retained fixture: home-path:sha256:32ccb01da90f6296dd53eb79bd90369e56c96a86a79b322e5687dc132a65ea3b
first installation interruption: Err(injected process-equivalent interruption after first install)
test rollback_preserves_an_unselected_owner_and_actual_readonly_file_modes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.48s


```
stderr, verbatim:
```text
    Finished `test` profile [unoptimized] target(s) in 0.09s
     Running tests/output_ownership_adversary.rs (target/debug/deps/output_ownership_adversary-f917312615a5063e)

```
Direct result:
```json
{"case":"rollback_preserves_an_unselected_owner_and_actual_readonly_file_modes","exit":0,"start":1788859574.032978269,"end":1788859575.622700859}

```

2. Exact added target suite

Command: `cargo test --offline --locked -p ess-cli --test output_ownership_adversary -- --nocapture`

Boundary: only the newly added four-case integration target; original targets are retained and not selected. This suite ran after all four cases had each executed alone. Actual exit101, executed4, passed2, failed2, ignored0, filtered0.

stdout, verbatim:
```text

running 4 tests
retained fixture: home-path:sha256:7ddaa919713a4b4d0e620a97708b306c14eb1f1a1bb37719b0ca0bf84497ca07
retained fixture: home-path:sha256:41e9ec3c98ab3e14623a9f793f13d76e9472609aab058e536cd952bdc4ff61ae
CLI ["compose", "--path", "home-path:sha256:e9e4883795b8a84c719b9576de5be5a49d1584b66b051ec9e38f1a241ecebea0", "--service", "todo=home-path:sha256:947a342c29e3a64764aceeaaededb486f83d42cd41103aa0a29ebd4d83a6e9ba", "--service", "usage=home-path:sha256:947a342c29e3a64764aceeaaededb486f83d42cd41103aa0a29ebd4d83a6e9ba", "--ownership-root", "anchor", "--out", "anchor/client"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/client

stderr:

CLI ["compose", "--path", "home-path:sha256:e9e4883795b8a84c719b9576de5be5a49d1584b66b051ec9e38f1a241ecebea0", "--service", "todo=home-path:sha256:947a342c29e3a64764aceeaaededb486f83d42cd41103aa0a29ebd4d83a6e9ba", "--service", "usage=home-path:sha256:947a342c29e3a64764aceeaaededb486f83d42cd41103aa0a29ebd4d83a6e9ba", "--ownership-root", "anchor", "--client-rust-out", "anchor/client"]: status=exit status: 1
stdout:

stderr:
error: output path has an incompatible file type or symlink: home-path:sha256:c97325d3b781cee1591d9a2cf10a08779666a629d1cac2a95edba8e92e1c5faa

test composition_replaces_its_owned_companion_with_a_client_directory ... FAILED
retained fixture: home-path:sha256:035a10c4861b71f70c3f7f1408aceefe4b0c729225937e4b9d2c504750ab6a6e
CLI ["schema", "typescript", "--schemas", "record.schema.json", "urn:record", "--root", "Record", "--out", "out/record.ts/"]: status=exit status: 0
stdout:
wrote out/record.ts/ from record.schema.json

stderr:

test standalone_generation_refuses_directory_spelling_before_enrollment ... FAILED
retained fixture: home-path:sha256:c4f5004f4e7e1e8c177617c8a09283d405a8e9341f434f87f7cb10d25ccf41fb
native É.json/é.json alias lookup: false
first installation interruption: Err(injected process-equivalent interruption after first install)
CLI ["compose", "--path", "home-path:sha256:e9e4883795b8a84c719b9576de5be5a49d1584b66b051ec9e38f1a241ecebea0", "--service", "todo=home-path:sha256:947a342c29e3a64764aceeaaededb486f83d42cd41103aa0a29ebd4d83a6e9ba", "--service", "usage=home-path:sha256:947a342c29e3a64764aceeaaededb486f83d42cd41103aa0a29ebd4d83a6e9ba", "--ownership-root", "anchor", "--out", "anchor/É.json", "--client-plan-out", "anchor/é.json"]: status=exit status: 0
stdout:
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to anchor/É.json; client plan written to anchor/é.json

stderr:

test unicode_companions_follow_actual_native_alias_behavior_before_any_write ... ok
test rollback_preserves_an_unselected_owner_and_actual_readonly_file_modes ... ok

failures:

failures:
    composition_replaces_its_owned_companion_with_a_client_directory
    standalone_generation_refuses_directory_spelling_before_enrollment

test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s


```
stderr, verbatim:
```text
    Finished `test` profile [unoptimized] target(s) in 0.17s
     Running tests/output_ownership_adversary.rs (target/debug/deps/output_ownership_adversary-f917312615a5063e)

thread 'composition_replaces_its_owned_companion_with_a_client_directory' (2303742) panicked at crates/edge/ess-cli/tests/output_ownership_adversary.rs:145:5:
the selected compose owner's file must be replaceable by its complete client tree
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'standalone_generation_refuses_directory_spelling_before_enrollment' (2303746) panicked at crates/edge/ess-cli/tests/output_ownership_adversary.rs:125:5:
a directory-spelled destination was silently published as a file
error: test failed, to rerun pass `-p ess-cli --test output_ownership_adversary`

```
Direct result:
```json
{"exit":101,"start":1788859622.887025781,"end":1788859624.159238441}
```

Strict Clippy: `cargo clippy --offline --locked -p ess-cli --test output_ownership_adversary -- -D warnings`, actual0; direct duration18.473273786s. Formatting: frozen rustfmt `--check --config skip_children=true --edition 2021 crates/edge/ess-cli/tests/output_ownership_adversary.rs`, actual0, empty stdout/stderr. Complete direct lint/format streams and result JSON are retained alongside this report. First cold case command took85.542217653s including compilation; suite command took1.272212660s. These are command wall-clock deltas, not model cost metrics.

3. Findings, origin and real reach

| Source signature | Verdict / origin | What was measured | What reaches it |
|---|---|---|---|
| crates/edge/ess-cli/src/output_ownership/mod.rs:280 | NEEDS-CHANGE / introduced | The standalone publisher drops trailing directory syntax and returns success after enrolling and creating a file for `--out out/record.ts/`. Test line125, exact first exit101, real CLI exit0. | `schema::typescript` calls `write_projection` at schema.rs:252, which now calls this helper; the retained fixture invokes the public schema typescript command with an ordinary schema and an existing output parent. The helper's `Path::file_name` normalizes away the trailing separator before constructing the new destination. |
| crates/edge/ess-cli/src/main.rs:2058 | NEEDS-CHANGE / introduced | Compose's preflight rejects an owned companion-to-client directory transition before the ownership transaction can retire the selected owner's file. Test line145, exact first exit101; first real compose exit0, second exit1. | The public compose command owns `anchor/client` through `--out`, then changes its complete same-anchor selection to `--client-rust-out anchor/client`. `preflight_generated_files` invokes `resolve_output_directory` and rejects that owned file before `publish_composition_outputs` is reached. |

Both findings cover candidate050c416 and the exact additive target. They are acceptance/boundary defects with current CLI workflows, not invented malformed checkpoint states. The first is introduced by the new named publisher replacing the baseline's final `fs::rename(&temporary, path)` with a normalized path. The published baseline source was read using `git show 1e618d2:crates/edge/ess-cli/src/schema.rs`; the tree was never moved or rebuilt at the base. For the second, the preflight guard predates this change, but its incompatibility is exposed by the newly enrolled, same-owner compose workflow and the new selected transition contract; the base has no enrolled compose owner or --ownership-root workflow. No claim is made that old ESS performed recoverable transitions.

The selected design's product behavior explicitly admits complete compose output replacement, selected stale retirement and owned file/directory transitions while preserving authored blockers. Retained W07 names file-to-directory and directory-to-file transitions; G04/W14 names changing companion/client sets. The existing engine-only transition case passes because it bypasses this caller. A correction should preserve lexical native-path validation while allowing the ownership engine to decide whether an existing shape is owned and replaceable. Ordinary filename spelling must be validated before the named publisher discards separator/dot syntax.

No additional judgement finding is returned. Root's known finite declaration-classification task was not counted as a defect. Native macOS execution remains coordinator-owned; the Unicode test gives it a concrete case to select.

4. Boundaries attacked without a break

Actual edited0400 preimages, an unselected owner sharing the parent, authored siblings, injected post-install error, repeated recovery and selected stale retirement remained intact in the added recovery case.
Distinct native Unicode companion names published distinct contents and recovered idempotently on the measured Linux filesystem; the actual alias branch remains unexecuted here.

5. Source/native retention and resource handoff

Assigned managed checkout: `home-path:sha256:a1710fb0f77092da51ecd0a34a3f643944ad8a9c2920772e8b9e842e0cf3f6e4`; branch `probe/output-ownership-wave19`; unchanged HEAD050c416650119852d213d464dfb76c51673121c6. The whole candidate delta, selected contract/model, touched code/tests and production callers were read; retained source-scoping records supplied the exact W01–W17 wording. Previous consumer attacks and removed trees were not resumed.

Full brief SHA2562a282b834bab69e697a0cb1b3de5a851c4dbd2fc3bcd67f94eebb461fe93ec48; resource grant SHA25648c1e384f952e46cdbff6326668d745676838c743201ff08ce570e6ad658bf9b; exact adversary0.8.1 charter SHA25675bb7514688c3ba89f5788b6150e1ce35fedb42c85a85d717c4dc979c2e27910. The initial compiler hold was respected, and only root's explicit handback admitted the first build.

- `source.tar`: all1230 current tracked/additive nonignored source files, with `source-files.nul` and per-file `source-files.sha256`; SHA2566a21a0c4f331535d345df7e460dc0084c4bf471f4521760dad9b9fd738cdcf8e. Creation and full tar content/metadata readback both actual0.
- `native.tar`: complete final own target excluding this report scratch, plus complete assigned external TMPDIR;2157 native entries. SHA2560dadfc5cf7672adf8916da34b84d15cabddc178c40384afcc4b0df876ab3f1e4. Creation and full tar content/metadata readback both actual0; inventory in native-inventory.txt. No extraction or cleanup was used.
- Exact candidate CLI `target/debug/ess`: SHA256e29737a2ad9f2281c058dd265b8f9922d4d48812610347d0ccabd94e777b2ab4, matching the separately supplied frozen candidate hash. It was built in this own target. No copy or execution from another checkout was needed.
- Final test binary `target/debug/deps/output_ownership_adversary-f917312615a5063e`: SHA256910e032635a4cd8b61a57df4c5a4d0ff53635476470a43b9c2dbaa4ba70ee3bf. Final added test source SHA256a2fd55ec049f1ad205b114e1053bf868e0912bb142ff2b74cc3ec42e73e88e77.

Only one adversary Cargo producer ran, jobs2; no historical checker, mutation build, Go or browser runtime ran. Before the cold build, target+TMP allocation was24576bytes, SSD free99791167488bytes and MemAvailable38573888KiB. During compilation sampled target allocations included121810944 and389525504bytes; the lowest observed MemAvailable was36264096KiB. After target Clippy allocation was764219392bytes plus233472bytes TMP. After native/source retention allocation was1592254464bytes plus233472bytes TMP, with SSD free96079241216bytes and MemAvailable40984428KiB. All measurements stayed below the5GiB allocation cap and above the2GiB free/8GiB memory floors. No resource stop occurred; these are periodic samples, not invented continuous maxima. Root's separately owned Node/site work may affect global free-memory/SSD observations.

All owned test/Clippy processes completed, all tool sessions were consumed, and process inspection found no remaining adversary Cargo producer. The compiler slot was explicitly returned to root before retention, allowing root's separate site-lab work. The held candidate and TMP stayed quiescent through archive readback. Own lease `ess-output-ownership-adversary-wave19-pass1` release is recorded in lease-release.stdout/stderr/result.json; the coordinator owns the next readback, correction, integration and cleanup. This is pass1 of the two-pass budget; no extra attack was launched. The harness exposes no reliable numerical model-token balance or billed cost, so none is invented.

6. Every outside write surface

`home-path:sha256:2da94aea5297cd04f430e98c24a8087790a4b3ebef6cba9f751df7531036a000` contains all retained test fixtures and any native compiler temporary files created under the assigned TMPDIR. The complete surviving tree is in native.tar; the eight retained fixture roots are:

- home-path:sha256:80a79386311eee61fe48e7a9236680a79224f8297ff023f8d6abaf1e3fa5343a
- home-path:sha256:c1a0300582093c6ef6b04d4161aec8f5af6b4f066cbd6ee464024474246d85c3
- home-path:sha256:e88cd3edc0473c3a1c51edc1d04bc0300bcad527e5d24c07f792a2448b216453
- home-path:sha256:32ccb01da90f6296dd53eb79bd90369e56c96a86a79b322e5687dc132a65ea3b
- home-path:sha256:7ddaa919713a4b4d0e620a97708b306c14eb1f1a1bb37719b0ca0bf84497ca07
- home-path:sha256:41e9ec3c98ab3e14623a9f793f13d76e9472609aab058e536cd952bdc4ff61ae
- home-path:sha256:035a10c4861b71f70c3f7f1408aceefe4b0c729225937e4b9d2c504750ab6a6e
- home-path:sha256:c4f5004f4e7e1e8c177617c8a09283d405a8e9341f434f87f7cb10d25ccf41fb

Cargo updated existing `home-path:sha256:8aee2fa6471d9cc82d7abbceaa38d5c76572ff8fb9553e64549bdeafc1128317` bookkeeping (observed mtime1788859302→1788859623 before the lint). Its existing `.package-cache` and `.package-cache-mutate` lock files were used; their size/mtime stayed unchanged. No dependency source/cache was removed or installed. These shared Cargo bookkeeping paths are not coordinator-owned disposable outputs.

The own lifecycle hook updated `home-path:sha256:4e6427430bd5ae6f4b6ed99313c321c933a044122ab45412f907cd91f6a1aab0` for this assigned session lease. Only that lease was acquired/renewed/released; no other lease or worktree lifecycle was changed. Shared registry bookkeeping is not an output to retire. All reports, source/native archives, patches and raw logs are inside the assigned checkout scratch. No /tmp, tmpfs, Atlas/Website, network integration or unassigned scratch write was made.

```findings
- file: crates/edge/ess-cli/src/output_ownership/mod.rs
  line: 280
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: The standalone publisher drops trailing directory syntax and returns success after enrolling and creating a file for `--out out/record.ts/`.
- file: crates/edge/ess-cli/src/main.rs
  line: 2058
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Compose's preflight rejects an owned companion-to-client directory transition before the ownership transaction can retire the selected owner's file.
```
