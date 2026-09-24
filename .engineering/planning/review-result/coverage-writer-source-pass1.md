---
format: aep.planning-md/1
id: review-result:coverage-writer-source-pass1
kind: review-result
status: active
title: 'Coverage writer source pass 1: full refusal rendering and strict Go diagnostic'
relations:
- reviews: story:review-conformance-coverage
revision: 1
---
unit: ESS coverage writer source pass 1, 874962d3c7f84d0337bb1477892da065aeadc2fd
verdict: NEEDS-CHANGE
cases: executed 642→646, red 3
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 513 retained paths under home-path:sha256:c7cdced08eb66cb30394fa5c6ddfe52ed26ffea4906e648caf1ea6e9f20537d9
needs-coordinator: route explicit D1 rendering correction and Go diagnostic; root owns source freeze, second attack, integration and actual AEP correspondence

1. git --no-pager diff --stat

```text
```

The command has empty output because these two additive test files are untracked. No Git staging was performed. The supplemental `git --no-pager diff --no-index --stat /dev/null <new-test>` observations are:

```text
 .../tests/coverage_writer_adversary_pass1.rs       | 186 +++++++++++++++++++++
 1 file changed, 186 insertions(+)
 .../tests/coverage_writer_adversary_pass1.rs       | 159 +++++++++++++++++++++
 1 file changed, 159 insertions(+)
```

Exact new paths:

```text
crates/edge/ess-cli/tests/coverage_writer_adversary_pass1.rs
crates/verify/ess-conformance/tests/coverage_writer_adversary_pass1.rs
```

All 1,100 originally tracked source files still match source-before.json, including all inherited assertions, production, lock/manifests, planning and public documentation. HEAD remains the frozen subject. The original implementation handoff's 35 seal entries were rehashed unchanged. The 27 copied non-Go preparation fixtures (including the semantic plan, input catalog, original model/authored bytes and seven legacy suite oracles) equal the previously sealed independent preparation byte for byte. Source-check-final.json, original-handoff-preserved.json, independent-fixtures-preserved.json and new-test-source-manifest.json retain those checks.

2. New cases and their focused observations, before the suite

The first authored case was the actual-browser name boundary. It ran alone and passed; no preemptive baseline or broader suite ran. It generated a real Billing replay bundle, verified Rust admission and actual Firefox state, then changed only a command name to append LF and recomputed the selected exact-byte reference. Both Rust and Firefox refused the changed original; the browser created neither a coverage banner nor a mounted replay app. The regex-boundary hypothesis was rejected, not promoted to a finding.

The remaining three cases were authored before their own focused runs. The two D1 cases compile the unchanged pinned Billing model/source fixtures through public compiler APIs, use actual synthesis/authored compilation to obtain original typed diagnostics, and compare the new public coverage builder's messages to full Refusal::to_string. The generated case measures all eight repeated missing-invariant occurrences. The authored case records both the one duplicate-source occurrence and two repeated undeclared-view occurrences before its first failing equality. Neither changes a fixture or loses an original assertion.

The Go diagnostic case starts with an actual CLI `conform author --suite-format 5 --format json` output, emits its admitted input through go::emit_input, and calls public Run against a Target whose BeginScenario returns ErrUnsupported. It checks the correctly emitted report first: selected count 1, skipped count 1, complete_inventory, inconclusive conformance, strict process nonzero. Only the assertion against the contradictory diagnostic is red. This is a diagnostic defect, not a false passing report or incorrect outcome partition.

Final case locations: CLI test line 9 (Go diagnostic, red), CLI test line 96 (browser refusal, green), conformance test line 43 (generated D1, red) and conformance test line 89 (authored D1, red). First-run logs retain their original pre-formatting line numbers. Rustfmt later formatted only these new files through stdin/stdout; before-format snapshots and formatter stderr are retained. No assertion was removed or relaxed.

The D1 authority must be read precisely. The original main binding and initial implementor brief required nonempty text including the original cause; they did not independently require the full Display header/help. The pre-writer root-resolutions.json selected full typed Refusal Display, SHA256 6feb6754888aefd7fbd9b371fdc09ed42ec30bbcbef3962176d7e1a9f8ca9f8b. During this review root explicitly recorded that narrower correspondence decision in the coordinator transport section “Concrete refusal rendering for coverage,” lines 92–103, and the immutable decision body. Initial and clarified transport snapshots are retained separately. The D1 red cases assert that selected correspondence decision, not an invented stronger interpretation of the earlier published cause-text minimum. Root's decision authorizes the later correction; this reviewer made none.

Full actual focused command receipts and output follow. `run-lane.py` records exact argv, cwd, configured environment, unset wrapper variables, test-source hashes, free-space measurements, process exits and wall seconds; it contains no product checker or semantic oracle.

### focused-first

```json
{
  "argv": [
    "cargo",
    "test",
    "--locked",
    "--offline",
    "-p",
    "ess-cli",
    "--test",
    "coverage_writer_adversary_pass1",
    "browser_refuses_a_command_name_with_a_final_line_feed_before_replay_state",
    "--",
    "--exact",
    "--nocapture"
  ],
  "cwd": "home-path:sha256:81a8aaad60388f74814838864c1b3182e25ef96fb5f2eac69a3110c015b31d04",
  "environment": {
    "PATH": "home-path:sha256:d972722bf5a8a5abf3ab363acc673b42b0d50eca552e59c1ad0efea4f8e3a23e",
    "RUSTC": "home-path:sha256:da7abda9718cfd0d0c1e74f56f34135d83b4a908741f60a13ad1ed963424c535",
    "RUSTDOC": "home-path:sha256:cfcca0fbc01edced245f4082ef3f4032907e9866e0d726bc4bd1861d26fbd53d",
    "TMPDIR": "home-path:sha256:a85023f45fdaccd57c62a4a1b51cee490171a57c6bb3db6c9cefabd69f61a130",
    "GOCACHE": "home-path:sha256:0c1bef61d39b03d4967ac4d3b1cdd105e585a69dc1e44ea973cffae93441bbc4",
    "GOMODCACHE": "home-path:sha256:c929e6078a0cc5289ec52e6bb58446312523dbff76bfa246689e269714ee6c63",
    "CARGO_HOME": "home-path:sha256:d30f9c4d89bf9b9f39d84a2047df7cbaa5c6a8acb7e20684ab0d33507967c60f",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:c7cdced08eb66cb30394fa5c6ddfe52ed26ffea4906e648caf1ea6e9f20537d9",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:f987e1de7e4f25b475aad4607bb483124032626ac901e6efbe02ae8aa697509e",
    "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_DEBUG": "0",
    "CARGO_CACHE_RUSTC_INFO": "0",
    "CARGO_BUILD_JOBS": "2",
    "CARGO_NET_OFFLINE": "true"
  },
  "unset": [
    "CARGO_TARGET_DIR",
    "RUSTC_WRAPPER",
    "SCCACHE_SERVER_UDS"
  ],
  "free_before": 22737612800,
  "started_epoch": 1788724849.680339,
  "new_tests": {
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass1.rs": "59d2ca9660e7186d736356a2b44465327e55b905435518067916c1bd3dbc6fbb"
  },
  "exit": 0,
  "elapsed_seconds": 38.09686570696067,
  "free_after": 22698106880
}
```

```text
   Compiling proc-macro2 v1.0.107
   Compiling unicode-ident v1.0.24
   Compiling quote v1.0.47
   Compiling serde_core v1.0.229
   Compiling syn v3.0.4
   Compiling memchr v2.8.3
   Compiling serde v1.0.229
   Compiling zmij v1.0.23
   Compiling cfg-if v1.0.4
   Compiling itoa v1.0.18
   Compiling serde_json v1.0.151
   Compiling serde_derive v1.0.229
   Compiling typenum v1.20.1
   Compiling syn v2.0.119
   Compiling hybrid-array v0.4.14
   Compiling block-buffer v0.12.1
   Compiling crypto-common v0.2.2
   Compiling foldhash v0.2.0
   Compiling equivalent v1.0.2
   Compiling const-oid v0.10.2
   Compiling allocator-api2 v0.2.21
   Compiling digest v0.11.3
   Compiling hashbrown v0.17.1
   Compiling cpufeatures v0.3.1
   Compiling sha2 v0.11.0
   Compiling serde_derive_internals v0.29.1
   Compiling schemars v0.8.22
   Compiling thiserror v2.0.20
   Compiling schemars_derive v0.8.22
   Compiling indexmap v2.14.1
   Compiling thiserror-impl v2.0.20
   Compiling unsafe-libyaml v0.2.11
   Compiling ryu v1.0.23
   Compiling dyn-clone v1.0.20
   Compiling serde_yaml v0.9.34+deprecated
   Compiling autocfg v1.5.1
   Compiling ess-primitives v0.20.0 (home-path:sha256:023a7ec9c4baa6d1cc2b055c33ff4f284fda7425258e04b1e2d84a65bca2ff39)
   Compiling num-traits v0.2.19
   Compiling libc v0.2.189
   Compiling ess-domain v0.20.0 (home-path:sha256:5dc48ee32bdcfb6b6531fcede0e40e5bf2c6b1fee19aeff3b8b7224c54dffe12)
   Compiling num-integer v0.1.47
   Compiling getrandom v0.3.4
   Compiling zerocopy v0.8.56
   Compiling version_check v0.9.5
   Compiling heck v0.5.0
   Compiling pulldown-cmark v0.13.4
   Compiling ahash v0.8.12
   Compiling num-bigint v0.4.8
   Compiling ess-compiler v0.20.0 (home-path:sha256:3bc6067a43285b74514bb9af08638497ee79278b1a934119ee774460e105b42f)
   Compiling bitflags v2.13.1
   Compiling unicase v2.9.0
   Compiling pulldown-cmark-escape v0.11.0
   Compiling ref-cast v1.0.27
   Compiling parking_lot_core v0.9.12
   Compiling regex-syntax v0.8.11
   Compiling num-rational v0.4.2
   Compiling num-iter v0.1.46
   Compiling num-complex v0.4.6
   Compiling ref-cast-impl v1.0.27
   Compiling aho-corasick v1.1.5
   Compiling once_cell v1.21.4
   Compiling smallvec v1.16.0
   Compiling utf8parse v0.2.2
   Compiling scopeguard v1.2.0
   Compiling lock_api v0.4.14
   Compiling anstyle-parse v1.0.0
   Compiling regex-automata v0.4.18
   Compiling num v0.4.3
   Compiling ess-gen v0.20.0 (home-path:sha256:9becbd77f058900ac1137c29d4ab62344dce2d3bf8292c8d48ea36f6ab645cda)
   Compiling infra-domain v0.20.0 (home-path:sha256:17ff7adea08c8d7f520ef548c6782574d0845e3fea23333d4c9b2059eec0bc1a)
   Compiling anstyle v1.0.14
   Compiling colorchoice v1.0.5
   Compiling anstyle-query v1.1.5
   Compiling unicode-general-category v1.1.0
   Compiling borrow-or-share v0.2.4
   Compiling bit-vec v0.8.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstream v1.0.0
   Compiling bit-set v0.8.0
   Compiling infra-compiler v0.20.0 (home-path:sha256:bed6271e7b917c3aab1b762a9e7a7f1ff2f9741e8573beb74f95f8deaa73043a)
   Compiling fluent-uri v0.4.1
   Compiling fraction v0.17.0
   Compiling parking_lot v0.12.5
   Compiling strum_macros v0.28.0
   Compiling bytecount v0.6.9
   Compiling strsim v0.11.1
   Compiling vsimd v0.8.0
   Compiling num-cmp v0.1.0
   Compiling percent-encoding v2.3.2
   Compiling micromap v0.3.0
   Compiling clap_lex v1.1.0
   Compiling outref v0.5.2
   Compiling clap_builder v4.6.6
   Compiling uuid-simd v0.8.0
   Compiling referencing v0.52.1
   Compiling jsonschema-value v0.52.1
   Compiling strum v0.28.0
   Compiling infra-analyze v0.20.0 (home-path:sha256:692408daeac51b82bcbca16da5b417c8027986bb97bba5454630d80dce1ecfef)
   Compiling fancy-regex v0.19.0
   Compiling regex v1.13.1
   Compiling jsonschema-regex v0.52.1
   Compiling clap_derive v4.6.4
   Compiling email_address v0.2.9
   Compiling data-encoding v2.11.1
   Compiling anyhow v1.0.104
   Compiling jsonschema v0.52.1
   Compiling clap v4.6.6
   Compiling infra-spec v0.20.0 (home-path:sha256:5fe1019747531706fddcdc48309c0267f716e1362016a58ffa8d78dd92860e3d)
   Compiling ess-conformance v0.20.0 (home-path:sha256:fb2af4db425baf24f833fe7f0be66e10d98c52642fdd023fe8c8d2f68266c344)
   Compiling ess-realization v0.20.0 (home-path:sha256:a56480f6286fbe5ae68fed8d524230f135205d34bb6bcb110902eb6c6eaf2f8b)
   Compiling semver v1.0.28
   Compiling base64 v0.22.1
   Compiling ess-deployment v0.20.0 (home-path:sha256:0db4f9f65afa6dfb9bfe6134f097a4511a1bbe5f13085097618e052df0e00c93)
   Compiling schema-contract v0.20.0 (home-path:sha256:59e491e7ffb494ae9905baec320f21cf3d49d97493d328be25c7ea7896369370)
   Compiling ess-diff v0.20.0 (home-path:sha256:5455152f24506d134276c0fa17104395c6cd66014ed2c8978ece31453da08d17)
   Compiling infra-project v0.20.0 (home-path:sha256:2518a4648c139107e578e6fb934f325373f31799eefe2eeb85be8c3643830dcf)
   Compiling ess-kubernetes v0.20.0 (home-path:sha256:b48f74815ced76f6c132664a9490e73fa3e7f91a4c397d97387871f45cb03a64)
   Compiling ess-synth v0.20.0 (home-path:sha256:86d0c66b43ae445f8298081b67d4624cc72d3379d7cbebd68899127289ffa9c9)
   Compiling ess-composition v0.20.0 (home-path:sha256:cf9c6e2be3ba3f0b1c2d4f47b62ab1102fe586f5d38602cdde999e25648dcc7a)
   Compiling ess-openapi v0.20.0 (home-path:sha256:bb891c3627776b8febf75959df9c028f38f33b1327e40dc25af05b9a4235c053)
   Compiling ess-cli v0.20.0 (home-path:sha256:5a10dd49c67cef5019797350b37afacc7816747d7c22f5c97b15b2438867815c)
    Finished `test` profile [unoptimized] target(s) in 36.41s
     Running tests/coverage_writer_adversary_pass1.rs (target/debug/deps/coverage_writer_adversary_pass1-96fa9eb4cce59dee)

running 1 test
Rust refusal: $suite: InvalidSuite: invalid qualified name identifier "billing.invoice.CreateInvoice\n": segment "CreateInvoice\n" contains '\n' at line 1 column 2118; ; Firefox: {"admitted":false,"banner":false,"error":"Error: Invalid coverage replay: invalid qualified name","mounted":false}
test browser_refuses_a_command_name_with_a_final_line_feed_before_replay_state ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.67s

```

Actual exit 0; wall 38.096866 seconds.

### focused-d1-generated

```json
{
  "argv": [
    "cargo",
    "test",
    "--locked",
    "--offline",
    "-p",
    "ess-conformance",
    "--test",
    "coverage_writer_adversary_pass1",
    "d1_generated_inventory_preserves_full_original_typed_refusal_rendering",
    "--",
    "--exact",
    "--nocapture"
  ],
  "cwd": "home-path:sha256:81a8aaad60388f74814838864c1b3182e25ef96fb5f2eac69a3110c015b31d04",
  "environment": {
    "PATH": "home-path:sha256:d972722bf5a8a5abf3ab363acc673b42b0d50eca552e59c1ad0efea4f8e3a23e",
    "RUSTC": "home-path:sha256:da7abda9718cfd0d0c1e74f56f34135d83b4a908741f60a13ad1ed963424c535",
    "RUSTDOC": "home-path:sha256:cfcca0fbc01edced245f4082ef3f4032907e9866e0d726bc4bd1861d26fbd53d",
    "TMPDIR": "home-path:sha256:a85023f45fdaccd57c62a4a1b51cee490171a57c6bb3db6c9cefabd69f61a130",
    "GOCACHE": "home-path:sha256:0c1bef61d39b03d4967ac4d3b1cdd105e585a69dc1e44ea973cffae93441bbc4",
    "GOMODCACHE": "home-path:sha256:c929e6078a0cc5289ec52e6bb58446312523dbff76bfa246689e269714ee6c63",
    "CARGO_HOME": "home-path:sha256:d30f9c4d89bf9b9f39d84a2047df7cbaa5c6a8acb7e20684ab0d33507967c60f",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:c7cdced08eb66cb30394fa5c6ddfe52ed26ffea4906e648caf1ea6e9f20537d9",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:feaa2c9e7feb4aadbc8f00741959c681dd2fb7c77685315947e933de8b0f0bb4",
    "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_DEBUG": "0",
    "CARGO_CACHE_RUSTC_INFO": "0",
    "CARGO_BUILD_JOBS": "2",
    "CARGO_NET_OFFLINE": "true"
  },
  "unset": [
    "CARGO_TARGET_DIR",
    "RUSTC_WRAPPER",
    "SCCACHE_SERVER_UDS"
  ],
  "free_before": 22410145792,
  "started_epoch": 1788725250.8780866,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass1.rs": "65eccc148c09d4d6b3fa88aa6ed8b210b266aee5775c0a65e05cde54293008ae",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass1.rs": "5815cdd2a6735b63565fa17dce9348f9f6382b94d468f64e60dd566a8fc453f9"
  },
  "exit": 101,
  "elapsed_seconds": 14.094496860983782,
  "free_after": 22245826560
}
```

```text
   Compiling serde_core v1.0.229
   Compiling syn v3.0.4
   Compiling serde v1.0.229
   Compiling serde_derive v1.0.229
   Compiling serde_json v1.0.151
   Compiling thiserror-impl v2.0.20
   Compiling hashbrown v0.17.1
   Compiling schemars v0.8.22
   Compiling indexmap v2.14.1
   Compiling thiserror v2.0.20
   Compiling ess-primitives v0.20.0 (home-path:sha256:023a7ec9c4baa6d1cc2b055c33ff4f284fda7425258e04b1e2d84a65bca2ff39)
   Compiling serde_yaml v0.9.34+deprecated
   Compiling ess-domain v0.20.0 (home-path:sha256:5dc48ee32bdcfb6b6531fcede0e40e5bf2c6b1fee19aeff3b8b7224c54dffe12)
   Compiling ess-compiler v0.20.0 (home-path:sha256:3bc6067a43285b74514bb9af08638497ee79278b1a934119ee774460e105b42f)
   Compiling ess-gen v0.20.0 (home-path:sha256:9becbd77f058900ac1137c29d4ab62344dce2d3bf8292c8d48ea36f6ab645cda)
   Compiling ess-conformance v0.20.0 (home-path:sha256:fb2af4db425baf24f833fe7f0be66e10d98c52642fdd023fe8c8d2f68266c344)
    Finished `test` profile [unoptimized] target(s) in 13.99s
     Running tests/coverage_writer_adversary_pass1.rs (target/debug/deps/coverage_writer_adversary_pass1-e33d0eba38e6ca1d)

running 1 test

thread 'd1_generated_inventory_preserves_full_original_typed_refusal_rendering' (1116179) panicked at crates/verify/ess-conformance/tests/coverage_writer_adversary_pass1.rs:44:5:
assertion `left == right` failed: D1 selected full Refusal::to_string, including original prefix and help
  left: [(Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CancelInvoice)), outcome: OutcomeName(cancelled) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CancelInvoice)), outcome: OutcomeName(cancelled) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CreateInvoice)), outcome: OutcomeName(accepted) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CreateInvoice)), outcome: OutcomeName(accepted) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.IssueInvoice)), outcome: OutcomeName(issued) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.IssueInvoice)), outcome: OutcomeName(issued) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.PayInvoice)), outcome: OutcomeName(settled) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.PayInvoice)), outcome: OutcomeName(settled) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity")]
 right: [(Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CancelInvoice)), outcome: OutcomeName(cancelled) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.CancelInvoice/cancelled`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CancelInvoice)), outcome: OutcomeName(cancelled) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.CancelInvoice/cancelled`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CreateInvoice)), outcome: OutcomeName(accepted) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.CreateInvoice/accepted`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CreateInvoice)), outcome: OutcomeName(accepted) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.CreateInvoice/accepted`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.IssueInvoice)), outcome: OutcomeName(issued) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.IssueInvoice/issued`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.IssueInvoice)), outcome: OutcomeName(issued) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.IssueInvoice/issued`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.PayInvoice)), outcome: OutcomeName(settled) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.PayInvoice/settled`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.PayInvoice)), outcome: OutcomeName(settled) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.PayInvoice/settled`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes")]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test d1_generated_inventory_preserves_full_original_typed_refusal_rendering ... FAILED

failures:

failures:
    d1_generated_inventory_preserves_full_original_typed_refusal_rendering

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.09s

error: test failed, to rerun pass `-p ess-conformance --test coverage_writer_adversary_pass1`
```

Actual exit 101; wall 14.094497 seconds.

### focused-d1-authored

```json
{
  "argv": [
    "cargo",
    "test",
    "--locked",
    "--offline",
    "-p",
    "ess-conformance",
    "--test",
    "coverage_writer_adversary_pass1",
    "d1_authored_inventory_preserves_full_original_typed_refusal_rendering",
    "--",
    "--exact",
    "--nocapture"
  ],
  "cwd": "home-path:sha256:81a8aaad60388f74814838864c1b3182e25ef96fb5f2eac69a3110c015b31d04",
  "environment": {
    "PATH": "home-path:sha256:d972722bf5a8a5abf3ab363acc673b42b0d50eca552e59c1ad0efea4f8e3a23e",
    "RUSTC": "home-path:sha256:da7abda9718cfd0d0c1e74f56f34135d83b4a908741f60a13ad1ed963424c535",
    "RUSTDOC": "home-path:sha256:cfcca0fbc01edced245f4082ef3f4032907e9866e0d726bc4bd1861d26fbd53d",
    "TMPDIR": "home-path:sha256:a85023f45fdaccd57c62a4a1b51cee490171a57c6bb3db6c9cefabd69f61a130",
    "GOCACHE": "home-path:sha256:0c1bef61d39b03d4967ac4d3b1cdd105e585a69dc1e44ea973cffae93441bbc4",
    "GOMODCACHE": "home-path:sha256:c929e6078a0cc5289ec52e6bb58446312523dbff76bfa246689e269714ee6c63",
    "CARGO_HOME": "home-path:sha256:d30f9c4d89bf9b9f39d84a2047df7cbaa5c6a8acb7e20684ab0d33507967c60f",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:c7cdced08eb66cb30394fa5c6ddfe52ed26ffea4906e648caf1ea6e9f20537d9",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:135e8a62e8578300b80bb4d52086bda26473cbb99b67be2fef556e4552e034ea",
    "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_DEBUG": "0",
    "CARGO_CACHE_RUSTC_INFO": "0",
    "CARGO_BUILD_JOBS": "2",
    "CARGO_NET_OFFLINE": "true"
  },
  "unset": [
    "CARGO_TARGET_DIR",
    "RUSTC_WRAPPER",
    "SCCACHE_SERVER_UDS"
  ],
  "free_before": 22047150080,
  "started_epoch": 1788725286.7496128,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass1.rs": "65eccc148c09d4d6b3fa88aa6ed8b210b266aee5775c0a65e05cde54293008ae",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass1.rs": "5815cdd2a6735b63565fa17dce9348f9f6382b94d468f64e60dd566a8fc453f9"
  },
  "exit": 101,
  "elapsed_seconds": 0.10880930896382779,
  "free_after": 22047133696
}
```

```text
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/coverage_writer_adversary_pass1.rs (target/debug/deps/coverage_writer_adversary_pass1-e33d0eba38e6ca1d)

running 1 test

thread 'd1_authored_inventory_preserves_full_original_typed_refusal_rendering' (1118877) panicked at crates/verify/ess-conformance/tests/coverage_writer_adversary_pass1.rs:71:9:
assertion `left == right` failed: D1 accepted-duplicate: selected full Refusal::to_string, with root-relative source and help
  left: [("b.yaml", Some(Authored { domain: DomainRef(QualifiedName(billing.invoice)), name: AuthoredName("a-scenario") }), "ESS-AUTHOR-003", "the same scenario is already declared in a.yaml")]
 right: [("b.yaml", Some(Authored { domain: DomainRef(QualifiedName(billing.invoice)), name: AuthoredName("a-scenario") }), "ESS-AUTHOR-003", "refusal[ESS-AUTHOR-003]: `billing.invoice/authored/a-scenario` in b.yaml\n  the same scenario is already declared in a.yaml\n  help: two files name one scenario in one domain; rename one of them")]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test d1_authored_inventory_preserves_full_original_typed_refusal_rendering ... FAILED

failures:

failures:
    d1_authored_inventory_preserves_full_original_typed_refusal_rendering

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p ess-conformance --test coverage_writer_adversary_pass1`
```

Actual exit 101; wall 0.108809 seconds.

### focused-go-diagnostic

```json
{
  "argv": [
    "cargo",
    "test",
    "--locked",
    "--offline",
    "-p",
    "ess-cli",
    "--test",
    "coverage_writer_adversary_pass1",
    "go_strict_diagnostic_does_not_label_known_suite5_inventory_as_legacy_unknown",
    "--",
    "--exact",
    "--nocapture"
  ],
  "cwd": "home-path:sha256:81a8aaad60388f74814838864c1b3182e25ef96fb5f2eac69a3110c015b31d04",
  "environment": {
    "PATH": "home-path:sha256:d972722bf5a8a5abf3ab363acc673b42b0d50eca552e59c1ad0efea4f8e3a23e",
    "RUSTC": "home-path:sha256:da7abda9718cfd0d0c1e74f56f34135d83b4a908741f60a13ad1ed963424c535",
    "RUSTDOC": "home-path:sha256:cfcca0fbc01edced245f4082ef3f4032907e9866e0d726bc4bd1861d26fbd53d",
    "TMPDIR": "home-path:sha256:a85023f45fdaccd57c62a4a1b51cee490171a57c6bb3db6c9cefabd69f61a130",
    "GOCACHE": "home-path:sha256:0c1bef61d39b03d4967ac4d3b1cdd105e585a69dc1e44ea973cffae93441bbc4",
    "GOMODCACHE": "home-path:sha256:c929e6078a0cc5289ec52e6bb58446312523dbff76bfa246689e269714ee6c63",
    "CARGO_HOME": "home-path:sha256:d30f9c4d89bf9b9f39d84a2047df7cbaa5c6a8acb7e20684ab0d33507967c60f",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:c7cdced08eb66cb30394fa5c6ddfe52ed26ffea4906e648caf1ea6e9f20537d9",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:3999c2b69a51b4629ae4706b632d381cb9365e6f6626fb8f7665289c783cf536",
    "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_DEBUG": "0",
    "CARGO_CACHE_RUSTC_INFO": "0",
    "CARGO_BUILD_JOBS": "2",
    "CARGO_NET_OFFLINE": "true"
  },
  "unset": [
    "CARGO_TARGET_DIR",
    "RUSTC_WRAPPER",
    "SCCACHE_SERVER_UDS"
  ],
  "free_before": 21868290048,
  "started_epoch": 1788725305.5754993,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass1.rs": "65eccc148c09d4d6b3fa88aa6ed8b210b266aee5775c0a65e05cde54293008ae",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass1.rs": "5815cdd2a6735b63565fa17dce9348f9f6382b94d468f64e60dd566a8fc453f9"
  },
  "exit": 101,
  "elapsed_seconds": 3.9364054680336267,
  "free_after": 21766455296
}
```

```text
   Compiling ess-cli v0.20.0 (home-path:sha256:5a10dd49c67cef5019797350b37afacc7816747d7c22f5c97b15b2438867815c)
    Finished `test` profile [unoptimized] target(s) in 0.36s
     Running tests/coverage_writer_adversary_pass1.rs (target/debug/deps/coverage_writer_adversary_pass1-96fa9eb4cce59dee)

running 1 test
=== RUN   TestStrictDiagnostic
    diagnostic_test.go:8: billing v3, 1 scenario(s), spec digest 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942
=== RUN   TestStrictDiagnostic/billing.invoice/authored/a-scenario
    runtime.go:2333: the target does not support this scenario: the target does not expose this
=== NAME  TestStrictDiagnostic
    diagnostic_test.go:8: strict conformance: inconclusive (legacy suite coverage is unknown)
--- FAIL: TestStrictDiagnostic (0.00s)
    --- SKIP: TestStrictDiagnostic/billing.invoice/authored/a-scenario (0.00s)
FAIL
FAIL	diagnostic/essconform	0.002s
FAIL


thread 'go_strict_diagnostic_does_not_label_known_suite5_inventory_as_legacy_unknown' (1121022) panicked at crates/edge/ess-cli/tests/coverage_writer_adversary_pass1.rs:64:5:
suite/5 diagnostic contradicts its admitted known inventory: === RUN   TestStrictDiagnostic
    diagnostic_test.go:8: billing v3, 1 scenario(s), spec digest 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942
=== RUN   TestStrictDiagnostic/billing.invoice/authored/a-scenario
    runtime.go:2333: the target does not support this scenario: the target does not expose this
=== NAME  TestStrictDiagnostic
    diagnostic_test.go:8: strict conformance: inconclusive (legacy suite coverage is unknown)
--- FAIL: TestStrictDiagnostic (0.00s)
    --- SKIP: TestStrictDiagnostic/billing.invoice/authored/a-scenario (0.00s)
FAIL
FAIL	diagnostic/essconform	0.002s
FAIL

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test go_strict_diagnostic_does_not_label_known_suite5_inventory_as_legacy_unknown ... FAILED

failures:

failures:
    go_strict_diagnostic_does_not_label_known_suite5_inventory_as_legacy_unknown

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 3.56s

error: test failed, to rerun pass `-p ess-cli --test coverage_writer_adversary_pass1`
```

Actual exit 101; wall 3.936405 seconds.

3. Appropriate full package suite and checks

The supplied pre-review baseline is the frozen implementor's final 642 passed, 0 failed/ignored, 60 runner summaries. The older 608-case implementation baseline is not used for this review. No baseline suite was rerun before the authored cases.

The actual full command below ran after all four focused cases. It executed 646 Rust cases: 643 passed, 3 failed, none ignored, across 62 runner summaries, exit 101 in 55.706447 seconds. The original 642 cases passed and the new browser control passed; only the three named new assertions failed. The actual generic-browser cases include both new paired admission and retained old-player execution. Vector loops and child Go processes are not added to the Rust count. The 95 lineage vectors and 314 closed-model vectors are exercised within the existing browser/Go tests, not counted as hundreds of extra tests here.

The permanent producer-export tests also ran within this package suite using a fresh exports-package-suite root. They are ordinary package-test results, not the root-owned final source-pinned AEP correspondence. Their cause-only local inventory expectation remains the D1 discrepancy; they must not be promoted to a successful D1 qualification. No mapper or AEP helper was executed.

### package-suite

```json
{
  "argv": [
    "cargo",
    "test",
    "--locked",
    "--offline",
    "-p",
    "ess-conformance",
    "-p",
    "ess-cli",
    "-p",
    "ess-diff",
    "--no-fail-fast"
  ],
  "cwd": "home-path:sha256:81a8aaad60388f74814838864c1b3182e25ef96fb5f2eac69a3110c015b31d04",
  "environment": {
    "PATH": "home-path:sha256:d972722bf5a8a5abf3ab363acc673b42b0d50eca552e59c1ad0efea4f8e3a23e",
    "RUSTC": "home-path:sha256:da7abda9718cfd0d0c1e74f56f34135d83b4a908741f60a13ad1ed963424c535",
    "RUSTDOC": "home-path:sha256:cfcca0fbc01edced245f4082ef3f4032907e9866e0d726bc4bd1861d26fbd53d",
    "TMPDIR": "home-path:sha256:a85023f45fdaccd57c62a4a1b51cee490171a57c6bb3db6c9cefabd69f61a130",
    "GOCACHE": "home-path:sha256:0c1bef61d39b03d4967ac4d3b1cdd105e585a69dc1e44ea973cffae93441bbc4",
    "GOMODCACHE": "home-path:sha256:c929e6078a0cc5289ec52e6bb58446312523dbff76bfa246689e269714ee6c63",
    "CARGO_HOME": "home-path:sha256:d30f9c4d89bf9b9f39d84a2047df7cbaa5c6a8acb7e20684ab0d33507967c60f",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:c7cdced08eb66cb30394fa5c6ddfe52ed26ffea4906e648caf1ea6e9f20537d9",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:397711ce843520ce570858f18c6e62ba637ee6a13c700f3516607b5f0d0de6a5",
    "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_DEBUG": "0",
    "CARGO_CACHE_RUSTC_INFO": "0",
    "CARGO_BUILD_JOBS": "2",
    "CARGO_NET_OFFLINE": "true"
  },
  "unset": [
    "CARGO_TARGET_DIR",
    "RUSTC_WRAPPER",
    "SCCACHE_SERVER_UDS"
  ],
  "free_before": 21004242944,
  "started_epoch": 1788725387.648095,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass1.rs": "dec5b723ccb77c6d732119e1820f92e1555e13b4b777a4d3d4cf5e4906a31736",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass1.rs": "950a4ac1cf1b5b8f1272396e5f4228718f7af26345b0cd34f234e3da587fc373"
  },
  "exit": 101,
  "elapsed_seconds": 55.706446619005874,
  "free_after": 20487991296
}
```

```text
   Compiling ess-diff v0.20.0 (home-path:sha256:5455152f24506d134276c0fa17104395c6cd66014ed2c8978ece31453da08d17)
   Compiling ess-cli v0.20.0 (home-path:sha256:5a10dd49c67cef5019797350b37afacc7816747d7c22f5c97b15b2438867815c)
   Compiling ess-conformance v0.20.0 (home-path:sha256:fb2af4db425baf24f833fe7f0be66e10d98c52642fdd023fe8c8d2f68266c344)
    Finished `test` profile [unoptimized] target(s) in 14.90s
     Running unittests src/main.rs (target/debug/deps/ess-63929f179d60f606)

running 12 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test coverage::tests::suite5_pair_refusal_precedes_target_construction ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/authored_scenarios.rs (target/debug/deps/authored_scenarios-3ba618a94d75005d)

running 25 tests
test author_nonmatching_only ... ok
test author_nested_only ... ok
test author_empty ... ok
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test run_empty ... ok
test ir_empty ... ok
test go_nested_only ... ok
test ir_nested_only ... ok
test go_nonmatching_only ... ok
test run_nonmatching_only ... ok
test ir_nonmatching_only ... ok
test web_empty ... ok
test web_nonmatching_only ... ok
test web_nested_only ... ok
test go_empty ... ok
test run_nested_only ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-fbc20393f78d5213)

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

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s

     Running tests/authored_site.rs (target/debug/deps/authored_site-1a420fe3beb09d76)

running 9 tests
test an_explicit_missing_front_page_is_an_error ... ok
test binary_downloads_are_not_silently_decoded ... ok
test asset_symlink_output_is_refused_before_any_page_changes ... ok
test a_page_identity_can_itself_end_in_html ... ok
test publication_without_strict_mode_preserves_unpublished_links ... ok
test strict_links_check_local_and_cross_page_fragments ... ok
test selected_sources_resolve_after_relocation_with_queries_fragments_and_downloads ... ok
test undeclared_existing_and_escaping_targets_are_refused_with_source_lines ... ok
test assets_cannot_collide_with_generated_output_or_escape_it ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-00fd33a384eed561)

running 1 test
test cli_composition_obeys_the_independently_authored_vectors ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running tests/binary64_count_adversary.rs (target/debug/deps/binary64_count_adversary-25ef58b4641736f8)

running 2 tests
test cli_binary64_model_and_original_suites_preserve_report_destinations_in_both_formats ... ok
test generated_go_binary64_shapes_refuse_before_factory_and_report_publication ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.08s

     Running tests/binary64_publication.rs (target/debug/deps/binary64_publication-ba03872d9e795d41)

running 1 test
test binary64_sparse_model_refuses_every_unsupported_publication_route ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-d76d1dd27f537ab9)

running 1 test
test binary64_publication_never_replaces_sources_or_partially_updates_a_library ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/command_surface.rs (target/debug/deps/command_surface-71a1dc5a6624aa01)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-604eebcbf9b30edb)

running 4 tests
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/count_reports.rs (target/debug/deps/count_reports-aeedc7adf842489b)

running 3 tests
test count_report_opt_in_has_a_distinct_detailed_surface_and_unknown_coverage ... ok
test count_cli_configuration_and_original_suite_refusals_preserve_destinations ... ok
test count_cli_preserves_default_bytes_and_standalone_detailed_pairing ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s

     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-1a7e7875d1ea7c87)

running 3 tests
test generated_go_skip_counts_preserve_opaque_ids_and_strictness_without_a_destination ... ok
test generated_go_abnormal_teardown_cannot_publish_a_completed_skip ... ok
test generated_go_rejects_closed_predicate_metadata_before_any_target ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.23s

     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-3af0e671176d1e0b)

running 2 tests
test generated_go_abnormal_unsupported_error_formatting_cannot_complete ... ok
test generated_go_admits_only_typed_predicate_paths_and_operator_envelopes ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.15s

     Running tests/coverage_browser.rs (target/debug/deps/coverage_browser-bf192040179e69f7)

running 4 tests
test retained_legacy_player_bytes_still_replay_in_actual_firefox ... ok
test actual_browser_admits_the_pair_before_creating_replay_state ... ok
test actual_browser_and_rust_refuse_every_closed_model_field_boundary ... ok
test actual_browser_checks_full_lineage_and_integer_metadata ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.48s

     Running tests/coverage_cli.rs (target/debug/deps/coverage_cli-994c933e9b49eced)

running 6 tests
test coverage_cli_refuses_binary64_model_before_each_new_production_surface ... ok
test coverage_cli_authored_roots_relocate_without_losing_exact_text_and_refuse_unrepresentable_paths ... ok
test explicit_suite5_cli_produces_exact_inventory_and_requires_report2_before_execution ... ok
test select_cli_preserves_all_parent_bytes_and_explicit_empty_selection ... ok
test impact_cli_requires_exact_complete_input_and_keeps_the_persisted_v3_shape ... ok
test generated_go_executes_the_admitted_coverage_inventory_and_preserves_pairing_defaults ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s

     Running tests/coverage_lineage.rs (target/debug/deps/coverage_lineage-a42eee64e165b553)

running 2 tests
test generated_go_checks_original_lineage_and_typed_defaults ... ok
test go_execution_adapts_only_selected_integer_fields_before_target_effects ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.39s

     Running tests/coverage_producers.rs (target/debug/deps/coverage_producers-1b45cfea40b4b957)

running 3 tests
test actual_rust_coverage_exports_match_the_independent_plan ... ok
test actual_coverage_producers_refuse_noninvoked_negative_clock_and_report1_without_output ... ok
test actual_go_coverage_exports_match_the_independent_plan ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.23s

     Running tests/coverage_writer_adversary_pass1.rs (target/debug/deps/coverage_writer_adversary_pass1-96fa9eb4cce59dee)

running 2 tests
test go_strict_diagnostic_does_not_label_known_suite5_inventory_as_legacy_unknown ... FAILED
test browser_refuses_a_command_name_with_a_final_line_feed_before_replay_state ... ok

failures:

---- go_strict_diagnostic_does_not_label_known_suite5_inventory_as_legacy_unknown stdout ----
=== RUN   TestStrictDiagnostic
    diagnostic_test.go:8: billing v3, 1 scenario(s), spec digest 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942
=== RUN   TestStrictDiagnostic/billing.invoice/authored/a-scenario
    runtime.go:2333: the target does not support this scenario: the target does not expose this
=== NAME  TestStrictDiagnostic
    diagnostic_test.go:8: strict conformance: inconclusive (legacy suite coverage is unknown)
--- FAIL: TestStrictDiagnostic (0.00s)
    --- SKIP: TestStrictDiagnostic/billing.invoice/authored/a-scenario (0.00s)
FAIL
FAIL	diagnostic/essconform	0.002s
FAIL


thread 'go_strict_diagnostic_does_not_label_known_suite5_inventory_as_legacy_unknown' (1147783) panicked at crates/edge/ess-cli/tests/coverage_writer_adversary_pass1.rs:89:5:
suite/5 diagnostic contradicts its admitted known inventory: === RUN   TestStrictDiagnostic
    diagnostic_test.go:8: billing v3, 1 scenario(s), spec digest 56090788443a14b4a51ad151eb5cb3ebded2b98f6defe9ac50826296ac5d0942
=== RUN   TestStrictDiagnostic/billing.invoice/authored/a-scenario
    runtime.go:2333: the target does not support this scenario: the target does not expose this
=== NAME  TestStrictDiagnostic
    diagnostic_test.go:8: strict conformance: inconclusive (legacy suite coverage is unknown)
--- FAIL: TestStrictDiagnostic (0.00s)
    --- SKIP: TestStrictDiagnostic/billing.invoice/authored/a-scenario (0.00s)
FAIL
FAIL	diagnostic/essconform	0.002s
FAIL

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    go_strict_diagnostic_does_not_label_known_suite5_inventory_as_legacy_unknown

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.59s

error: test failed, to rerun pass `-p ess-cli --test coverage_writer_adversary_pass1`
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-ac70ba28528e59d8)

running 4 tests
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_rust_cli_writes ... ok
test a_binding_function_capture_is_refused_before_web_cli_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-d5a347d9ae9103cd)

running 13 tests
test count_go_predicate_admission_matches_rust_leaf_grammar ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test count_report_skip_only_is_inconclusive_without_actual_failures ... ok
test count_retained_runtime_preserves_legacy_behavior_and_does_not_gain_version_checks ... ok
test count_go_empty_selection_is_inconclusive_and_clock_conversion_is_checked ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test count_go_actual_producers_keep_skip_error_and_teardown_categories ... ok
test count_go_refusals_precede_targets_and_incomplete_runs_never_publish ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.21s

     Running tests/model_types.rs (target/debug/deps/model_types-b0076ddf3b38585d)

running 3 tests
test all_type_binary64_libraries_publish_finite_codecs_with_atomic_preflight ... ok
test root_selection_and_output_refusals_preserve_existing_files ... ok
test each_target_retains_the_same_model_selection_and_distinct_input_provenance ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/normalization.rs (target/debug/deps/normalization-47a1f33d5a547d54)

running 17 tests
test unsupported_go_pattern_has_no_successful_partial_artifact ... ok
test incompatible_later_destination_preserves_the_entire_existing_output ... ok
test generated_paths_cannot_replace_canonical_source_inputs_or_follow_links ... ok
test output_symlinks_and_hardlinks_are_refused_without_mutation ... ok
test stale_bundle_missing_source_and_unknown_dispatch_refuse ... ok
test explicit_binary64_inputs_run_through_the_cli_without_weakening_version_one ... ok
test qualified_go_base64_pattern_is_published_without_decoding_the_value ... ok
test checked_recipe_and_run_are_deterministic_with_json_only_stdout ... ok
test output_cannot_replace_any_declared_input ... ok
test positional_cli_refuses_unused_branches_before_publication ... ok
test generation_checks_refuse_before_creating_or_changing_destinations ... ok
test failed_check_or_execution_preserves_output_and_does_not_emit_a_result ... ok
test raw_json_capture_runs_before_schema_and_keeps_failure_output_untouched ... ok
test generated_libraries_match_the_api_and_drift_check_never_repairs_files ... ok
test positional_cli_prepares_text_and_refuses_before_publication ... ok
test model_sources_are_compiled_pinned_and_protected_by_the_cli ... ok
test binary64_cli_keeps_numeric_identity_and_emits_checked_format_five ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.90s

     Running tests/normalization_positional_adversary.rs (target/debug/deps/normalization_positional_adversary-406279f2f9d35352)

running 2 tests
test cli_runtime_grammar_controls_preserve_existing_output ... ok
test cli_invalid_policy_blocks_all_publication_and_valid_metadata_is_drift_checked ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

     Running tests/normalization_typescript.rs (target/debug/deps/normalization_typescript-5542aa1874e06ad7)

running 7 tests
test unqualified_profile_refuses_with_source_pointer_and_no_partial_files ... ok
test model_inputs_are_recompiled_pinned_and_protected_before_typescript_generation ... ok
test typescript_cli_emits_an_accounted_executable_package ... ok
test retained_recipe_and_bundle_inputs_cannot_be_overwritten ... ok
test module_and_late_path_conflicts_refuse_before_any_publication ... ok
test generated_file_and_parent_links_refuse_without_touching_their_destinations ... ok
test every_planned_file_participates_in_read_only_drift_checks ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s

     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-1ef9acb3dfcb1eba)

running 1 test
test generation_and_check_need_no_native_tools_and_module_refusal_preserves_bytes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/openapi_accounting.rs (target/debug/deps/openapi_accounting-3024341d0a106a96)

running 6 tests
test complete_projection_preserves_supported_yaml_through_both_spellings ... ok
test both_import_spellings_write_the_accounted_envelope_for_every_presentation ... ok
test a_refused_import_keeps_existing_output_and_creates_no_new_file ... ok
test legacy_projection_refuses_before_destination_mutation ... ok
test unresolved_projection_reports_the_exact_reference_and_preserves_destinations ... ok
test partial_projection_reports_the_durable_gap_and_preserves_destinations ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/openapi_adversary_pass1.rs (target/debug/deps/openapi_adversary_pass1-d2bcefe19c39a1dd)

running 2 tests
test cli_rejects_duplicate_keys_and_tampered_accounting_before_output ... ok
test cli_rejects_unknown_unit_fields_before_any_projection_output ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/openapi_adversary_pass2.rs (target/debug/deps/openapi_adversary_pass2-ae258705b1349988)

running 1 test
test same_path_projection_refusal_preserves_the_unadmitted_input ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/output_containment.rs (target/debug/deps/output_containment-5c56b28ecc91698b)

running 19 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-8deef286ecfcabd1)

running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/schema_bundle.rs (target/debug/deps/schema_bundle-6f1b8cef6ccfdb14)

running 8 tests
test missing_dialect_and_incomplete_import_leave_existing_output_alone ... ok
test document_import_refusals_preserve_source_and_existing_output ... ok
test corrupted_import_cannot_be_projected_and_output_cannot_replace_source ... ok
test type_planning_and_output_refusals_leave_no_partial_library ... ok
test types_bundle_is_deterministic_and_never_replaces_its_input ... ok
test import_reload_projection_and_instance_validation_keep_original_data ... ok
test document_root_survives_reload_projection_and_each_type_target ... ok
test native_bundle_targets_require_identity_and_emit_build_metadata ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/schema_registry_identity.rs (target/debug/deps/schema_registry_identity-1d7aadfcc5659f8d)

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

     Running tests/schema_registry_identity_adversary.rs (target/debug/deps/schema_registry_identity_adversary-cf3f1a6623400628)

running 6 tests
test registry_admission_and_selected_typescript_projection_have_distinct_boundaries ... ok
test inlining_idless_generated_payloads_breaks_their_original_root_references ... ok
test decoded_duplicate_ids_are_collisions_and_filenames_supply_no_identity ... ok
test an_unselected_invalid_resource_blocks_both_documented_pairs ... ok
test envelope_definitions_cannot_shadow_either_payload_resource_root ... ok
test the_selected_schema_checks_its_selector_and_nonobject_payload_as_one_instance ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

     Running tests/target_failure.rs (target/debug/deps/target_failure-7d9cc8e18982b595)

running 1 test
test fatal_synthesis_preserves_destinations_and_has_a_typed_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

     Running unittests src/lib.rs (target/debug/deps/ess_conformance-5e50d807cc4deaf5)

running 70 tests
test counts::tests::exact_unsigned_scalar_vectors_do_not_use_binary64 ... ok
test counts::tests::payload_number_and_utf8_canonical_profile_is_frozen_separately_from_scalars ... ok
test decision::tests::exactly_one_reason_says_another_candidate_would_help ... ok
test evidence::tests::a_standalone_report_carries_every_field_an_adapter_needs ... ok
test decision::tests::a_refusal_renders_the_predicate_the_command_and_every_reason ... ok
test decision::tests::a_decision_reads_its_two_other_cases_as_neither_satisfied_nor_the_other ... ok
test evidence::tests::report_readers_do_not_guess_a_producer_from_status_vocabulary ... ok
test evidence::tests::report_readers_refuse_more_nonpasses_than_executed_scenarios ... ok
test evidence::tests::report_readers_preserve_go_producer_bytes_and_historical_nonpass_counts ... ok
test faulty::tests::a_fault_is_injected_into_the_system_that_declares_what_it_breaks ... ok
test faulty::tests::every_fault_says_what_it_is_and_where_it_goes ... ok
test faulty::tests::no_two_faults_claim_the_same_scenario ... ok
test evidence::tests::unknown_report_fields_are_refused ... ok
test evidence::tests::the_closed_report_round_trips_with_identical_canonical_bytes ... ok
test faulty::tests::only_the_two_faults_the_boundary_cannot_express_are_injected_in_the_implementation ... ok
test input::tests::a_primitive_refuses_a_node_of_the_wrong_shape_rather_than_coercing_it ... ok
test input::tests::every_primitive_projects_to_the_one_fact_value_that_can_hold_it ... ok
test input::tests::shape_errors_render_one_per_line_and_name_the_input_root_by_name ... ok
test go::tests::every_go_file_is_in_the_package_the_readme_names ... ok
test report::tests::a_scenario_status_is_the_strongest_of_its_checks_and_a_contradiction_outranks_everything ... ok
test report::tests::a_quoted_input_reads_as_the_call_that_was_made ... ok
test report::tests::a_diagnostic_answers_all_five_of_the_questions_a_failure_has_to_answer ... ok
test report::tests::an_unsupported_scenario_makes_the_run_fail_rather_than_look_like_a_pass ... ok
test report::tests::every_check_code_has_a_distinct_name_and_a_rule_sentence ... ok
test go::tests::the_runner_is_a_constant_and_only_the_suite_moves ... ok
test evidence::tests::report_readers_refuse_nonpass_count_and_list_disagreement ... ok
test runner::tests::a_count_is_the_half_of_an_ordering_claim_that_says_the_rows_were_there ... ok
test runner::tests::a_count_with_neither_bound_is_a_suite_defect_and_not_a_satisfied_assertion ... ok
test runner::tests::a_declared_order_is_checked_on_adjacent_rows_and_the_next_key_breaks_a_tie ... ok
test runner::tests::a_nested_row_binds_the_paths_a_predicate_spells ... ok
test runner::tests::a_position_in_a_view_that_declares_no_order_is_a_suite_defect ... ok
test runner::tests::a_position_names_both_ends_and_a_row_that_is_not_there_is_not_a_match ... ok
test runner::tests::a_ranking_key_a_row_does_not_publish_is_undecidable_rather_than_out_of_order ... ok
test runner::tests::a_predicate_a_row_cannot_answer_is_reported_rather_than_retried ... ok
test runner::tests::an_empty_field_set_means_a_row_exists_and_not_that_anything_will_do ... ok
test runner::tests::a_view_that_holds_nothing_does_not_satisfy_an_invariant_by_being_empty ... ok
test runner::tests::an_order_over_fewer_than_two_rows_holds_and_does_not_double_as_a_non_emptiness_claim ... ok
test runner::tests::the_runners_clock_advances_on_every_read_so_a_deadline_can_bound_anything ... ok
test runner::tests::ids_come_from_the_suite_and_from_nothing_ambient ... ok
test scenario::tests::a_declared_leaf_admits_what_its_type_admits_and_absence_only_where_the_type_permits_it ... ok
test scenario::tests::a_purpose_is_one_line_and_says_something ... ok
test scenario::tests::a_scenario_id_names_the_construct_it_exercises_rather_than_its_position ... ok
test scenario::tests::a_payload_shape_round_trips_through_the_form_a_suite_is_stored_in ... ok
test scenario::tests::a_suite_format_from_a_later_build_is_refused_rather_than_guessed ... ok
test scenario::tests::a_scenario_id_that_names_no_construct_is_refused ... ok
test scenario::tests::a_semantic_reference_renders_the_way_the_design_writes_one ... ok
test scenario::tests::a_transition_ref_refuses_a_name_no_lifecycle_can_declare ... ok
test scenario::tests::a_suite_refuses_a_second_scenario_under_one_id ... ok
test scenario::tests::an_invariant_scenario_is_keyed_by_the_entity_and_the_branch_and_never_by_a_position ... ok
test scenario::tests::every_binding_aspect_is_in_the_list_that_is_walked_to_produce_them ... ok
test scenario::tests::two_scenarios_about_the_same_thing_in_the_same_way_are_one_id ... ok
test scenario::tests::the_ids_of_a_suite_sort_the_way_a_reader_sorts_the_file ... ok
test scenario::tests::every_scenario_id_reads_back_from_the_form_a_report_prints ... ok
test synthesize::tests::a_refusal_names_the_construct_the_code_and_the_repair ... ok
test evidence::tests::report_readers_refuse_unknown_report_formats ... ok
test witness::tests::a_text_witness_is_its_own_path_so_two_fields_of_one_type_never_agree ... ok
test synthesize::tests::every_refusal_carries_a_distinct_code_in_one_family ... ok
test web::tests::no_comparison_sits_in_a_text_node_mustache ... ok
test witness::tests::an_enum_offers_every_variant_it_declares_and_the_first_one_only_once ... ok
test witness::tests::an_integer_leaf_is_never_offered_a_fractional_candidate ... ok
test witness::tests::the_alternatives_for_a_number_are_the_guards_own_literals_either_side ... ok
test web::tests::the_page_calls_nothing_the_player_does_not_return ... ok
test witness::tests::the_candidate_count_is_bounded_however_many_fields_a_guard_reads ... ok
test witness::tests::two_uuid_witnesses_differ_and_neither_moves_when_a_third_field_appears ... ok
test evidence::tests::report_readers_refuse_nonpass_entries_without_a_known_nonpass_status ... ok
test evidence::tests::report_readers_refuse_malformed_and_unsupported_suite_versions ... ok
test web::tests::the_page_is_specification_neutral ... ok
test evidence::tests::report_readers_refuse_status_claims_that_contradict_the_list ... ok
test evidence::tests::report_readers_preserve_rust_producer_bytes_for_every_supported_suite ... ok
test coverage_build::tests::actual_duplicate_insertion_keeps_the_generated_survivor_and_refusal_through_execution ... ok

test result: ok. 70 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-076db2c2086144b9)

running 4 tests
test authored_semantic_depth_is_distinct_from_the_projection_limit ... ok
test malformed_bound_operands_refuse_before_the_collection_projection_gap ... ok
test authored_optional_enum_membership_rejects_later_invalid_values ... ok
test authored_rows_preserve_scalar_controls_and_reject_aggregate_and_unprojected_reads ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/authored.rs (target/debug/deps/authored-fb795c5277d56862)

running 53 tests
test a_halt_of_a_listing_the_model_calls_eventual_retries_because_the_model_said_so ... ok
test a_name_a_closed_set_does_not_have_is_refused_with_the_set ... ok
test a_halt_claimed_of_a_listing_with_no_declared_order_is_refused_by_the_code_that_already_says_so ... ok
test a_bounded_negative_that_forbids_no_event_is_refused ... ok
test a_field_the_surface_does_not_declare_is_refused_by_name ... ok
test a_command_the_model_does_not_declare_is_refused_by_name ... ok
test a_document_that_is_not_one_is_refused_rather_than_read_as_an_empty_scenario ... ok
test a_declared_field_nothing_supplies_is_refused_by_name ... ok
test a_halt_after_no_rows_at_all_is_refused_rather_than_compiled ... ok
test a_halt_compiles_to_a_step_of_its_own_and_not_to_a_claim_about_rows ... ok
test a_halt_stated_beside_another_claim_is_two_assertions_filed_as_one ... ok
test a_format_this_build_does_not_implement_is_refused_before_anything_is_read ... ok
test a_positional_claim_takes_the_order_from_the_view_rather_than_from_the_author ... ok
test a_claim_the_timelines_own_instants_contradict_is_refused ... ok
test a_reference_where_the_suite_compares_a_value_it_carries_is_refused ... ok
test a_domain_the_model_does_not_declare_is_refused_by_name ... ok
test a_position_in_a_view_that_declares_no_order_is_refused ... ok
test a_scenario_that_runs_nothing_is_refused_rather_than_counted_as_a_check ... ok
test a_state_the_lifecycle_does_not_declare_is_refused_as_a_state_and_not_as_a_variant ... ok
test a_predicate_reading_something_the_view_does_not_publish_is_refused ... ok
test a_scenario_compiles_to_the_id_the_domain_and_the_name_make ... ok
test a_timeline_whose_instants_do_not_ascend_is_refused ... ok
test a_value_read_off_an_event_nothing_required_is_refused ... ok
test a_value_the_declared_type_does_not_admit_is_refused_where_it_sits ... ok
test an_act_cannot_open_a_window_at_its_own_instant ... ok
test an_assertion_that_states_other_than_one_claim_is_refused ... ok
test a_view_the_model_does_not_declare_is_refused_by_name ... ok
test a_window_of_no_seconds_is_refused_rather_than_compiled_into_a_check_that_cannot_fail ... ok
test an_actor_the_model_does_not_declare_is_refused_by_name ... ok
test a_window_measured_from_an_instant_nothing_marked_is_refused_with_the_ones_that_are ... ok
test a_window_that_states_other_than_one_bound_is_refused ... ok
test an_elapsed_claim_compiles_to_the_four_steps_that_carry_it_and_they_come_before_the_act ... ok
test an_actor_the_specification_does_not_grant_the_command_is_refused ... ok
test an_entity_the_model_does_not_declare_is_refused_by_name ... ok
test an_instance_bound_to_a_field_that_cannot_hold_an_identity_is_refused ... ok
test an_instance_named_before_anything_binds_it_is_refused ... ok
test an_instance_the_arrangement_does_not_declare_is_refused_by_name ... ok
test one_name_for_two_instants_is_refused_rather_than_read_as_the_later_one ... ok
test the_order_the_files_are_handed_over_in_does_not_reach_the_result ... ok
test the_steps_are_the_vocabulary_a_generated_scenario_already_uses ... ok
test an_outcome_the_command_does_not_declare_is_refused_with_the_ones_it_does ... ok
test an_error_the_model_does_not_declare_is_refused_by_name ... ok
test authored_aggregate_presence_keeps_026_and_valid_scalar_reads_keep_the_predicate ... ok
test authored_predicate_operand_errors_have_their_own_refusal ... ok
test an_event_the_model_does_not_declare_is_refused_by_name ... ok
test the_committed_billing_suite_holds_the_authored_scenario_beside_the_generated_ones ... ok
test two_files_naming_one_scenario_are_refused_rather_than_one_displacing_the_other ... ok
test every_cause_is_reachable_from_a_document ... ok
test two_compilations_of_one_file_produce_identical_bytes ... ok
test coverage_builder_retains_authored_duplicate_ownership_and_original_source_bytes ... ok
test independently_successful_authored_batches_are_refused_only_at_final_merge ... ok
test paired_browser_emission_retains_original_input_and_checks_the_actual_model ... ok
test rejected_authored_candidate_needs_are_proved_independently_of_an_outside_survivor ... ok

test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/binary64_count_adversary.rs (target/debug/deps/binary64_count_adversary-f88a68c753c9cda9)

running 3 tests
test both_fallible_runners_preserve_all_binary64_issues_before_effects ... ok
test unified_model_issues_survive_authored_synthesis_and_web_boundaries ... ok
test original_byte_and_serde_admission_refuse_binary64_across_all_legacy_suite_majors ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/count_reports.rs (target/debug/deps/count_reports-149a201f4b530ab1)

running 7 tests
test legacy_dto_is_not_original_byte_admission_and_typed_execution_still_checks_versions ... ok
test known_complete_coverage_qualifies_actual_nonempty_passes ... ok
test suite_admission_closes_structural_variants_before_target_identity ... ok
test detailed_admission_checks_fields_outcome_order_and_checked_time ... ok
test exact_bytes_profiles_partition_and_identity_cannot_be_guessed ... ok
test new_scalar_tokens_are_exact_unsigned_in_both_surfaces ... ok
test actual_rust_producer_pairs_preserve_categories_precedence_empty_and_high_u64 ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-5e296baa7e67033a)

running 1 test
test a_completed_run_cannot_be_rebound_to_different_admitted_original_bytes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-971ce9c755546c0c)

running 1 test
test cloned_execution_binding_survives_mutation_of_extracted_legacy_diagnostics ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/coverage_admission.rs (target/debug/deps/coverage_admission-4eb9e754e0e399d6)

running 7 tests
test input_carrier_is_structural_and_cannot_mint_admission ... ok
test complete_inventory_is_admitted_from_its_original_bytes ... ok
test all_input_is_admitted_only_without_unused_parents ... ok
test coverage_source_identity_is_checked_without_normalizing_it ... ok
test coverage_integer_tokens_and_closed_fields_are_checked_before_serde ... ok
test inventory_corruption_is_refused_at_admission ... ok
test explicit_selection_retains_original_parents_and_refuses_direct_admission ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/coverage_writer_adversary_pass1.rs (target/debug/deps/coverage_writer_adversary_pass1-257bfdcc793f80b8)

running 2 tests
test d1_authored_inventory_preserves_full_original_typed_refusal_rendering ... FAILED
test d1_generated_inventory_preserves_full_original_typed_refusal_rendering ... FAILED

failures:

---- d1_authored_inventory_preserves_full_original_typed_refusal_rendering stdout ----

thread 'd1_authored_inventory_preserves_full_original_typed_refusal_rendering' (1154847) panicked at crates/verify/ess-conformance/tests/coverage_writer_adversary_pass1.rs:154:9:
assertion `left == right` failed: D1 accepted-duplicate: selected full Refusal::to_string, with root-relative source and help
  left: [("b.yaml", Some(Authored { domain: DomainRef(QualifiedName(billing.invoice)), name: AuthoredName("a-scenario") }), "ESS-AUTHOR-003", "the same scenario is already declared in a.yaml")]
 right: [("b.yaml", Some(Authored { domain: DomainRef(QualifiedName(billing.invoice)), name: AuthoredName("a-scenario") }), "ESS-AUTHOR-003", "refusal[ESS-AUTHOR-003]: `billing.invoice/authored/a-scenario` in b.yaml\n  the same scenario is already declared in a.yaml\n  help: two files name one scenario in one domain; rename one of them")]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- d1_generated_inventory_preserves_full_original_typed_refusal_rendering stdout ----

thread 'd1_generated_inventory_preserves_full_original_typed_refusal_rendering' (1154848) panicked at crates/verify/ess-conformance/tests/coverage_writer_adversary_pass1.rs:82:5:
assertion `left == right` failed: D1 selected full Refusal::to_string, including original prefix and help
  left: [(Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CancelInvoice)), outcome: OutcomeName(cancelled) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CancelInvoice)), outcome: OutcomeName(cancelled) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CreateInvoice)), outcome: OutcomeName(accepted) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CreateInvoice)), outcome: OutcomeName(accepted) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.IssueInvoice)), outcome: OutcomeName(issued) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.IssueInvoice)), outcome: OutcomeName(issued) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.PayInvoice)), outcome: OutcomeName(settled) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.PayInvoice)), outcome: OutcomeName(settled) } }), "ESS-SYNTH-011", "`reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n  - `reminder_count` is published by no view of the entity")]
 right: [(Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CancelInvoice)), outcome: OutcomeName(cancelled) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.CancelInvoice/cancelled`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CancelInvoice)), outcome: OutcomeName(cancelled) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.CancelInvoice/cancelled`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CreateInvoice)), outcome: OutcomeName(accepted) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.CreateInvoice/accepted`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.CreateInvoice)), outcome: OutcomeName(accepted) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.CreateInvoice/accepted`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.IssueInvoice)), outcome: OutcomeName(issued) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.IssueInvoice/issued`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.IssueInvoice)), outcome: OutcomeName(issued) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.IssueInvoice/issued`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.PayInvoice)), outcome: OutcomeName(settled) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.PayInvoice/settled`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes"), (Some(Invariant { entity: EntityRef(QualifiedName(billing.invoice.Invoice)), after: OutcomeRef { command: CommandRef(QualifiedName(billing.invoice.PayInvoice)), outcome: OutcomeName(settled) } }), "ESS-SYNTH-011", "refusal[ESS-SYNTH-011]: entity billing.invoice.Invoice has no scenario `billing.invoice.Invoice/invariant/after/billing.invoice.PayInvoice/settled`\n  `reminder_count >= 0` reads what no view of `billing.invoice.Invoice` publishes\n    - `reminder_count` is published by no view of the entity\n  help: publish the fields the invariant reads in a view of this entity, or state the invariant over what one already publishes")]


failures:
    d1_authored_inventory_preserves_full_original_typed_refusal_rendering
    d1_generated_inventory_preserves_full_original_typed_refusal_rendering

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

error: test failed, to rerun pass `-p ess-conformance --test coverage_writer_adversary_pass1`
     Running tests/elapsed.rs (target/debug/deps/elapsed-b5312b79e8e5d6a8)

running 7 tests
test a_window_opened_at_an_instant_nothing_marked_is_a_suite_defect_and_not_a_failed_implementation ... ok
test a_deadline_the_target_ran_past_fails_the_within_claim ... ok
test a_target_that_holds_the_window_and_reports_it_passes ... ok
test a_target_with_no_clock_reports_unsupported_and_the_run_fails ... ok
test a_target_whose_clock_never_moves_fails_rather_than_being_read_as_having_waited ... ok
test an_event_published_inside_the_window_fails_the_bounded_negative_and_nothing_else ... ok
test two_runs_over_one_window_produce_byte_identical_reports ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/execution.rs (target/debug/deps/execution-ae712a3af979f44f)

running 12 tests
test an_eventual_assertion_asks_again_within_a_deadline_and_never_sleeps ... ok
test an_eventual_view_is_read_again_and_a_read_your_writes_view_is_not ... ok
test a_scenario_whose_input_no_longer_reaches_its_branch_fails_with_a_diagnostic_naming_the_defect ... ok
test a_target_that_cannot_expose_an_observation_fails_the_run_rather_than_skipping_it ... ok
test a_view_assertion_names_the_instance_the_scenario_created_rather_than_any_row ... ok
test a_value_of_the_wrong_declared_type_is_caught_by_the_same_check_as_a_missing_one ... ok
test a_view_answered_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test every_scenario_the_billing_specification_obliges_passes_against_the_reference_implementation ... ok
test an_event_missing_a_field_it_declares_is_named_leaf_by_leaf_rather_than_reported_as_absent ... ok
test every_scenario_checked_something_and_no_family_of_them_was_silently_empty ... ok
test a_read_your_writes_view_is_not_quietly_read_at_current_when_no_token_came_back ... ok
test two_runs_of_one_suite_against_one_target_produce_byte_identical_reports ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running tests/faults.rs (target/debug/deps/faults-592913acd22499fb)

running 11 tests
test a_fault_nothing_catches_is_recorded_rather_than_quietly_dropped ... ok
test the_diagnostic_of_a_caught_fault_names_the_defect_rather_than_reporting_that_something_broke ... ok
test every_fault_that_could_be_a_boundary_perturbation_is_one ... ok
test a_wrong_mapping_is_invisible_to_a_target_that_cannot_show_its_invocations ... ok
test dropping_one_binding_leaves_the_other_two_green ... ok
test the_widest_blast_radius_is_scenarios_that_could_not_be_arranged_rather_than_extra_verdicts ... ok
test each_specification_is_passed_in_full_by_the_implementation_written_from_it ... ok
test two_runs_against_one_faulty_target_produce_byte_identical_reports ... ok
test each_fault_fails_the_scenario_that_exists_to_catch_it ... ok
test a_faults_blast_radius_is_accounted_for ... ok
test a_fault_does_not_simply_break_everything ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s

     Running tests/halt.rs (target/debug/deps/halt-e9156dff15c9e526)

running 7 tests
test a_halt_of_an_eventual_listing_is_asked_again_while_the_projection_catches_up ... ok
test a_target_that_cannot_read_a_row_at_a_time_reports_unsupported_and_the_run_fails ... ok
test retrying_does_not_rescue_a_producer_that_never_stops ... ok
test a_listing_that_ran_out_before_the_reader_stopped_it_is_not_a_halt ... ok
test a_target_whose_producer_stops_when_the_reader_does_passes ... ok
test a_target_that_reads_the_whole_listing_fails_rather_than_being_read_as_having_stopped ... ok
test two_runs_over_one_halt_claim_produce_byte_identical_reports ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/report_reader_adversary.rs (target/debug/deps/report_reader_adversary-c5e2e8b3df383188)

running 4 tests
test duplicate_claims_cannot_hide_behind_a_valid_last_value ... ok
test count_extremes_refuse_contradictions_without_inventing_coverage ... ok
test closed_wire_fields_preserve_their_formats_scalar_contracts ... ok
test aggregate_status_does_not_depend_on_nonpass_order_or_multiplicity ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

     Running tests/suite.rs (target/debug/deps/suite-b620394c52c8390b)

running 16 tests
test a_suite_naming_something_that_is_not_an_ess_name_is_refused_while_it_is_read ... ok
test the_scan_for_a_clock_finds_one_and_does_not_find_a_word_that_merely_ends_in_a_banned_token ... ok
test a_suite_parses_from_text_alone_without_an_ir ... ok
test the_steps_a_binding_and_an_invariant_need_survive_being_read_back_from_text ... ok
test a_count_and_a_position_read_back_as_what_a_runner_in_another_language_must_read ... ok
test sparse_models_cannot_publish_an_empty_success_for_binary64 ... ok
test every_scenario_id_the_billing_model_can_produce_reads_back ... ok
test the_step_vocabulary_expresses_the_worked_example_from_section_ten ... ok
test a_suite_serialised_in_one_process_resolves_in_another ... ok
test the_scenario_ids_appear_in_the_file_in_the_order_a_sorted_key_list_would_be ... ok
test the_dependency_set_names_a_type_no_derived_from_would_have_mentioned ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test serialising_a_suite_twice_produces_byte_identical_json ... ok
test inserting_one_outcome_re_keys_nothing_around_it ... ok
test the_suite_records_the_same_model_digest_the_projections_do ... ok
test typed_binary64_suites_refuse_serialization_emission_and_target_effects ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/synthesis.rs (target/debug/deps/synthesis-467793b7b98b58f7)

running 53 tests
test a_guard_no_candidate_can_satisfy_is_refused_with_the_number_tried ... ok
test a_binding_whose_branch_the_event_decides_refuses_the_flow_and_still_checks_the_mapping ... ok
test a_command_that_declares_no_wrong_state_answer_is_refused_by_name_beside_its_scenario ... ok
test a_command_that_accepts_a_wrong_state_is_asserted_as_accepting_rather_than_refusing ... ok
test a_filter_reading_something_no_scenario_knows_refuses_rather_than_guessing ... ok
test a_parameterised_view_is_queried_with_the_value_the_scenario_put_in_the_row ... ok
test a_state_reached_only_through_a_branch_no_input_reaches_is_refused_rather_than_arranged ... ok
test a_component_nothing_declares_is_refused_by_name ... ok
test a_binding_that_drops_its_failures_refuses_that_check_and_names_the_reason ... ok
test a_binding_that_retries_forces_one_failure_and_still_requires_the_consequence ... ok
test a_binding_mapping_names_the_source_the_document_wrote_and_not_its_same_typed_sibling ... ok
test a_read_your_writes_view_filled_by_the_command_that_ran_is_asserted_to_hold_a_row ... ok
test an_entity_nothing_creates_cannot_be_acted_on_and_says_so ... ok
test a_value_object_nothing_observable_holds_keeps_a_refusal_naming_what_would_close_it ... ok
test a_binding_flow_is_proved_through_the_event_the_invoked_command_publishes ... ok
test a_declared_order_is_asserted_against_two_rows_the_scenario_arranged_itself ... ok
test a_scenario_that_moves_an_instance_names_the_one_an_earlier_step_created ... ok
test a_move_is_observed_through_the_view_the_state_it_left_is_filtered_on ... ok
test a_declared_error_is_asserted_by_name_and_never_by_an_invented_payload ... ok
test an_undecidable_guard_refuses_and_does_not_spend_the_candidate_budget ... ok
test an_order_the_specification_cannot_put_two_rows_under_is_refused_and_not_asserted ... ok
test a_value_objects_own_invariants_are_read_at_every_field_position_a_view_holds_one ... ok
test a_synthesised_count_is_a_floor_the_scenario_arranged_and_never_a_ceiling ... ok
test a_move_that_is_illegal_in_a_state_is_attempted_with_the_input_that_would_have_worked ... ok
test a_whole_system_suite_does_not_mention_a_component ... ok
test an_actor_is_named_only_where_the_specification_grants_the_command ... ok
test a_view_the_entity_has_not_reached_yet_is_asserted_to_exclude_the_instance_by_name ... ok
test an_outcome_that_updates_an_instance_acts_on_one_the_scenario_created ... ok
test a_suite_for_one_component_holds_only_what_that_component_realises ... ok
test an_invariant_over_a_field_no_view_publishes_refuses_rather_than_being_dropped ... ok
test an_at_least_once_binding_delivers_the_event_twice_and_requires_no_count ... ok
test a_binding_that_escalates_requires_the_event_the_escalation_declares ... ok
test an_event_assertion_carries_the_declared_shape_and_exactly_the_values_the_payload_determines ... ok
test a_view_is_asserted_in_the_block_its_own_consistency_decides ... ok
test a_view_that_does_not_hold_the_instance_yet_is_not_asked_about_its_invariants ... ok
test an_illegal_move_requires_the_branch_and_the_declared_error_rather_than_merely_failing ... ok
test a_synthesised_suite_survives_being_written_and_read_back ... ok
test an_invariant_is_asserted_against_every_view_that_publishes_what_it_reads ... ok
test every_command_names_an_instance_an_earlier_step_of_the_same_scenario_bound ... ok
test an_outcome_no_input_decides_is_reached_by_injection_and_by_nothing_else ... ok
test the_failure_control_is_armed_after_the_arrangement_and_before_the_command_that_triggers_it ... ok
test the_refusal_branch_asserts_that_no_event_the_specification_declares_occurred ... ok
test every_declared_transition_has_a_scenario_that_proves_it_can_occur ... ok
test the_dependency_set_names_the_types_the_scenario_is_made_of ... ok
test every_declared_outcome_is_either_a_scenario_or_a_named_refusal_or_asserted_by_the_state_family ... ok
test the_input_a_scenario_sends_is_re_decided_against_the_guard_it_claims_to_reach ... ok
test each_example_synthesises_the_families_its_specification_declares ... ok
test every_clause_of_every_binding_is_either_a_scenario_or_a_named_refusal ... ok
test synthesising_the_same_specification_twice_produces_byte_identical_output ... ok
test coverage_builder_records_the_complete_generated_inventory_and_component_omissions ... ok
test coverage_all_missing_invariants_keep_null_survivors_and_component_proofs_stay_conservative ... ok
test canonical_expression_compatibility_fixtures ... ok
test coverage_missing_lifecycle_and_view_checks_remain_beside_actual_passing_results ... ok

test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s

     Running tests/witness.rs (target/debug/deps/witness-9e1a2f8f8747eefd)

running 28 tests
test a_candidate_carrying_a_field_no_type_declares_is_refused ... ok
test a_list_a_map_and_a_union_bind_no_fact_in_the_current_projection ... ok
test a_candidate_input_projects_one_fact_per_scalar_leaf ... ok
test a_candidate_missing_a_required_field_is_refused_before_any_guard_is_read ... ok
test a_newtype_is_transparent_so_a_deep_path_reaches_through_it_without_a_segment ... ok
test a_conjunction_of_two_undecidable_leaves_reports_both ... ok
test a_disjunction_one_of_whose_branches_holds_is_satisfied_despite_an_undecidable_branch ... ok
test a_path_into_a_list_or_a_union_names_the_aggregate_rather_than_the_missing_element ... ok
test a_refusal_names_the_predicate_the_command_and_the_path ... ok
test a_refuted_guard_carries_the_leaf_and_the_value_that_refuted_it ... ok
test a_scalar_of_the_wrong_shape_is_refused_rather_than_coerced ... ok
test a_newtype_is_transparent_when_a_path_is_resolved_as_well_as_when_it_is_projected ... ok
test a_path_landing_on_an_aggregate_is_unevaluable_by_construction ... ok
test an_absent_optional_binds_nothing_rather_than_binding_a_default ... ok
test an_absent_optional_is_unevaluable_but_says_a_candidate_could_repair_it ... ok
test equality_over_two_texts_is_decided_even_though_ordering_them_is_not ... ok
test a_long_recursive_read_validates_beyond_the_projection_limit ... ok
test legal_collection_cardinality_is_not_currently_projected ... ok
test expression_search_limits_do_not_define_type_correctness ... ok
test only_an_absent_value_says_another_candidate_would_help ... ok
test ordering_across_two_types_is_unevaluable_not_false ... ok
test otherwise_and_external_are_not_guards_over_the_input ... ok
test resolved_adapter_keeps_semantics_separate_from_collection_projection ... ok
test malformed_declarations_refuse_early_and_direct_bad_reads_remain_unknown ... ok
test ordering_two_texts_is_unevaluable_because_an_ess_specification_declares_no_scale ... ok
test the_normative_shape_of_guard_is_decidable_for_both_signs ... ok
test the_same_text_ordering_is_decidable_once_a_scale_contains_both_values ... ok
test unclassified_is_a_drift_alarm_and_no_enumerated_source_trips_it ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/lib.rs (target/debug/deps/ess_diff-06167d7f60ffb115)

running 9 tests
test change::tests::a_change_id_names_its_category_subject_subtype_and_member_in_that_order ... ok
test change::tests::a_change_with_no_member_renders_three_parts_rather_than_a_trailing_slash ... ok
test change::tests::the_canonical_order_is_the_category_order_and_not_the_alphabet ... ok
test change::tests::only_a_grant_and_a_variant_decide_a_direction ... ok
test delta::tests::a_delta_puts_its_changes_in_canonical_order_however_they_arrive ... ok
test impact::tests::a_whole_answer_absorbs_a_narrowing_whichever_way_round_they_are_joined ... ok
test impact::tests::an_unfollowed_file_is_not_an_artifact_that_owes_regeneration ... ok
test impact::tests::a_change_to_the_specification_itself_owes_the_whole_suite ... ok
test impact::tests::a_suite_resting_on_a_construct_the_graph_has_no_node_for_owes_the_whole_suite ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/artifacts.rs (target/debug/deps/artifacts-fe289ed3b5544f1f)

running 13 tests
test an_owed_artifacts_path_explains_the_membership_hop_by_hop ... ok
test an_artifact_whose_slice_nothing_reached_is_absent_from_the_answer ... ok
test the_six_change_delta_owes_a_strict_subset_of_the_artifacts ... ok
test the_artifacts_the_currency_changes_reach_are_owed_and_named ... ok
test a_change_to_the_system_header_owes_every_artifact ... ok
test the_two_predicate_edits_narrow_the_artifacts_differently_and_both_subsets_are_named ... ok
test a_grant_change_owes_the_documents_that_read_grants_and_not_the_ones_that_do_not ... ok
test whole_model_artifacts_are_owed_by_any_change_at_all ... ok
test the_artifact_answer_is_byte_identical_between_runs ... ok
test a_committed_tree_is_answered_for_fail_closed_file_by_file ... ok
test a_committed_artifact_with_a_false_contract_digest_is_owed_as_a_false_claim ... ok
test review_whole_model_hashes_and_index_bytes_remain_frozen ... ok
test review_legacy_slice_stamps_are_owed_even_when_raw_hashes_match ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

     Running tests/canonical.rs (target/debug/deps/canonical-ce09a7a45878a366)

running 20 tests
test a_binding_still_has_one_delivery_a_document_can_write ... ok
test a_change_is_spelt_the_same_way_in_its_id_and_in_the_document ... ok
test every_change_variant_has_something_to_say_for_itself ... ok
test review_freeze_legacy_delta_bytes ... ok
test review_version_admission_refuses_new_vocabulary_in_legacy_envelopes ... ok
test no_source_file_in_the_diff_engine_reads_a_clock_or_an_unordered_map ... ok
test no_source_file_in_the_diff_engine_calls_an_ir_handle_accessor ... ok
test a_system_still_has_no_naming_a_document_can_set ... ok
test a_delta_whose_changes_are_out_of_order_is_refused ... ok
test a_document_with_six_defects_reports_six ... ok
test every_change_in_a_delta_has_its_own_id ... ok
test canonical_json_ends_in_a_newline ... ok
test a_delta_naming_two_systems_is_refused_on_the_way_in_as_well ... ok
test a_delta_written_in_a_format_this_build_does_not_read_is_refused ... ok
test review_new_default_delta_format_is_version_two ... ok
test a_delta_this_build_wrote_is_read_back_without_complaint ... ok
test a_delta_whose_id_was_edited_is_refused ... ok
test the_changes_are_written_in_the_category_order_and_not_the_alphabet ... ok
test a_delta_whose_relation_was_edited_is_refused ... ok
test diffing_the_same_pair_twice_produces_byte_identical_json ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/families.rs (target/debug/deps/families-8952d48644da49b5)

running 69 tests
test a_filter_removed_reads_as_containing_every_instance ... ok
test a_bindings_failure_policy_is_compared ... ok
test a_component_accepting_a_new_command_is_changed_and_not_widened ... ok
test a_binding_reacting_to_a_different_event_is_reported ... ok
test a_binding_invoking_a_different_command_moves_its_mapping_with_it ... ok
test a_bindings_naming_is_compared_key_by_key ... ok
test a_mapping_filled_from_somewhere_else_is_reported_with_both_sources ... ok
test a_construct_moving_between_files_is_not_a_change ... ok
test a_commands_naming_is_compared_key_by_key ... ok
test a_command_added_is_one_change ... ok
test a_filter_respaced_is_the_same_predicate_and_no_change ... ok
test a_binding_added_is_one_change ... ok
test a_newtype_that_wraps_something_else_is_reported ... ok
test a_component_that_no_longer_publishes_an_event_is_reported ... ok
test a_guard_respaced_is_the_same_predicate_and_no_change ... ok
test a_filter_that_contains_different_instances_is_changed_with_no_direction ... ok
test a_payload_declaration_arriving_is_a_payload_change ... ok
test a_new_transition_arrives_with_the_outcome_that_takes_it ... ok
test a_types_own_invariants_are_reported_as_different_and_never_as_stronger ... ok
test an_actor_declared_with_no_grants_at_all_is_still_a_change_to_report ... ok
test a_type_that_became_a_different_kind_of_thing_is_reported_as_that_and_nothing_else ... ok
test a_union_that_is_tagged_by_another_field_is_reported ... ok
test a_union_variant_that_carries_something_else_is_not_a_variant_removed_and_added ... ok
test a_view_added_is_one_change ... ok
test a_union_gaining_a_variant_widens_it_just_as_an_enum_does ... ok
test a_view_exposing_a_new_field_is_reported_with_the_type_it_carries ... ok
test a_view_fields_naming_is_compared_key_by_key ... ok
test a_view_projecting_a_different_entity_is_a_source_change ... ok
test a_struct_field_that_changed_type_is_reported ... ok
test a_views_consistency_promise_is_compared_and_not_classified ... ok
test an_entity_field_that_changed_type_is_reported ... ok
test an_entity_added_arrives_with_its_synthesised_state_enum_and_nothing_is_diffed_inside ... ok
test an_entity_field_replaced_is_removed_and_added_and_never_a_rename ... ok
test an_error_that_gained_a_field_is_reported_with_the_type_it_carries ... ok
test a_views_naming_is_compared_key_by_key ... ok
test an_event_field_that_changed_type_is_reported ... ok
test an_entitys_naming_is_compared_key_by_key ... ok
test an_entity_fields_naming_is_compared_key_by_key ... ok
test an_event_renamed_is_reported_as_removed_and_added_and_never_as_a_rename ... ok
test an_events_wire_name_moving_is_not_the_event_moving ... ok
test an_input_added_is_reported_with_the_type_it_carries ... ok
test reordering_a_commands_input_is_reported_once ... ok
test an_identitys_display_name_and_summary_are_compared ... ok
test an_input_fields_naming_is_compared_key_by_key ... ok
test an_input_that_changed_type_is_reported ... ok
test an_outcomes_summary_is_compared ... ok
test an_invariant_statement_reworded_without_moving_the_predicate_is_still_a_change ... ok
test an_outcome_added_is_one_change_and_claims_no_direction ... ok
test renaming_an_entitys_identity_is_the_one_rename_this_crate_reports ... ok
test reordering_a_views_fields_is_reported_once ... ok
test reordering_an_enums_variants_is_reported_without_claiming_a_direction ... ok
test reordering_an_event_payload_is_reported_once_and_not_as_a_field_change ... ok
test reordering_an_entitys_fields_is_reported_once ... ok
test reordering_a_commands_outcomes_is_a_real_change ... ok
test what_an_outcome_emits_is_compared_in_order ... ok
test the_error_a_branch_reports_is_compared ... ok
test the_paragraph_saying_what_the_system_is_is_compared ... ok
test the_specifications_version_moving_is_reported_and_is_not_the_identity ... ok
test what_an_error_tells_the_caller_is_compared ... ok
test writing_out_a_naming_default_is_not_a_change ... ok
test review_view_parameter_naming_is_compared_without_a_filter_edit ... ok
test review_reach_is_a_change_without_an_unrelated_surface_edit ... ok
test review_outcome_refusal_is_independent_of_its_error ... ok
test review_cli_top_level_grouped_views_and_binary_are_changes ... ok
test review_view_ranking_is_compared_without_a_filter_edit ... ok
test review_outcome_sets_are_independent_of_event_payload ... ok
test review_residual_refs_cannot_hide_beside_a_classified_change ... ok
test review_unclassified_transition_order_cannot_hide_beside_a_classified_edit ... ok
test review_relation_cardinality_name_and_removal_are_changes ... ok

test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s

     Running tests/graph.rs (target/debug/deps/graph-746a5caf03f88bea)

running 10 tests
test correction2_row_shape_is_a_distinct_dependency_and_survives_graph_union ... ok
test a_component_is_reached_through_what_it_accepts_and_publishes ... ok
test the_graph_records_the_reference_an_author_wrote_and_not_its_reverse ... ok
test a_closure_over_the_whole_model_terminates_and_stays_inside_it ... ok
test a_type_is_reached_through_the_declarations_that_hold_it_and_not_by_name ... ok
test building_the_same_graph_twice_produces_the_same_edges_in_the_same_order ... ok
test review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union ... ok
test review_cli_views_and_parameter_types_are_forward_slice_dependencies ... ok
test correction2_network_exposure_matches_actual_routes_and_owned_domains ... ok
test every_relation_in_the_vocabulary_is_minted_by_a_specification_this_repository_ships ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/impact.rs (target/debug/deps/impact-1ac5d4ffb424fb86)

running 17 tests
test a_suite_produced_from_the_later_revision_is_refused_rather_than_narrowed ... ok
test two_specifications_of_different_systems_are_refused_here_too ... ok
test raw_legacy_impact_cannot_discard_a_coverage_inventory ... ok
test a_suite_whose_contract_digest_its_model_does_not_compute_is_refused ... ok
test a_suite_for_another_system_is_refused ... ok
test a_variant_removed_from_an_enum_reaches_the_entity_that_holds_it_transitively ... ok
test an_edited_entity_invariant_owes_every_scenario_that_rests_on_the_entity_and_no_other ... ok
test taking_a_grant_from_an_actor_owes_only_the_scenarios_that_act_as_that_actor ... ok
test a_narrowed_answer_never_reports_more_scenarios_than_the_suite_holds ... ok
test every_scenario_resting_directly_on_a_changed_construct_is_owed_again ... ok
test an_edited_outcome_guard_owes_every_scenario_because_every_scenario_creates_through_it ... ok
test a_suite_resting_on_a_construct_no_graph_has_a_node_for_owes_the_whole_suite ... ok
test the_suite_the_fixture_obliges_is_ten_scenarios_and_the_delta_is_six_changes ... ok
test analysing_the_same_pair_twice_produces_byte_identical_json ... ok
test a_change_in_a_family_the_delta_still_does_not_compare_owes_the_whole_suite ... ok
test a_domains_naming_moving_owes_the_whole_suite_because_no_family_compares_a_domain ... ok
test coverage_impact_keeps_exact_selection_context_out_of_persisted_impact3 ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s

     Running tests/review_adversary_f01.rs (target/debug/deps/review_adversary_f01-233b78233bca1148)

running 7 tests
test explicit_domain_naming_defaults_remain_semantically_equivalent ... ok
test complete_generated_and_authored_suite_four_bytes_remain_frozen ... ok
test relation_delta_versions_refuse_relabeling_and_public_serialize_bypasses ... ok
test moved_outcome_reference_is_independent_of_a_classified_edit ... ok
test moved_outcome_reference_retains_its_owner ... ok
test ownership_cardinality_invalidates_both_emitted_schema_ends ... ok
test incomplete_schema_stamp_is_owed_by_the_real_impact_reader ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s

     Running tests/review_adversary_f01_pass2.rs (target/debug/deps/review_adversary_f01_pass2-46209d7fd7b87abf)

running 5 tests
test ranking_precedence_survives_the_checked_delta_roundtrip ... ok
test reusable_row_type_belongs_to_the_view_slice_it_supplies ... ok
test switching_equal_row_shapes_retains_independent_residual_coverage ... ok
test reusable_row_invariant_change_reaches_its_openapi_artifact ... ok
test served_view_change_reaches_its_openapi_artifact ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/revision_pair.rs (target/debug/deps/revision_pair-ba032edad7953c23)

running 11 tests
test a_revision_compared_with_itself_reports_nothing ... ok
test taking_a_command_from_an_actor_narrows_what_the_system_permits ... ok
test the_fixture_pair_differs_by_exactly_six_changes ... ok
test nothing_the_after_revision_only_rewrote_reaches_the_delta ... ok
test granting_a_command_to_an_actor_widens_what_the_system_permits ... ok
test rewriting_an_entitys_invariant_is_changed_and_quotes_both_statements ... ok
test the_delta_survives_being_written_and_read_back ... ok
test two_different_systems_are_refused_rather_than_reported_as_a_rewrite ... ok
test adding_an_enum_variant_widens_the_type_that_accepts_it ... ok
test removing_an_enum_variant_narrows_the_type_that_accepted_it ... ok
test rewriting_an_outcomes_when_is_changed_and_renders_both_guards_canonically ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

   Doc-tests ess_conformance

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_diff

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 2 targets failed:
    `-p ess-cli --test coverage_writer_adversary_pass1`
    `-p ess-conformance --test coverage_writer_adversary_pass1`
```

Actual exit 101; wall 55.706447 seconds.
Formatting check and strict all-target package Clippy both exited 0. No test/source changes followed the package suite. Their full output is retained below.

### fmt-check

```json
{
  "argv": [
    "cargo",
    "fmt",
    "--check",
    "-p",
    "ess-conformance",
    "-p",
    "ess-cli",
    "-p",
    "ess-diff"
  ],
  "cwd": "home-path:sha256:81a8aaad60388f74814838864c1b3182e25ef96fb5f2eac69a3110c015b31d04",
  "environment": {
    "PATH": "home-path:sha256:d972722bf5a8a5abf3ab363acc673b42b0d50eca552e59c1ad0efea4f8e3a23e",
    "RUSTC": "home-path:sha256:da7abda9718cfd0d0c1e74f56f34135d83b4a908741f60a13ad1ed963424c535",
    "RUSTDOC": "home-path:sha256:cfcca0fbc01edced245f4082ef3f4032907e9866e0d726bc4bd1861d26fbd53d",
    "TMPDIR": "home-path:sha256:a85023f45fdaccd57c62a4a1b51cee490171a57c6bb3db6c9cefabd69f61a130",
    "GOCACHE": "home-path:sha256:0c1bef61d39b03d4967ac4d3b1cdd105e585a69dc1e44ea973cffae93441bbc4",
    "GOMODCACHE": "home-path:sha256:c929e6078a0cc5289ec52e6bb58446312523dbff76bfa246689e269714ee6c63",
    "CARGO_HOME": "home-path:sha256:d30f9c4d89bf9b9f39d84a2047df7cbaa5c6a8acb7e20684ab0d33507967c60f",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:c7cdced08eb66cb30394fa5c6ddfe52ed26ffea4906e648caf1ea6e9f20537d9",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:ba702cd8afbb9b2b51db61d90b5cd68c0bef28ec5a525798adac54e71bdc9323",
    "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_DEBUG": "0",
    "CARGO_CACHE_RUSTC_INFO": "0",
    "CARGO_BUILD_JOBS": "2",
    "CARGO_NET_OFFLINE": "true"
  },
  "unset": [
    "CARGO_TARGET_DIR",
    "RUSTC_WRAPPER",
    "SCCACHE_SERVER_UDS"
  ],
  "free_before": 20486881280,
  "started_epoch": 1788725489.329218,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass1.rs": "dec5b723ccb77c6d732119e1820f92e1555e13b4b777a4d3d4cf5e4906a31736",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass1.rs": "950a4ac1cf1b5b8f1272396e5f4228718f7af26345b0cd34f234e3da587fc373"
  },
  "exit": 0,
  "elapsed_seconds": 0.43021910602692515,
  "free_after": 20486877184
}
```

```text
```

Actual exit 0; wall 0.430219 seconds.

### strict-clippy

```json
{
  "argv": [
    "cargo",
    "clippy",
    "--locked",
    "--offline",
    "-p",
    "ess-conformance",
    "-p",
    "ess-cli",
    "-p",
    "ess-diff",
    "--all-targets",
    "--",
    "-D",
    "warnings"
  ],
  "cwd": "home-path:sha256:81a8aaad60388f74814838864c1b3182e25ef96fb5f2eac69a3110c015b31d04",
  "environment": {
    "PATH": "home-path:sha256:d972722bf5a8a5abf3ab363acc673b42b0d50eca552e59c1ad0efea4f8e3a23e",
    "RUSTC": "home-path:sha256:da7abda9718cfd0d0c1e74f56f34135d83b4a908741f60a13ad1ed963424c535",
    "RUSTDOC": "home-path:sha256:cfcca0fbc01edced245f4082ef3f4032907e9866e0d726bc4bd1861d26fbd53d",
    "TMPDIR": "home-path:sha256:a85023f45fdaccd57c62a4a1b51cee490171a57c6bb3db6c9cefabd69f61a130",
    "GOCACHE": "home-path:sha256:0c1bef61d39b03d4967ac4d3b1cdd105e585a69dc1e44ea973cffae93441bbc4",
    "GOMODCACHE": "home-path:sha256:c929e6078a0cc5289ec52e6bb58446312523dbff76bfa246689e269714ee6c63",
    "CARGO_HOME": "home-path:sha256:d30f9c4d89bf9b9f39d84a2047df7cbaa5c6a8acb7e20684ab0d33507967c60f",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:c7cdced08eb66cb30394fa5c6ddfe52ed26ffea4906e648caf1ea6e9f20537d9",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:d0a6fbf98ae33ac80b21f1e2b2762e53f7e02b4114d01e3f87a2a4491f2c8bbe",
    "CARGO_INCREMENTAL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "0",
    "CARGO_PROFILE_TEST_DEBUG": "0",
    "CARGO_CACHE_RUSTC_INFO": "0",
    "CARGO_BUILD_JOBS": "2",
    "CARGO_NET_OFFLINE": "true"
  },
  "unset": [
    "CARGO_TARGET_DIR",
    "RUSTC_WRAPPER",
    "SCCACHE_SERVER_UDS"
  ],
  "free_before": 20486725632,
  "started_epoch": 1788725502.0785294,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass1.rs": "dec5b723ccb77c6d732119e1820f92e1555e13b4b777a4d3d4cf5e4906a31736",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass1.rs": "950a4ac1cf1b5b8f1272396e5f4228718f7af26345b0cd34f234e3da587fc373"
  },
  "exit": 0,
  "elapsed_seconds": 25.91365824698005,
  "free_after": 20485578752
}
```

```text
    Checking serde_core v1.0.229
    Checking memchr v2.8.3
    Checking itoa v1.0.18
    Checking cfg-if v1.0.4
    Checking zmij v1.0.23
    Checking typenum v1.20.1
    Checking serde v1.0.229
    Checking serde_json v1.0.151
    Checking hybrid-array v0.4.14
    Checking crypto-common v0.2.2
    Checking block-buffer v0.12.1
    Checking const-oid v0.10.2
    Checking allocator-api2 v0.2.21
    Checking equivalent v1.0.2
    Checking foldhash v0.2.0
    Checking digest v0.11.3
    Checking hashbrown v0.17.1
    Checking cpufeatures v0.3.1
    Checking sha2 v0.11.0
    Checking indexmap v2.14.1
    Checking ryu v1.0.23
    Checking dyn-clone v1.0.20
    Checking unsafe-libyaml v0.2.11
    Checking schemars v0.8.22
    Checking serde_yaml v0.9.34+deprecated
    Checking thiserror v2.0.20
    Checking ess-primitives v0.20.0 (home-path:sha256:023a7ec9c4baa6d1cc2b055c33ff4f284fda7425258e04b1e2d84a65bca2ff39)
    Checking bitflags v2.13.1
    Checking ess-domain v0.20.0 (home-path:sha256:5dc48ee32bdcfb6b6531fcede0e40e5bf2c6b1fee19aeff3b8b7224c54dffe12)
    Checking unicase v2.9.0
    Checking pulldown-cmark-escape v0.11.0
    Checking pulldown-cmark v0.13.4
    Checking num-traits v0.2.19
    Checking libc v0.2.189
    Checking ess-compiler v0.20.0 (home-path:sha256:3bc6067a43285b74514bb9af08638497ee79278b1a934119ee774460e105b42f)
    Checking num-integer v0.1.47
    Checking ess-gen v0.20.0 (home-path:sha256:9becbd77f058900ac1137c29d4ab62344dce2d3bf8292c8d48ea36f6ab645cda)
    Checking num-bigint v0.4.8
    Checking regex-syntax v0.8.11
    Checking ess-conformance v0.20.0 (home-path:sha256:fb2af4db425baf24f833fe7f0be66e10d98c52642fdd023fe8c8d2f68266c344)
    Checking num-rational v0.4.2
    Checking getrandom v0.3.4
    Checking zerocopy v0.8.56
    Checking ess-diff v0.20.0 (home-path:sha256:5455152f24506d134276c0fa17104395c6cd66014ed2c8978ece31453da08d17)
    Checking num-iter v0.1.46
    Checking num-complex v0.4.6
    Checking aho-corasick v1.1.5
    Checking scopeguard v1.2.0
    Checking smallvec v1.16.0
    Checking once_cell v1.21.4
    Checking utf8parse v0.2.2
    Checking regex-automata v0.4.18
    Checking anstyle-parse v1.0.0
    Checking ahash v0.8.12
    Checking parking_lot_core v0.9.12
    Checking lock_api v0.4.14
    Checking ref-cast v1.0.27
    Checking num v0.4.3
    Checking infra-domain v0.20.0 (home-path:sha256:17ff7adea08c8d7f520ef548c6782574d0845e3fea23333d4c9b2059eec0bc1a)
    Checking anstyle v1.0.14
    Checking is_terminal_polyfill v1.70.2
    Checking bit-vec v0.8.0
    Checking colorchoice v1.0.5
    Checking borrow-or-share v0.2.4
    Checking anstyle-query v1.1.5
    Checking fluent-uri v0.4.1
    Checking anstream v1.0.0
    Checking bit-set v0.8.0
    Checking infra-compiler v0.20.0 (home-path:sha256:bed6271e7b917c3aab1b762a9e7a7f1ff2f9741e8573beb74f95f8deaa73043a)
    Checking fraction v0.17.0
    Checking parking_lot v0.12.5
    Checking num-cmp v0.1.0
    Checking percent-encoding v2.3.2
    Checking micromap v0.3.0
    Checking strsim v0.11.1
    Checking clap_lex v1.1.0
    Checking outref v0.5.2
    Checking vsimd v0.8.0
    Checking bytecount v0.6.9
    Checking jsonschema-value v0.52.1
    Checking uuid-simd v0.8.0
    Checking clap_builder v4.6.6
    Checking referencing v0.52.1
    Checking infra-analyze v0.20.0 (home-path:sha256:692408daeac51b82bcbca16da5b417c8027986bb97bba5454630d80dce1ecfef)
    Checking strum v0.28.0
    Checking unicode-general-category v1.1.0
    Checking fancy-regex v0.19.0
    Checking regex v1.13.1
    Checking jsonschema-regex v0.52.1
    Checking email_address v0.2.9
    Checking data-encoding v2.11.1
    Checking clap v4.6.6
    Checking infra-spec v0.20.0 (home-path:sha256:5fe1019747531706fddcdc48309c0267f716e1362016a58ffa8d78dd92860e3d)
    Checking jsonschema v0.52.1
    Checking ess-realization v0.20.0 (home-path:sha256:a56480f6286fbe5ae68fed8d524230f135205d34bb6bcb110902eb6c6eaf2f8b)
    Checking semver v1.0.28
    Checking base64 v0.22.1
    Checking ess-deployment v0.20.0 (home-path:sha256:0db4f9f65afa6dfb9bfe6134f097a4511a1bbe5f13085097618e052df0e00c93)
    Checking infra-project v0.20.0 (home-path:sha256:2518a4648c139107e578e6fb934f325373f31799eefe2eeb85be8c3643830dcf)
    Checking ess-kubernetes v0.20.0 (home-path:sha256:b48f74815ced76f6c132664a9490e73fa3e7f91a4c397d97387871f45cb03a64)
    Checking schema-contract v0.20.0 (home-path:sha256:59e491e7ffb494ae9905baec320f21cf3d49d97493d328be25c7ea7896369370)
    Checking anyhow v1.0.104
    Checking ess-synth v0.20.0 (home-path:sha256:86d0c66b43ae445f8298081b67d4624cc72d3379d7cbebd68899127289ffa9c9)
    Checking ess-composition v0.20.0 (home-path:sha256:cf9c6e2be3ba3f0b1c2d4f47b62ab1102fe586f5d38602cdde999e25648dcc7a)
    Checking ess-openapi v0.20.0 (home-path:sha256:bb891c3627776b8febf75959df9c028f38f33b1327e40dc25af05b9a4235c053)
    Checking ess-cli v0.20.0 (home-path:sha256:5a10dd49c67cef5019797350b37afacc7816747d7c22f5c97b15b2438867815c)
    Finished `dev` profile [unoptimized] target(s) in 25.87s
```

Actual exit 0; wall 25.913658 seconds.

4. Findings covering frozen subject 874962d3c7f84d0337bb1477892da065aeadc2fd

| File:line | Category | Severity | Verdict | Origin | Finding |
|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/coverage_build.rs:219 | contract-drift | blocker | NEEDS-CHANGE | introduced | Generated and authored coverage messages retain only Cause Display instead of the full original typed Refusal Display selected by the sealed D1 correspondence decision. |
| crates/verify/ess-conformance/src/go/runtime.go:3693 | contract-drift | warning | CONFIRMED | introduced | A strict skipped run over known complete suite/5 prints that legacy suite coverage is unknown even though its admitted inventory and emitted report both say complete_inventory. |

D1 measured: coverage_build.rs:219 is the generated mapping; the same issue is at :466 for authored mapping. Both focused tests exit 101 on exact diagnostic equality, after asserting real occurrence/source/scenario counts. Authored repeated-refusal data is retained in its comparison.json even though the duplicate-source equality is the first panic. The first generated comparison shows all eight causes preserved but all eight full diagnostic headers/help omitted. This is one mapping finding across the two producing branches.

D1 reachability: public coverage_build::build calls actual synthesis and authored compilation, and the CLI's fresh coverage path uses that builder for synthesize, author, run and web. The tested fixtures are unchanged inputs/models/billing-repeated-gap and inputs/authored/{accepted-duplicate,refused-then-accepted}; the exact preparation copies are pinned independently. The producer exporter check_structure currently substitutes required_original_cause as the complete expected message at tests/support/coverage_producer.rs:266. It therefore passes with the writer while failing to enforce the separate full-rendering D1 selection. The blocker is specifically the root-required selected D1 correspondence before qualification; this report does not classify the earlier cause-only suite as malformed under the original broader minimum. Minimum correction: preserve each typed Refusal before moving its fields and format the full original diagnostic in both branches; add the authorized separate D1 pin to the new export expectations, retaining original plan/oracles and every occurrence, source, selection and count assertion. Do not rewrite the old diagnostic implementation or add ordinal text.

Go diagnostic measured: runtime.go:3693 prints the legacy-unknown explanation after the actual known suite/5 skipped run; the regression exits 101. CountReport::from_json pairs the real report to the exact admitted selected suite, and its category, knowledge and conformance assertions pass before the diagnostic assertion fails. The actual Go process exits 1 as required for strict inconclusive execution.

Go diagnostic reachability: ordinary `conform author --suite-format 5` → go::emit_input → Run with ESS_REPORT_FORMAT=2 and ESS_CONFORMANCE_STRICT=1; a supported public Target can return ErrUnsupported from BeginScenario. The minimum correction is to derive the explanation from the admitted coverage and actual conformance reason, or use truthful status-only wording, retaining the nonzero strict result and report bytes/semantics. Do not change skipped to failed or claim missing inventory.

Origin accounting uses read-only base objects at d2057ffb944455d0ef3a90ab7c5043ae70027289 and the frozen diff. Coverage serialization is new. The Go wording already existed, but the base refused suite/5 and could not have known suite/5 coverage; this unit exposes that wording through the newly reachable known-inventory path. Both findings are introduced on the relevant route. No base checkout/build was performed and no pre-existing finding is inferred merely from an old line. origin-analysis.json and base-go-runtime.go retain the source evidence.

5. Bounded scope and limits

- New actual-browser LF control: Rust and Firefox both refuse before replay state; no finding.
- Existing package controls: full original lineage, unused/reordered/missing parent refusal, exact unsigned metadata versus finite Node payloads, typed defaults, D7 selected host-int refusal and parent-only oversized positive, refusal multiplicity, source ownership/component classification and conservative impact all passed their existing cases.
- Existing actual generic Firefox cases: paired model/reference admission, UTF-8 refusal, shared lineage/closed-model mutations and retained legacy-player behavior passed; this is not a closure of F15 literal-assignment or full-view-evaluation omissions.
- Public parser source inspection covered arbitrary declared Node maps and separate integer/string handling. No new dedicated arbitrary-key runtime case was executed and no whole-callgraph or exhaustive correctness claim is made. Source/contract reconnaissance continued during focused builds.
- No AEP dependency, helper execution, store/lifecycle action, Git mutation, source gate, Website/Atlas publication, release, integration or cleanup was performed. Root owns source correction and the next bounded source attack (at most two total), final source/export freeze and actual AEP qualification.
- Toolchains measured: Rust/Cargo 1.98.0, Node v24.20.0, Go go1.26.5-X:nodwarf5 linux/amd64, Firefox 153.0. Actual browser session/PID/WebSocket/BiDi receipts remain with each case. This report is agent judgement plus actual command records; it grants no approval or claimed independent qualification.
- The first scratch setup command exited 1 at mkdir because root had already provisioned the empty assigned directories; it wrote nothing. The adjusted setup reused those exact paths. A read for unit-brief.md in the coordinator (rather than the unit) also exited 1; the actual unit brief was then read in full. These are setup/read observations, not semantic test failures. No compile-failure test is represented as a finding.
- New scratch writes are under this report's directory, plus the two additive tests and the assigned unit target build output. CARGO_HOME uses the explicitly borrowed literal registry/git links to home-path:sha256:795cc87cb07760b6d5b7912377177844252ab3ba669ae55f77aedbde05356ea5 and home-path:sha256:1843c051732303179e2b8a37e95d422e14ff158d88c4699065357e4bab0f4b2c, without following them for ownership/census. No shared target or cache daemon was created. The measured free-space floor was 8,589,934,592 bytes; setup measured 23,678,291,968 and the final strict-Clippy receipt measured 20,485,578,752 free bytes. Every lane's before/after values are retained.
- All assertions remain intact. Four final Rust/CLI executable copies are retained and hashed for provenance; the first focused executable was not separately copied before later additive test compilation, so its exact binary hash is not invented. The first command records its test-source hashes and full build output.

6. Every retained path written outside the worktree

Only home-path:sha256:c7cdced08eb66cb30394fa5c6ddfe52ed26ffea4906e648caf1ea6e9f20537d9 was used outside the assigned unit. The following is the complete post-run census, also in external-browser-paths.txt; external-browser-catalog.jsonl records full path/native bytes, lstat mode/owner/size/inode/device/mtime, literal symlink targets and SHA256 for regular files without following links. It contains 513 entries: 199 directories, 308 files (161,983,501 bytes), six symlinks and no sockets at capture. Per-case command/profile/PID/BiDi receipts retain the observed transient browser endpoints. This census is not a syscall trace of temporary files internally created and removed by tools. No reviewer cleanup was run.

```text
home-path:sha256:c7cdced08eb66cb30394fa5c6ddfe52ed26ffea4906e648caf1ea6e9f20537d9
home-path:sha256:f1256fe005f783df0f47bef8a93a8d01f30fc530f8ff6c7df6e8b2ac881752f7
home-path:sha256:ee4a4e5c20709eaa31fdb7bd8e088394ad87ef4130fa09b369f36c7b3810e9e3
home-path:sha256:2f251a31426e6a79e222d02e3b476b4b16d1f71cd1271dbdeaf1f9fde08868f0
home-path:sha256:06ada8e18831523d5c922817fea2946a96affba5fd6616fd329ae262212f5dcf
home-path:sha256:297909ef908d58bd04968b5be800b91a853884c9b8584d4fc8a8d47ebac4065e
home-path:sha256:6ad4162c15dc9b27ef7bf665c13570328ac151f2b6ec3a1587fbee253766f21b
home-path:sha256:49aa24cbbbce38001c4ad85574f6d01d9cb6d694e80b17b501f580bcea3f5b73
home-path:sha256:56b27f62cae3fcc91a183255285173cef7e1309af561a2e18c7440c07089e31c
home-path:sha256:50518450a9b25bfb327769cd966306366a3654355d7b5b8a3c3d08ef09e356f1
home-path:sha256:43d325cf0a7dbb0ccee5d5a75916040d52fbfbb3c860109f2eb28e19265a0c8d
home-path:sha256:20cc1d466f391118225705fa96849d89d9c894c63f8d46f0e8482cf74440c255
home-path:sha256:7525df2e7a9a3d9c8144393ceb575abbb51a3e196ade7b72f8699b7ff24833c5
home-path:sha256:a2cf3095cc871f76baf7813e0e9cc7a1c4a7d4f8ca2979647b603f358b3a30fa
home-path:sha256:750a09f11d7ce5f8246bd891704fb8b0fdcb1c37888039fc80337b401c72ea87
home-path:sha256:2efa781b4c59958ee1cac08985a4e2400b12552ce3f83300edeec7b67e4e7ab0
home-path:sha256:34e16b2e7c722f37f398f4d1a72c7a783ef18906f3dbd76bab9b1b85ad8c094c
home-path:sha256:10a714dfd315bd08e14ad92dba30d5fb8af26f2229fc3c4cbf4306a2636858fe
home-path:sha256:3d7744f912890908ac1842e59f519cd3ac8a4a1ad09ea66e2769ecf33c170b8a
home-path:sha256:64ec63f17ce2aecf1e2a2ef98730896cf8f16051caa45d86495026a4505a46a1
home-path:sha256:84642242b60c6025f88c2ccbf925f599330ad81ec50bdf8e792f542ab70fa2e4
home-path:sha256:82dc8919b6671ca598df4e1eeaeef58fcd05ef97d9f952f0b3c94712e649112b
home-path:sha256:3f7c3b58e974b26df4a7c863e480dc9289ffd56f07a5025d2675f1ec86cadd72
home-path:sha256:c3c0b95b729dd44f5d77878afa80ed2b4e8c19d018b2d93129ac706989a8ce57
home-path:sha256:4094d2920ae84eaaec7cb3a39b66d2633a226efd37aa892fca702418420fbed1
home-path:sha256:8850e5df3961fba2852591d204f825bcd085b215c8cab40a0245d392498693bc
home-path:sha256:8ee0fc5086a4cab5c8b16b109a4d96d9e266c9fc99073bb5d45651013d5dfd8f
home-path:sha256:a61ef94becd338fc50488ac2ea1b0f242299f831d0afb69a4329ad25e8f37ee7
home-path:sha256:a54787f83d32e9355907c28a477801c2ddcc2c6c59ed2b0cdf9184132e2bcdb1
home-path:sha256:537946c04fdf42729bd0f5e67922a4f6199d8f32e4595b651c08f8627e2f375a
home-path:sha256:3aad8ff523bb8a0165103c804809c2a6ad019cc8fe610f500b54ee6ad0c2c0e8
home-path:sha256:368728aca2aa9c221c2fbb712a8e3404ef4fb9bb22ba39daebf2bf9f77fc7397
home-path:sha256:8d6a616d5c41dede278accac6fb3910266d435bf6e350462740e75eb10c4d584
home-path:sha256:3b6eb19b78d896859d350b0fc4db56e813e6d9f52925cd9dc840f07c126081c6
home-path:sha256:d73e3e23bcacdde0da0c20bb5cae7a927a88011c7c8ed905bf522383951e8e96
home-path:sha256:b27c511ac53001da69ab79f08a5a5894e9a6bae7d8c92081804591e73229826e
home-path:sha256:15480609fd553063d8fa419b03a95fb62da99589e99b0a9e6a2e5eed2804b757
home-path:sha256:a1dedd0043a2cb8d842efeadaecffd259fe5f661d5498f93299e94731cb2788c
home-path:sha256:256715eb747ee9723e276190f11b067b957935465395e8620a3957e2a6e4efa9
home-path:sha256:0d29cb617356157c68a2eac886872827d34d770058e99a460d6722041e648447
home-path:sha256:c7a458b7cc6f64aa5fbf2a8df3c4979d2bf9bf9c9956f9ef0e1e2984e90febdf
home-path:sha256:d4dceccf4c2ff84e822e0548c7dabd69d6c36f916a3497555508fe800756e203
home-path:sha256:b5ac3937962b4bb341651d1b9f8b4f4b8aa562d4ed7d296175500f88da8e6149
home-path:sha256:950250658f5da4b5fbc16c7bf68e47e4113b1387fa7c033bdb00e219045ce127
home-path:sha256:d45c5823aa87d0d8b78f12aad1f6b075513d9adc0e245bccf5db85625d532227
home-path:sha256:020a223be223db84d7299558432a0c66d69da3dd7e8393eccd7d75069d4d9141
home-path:sha256:210108d3f8258343eb62727242ba3b42166249599bdd2f69a9cbb546eac79dba
home-path:sha256:c7d9eed814189d12894791738d09b4e8f621bae945ccfd53a5008fd065755189
home-path:sha256:2cd6610d9bf36fcc14a0ea93a44304d5a1b882b45ccff5305624d9fb658d93c9
home-path:sha256:b63532863a90fe3a5dfe16f8e6bac8884e254f9838765bce10cabf25660013f6
home-path:sha256:420a0c8abcc50532d517a97bc77e33bf1ff598a7cbfedd357cde9ea3373d0c81
home-path:sha256:c09d76534bafd41a717a42f5b8bc9ce0c0761dc5997832156fb7deb5e93def36
home-path:sha256:da868cd6c53703abb9056c89bfe25197bc96bb9079d68d5c2c41f8c513e008d4
home-path:sha256:1b9f585a5ab191376bb488bf154cc48064918c1f00035fbc7f1665ec652b1bf7
home-path:sha256:aafd2d4f73de687405c4f0bd462924625bff0e848eb84f75a542a9c230751cf7
home-path:sha256:ef25961ab149b72e56f9b347fcaceb88db5e90d44769fdbeda5ff97811586d36
home-path:sha256:9a94d25c287fb173c724438ce6033f677d60b1deca78815f99ce0b317e58cb89
home-path:sha256:f5db5ae901a186567f942d973086cf6a6af0dc69e1b0fb94c976ba8b844ee1d8
home-path:sha256:c2ed091510def0ed89b4c2066c837eb5ef4921d65d2419fcfb2b007c8df1db43
home-path:sha256:3fa487ccde51197f545111279e8a017f18479e982f86be91c34a751bc750f11a
home-path:sha256:e22c91529bb3e5b6715c6e24bccf3c5b8f221c37148f9a3061147bea78e70036
home-path:sha256:b1ce0b50d33b154f905c23abcb2fc8553e1ea03394c0ca9d6754664084b4a9b6
home-path:sha256:b24f0699d22ccc37859f5da240a17651a60b123fdb1aee2ca3aec5d18298062e
home-path:sha256:eec0ae31905f9184a42d65c6894f0d3013c3f2f14dc8acbe50d02102a9dd4bcf
home-path:sha256:2037089f8aaca8c46e1c5f00506d16ec72acf510d2eb5bf4006f2fe581a772b6
home-path:sha256:eb48540a46d90169170233e246e930daafa2523b6a40e279b5c899712805de8d
home-path:sha256:996facbad26a13e2f64d57fef12fb88cb60068496381fecd90f4e5d39fee3c96
home-path:sha256:149ed5e8c6d154aab4b50a84a2c7e45b4ec5c16b9e5d252c3b009eadf94db742
home-path:sha256:0bd9361dad82c80e3c5a4cc0078d261cb00db4da3be81ce21ae4e216eca470a4
home-path:sha256:72f677ee7ece6e54ebd8f9553701f544641a3cfab63f0e32250fe09ffac81e3f
home-path:sha256:3c291fec57eb8e78763bcf7c76e9f669eaee7a8e281a431acc08cc664040939c
home-path:sha256:87ca83e3bf404d98b010bef45ecf8b51012bd0075b4ff93aed9c8f24fab6071a
home-path:sha256:c84f9e3c748ac8226e2c5c6b8461132d5f97eb1c3c6915461dfea45d03938c1d
home-path:sha256:7f87f4a057c16405018a0182d74e3db80ada84da316be542551a1aa2f833cc94
home-path:sha256:1c213c684995d4a6129aed12e3297d6833d7858dc7da95c7551e1605e85d38d9
home-path:sha256:31161783cc981085052028dbabd612cd8c114062ba6f716739273240bdb5ca0a
home-path:sha256:24245c1c72bcd141541e654ec04d2d52acdd363bad5673551845514441949a2c
home-path:sha256:c4ef514e93f6b4f0d2ea687d0e40e4f7fd36a3b81eb2589dccfc5726806b27a8
home-path:sha256:f9882dd403cdf3953d47cf9c8313721708dc9595585dc2ae471c9c224b7a667d
home-path:sha256:f9a9a454f9a623ca2a3b14e8e0589a1fbee5a21f63ed70f9f1b6ae6bd67e2dfe
home-path:sha256:fb2f0479b50c08b4d1d5f1e9c015845e0780b3e46a48eef502ff934a7acc0e9b
home-path:sha256:6547027798b8b9d995d5ad41b9983138bea8489f5974a37280e10811fce5a3b0
home-path:sha256:0c475ef39bf567bb311c3d716dd75776b972216943e10a000e1574352f35490f
home-path:sha256:c7b9947953d89d96f2f23cddcb2e7d8756d9f7cc5ba237145abb8f1e04b46849
home-path:sha256:e435ea991f0feeaa4ca6281ef3666f1425d9d6dc7ffc6e5d34690e7a47db57c0
home-path:sha256:02799737a40a11163878cd2b9613786cad6c840f613585420dd66df85e6780bb
home-path:sha256:08f71120e58bbb75655c0cfabb9859f64025d304b323f7b812579871c7f8e298
home-path:sha256:b7522a52ed7c2b9f7f9b246943b71ee3e49f079c24ff1f46fe94fc6dff34b3c2
home-path:sha256:97e66c63ef9d80599ece53533a5809bd57c10ef0786e90016ab37f62ba3e3f8a
home-path:sha256:9de6728d192871ba1f61033f0b09dd059568486308bb4b9829721ef7849b6085
home-path:sha256:5454a6edbc03427a8c4ac35fd8e0713b410db8f15fa2cc0df47bdc3892e63d24
home-path:sha256:9ad0ea0aac8a46679267516980b993e628a02c91e12f687f28c09b1f7646c6c3
home-path:sha256:02d1799d11e44628e372a1957016912ea5ae975863dade67b79888f455f125d6
home-path:sha256:87d696c0370aac4dcda0b90903fdf4427549d5f1cb2915fd6d47e17307b80552
home-path:sha256:899c5a41671e5d8458acee2cb3370536e0252a7e0dc26735bef33b6bb26b88c9
home-path:sha256:92372b0c4ad9b64313210165eb5157161b72aa1c4277b9f546bab51008df0ac7
home-path:sha256:c3996db930c55d66790093b7cb9ca86cfe534ab5bbc62113b1d588d9666dbc7d
home-path:sha256:db661c548fa728b30e0fa30072d3bb1c9c994b6ce9e1ca8ffee3804e5a3e21fc
home-path:sha256:44fe27903d442ea8bfb14a78f3f23616107d04c0cf98be2e5fa9b7a59a116a02
home-path:sha256:34a6e66f61e7d1f8c7ce3b5b2ebe5b120d0ad16695d93ca338810b166c973ee3
home-path:sha256:d16b4d54932ea0850bca70fb545fa8acdd95d8bdd3fac37ca8dc90c70319b625
home-path:sha256:aa19e2ad738455fda0abe696c7a41088c75348538c5ef625cda92d3cb44a25b1
home-path:sha256:e35e7b8396e97169efd31fd78153341df56a1a4982aaeeee70e564feeaa7a0d4
home-path:sha256:05d1f8187f1aeb26a8bb7360ba672734cbfb0e46dfb75412dc92ff20aa11c837
home-path:sha256:1ef149368a37e311bac65e69694b2bfca1743854673ffc65ea7bf656a8f10179
home-path:sha256:abe4a262211fccf84c9adadebd5cc93b1976f99acf779643df0dbf2bbed289ab
home-path:sha256:48e407746d2d1b39674b45504fd85c29e970b71a356740fb4f39f2c611557b33
home-path:sha256:99bb6b2c671f5423f521bfe0f6d331ae432b7502769d532f220010f3a5c2d193
home-path:sha256:bc6eb951cdd10cf1a722bf2f6bece358508978f7966bd04532fbf3165f7ff48b
home-path:sha256:6a41a85ea5d56134973453ab6bfe9848032c8293ccd19cb2b353c1918612804c
home-path:sha256:e854d1f6579a7c27869743613246e5df725f1450fcd5f65cac4d9af3c49773c0
home-path:sha256:f9c2e2b049edd276e96d95dd07a82a51bbe478655739b49f9848070df1fbce55
home-path:sha256:5120cf73b6030d5702288664c868efbf5736d916d326347cad509c2168902d89
home-path:sha256:64d41936530f79e6613abd0e69c62d0ced1f187c80f768df4a2deb1b7bf4e100
home-path:sha256:c728dec115edf295c909289c0597997359b5e27eeba3700b18dca4e8087ca38d
home-path:sha256:463185cc77022661f14e3b2467d688f4abcf120b0e2c68ccc27db1e24bf33cc6
home-path:sha256:41c981e3179bf1464b9bb06cb82e3a25926c3c4fa054eed19659fd2aa33b7fe2
home-path:sha256:7d0cad8b83923ddd9930fd1765e1ae3f7c09a9ad0dcfd63cd3a17cfdd387b35f
home-path:sha256:072c979bc3b568857a58d6860406912801eb7a491bd5ee4be4865da6e978f5fe
home-path:sha256:9963cfe530a89f5ae5ddb2dbae12aa1d8687729a69f598d73e614b6f742d649a
home-path:sha256:05a611a78bb85234ea3bb525f2ec50d252b8b5f6be9d8602134596fbe53ec15b
home-path:sha256:4b6cbe88e47dec48cc13a79e032f74fb615dcea4a2636c7d19684c06f26b3dbc
home-path:sha256:f2a20aaf43645ef68d34707915a5282936725e451ed2fdcc969848d9f47ae872
home-path:sha256:ee692e6441fe3e07e185cab52e8996ab2fc664bf9e0671ff197abfae8caf91d7
home-path:sha256:8cfc14457d5a2f52042bed249abaf7c90fe757997c552e42449293107c9f4222
home-path:sha256:0f03cdf88aae575e14d6ccfcf9b9f7f08950348c5c52aa8d2247b35c701601ee
home-path:sha256:d365733f4743812e28d5f18f9d574c728bdfa10cfb223a8752777e3220fc6647
home-path:sha256:a6b652cfb89fbe0e661d2fbe7cae863c07a9df88d7494bedd4f15cf873cb9c50
home-path:sha256:9a601e63f17e4cbe11e1c1131b5012c1d019dcc36a383a3424d7da390125764e
home-path:sha256:fd856bcf2dc261bb0da999c55761e46c2b1c91ab2c075e7a30129da474db9f67
home-path:sha256:45e4030d874bed6a9e9e311bbd9c54571b5f13234c2deaf1e78a2ba20b391a13
home-path:sha256:38596881dc25ac2d2ef2034e700ab4db7d88df1bb3c87eb38a215b6047ca8609
home-path:sha256:82e9c56cac5eeb80567888dea1e19f4bb167457f37d35403c53c947861a7c996
home-path:sha256:96ee5c559745bbec66dc684a5eb0869ec35ef784e58eba88affdb6a21b7b73c1
home-path:sha256:fa6d0c18ac675291a0d7ab2b92b627dc8a80be615adfc0d21970ee6ad58ea717
home-path:sha256:cb78677533bf0be2b9cac3789d9d29919ef6d4786761ae8cb9fdc3c6e68a4153
home-path:sha256:bd3f3f585c55a423cd71ee873c227acf051dd12ca049ec3d1a303a527632bc29
home-path:sha256:a31816156469cca87d9a5046834366e211288e1cb46335eb86cc1e77ef2aa70a
home-path:sha256:7adf60552c87c87e0520dcf0f05fc6f43f0dc2bc6da3ef8470520b1fefee7d5f
home-path:sha256:ad85916a8542d2fd5ee9b00af92849a5a0edf91ea5b3d48facedec3265ab13be
home-path:sha256:2501e25890efbde7865b29212c7721b48835cafac8811b45ba809889360e2797
home-path:sha256:8522396210ee65aa95bcfa687fc5ed3e982ac8bec5579f267ecea8953b9e08d3
home-path:sha256:db515861348933247b521e01755c75e1095e9bba4864bf8d9700b0f4823ae5d4
home-path:sha256:42484a3e52a849d7bbed90c1437d41965a999ca4ed88a650b62dd8273e5afab4
home-path:sha256:ae5a25e356d7c1e3cb0e3c5bb8c581744a43baf9c35fe3a533a1b109fb56fce8
home-path:sha256:4bf5c6296d27093c2a6378228bbe0c04532ffc137f9fc4f94733759adc37123b
home-path:sha256:9d96f6fcd1847f5c18c8a2e53806fe662159e6d68b2165a3d6d211899dab950c
home-path:sha256:a08f265af38e5b114afb000feb387c7f2a2ae37798b5e86afcd5b8f398dd1a32
home-path:sha256:41a1df590856b5921452c015fed616a082f67f8c54d6f8df0c6d99a65fe68369
home-path:sha256:df66ea86e7af949205008e7f2f7ac023a8e984a2818cca06b13dc59e74868c64
home-path:sha256:3a818bdae2ed9c4e5ae1ffc2245778e252de8f21e5d71bd151ed603e4426f087
home-path:sha256:2d0b700f53a24dd4c34036c0f51309abceb937c60ef8ee99e89252f05ab765b4
home-path:sha256:c9d93e0876b3fd2025da7cf099c76a30534e2cbc50cc13c21a5b5f20fdf156dc
home-path:sha256:6f22084187065398d491344ad30922ae08a3621d7e8f348d5f32cbae3c139067
home-path:sha256:e791a97b8d9b06d6a6e2a08abe22ad0f4a2448339c20ed1d54a486178c346706
home-path:sha256:0cf8df47c8f2c543074e2aed95d4397aa469e441a61240a1677b301eb3f46593
home-path:sha256:026f9cab27fb375c7de12ca141c346a0e0b979320af9998cdff14e3270c200cd
home-path:sha256:567ec288fd6630538db15e07814d97b3be77bc75ae333b567e2099a5b855d00a
home-path:sha256:68c3b57383f76e37498ab1fdcd7bb4e456f367c9a2aaa48689e863a88cf68212
home-path:sha256:511fdfc42ea7e5bd7230b62c9fe4f0cfb9f5d59ed544c687e66c886816182825
home-path:sha256:13b6b640c02035a480cc8fc395895c93462863427839fe1c1aea89329ec66cac
home-path:sha256:80d0906905b58299bf5c8cc329fa54cdd9c4babc720dd27e30cc7fc8836dd46f
home-path:sha256:5011e52055d5e4e11b989c48b73d130c34a65322896bf6b15f0d5103b3a3671f
home-path:sha256:1212fd85e94e769fc3c46c771b4813b8ff61396a7ab3f7366ce16d46bd46373e
home-path:sha256:490bf68d5d772557e18526aa495838b5162a31d82e3c36ffa70aee1c8cb4e54a
home-path:sha256:c28ffc1094d2b3796d787d794be29eb774cc9d0d718b52f269ab895b64c996ea
home-path:sha256:12d86657baf9b31dbdb9e11dd32d7c32f47e8ed3b94e7669cf2361da23f2da6f
home-path:sha256:9e28905fe2b106b80bf8937b04023d843385b0a07bb75d51892b804004a0afa6
home-path:sha256:dde965cad25712fa6a70a7353076a57fed5e77106e87f110a8a939e06b5e5656
home-path:sha256:57d940160f41b226b0990832386d33a19e46d7307e4e2ae870592a2194304dd9
home-path:sha256:f6912910def2b874387885cf068aca0652a0ad8b4ad0868327d628713c82804f
home-path:sha256:ea940e0e47a55733d7a9a73c7afd8bd875b4c1855c0a189c77457a9527aa46c7
home-path:sha256:f97a955bd1ee275011e3d567a467780aa9889afe7f71e2c489f89085d9bcbe6d
home-path:sha256:de0bfec3c8cb1ede86fc91637fa6e2524a5557e24d01498791c6c2a23ede86f2
home-path:sha256:dee572232e3b049389c5d428132fa402be21067064941d8e11d4e43360581cb9
home-path:sha256:ae6f3c7c94adde64df49d12024d771fc99a5cc2bf9872fb28ace48f15f6984b7
home-path:sha256:a5c2e5683f6af81d63024c61f4feb5d65904d2931641f52c0e93dc4b1a253178
home-path:sha256:e5bac562a58bcfffece6fc7c05ba5df1241d723393f8cc7d2ed52ba2612c94eb
home-path:sha256:5b628bfcd466f23aaf7d692c5498a0d8079b5e0feb0115c4e337c4340d266fe3
home-path:sha256:80e100283145f17d85be88c60df94763b5398cccf7f91e6477bceb7d592cd662
home-path:sha256:774cc5d6caf355a8e603e97d4cf8a731e6f14acdd42c3789d6f6c337fa79d184
home-path:sha256:135a46ab1af0a0e059424e6c15f45e0691f33f4be6701ec0f1c6eb8fe1f80e65
home-path:sha256:7e10fb77bf9f5f189a700306fe38b2bf679b709b3d10f2ecd73e6b4f665d0e81
home-path:sha256:de0986530e3700146903db1e7841c258e0f436e07a9f54828d84c67252a068d7
home-path:sha256:15fde2dda722cdca78b8be0bf99e488b06303f610da812f2fb10debb933a2101
home-path:sha256:83bd4c2c5ab90a35acfc9e19e48d0a5432009f91e8ba32a49989d4bb86fa9080
home-path:sha256:99d689a6371bd29d3944842070079328db9906d8a25c617dcf34ad250253f83d
home-path:sha256:ca1cdbdd721ba1a5a0fa21b721246fc5dcd49463b52e9fbc4320be38772dedd1
home-path:sha256:662d787984aded4a4d84d83aa95a8b9b8ca67a5277bd935b6208610b6f98bb6c
home-path:sha256:fa57123c4e7702d3472a277abd4ede56a7f731974da8af323b7f25b0a80872a2
home-path:sha256:964a34b42fd8b0dbf4850569905dc79a0a69bf313ab6cfc727dbaa2a2f720046
home-path:sha256:d040549974c87755bde28cab16c85c2c2377eeaf9e4de6939170ca77234be76f
home-path:sha256:e86b096d8d2e2f28f7efa52e50eaa6ece80c24f5872cad2d3321594587f124da
home-path:sha256:97a3703ca06c5fd654ea8bc61d31ac6710d0b6bc14ba303f018cecfded75edf5
home-path:sha256:d3c1acea29b569028c04b889885d8f26fe656ceada6e28b9f8ba28f3acbfb67c
home-path:sha256:a4ad65ce33a25ba180e374ca98d5c7a2628430161df28a36def590293bfb7f16
home-path:sha256:3886f46b29f3477eb1733c64375a066e2000b211363283bf3685a580b35fdc4f
home-path:sha256:c08f04cdfd1545ccfec3aa5c4ee71c2caca1992ab7f3fdeee455b4196f3113bb
home-path:sha256:705368bc8317f48103788d58fbd29185038b6e43eb64c14c3f8ba745b84a96e5
home-path:sha256:aff2745116be477f0cc119c6990ff30a7017fe3ff6284040597d5641c6d24498
home-path:sha256:8cb49184e3f60aec925c89daab6b2635e45c3f0295cb82505f35bc271470cb69
home-path:sha256:da5b1fd3a732ce8b1765ea4fb2ab936866cc70d7bd1e3d6176d8d3703ef0d704
home-path:sha256:0b1556481f714fcb31d6136aa8b9a873b13d8f6932e3b9b39ed4cc217a7eb4b7
home-path:sha256:0f95c06b0de76811b8a604334ecbe6f6aac73233555922f96dc9227c047f17d3
home-path:sha256:fb0c23843505686082ec60b0f328ec47d4ae738eca82f78f529ad8064316fb96
home-path:sha256:ba6e00a0e2b48ad9acafbff8adee5567ea4336c03f93d78f120c77ec8fad0586
home-path:sha256:2f5176de64f0affcee2bd3a965081333652ec2d196ebf27179d018f145197ff5
home-path:sha256:9f8d723dec966352b28edc862b92020ab57d09eb02e48ce7d0dc75216ab36741
home-path:sha256:731b5a362ccb44e3e9b3b3b63f3600fe90d3a6c274f81e8a7f0aaae98ad35f15
home-path:sha256:9d64c5195c564d9b11ee932469a6c9b958f2d3f0d99ca388bba22893b1ec2e00
home-path:sha256:8f6e365ed74444d7e4f39e95c70409e764fb56cc78913493ff21ab4b9ece472d
home-path:sha256:2b4b99335a0a06c5fb9627bba8b58abd2599bce1be5e90518a6306b1e734434f
home-path:sha256:c95dbc246933428fc24673af3af85eacf417ccc3fe2fba9217e99abf2e31c281
home-path:sha256:838ea942cc37f1bd9493c6ce3bf525e36e6631e86f304ae53a5330537ae08b57
home-path:sha256:a6642986867d3c04d6250e059e96741eb2764445465f9880aee2b4a707ccd93b
home-path:sha256:9f7f8648483c270ab235d6847c87829969457c9f231386fa5d12a962afd7c06d
home-path:sha256:495c44fe83ca9d2baff42f7a1cba6d5fbaf4bdbeddc16a7a33c0934465392768
home-path:sha256:b7260c04fbce1dc9e48dfb5a62439cc4e0f5e20caed7961a7549a27009da0267
home-path:sha256:a108f389d47771db9dbac64c663b71d77935f2915d86be63001696f81e97b3fe
home-path:sha256:1e9a6e8ff13eedbd0240b573de9cba6349541e8b767f90fc711552cd80fd9c2f
home-path:sha256:61f7b5f3e86d02f0162bbd1d2357222a2f93f12efe4cb97e9e288b272b766c49
home-path:sha256:e9da13044224863c23667653e28b377eba7b0b291ab78fd8ec7814ef25132503
home-path:sha256:a7b5cfcbed718476065b1cb017f3da89bf5af9652b561e7fa656b5770456be58
home-path:sha256:771355a4e5eea328cecb66d97e44690dadd4465df95ecc783427bd88368cca89
home-path:sha256:826eaf10187f41841bcb1316f0465b49274ae5cdd50d22389390636d5e225553
home-path:sha256:c1fce04478f3b6abf230784150b83541d4fc9a00036f355d23bfd3d9dcd3ae7f
home-path:sha256:8e662137da3462013d6dc367cd18177bbbbd6dc926c483dd596a4445b1094a7a
home-path:sha256:fa35c37b8a029c4e1de826442fcae440e511195fa5b01cc3d318aeab398c9a9e
home-path:sha256:b50ee7611f373c01a2cd99b46ece6552ba2a4d2c4b1a12b8221053786eeaa221
home-path:sha256:48c4e9459c7cf283a05390ec2f324866e37d114b7045220bed8f9bd2d56ea165
home-path:sha256:194d84c437050e91227f22f9a0e188ac3282075129863d8dec26c8d9da0bf7c0
home-path:sha256:63dd1c3a1924a8840df2771607db7e11a1673eba9f272bfbe3e5ba8e0fb5eb5f
home-path:sha256:946745aafbb9bf5af00151dc5bc6bd6775d99181da57562e888754058efd771d
home-path:sha256:622660d01d14d5527050d5d30990537b57fc98e903df529389b0b51306a72d71
home-path:sha256:73a2e45fb2f3568b2ae8081a64935425d0fa1497bc107052ac1a4753434a4be6
home-path:sha256:a74b7bb671e9b424eda274e5fb26b26f1c8735e0e0e582565b589c90c3c51262
home-path:sha256:ae6496cef647f74ae2b39a926e96c74ba4eec2cdafd2dd5a96866cb458f190c7
home-path:sha256:7d957dfad186028813f4ef720f0ef21258214881795b3ff7e54a26a6befeb297
home-path:sha256:5338a411a33315ad4f86c066b8a81d72d378931a83afd8b6e88baffb7c3a803c
home-path:sha256:0271cc9da56db7521bcdc5e59188a6d5534b86708a8ec51f48a031d791116dc1
home-path:sha256:0bb7cbaf24bcddad35e6f2e2c159f476b380202f086398c181ac4d3dbee58022
home-path:sha256:72c57f79637549423fd7dc11d02c85c6ac299b5df2373c4f3fc9e7e96992bc29
home-path:sha256:cce78189aec77e28edc04a6571f13d3eccfd7201d013e72a7a0b225c3ce0ca7e
home-path:sha256:9c3f416e22f195b22d5b302f428ee690dc0a9bd880bc195ad088cad0de8bf7db
home-path:sha256:601f47968c69e9e0dd93e3469da6dc88e198886e5f7049d8c188b7f57fd2e25e
home-path:sha256:a8cbad1ec137eea1f9a806b011b56a82687485c3e2073613acf997a5c73f71bf
home-path:sha256:d5317e91c879efe7e95e4c7d3d38911440336fa407f9ca1ad4421a917132a083
home-path:sha256:ba2915b9bf488b74a20a6f3705bde8c4dd22cce2f343585c5a59f57f323c4011
home-path:sha256:ecfd89004d5b7a6e4a5cb5fa4b1ef9e9d9520c3e4dd8288e7e99df16454ea955
home-path:sha256:083e9634ac3203df57c0e1eaa601814edd3d5ec29bb5e53c916079d47e29fe56
home-path:sha256:78bcf253cb20bebfba5f9cd6176ba85a5cc800d211b8d321ced43aad758b26c9
home-path:sha256:dd5af0b71eea85681f2c6b0b7cbd6d8765b485a64a81ec0758726c4695eb18c6
home-path:sha256:188fabbcd45e802a6ade2e2fb93aeecf66efb70c739c096120fd5c4fd710f23d
home-path:sha256:a897834ae18b3ab74436d54216dd276ddc7cf59dd25780c4862d18bb7d4625f7
home-path:sha256:16cd767965a6d069ad982deb084a63f9f9e45d8831d118e284e9d3e2cbf3a69d
home-path:sha256:4dc1c4dec4b826490335d736c0030a53f8fb551a10545988c1df7ba70dded113
home-path:sha256:1e20582c6d15a67f98122483e53721a0705c5b9daa6e3f545a690b052140835c
home-path:sha256:7a23b98c617684de1161745404c7ec0309c8b6bfc2277acf9c09d0865e063f22
home-path:sha256:f71dd22ff99bc9e8ca8977cd5288d63953036c3f2fe2bc6478dc7634a436f12b
home-path:sha256:0e6faefa4f70ca8e31a191ea2f3b49c5f82bc1a2e13c82bfcc1a7a5cf479c994
home-path:sha256:c7102e15c7dbff9c6b12c81fbb2638b173c17794d9169bf39f7cc54a77987c11
home-path:sha256:f986361dbee40483c97eacd37d05bfc645111fe4105ceefe3c5177a0deed6c27
home-path:sha256:7b504d9cb60598994f958cd43ce8c13b82815400a82a32f7d14860b18ec1f67d
home-path:sha256:b1d015bf26cfb90ff429b1af385ddd92e2e99ab82761e6b6eec87c46571dec9e
home-path:sha256:0beb27c152c9419cd690004197ec881d3dde14f866a94a9b65147d204f75b136
home-path:sha256:8184f6e845709f465d2115f05628e3579f92742fa5ee4a041c8535c704fb4e58
home-path:sha256:09e3591dfdc9594481071adb8d050729e3b2fe075444c4cd8f5dfb36fdba30f9
home-path:sha256:694ca5e0044d7fda9662965e7c2bee6cc935c28920350a95f6785e21e60de10a
home-path:sha256:280bf0fc64ba42cd478bada0b4b442084162a1c99d85b880db9adc4eca32c109
home-path:sha256:19cce4705ec3a91a6e3e225fcbd069ceb4a31a58ef946c49f269ec12fd7b8c01
home-path:sha256:7d14ebc3b13eac56bd3b933b6a9be4fe8418b8744476a5195e37e7cd1b4d731b
home-path:sha256:28b50e7680c66b623ee16d16adbc722254a9043637c677178c31807b72d08fea
home-path:sha256:a19866ef9841cf110df33ff08ddc2fa01cbbacb923b2f91ff281dc8a403c98cf
home-path:sha256:5d776abd3bab46666da8e0c4469d169bc4c4955f835272cb501884aaa2b66a41
home-path:sha256:c4a627d0ed3aabf2130cfc889d8ad65e75974f130ac8e91db66a45cc31cb59c0
home-path:sha256:ed948b10ae906cf2d46536aa3ebb749ec746b2c88c74cf23348348a20cade2b8
home-path:sha256:c8fb8d62f9b3951fec4904f244d3dc9fcd3b193bafc81a4ccc95db7f2b1a188e
home-path:sha256:6056f6da58552d53c615ae0ec58c0fe3fc4352d17091fb6edd9a397125264246
home-path:sha256:c8412a8c5c7e2456d630c5bed40cd8bb6d24a0eabbb180fd278b8c133a4e4c34
home-path:sha256:8a81341d9bb77431a23d373db6b6ce74e635320f0357f83cfeeb04b9f288834f
home-path:sha256:06d7bf9a1fef6ab8ca98772281ba3b322a43218f566bb4113bb1262214861378
home-path:sha256:a22a679f6f97292e5095c6867703a5f99b93fcec430703c14703d95cfa6e6459
home-path:sha256:3f3130ebbb8ef7114f7cbc9ef0c341f9e4075859a63c7665767cedcbed4b707c
home-path:sha256:2339098e8a67de2c33cb6fc706ea3ed93c6ff751d283f26a2fd9d1f06119d235
home-path:sha256:a1e35915182bfe16db97bf7b554be3e1cb0d4e62539ea47ceac97d4d675ddd1d
home-path:sha256:f7b8a385efd8575acdedb2cc27c8886da1c8d9a914f49660a684d6345e68769e
home-path:sha256:8ca332e86d4a366a9572fcc503eaacc2f2484587dd4266ffedb6c0ff77517de1
home-path:sha256:e6dbbec8185e1558a1fe870bea816977015d4354090c030691ea9b109e970364
home-path:sha256:90811b2516d89d45bd92dcc7164f09bd3da8c26869153b36a9f3c440e45d19c3
home-path:sha256:47bae9914b23531c56800d8aad683bd4cc4a94d7d4f99fb476ef83b2bbdcdab7
home-path:sha256:19c80de1a9ee2c18a4b9acde21828ce2d46b784b8c531f9084304d45e2723cc1
home-path:sha256:a675dd8cd68ee0dd1367599219ba772965bffa4d347538c845d808b194df32f9
home-path:sha256:06e0f7cf741f948d331b561c1d01a6eb26d1a5b71bf9c941b456285358a1c913
home-path:sha256:dc194e919c762abaa6f5760a99f2e732b8c8c965fa178d3b2110b1e015455277
home-path:sha256:613bf7cd781145ff46250bebbba41165feb0b09dc3e9cafe24e995469e27f6aa
home-path:sha256:d2b91cd532f3d89386f4e684492968f35fb9edbcab1f9671a90edf4f99b95138
home-path:sha256:7f37238f8f6a244ab64143da4769cfe5f11e7b182fddf0f884e38e3e860961bf
home-path:sha256:b9c9313c90e36bf3e9e0e5bc136d6cc24d6df4ceae826363d412b1dcd9d7f5a6
home-path:sha256:38a60b85e3d1248ba532b8a620222e1cc8e077b3a0061b40b1fc5dbce72f2d21
home-path:sha256:a79a1004756f1aeef1b98305cd538310360bae4d01f09c67708afc6df460d11c
home-path:sha256:f9001fc154f299edf9fd3c4e4f559f5b68b74440bbd7830f5546ca17165c20e0
home-path:sha256:841b34c160957a6b87fee4e57e217f5de9cb94b41b1d7afe8d676fa1153eec62
home-path:sha256:137a5f641be723aa78c46687a3ea808c0f22351e4cf5c748931e8b2942b568c5
home-path:sha256:584b3eb0a70c1966a2ff89ecd134cffa6d7216d38992a3c0dd82a3b744d9cfad
home-path:sha256:dccdb64c2a83409318d876f2e0ea2a9b0534ced56018d175c49f87a29db458f4
home-path:sha256:0e2f497bc70cbcef17946736fa9390997b8decab0ebde68cba47b7882f539ecc
home-path:sha256:c41305a13a5cc6e969222e5dd52f4d1aa75b6cc67af3c53b864c4ac16169757f
home-path:sha256:975572c6f23c826b3b1a3d751e535482318c890a9fdb678b59106ffc6b6678d1
home-path:sha256:30835366f3f194662f0ad6e404e3502802851966de38f22a71737efe16d4bc63
home-path:sha256:275c97294e5a24a1a4f912dd88c707d24ff9b73e77daed3dcb736885029f0768
home-path:sha256:7978cc901372d61ce457913b0b3846917bd94f9f27842cad8a7791ce62cd3b8d
home-path:sha256:5945cfeb5bf565699e5b3e5720175c5f23ca7c62ce6e91f82eb8ea5ce7ab95c0
home-path:sha256:3f21a2706e6b234792f21046dbcb1d9357ba436d1d9eec3ccf15b9baec38a544
home-path:sha256:95158e46ef988d7692a4aca9c8c417c62f18e152c97756e43cf4150f739143e1
home-path:sha256:d11b7575821bb07b81c1d5d605baa306c9af08368eb8d481a0cad4f05083b337
home-path:sha256:d79b3ac9645cbd0bdfbb8a243b74768f10e2b2463c4a9ea8d3c3632d3ad1d6fd
home-path:sha256:093652dcba310a794c06cdf363fd6b8f11453b254c3c47aa939010847ca3a540
home-path:sha256:3ba0cb2b2a83191641eed4235c4dcde01d3edb3ed705ae3336cb823c367e060a
home-path:sha256:166de4e0776493209e489cff01be261a60309991435be906f04b592187141532
home-path:sha256:08fb35b6078a02df1edc1900aa9a602252c91b5c4e395515773751a28612d6b0
home-path:sha256:27542f31f373e62c7d65a51ec7a5535a885aeaa88768b6d4b75c825203063e42
home-path:sha256:cdceed6e0616ade58a486b3f72b591409b1d2572e3a0ea681b43dbce8b131f06
home-path:sha256:db878d0a4f4246404c86cebab27ae9bde48abb1d654df3391b72b37d92502036
home-path:sha256:8c2edb85a69c946a34005894c46cbc5ef11ecfe8d3725de9ee61e68324f6edcb
home-path:sha256:aa13a27dbfe97ec47d408e623b3ab2dda509a0e1da9303ec9b1c8b7a4681f3d5
home-path:sha256:33a264a21b2bb589ba764109c5aea0e56d01bf850b02dfb2181a447b91067365
home-path:sha256:2906a621fa0c3e39df545e6b56a08b30f7e1bf9aceb2ccf741e0e90f8e6dd43d
home-path:sha256:0ec5336331c5bfea3f941b9305a506ca8c99257c4c5e4e051fdccb6b7c4c6aca
home-path:sha256:f8fbb9f47abfd01aaf7a2a20a6bb2cf21f47f15d8824b3999f1e65cfbeb6dd1a
home-path:sha256:116686616526294031cc4ad24786053743047aad663d3d00adea3ce4c0bef8f7
home-path:sha256:3c7bd044b8a463c7db2939fb64662d37bed01f2ed6cd858425ffa0603802e3ce
home-path:sha256:7c18a225cfe52328fd62e16f56391bbe6c2af6b514e86365bd734c8444c1bd68
home-path:sha256:b94dadd071bc4692bfb648f9972fc51db56c1414e2a4aaa5a6182c192f470ffb
home-path:sha256:3bd377778bb473f2e468fb2abebe027af6601e17e3218862a4b50be785872257
home-path:sha256:104e92b67dda4df0a34addb102023d47ef6a02e336260962700aef88b9e21d1b
home-path:sha256:5fc6ef253a67ac0443adba93e9671971d5e5fee111ed2819ea762eb3a93a6202
home-path:sha256:4725c69e87e730bbbc2ef68837e66c681e02e16860d25a1922b5a6aaf42e60fa
home-path:sha256:59cfe6e3784f073970f27f0832a08a1e1851e38c7d5e87e2491dc5d9765b019c
home-path:sha256:1ccfa0b88efc43d40d3cad341efdc69a9f7c86bedac93ce335be000a6c51ef47
home-path:sha256:145e6dd1d1d4b96b4a258bdd945987b9295746d642db7b5c945c92879f03b033
home-path:sha256:eee15520f3249550447300c335ed1c6812d8704e2c4eae84b22bac2ff72387a3
home-path:sha256:9cf99ea191c1e4e2e62128f109d1bca48d146399fa8e3ef705a1f66b68dc09ee
home-path:sha256:94f0f87a1c3768775c9eae189028b68357b9dc233e73223066a329bef6c3aa26
home-path:sha256:74e4970c4523d78daa6e25e47115a40c4cc440086d2d4341cc054b900c91fdc4
home-path:sha256:68bad1e926bcbd122f4a0b9849198c38b550a8c602de40974b4da2bcf25937f6
home-path:sha256:7f0bd0687e96caf505264916d180dbeb8aa7a5f28d98b3cf3ca8481794fa1ffa
home-path:sha256:2ddddaee23ce8b19e5d16711662271fc4f14b179882a72c0397c442cc2849fe1
home-path:sha256:f1df0b022df4dab3096d6ca3a8adffaa0d45d25d5428991ca0ad8b7f7b4bb545
home-path:sha256:50c96c6cb9744204527d633e757cd45b26070f93c9215232e3c58c389106b392
home-path:sha256:5bc14364a8fba7a53ff3b604df5c49ab1bd506d149bb2aae47e7ef3c5361141b
home-path:sha256:2393ba5390d87afbfb407b1efd412ad98771cc8520fa5015b5d66cdba919b549
home-path:sha256:1a30551d47a439203d27730f211547ba1630f02f018b38db82d81f364f4e7dca
home-path:sha256:6d123ca46d6a9a149b43bf80c2c08f45549fc802f501f716e681db668b2e7aea
home-path:sha256:4e0ba2c448c78fa99e24ebf074ae956ce95d013f62cab71ce2d00e332bbc7181
home-path:sha256:707ce476a89405573dff17afadd7afcb65ea648a25008e9f37815e444b62ee39
home-path:sha256:b152fb3df6f5f4a24d1e855ed2055b348d1a2193d6794f09010710fbda5f3765
home-path:sha256:a70102a0af5ddd968fe3bf70c70df41f7334cf004b3afe4ba777a2b3a96384dd
home-path:sha256:a2ee50b137d7cc270a59edc71e329443876d25ca12ece88f28b11432c47bf376
home-path:sha256:6b044496f497ca1f740267c3868b749ca84043b6287294f00d75f36dd45d8455
home-path:sha256:500f621ea98aac12675b6af4e7565f3828db8db385afc3977555b8304dbfa8cd
home-path:sha256:7aa2168effb7e9c8131e43b3cf7bbdfda29312929cecc7e409bc890fca765456
home-path:sha256:884c1a97404215807dc7ca5066497607ccab6a3c63e1c5cfdc2612ed3a23d2db
home-path:sha256:8a4b50ad3f86378f7cfa00a7eff7194d41b42a5ca56d4c5776086af1aa36d5bf
home-path:sha256:06ad419f49862f4f0dcf80963ba094ce1ce54aa1facb203e3918dc3c27c298a2
home-path:sha256:3f05e59b15bffcb3ae3d3bfe68033c58d3f8edbac0e42abe7301adb9014d5618
home-path:sha256:d6592b70a828dc09f2cb99cd8e3a90536dabaab5daf05805577a0a23beae0103
home-path:sha256:25d9777c4c119221d29cef44a5dbd7096edfde5078df2a70a453f0c22131cf4f
home-path:sha256:7d9b5dba1b5b86fcc2fea6b894c8a38731bdd647c023d849c62bc7916d665605
home-path:sha256:191c46ec6779d3cb94b6b5cfb026a7337385a730a71636bf952d45e18f73b427
home-path:sha256:09963e2bec7cafdda88cc94d0ec1f1da6d2e0a4db6ce62c422be13910b9468a3
home-path:sha256:4a78aac97d29946fe9e95104074a79d93298e5dc0fde9c5ad4427cc9e07aa80f
home-path:sha256:9230019397697dd96bc3b5728a34f379050da7ea3ad955d0f79249d1a95e280a
home-path:sha256:8b18881f9ccae24402bd92d1e469c0ad553861b54722a9bf1090fd9b627f722a
home-path:sha256:32da4e42cbf567e8621c5e7ece99751909652ca12e18cc9ef1d8982572766770
home-path:sha256:c1f5365311229005bef83fc00d8944cb09f02e68396277f8f5a2b716c4dab870
home-path:sha256:3b813d820efce24a61a2e895e1a22a7146120927dcc603407dd0f20a905b085f
home-path:sha256:476a0f87bbd149f07f2cf3122b791085931b516a78067812caae4b206081c8ab
home-path:sha256:b219f553449f351db7158fb0cc1738ec5c232d378b8653d8c915e5ce3069a922
home-path:sha256:89d60fa3e908aa4cf862668888e4cd8d2ffb616de0f5314dbbc4d3dc58ec9807
home-path:sha256:c91a66f1b9bef6350ce8b16e6478d79bef9672a9307ae14ae1390c906474f5d0
home-path:sha256:4b6b74560872eb60d4531a769f897f14ce94407b1e344031467571b7b8c938de
home-path:sha256:e360dfff21e8b558b76bee89096af8ecefb18ebb26f1726b1b74012f178b03e4
home-path:sha256:d858f7d085051a983afbd4540ec4b3b33714f3332dba8dff8e29108b3dacf75e
home-path:sha256:b830c80b1d81c857c5bb150f7a3e76acb94d45c745588467f5e01c85ee7ead7d
home-path:sha256:c38d7cc3f73e8080c3d3ae722a8d58d110dd25e731621d7936ce971fa6903a3d
home-path:sha256:ddb651f3e5c504f93ee53816c70acc6012c1beb3502fd45d5c1239cc5e2690f0
home-path:sha256:d8e18039edb2e9ec90d19d791a3d77421eb7ddac8ec2979b4f5b3d747eb0677f
home-path:sha256:52a4fc8dc54fa09bbe8d70d70edd1c7f7ed3d3acdd673f6c137bdd2cb1b70a33
home-path:sha256:5735da72b410d59b984c926bb76ba730623cb1f63a5459e3aa673a1504af7f2b
home-path:sha256:d36879dccf1c31f7e91140f141d08d2c6cacde7408854df13f381580404d9d73
home-path:sha256:2047318de4f04310b013cd9d6484697a3498ba053ad6aa6c4d4438df0897311f
home-path:sha256:0e34adbf14a8bb6116469a9c21d79745a33dc4b225a17721251123491798baed
home-path:sha256:938a21530451388f186900d2ae1df4264f7d6e945589f7e7f2d79409cda3c316
home-path:sha256:2f199f69440d3f44c4a003dd790d557f50ac87c134d9d09fad0cf42f8f8b6090
home-path:sha256:22de8f2b5e74feabdb146ac588485f7c6989c597a068a1403a2f51181c1550c3
home-path:sha256:f73c9a9842ed3012df77892a95b2c5738896d463e12f9e36cba0fb7128376d29
home-path:sha256:bfec24d3c05108e50b307bc6b38e306d52399ee7aed8763c4ae14b8ef729909d
home-path:sha256:e92b58bfee640c97d38945333d31c573cd105f3fb9af6250e8c62cd4f3411937
home-path:sha256:999ac7d0a2aea8d1c9f25d72340503f5c2ee20bd553b56c91951fcc72589551a
home-path:sha256:42cf095ebdc68a1d8f315a27cd3b3f62b65251599018727de2db5d76f8042f4e
home-path:sha256:3b5ce62eb51409db38971da32f8941a1ee3236365b4099063655c97317262523
home-path:sha256:393a9b34bf84a8c54f3ec7902f3d7017f73d4de3edcb8d42502fc21d81d554b0
home-path:sha256:4b68436f668dd72c14e785320b040e3a9b521704219001562a075cdd5923f850
home-path:sha256:8df21cba197ce9532c722d089cafab07ca7c95faa8bf5a1f342e81a00e689147
home-path:sha256:4edfeefe48bd27e29437bd85095751350e8c685247bb20bb32cd3f2c564d651d
home-path:sha256:aee07f4b8c96bf831e1fe46cd0d771cbc4874815633ad326bd926440df82b067
home-path:sha256:9272e309ffeddb0a23897a7054a1a9cd14fc79017f9afec1908fc59ffccc013f
home-path:sha256:6406b82824265cab1e2e8e54b1d116bd711684c99aad151dde0f9b57ada2fcb1
home-path:sha256:426d39dd0169fa9507a2c94f69ee07a0447059ef6bb9d5713652ad7f757a2d40
home-path:sha256:456a4024b0075bec40b13a2ca601713fb682511266cd1f57424127daab03a726
home-path:sha256:a2fa16d4705c35052defdb55b3e1671af26c648b89cbe4a91dcf190c725dce91
home-path:sha256:1e23e7013409fbad8824dfa23a5bab01db38fee43c6f598407170884ff328ddc
home-path:sha256:87ee079523503efb304286ad3f95cf6acf01c44c4a04e1fb78fb088d7f3c9d85
home-path:sha256:8d949c15126fe7866c75cbc2428dfb86916428636edfb24fff8b46bb47b8fce4
home-path:sha256:d388f3dc3932eae88547812c53ae45e72bab5f7c5fec76df121ce8bc6ec96ddb
home-path:sha256:93c8a64bd18893d8f79fde6cbb2b5dec00f873847ce7b1ed6189ccb98942d5a4
home-path:sha256:6e4d2c28f7f57b339cee50a48f6e9083ed9ac9c7e28467854e68d59764572071
home-path:sha256:776d5117cbe54396d001484a61ee6cb364537dbf5738374878fa91c74d529580
home-path:sha256:3ea451cc4319e58b397478062d5c406dd7d5cbf19f6998ed11b7be4e19c9390a
home-path:sha256:929561f8bb1c4424324d38ee1cce701b9ea70f60dfcdba001825d67c73cbca02
home-path:sha256:e949f99786fdb27be99f0d760ba067633c42a23bf4c7d73045f0298ba0d33a9d
home-path:sha256:68ac101e8efa6b0e554710de41096d506445645963a214c53fd5fd2cbfc10eab
home-path:sha256:1fd408982271675b98a380edff62eceae64b2ed697f5d200b8501d955ee9738b
home-path:sha256:f0b498995842741303c41ead36e4c5d8cd65882395ae5661fd6f180cae00b1e5
home-path:sha256:9076e99325a61e86bdcb03628e7a8c7f9b2aae3efeb841fb122c2c1ed047e29a
home-path:sha256:1a44f5eedf56718d7ef6983034da444c962f2a7a400c0fe299fe14dd1765d17d
home-path:sha256:2cdfdedec0e633ed3d9787188ea9141ab3db02eaf4aba53fe5448482623044ba
home-path:sha256:c5cb1484d08edcdad9bab27ec533b5fa355569a7add71764d8f98f46c497f484
home-path:sha256:390654ebb6059d7d64d25e3b5397efbcf4542e39481e35439ffbf290547127fc
home-path:sha256:45fd8c6dcd3eb74a564fd7e113f3686520ab84725e4935947687c400aef9b9bc
home-path:sha256:e72f66232b98e32bd674e39e6bfa0a4c3a0262a66373f1db4cc7dbd87565a07a
home-path:sha256:30b16bac9eb33cce2ddb93b761d283bfa66b8a5783e0fa078c2ba42206873193
home-path:sha256:5eb5c1cfbfeb1d70a97c3c5e59382472f4e32646122505bb6edd4f28973a7a36
home-path:sha256:cb1ec243e80074a51e8924ec40b9857a9fd01f3b3982d2104dc3f52b5e344a35
home-path:sha256:2cb8ef0d79a9c6c10681fcfd8b9bb9d20e94fa97b4202172a159ca4bf32857d0
home-path:sha256:29ea2180dbfc603ef586e7f4ae0c9802536698c1de50a151ed2f87f74e45562c
home-path:sha256:d656a0f6bcbced9f2a808ea68ae7726cb60e5e91be700682d43ed47feede65e5
home-path:sha256:2f98970ae02cfa67d43af8884e8609f9851c2841f203324235427bce355bfe18
home-path:sha256:8fd36f0bb7c52847b9b0f5f774c5cc8d195c977e6cc3e82226665b9c6519f148
home-path:sha256:a9a1daa2bb3fbfa36a38bc26396a92353a19cd3b5dafffa52c37c43c7ee8a709
home-path:sha256:6fcf606715055e51cda4825fe8bdd75e84b597f5e517f6be537c2fd40ad20d5b
home-path:sha256:7b0380b2c4d00ef83b875c3706023840d4275448c69c3a15ba68f7ed502ef2ec
home-path:sha256:7ee8aeda4f1b3b7cfcb657c08e4e130646ccd31afd3745bd1602104a9b970e7c
home-path:sha256:c74a0495647b4a487393c8e052255e28b4d26486300ffe360ef219f238995672
home-path:sha256:b27cc4d55e68760b1af93086cb0dcaf74fbb7d911a14b61804d0b18670560c33
home-path:sha256:5cd81d89867d67e2747f2a23ab48487ffabc970f8b492bf595a9c7b93185b8b2
home-path:sha256:188eda987e54eb59f0a6030f050e76868a8bc1b1b366dd2164530456950ef987
home-path:sha256:a32be1ff0e5e30ad1d16c8233eef9a6a6e457d8f445e6d3dcea4df16261929dc
home-path:sha256:cebce1594d991ec6c55820e698c7a7a958b3ee6a7ccdeae02e062bf7088ec88b
home-path:sha256:d464ddd041b706c9652b197badc2fc8414acb41153149112f17519bee6357a1a
home-path:sha256:41705f4829f8359d4a7db2c2edefedc51c39faf07d22f1831cffbef0d221be7e
home-path:sha256:62400a8f9d29d71136728a94c46c6467e77eba9660a4eb360ed515f9a4efe1d4
home-path:sha256:110666be960c1635cf070b91813dfe9da24b218281a2bd242a74cc57cb260cc5
home-path:sha256:d210e82569124ce36a084afd4179c09ce8c9018306687e379c7a2d7c253bc935
home-path:sha256:57078fdb644abb3cf252c0d2ad4a0f42bad3afd6856cfc3b90141dd261873539
home-path:sha256:a4b89200b475144cfb997a5c2e50ed6a544dc6c45c566625391efdae131f1d6d
home-path:sha256:362b415731f05bb2e1c81b844416eccc5697f3f4b7eff5526648e4685e191f41
home-path:sha256:edee7fe24dc802a7cb1584f1e0d5eb2c13248a85621b169b73c02edac89b7b31
home-path:sha256:a5a9ae457a553c6e3b6f2b293e8951476bfc3c129c3ff172dfe0f817094a2c42
home-path:sha256:b2cd0b204380e3f9b43e3f2a2685e0cc219a38a5ecc667656568aaa1aff35481
home-path:sha256:1f5380fa54828ad3b930c278fe3a2d10d65cffaf3420822ee834fad34e9744fe
home-path:sha256:c7c8935bcf997c87d12b2a072b4bcd139520e36cafcd32c61418857692cff0cd
home-path:sha256:77b39fa0b00ad5e3655fa007afa9d50af20ee1687385f7b6cc148f035ea6a780
home-path:sha256:e387692255252ff961e9252bc559187bce45aab7e7c3b6acfb5f7a22a92d6be7
home-path:sha256:e013305032032dad56d819438d6ccd2a962a1285c2b8a1b5c43111b74b2a478e
home-path:sha256:73b91d743092b7f91f239ffd5081914c129cebee5598a3da8bdef30afc2a7649
home-path:sha256:3d430fc3068be2a21719b7170fbf3ef0126489a3dfed888f18f66537207794e3
home-path:sha256:f091f5f6015dfc2b7f3d07d961a90d4876a9d0f49329529d93270d8be069ce20
home-path:sha256:66115d530eae245097d3514f7eaf6266d76f5762f19c0951b5729ecb5bfbb6db
home-path:sha256:aea8d60e9e2a709ae8c967e773c0fdb29acf6be83a32fc79fa98a2f07198eb63
home-path:sha256:f50a290674bb9243e9f7cd4d1c5301aecf036ccfa8ce695bc3ed4365f9ea3a8a
home-path:sha256:b9464bfa00651dac0d4f9fd55811b89b9ac87c24980cf3c85267ff2166c105a3
home-path:sha256:643c6607fdb02ae402e645ffe28836b66980b69f73576cd1249c32da30ef7038
home-path:sha256:bfd758e81bc5b4dbfeb53c5345b8408851bdee280c874f2319beea8ea58ac7dd
home-path:sha256:77e70338891fbd6da9dc4cd8124e7ca2ee8fdad4b561ac48874efb3d1b54915c
home-path:sha256:69e9a8422a1ce7a5dde7dcb16e1b7803681f7fa12f6b3bee8c9cbd57c38607cf
home-path:sha256:4c141879d735c2148a2b16cf3eb9e124c1bf72d6bdbdd2f8725f6771901dd126
home-path:sha256:b5fd78c6c289dda2d7b761943c3aef6c0d6616948b5f4c56d6ea5c24cd45a200
home-path:sha256:4746229192fb5ab306decb0c63b14deee826c7bb19863cadbbdc1498a3b4770b
home-path:sha256:0e1b3c2c9333149c13b138d9d45cc6076c56c4dc7a43cece5b9ab65d8bc1e05c
home-path:sha256:2e04a295a6813705337c2e0742d33ea104fb28a423744baf2b1faebf5eab6c06
home-path:sha256:fc286c2e4c8ba83e262e917a9847cc313630100c09a9965badf8d6c1a14e8b7a
home-path:sha256:ac1c6926e83af2b0650a88e6de58a0c87c73efdf6ee1bf48d391f488ad02323c
home-path:sha256:9cc2f6a10d6afb9d7cdbb1a13ec930e0b422e3c72fcbc7cd78e9f603aa86d195
home-path:sha256:7f41157b39ba93be864ded6aae7da5aca6e119392ebe6ab74d6d90e166b6dc3e
home-path:sha256:6f669a5bc5cf65fa11f1ea968f3d4f26855b72714760dae4a61770c386eaa242
home-path:sha256:8a6e0aad87ad2f4674c89386b3cd6d2979968561526588e5e27d342a1d3c74d0
home-path:sha256:5c3e2e8b11c08f1109bdfe9bafb284cf4293e190551b3f474d9ad0db1235d257
home-path:sha256:6ec121b449706361d0b5d4fa04020bd5984c873681abc6cec84c0605830144a5
home-path:sha256:395a03f391795f3da10ffe0760ea5e6384996ebe4bb751b21157be8921163e8e
home-path:sha256:5fcd61bc9e9643881667a06694c21a177d28ddf1d3383803cb4e8b955f185dc3
home-path:sha256:57a6f1725e8d61822065481bd1c31542299b7c37eb51dbd2d54208ddddab96f9
home-path:sha256:576c030b49994e5650ad80539d500932ceb5b3501faccaa48bf8b18769b0bdfc
home-path:sha256:f511bb388d5ce1862a0d62523b37c4551e1fa7fd18b8cf5de382639bebf0165d
home-path:sha256:8af787ccf1163dee212d35db5ed77ccfc9fbcd17bdcf265e80dbd3ca07bd29af
home-path:sha256:efb51e3e2e0994fdc2d1f227ef0331977fb1b3ca1f1f8a605495362448b0c764
home-path:sha256:de337d4c291dff5fa40ec0cd110551b9f9ffe9a5915c35ead25a5ba28866923f
home-path:sha256:c623987ff7eb44fd558ee4b0886726f30ebe6b87795de5129e78f82e2a4f3cd8
home-path:sha256:10229aa17bcc75061794755950c5a62321638772abdc79a577833f20349d6018
home-path:sha256:85d33434730d4e824700f9d77133b94e1df30417251606d05f64759abdff3f1d
home-path:sha256:a81b7d87e711e8b46cd3eedaf45b91885cbaeb168df1836f23cd7534d507c32f
home-path:sha256:e2691abe5c040eeff6b02349f696c7655ee380242275f9af6d6c2cf9315970d7
home-path:sha256:efc7397901976cdb2cf29f8f56e74afc012d37bc646be60cdb564a543e1477bb
home-path:sha256:86d4d7b349091e1744516bddce5871f949e7a5ac77cb830c2e67a4ccfcaa8484
home-path:sha256:f1b3480129c9a4cd9d7ca03e93254d60d521275754fcc99a01df0a33b3e77797
home-path:sha256:1e485a892867c6a29cac71327221d9c0b4915d19c74df139b822000d34366ca0
home-path:sha256:f9db97dcc0420a5f64aff6d02af8227d3619ef00351722a0b07b5f13196adf78
home-path:sha256:c110f6ed70c22421264d248399f05cac1acc55db61a75c94a623ddb3250c7b56
home-path:sha256:ec8f806ff95b3c1bd47e910affdb9ec8b01f7d43c0cecd975081c6f939670c11
home-path:sha256:1ea3427fe49dca514a099297fb22b97e41b93fe8c1c7cd1e9cbadc02e2a69ae8
home-path:sha256:60cae7a8f9626e2013acd74190bce8b9e96374d9c7dd6f079eed18e9dfc66a5b
home-path:sha256:d54ab2ee1270cc3e1a9d6c1b4f2f158f1a5e7d22d272b602f64c2af744c634c8
home-path:sha256:55d59e5d68982665e4442ffdda3fdaa3628d7e41cf2cb00b5ea1eed7cf216753
```

The evidence-manifest.jsonl is the complete native-byte census of retained review scratch (excluding only the report/manifest/summary/seal self-reference set); SEAL-SHA256SUMS pins that manifest, this complete report, findings, summary and source/authority manifests. No original preparation or implementor record was overwritten. All source, test, scratch and external-browser writes are relinquished with this seal. No further attack is launched.

7. Structured findings (exactly the table above)

```findings
- file: crates/verify/ess-conformance/src/coverage_build.rs
  line: 219
  category: "contract-drift"
  severity: "blocker"
  verdict: "NEEDS-CHANGE"
  origin: "introduced"
  message: "Generated and authored coverage messages retain only Cause Display instead of the full original typed Refusal Display selected by the sealed D1 correspondence decision."
- file: crates/verify/ess-conformance/src/go/runtime.go
  line: 3693
  category: "contract-drift"
  severity: "warning"
  verdict: "CONFIRMED"
  origin: "introduced"
  message: "A strict skipped run over known complete suite/5 prints that legacy suite coverage is unknown even though its admitted inventory and emitted report both say complete_inventory."
```
