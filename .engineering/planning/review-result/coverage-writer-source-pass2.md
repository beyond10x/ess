---
format: aep.planning-md/2
id: review-result:coverage-writer-source-pass2
kind: review-result
status: active
title: 'Coverage writer final source pass: no implementation findings, view test scope disposition'
relations:
- reviews: story:review-conformance-coverage
revision: 1
---
unit: ESS coverage writer final source pass 2, 8560854dae5998a3b4f1b03abb60b75af4783893
verdict: nothing found
cases: executed 646→648, red 1
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1653 retained paths under home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea
needs-coordinator: record this red pass unchanged, then apply the explicit new-test scope disposition; root owns all later verification, integration and actual AEP correspondence

1. git --no-pager diff --stat

```text
```

The tracked diff is empty: only two untracked additive test files were written. Actual supplemental `git --no-pager diff --no-index --stat /dev/null <new-test>` output:

```text
 .../tests/coverage_writer_adversary_pass2.rs       | 218 +++++++++++++++++++++
 1 file changed, 218 insertions(+)
 .../tests/coverage_writer_adversary_pass2.rs       | 149 +++++++++++++++++++++
 1 file changed, 149 insertions(+)
```

The exact paths and final source hashes are:

```json
[
  {
    "path": "crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs",
    "sha256": "b083325230517b72ed7d90e70753d2552115453212aaec0719d287619e4cfc0c",
    "lines": 218
  },
  {
    "path": "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass2.rs",
    "sha256": "a1a51ded78d28fd5c3a180c3102b07f76d8550f5600d5e813dd912c19072b03f",
    "lines": 149
  }
]
```

This is attack 2 of 2 against the corrected clean bot freeze, base d2057ffb944455d0ef3a90ab7c5043ae70027289. No third attack or production repair was performed. All 1,103 inherited tracked files and their retained before snapshots match the freeze. Existing cases, original fixtures, diagnostics, documents, manifests, lockfiles and store bytes remain unchanged. Formatting/helper extraction changed only these new test files, with all assertions preserved.

2. Cases authored before execution, and their first actual results

The first test was authored before any test execution; no baseline suite was run first. The supplied corrected implementor baseline is 646 passed, zero failed/ignored, 62 summaries, exit 0 in 42.390921502 seconds, report a6c65b16dc7fc90b0f100bb144033c48ba51cdc766f6ff68cd6bf1b646abc1df. This report does not substitute the older implementation's 608/642 counts for that exact corrected baseline.

The CLI case `browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner` is red, but its final assertion is an overbroad internal-representation assumption, not an established implementation defect. The final assertion is still present at crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs:102. A real CLI invocation emits the Billing suite/5 paired browser bundle and Rust admits it. Test-authored loaded-suite inputs then place seven arbitrary object keys at twelve declared Node payload owners. The accepted public original-input/select APIs create a real parent/child chain; each negative changes one surviving nested __proto__.sentinel value and independently rehashes the exact changed inner bytes with sha256sum. Rust and actual Firefox admit the positive and refuse all twelve changed survivor definitions. All positive returned payloads preserve the seven keys, and the captured own-property booleans are true at every owner. Nine returned owners contain ordinary finite Number values; the three unused nested view-expectation owners contain numeric tokens. The test reaches all admission assertions before failing whole returned-object equality. Its final own-property assertion and concluding println are after that failure and do not execute; the booleans themselves are captured actual browser observations, separately read back without another runtime execution.

This fixture is a supported loaded-suite wire input. Its test-authored steps were not synthesized from the Billing model and are not an inventory-honesty or model-execution claim. Browser pairing checks declared identity under the existing finite projection; no actual producer expectation is derived from a new report.

The conformance case `final_merge_orders_sources_and_preserves_full_d1_diagnostics_through_empty_selection` is green. It compiles three distinct checked relative identities with identical original authored bytes as three separate admitted batches. Each batch actually accepts one candidate with no refusal. All six batch arrival orders use the public final merge, choose a/a.yaml as owner, and retain the exact two original legacy duplicate diagnostics, including header/help and relative names. The expectation comes from the original authored compiler supplied the independently sorted input set. An explicit empty selection then preserves every source/refusal record and both full messages through original carrier serialization and re-admission. Six internal permutations remain one Rust runner case.

The initial merge test command did not compile: Code::to_owned retained a Code where actual inventory stores String. It ran zero cases and establishes no semantic finding. The type-only correction uses the original Code Display via to_string; its initial source/log are preserved. Its next focused command compiled and passed one actual case before the full suite. Rustfmt changed layout only. The exact focused command receipts and complete outputs follow in actual order.

focused-first

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
    "coverage_writer_adversary_pass2",
    "browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner",
    "--",
    "--exact",
    "--nocapture"
  ],
  "cwd": "home-path:sha256:81a8aaad60388f74814838864c1b3182e25ef96fb5f2eac69a3110c015b31d04",
  "environment": {
    "PATH": "home-path:sha256:d972722bf5a8a5abf3ab363acc673b42b0d50eca552e59c1ad0efea4f8e3a23e",
    "RUSTC": "home-path:sha256:da7abda9718cfd0d0c1e74f56f34135d83b4a908741f60a13ad1ed963424c535",
    "RUSTDOC": "home-path:sha256:cfcca0fbc01edced245f4082ef3f4032907e9866e0d726bc4bd1861d26fbd53d",
    "TMPDIR": "home-path:sha256:92f8f1612ffcef2332a9e6288ffde9aa702f0ce8d0ba5ea3c473d6633073f9d7",
    "GOCACHE": "home-path:sha256:9647f19e1d4145b9a042080ad03b73cbb720546702ce729eb9992448c1969bf3",
    "GOMODCACHE": "home-path:sha256:be803e54d491c47a3476932da1e25dd63349228ddc27fe82f682c806e5400335",
    "CARGO_HOME": "home-path:sha256:d86081df2851a911b2fd0fae5ec8894c10b21569abe2ec6161e1223327a25796",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:83e5d9ec89326d2852192c7f9c8a1681164b38f653d6575da05bdcb64b47b220",
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
  "free_before": 24845074432,
  "started_epoch": 1788728263.7293901,
  "new_tests": {
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs": "fa4cd9c0edf6e7efc1579cbc0441f2e87e21073fc7c543acb71017b0cda0b5b9"
  },
  "exit": 101,
  "elapsed_seconds": 43.64390492497478,
  "free_after": 24764428288
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
   Compiling itoa v1.0.18
   Compiling serde_json v1.0.151
   Compiling cfg-if v1.0.4
   Compiling serde_derive v1.0.229
   Compiling typenum v1.20.1
   Compiling syn v2.0.119
   Compiling hybrid-array v0.4.14
   Compiling block-buffer v0.12.1
   Compiling crypto-common v0.2.2
   Compiling allocator-api2 v0.2.21
   Compiling equivalent v1.0.2
   Compiling foldhash v0.2.0
   Compiling const-oid v0.10.2
   Compiling hashbrown v0.17.1
   Compiling digest v0.11.3
   Compiling cpufeatures v0.3.1
   Compiling sha2 v0.11.0
   Compiling serde_derive_internals v0.29.1
   Compiling schemars v0.8.22
   Compiling thiserror v2.0.20
   Compiling schemars_derive v0.8.22
   Compiling indexmap v2.14.1
   Compiling thiserror-impl v2.0.20
   Compiling dyn-clone v1.0.20
   Compiling ryu v1.0.23
   Compiling unsafe-libyaml v0.2.11
   Compiling serde_yaml v0.9.34+deprecated
   Compiling autocfg v1.5.1
   Compiling num-traits v0.2.19
   Compiling ess-primitives v0.20.0 (home-path:sha256:023a7ec9c4baa6d1cc2b055c33ff4f284fda7425258e04b1e2d84a65bca2ff39)
   Compiling libc v0.2.189
   Compiling ess-domain v0.20.0 (home-path:sha256:5dc48ee32bdcfb6b6531fcede0e40e5bf2c6b1fee19aeff3b8b7224c54dffe12)
   Compiling num-integer v0.1.47
   Compiling zerocopy v0.8.56
   Compiling version_check v0.9.5
   Compiling pulldown-cmark v0.13.4
   Compiling getrandom v0.3.4
   Compiling heck v0.5.0
   Compiling ahash v0.8.12
   Compiling num-bigint v0.4.8
   Compiling regex-syntax v0.8.11
   Compiling ess-compiler v0.20.0 (home-path:sha256:3bc6067a43285b74514bb9af08638497ee79278b1a934119ee774460e105b42f)
   Compiling bitflags v2.13.1
   Compiling parking_lot_core v0.9.12
   Compiling unicase v2.9.0
   Compiling pulldown-cmark-escape v0.11.0
   Compiling ref-cast v1.0.27
   Compiling num-rational v0.4.2
   Compiling num-iter v0.1.46
   Compiling num-complex v0.4.6
   Compiling ref-cast-impl v1.0.27
   Compiling aho-corasick v1.1.5
   Compiling scopeguard v1.2.0
   Compiling once_cell v1.21.4
   Compiling smallvec v1.16.0
   Compiling utf8parse v0.2.2
   Compiling anstyle-parse v1.0.0
   Compiling lock_api v0.4.14
   Compiling regex-automata v0.4.18
   Compiling num v0.4.3
   Compiling ess-gen v0.20.0 (home-path:sha256:9becbd77f058900ac1137c29d4ab62344dce2d3bf8292c8d48ea36f6ab645cda)
   Compiling infra-domain v0.20.0 (home-path:sha256:17ff7adea08c8d7f520ef548c6782574d0845e3fea23333d4c9b2059eec0bc1a)
   Compiling bit-vec v0.8.0
   Compiling anstyle v1.0.14
   Compiling is_terminal_polyfill v1.70.2
   Compiling unicode-general-category v1.1.0
   Compiling borrow-or-share v0.2.4
   Compiling anstyle-query v1.1.5
   Compiling colorchoice v1.0.5
   Compiling anstream v1.0.0
   Compiling fluent-uri v0.4.1
   Compiling infra-compiler v0.20.0 (home-path:sha256:bed6271e7b917c3aab1b762a9e7a7f1ff2f9741e8573beb74f95f8deaa73043a)
   Compiling bit-set v0.8.0
   Compiling fraction v0.17.0
   Compiling parking_lot v0.12.5
   Compiling strum_macros v0.28.0
   Compiling clap_lex v1.1.0
   Compiling micromap v0.3.0
   Compiling strsim v0.11.1
   Compiling num-cmp v0.1.0
   Compiling vsimd v0.8.0
   Compiling outref v0.5.2
   Compiling bytecount v0.6.9
   Compiling percent-encoding v2.3.2
   Compiling jsonschema-value v0.52.1
   Compiling referencing v0.52.1
   Compiling uuid-simd v0.8.0
   Compiling clap_builder v4.6.6
   Compiling strum v0.28.0
   Compiling infra-analyze v0.20.0 (home-path:sha256:692408daeac51b82bcbca16da5b417c8027986bb97bba5454630d80dce1ecfef)
   Compiling fancy-regex v0.19.0
   Compiling regex v1.13.1
   Compiling jsonschema-regex v0.52.1
   Compiling clap_derive v4.6.4
   Compiling email_address v0.2.9
   Compiling data-encoding v2.11.1
   Compiling anyhow v1.0.104
   Compiling clap v4.6.6
   Compiling jsonschema v0.52.1
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
    Finished `test` profile [unoptimized] target(s) in 41.93s
     Running tests/coverage_writer_adversary_pass2.rs (target/debug/deps/coverage_writer_adversary_pass2-efcbea4be28fb826)

running 1 test

thread 'browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner' (1492118) panicked at crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs:114:62:
assertion `left == right` failed
  left: Object {"\0": Array [Null, Object {"raw": String("2.0")}], "__proto__": Object {"sentinel": String("retained")}, "constructor": Object {"prototype": Object {"raw": String("9.0")}}, "hasOwnProperty": Bool(false), "toString": String("ordinary data"), "\u{e000}": String("BMP key"), "😀": String("astral key")}
 right: Object {"\0": Array [Null, Number(2)], "__proto__": Object {"sentinel": String("retained")}, "constructor": Object {"prototype": Number(9)}, "hasOwnProperty": Bool(false), "toString": String("ordinary data"), "\u{e000}": String("BMP key"), "😀": String("astral key")}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner ... FAILED

failures:

failures:
    browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.69s

error: test failed, to rerun pass `-p ess-cli --test coverage_writer_adversary_pass2`
```

focused-final-merge

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
    "coverage_writer_adversary_pass2",
    "final_merge_orders_sources_and_preserves_full_d1_diagnostics_through_empty_selection",
    "--",
    "--exact",
    "--nocapture"
  ],
  "cwd": "home-path:sha256:81a8aaad60388f74814838864c1b3182e25ef96fb5f2eac69a3110c015b31d04",
  "environment": {
    "PATH": "home-path:sha256:d972722bf5a8a5abf3ab363acc673b42b0d50eca552e59c1ad0efea4f8e3a23e",
    "RUSTC": "home-path:sha256:da7abda9718cfd0d0c1e74f56f34135d83b4a908741f60a13ad1ed963424c535",
    "RUSTDOC": "home-path:sha256:cfcca0fbc01edced245f4082ef3f4032907e9866e0d726bc4bd1861d26fbd53d",
    "TMPDIR": "home-path:sha256:92f8f1612ffcef2332a9e6288ffde9aa702f0ce8d0ba5ea3c473d6633073f9d7",
    "GOCACHE": "home-path:sha256:9647f19e1d4145b9a042080ad03b73cbb720546702ce729eb9992448c1969bf3",
    "GOMODCACHE": "home-path:sha256:be803e54d491c47a3476932da1e25dd63349228ddc27fe82f682c806e5400335",
    "CARGO_HOME": "home-path:sha256:d86081df2851a911b2fd0fae5ec8894c10b21569abe2ec6161e1223327a25796",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:896f7b0108491041b75a587632359df18b1113949dd21a554a27c9344eaf7fb4",
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
  "free_before": 24419381248,
  "started_epoch": 1788728453.6593947,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass2.rs": "7af756d421db0349e124c2242bdd9206aa4418c26bbf873a802b43999f19aee6",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs": "fa4cd9c0edf6e7efc1579cbc0441f2e87e21073fc7c543acb71017b0cda0b5b9"
  },
  "exit": 101,
  "elapsed_seconds": 14.20105807390064,
  "free_after": 24333688832
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
error[E0277]: can't compare `(std::string::String, std::string::String, std::string::String)` with `(std::string::String, Code, std::string::String)`
  --> crates/verify/ess-conformance/tests/coverage_writer_adversary_pass2.rs:66:9
   |
66 |         assert_eq!(actual, expected, "full original diagnostic depends on sorted identities, not batch arrival");
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no implementation for `(std::string::String, std::string::String, std::string::String) == (std::string::String, Code, std::string::String)`
   |
   = help: the trait `PartialEq<(std::string::String, Code, std::string::String)>` is not implemented for `(std::string::String, std::string::String, std::string::String)`
   = help: the following other types implement trait `PartialEq<Rhs>`:
             ()
             (A, Z, Y, X, W, V, U, T)
             (B, A, Z, Y, X, W, V, U, T)
             (C, B, A, Z, Y, X, W, V, U, T)
             (D, C, B, A, Z, Y, X, W, V, U, T)
             (E, D, C, B, A, Z, Y, X, W, V, U, T)
             (T,)
             (U, T)
           and 5 others
   = note: required for `Vec<(std::string::String, std::string::String, std::string::String)>` to implement `PartialEq<Vec<(std::string::String, Code, std::string::String)>>`

For more information about this error, try `rustc --explain E0277`.
error: could not compile `ess-conformance` (test "coverage_writer_adversary_pass2") due to 1 previous error
```

focused-final-merge-compile-fix

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
    "coverage_writer_adversary_pass2",
    "final_merge_orders_sources_and_preserves_full_d1_diagnostics_through_empty_selection",
    "--",
    "--exact",
    "--nocapture"
  ],
  "cwd": "home-path:sha256:81a8aaad60388f74814838864c1b3182e25ef96fb5f2eac69a3110c015b31d04",
  "environment": {
    "PATH": "home-path:sha256:d972722bf5a8a5abf3ab363acc673b42b0d50eca552e59c1ad0efea4f8e3a23e",
    "RUSTC": "home-path:sha256:da7abda9718cfd0d0c1e74f56f34135d83b4a908741f60a13ad1ed963424c535",
    "RUSTDOC": "home-path:sha256:cfcca0fbc01edced245f4082ef3f4032907e9866e0d726bc4bd1861d26fbd53d",
    "TMPDIR": "home-path:sha256:92f8f1612ffcef2332a9e6288ffde9aa702f0ce8d0ba5ea3c473d6633073f9d7",
    "GOCACHE": "home-path:sha256:9647f19e1d4145b9a042080ad03b73cbb720546702ce729eb9992448c1969bf3",
    "GOMODCACHE": "home-path:sha256:be803e54d491c47a3476932da1e25dd63349228ddc27fe82f682c806e5400335",
    "CARGO_HOME": "home-path:sha256:d86081df2851a911b2fd0fae5ec8894c10b21569abe2ec6161e1223327a25796",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:c467f2a0ef99a7161140919bf111f7f93e9b60f7cce4045593588b481146b1f6",
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
  "free_before": 24276598784,
  "started_epoch": 1788728517.6352165,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass2.rs": "6a01f9fec24d736a52be89b6312d62154116e92dc219a24a90d198c1e2dca407",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs": "4b84774cc101084ec3f0776d61340352e94daaefe8af693ede7fea5418bd5787"
  },
  "exit": 0,
  "elapsed_seconds": 0.5200121640227735,
  "free_after": 24254984192
}
```

```text
   Compiling ess-conformance v0.20.0 (home-path:sha256:fb2af4db425baf24f833fe7f0be66e10d98c52642fdd023fe8c8d2f68266c344)
    Finished `test` profile [unoptimized] target(s) in 0.33s
     Running tests/coverage_writer_adversary_pass2.rs (target/debug/deps/coverage_writer_adversary_pass2-ad1f6363457dabc4)

running 1 test
six real final-merge permutations; two exact legacy duplicate diagnostics each; empty selection retains all original source/refusal records
test final_merge_orders_sources_and_preserves_full_d1_diagnostics_through_empty_selection ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

```

3. Complete package execution and required checks

Final command: cargo test --locked --offline -p ess-conformance -p ess-cli -p ess-diff --no-fail-fast. Final result: 647 passed, 1 failed, 0 ignored, 648 executed, 64 summaries, exit 101 in 46.61662236903794 seconds. The only red is the unchanged overbroad view-return assertion. All 646 inherited cases and the new final-merge case pass. This is not a green package verdict and not a production approval.

Three complete package executions are retained. The first is the first broader execution after both new cases were authored and focused. New-test-only lint fixes necessitated the following final executions: CLI variable naming and helper extraction, then a borrowed &str correction; the conformance test's recording helper was extracted when Clippy measured 102 lines against 100. None changes an assertion, fixture value, comparison or selected test. assertions-preserved.json compares every complete assertion macro before/after extraction. The final strict Clippy and final format checks both exit 0; all earlier lint refusals remain below. No source or test changed after package-suite-final2. No additional source attack followed.

Every lane's actual measured totals are below. Compile/lint failures have zero test summaries and are not counted as semantic red cases. Complete raw output for each lane follows; no failed attempt is replaced by a later successful one.

```json
[
  {
    "lane": "fmt-check-final",
    "summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "executed": 0,
    "exit": 0,
    "elapsed_seconds": 0.43341162300202996
  },
  {
    "lane": "fmt-check",
    "summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "executed": 0,
    "exit": 0,
    "elapsed_seconds": 0.44692356197629124
  },
  {
    "lane": "focused-final-merge-compile-fix",
    "summaries": 1,
    "passed": 1,
    "failed": 0,
    "ignored": 0,
    "executed": 1,
    "exit": 0,
    "elapsed_seconds": 0.5200121640227735
  },
  {
    "lane": "focused-final-merge",
    "summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "executed": 0,
    "exit": 101,
    "elapsed_seconds": 14.20105807390064
  },
  {
    "lane": "focused-first",
    "summaries": 1,
    "passed": 0,
    "failed": 1,
    "ignored": 0,
    "executed": 1,
    "exit": 101,
    "elapsed_seconds": 43.64390492497478
  },
  {
    "lane": "package-suite-final",
    "summaries": 64,
    "passed": 647,
    "failed": 1,
    "ignored": 0,
    "executed": 648,
    "exit": 101,
    "elapsed_seconds": 49.594036841066554
  },
  {
    "lane": "package-suite-final2",
    "summaries": 64,
    "passed": 647,
    "failed": 1,
    "ignored": 0,
    "executed": 648,
    "exit": 101,
    "elapsed_seconds": 46.61662236903794
  },
  {
    "lane": "package-suite",
    "summaries": 64,
    "passed": 647,
    "failed": 1,
    "ignored": 0,
    "executed": 648,
    "exit": 101,
    "elapsed_seconds": 65.76848476589657
  },
  {
    "lane": "strict-clippy-final",
    "summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "executed": 0,
    "exit": 101,
    "elapsed_seconds": 0.30629109195433557
  },
  {
    "lane": "strict-clippy-final2",
    "summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "executed": 0,
    "exit": 101,
    "elapsed_seconds": 4.321357293985784
  },
  {
    "lane": "strict-clippy-final3",
    "summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "executed": 0,
    "exit": 0,
    "elapsed_seconds": 2.4562247489811853
  },
  {
    "lane": "strict-clippy",
    "summaries": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "executed": 0,
    "exit": 101,
    "elapsed_seconds": 21.78004849201534
  }
]
```

package-suite

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
    "TMPDIR": "home-path:sha256:92f8f1612ffcef2332a9e6288ffde9aa702f0ce8d0ba5ea3c473d6633073f9d7",
    "GOCACHE": "home-path:sha256:9647f19e1d4145b9a042080ad03b73cbb720546702ce729eb9992448c1969bf3",
    "GOMODCACHE": "home-path:sha256:be803e54d491c47a3476932da1e25dd63349228ddc27fe82f682c806e5400335",
    "CARGO_HOME": "home-path:sha256:d86081df2851a911b2fd0fae5ec8894c10b21569abe2ec6161e1223327a25796",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:eaa95518f71e2f25118abbea2bff72f731701d46f6325ef5ce7ff392fcfd4714",
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
  "free_before": 24258678784,
  "started_epoch": 1788728547.5699656,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass2.rs": "6a01f9fec24d736a52be89b6312d62154116e92dc219a24a90d198c1e2dca407",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs": "4b84774cc101084ec3f0776d61340352e94daaefe8af693ede7fea5418bd5787"
  },
  "exit": 101,
  "elapsed_seconds": 65.76848476589657,
  "free_after": 23666298880
}
```

```text
   Compiling ess-diff v0.20.0 (home-path:sha256:5455152f24506d134276c0fa17104395c6cd66014ed2c8978ece31453da08d17)
   Compiling ess-cli v0.20.0 (home-path:sha256:5a10dd49c67cef5019797350b37afacc7816747d7c22f5c97b15b2438867815c)
   Compiling ess-conformance v0.20.0 (home-path:sha256:fb2af4db425baf24f833fe7f0be66e10d98c52642fdd023fe8c8d2f68266c344)
    Finished `test` profile [unoptimized] target(s) in 15.68s
     Running unittests src/main.rs (target/debug/deps/ess-63929f179d60f606)

running 12 tests
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test coverage::tests::suite5_pair_refusal_precedes_target_construction ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/authored_scenarios.rs (target/debug/deps/authored_scenarios-3ba618a94d75005d)

running 25 tests
test author_nested_only ... ok
test author_empty ... ok
test author_nonmatching_only ... ok
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test run_nonmatching_only ... ok
test ir_empty ... ok
test go_empty ... ok
test web_empty ... ok
test ir_nested_only ... ok
test ir_nonmatching_only ... ok
test go_nested_only ... ok
test web_nested_only ... ok
test run_nested_only ... ok
test run_empty ... ok
test go_nonmatching_only ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test web_nonmatching_only ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s

     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-fbc20393f78d5213)

running 9 tests
test selected_directory_symlink_reports_the_requested_path_and_stays_shallow ... ok
test non_utf8_matching_content_is_refused_before_any_output_write ... ok
test empty_selection_cannot_follow_or_replace_an_output_symlink ... ok
test lexical_selection_order_decides_the_first_read_error ... ok
test matching_broken_links_are_read_errors_even_beside_a_valid_source ... ok
test committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners ... ok
test omitted_scenarios_ignore_a_poisoned_working_directory_default ... ok
test empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner ... ok
test valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s

     Running tests/authored_site.rs (target/debug/deps/authored_site-1a420fe3beb09d76)

running 9 tests
test an_explicit_missing_front_page_is_an_error ... ok
test binary_downloads_are_not_silently_decoded ... ok
test asset_symlink_output_is_refused_before_any_page_changes ... ok
test publication_without_strict_mode_preserves_unpublished_links ... ok
test a_page_identity_can_itself_end_in_html ... ok
test strict_links_check_local_and_cross_page_fragments ... ok
test assets_cannot_collide_with_generated_output_or_escape_it ... ok
test selected_sources_resolve_after_relocation_with_queries_fragments_and_downloads ... ok
test undeclared_existing_and_escaping_targets_are_refused_with_source_lines ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-00fd33a384eed561)

running 1 test
test cli_composition_obeys_the_independently_authored_vectors ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running tests/binary64_count_adversary.rs (target/debug/deps/binary64_count_adversary-25ef58b4641736f8)

running 2 tests
test cli_binary64_model_and_original_suites_preserve_report_destinations_in_both_formats ... ok
test generated_go_binary64_shapes_refuse_before_factory_and_report_publication ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.25s

     Running tests/binary64_publication.rs (target/debug/deps/binary64_publication-ba03872d9e795d41)

running 1 test
test binary64_sparse_model_refuses_every_unsupported_publication_route ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-d76d1dd27f537ab9)

running 1 test
test binary64_publication_never_replaces_sources_or_partially_updates_a_library ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/command_surface.rs (target/debug/deps/command_surface-71a1dc5a6624aa01)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s

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

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.80s

     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-3af0e671176d1e0b)

running 2 tests
test generated_go_abnormal_unsupported_error_formatting_cannot_complete ... ok
test generated_go_admits_only_typed_predicate_paths_and_operator_envelopes ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.28s

     Running tests/coverage_browser.rs (target/debug/deps/coverage_browser-bf192040179e69f7)

running 4 tests
test retained_legacy_player_bytes_still_replay_in_actual_firefox ... ok
test actual_browser_admits_the_pair_before_creating_replay_state ... ok
test actual_browser_and_rust_refuse_every_closed_model_field_boundary ... ok
test actual_browser_checks_full_lineage_and_integer_metadata ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.64s

     Running tests/coverage_cli.rs (target/debug/deps/coverage_cli-994c933e9b49eced)

running 6 tests
test coverage_cli_refuses_binary64_model_before_each_new_production_surface ... ok
test coverage_cli_authored_roots_relocate_without_losing_exact_text_and_refuse_unrepresentable_paths ... ok
test explicit_suite5_cli_produces_exact_inventory_and_requires_report2_before_execution ... ok
test select_cli_preserves_all_parent_bytes_and_explicit_empty_selection ... ok
test impact_cli_requires_exact_complete_input_and_keeps_the_persisted_v3_shape ... ok
test generated_go_executes_the_admitted_coverage_inventory_and_preserves_pairing_defaults ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s

     Running tests/coverage_lineage.rs (target/debug/deps/coverage_lineage-a42eee64e165b553)

running 2 tests
test generated_go_checks_original_lineage_and_typed_defaults ... ok
test go_execution_adapts_only_selected_integer_fields_before_target_effects ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.46s

     Running tests/coverage_producers.rs (target/debug/deps/coverage_producers-1b45cfea40b4b957)

running 3 tests
test actual_rust_coverage_exports_match_the_independent_plan ... ok
test actual_coverage_producers_refuse_noninvoked_negative_clock_and_report1_without_output ... ok
test actual_go_coverage_exports_match_the_independent_plan ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.89s

     Running tests/coverage_writer_adversary_pass1.rs (target/debug/deps/coverage_writer_adversary_pass1-96fa9eb4cce59dee)

running 2 tests
test go_strict_diagnostic_does_not_label_known_suite5_inventory_as_legacy_unknown ... ok
test browser_refuses_a_command_name_with_a_final_line_feed_before_replay_state ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.62s

     Running tests/coverage_writer_adversary_pass2.rs (target/debug/deps/coverage_writer_adversary_pass2-efcbea4be28fb826)

running 1 test
test browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner ... FAILED

failures:

---- browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner stdout ----

thread 'browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner' (1549745) panicked at crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs:197:9:
assertion `left == right` failed
  left: Object {"\0": Array [Null, Object {"raw": String("2.0")}], "__proto__": Object {"sentinel": String("retained")}, "constructor": Object {"prototype": Object {"raw": String("9.0")}}, "hasOwnProperty": Bool(false), "toString": String("ordinary data"), "\u{e000}": String("BMP key"), "😀": String("astral key")}
 right: Object {"\0": Array [Null, Number(2)], "__proto__": Object {"sentinel": String("retained")}, "constructor": Object {"prototype": Number(9)}, "hasOwnProperty": Bool(false), "toString": String("ordinary data"), "\u{e000}": String("BMP key"), "😀": String("astral key")}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.76s

error: test failed, to rerun pass `-p ess-cli --test coverage_writer_adversary_pass2`
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-ac70ba28528e59d8)

running 4 tests
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_rust_cli_writes ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_web_cli_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-d5a347d9ae9103cd)

running 13 tests
test count_go_predicate_admission_matches_rust_leaf_grammar ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test count_report_skip_only_is_inconclusive_without_actual_failures ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test count_retained_runtime_preserves_legacy_behavior_and_does_not_gain_version_checks ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test count_go_empty_selection_is_inconclusive_and_clock_conversion_is_checked ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test count_go_actual_producers_keep_skip_error_and_teardown_categories ... ok
test count_go_refusals_precede_targets_and_incomplete_runs_never_publish ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.29s

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
test output_symlinks_and_hardlinks_are_refused_without_mutation ... ok
test generated_paths_cannot_replace_canonical_source_inputs_or_follow_links ... ok
test stale_bundle_missing_source_and_unknown_dispatch_refuse ... ok
test qualified_go_base64_pattern_is_published_without_decoding_the_value ... ok
test output_cannot_replace_any_declared_input ... ok
test explicit_binary64_inputs_run_through_the_cli_without_weakening_version_one ... ok
test checked_recipe_and_run_are_deterministic_with_json_only_stdout ... ok
test positional_cli_refuses_unused_branches_before_publication ... ok
test generation_checks_refuse_before_creating_or_changing_destinations ... ok
test failed_check_or_execution_preserves_output_and_does_not_emit_a_result ... ok
test raw_json_capture_runs_before_schema_and_keeps_failure_output_untouched ... ok
test generated_libraries_match_the_api_and_drift_check_never_repairs_files ... ok
test positional_cli_prepares_text_and_refuses_before_publication ... ok
test model_sources_are_compiled_pinned_and_protected_by_the_cli ... ok
test binary64_cli_keeps_numeric_identity_and_emits_checked_format_five ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.96s

     Running tests/normalization_positional_adversary.rs (target/debug/deps/normalization_positional_adversary-406279f2f9d35352)

running 2 tests
test cli_runtime_grammar_controls_preserve_existing_output ... ok
test cli_invalid_policy_blocks_all_publication_and_valid_metadata_is_drift_checked ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

     Running tests/normalization_typescript.rs (target/debug/deps/normalization_typescript-5542aa1874e06ad7)

running 7 tests
test unqualified_profile_refuses_with_source_pointer_and_no_partial_files ... ok
test model_inputs_are_recompiled_pinned_and_protected_before_typescript_generation ... ok
test retained_recipe_and_bundle_inputs_cannot_be_overwritten ... ok
test typescript_cli_emits_an_accounted_executable_package ... ok
test module_and_late_path_conflicts_refuse_before_any_publication ... ok
test generated_file_and_parent_links_refuse_without_touching_their_destinations ... ok
test every_planned_file_participates_in_read_only_drift_checks ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s

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

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/openapi_adversary_pass1.rs (target/debug/deps/openapi_adversary_pass1-d2bcefe19c39a1dd)

running 2 tests
test cli_rejects_duplicate_keys_and_tampered_accounting_before_output ... ok
test cli_rejects_unknown_unit_fields_before_any_projection_output ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/openapi_adversary_pass2.rs (target/debug/deps/openapi_adversary_pass2-ae258705b1349988)

running 1 test
test same_path_projection_refusal_preserves_the_unadmitted_input ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/output_containment.rs (target/debug/deps/output_containment-5c56b28ecc91698b)

running 19 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-8deef286ecfcabd1)

running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

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

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

     Running tests/schema_registry_identity_adversary.rs (target/debug/deps/schema_registry_identity_adversary-cf3f1a6623400628)

running 6 tests
test registry_admission_and_selected_typescript_projection_have_distinct_boundaries ... ok
test inlining_idless_generated_payloads_breaks_their_original_root_references ... ok
test decoded_duplicate_ids_are_collisions_and_filenames_supply_no_identity ... ok
test an_unselected_invalid_resource_blocks_both_documented_pairs ... ok
test envelope_definitions_cannot_shadow_either_payload_resource_root ... ok
test the_selected_schema_checks_its_selector_and_nonobject_payload_as_one_instance ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s

     Running tests/target_failure.rs (target/debug/deps/target_failure-7d9cc8e18982b595)

running 1 test
test fatal_synthesis_preserves_destinations_and_has_a_typed_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

     Running unittests src/lib.rs (target/debug/deps/ess_conformance-5e50d807cc4deaf5)

running 70 tests
test counts::tests::exact_unsigned_scalar_vectors_do_not_use_binary64 ... ok
test counts::tests::payload_number_and_utf8_canonical_profile_is_frozen_separately_from_scalars ... ok
test decision::tests::exactly_one_reason_says_another_candidate_would_help ... ok
test decision::tests::a_decision_reads_its_two_other_cases_as_neither_satisfied_nor_the_other ... ok
test evidence::tests::a_standalone_report_carries_every_field_an_adapter_needs ... ok
test evidence::tests::report_readers_do_not_guess_a_producer_from_status_vocabulary ... ok
test decision::tests::a_refusal_renders_the_predicate_the_command_and_every_reason ... ok
test evidence::tests::report_readers_refuse_more_nonpasses_than_executed_scenarios ... ok
test faulty::tests::a_fault_is_injected_into_the_system_that_declares_what_it_breaks ... ok
test faulty::tests::every_fault_says_what_it_is_and_where_it_goes ... ok
test evidence::tests::unknown_report_fields_are_refused ... ok
test faulty::tests::no_two_faults_claim_the_same_scenario ... ok
test faulty::tests::only_the_two_faults_the_boundary_cannot_express_are_injected_in_the_implementation ... ok
test evidence::tests::the_closed_report_round_trips_with_identical_canonical_bytes ... ok
test input::tests::a_primitive_refuses_a_node_of_the_wrong_shape_rather_than_coercing_it ... ok
test input::tests::shape_errors_render_one_per_line_and_name_the_input_root_by_name ... ok
test input::tests::every_primitive_projects_to_the_one_fact_value_that_can_hold_it ... ok
test go::tests::every_go_file_is_in_the_package_the_readme_names ... ok
test report::tests::a_quoted_input_reads_as_the_call_that_was_made ... ok
test report::tests::a_diagnostic_answers_all_five_of_the_questions_a_failure_has_to_answer ... ok
test report::tests::a_scenario_status_is_the_strongest_of_its_checks_and_a_contradiction_outranks_everything ... ok
test report::tests::an_unsupported_scenario_makes_the_run_fail_rather_than_look_like_a_pass ... ok
test runner::tests::a_count_with_neither_bound_is_a_suite_defect_and_not_a_satisfied_assertion ... ok
test go::tests::the_runner_is_a_constant_and_only_the_suite_moves ... ok
test evidence::tests::report_readers_refuse_nonpass_count_and_list_disagreement ... ok
test runner::tests::a_position_in_a_view_that_declares_no_order_is_a_suite_defect ... ok
test runner::tests::a_declared_order_is_checked_on_adjacent_rows_and_the_next_key_breaks_a_tie ... ok
test runner::tests::a_count_is_the_half_of_an_ordering_claim_that_says_the_rows_were_there ... ok
test runner::tests::a_position_names_both_ends_and_a_row_that_is_not_there_is_not_a_match ... ok
test evidence::tests::report_readers_preserve_go_producer_bytes_and_historical_nonpass_counts ... ok
test runner::tests::a_ranking_key_a_row_does_not_publish_is_undecidable_rather_than_out_of_order ... ok
test report::tests::every_check_code_has_a_distinct_name_and_a_rule_sentence ... ok
test runner::tests::a_nested_row_binds_the_paths_a_predicate_spells ... ok
test runner::tests::a_predicate_a_row_cannot_answer_is_reported_rather_than_retried ... ok
test runner::tests::an_order_over_fewer_than_two_rows_holds_and_does_not_double_as_a_non_emptiness_claim ... ok
test runner::tests::a_view_that_holds_nothing_does_not_satisfy_an_invariant_by_being_empty ... ok
test runner::tests::the_runners_clock_advances_on_every_read_so_a_deadline_can_bound_anything ... ok
test runner::tests::ids_come_from_the_suite_and_from_nothing_ambient ... ok
test runner::tests::an_empty_field_set_means_a_row_exists_and_not_that_anything_will_do ... ok
test scenario::tests::a_declared_leaf_admits_what_its_type_admits_and_absence_only_where_the_type_permits_it ... ok
test scenario::tests::a_purpose_is_one_line_and_says_something ... ok
test scenario::tests::a_scenario_id_names_the_construct_it_exercises_rather_than_its_position ... ok
test scenario::tests::a_semantic_reference_renders_the_way_the_design_writes_one ... ok
test evidence::tests::report_readers_refuse_malformed_and_unsupported_suite_versions ... ok
test scenario::tests::a_suite_format_from_a_later_build_is_refused_rather_than_guessed ... ok
test scenario::tests::a_transition_ref_refuses_a_name_no_lifecycle_can_declare ... ok
test evidence::tests::report_readers_refuse_unknown_report_formats ... ok
test scenario::tests::a_suite_refuses_a_second_scenario_under_one_id ... ok
test scenario::tests::an_invariant_scenario_is_keyed_by_the_entity_and_the_branch_and_never_by_a_position ... ok
test scenario::tests::a_payload_shape_round_trips_through_the_form_a_suite_is_stored_in ... ok
test scenario::tests::a_scenario_id_that_names_no_construct_is_refused ... ok
test scenario::tests::every_binding_aspect_is_in_the_list_that_is_walked_to_produce_them ... ok
test scenario::tests::two_scenarios_about_the_same_thing_in_the_same_way_are_one_id ... ok
test scenario::tests::the_ids_of_a_suite_sort_the_way_a_reader_sorts_the_file ... ok
test synthesize::tests::a_refusal_names_the_construct_the_code_and_the_repair ... ok
test scenario::tests::every_scenario_id_reads_back_from_the_form_a_report_prints ... ok
test synthesize::tests::every_refusal_carries_a_distinct_code_in_one_family ... ok
test web::tests::no_comparison_sits_in_a_text_node_mustache ... ok
test witness::tests::a_text_witness_is_its_own_path_so_two_fields_of_one_type_never_agree ... ok
test witness::tests::an_enum_offers_every_variant_it_declares_and_the_first_one_only_once ... ok
test web::tests::the_page_calls_nothing_the_player_does_not_return ... ok
test witness::tests::an_integer_leaf_is_never_offered_a_fractional_candidate ... ok
test witness::tests::the_alternatives_for_a_number_are_the_guards_own_literals_either_side ... ok
test witness::tests::the_candidate_count_is_bounded_however_many_fields_a_guard_reads ... ok
test witness::tests::two_uuid_witnesses_differ_and_neither_moves_when_a_third_field_appears ... ok
test evidence::tests::report_readers_refuse_nonpass_entries_without_a_known_nonpass_status ... ok
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
test a_field_the_surface_does_not_declare_is_refused_by_name ... ok
test a_document_that_is_not_one_is_refused_rather_than_read_as_an_empty_scenario ... ok
test a_reference_where_the_suite_compares_a_value_it_carries_is_refused ... ok
test a_format_this_build_does_not_implement_is_refused_before_anything_is_read ... ok
test a_command_the_model_does_not_declare_is_refused_by_name ... ok
test a_domain_the_model_does_not_declare_is_refused_by_name ... ok
test a_declared_field_nothing_supplies_is_refused_by_name ... ok
test a_bounded_negative_that_forbids_no_event_is_refused ... ok
test a_halt_claimed_of_a_listing_with_no_declared_order_is_refused_by_the_code_that_already_says_so ... ok
test a_halt_compiles_to_a_step_of_its_own_and_not_to_a_claim_about_rows ... ok
test a_positional_claim_takes_the_order_from_the_view_rather_than_from_the_author ... ok
test a_halt_stated_beside_another_claim_is_two_assertions_filed_as_one ... ok
test a_predicate_reading_something_the_view_does_not_publish_is_refused ... ok
test a_halt_after_no_rows_at_all_is_refused_rather_than_compiled ... ok
test a_claim_the_timelines_own_instants_contradict_is_refused ... ok
test a_position_in_a_view_that_declares_no_order_is_refused ... ok
test a_state_the_lifecycle_does_not_declare_is_refused_as_a_state_and_not_as_a_variant ... ok
test a_scenario_that_runs_nothing_is_refused_rather_than_counted_as_a_check ... ok
test a_scenario_compiles_to_the_id_the_domain_and_the_name_make ... ok
test a_window_of_no_seconds_is_refused_rather_than_compiled_into_a_check_that_cannot_fail ... ok
test a_view_the_model_does_not_declare_is_refused_by_name ... ok
test a_timeline_whose_instants_do_not_ascend_is_refused ... ok
test an_act_cannot_open_a_window_at_its_own_instant ... ok
test a_value_the_declared_type_does_not_admit_is_refused_where_it_sits ... ok
test a_value_read_off_an_event_nothing_required_is_refused ... ok
test a_window_that_states_other_than_one_bound_is_refused ... ok
test an_entity_the_model_does_not_declare_is_refused_by_name ... ok
test an_elapsed_claim_compiles_to_the_four_steps_that_carry_it_and_they_come_before_the_act ... ok
test an_outcome_the_command_does_not_declare_is_refused_with_the_ones_it_does ... ok
test a_window_measured_from_an_instant_nothing_marked_is_refused_with_the_ones_that_are ... ok
test an_actor_the_model_does_not_declare_is_refused_by_name ... ok
test an_actor_the_specification_does_not_grant_the_command_is_refused ... ok
test an_event_the_model_does_not_declare_is_refused_by_name ... ok
test an_instance_named_before_anything_binds_it_is_refused ... ok
test an_error_the_model_does_not_declare_is_refused_by_name ... ok
test an_assertion_that_states_other_than_one_claim_is_refused ... ok
test authored_aggregate_presence_keeps_026_and_valid_scalar_reads_keep_the_predicate ... ok
test an_instance_bound_to_a_field_that_cannot_hold_an_identity_is_refused ... ok
test an_instance_the_arrangement_does_not_declare_is_refused_by_name ... ok
test the_steps_are_the_vocabulary_a_generated_scenario_already_uses ... ok
test one_name_for_two_instants_is_refused_rather_than_read_as_the_later_one ... ok
test the_order_the_files_are_handed_over_in_does_not_reach_the_result ... ok
test two_files_naming_one_scenario_are_refused_rather_than_one_displacing_the_other ... ok
test the_committed_billing_suite_holds_the_authored_scenario_beside_the_generated_ones ... ok
test authored_predicate_operand_errors_have_their_own_refusal ... ok
test every_cause_is_reachable_from_a_document ... ok
test two_compilations_of_one_file_produce_identical_bytes ... ok
test independently_successful_authored_batches_are_refused_only_at_final_merge ... ok
test coverage_builder_retains_authored_duplicate_ownership_and_original_source_bytes ... ok
test paired_browser_emission_retains_original_input_and_checks_the_actual_model ... ok
test rejected_authored_candidate_needs_are_proved_independently_of_an_outside_survivor ... ok

test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

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
test d1_authored_inventory_preserves_full_original_typed_refusal_rendering ... ok
test d1_generated_inventory_preserves_full_original_typed_refusal_rendering ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/coverage_writer_adversary_pass2.rs (target/debug/deps/coverage_writer_adversary_pass2-cf711d0e33f85f1e)

running 1 test
test final_merge_orders_sources_and_preserves_full_d1_diagnostics_through_empty_selection ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

     Running tests/elapsed.rs (target/debug/deps/elapsed-b5312b79e8e5d6a8)

running 7 tests
test a_deadline_the_target_ran_past_fails_the_within_claim ... ok
test a_window_opened_at_an_instant_nothing_marked_is_a_suite_defect_and_not_a_failed_implementation ... ok
test a_target_that_holds_the_window_and_reports_it_passes ... ok
test a_target_whose_clock_never_moves_fails_rather_than_being_read_as_having_waited ... ok
test a_target_with_no_clock_reports_unsupported_and_the_run_fails ... ok
test an_event_published_inside_the_window_fails_the_bounded_negative_and_nothing_else ... ok
test two_runs_over_one_window_produce_byte_identical_reports ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/execution.rs (target/debug/deps/execution-ae712a3af979f44f)

running 12 tests
test an_eventual_assertion_asks_again_within_a_deadline_and_never_sleeps ... ok
test an_eventual_view_is_read_again_and_a_read_your_writes_view_is_not ... ok
test a_view_assertion_names_the_instance_the_scenario_created_rather_than_any_row ... ok
test a_view_answered_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test a_target_that_cannot_expose_an_observation_fails_the_run_rather_than_skipping_it ... ok
test a_value_of_the_wrong_declared_type_is_caught_by_the_same_check_as_a_missing_one ... ok
test an_event_missing_a_field_it_declares_is_named_leaf_by_leaf_rather_than_reported_as_absent ... ok
test every_scenario_checked_something_and_no_family_of_them_was_silently_empty ... ok
test every_scenario_the_billing_specification_obliges_passes_against_the_reference_implementation ... ok
test a_read_your_writes_view_is_not_quietly_read_at_current_when_no_token_came_back ... ok
test a_scenario_whose_input_no_longer_reaches_its_branch_fails_with_a_diagnostic_naming_the_defect ... ok
test two_runs_of_one_suite_against_one_target_produce_byte_identical_reports ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

     Running tests/faults.rs (target/debug/deps/faults-592913acd22499fb)

running 11 tests
test a_fault_nothing_catches_is_recorded_rather_than_quietly_dropped ... ok
test the_diagnostic_of_a_caught_fault_names_the_defect_rather_than_reporting_that_something_broke ... ok
test a_wrong_mapping_is_invisible_to_a_target_that_cannot_show_its_invocations ... ok
test dropping_one_binding_leaves_the_other_two_green ... ok
test every_fault_that_could_be_a_boundary_perturbation_is_one ... ok
test the_widest_blast_radius_is_scenarios_that_could_not_be_arranged_rather_than_extra_verdicts ... ok
test each_specification_is_passed_in_full_by_the_implementation_written_from_it ... ok
test two_runs_against_one_faulty_target_produce_byte_identical_reports ... ok
test a_faults_blast_radius_is_accounted_for ... ok
test each_fault_fails_the_scenario_that_exists_to_catch_it ... ok
test a_fault_does_not_simply_break_everything ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.82s

     Running tests/halt.rs (target/debug/deps/halt-e9156dff15c9e526)

running 7 tests
test a_halt_of_an_eventual_listing_is_asked_again_while_the_projection_catches_up ... ok
test a_target_whose_producer_stops_when_the_reader_does_passes ... ok
test a_listing_that_ran_out_before_the_reader_stopped_it_is_not_a_halt ... ok
test a_target_that_cannot_read_a_row_at_a_time_reports_unsupported_and_the_run_fails ... ok
test a_target_that_reads_the_whole_listing_fails_rather_than_being_read_as_having_stopped ... ok
test retrying_does_not_rescue_a_producer_that_never_stops ... ok
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
test a_suite_parses_from_text_alone_without_an_ir ... ok
test the_scan_for_a_clock_finds_one_and_does_not_find_a_word_that_merely_ends_in_a_banned_token ... ok
test a_count_and_a_position_read_back_as_what_a_runner_in_another_language_must_read ... ok
test the_steps_a_binding_and_an_invariant_need_survive_being_read_back_from_text ... ok
test sparse_models_cannot_publish_an_empty_success_for_binary64 ... ok
test every_scenario_id_the_billing_model_can_produce_reads_back ... ok
test the_step_vocabulary_expresses_the_worked_example_from_section_ten ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test a_suite_serialised_in_one_process_resolves_in_another ... ok
test the_suite_records_the_same_model_digest_the_projections_do ... ok
test inserting_one_outcome_re_keys_nothing_around_it ... ok
test the_dependency_set_names_a_type_no_derived_from_would_have_mentioned ... ok
test the_scenario_ids_appear_in_the_file_in_the_order_a_sorted_key_list_would_be ... ok
test typed_binary64_suites_refuse_serialization_emission_and_target_effects ... ok
test serialising_a_suite_twice_produces_byte_identical_json ... ok

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
test a_binding_that_retries_forces_one_failure_and_still_requires_the_consequence ... ok
test a_binding_mapping_names_the_source_the_document_wrote_and_not_its_same_typed_sibling ... ok
test a_binding_that_drops_its_failures_refuses_that_check_and_names_the_reason ... ok
test a_read_your_writes_view_filled_by_the_command_that_ran_is_asserted_to_hold_a_row ... ok
test an_entity_nothing_creates_cannot_be_acted_on_and_says_so ... ok
test a_binding_flow_is_proved_through_the_event_the_invoked_command_publishes ... ok
test a_move_is_observed_through_the_view_the_state_it_left_is_filtered_on ... ok
test a_scenario_that_moves_an_instance_names_the_one_an_earlier_step_created ... ok
test a_declared_error_is_asserted_by_name_and_never_by_an_invented_payload ... ok
test a_declared_order_is_asserted_against_two_rows_the_scenario_arranged_itself ... ok
test a_move_that_is_illegal_in_a_state_is_attempted_with_the_input_that_would_have_worked ... ok
test an_order_the_specification_cannot_put_two_rows_under_is_refused_and_not_asserted ... ok
test a_value_objects_own_invariants_are_read_at_every_field_position_a_view_holds_one ... ok
test an_undecidable_guard_refuses_and_does_not_spend_the_candidate_budget ... ok
test a_view_the_entity_has_not_reached_yet_is_asserted_to_exclude_the_instance_by_name ... ok
test a_value_object_nothing_observable_holds_keeps_a_refusal_naming_what_would_close_it ... ok
test a_synthesised_count_is_a_floor_the_scenario_arranged_and_never_a_ceiling ... ok
test an_actor_is_named_only_where_the_specification_grants_the_command ... ok
test an_at_least_once_binding_delivers_the_event_twice_and_requires_no_count ... ok
test a_view_that_does_not_hold_the_instance_yet_is_not_asked_about_its_invariants ... ok
test a_binding_that_escalates_requires_the_event_the_escalation_declares ... ok
test an_invariant_over_a_field_no_view_publishes_refuses_rather_than_being_dropped ... ok
test a_suite_for_one_component_holds_only_what_that_component_realises ... ok
test a_whole_system_suite_does_not_mention_a_component ... ok
test a_view_is_asserted_in_the_block_its_own_consistency_decides ... ok
test an_event_assertion_carries_the_declared_shape_and_exactly_the_values_the_payload_determines ... ok
test an_invariant_is_asserted_against_every_view_that_publishes_what_it_reads ... ok
test a_synthesised_suite_survives_being_written_and_read_back ... ok
test an_outcome_that_updates_an_instance_acts_on_one_the_scenario_created ... ok
test an_illegal_move_requires_the_branch_and_the_declared_error_rather_than_merely_failing ... ok
test every_declared_outcome_is_either_a_scenario_or_a_named_refusal_or_asserted_by_the_state_family ... ok
test every_declared_transition_has_a_scenario_that_proves_it_can_occur ... ok
test every_command_names_an_instance_an_earlier_step_of_the_same_scenario_bound ... ok
test an_outcome_no_input_decides_is_reached_by_injection_and_by_nothing_else ... ok
test the_failure_control_is_armed_after_the_arrangement_and_before_the_command_that_triggers_it ... ok
test the_input_a_scenario_sends_is_re_decided_against_the_guard_it_claims_to_reach ... ok
test the_refusal_branch_asserts_that_no_event_the_specification_declares_occurred ... ok
test the_dependency_set_names_the_types_the_scenario_is_made_of ... ok
test every_clause_of_every_binding_is_either_a_scenario_or_a_named_refusal ... ok
test each_example_synthesises_the_families_its_specification_declares ... ok
test synthesising_the_same_specification_twice_produces_byte_identical_output ... ok
test coverage_builder_records_the_complete_generated_inventory_and_component_omissions ... ok
test coverage_all_missing_invariants_keep_null_survivors_and_component_proofs_stay_conservative ... ok
test canonical_expression_compatibility_fixtures ... ok
test coverage_missing_lifecycle_and_view_checks_remain_beside_actual_passing_results ... ok

test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s

     Running tests/witness.rs (target/debug/deps/witness-9e1a2f8f8747eefd)

running 28 tests
test a_newtype_is_transparent_so_a_deep_path_reaches_through_it_without_a_segment ... ok
test a_refuted_guard_carries_the_leaf_and_the_value_that_refuted_it ... ok
test an_absent_optional_binds_nothing_rather_than_binding_a_default ... ok
test a_candidate_carrying_a_field_no_type_declares_is_refused ... ok
test a_list_a_map_and_a_union_bind_no_fact_in_the_current_projection ... ok
test a_disjunction_one_of_whose_branches_holds_is_satisfied_despite_an_undecidable_branch ... ok
test a_path_landing_on_an_aggregate_is_unevaluable_by_construction ... ok
test a_scalar_of_the_wrong_shape_is_refused_rather_than_coerced ... ok
test an_absent_optional_is_unevaluable_but_says_a_candidate_could_repair_it ... ok
test a_newtype_is_transparent_when_a_path_is_resolved_as_well_as_when_it_is_projected ... ok
test a_path_into_a_list_or_a_union_names_the_aggregate_rather_than_the_missing_element ... ok
test a_refusal_names_the_predicate_the_command_and_the_path ... ok
test a_candidate_input_projects_one_fact_per_scalar_leaf ... ok
test a_conjunction_of_two_undecidable_leaves_reports_both ... ok
test a_candidate_missing_a_required_field_is_refused_before_any_guard_is_read ... ok
test legal_collection_cardinality_is_not_currently_projected ... ok
test equality_over_two_texts_is_decided_even_though_ordering_them_is_not ... ok
test only_an_absent_value_says_another_candidate_would_help ... ok
test a_long_recursive_read_validates_beyond_the_projection_limit ... ok
test expression_search_limits_do_not_define_type_correctness ... ok
test resolved_adapter_keeps_semantics_separate_from_collection_projection ... ok
test the_normative_shape_of_guard_is_decidable_for_both_signs ... ok
test malformed_declarations_refuse_early_and_direct_bad_reads_remain_unknown ... ok
test ordering_two_texts_is_unevaluable_because_an_ess_specification_declares_no_scale ... ok
test ordering_across_two_types_is_unevaluable_not_false ... ok
test otherwise_and_external_are_not_guards_over_the_input ... ok
test the_same_text_ordering_is_decidable_once_a_scale_contains_both_values ... ok
test unclassified_is_a_drift_alarm_and_no_enumerated_source_trips_it ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/lib.rs (target/debug/deps/ess_diff-06167d7f60ffb115)

running 9 tests
test change::tests::a_change_id_names_its_category_subject_subtype_and_member_in_that_order ... ok
test change::tests::a_change_with_no_member_renders_three_parts_rather_than_a_trailing_slash ... ok
test delta::tests::a_delta_puts_its_changes_in_canonical_order_however_they_arrive ... ok
test change::tests::only_a_grant_and_a_variant_decide_a_direction ... ok
test impact::tests::a_whole_answer_absorbs_a_narrowing_whichever_way_round_they_are_joined ... ok
test impact::tests::an_unfollowed_file_is_not_an_artifact_that_owes_regeneration ... ok
test change::tests::the_canonical_order_is_the_category_order_and_not_the_alphabet ... ok
test impact::tests::a_change_to_the_specification_itself_owes_the_whole_suite ... ok
test impact::tests::a_suite_resting_on_a_construct_the_graph_has_no_node_for_owes_the_whole_suite ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/artifacts.rs (target/debug/deps/artifacts-fe289ed3b5544f1f)

running 13 tests
test an_artifact_whose_slice_nothing_reached_is_absent_from_the_answer ... ok
test a_grant_change_owes_the_documents_that_read_grants_and_not_the_ones_that_do_not ... ok
test the_six_change_delta_owes_a_strict_subset_of_the_artifacts ... ok
test a_change_to_the_system_header_owes_every_artifact ... ok
test an_owed_artifacts_path_explains_the_membership_hop_by_hop ... ok
test whole_model_artifacts_are_owed_by_any_change_at_all ... ok
test the_two_predicate_edits_narrow_the_artifacts_differently_and_both_subsets_are_named ... ok
test the_artifacts_the_currency_changes_reach_are_owed_and_named ... ok
test a_committed_tree_is_answered_for_fail_closed_file_by_file ... ok
test the_artifact_answer_is_byte_identical_between_runs ... ok
test a_committed_artifact_with_a_false_contract_digest_is_owed_as_a_false_claim ... ok
test review_whole_model_hashes_and_index_bytes_remain_frozen ... ok
test review_legacy_slice_stamps_are_owed_even_when_raw_hashes_match ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s

     Running tests/canonical.rs (target/debug/deps/canonical-ce09a7a45878a366)

running 20 tests
test a_binding_still_has_one_delivery_a_document_can_write ... ok
test every_change_variant_has_something_to_say_for_itself ... ok
test a_change_is_spelt_the_same_way_in_its_id_and_in_the_document ... ok
test review_freeze_legacy_delta_bytes ... ok
test review_version_admission_refuses_new_vocabulary_in_legacy_envelopes ... ok
test no_source_file_in_the_diff_engine_reads_a_clock_or_an_unordered_map ... ok
test no_source_file_in_the_diff_engine_calls_an_ir_handle_accessor ... ok
test a_system_still_has_no_naming_a_document_can_set ... ok
test a_delta_whose_id_was_edited_is_refused ... ok
test every_change_in_a_delta_has_its_own_id ... ok
test the_changes_are_written_in_the_category_order_and_not_the_alphabet ... ok
test a_delta_written_in_a_format_this_build_does_not_read_is_refused ... ok
test a_delta_whose_changes_are_out_of_order_is_refused ... ok
test a_document_with_six_defects_reports_six ... ok
test a_delta_this_build_wrote_is_read_back_without_complaint ... ok
test a_delta_whose_relation_was_edited_is_refused ... ok
test review_new_default_delta_format_is_version_two ... ok
test canonical_json_ends_in_a_newline ... ok
test a_delta_naming_two_systems_is_refused_on_the_way_in_as_well ... ok
test diffing_the_same_pair_twice_produces_byte_identical_json ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/families.rs (target/debug/deps/families-8952d48644da49b5)

running 69 tests
test a_component_accepting_a_new_command_is_changed_and_not_widened ... ok
test a_bindings_failure_policy_is_compared ... ok
test a_binding_invoking_a_different_command_moves_its_mapping_with_it ... ok
test a_filter_respaced_is_the_same_predicate_and_no_change ... ok
test a_component_that_no_longer_publishes_an_event_is_reported ... ok
test a_mapping_filled_from_somewhere_else_is_reported_with_both_sources ... ok
test a_construct_moving_between_files_is_not_a_change ... ok
test a_guard_respaced_is_the_same_predicate_and_no_change ... ok
test a_type_that_became_a_different_kind_of_thing_is_reported_as_that_and_nothing_else ... ok
test a_filter_that_contains_different_instances_is_changed_with_no_direction ... ok
test a_binding_reacting_to_a_different_event_is_reported ... ok
test a_bindings_naming_is_compared_key_by_key ... ok
test a_command_added_is_one_change ... ok
test a_payload_declaration_arriving_is_a_payload_change ... ok
test a_new_transition_arrives_with_the_outcome_that_takes_it ... ok
test a_binding_added_is_one_change ... ok
test a_filter_removed_reads_as_containing_every_instance ... ok
test a_struct_field_that_changed_type_is_reported ... ok
test a_commands_naming_is_compared_key_by_key ... ok
test a_newtype_that_wraps_something_else_is_reported ... ok
test an_entity_added_arrives_with_its_synthesised_state_enum_and_nothing_is_diffed_inside ... ok
test an_entity_fields_naming_is_compared_key_by_key ... ok
test a_view_projecting_a_different_entity_is_a_source_change ... ok
test a_union_that_is_tagged_by_another_field_is_reported ... ok
test an_entity_field_replaced_is_removed_and_added_and_never_a_rename ... ok
test a_union_variant_that_carries_something_else_is_not_a_variant_removed_and_added ... ok
test a_types_own_invariants_are_reported_as_different_and_never_as_stronger ... ok
test a_views_naming_is_compared_key_by_key ... ok
test a_view_exposing_a_new_field_is_reported_with_the_type_it_carries ... ok
test a_views_consistency_promise_is_compared_and_not_classified ... ok
test a_union_gaining_a_variant_widens_it_just_as_an_enum_does ... ok
test an_actor_declared_with_no_grants_at_all_is_still_a_change_to_report ... ok
test a_view_fields_naming_is_compared_key_by_key ... ok
test an_entitys_naming_is_compared_key_by_key ... ok
test a_view_added_is_one_change ... ok
test an_entity_field_that_changed_type_is_reported ... ok
test an_event_field_that_changed_type_is_reported ... ok
test an_error_that_gained_a_field_is_reported_with_the_type_it_carries ... ok
test an_event_renamed_is_reported_as_removed_and_added_and_never_as_a_rename ... ok
test reordering_a_commands_input_is_reported_once ... ok
test reordering_a_commands_outcomes_is_a_real_change ... ok
test an_outcomes_summary_is_compared ... ok
test an_input_added_is_reported_with_the_type_it_carries ... ok
test an_identitys_display_name_and_summary_are_compared ... ok
test an_input_fields_naming_is_compared_key_by_key ... ok
test an_events_wire_name_moving_is_not_the_event_moving ... ok
test an_input_that_changed_type_is_reported ... ok
test reordering_an_entitys_fields_is_reported_once ... ok
test reordering_a_views_fields_is_reported_once ... ok
test an_outcome_added_is_one_change_and_claims_no_direction ... ok
test reordering_an_enums_variants_is_reported_without_claiming_a_direction ... ok
test an_invariant_statement_reworded_without_moving_the_predicate_is_still_a_change ... ok
test renaming_an_entitys_identity_is_the_one_rename_this_crate_reports ... ok
test reordering_an_event_payload_is_reported_once_and_not_as_a_field_change ... ok
test the_paragraph_saying_what_the_system_is_is_compared ... ok
test the_error_a_branch_reports_is_compared ... ok
test the_specifications_version_moving_is_reported_and_is_not_the_identity ... ok
test what_an_error_tells_the_caller_is_compared ... ok
test what_an_outcome_emits_is_compared_in_order ... ok
test writing_out_a_naming_default_is_not_a_change ... ok
test review_view_parameter_naming_is_compared_without_a_filter_edit ... ok
test review_outcome_refusal_is_independent_of_its_error ... ok
test review_reach_is_a_change_without_an_unrelated_surface_edit ... ok
test review_cli_top_level_grouped_views_and_binary_are_changes ... ok
test review_view_ranking_is_compared_without_a_filter_edit ... ok
test review_outcome_sets_are_independent_of_event_payload ... ok
test review_residual_refs_cannot_hide_beside_a_classified_change ... ok
test review_unclassified_transition_order_cannot_hide_beside_a_classified_edit ... ok
test review_relation_cardinality_name_and_removal_are_changes ... ok

test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.61s

     Running tests/graph.rs (target/debug/deps/graph-746a5caf03f88bea)

running 10 tests
test correction2_row_shape_is_a_distinct_dependency_and_survives_graph_union ... ok
test a_component_is_reached_through_what_it_accepts_and_publishes ... ok
test a_type_is_reached_through_the_declarations_that_hold_it_and_not_by_name ... ok
test the_graph_records_the_reference_an_author_wrote_and_not_its_reverse ... ok
test a_closure_over_the_whole_model_terminates_and_stays_inside_it ... ok
test building_the_same_graph_twice_produces_the_same_edges_in_the_same_order ... ok
test correction2_network_exposure_matches_actual_routes_and_owned_domains ... ok
test review_cli_views_and_parameter_types_are_forward_slice_dependencies ... ok
test review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union ... ok
test every_relation_in_the_vocabulary_is_minted_by_a_specification_this_repository_ships ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/impact.rs (target/debug/deps/impact-1ac5d4ffb424fb86)

running 17 tests
test a_suite_produced_from_the_later_revision_is_refused_rather_than_narrowed ... ok
test two_specifications_of_different_systems_are_refused_here_too ... ok
test raw_legacy_impact_cannot_discard_a_coverage_inventory ... ok
test a_suite_whose_contract_digest_its_model_does_not_compute_is_refused ... ok
test a_suite_for_another_system_is_refused ... ok
test a_narrowed_answer_never_reports_more_scenarios_than_the_suite_holds ... ok
test a_variant_removed_from_an_enum_reaches_the_entity_that_holds_it_transitively ... ok
test every_scenario_resting_directly_on_a_changed_construct_is_owed_again ... ok
test an_edited_outcome_guard_owes_every_scenario_because_every_scenario_creates_through_it ... ok
test a_suite_resting_on_a_construct_no_graph_has_a_node_for_owes_the_whole_suite ... ok
test an_edited_entity_invariant_owes_every_scenario_that_rests_on_the_entity_and_no_other ... ok
test the_suite_the_fixture_obliges_is_ten_scenarios_and_the_delta_is_six_changes ... ok
test taking_a_grant_from_an_actor_owes_only_the_scenarios_that_act_as_that_actor ... ok
test analysing_the_same_pair_twice_produces_byte_identical_json ... ok
test a_change_in_a_family_the_delta_still_does_not_compare_owes_the_whole_suite ... ok
test a_domains_naming_moving_owes_the_whole_suite_because_no_family_compares_a_domain ... ok
test coverage_impact_keeps_exact_selection_context_out_of_persisted_impact3 ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s

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
test reusable_row_type_belongs_to_the_view_slice_it_supplies ... ok
test ranking_precedence_survives_the_checked_delta_roundtrip ... ok
test switching_equal_row_shapes_retains_independent_residual_coverage ... ok
test reusable_row_invariant_change_reaches_its_openapi_artifact ... ok
test served_view_change_reaches_its_openapi_artifact ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/revision_pair.rs (target/debug/deps/revision_pair-ba032edad7953c23)

running 11 tests
test two_different_systems_are_refused_rather_than_reported_as_a_rewrite ... ok
test adding_an_enum_variant_widens_the_type_that_accepts_it ... ok
test taking_a_command_from_an_actor_narrows_what_the_system_permits ... ok
test nothing_the_after_revision_only_rewrote_reaches_the_delta ... ok
test the_fixture_pair_differs_by_exactly_six_changes ... ok
test rewriting_an_entitys_invariant_is_changed_and_quotes_both_statements ... ok
test a_revision_compared_with_itself_reports_nothing ... ok
test removing_an_enum_variant_narrows_the_type_that_accepted_it ... ok
test rewriting_an_outcomes_when_is_changed_and_renders_both_guards_canonically ... ok
test granting_a_command_to_an_actor_widens_what_the_system_permits ... ok
test the_delta_survives_being_written_and_read_back ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

   Doc-tests ess_conformance

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_diff

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p ess-cli --test coverage_writer_adversary_pass2`
```

fmt-check

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
    "TMPDIR": "home-path:sha256:92f8f1612ffcef2332a9e6288ffde9aa702f0ce8d0ba5ea3c473d6633073f9d7",
    "GOCACHE": "home-path:sha256:9647f19e1d4145b9a042080ad03b73cbb720546702ce729eb9992448c1969bf3",
    "GOMODCACHE": "home-path:sha256:be803e54d491c47a3476932da1e25dd63349228ddc27fe82f682c806e5400335",
    "CARGO_HOME": "home-path:sha256:d86081df2851a911b2fd0fae5ec8894c10b21569abe2ec6161e1223327a25796",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:ac2b8aed015b0a5fdd2dcc912fb82ebda3001a311b21fb03bf8b2d15b829d9f1",
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
  "free_before": 24001081344,
  "started_epoch": 1788728584.4683602,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass2.rs": "6a01f9fec24d736a52be89b6312d62154116e92dc219a24a90d198c1e2dca407",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs": "4b84774cc101084ec3f0776d61340352e94daaefe8af693ede7fea5418bd5787"
  },
  "exit": 0,
  "elapsed_seconds": 0.44692356197629124,
  "free_after": 24001093632
}
```

```text
```

strict-clippy

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
    "TMPDIR": "home-path:sha256:92f8f1612ffcef2332a9e6288ffde9aa702f0ce8d0ba5ea3c473d6633073f9d7",
    "GOCACHE": "home-path:sha256:9647f19e1d4145b9a042080ad03b73cbb720546702ce729eb9992448c1969bf3",
    "GOMODCACHE": "home-path:sha256:be803e54d491c47a3476932da1e25dd63349228ddc27fe82f682c806e5400335",
    "CARGO_HOME": "home-path:sha256:d86081df2851a911b2fd0fae5ec8894c10b21569abe2ec6161e1223327a25796",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:efd3a5c8ffc9d8d6d14bcbdede620d42b21aa9e8f6be17fe1d9442d82448c3e3",
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
  "free_before": 23665016832,
  "started_epoch": 1788728632.5637054,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass2.rs": "6a01f9fec24d736a52be89b6312d62154116e92dc219a24a90d198c1e2dca407",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs": "4b84774cc101084ec3f0776d61340352e94daaefe8af693ede7fea5418bd5787"
  },
  "exit": 101,
  "elapsed_seconds": 21.78004849201534,
  "free_after": 23663308800
}
```

```text
    Checking serde_core v1.0.229
    Checking memchr v2.8.3
    Checking cfg-if v1.0.4
    Checking itoa v1.0.18
    Checking zmij v1.0.23
    Checking typenum v1.20.1
    Checking serde v1.0.229
    Checking serde_json v1.0.151
    Checking hybrid-array v0.4.14
    Checking block-buffer v0.12.1
    Checking crypto-common v0.2.2
    Checking equivalent v1.0.2
    Checking foldhash v0.2.0
    Checking allocator-api2 v0.2.21
    Checking const-oid v0.10.2
    Checking hashbrown v0.17.1
    Checking digest v0.11.3
    Checking cpufeatures v0.3.1
    Checking sha2 v0.11.0
    Checking indexmap v2.14.1
    Checking unsafe-libyaml v0.2.11
    Checking dyn-clone v1.0.20
    Checking ryu v1.0.23
    Checking schemars v0.8.22
    Checking serde_yaml v0.9.34+deprecated
    Checking thiserror v2.0.20
    Checking ess-primitives v0.20.0 (home-path:sha256:023a7ec9c4baa6d1cc2b055c33ff4f284fda7425258e04b1e2d84a65bca2ff39)
    Checking pulldown-cmark-escape v0.11.0
    Checking bitflags v2.13.1
    Checking unicase v2.9.0
    Checking ess-domain v0.20.0 (home-path:sha256:5dc48ee32bdcfb6b6531fcede0e40e5bf2c6b1fee19aeff3b8b7224c54dffe12)
    Checking pulldown-cmark v0.13.4
    Checking num-traits v0.2.19
    Checking libc v0.2.189
    Checking ess-compiler v0.20.0 (home-path:sha256:3bc6067a43285b74514bb9af08638497ee79278b1a934119ee774460e105b42f)
    Checking num-integer v0.1.47
    Checking num-bigint v0.4.8
    Checking ess-gen v0.20.0 (home-path:sha256:9becbd77f058900ac1137c29d4ab62344dce2d3bf8292c8d48ea36f6ab645cda)
    Checking regex-syntax v0.8.11
    Checking ess-conformance v0.20.0 (home-path:sha256:fb2af4db425baf24f833fe7f0be66e10d98c52642fdd023fe8c8d2f68266c344)
    Checking num-rational v0.4.2
    Checking zerocopy v0.8.56
    Checking ess-diff v0.20.0 (home-path:sha256:5455152f24506d134276c0fa17104395c6cd66014ed2c8978ece31453da08d17)
    Checking getrandom v0.3.4
    Checking num-iter v0.1.46
    Checking num-complex v0.4.6
    Checking aho-corasick v1.1.5
    Checking once_cell v1.21.4
    Checking utf8parse v0.2.2
    Checking smallvec v1.16.0
    Checking scopeguard v1.2.0
    Checking lock_api v0.4.14
    Checking parking_lot_core v0.9.12
    Checking regex-automata v0.4.18
    Checking anstyle-parse v1.0.0
    Checking ahash v0.8.12
    Checking ref-cast v1.0.27
    Checking num v0.4.3
    Checking infra-domain v0.20.0 (home-path:sha256:17ff7adea08c8d7f520ef548c6782574d0845e3fea23333d4c9b2059eec0bc1a)
    Checking anstyle v1.0.14
    Checking anstyle-query v1.1.5
    Checking is_terminal_polyfill v1.70.2
    Checking borrow-or-share v0.2.4
    Checking bit-vec v0.8.0
    Checking colorchoice v1.0.5
    Checking anstream v1.0.0
    Checking bit-set v0.8.0
    Checking fluent-uri v0.4.1
    Checking infra-compiler v0.20.0 (home-path:sha256:bed6271e7b917c3aab1b762a9e7a7f1ff2f9741e8573beb74f95f8deaa73043a)
    Checking fraction v0.17.0
    Checking parking_lot v0.12.5
    Checking vsimd v0.8.0
    Checking micromap v0.3.0
    Checking bytecount v0.6.9
    Checking percent-encoding v2.3.2
    Checking outref v0.5.2
    Checking strsim v0.11.1
    Checking num-cmp v0.1.0
    Checking clap_lex v1.1.0
    Checking jsonschema-value v0.52.1
    Checking clap_builder v4.6.6
    Checking uuid-simd v0.8.0
    Checking referencing v0.52.1
    Checking strum v0.28.0
    Checking infra-analyze v0.20.0 (home-path:sha256:692408daeac51b82bcbca16da5b417c8027986bb97bba5454630d80dce1ecfef)
    Checking fancy-regex v0.19.0
    Checking unicode-general-category v1.1.0
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
    Checking schema-contract v0.20.0 (home-path:sha256:59e491e7ffb494ae9905baec320f21cf3d49d97493d328be25c7ea7896369370)
    Checking ess-kubernetes v0.20.0 (home-path:sha256:b48f74815ced76f6c132664a9490e73fa3e7f91a4c397d97387871f45cb03a64)
    Checking anyhow v1.0.104
    Checking ess-synth v0.20.0 (home-path:sha256:86d0c66b43ae445f8298081b67d4624cc72d3379d7cbebd68899127289ffa9c9)
    Checking ess-composition v0.20.0 (home-path:sha256:cf9c6e2be3ba3f0b1c2d4f47b62ab1102fe586f5d38602cdde999e25648dcc7a)
    Checking ess-openapi v0.20.0 (home-path:sha256:bb891c3627776b8febf75959df9c028f38f33b1327e40dc25af05b9a4235c053)
    Checking ess-cli v0.20.0 (home-path:sha256:5a10dd49c67cef5019797350b37afacc7816747d7c22f5c97b15b2438867815c)
error: binding's name is too similar to existing binding
  --> crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs:43:13
   |
43 |     let mut suite: Value =
   |             ^^^^^
   |
note: existing binding defined here
  --> crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs:21:9
   |
21 |     let site = evidence.join("site");
   |         ^^^^
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#similar_names
   = note: `-D clippy::similar-names` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::similar_names)]`

error: this function has too many lines (187/100)
  --> crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs:14:1
   |
14 | fn browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner() {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_lines
   = note: `-D clippy::too-many-lines` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::too_many_lines)]`

error: could not compile `ess-cli` (test "coverage_writer_adversary_pass2") due to 2 previous errors
warning: build failed, waiting for other jobs to finish...
```

strict-clippy-final

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
    "TMPDIR": "home-path:sha256:92f8f1612ffcef2332a9e6288ffde9aa702f0ce8d0ba5ea3c473d6633073f9d7",
    "GOCACHE": "home-path:sha256:9647f19e1d4145b9a042080ad03b73cbb720546702ce729eb9992448c1969bf3",
    "GOMODCACHE": "home-path:sha256:be803e54d491c47a3476932da1e25dd63349228ddc27fe82f682c806e5400335",
    "CARGO_HOME": "home-path:sha256:d86081df2851a911b2fd0fae5ec8894c10b21569abe2ec6161e1223327a25796",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:7ce0ed1643ad0bcfee3c42a8eaf6ea4d6bcade1ae7be9b32e6310753497e9636",
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
  "free_before": 22901809152,
  "started_epoch": 1788728760.9152524,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass2.rs": "6a01f9fec24d736a52be89b6312d62154116e92dc219a24a90d198c1e2dca407",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs": "620776b39800e105bf6a1bf7466f68bc43f2226d78b16ccc9c7c75c28e9e4235"
  },
  "exit": 101,
  "elapsed_seconds": 0.30629109195433557,
  "free_after": 22901792768
}
```

```text
    Checking ess-diff v0.20.0 (home-path:sha256:5455152f24506d134276c0fa17104395c6cd66014ed2c8978ece31453da08d17)
    Checking ess-cli v0.20.0 (home-path:sha256:5a10dd49c67cef5019797350b37afacc7816747d7c22f5c97b15b2438867815c)
error: the borrowed expression implements the required traits
   --> crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs:193:28
    |
193 |     fs::write(&child_path, &original_child).unwrap();
    |                            ^^^^^^^^^^^^^^^ help: change this to: `original_child`
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#needless_borrows_for_generic_args
    = note: `-D clippy::needless-borrows-for-generic-args` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::needless_borrows_for_generic_args)]`

error: could not compile `ess-cli` (test "coverage_writer_adversary_pass2") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```

strict-clippy-final2

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
    "TMPDIR": "home-path:sha256:92f8f1612ffcef2332a9e6288ffde9aa702f0ce8d0ba5ea3c473d6633073f9d7",
    "GOCACHE": "home-path:sha256:9647f19e1d4145b9a042080ad03b73cbb720546702ce729eb9992448c1969bf3",
    "GOMODCACHE": "home-path:sha256:be803e54d491c47a3476932da1e25dd63349228ddc27fe82f682c806e5400335",
    "CARGO_HOME": "home-path:sha256:d86081df2851a911b2fd0fae5ec8894c10b21569abe2ec6161e1223327a25796",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:aa9ea1371f9827ff084451618068a7bcd3d20e535634b328cde4dcbf73eb0d53",
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
  "free_before": 22900781056,
  "started_epoch": 1788728794.3796105,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass2.rs": "6a01f9fec24d736a52be89b6312d62154116e92dc219a24a90d198c1e2dca407",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs": "b083325230517b72ed7d90e70753d2552115453212aaec0719d287619e4cfc0c"
  },
  "exit": 101,
  "elapsed_seconds": 4.321357293985784,
  "free_after": 22904254464
}
```

```text
    Checking ess-diff v0.20.0 (home-path:sha256:5455152f24506d134276c0fa17104395c6cd66014ed2c8978ece31453da08d17)
    Checking ess-cli v0.20.0 (home-path:sha256:5a10dd49c67cef5019797350b37afacc7816747d7c22f5c97b15b2438867815c)
    Checking ess-conformance v0.20.0 (home-path:sha256:fb2af4db425baf24f833fe7f0be66e10d98c52642fdd023fe8c8d2f68266c344)
error: this function has too many lines (102/100)
  --> crates/verify/ess-conformance/tests/coverage_writer_adversary_pass2.rs:42:1
   |
42 | fn final_merge_orders_sources_and_preserves_full_d1_diagnostics_through_empty_selection() {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#too_many_lines
   = note: `-D clippy::too-many-lines` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::too_many_lines)]`

error: could not compile `ess-conformance` (test "coverage_writer_adversary_pass2") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```

package-suite-final

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
    "TMPDIR": "home-path:sha256:92f8f1612ffcef2332a9e6288ffde9aa702f0ce8d0ba5ea3c473d6633073f9d7",
    "GOCACHE": "home-path:sha256:9647f19e1d4145b9a042080ad03b73cbb720546702ce729eb9992448c1969bf3",
    "GOMODCACHE": "home-path:sha256:be803e54d491c47a3476932da1e25dd63349228ddc27fe82f682c806e5400335",
    "CARGO_HOME": "home-path:sha256:d86081df2851a911b2fd0fae5ec8894c10b21569abe2ec6161e1223327a25796",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:6d093a0d4b64ba139d8085c21a13c04901345bfec6b6ef48e0c4638f1b4fe55d",
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
  "free_before": 22900686848,
  "started_epoch": 1788728795.5679724,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass2.rs": "6a01f9fec24d736a52be89b6312d62154116e92dc219a24a90d198c1e2dca407",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs": "b083325230517b72ed7d90e70753d2552115453212aaec0719d287619e4cfc0c"
  },
  "exit": 101,
  "elapsed_seconds": 49.594036841066554,
  "free_after": 22312243200
}
```

```text
    Blocking waiting for file lock on build directory
   Compiling ess-cli v0.20.0 (home-path:sha256:5a10dd49c67cef5019797350b37afacc7816747d7c22f5c97b15b2438867815c)
    Finished `test` profile [unoptimized] target(s) in 3.44s
     Running unittests src/main.rs (target/debug/deps/ess-63929f179d60f606)

running 12 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test coverage::tests::suite5_pair_refusal_precedes_target_construction ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
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
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test author_empty ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test run_empty ... ok
test ir_nested_only ... ok
test run_nested_only ... ok
test go_nested_only ... ok
test go_nonmatching_only ... ok
test web_empty ... ok
test ir_empty ... ok
test go_empty ... ok
test web_nonmatching_only ... ok
test run_nonmatching_only ... ok
test ir_nonmatching_only ... ok
test web_nested_only ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s

     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-fbc20393f78d5213)

running 9 tests
test selected_directory_symlink_reports_the_requested_path_and_stays_shallow ... ok
test non_utf8_matching_content_is_refused_before_any_output_write ... ok
test empty_selection_cannot_follow_or_replace_an_output_symlink ... ok
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
test publication_without_strict_mode_preserves_unpublished_links ... ok
test a_page_identity_can_itself_end_in_html ... ok
test strict_links_check_local_and_cross_page_fragments ... ok
test assets_cannot_collide_with_generated_output_or_escape_it ... ok
test selected_sources_resolve_after_relocation_with_queries_fragments_and_downloads ... ok
test undeclared_existing_and_escaping_targets_are_refused_with_source_lines ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/binary64_adversary.rs (target/debug/deps/binary64_adversary-00fd33a384eed561)

running 1 test
test cli_composition_obeys_the_independently_authored_vectors ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running tests/binary64_count_adversary.rs (target/debug/deps/binary64_count_adversary-25ef58b4641736f8)

running 2 tests
test cli_binary64_model_and_original_suites_preserve_report_destinations_in_both_formats ... ok
test generated_go_binary64_shapes_refuse_before_factory_and_report_publication ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.51s

     Running tests/binary64_publication.rs (target/debug/deps/binary64_publication-ba03872d9e795d41)

running 1 test
test binary64_sparse_model_refuses_every_unsupported_publication_route ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/binary64_structural_adversary.rs (target/debug/deps/binary64_structural_adversary-d76d1dd27f537ab9)

running 1 test
test binary64_publication_never_replaces_sources_or_partially_updates_a_library ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/command_surface.rs (target/debug/deps/command_surface-71a1dc5a6624aa01)

running 5 tests
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test the_help_offers_exactly_the_four_areas ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-604eebcbf9b30edb)

running 4 tests
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/count_reports.rs (target/debug/deps/count_reports-aeedc7adf842489b)

running 3 tests
test count_report_opt_in_has_a_distinct_detailed_surface_and_unknown_coverage ... ok
test count_cli_configuration_and_original_suite_refusals_preserve_destinations ... ok
test count_cli_preserves_default_bytes_and_standalone_detailed_pairing ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s

     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-1a7e7875d1ea7c87)

running 3 tests
test generated_go_skip_counts_preserve_opaque_ids_and_strictness_without_a_destination ... ok
test generated_go_abnormal_teardown_cannot_publish_a_completed_skip ... ok
test generated_go_rejects_closed_predicate_metadata_before_any_target ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.48s

     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-3af0e671176d1e0b)

running 2 tests
test generated_go_abnormal_unsupported_error_formatting_cannot_complete ... ok
test generated_go_admits_only_typed_predicate_paths_and_operator_envelopes ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.30s

     Running tests/coverage_browser.rs (target/debug/deps/coverage_browser-bf192040179e69f7)

running 4 tests
test retained_legacy_player_bytes_still_replay_in_actual_firefox ... ok
test actual_browser_admits_the_pair_before_creating_replay_state ... ok
test actual_browser_and_rust_refuse_every_closed_model_field_boundary ... ok
test actual_browser_checks_full_lineage_and_integer_metadata ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.94s

     Running tests/coverage_cli.rs (target/debug/deps/coverage_cli-994c933e9b49eced)

running 6 tests
test coverage_cli_refuses_binary64_model_before_each_new_production_surface ... ok
test coverage_cli_authored_roots_relocate_without_losing_exact_text_and_refuse_unrepresentable_paths ... ok
test explicit_suite5_cli_produces_exact_inventory_and_requires_report2_before_execution ... ok
test select_cli_preserves_all_parent_bytes_and_explicit_empty_selection ... ok
test impact_cli_requires_exact_complete_input_and_keeps_the_persisted_v3_shape ... ok
test generated_go_executes_the_admitted_coverage_inventory_and_preserves_pairing_defaults ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s

     Running tests/coverage_lineage.rs (target/debug/deps/coverage_lineage-a42eee64e165b553)

running 2 tests
test generated_go_checks_original_lineage_and_typed_defaults ... ok
test go_execution_adapts_only_selected_integer_fields_before_target_effects ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.55s

     Running tests/coverage_producers.rs (target/debug/deps/coverage_producers-1b45cfea40b4b957)

running 3 tests
test actual_rust_coverage_exports_match_the_independent_plan ... ok
test actual_coverage_producers_refuse_noninvoked_negative_clock_and_report1_without_output ... ok
test actual_go_coverage_exports_match_the_independent_plan ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.81s

     Running tests/coverage_writer_adversary_pass1.rs (target/debug/deps/coverage_writer_adversary_pass1-96fa9eb4cce59dee)

running 2 tests
test go_strict_diagnostic_does_not_label_known_suite5_inventory_as_legacy_unknown ... ok
test browser_refuses_a_command_name_with_a_final_line_feed_before_replay_state ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.75s

     Running tests/coverage_writer_adversary_pass2.rs (target/debug/deps/coverage_writer_adversary_pass2-efcbea4be28fb826)

running 1 test
test browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner ... FAILED

failures:

---- browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner stdout ----

thread 'browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner' (1606602) panicked at crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs:102:9:
assertion `left == right` failed
  left: Object {"\0": Array [Null, Object {"raw": String("2.0")}], "__proto__": Object {"sentinel": String("retained")}, "constructor": Object {"prototype": Object {"raw": String("9.0")}}, "hasOwnProperty": Bool(false), "toString": String("ordinary data"), "\u{e000}": String("BMP key"), "😀": String("astral key")}
 right: Object {"\0": Array [Null, Number(2)], "__proto__": Object {"sentinel": String("retained")}, "constructor": Object {"prototype": Number(9)}, "hasOwnProperty": Bool(false), "toString": String("ordinary data"), "\u{e000}": String("BMP key"), "😀": String("astral key")}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.70s

error: test failed, to rerun pass `-p ess-cli --test coverage_writer_adversary_pass2`
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-ac70ba28528e59d8)

running 4 tests
test a_binding_function_capture_is_refused_before_rust_cli_writes ... ok
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_web_cli_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-d5a347d9ae9103cd)

running 13 tests
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test count_go_predicate_admission_matches_rust_leaf_grammar ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test count_report_skip_only_is_inconclusive_without_actual_failures ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test count_retained_runtime_preserves_legacy_behavior_and_does_not_gain_version_checks ... ok
test count_go_empty_selection_is_inconclusive_and_clock_conversion_is_checked ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test count_go_actual_producers_keep_skip_error_and_teardown_categories ... ok
test count_go_refusals_precede_targets_and_incomplete_runs_never_publish ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.10s

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
test output_symlinks_and_hardlinks_are_refused_without_mutation ... ok
test generated_paths_cannot_replace_canonical_source_inputs_or_follow_links ... ok
test stale_bundle_missing_source_and_unknown_dispatch_refuse ... ok
test qualified_go_base64_pattern_is_published_without_decoding_the_value ... ok
test positional_cli_refuses_unused_branches_before_publication ... ok
test explicit_binary64_inputs_run_through_the_cli_without_weakening_version_one ... ok
test checked_recipe_and_run_are_deterministic_with_json_only_stdout ... ok
test output_cannot_replace_any_declared_input ... ok
test generation_checks_refuse_before_creating_or_changing_destinations ... ok
test failed_check_or_execution_preserves_output_and_does_not_emit_a_result ... ok
test raw_json_capture_runs_before_schema_and_keeps_failure_output_untouched ... ok
test generated_libraries_match_the_api_and_drift_check_never_repairs_files ... ok
test positional_cli_prepares_text_and_refuses_before_publication ... ok
test model_sources_are_compiled_pinned_and_protected_by_the_cli ... ok
test binary64_cli_keeps_numeric_identity_and_emits_checked_format_five ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.93s

     Running tests/normalization_positional_adversary.rs (target/debug/deps/normalization_positional_adversary-406279f2f9d35352)

running 2 tests
test cli_runtime_grammar_controls_preserve_existing_output ... ok
test cli_invalid_policy_blocks_all_publication_and_valid_metadata_is_drift_checked ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

     Running tests/normalization_typescript.rs (target/debug/deps/normalization_typescript-5542aa1874e06ad7)

running 7 tests
test unqualified_profile_refuses_with_source_pointer_and_no_partial_files ... ok
test retained_recipe_and_bundle_inputs_cannot_be_overwritten ... ok
test typescript_cli_emits_an_accounted_executable_package ... ok
test model_inputs_are_recompiled_pinned_and_protected_before_typescript_generation ... ok
test module_and_late_path_conflicts_refuse_before_any_publication ... ok
test generated_file_and_parent_links_refuse_without_touching_their_destinations ... ok
test every_planned_file_participates_in_read_only_drift_checks ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s

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

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/output_containment.rs (target/debug/deps/output_containment-5c56b28ecc91698b)

running 19 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
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
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/schema_bundle.rs (target/debug/deps/schema_bundle-6f1b8cef6ccfdb14)

running 8 tests
test missing_dialect_and_incomplete_import_leave_existing_output_alone ... ok
test document_import_refusals_preserve_source_and_existing_output ... ok
test type_planning_and_output_refusals_leave_no_partial_library ... ok
test corrupted_import_cannot_be_projected_and_output_cannot_replace_source ... ok
test types_bundle_is_deterministic_and_never_replaces_its_input ... ok
test import_reload_projection_and_instance_validation_keep_original_data ... ok
test document_root_survives_reload_projection_and_each_type_target ... ok
test native_bundle_targets_require_identity_and_emit_build_metadata ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

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
test decision::tests::exactly_one_reason_says_another_candidate_would_help ... ok
test counts::tests::exact_unsigned_scalar_vectors_do_not_use_binary64 ... ok
test decision::tests::a_refusal_renders_the_predicate_the_command_and_every_reason ... ok
test counts::tests::payload_number_and_utf8_canonical_profile_is_frozen_separately_from_scalars ... ok
test decision::tests::a_decision_reads_its_two_other_cases_as_neither_satisfied_nor_the_other ... ok
test evidence::tests::report_readers_refuse_more_nonpasses_than_executed_scenarios ... ok
test evidence::tests::a_standalone_report_carries_every_field_an_adapter_needs ... ok
test evidence::tests::report_readers_do_not_guess_a_producer_from_status_vocabulary ... ok
test faulty::tests::a_fault_is_injected_into_the_system_that_declares_what_it_breaks ... ok
test evidence::tests::the_closed_report_round_trips_with_identical_canonical_bytes ... ok
test faulty::tests::every_fault_says_what_it_is_and_where_it_goes ... ok
test evidence::tests::report_readers_preserve_go_producer_bytes_and_historical_nonpass_counts ... ok
test faulty::tests::no_two_faults_claim_the_same_scenario ... ok
test faulty::tests::only_the_two_faults_the_boundary_cannot_express_are_injected_in_the_implementation ... ok
test evidence::tests::unknown_report_fields_are_refused ... ok
test evidence::tests::report_readers_refuse_nonpass_count_and_list_disagreement ... ok
test go::tests::every_go_file_is_in_the_package_the_readme_names ... ok
test input::tests::shape_errors_render_one_per_line_and_name_the_input_root_by_name ... ok
test input::tests::every_primitive_projects_to_the_one_fact_value_that_can_hold_it ... ok
test input::tests::a_primitive_refuses_a_node_of_the_wrong_shape_rather_than_coercing_it ... ok
test report::tests::every_check_code_has_a_distinct_name_and_a_rule_sentence ... ok
test report::tests::a_diagnostic_answers_all_five_of_the_questions_a_failure_has_to_answer ... ok
test runner::tests::a_count_is_the_half_of_an_ordering_claim_that_says_the_rows_were_there ... ok
test runner::tests::a_count_with_neither_bound_is_a_suite_defect_and_not_a_satisfied_assertion ... ok
test report::tests::a_quoted_input_reads_as_the_call_that_was_made ... ok
test report::tests::a_scenario_status_is_the_strongest_of_its_checks_and_a_contradiction_outranks_everything ... ok
test report::tests::an_unsupported_scenario_makes_the_run_fail_rather_than_look_like_a_pass ... ok
test runner::tests::a_declared_order_is_checked_on_adjacent_rows_and_the_next_key_breaks_a_tie ... ok
test runner::tests::a_nested_row_binds_the_paths_a_predicate_spells ... ok
test runner::tests::a_position_in_a_view_that_declares_no_order_is_a_suite_defect ... ok
test runner::tests::a_position_names_both_ends_and_a_row_that_is_not_there_is_not_a_match ... ok
test runner::tests::a_ranking_key_a_row_does_not_publish_is_undecidable_rather_than_out_of_order ... ok
test runner::tests::a_predicate_a_row_cannot_answer_is_reported_rather_than_retried ... ok
test runner::tests::an_empty_field_set_means_a_row_exists_and_not_that_anything_will_do ... ok
test go::tests::the_runner_is_a_constant_and_only_the_suite_moves ... ok
test runner::tests::a_view_that_holds_nothing_does_not_satisfy_an_invariant_by_being_empty ... ok
test runner::tests::an_order_over_fewer_than_two_rows_holds_and_does_not_double_as_a_non_emptiness_claim ... ok
test scenario::tests::a_declared_leaf_admits_what_its_type_admits_and_absence_only_where_the_type_permits_it ... ok
test runner::tests::ids_come_from_the_suite_and_from_nothing_ambient ... ok
test runner::tests::the_runners_clock_advances_on_every_read_so_a_deadline_can_bound_anything ... ok
test scenario::tests::a_purpose_is_one_line_and_says_something ... ok
test scenario::tests::a_scenario_id_names_the_construct_it_exercises_rather_than_its_position ... ok
test scenario::tests::a_semantic_reference_renders_the_way_the_design_writes_one ... ok
test scenario::tests::a_scenario_id_that_names_no_construct_is_refused ... ok
test scenario::tests::a_suite_format_from_a_later_build_is_refused_rather_than_guessed ... ok
test evidence::tests::report_readers_refuse_unknown_report_formats ... ok
test scenario::tests::a_payload_shape_round_trips_through_the_form_a_suite_is_stored_in ... ok
test scenario::tests::a_transition_ref_refuses_a_name_no_lifecycle_can_declare ... ok
test scenario::tests::a_suite_refuses_a_second_scenario_under_one_id ... ok
test scenario::tests::an_invariant_scenario_is_keyed_by_the_entity_and_the_branch_and_never_by_a_position ... ok
test scenario::tests::every_binding_aspect_is_in_the_list_that_is_walked_to_produce_them ... ok
test evidence::tests::report_readers_refuse_malformed_and_unsupported_suite_versions ... ok
test scenario::tests::the_ids_of_a_suite_sort_the_way_a_reader_sorts_the_file ... ok
test scenario::tests::two_scenarios_about_the_same_thing_in_the_same_way_are_one_id ... ok
test synthesize::tests::a_refusal_names_the_construct_the_code_and_the_repair ... ok
test scenario::tests::every_scenario_id_reads_back_from_the_form_a_report_prints ... ok
test synthesize::tests::every_refusal_carries_a_distinct_code_in_one_family ... ok
test web::tests::no_comparison_sits_in_a_text_node_mustache ... ok
test witness::tests::a_text_witness_is_its_own_path_so_two_fields_of_one_type_never_agree ... ok
test witness::tests::an_enum_offers_every_variant_it_declares_and_the_first_one_only_once ... ok
test witness::tests::an_integer_leaf_is_never_offered_a_fractional_candidate ... ok
test witness::tests::the_candidate_count_is_bounded_however_many_fields_a_guard_reads ... ok
test witness::tests::two_uuid_witnesses_differ_and_neither_moves_when_a_third_field_appears ... ok
test witness::tests::the_alternatives_for_a_number_are_the_guards_own_literals_either_side ... ok
test web::tests::the_page_calls_nothing_the_player_does_not_return ... ok
test evidence::tests::report_readers_refuse_nonpass_entries_without_a_known_nonpass_status ... ok
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
test a_positional_claim_takes_the_order_from_the_view_rather_than_from_the_author ... ok
test a_position_in_a_view_that_declares_no_order_is_refused ... ok
test a_claim_the_timelines_own_instants_contradict_is_refused ... ok
test a_declared_field_nothing_supplies_is_refused_by_name ... ok
test a_bounded_negative_that_forbids_no_event_is_refused ... ok
test a_halt_stated_beside_another_claim_is_two_assertions_filed_as_one ... ok
test a_field_the_surface_does_not_declare_is_refused_by_name ... ok
test a_halt_claimed_of_a_listing_with_no_declared_order_is_refused_by_the_code_that_already_says_so ... ok
test a_document_that_is_not_one_is_refused_rather_than_read_as_an_empty_scenario ... ok
test a_domain_the_model_does_not_declare_is_refused_by_name ... ok
test a_command_the_model_does_not_declare_is_refused_by_name ... ok
test a_format_this_build_does_not_implement_is_refused_before_anything_is_read ... ok
test a_predicate_reading_something_the_view_does_not_publish_is_refused ... ok
test a_halt_compiles_to_a_step_of_its_own_and_not_to_a_claim_about_rows ... ok
test a_scenario_compiles_to_the_id_the_domain_and_the_name_make ... ok
test a_reference_where_the_suite_compares_a_value_it_carries_is_refused ... ok
test a_scenario_that_runs_nothing_is_refused_rather_than_counted_as_a_check ... ok
test an_actor_the_model_does_not_declare_is_refused_by_name ... ok
test an_elapsed_claim_compiles_to_the_four_steps_that_carry_it_and_they_come_before_the_act ... ok
test a_window_of_no_seconds_is_refused_rather_than_compiled_into_a_check_that_cannot_fail ... ok
test a_window_measured_from_an_instant_nothing_marked_is_refused_with_the_ones_that_are ... ok
test a_value_read_off_an_event_nothing_required_is_refused ... ok
test a_state_the_lifecycle_does_not_declare_is_refused_as_a_state_and_not_as_a_variant ... ok
test a_timeline_whose_instants_do_not_ascend_is_refused ... ok
test an_actor_the_specification_does_not_grant_the_command_is_refused ... ok
test a_view_the_model_does_not_declare_is_refused_by_name ... ok
test an_act_cannot_open_a_window_at_its_own_instant ... ok
test an_assertion_that_states_other_than_one_claim_is_refused ... ok
test a_halt_after_no_rows_at_all_is_refused_rather_than_compiled ... ok
test a_window_that_states_other_than_one_bound_is_refused ... ok
test a_value_the_declared_type_does_not_admit_is_refused_where_it_sits ... ok
test an_error_the_model_does_not_declare_is_refused_by_name ... ok
test an_instance_bound_to_a_field_that_cannot_hold_an_identity_is_refused ... ok
test an_event_the_model_does_not_declare_is_refused_by_name ... ok
test an_instance_named_before_anything_binds_it_is_refused ... ok
test an_entity_the_model_does_not_declare_is_refused_by_name ... ok
test authored_predicate_operand_errors_have_their_own_refusal ... ok
test an_outcome_the_command_does_not_declare_is_refused_with_the_ones_it_does ... ok
test authored_aggregate_presence_keeps_026_and_valid_scalar_reads_keep_the_predicate ... ok
test one_name_for_two_instants_is_refused_rather_than_read_as_the_later_one ... ok
test an_instance_the_arrangement_does_not_declare_is_refused_by_name ... ok
test the_steps_are_the_vocabulary_a_generated_scenario_already_uses ... ok
test two_files_naming_one_scenario_are_refused_rather_than_one_displacing_the_other ... ok
test the_committed_billing_suite_holds_the_authored_scenario_beside_the_generated_ones ... ok
test the_order_the_files_are_handed_over_in_does_not_reach_the_result ... ok
test every_cause_is_reachable_from_a_document ... ok
test coverage_builder_retains_authored_duplicate_ownership_and_original_source_bytes ... ok
test two_compilations_of_one_file_produce_identical_bytes ... ok
test paired_browser_emission_retains_original_input_and_checks_the_actual_model ... ok
test independently_successful_authored_batches_are_refused_only_at_final_merge ... ok
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
test coverage_integer_tokens_and_closed_fields_are_checked_before_serde ... ok
test coverage_source_identity_is_checked_without_normalizing_it ... ok
test inventory_corruption_is_refused_at_admission ... ok
test explicit_selection_retains_original_parents_and_refuses_direct_admission ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/coverage_writer_adversary_pass1.rs (target/debug/deps/coverage_writer_adversary_pass1-257bfdcc793f80b8)

running 2 tests
test d1_authored_inventory_preserves_full_original_typed_refusal_rendering ... ok
test d1_generated_inventory_preserves_full_original_typed_refusal_rendering ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/coverage_writer_adversary_pass2.rs (target/debug/deps/coverage_writer_adversary_pass2-cf711d0e33f85f1e)

running 1 test
test final_merge_orders_sources_and_preserves_full_d1_diagnostics_through_empty_selection ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

     Running tests/elapsed.rs (target/debug/deps/elapsed-b5312b79e8e5d6a8)

running 7 tests
test a_window_opened_at_an_instant_nothing_marked_is_a_suite_defect_and_not_a_failed_implementation ... ok
test a_deadline_the_target_ran_past_fails_the_within_claim ... ok
test a_target_whose_clock_never_moves_fails_rather_than_being_read_as_having_waited ... ok
test a_target_with_no_clock_reports_unsupported_and_the_run_fails ... ok
test a_target_that_holds_the_window_and_reports_it_passes ... ok
test an_event_published_inside_the_window_fails_the_bounded_negative_and_nothing_else ... ok
test two_runs_over_one_window_produce_byte_identical_reports ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/execution.rs (target/debug/deps/execution-ae712a3af979f44f)

running 12 tests
test an_eventual_assertion_asks_again_within_a_deadline_and_never_sleeps ... ok
test an_eventual_view_is_read_again_and_a_read_your_writes_view_is_not ... ok
test a_scenario_whose_input_no_longer_reaches_its_branch_fails_with_a_diagnostic_naming_the_defect ... ok
test a_value_of_the_wrong_declared_type_is_caught_by_the_same_check_as_a_missing_one ... ok
test an_event_missing_a_field_it_declares_is_named_leaf_by_leaf_rather_than_reported_as_absent ... ok
test a_view_assertion_names_the_instance_the_scenario_created_rather_than_any_row ... ok
test a_view_answered_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test a_target_that_cannot_expose_an_observation_fails_the_run_rather_than_skipping_it ... ok
test every_scenario_the_billing_specification_obliges_passes_against_the_reference_implementation ... ok
test a_read_your_writes_view_is_not_quietly_read_at_current_when_no_token_came_back ... ok
test every_scenario_checked_something_and_no_family_of_them_was_silently_empty ... ok
test two_runs_of_one_suite_against_one_target_produce_byte_identical_reports ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

     Running tests/faults.rs (target/debug/deps/faults-592913acd22499fb)

running 11 tests
test a_fault_nothing_catches_is_recorded_rather_than_quietly_dropped ... ok
test the_diagnostic_of_a_caught_fault_names_the_defect_rather_than_reporting_that_something_broke ... ok
test dropping_one_binding_leaves_the_other_two_green ... ok
test a_wrong_mapping_is_invisible_to_a_target_that_cannot_show_its_invocations ... ok
test the_widest_blast_radius_is_scenarios_that_could_not_be_arranged_rather_than_extra_verdicts ... ok
test every_fault_that_could_be_a_boundary_perturbation_is_one ... ok
test each_specification_is_passed_in_full_by_the_implementation_written_from_it ... ok
test two_runs_against_one_faulty_target_produce_byte_identical_reports ... ok
test a_faults_blast_radius_is_accounted_for ... ok
test a_fault_does_not_simply_break_everything ... ok
test each_fault_fails_the_scenario_that_exists_to_catch_it ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s

     Running tests/halt.rs (target/debug/deps/halt-e9156dff15c9e526)

running 7 tests
test a_halt_of_an_eventual_listing_is_asked_again_while_the_projection_catches_up ... ok
test a_listing_that_ran_out_before_the_reader_stopped_it_is_not_a_halt ... ok
test a_target_that_cannot_read_a_row_at_a_time_reports_unsupported_and_the_run_fails ... ok
test a_target_whose_producer_stops_when_the_reader_does_passes ... ok
test a_target_that_reads_the_whole_listing_fails_rather_than_being_read_as_having_stopped ... ok
test retrying_does_not_rescue_a_producer_that_never_stops ... ok
test two_runs_over_one_halt_claim_produce_byte_identical_reports ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/report_reader_adversary.rs (target/debug/deps/report_reader_adversary-c5e2e8b3df383188)

running 4 tests
test duplicate_claims_cannot_hide_behind_a_valid_last_value ... ok
test count_extremes_refuse_contradictions_without_inventing_coverage ... ok
test closed_wire_fields_preserve_their_formats_scalar_contracts ... ok
test aggregate_status_does_not_depend_on_nonpass_order_or_multiplicity ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

     Running tests/suite.rs (target/debug/deps/suite-b620394c52c8390b)

running 16 tests
test a_suite_naming_something_that_is_not_an_ess_name_is_refused_while_it_is_read ... ok
test a_suite_parses_from_text_alone_without_an_ir ... ok
test the_scan_for_a_clock_finds_one_and_does_not_find_a_word_that_merely_ends_in_a_banned_token ... ok
test a_count_and_a_position_read_back_as_what_a_runner_in_another_language_must_read ... ok
test the_steps_a_binding_and_an_invariant_need_survive_being_read_back_from_text ... ok
test sparse_models_cannot_publish_an_empty_success_for_binary64 ... ok
test every_scenario_id_the_billing_model_can_produce_reads_back ... ok
test the_step_vocabulary_expresses_the_worked_example_from_section_ten ... ok
test the_scenario_ids_appear_in_the_file_in_the_order_a_sorted_key_list_would_be ... ok
test a_suite_serialised_in_one_process_resolves_in_another ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test the_suite_records_the_same_model_digest_the_projections_do ... ok
test inserting_one_outcome_re_keys_nothing_around_it ... ok
test the_dependency_set_names_a_type_no_derived_from_would_have_mentioned ... ok
test serialising_a_suite_twice_produces_byte_identical_json ... ok
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
test a_component_nothing_declares_is_refused_by_name ... ok
test a_state_reached_only_through_a_branch_no_input_reaches_is_refused_rather_than_arranged ... ok
test a_binding_that_retries_forces_one_failure_and_still_requires_the_consequence ... ok
test a_binding_mapping_names_the_source_the_document_wrote_and_not_its_same_typed_sibling ... ok
test a_read_your_writes_view_filled_by_the_command_that_ran_is_asserted_to_hold_a_row ... ok
test a_binding_that_drops_its_failures_refuses_that_check_and_names_the_reason ... ok
test an_entity_nothing_creates_cannot_be_acted_on_and_says_so ... ok
test a_value_object_nothing_observable_holds_keeps_a_refusal_naming_what_would_close_it ... ok
test a_move_is_observed_through_the_view_the_state_it_left_is_filtered_on ... ok
test a_binding_flow_is_proved_through_the_event_the_invoked_command_publishes ... ok
test a_declared_error_is_asserted_by_name_and_never_by_an_invented_payload ... ok
test a_value_objects_own_invariants_are_read_at_every_field_position_a_view_holds_one ... ok
test a_view_is_asserted_in_the_block_its_own_consistency_decides ... ok
test a_view_that_does_not_hold_the_instance_yet_is_not_asked_about_its_invariants ... ok
test a_scenario_that_moves_an_instance_names_the_one_an_earlier_step_created ... ok
test an_order_the_specification_cannot_put_two_rows_under_is_refused_and_not_asserted ... ok
test an_undecidable_guard_refuses_and_does_not_spend_the_candidate_budget ... ok
test a_declared_order_is_asserted_against_two_rows_the_scenario_arranged_itself ... ok
test a_synthesised_count_is_a_floor_the_scenario_arranged_and_never_a_ceiling ... ok
test a_view_the_entity_has_not_reached_yet_is_asserted_to_exclude_the_instance_by_name ... ok
test a_binding_that_escalates_requires_the_event_the_escalation_declares ... ok
test a_move_that_is_illegal_in_a_state_is_attempted_with_the_input_that_would_have_worked ... ok
test an_at_least_once_binding_delivers_the_event_twice_and_requires_no_count ... ok
test a_suite_for_one_component_holds_only_what_that_component_realises ... ok
test an_outcome_that_updates_an_instance_acts_on_one_the_scenario_created ... ok
test an_actor_is_named_only_where_the_specification_grants_the_command ... ok
test an_illegal_move_requires_the_branch_and_the_declared_error_rather_than_merely_failing ... ok
test a_whole_system_suite_does_not_mention_a_component ... ok
test an_event_assertion_carries_the_declared_shape_and_exactly_the_values_the_payload_determines ... ok
test an_invariant_over_a_field_no_view_publishes_refuses_rather_than_being_dropped ... ok
test every_declared_outcome_is_either_a_scenario_or_a_named_refusal_or_asserted_by_the_state_family ... ok
test the_failure_control_is_armed_after_the_arrangement_and_before_the_command_that_triggers_it ... ok
test an_invariant_is_asserted_against_every_view_that_publishes_what_it_reads ... ok
test every_declared_transition_has_a_scenario_that_proves_it_can_occur ... ok
test every_command_names_an_instance_an_earlier_step_of_the_same_scenario_bound ... ok
test a_synthesised_suite_survives_being_written_and_read_back ... ok
test the_dependency_set_names_the_types_the_scenario_is_made_of ... ok
test the_input_a_scenario_sends_is_re_decided_against_the_guard_it_claims_to_reach ... ok
test an_outcome_no_input_decides_is_reached_by_injection_and_by_nothing_else ... ok
test the_refusal_branch_asserts_that_no_event_the_specification_declares_occurred ... ok
test each_example_synthesises_the_families_its_specification_declares ... ok
test every_clause_of_every_binding_is_either_a_scenario_or_a_named_refusal ... ok
test synthesising_the_same_specification_twice_produces_byte_identical_output ... ok
test coverage_all_missing_invariants_keep_null_survivors_and_component_proofs_stay_conservative ... ok
test coverage_builder_records_the_complete_generated_inventory_and_component_omissions ... ok
test canonical_expression_compatibility_fixtures ... ok
test coverage_missing_lifecycle_and_view_checks_remain_beside_actual_passing_results ... ok

test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s

     Running tests/witness.rs (target/debug/deps/witness-9e1a2f8f8747eefd)

running 28 tests
test a_refuted_guard_carries_the_leaf_and_the_value_that_refuted_it ... ok
test a_path_landing_on_an_aggregate_is_unevaluable_by_construction ... ok
test a_newtype_is_transparent_when_a_path_is_resolved_as_well_as_when_it_is_projected ... ok
test a_disjunction_one_of_whose_branches_holds_is_satisfied_despite_an_undecidable_branch ... ok
test a_long_recursive_read_validates_beyond_the_projection_limit ... ok
test a_candidate_input_projects_one_fact_per_scalar_leaf ... ok
test a_candidate_carrying_a_field_no_type_declares_is_refused ... ok
test a_candidate_missing_a_required_field_is_refused_before_any_guard_is_read ... ok
test a_conjunction_of_two_undecidable_leaves_reports_both ... ok
test a_refusal_names_the_predicate_the_command_and_the_path ... ok
test a_list_a_map_and_a_union_bind_no_fact_in_the_current_projection ... ok
test an_absent_optional_is_unevaluable_but_says_a_candidate_could_repair_it ... ok
test a_path_into_a_list_or_a_union_names_the_aggregate_rather_than_the_missing_element ... ok
test a_newtype_is_transparent_so_a_deep_path_reaches_through_it_without_a_segment ... ok
test a_scalar_of_the_wrong_shape_is_refused_rather_than_coerced ... ok
test equality_over_two_texts_is_decided_even_though_ordering_them_is_not ... ok
test legal_collection_cardinality_is_not_currently_projected ... ok
test an_absent_optional_binds_nothing_rather_than_binding_a_default ... ok
test expression_search_limits_do_not_define_type_correctness ... ok
test only_an_absent_value_says_another_candidate_would_help ... ok
test otherwise_and_external_are_not_guards_over_the_input ... ok
test the_same_text_ordering_is_decidable_once_a_scale_contains_both_values ... ok
test ordering_two_texts_is_unevaluable_because_an_ess_specification_declares_no_scale ... ok
test ordering_across_two_types_is_unevaluable_not_false ... ok
test the_normative_shape_of_guard_is_decidable_for_both_signs ... ok
test malformed_declarations_refuse_early_and_direct_bad_reads_remain_unknown ... ok
test unclassified_is_a_drift_alarm_and_no_enumerated_source_trips_it ... ok
test resolved_adapter_keeps_semantics_separate_from_collection_projection ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/lib.rs (target/debug/deps/ess_diff-06167d7f60ffb115)

running 9 tests
test change::tests::only_a_grant_and_a_variant_decide_a_direction ... ok
test change::tests::a_change_with_no_member_renders_three_parts_rather_than_a_trailing_slash ... ok
test change::tests::a_change_id_names_its_category_subject_subtype_and_member_in_that_order ... ok
test change::tests::the_canonical_order_is_the_category_order_and_not_the_alphabet ... ok
test delta::tests::a_delta_puts_its_changes_in_canonical_order_however_they_arrive ... ok
test impact::tests::a_whole_answer_absorbs_a_narrowing_whichever_way_round_they_are_joined ... ok
test impact::tests::an_unfollowed_file_is_not_an_artifact_that_owes_regeneration ... ok
test impact::tests::a_change_to_the_specification_itself_owes_the_whole_suite ... ok
test impact::tests::a_suite_resting_on_a_construct_the_graph_has_no_node_for_owes_the_whole_suite ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/artifacts.rs (target/debug/deps/artifacts-fe289ed3b5544f1f)

running 13 tests
test a_change_to_the_system_header_owes_every_artifact ... ok
test whole_model_artifacts_are_owed_by_any_change_at_all ... ok
test the_two_predicate_edits_narrow_the_artifacts_differently_and_both_subsets_are_named ... ok
test an_artifact_whose_slice_nothing_reached_is_absent_from_the_answer ... ok
test an_owed_artifacts_path_explains_the_membership_hop_by_hop ... ok
test the_artifacts_the_currency_changes_reach_are_owed_and_named ... ok
test the_six_change_delta_owes_a_strict_subset_of_the_artifacts ... ok
test a_grant_change_owes_the_documents_that_read_grants_and_not_the_ones_that_do_not ... ok
test the_artifact_answer_is_byte_identical_between_runs ... ok
test review_whole_model_hashes_and_index_bytes_remain_frozen ... ok
test a_committed_artifact_with_a_false_contract_digest_is_owed_as_a_false_claim ... ok
test a_committed_tree_is_answered_for_fail_closed_file_by_file ... ok
test review_legacy_slice_stamps_are_owed_even_when_raw_hashes_match ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s

     Running tests/canonical.rs (target/debug/deps/canonical-ce09a7a45878a366)

running 20 tests
test a_binding_still_has_one_delivery_a_document_can_write ... ok
test every_change_variant_has_something_to_say_for_itself ... ok
test review_freeze_legacy_delta_bytes ... ok
test a_change_is_spelt_the_same_way_in_its_id_and_in_the_document ... ok
test review_version_admission_refuses_new_vocabulary_in_legacy_envelopes ... ok
test no_source_file_in_the_diff_engine_reads_a_clock_or_an_unordered_map ... ok
test no_source_file_in_the_diff_engine_calls_an_ir_handle_accessor ... ok
test a_system_still_has_no_naming_a_document_can_set ... ok
test a_document_with_six_defects_reports_six ... ok
test a_delta_written_in_a_format_this_build_does_not_read_is_refused ... ok
test a_delta_naming_two_systems_is_refused_on_the_way_in_as_well ... ok
test a_delta_whose_id_was_edited_is_refused ... ok
test review_new_default_delta_format_is_version_two ... ok
test a_delta_this_build_wrote_is_read_back_without_complaint ... ok
test canonical_json_ends_in_a_newline ... ok
test a_delta_whose_relation_was_edited_is_refused ... ok
test the_changes_are_written_in_the_category_order_and_not_the_alphabet ... ok
test every_change_in_a_delta_has_its_own_id ... ok
test a_delta_whose_changes_are_out_of_order_is_refused ... ok
test diffing_the_same_pair_twice_produces_byte_identical_json ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/families.rs (target/debug/deps/families-8952d48644da49b5)

running 69 tests
test a_filter_that_contains_different_instances_is_changed_with_no_direction ... ok
test a_newtype_that_wraps_something_else_is_reported ... ok
test a_command_added_is_one_change ... ok
test a_new_transition_arrives_with_the_outcome_that_takes_it ... ok
test a_component_that_no_longer_publishes_an_event_is_reported ... ok
test a_commands_naming_is_compared_key_by_key ... ok
test a_construct_moving_between_files_is_not_a_change ... ok
test a_guard_respaced_is_the_same_predicate_and_no_change ... ok
test a_mapping_filled_from_somewhere_else_is_reported_with_both_sources ... ok
test a_binding_added_is_one_change ... ok
test a_binding_reacting_to_a_different_event_is_reported ... ok
test a_payload_declaration_arriving_is_a_payload_change ... ok
test a_binding_invoking_a_different_command_moves_its_mapping_with_it ... ok
test a_bindings_failure_policy_is_compared ... ok
test a_filter_respaced_is_the_same_predicate_and_no_change ... ok
test a_type_that_became_a_different_kind_of_thing_is_reported_as_that_and_nothing_else ... ok
test a_filter_removed_reads_as_containing_every_instance ... ok
test a_bindings_naming_is_compared_key_by_key ... ok
test a_component_accepting_a_new_command_is_changed_and_not_widened ... ok
test a_union_variant_that_carries_something_else_is_not_a_variant_removed_and_added ... ok
test a_struct_field_that_changed_type_is_reported ... ok
test a_types_own_invariants_are_reported_as_different_and_never_as_stronger ... ok
test a_union_gaining_a_variant_widens_it_just_as_an_enum_does ... ok
test a_view_exposing_a_new_field_is_reported_with_the_type_it_carries ... ok
test a_view_added_is_one_change ... ok
test an_entity_field_replaced_is_removed_and_added_and_never_a_rename ... ok
test a_views_consistency_promise_is_compared_and_not_classified ... ok
test a_view_projecting_a_different_entity_is_a_source_change ... ok
test a_views_naming_is_compared_key_by_key ... ok
test an_actor_declared_with_no_grants_at_all_is_still_a_change_to_report ... ok
test an_entity_field_that_changed_type_is_reported ... ok
test a_view_fields_naming_is_compared_key_by_key ... ok
test a_union_that_is_tagged_by_another_field_is_reported ... ok
test an_entity_added_arrives_with_its_synthesised_state_enum_and_nothing_is_diffed_inside ... ok
test an_entitys_naming_is_compared_key_by_key ... ok
test an_error_that_gained_a_field_is_reported_with_the_type_it_carries ... ok
test an_entity_fields_naming_is_compared_key_by_key ... ok
test an_identitys_display_name_and_summary_are_compared ... ok
test an_input_added_is_reported_with_the_type_it_carries ... ok
test an_outcome_added_is_one_change_and_claims_no_direction ... ok
test an_event_renamed_is_reported_as_removed_and_added_and_never_as_a_rename ... ok
test an_input_that_changed_type_is_reported ... ok
test an_input_fields_naming_is_compared_key_by_key ... ok
test an_event_field_that_changed_type_is_reported ... ok
test an_events_wire_name_moving_is_not_the_event_moving ... ok
test reordering_an_event_payload_is_reported_once_and_not_as_a_field_change ... ok
test an_invariant_statement_reworded_without_moving_the_predicate_is_still_a_change ... ok
test renaming_an_entitys_identity_is_the_one_rename_this_crate_reports ... ok
test an_outcomes_summary_is_compared ... ok
test reordering_a_views_fields_is_reported_once ... ok
test reordering_a_commands_outcomes_is_a_real_change ... ok
test reordering_an_enums_variants_is_reported_without_claiming_a_direction ... ok
test the_error_a_branch_reports_is_compared ... ok
test reordering_a_commands_input_is_reported_once ... ok
test reordering_an_entitys_fields_is_reported_once ... ok
test the_paragraph_saying_what_the_system_is_is_compared ... ok
test the_specifications_version_moving_is_reported_and_is_not_the_identity ... ok
test what_an_outcome_emits_is_compared_in_order ... ok
test what_an_error_tells_the_caller_is_compared ... ok
test writing_out_a_naming_default_is_not_a_change ... ok
test review_reach_is_a_change_without_an_unrelated_surface_edit ... ok
test review_view_parameter_naming_is_compared_without_a_filter_edit ... ok
test review_outcome_refusal_is_independent_of_its_error ... ok
test review_cli_top_level_grouped_views_and_binary_are_changes ... ok
test review_outcome_sets_are_independent_of_event_payload ... ok
test review_view_ranking_is_compared_without_a_filter_edit ... ok
test review_residual_refs_cannot_hide_beside_a_classified_change ... ok
test review_unclassified_transition_order_cannot_hide_beside_a_classified_edit ... ok
test review_relation_cardinality_name_and_removal_are_changes ... ok

test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s

     Running tests/graph.rs (target/debug/deps/graph-746a5caf03f88bea)

running 10 tests
test correction2_row_shape_is_a_distinct_dependency_and_survives_graph_union ... ok
test a_type_is_reached_through_the_declarations_that_hold_it_and_not_by_name ... ok
test a_component_is_reached_through_what_it_accepts_and_publishes ... ok
test a_closure_over_the_whole_model_terminates_and_stays_inside_it ... ok
test the_graph_records_the_reference_an_author_wrote_and_not_its_reverse ... ok
test building_the_same_graph_twice_produces_the_same_edges_in_the_same_order ... ok
test review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union ... ok
test correction2_network_exposure_matches_actual_routes_and_owned_domains ... ok
test review_cli_views_and_parameter_types_are_forward_slice_dependencies ... ok
test every_relation_in_the_vocabulary_is_minted_by_a_specification_this_repository_ships ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/impact.rs (target/debug/deps/impact-1ac5d4ffb424fb86)

running 17 tests
test a_suite_produced_from_the_later_revision_is_refused_rather_than_narrowed ... ok
test a_suite_whose_contract_digest_its_model_does_not_compute_is_refused ... ok
test two_specifications_of_different_systems_are_refused_here_too ... ok
test raw_legacy_impact_cannot_discard_a_coverage_inventory ... ok
test a_suite_for_another_system_is_refused ... ok
test a_suite_resting_on_a_construct_no_graph_has_a_node_for_owes_the_whole_suite ... ok
test an_edited_entity_invariant_owes_every_scenario_that_rests_on_the_entity_and_no_other ... ok
test a_narrowed_answer_never_reports_more_scenarios_than_the_suite_holds ... ok
test a_variant_removed_from_an_enum_reaches_the_entity_that_holds_it_transitively ... ok
test every_scenario_resting_directly_on_a_changed_construct_is_owed_again ... ok
test taking_a_grant_from_an_actor_owes_only_the_scenarios_that_act_as_that_actor ... ok
test the_suite_the_fixture_obliges_is_ten_scenarios_and_the_delta_is_six_changes ... ok
test an_edited_outcome_guard_owes_every_scenario_because_every_scenario_creates_through_it ... ok
test analysing_the_same_pair_twice_produces_byte_identical_json ... ok
test a_domains_naming_moving_owes_the_whole_suite_because_no_family_compares_a_domain ... ok
test a_change_in_a_family_the_delta_still_does_not_compare_owes_the_whole_suite ... ok
test coverage_impact_keeps_exact_selection_context_out_of_persisted_impact3 ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s

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
test reusable_row_type_belongs_to_the_view_slice_it_supplies ... ok
test ranking_precedence_survives_the_checked_delta_roundtrip ... ok
test switching_equal_row_shapes_retains_independent_residual_coverage ... ok
test reusable_row_invariant_change_reaches_its_openapi_artifact ... ok
test served_view_change_reaches_its_openapi_artifact ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/revision_pair.rs (target/debug/deps/revision_pair-ba032edad7953c23)

running 11 tests
test adding_an_enum_variant_widens_the_type_that_accepts_it ... ok
test rewriting_an_entitys_invariant_is_changed_and_quotes_both_statements ... ok
test nothing_the_after_revision_only_rewrote_reaches_the_delta ... ok
test rewriting_an_outcomes_when_is_changed_and_renders_both_guards_canonically ... ok
test taking_a_command_from_an_actor_narrows_what_the_system_permits ... ok
test removing_an_enum_variant_narrows_the_type_that_accepted_it ... ok
test the_delta_survives_being_written_and_read_back ... ok
test two_different_systems_are_refused_rather_than_reported_as_a_rewrite ... ok
test a_revision_compared_with_itself_reports_nothing ... ok
test the_fixture_pair_differs_by_exactly_six_changes ... ok
test granting_a_command_to_an_actor_widens_what_the_system_permits ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests ess_conformance

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_diff

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p ess-cli --test coverage_writer_adversary_pass2`
```

strict-clippy-final3

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
    "TMPDIR": "home-path:sha256:92f8f1612ffcef2332a9e6288ffde9aa702f0ce8d0ba5ea3c473d6633073f9d7",
    "GOCACHE": "home-path:sha256:9647f19e1d4145b9a042080ad03b73cbb720546702ce729eb9992448c1969bf3",
    "GOMODCACHE": "home-path:sha256:be803e54d491c47a3476932da1e25dd63349228ddc27fe82f682c806e5400335",
    "CARGO_HOME": "home-path:sha256:d86081df2851a911b2fd0fae5ec8894c10b21569abe2ec6161e1223327a25796",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:0b20b12989d666b0f821e89e8c272e118b443e2a79ce7727b78ed8d8076889b2",
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
  "free_before": 22151340032,
  "started_epoch": 1788728906.369376,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass2.rs": "a1a51ded78d28fd5c3a180c3102b07f76d8550f5600d5e813dd912c19072b03f",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs": "b083325230517b72ed7d90e70753d2552115453212aaec0719d287619e4cfc0c"
  },
  "exit": 0,
  "elapsed_seconds": 2.4562247489811853,
  "free_after": 22150971392
}
```

```text
    Checking ess-conformance v0.20.0 (home-path:sha256:fb2af4db425baf24f833fe7f0be66e10d98c52642fdd023fe8c8d2f68266c344)
    Finished `dev` profile [unoptimized] target(s) in 2.41s
```

fmt-check-final

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
    "TMPDIR": "home-path:sha256:92f8f1612ffcef2332a9e6288ffde9aa702f0ce8d0ba5ea3c473d6633073f9d7",
    "GOCACHE": "home-path:sha256:9647f19e1d4145b9a042080ad03b73cbb720546702ce729eb9992448c1969bf3",
    "GOMODCACHE": "home-path:sha256:be803e54d491c47a3476932da1e25dd63349228ddc27fe82f682c806e5400335",
    "CARGO_HOME": "home-path:sha256:d86081df2851a911b2fd0fae5ec8894c10b21569abe2ec6161e1223327a25796",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:814950499ef0d9d22dffd9fd30b29462cae43d3bad69143ef20d826d91ac5fe6",
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
  "free_before": 22151016448,
  "started_epoch": 1788728907.5511625,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass2.rs": "a1a51ded78d28fd5c3a180c3102b07f76d8550f5600d5e813dd912c19072b03f",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs": "b083325230517b72ed7d90e70753d2552115453212aaec0719d287619e4cfc0c"
  },
  "exit": 0,
  "elapsed_seconds": 0.43341162300202996,
  "free_after": 22151000064
}
```

```text
```

package-suite-final2

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
    "TMPDIR": "home-path:sha256:92f8f1612ffcef2332a9e6288ffde9aa702f0ce8d0ba5ea3c473d6633073f9d7",
    "GOCACHE": "home-path:sha256:9647f19e1d4145b9a042080ad03b73cbb720546702ce729eb9992448c1969bf3",
    "GOMODCACHE": "home-path:sha256:be803e54d491c47a3476932da1e25dd63349228ddc27fe82f682c806e5400335",
    "CARGO_HOME": "home-path:sha256:d86081df2851a911b2fd0fae5ec8894c10b21569abe2ec6161e1223327a25796",
    "ESS_BROWSER_TMPDIR": "home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea",
    "ESS_COVERAGE_EXPORT_ROOT": "home-path:sha256:234fb9027b63d4e9e1c7d436ff9fc5329f51c970e32b5d6580d3311ab128c08e",
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
  "free_before": 22149042176,
  "started_epoch": 1788728922.9885933,
  "new_tests": {
    "crates/verify/ess-conformance/tests/coverage_writer_adversary_pass2.rs": "a1a51ded78d28fd5c3a180c3102b07f76d8550f5600d5e813dd912c19072b03f",
    "crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs": "b083325230517b72ed7d90e70753d2552115453212aaec0719d287619e4cfc0c"
  },
  "exit": 101,
  "elapsed_seconds": 46.61662236903794,
  "free_after": 21550440448
}
```

```text
   Compiling ess-conformance v0.20.0 (home-path:sha256:fb2af4db425baf24f833fe7f0be66e10d98c52642fdd023fe8c8d2f68266c344)
    Finished `test` profile [unoptimized] target(s) in 0.34s
     Running unittests src/main.rs (target/debug/deps/ess-63929f179d60f606)

running 12 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test coverage::tests::suite5_pair_refusal_precedes_target_construction ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/authored_scenarios.rs (target/debug/deps/authored_scenarios-3ba618a94d75005d)

running 25 tests
test author_empty ... ok
test author_nonmatching_only ... ok
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test author_nested_only ... ok
test go_nonmatching_only ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test go_nested_only ... ok
test run_nested_only ... ok
test ir_nonmatching_only ... ok
test ir_nested_only ... ok
test run_empty ... ok
test run_nonmatching_only ... ok
test ir_empty ... ok
test web_nested_only ... ok
test web_empty ... ok
test go_empty ... ok
test web_nonmatching_only ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s

     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-fbc20393f78d5213)

running 9 tests
test non_utf8_matching_content_is_refused_before_any_output_write ... ok
test selected_directory_symlink_reports_the_requested_path_and_stays_shallow ... ok
test empty_selection_cannot_follow_or_replace_an_output_symlink ... ok
test lexical_selection_order_decides_the_first_read_error ... ok
test matching_broken_links_are_read_errors_even_beside_a_valid_source ... ok
test committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners ... ok
test omitted_scenarios_ignore_a_poisoned_working_directory_default ... ok
test empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner ... ok
test valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

     Running tests/authored_site.rs (target/debug/deps/authored_site-1a420fe3beb09d76)

running 9 tests
test binary_downloads_are_not_silently_decoded ... ok
test an_explicit_missing_front_page_is_an_error ... ok
test asset_symlink_output_is_refused_before_any_page_changes ... ok
test publication_without_strict_mode_preserves_unpublished_links ... ok
test a_page_identity_can_itself_end_in_html ... ok
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

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.21s

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

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.35s

     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-3af0e671176d1e0b)

running 2 tests
test generated_go_abnormal_unsupported_error_formatting_cannot_complete ... ok
test generated_go_admits_only_typed_predicate_paths_and_operator_envelopes ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.29s

     Running tests/coverage_browser.rs (target/debug/deps/coverage_browser-bf192040179e69f7)

running 4 tests
test retained_legacy_player_bytes_still_replay_in_actual_firefox ... ok
test actual_browser_admits_the_pair_before_creating_replay_state ... ok
test actual_browser_and_rust_refuse_every_closed_model_field_boundary ... ok
test actual_browser_checks_full_lineage_and_integer_metadata ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.79s

     Running tests/coverage_cli.rs (target/debug/deps/coverage_cli-994c933e9b49eced)

running 6 tests
test coverage_cli_refuses_binary64_model_before_each_new_production_surface ... ok
test coverage_cli_authored_roots_relocate_without_losing_exact_text_and_refuse_unrepresentable_paths ... ok
test explicit_suite5_cli_produces_exact_inventory_and_requires_report2_before_execution ... ok
test select_cli_preserves_all_parent_bytes_and_explicit_empty_selection ... ok
test impact_cli_requires_exact_complete_input_and_keeps_the_persisted_v3_shape ... ok
test generated_go_executes_the_admitted_coverage_inventory_and_preserves_pairing_defaults ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s

     Running tests/coverage_lineage.rs (target/debug/deps/coverage_lineage-a42eee64e165b553)

running 2 tests
test generated_go_checks_original_lineage_and_typed_defaults ... ok
test go_execution_adapts_only_selected_integer_fields_before_target_effects ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.57s

     Running tests/coverage_producers.rs (target/debug/deps/coverage_producers-1b45cfea40b4b957)

running 3 tests
test actual_rust_coverage_exports_match_the_independent_plan ... ok
test actual_coverage_producers_refuse_noninvoked_negative_clock_and_report1_without_output ... ok
test actual_go_coverage_exports_match_the_independent_plan ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.62s

     Running tests/coverage_writer_adversary_pass1.rs (target/debug/deps/coverage_writer_adversary_pass1-96fa9eb4cce59dee)

running 2 tests
test go_strict_diagnostic_does_not_label_known_suite5_inventory_as_legacy_unknown ... ok
test browser_refuses_a_command_name_with_a_final_line_feed_before_replay_state ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.76s

     Running tests/coverage_writer_adversary_pass2.rs (target/debug/deps/coverage_writer_adversary_pass2-efcbea4be28fb826)

running 1 test
test browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner ... FAILED

failures:

---- browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner stdout ----

thread 'browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner' (1647408) panicked at crates/edge/ess-cli/tests/coverage_writer_adversary_pass2.rs:102:9:
assertion `left == right` failed
  left: Object {"\0": Array [Null, Object {"raw": String("2.0")}], "__proto__": Object {"sentinel": String("retained")}, "constructor": Object {"prototype": Object {"raw": String("9.0")}}, "hasOwnProperty": Bool(false), "toString": String("ordinary data"), "\u{e000}": String("BMP key"), "😀": String("astral key")}
 right: Object {"\0": Array [Null, Number(2)], "__proto__": Object {"sentinel": String("retained")}, "constructor": Object {"prototype": Number(9)}, "hasOwnProperty": Bool(false), "toString": String("ordinary data"), "\u{e000}": String("BMP key"), "😀": String("astral key")}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    browser_preserves_arbitrary_node_keys_and_checks_each_surviving_payload_owner

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.84s

error: test failed, to rerun pass `-p ess-cli --test coverage_writer_adversary_pass2`
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-ac70ba28528e59d8)

running 4 tests
test a_binding_function_capture_is_refused_before_rust_cli_writes ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_web_cli_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-d5a347d9ae9103cd)

running 13 tests
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test count_go_predicate_admission_matches_rust_leaf_grammar ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test count_report_skip_only_is_inconclusive_without_actual_failures ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test count_retained_runtime_preserves_legacy_behavior_and_does_not_gain_version_checks ... ok
test count_go_empty_selection_is_inconclusive_and_clock_conversion_is_checked ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test count_go_actual_producers_keep_skip_error_and_teardown_categories ... ok
test count_go_refusals_precede_targets_and_incomplete_runs_never_publish ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.52s

     Running tests/model_types.rs (target/debug/deps/model_types-b0076ddf3b38585d)

running 3 tests
test all_type_binary64_libraries_publish_finite_codecs_with_atomic_preflight ... ok
test root_selection_and_output_refusals_preserve_existing_files ... ok
test each_target_retains_the_same_model_selection_and_distinct_input_provenance ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/normalization.rs (target/debug/deps/normalization-47a1f33d5a547d54)

running 17 tests
test incompatible_later_destination_preserves_the_entire_existing_output ... ok
test unsupported_go_pattern_has_no_successful_partial_artifact ... ok
test output_symlinks_and_hardlinks_are_refused_without_mutation ... ok
test generated_paths_cannot_replace_canonical_source_inputs_or_follow_links ... ok
test stale_bundle_missing_source_and_unknown_dispatch_refuse ... ok
test output_cannot_replace_any_declared_input ... ok
test qualified_go_base64_pattern_is_published_without_decoding_the_value ... ok
test positional_cli_refuses_unused_branches_before_publication ... ok
test explicit_binary64_inputs_run_through_the_cli_without_weakening_version_one ... ok
test checked_recipe_and_run_are_deterministic_with_json_only_stdout ... ok
test generation_checks_refuse_before_creating_or_changing_destinations ... ok
test failed_check_or_execution_preserves_output_and_does_not_emit_a_result ... ok
test raw_json_capture_runs_before_schema_and_keeps_failure_output_untouched ... ok
test generated_libraries_match_the_api_and_drift_check_never_repairs_files ... ok
test positional_cli_prepares_text_and_refuses_before_publication ... ok
test model_sources_are_compiled_pinned_and_protected_by_the_cli ... ok
test binary64_cli_keeps_numeric_identity_and_emits_checked_format_five ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.00s

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

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s

     Running tests/normalization_typescript_adversary.rs (target/debug/deps/normalization_typescript_adversary-1ef9acb3dfcb1eba)

running 1 test
test generation_and_check_need_no_native_tools_and_module_refusal_preserves_bytes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/openapi_accounting.rs (target/debug/deps/openapi_accounting-3024341d0a106a96)

running 6 tests
test complete_projection_preserves_supported_yaml_through_both_spellings ... ok
test both_import_spellings_write_the_accounted_envelope_for_every_presentation ... ok
test a_refused_import_keeps_existing_output_and_creates_no_new_file ... ok
test legacy_projection_refuses_before_destination_mutation ... ok
test partial_projection_reports_the_durable_gap_and_preserves_destinations ... ok
test unresolved_projection_reports_the_exact_reference_and_preserves_destinations ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/openapi_adversary_pass1.rs (target/debug/deps/openapi_adversary_pass1-d2bcefe19c39a1dd)

running 2 tests
test cli_rejects_duplicate_keys_and_tampered_accounting_before_output ... ok
test cli_rejects_unknown_unit_fields_before_any_projection_output ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/openapi_adversary_pass2.rs (target/debug/deps/openapi_adversary_pass2-ae258705b1349988)

running 1 test
test same_path_projection_refusal_preserves_the_unadmitted_input ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/output_containment.rs (target/debug/deps/output_containment-5c56b28ecc91698b)

running 19 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-8deef286ecfcabd1)

running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

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

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s

     Running tests/schema_registry_identity_adversary.rs (target/debug/deps/schema_registry_identity_adversary-cf3f1a6623400628)

running 6 tests
test registry_admission_and_selected_typescript_projection_have_distinct_boundaries ... ok
test inlining_idless_generated_payloads_breaks_their_original_root_references ... ok
test decoded_duplicate_ids_are_collisions_and_filenames_supply_no_identity ... ok
test an_unselected_invalid_resource_blocks_both_documented_pairs ... ok
test envelope_definitions_cannot_shadow_either_payload_resource_root ... ok
test the_selected_schema_checks_its_selector_and_nonobject_payload_as_one_instance ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s

     Running tests/target_failure.rs (target/debug/deps/target_failure-7d9cc8e18982b595)

running 1 test
test fatal_synthesis_preserves_destinations_and_has_a_typed_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

     Running unittests src/lib.rs (target/debug/deps/ess_conformance-5e50d807cc4deaf5)

running 70 tests
test counts::tests::exact_unsigned_scalar_vectors_do_not_use_binary64 ... ok
test decision::tests::a_decision_reads_its_two_other_cases_as_neither_satisfied_nor_the_other ... ok
test decision::tests::exactly_one_reason_says_another_candidate_would_help ... ok
test counts::tests::payload_number_and_utf8_canonical_profile_is_frozen_separately_from_scalars ... ok
test evidence::tests::a_standalone_report_carries_every_field_an_adapter_needs ... ok
test decision::tests::a_refusal_renders_the_predicate_the_command_and_every_reason ... ok
test evidence::tests::report_readers_do_not_guess_a_producer_from_status_vocabulary ... ok
test evidence::tests::report_readers_refuse_more_nonpasses_than_executed_scenarios ... ok
test faulty::tests::a_fault_is_injected_into_the_system_that_declares_what_it_breaks ... ok
test evidence::tests::the_closed_report_round_trips_with_identical_canonical_bytes ... ok
test faulty::tests::no_two_faults_claim_the_same_scenario ... ok
test faulty::tests::every_fault_says_what_it_is_and_where_it_goes ... ok
test faulty::tests::only_the_two_faults_the_boundary_cannot_express_are_injected_in_the_implementation ... ok
test evidence::tests::unknown_report_fields_are_refused ... ok
test input::tests::every_primitive_projects_to_the_one_fact_value_that_can_hold_it ... ok
test evidence::tests::report_readers_refuse_nonpass_count_and_list_disagreement ... ok
test input::tests::shape_errors_render_one_per_line_and_name_the_input_root_by_name ... ok
test report::tests::a_scenario_status_is_the_strongest_of_its_checks_and_a_contradiction_outranks_everything ... ok
test report::tests::a_quoted_input_reads_as_the_call_that_was_made ... ok
test report::tests::an_unsupported_scenario_makes_the_run_fail_rather_than_look_like_a_pass ... ok
test go::tests::the_runner_is_a_constant_and_only_the_suite_moves ... ok
test report::tests::every_check_code_has_a_distinct_name_and_a_rule_sentence ... ok
test runner::tests::a_count_with_neither_bound_is_a_suite_defect_and_not_a_satisfied_assertion ... ok
test runner::tests::a_count_is_the_half_of_an_ordering_claim_that_says_the_rows_were_there ... ok
test evidence::tests::report_readers_preserve_go_producer_bytes_and_historical_nonpass_counts ... ok
test go::tests::every_go_file_is_in_the_package_the_readme_names ... ok
test runner::tests::a_nested_row_binds_the_paths_a_predicate_spells ... ok
test runner::tests::a_position_in_a_view_that_declares_no_order_is_a_suite_defect ... ok
test runner::tests::a_declared_order_is_checked_on_adjacent_rows_and_the_next_key_breaks_a_tie ... ok
test input::tests::a_primitive_refuses_a_node_of_the_wrong_shape_rather_than_coercing_it ... ok
test report::tests::a_diagnostic_answers_all_five_of_the_questions_a_failure_has_to_answer ... ok
test runner::tests::a_position_names_both_ends_and_a_row_that_is_not_there_is_not_a_match ... ok
test runner::tests::a_ranking_key_a_row_does_not_publish_is_undecidable_rather_than_out_of_order ... ok
test runner::tests::a_predicate_a_row_cannot_answer_is_reported_rather_than_retried ... ok
test runner::tests::an_empty_field_set_means_a_row_exists_and_not_that_anything_will_do ... ok
test runner::tests::an_order_over_fewer_than_two_rows_holds_and_does_not_double_as_a_non_emptiness_claim ... ok
test runner::tests::the_runners_clock_advances_on_every_read_so_a_deadline_can_bound_anything ... ok
test runner::tests::ids_come_from_the_suite_and_from_nothing_ambient ... ok
test scenario::tests::a_declared_leaf_admits_what_its_type_admits_and_absence_only_where_the_type_permits_it ... ok
test runner::tests::a_view_that_holds_nothing_does_not_satisfy_an_invariant_by_being_empty ... ok
test scenario::tests::a_purpose_is_one_line_and_says_something ... ok
test scenario::tests::a_scenario_id_names_the_construct_it_exercises_rather_than_its_position ... ok
test scenario::tests::a_semantic_reference_renders_the_way_the_design_writes_one ... ok
test scenario::tests::a_payload_shape_round_trips_through_the_form_a_suite_is_stored_in ... ok
test scenario::tests::a_scenario_id_that_names_no_construct_is_refused ... ok
test scenario::tests::a_suite_format_from_a_later_build_is_refused_rather_than_guessed ... ok
test scenario::tests::a_transition_ref_refuses_a_name_no_lifecycle_can_declare ... ok
test scenario::tests::a_suite_refuses_a_second_scenario_under_one_id ... ok
test scenario::tests::an_invariant_scenario_is_keyed_by_the_entity_and_the_branch_and_never_by_a_position ... ok
test scenario::tests::every_binding_aspect_is_in_the_list_that_is_walked_to_produce_them ... ok
test evidence::tests::report_readers_refuse_unknown_report_formats ... ok
test scenario::tests::the_ids_of_a_suite_sort_the_way_a_reader_sorts_the_file ... ok
test scenario::tests::two_scenarios_about_the_same_thing_in_the_same_way_are_one_id ... ok
test scenario::tests::every_scenario_id_reads_back_from_the_form_a_report_prints ... ok
test synthesize::tests::a_refusal_names_the_construct_the_code_and_the_repair ... ok
test synthesize::tests::every_refusal_carries_a_distinct_code_in_one_family ... ok
test web::tests::no_comparison_sits_in_a_text_node_mustache ... ok
test witness::tests::a_text_witness_is_its_own_path_so_two_fields_of_one_type_never_agree ... ok
test witness::tests::an_enum_offers_every_variant_it_declares_and_the_first_one_only_once ... ok
test witness::tests::an_integer_leaf_is_never_offered_a_fractional_candidate ... ok
test web::tests::the_page_calls_nothing_the_player_does_not_return ... ok
test witness::tests::the_alternatives_for_a_number_are_the_guards_own_literals_either_side ... ok
test witness::tests::the_candidate_count_is_bounded_however_many_fields_a_guard_reads ... ok
test witness::tests::two_uuid_witnesses_differ_and_neither_moves_when_a_third_field_appears ... ok
test evidence::tests::report_readers_refuse_malformed_and_unsupported_suite_versions ... ok
test evidence::tests::report_readers_refuse_nonpass_entries_without_a_known_nonpass_status ... ok
test web::tests::the_page_is_specification_neutral ... ok
test evidence::tests::report_readers_refuse_status_claims_that_contradict_the_list ... ok
test evidence::tests::report_readers_preserve_rust_producer_bytes_for_every_supported_suite ... ok
test coverage_build::tests::actual_duplicate_insertion_keeps_the_generated_survivor_and_refusal_through_execution ... ok

test result: ok. 70 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

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
test a_position_in_a_view_that_declares_no_order_is_refused ... ok
test a_command_the_model_does_not_declare_is_refused_by_name ... ok
test a_domain_the_model_does_not_declare_is_refused_by_name ... ok
test a_field_the_surface_does_not_declare_is_refused_by_name ... ok
test a_halt_claimed_of_a_listing_with_no_declared_order_is_refused_by_the_code_that_already_says_so ... ok
test a_document_that_is_not_one_is_refused_rather_than_read_as_an_empty_scenario ... ok
test a_declared_field_nothing_supplies_is_refused_by_name ... ok
test a_halt_after_no_rows_at_all_is_refused_rather_than_compiled ... ok
test a_positional_claim_takes_the_order_from_the_view_rather_than_from_the_author ... ok
test a_predicate_reading_something_the_view_does_not_publish_is_refused ... ok
test a_halt_stated_beside_another_claim_is_two_assertions_filed_as_one ... ok
test a_claim_the_timelines_own_instants_contradict_is_refused ... ok
test a_reference_where_the_suite_compares_a_value_it_carries_is_refused ... ok
test a_state_the_lifecycle_does_not_declare_is_refused_as_a_state_and_not_as_a_variant ... ok
test a_scenario_that_runs_nothing_is_refused_rather_than_counted_as_a_check ... ok
test a_halt_compiles_to_a_step_of_its_own_and_not_to_a_claim_about_rows ... ok
test a_scenario_compiles_to_the_id_the_domain_and_the_name_make ... ok
test a_bounded_negative_that_forbids_no_event_is_refused ... ok
test a_format_this_build_does_not_implement_is_refused_before_anything_is_read ... ok
test a_timeline_whose_instants_do_not_ascend_is_refused ... ok
test an_act_cannot_open_a_window_at_its_own_instant ... ok
test a_value_the_declared_type_does_not_admit_is_refused_where_it_sits ... ok
test a_view_the_model_does_not_declare_is_refused_by_name ... ok
test a_window_that_states_other_than_one_bound_is_refused ... ok
test a_window_of_no_seconds_is_refused_rather_than_compiled_into_a_check_that_cannot_fail ... ok
test an_error_the_model_does_not_declare_is_refused_by_name ... ok
test an_elapsed_claim_compiles_to_the_four_steps_that_carry_it_and_they_come_before_the_act ... ok
test an_actor_the_specification_does_not_grant_the_command_is_refused ... ok
test a_window_measured_from_an_instant_nothing_marked_is_refused_with_the_ones_that_are ... ok
test an_assertion_that_states_other_than_one_claim_is_refused ... ok
test an_actor_the_model_does_not_declare_is_refused_by_name ... ok
test an_entity_the_model_does_not_declare_is_refused_by_name ... ok
test a_value_read_off_an_event_nothing_required_is_refused ... ok
test the_steps_are_the_vocabulary_a_generated_scenario_already_uses ... ok
test an_instance_the_arrangement_does_not_declare_is_refused_by_name ... ok
test an_instance_bound_to_a_field_that_cannot_hold_an_identity_is_refused ... ok
test an_instance_named_before_anything_binds_it_is_refused ... ok
test the_committed_billing_suite_holds_the_authored_scenario_beside_the_generated_ones ... ok
test one_name_for_two_instants_is_refused_rather_than_read_as_the_later_one ... ok
test an_event_the_model_does_not_declare_is_refused_by_name ... ok
test the_order_the_files_are_handed_over_in_does_not_reach_the_result ... ok
test two_files_naming_one_scenario_are_refused_rather_than_one_displacing_the_other ... ok
test an_outcome_the_command_does_not_declare_is_refused_with_the_ones_it_does ... ok
test authored_predicate_operand_errors_have_their_own_refusal ... ok
test authored_aggregate_presence_keeps_026_and_valid_scalar_reads_keep_the_predicate ... ok
test two_compilations_of_one_file_produce_identical_bytes ... ok
test every_cause_is_reachable_from_a_document ... ok
test coverage_builder_retains_authored_duplicate_ownership_and_original_source_bytes ... ok
test independently_successful_authored_batches_are_refused_only_at_final_merge ... ok
test paired_browser_emission_retains_original_input_and_checks_the_actual_model ... ok
test rejected_authored_candidate_needs_are_proved_independently_of_an_outside_survivor ... ok

test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

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

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/coverage_writer_adversary_pass1.rs (target/debug/deps/coverage_writer_adversary_pass1-257bfdcc793f80b8)

running 2 tests
test d1_authored_inventory_preserves_full_original_typed_refusal_rendering ... ok
test d1_generated_inventory_preserves_full_original_typed_refusal_rendering ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s

     Running tests/coverage_writer_adversary_pass2.rs (target/debug/deps/coverage_writer_adversary_pass2-cf711d0e33f85f1e)

running 1 test
test final_merge_orders_sources_and_preserves_full_d1_diagnostics_through_empty_selection ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

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
test a_value_of_the_wrong_declared_type_is_caught_by_the_same_check_as_a_missing_one ... ok
test a_target_that_cannot_expose_an_observation_fails_the_run_rather_than_skipping_it ... ok
test a_view_assertion_names_the_instance_the_scenario_created_rather_than_any_row ... ok
test every_scenario_checked_something_and_no_family_of_them_was_silently_empty ... ok
test a_view_answered_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test an_event_missing_a_field_it_declares_is_named_leaf_by_leaf_rather_than_reported_as_absent ... ok
test every_scenario_the_billing_specification_obliges_passes_against_the_reference_implementation ... ok
test a_read_your_writes_view_is_not_quietly_read_at_current_when_no_token_came_back ... ok
test a_scenario_whose_input_no_longer_reaches_its_branch_fails_with_a_diagnostic_naming_the_defect ... ok
test two_runs_of_one_suite_against_one_target_produce_byte_identical_reports ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

     Running tests/faults.rs (target/debug/deps/faults-592913acd22499fb)

running 11 tests
test a_fault_nothing_catches_is_recorded_rather_than_quietly_dropped ... ok
test dropping_one_binding_leaves_the_other_two_green ... ok
test a_wrong_mapping_is_invisible_to_a_target_that_cannot_show_its_invocations ... ok
test the_diagnostic_of_a_caught_fault_names_the_defect_rather_than_reporting_that_something_broke ... ok
test every_fault_that_could_be_a_boundary_perturbation_is_one ... ok
test the_widest_blast_radius_is_scenarios_that_could_not_be_arranged_rather_than_extra_verdicts ... ok
test each_specification_is_passed_in_full_by_the_implementation_written_from_it ... ok
test two_runs_against_one_faulty_target_produce_byte_identical_reports ... ok
test a_fault_does_not_simply_break_everything ... ok
test a_faults_blast_radius_is_accounted_for ... ok
test each_fault_fails_the_scenario_that_exists_to_catch_it ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s

     Running tests/halt.rs (target/debug/deps/halt-e9156dff15c9e526)

running 7 tests
test a_halt_of_an_eventual_listing_is_asked_again_while_the_projection_catches_up ... ok
test a_target_that_reads_the_whole_listing_fails_rather_than_being_read_as_having_stopped ... ok
test a_listing_that_ran_out_before_the_reader_stopped_it_is_not_a_halt ... ok
test a_target_whose_producer_stops_when_the_reader_does_passes ... ok
test retrying_does_not_rescue_a_producer_that_never_stops ... ok
test a_target_that_cannot_read_a_row_at_a_time_reports_unsupported_and_the_run_fails ... ok
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
test a_suite_parses_from_text_alone_without_an_ir ... ok
test the_scan_for_a_clock_finds_one_and_does_not_find_a_word_that_merely_ends_in_a_banned_token ... ok
test a_count_and_a_position_read_back_as_what_a_runner_in_another_language_must_read ... ok
test the_steps_a_binding_and_an_invariant_need_survive_being_read_back_from_text ... ok
test sparse_models_cannot_publish_an_empty_success_for_binary64 ... ok
test every_scenario_id_the_billing_model_can_produce_reads_back ... ok
test the_dependency_set_names_a_type_no_derived_from_would_have_mentioned ... ok
test the_scenario_ids_appear_in_the_file_in_the_order_a_sorted_key_list_would_be ... ok
test a_suite_serialised_in_one_process_resolves_in_another ... ok
test the_step_vocabulary_expresses_the_worked_example_from_section_ten ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test typed_binary64_suites_refuse_serialization_emission_and_target_effects ... ok
test serialising_a_suite_twice_produces_byte_identical_json ... ok
test inserting_one_outcome_re_keys_nothing_around_it ... ok
test the_suite_records_the_same_model_digest_the_projections_do ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/synthesis.rs (target/debug/deps/synthesis-467793b7b98b58f7)

running 53 tests
test a_guard_no_candidate_can_satisfy_is_refused_with_the_number_tried ... ok
test a_binding_whose_branch_the_event_decides_refuses_the_flow_and_still_checks_the_mapping ... ok
test a_command_that_accepts_a_wrong_state_is_asserted_as_accepting_rather_than_refusing ... ok
test a_command_that_declares_no_wrong_state_answer_is_refused_by_name_beside_its_scenario ... ok
test a_filter_reading_something_no_scenario_knows_refuses_rather_than_guessing ... ok
test a_parameterised_view_is_queried_with_the_value_the_scenario_put_in_the_row ... ok
test a_state_reached_only_through_a_branch_no_input_reaches_is_refused_rather_than_arranged ... ok
test a_component_nothing_declares_is_refused_by_name ... ok
test a_binding_that_retries_forces_one_failure_and_still_requires_the_consequence ... ok
test a_read_your_writes_view_filled_by_the_command_that_ran_is_asserted_to_hold_a_row ... ok
test a_binding_that_drops_its_failures_refuses_that_check_and_names_the_reason ... ok
test an_entity_nothing_creates_cannot_be_acted_on_and_says_so ... ok
test a_move_is_observed_through_the_view_the_state_it_left_is_filtered_on ... ok
test a_binding_mapping_names_the_source_the_document_wrote_and_not_its_same_typed_sibling ... ok
test a_scenario_that_moves_an_instance_names_the_one_an_earlier_step_created ... ok
test a_declared_order_is_asserted_against_two_rows_the_scenario_arranged_itself ... ok
test an_order_the_specification_cannot_put_two_rows_under_is_refused_and_not_asserted ... ok
test a_declared_error_is_asserted_by_name_and_never_by_an_invented_payload ... ok
test a_value_object_nothing_observable_holds_keeps_a_refusal_naming_what_would_close_it ... ok
test a_value_objects_own_invariants_are_read_at_every_field_position_a_view_holds_one ... ok
test a_view_is_asserted_in_the_block_its_own_consistency_decides ... ok
test a_synthesised_count_is_a_floor_the_scenario_arranged_and_never_a_ceiling ... ok
test a_move_that_is_illegal_in_a_state_is_attempted_with_the_input_that_would_have_worked ... ok
test a_view_the_entity_has_not_reached_yet_is_asserted_to_exclude_the_instance_by_name ... ok
test an_undecidable_guard_refuses_and_does_not_spend_the_candidate_budget ... ok
test a_binding_flow_is_proved_through_the_event_the_invoked_command_publishes ... ok
test an_invariant_over_a_field_no_view_publishes_refuses_rather_than_being_dropped ... ok
test a_binding_that_escalates_requires_the_event_the_escalation_declares ... ok
test an_actor_is_named_only_where_the_specification_grants_the_command ... ok
test a_view_that_does_not_hold_the_instance_yet_is_not_asked_about_its_invariants ... ok
test an_invariant_is_asserted_against_every_view_that_publishes_what_it_reads ... ok
test an_outcome_that_updates_an_instance_acts_on_one_the_scenario_created ... ok
test a_suite_for_one_component_holds_only_what_that_component_realises ... ok
test a_synthesised_suite_survives_being_written_and_read_back ... ok
test a_whole_system_suite_does_not_mention_a_component ... ok
test an_event_assertion_carries_the_declared_shape_and_exactly_the_values_the_payload_determines ... ok
test an_at_least_once_binding_delivers_the_event_twice_and_requires_no_count ... ok
test an_illegal_move_requires_the_branch_and_the_declared_error_rather_than_merely_failing ... ok
test the_failure_control_is_armed_after_the_arrangement_and_before_the_command_that_triggers_it ... ok
test every_declared_transition_has_a_scenario_that_proves_it_can_occur ... ok
test the_dependency_set_names_the_types_the_scenario_is_made_of ... ok
test an_outcome_no_input_decides_is_reached_by_injection_and_by_nothing_else ... ok
test every_declared_outcome_is_either_a_scenario_or_a_named_refusal_or_asserted_by_the_state_family ... ok
test every_command_names_an_instance_an_earlier_step_of_the_same_scenario_bound ... ok
test the_input_a_scenario_sends_is_re_decided_against_the_guard_it_claims_to_reach ... ok
test the_refusal_branch_asserts_that_no_event_the_specification_declares_occurred ... ok
test each_example_synthesises_the_families_its_specification_declares ... ok
test every_clause_of_every_binding_is_either_a_scenario_or_a_named_refusal ... ok
test synthesising_the_same_specification_twice_produces_byte_identical_output ... ok
test coverage_builder_records_the_complete_generated_inventory_and_component_omissions ... ok
test coverage_all_missing_invariants_keep_null_survivors_and_component_proofs_stay_conservative ... ok
test canonical_expression_compatibility_fixtures ... ok
test coverage_missing_lifecycle_and_view_checks_remain_beside_actual_passing_results ... ok

test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s

     Running tests/witness.rs (target/debug/deps/witness-9e1a2f8f8747eefd)

running 28 tests
test a_candidate_input_projects_one_fact_per_scalar_leaf ... ok
test a_newtype_is_transparent_so_a_deep_path_reaches_through_it_without_a_segment ... ok
test a_candidate_carrying_a_field_no_type_declares_is_refused ... ok
test a_candidate_missing_a_required_field_is_refused_before_any_guard_is_read ... ok
test a_conjunction_of_two_undecidable_leaves_reports_both ... ok
test a_disjunction_one_of_whose_branches_holds_is_satisfied_despite_an_undecidable_branch ... ok
test a_list_a_map_and_a_union_bind_no_fact_in_the_current_projection ... ok
test a_refuted_guard_carries_the_leaf_and_the_value_that_refuted_it ... ok
test a_refusal_names_the_predicate_the_command_and_the_path ... ok
test a_scalar_of_the_wrong_shape_is_refused_rather_than_coerced ... ok
test a_path_landing_on_an_aggregate_is_unevaluable_by_construction ... ok
test a_newtype_is_transparent_when_a_path_is_resolved_as_well_as_when_it_is_projected ... ok
test a_path_into_a_list_or_a_union_names_the_aggregate_rather_than_the_missing_element ... ok
test an_absent_optional_is_unevaluable_but_says_a_candidate_could_repair_it ... ok
test an_absent_optional_binds_nothing_rather_than_binding_a_default ... ok
test equality_over_two_texts_is_decided_even_though_ordering_them_is_not ... ok
test legal_collection_cardinality_is_not_currently_projected ... ok
test a_long_recursive_read_validates_beyond_the_projection_limit ... ok
test expression_search_limits_do_not_define_type_correctness ... ok
test ordering_two_texts_is_unevaluable_because_an_ess_specification_declares_no_scale ... ok
test the_normative_shape_of_guard_is_decidable_for_both_signs ... ok
test ordering_across_two_types_is_unevaluable_not_false ... ok
test otherwise_and_external_are_not_guards_over_the_input ... ok
test resolved_adapter_keeps_semantics_separate_from_collection_projection ... ok
test only_an_absent_value_says_another_candidate_would_help ... ok
test unclassified_is_a_drift_alarm_and_no_enumerated_source_trips_it ... ok
test the_same_text_ordering_is_decidable_once_a_scale_contains_both_values ... ok
test malformed_declarations_refuse_early_and_direct_bad_reads_remain_unknown ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/lib.rs (target/debug/deps/ess_diff-06167d7f60ffb115)

running 9 tests
test change::tests::the_canonical_order_is_the_category_order_and_not_the_alphabet ... ok
test delta::tests::a_delta_puts_its_changes_in_canonical_order_however_they_arrive ... ok
test change::tests::only_a_grant_and_a_variant_decide_a_direction ... ok
test change::tests::a_change_with_no_member_renders_three_parts_rather_than_a_trailing_slash ... ok
test change::tests::a_change_id_names_its_category_subject_subtype_and_member_in_that_order ... ok
test impact::tests::a_whole_answer_absorbs_a_narrowing_whichever_way_round_they_are_joined ... ok
test impact::tests::an_unfollowed_file_is_not_an_artifact_that_owes_regeneration ... ok
test impact::tests::a_change_to_the_specification_itself_owes_the_whole_suite ... ok
test impact::tests::a_suite_resting_on_a_construct_the_graph_has_no_node_for_owes_the_whole_suite ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/artifacts.rs (target/debug/deps/artifacts-fe289ed3b5544f1f)

running 13 tests
test an_artifact_whose_slice_nothing_reached_is_absent_from_the_answer ... ok
test a_grant_change_owes_the_documents_that_read_grants_and_not_the_ones_that_do_not ... ok
test a_change_to_the_system_header_owes_every_artifact ... ok
test whole_model_artifacts_are_owed_by_any_change_at_all ... ok
test the_artifacts_the_currency_changes_reach_are_owed_and_named ... ok
test the_two_predicate_edits_narrow_the_artifacts_differently_and_both_subsets_are_named ... ok
test the_six_change_delta_owes_a_strict_subset_of_the_artifacts ... ok
test an_owed_artifacts_path_explains_the_membership_hop_by_hop ... ok
test a_committed_artifact_with_a_false_contract_digest_is_owed_as_a_false_claim ... ok
test a_committed_tree_is_answered_for_fail_closed_file_by_file ... ok
test review_whole_model_hashes_and_index_bytes_remain_frozen ... ok
test the_artifact_answer_is_byte_identical_between_runs ... ok
test review_legacy_slice_stamps_are_owed_even_when_raw_hashes_match ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s

     Running tests/canonical.rs (target/debug/deps/canonical-ce09a7a45878a366)

running 20 tests
test a_binding_still_has_one_delivery_a_document_can_write ... ok
test every_change_variant_has_something_to_say_for_itself ... ok
test review_freeze_legacy_delta_bytes ... ok
test a_change_is_spelt_the_same_way_in_its_id_and_in_the_document ... ok
test review_version_admission_refuses_new_vocabulary_in_legacy_envelopes ... ok
test no_source_file_in_the_diff_engine_reads_a_clock_or_an_unordered_map ... ok
test no_source_file_in_the_diff_engine_calls_an_ir_handle_accessor ... ok
test a_system_still_has_no_naming_a_document_can_set ... ok
test a_delta_whose_id_was_edited_is_refused ... ok
test a_delta_whose_changes_are_out_of_order_is_refused ... ok
test a_delta_written_in_a_format_this_build_does_not_read_is_refused ... ok
test a_delta_this_build_wrote_is_read_back_without_complaint ... ok
test every_change_in_a_delta_has_its_own_id ... ok
test the_changes_are_written_in_the_category_order_and_not_the_alphabet ... ok
test canonical_json_ends_in_a_newline ... ok
test a_delta_naming_two_systems_is_refused_on_the_way_in_as_well ... ok
test a_delta_whose_relation_was_edited_is_refused ... ok
test review_new_default_delta_format_is_version_two ... ok
test a_document_with_six_defects_reports_six ... ok
test diffing_the_same_pair_twice_produces_byte_identical_json ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/families.rs (target/debug/deps/families-8952d48644da49b5)

running 69 tests
test a_bindings_naming_is_compared_key_by_key ... ok
test a_commands_naming_is_compared_key_by_key ... ok
test a_filter_that_contains_different_instances_is_changed_with_no_direction ... ok
test a_component_that_no_longer_publishes_an_event_is_reported ... ok
test a_filter_respaced_is_the_same_predicate_and_no_change ... ok
test a_construct_moving_between_files_is_not_a_change ... ok
test a_bindings_failure_policy_is_compared ... ok
test a_guard_respaced_is_the_same_predicate_and_no_change ... ok
test a_filter_removed_reads_as_containing_every_instance ... ok
test a_payload_declaration_arriving_is_a_payload_change ... ok
test a_binding_invoking_a_different_command_moves_its_mapping_with_it ... ok
test a_binding_reacting_to_a_different_event_is_reported ... ok
test a_mapping_filled_from_somewhere_else_is_reported_with_both_sources ... ok
test a_command_added_is_one_change ... ok
test a_component_accepting_a_new_command_is_changed_and_not_widened ... ok
test a_binding_added_is_one_change ... ok
test a_new_transition_arrives_with_the_outcome_that_takes_it ... ok
test a_newtype_that_wraps_something_else_is_reported ... ok
test a_struct_field_that_changed_type_is_reported ... ok
test a_type_that_became_a_different_kind_of_thing_is_reported_as_that_and_nothing_else ... ok
test a_views_consistency_promise_is_compared_and_not_classified ... ok
test a_view_projecting_a_different_entity_is_a_source_change ... ok
test an_entity_fields_naming_is_compared_key_by_key ... ok
test a_union_variant_that_carries_something_else_is_not_a_variant_removed_and_added ... ok
test a_union_that_is_tagged_by_another_field_is_reported ... ok
test a_view_fields_naming_is_compared_key_by_key ... ok
test an_entity_field_that_changed_type_is_reported ... ok
test a_types_own_invariants_are_reported_as_different_and_never_as_stronger ... ok
test an_entity_added_arrives_with_its_synthesised_state_enum_and_nothing_is_diffed_inside ... ok
test an_actor_declared_with_no_grants_at_all_is_still_a_change_to_report ... ok
test a_union_gaining_a_variant_widens_it_just_as_an_enum_does ... ok
test an_entity_field_replaced_is_removed_and_added_and_never_a_rename ... ok
test a_view_added_is_one_change ... ok
test a_views_naming_is_compared_key_by_key ... ok
test an_event_field_that_changed_type_is_reported ... ok
test a_view_exposing_a_new_field_is_reported_with_the_type_it_carries ... ok
test an_entitys_naming_is_compared_key_by_key ... ok
test an_error_that_gained_a_field_is_reported_with_the_type_it_carries ... ok
test an_identitys_display_name_and_summary_are_compared ... ok
test an_event_renamed_is_reported_as_removed_and_added_and_never_as_a_rename ... ok
test an_input_added_is_reported_with_the_type_it_carries ... ok
test an_input_fields_naming_is_compared_key_by_key ... ok
test an_outcomes_summary_is_compared ... ok
test reordering_a_commands_input_is_reported_once ... ok
test reordering_a_commands_outcomes_is_a_real_change ... ok
test renaming_an_entitys_identity_is_the_one_rename_this_crate_reports ... ok
test an_invariant_statement_reworded_without_moving_the_predicate_is_still_a_change ... ok
test an_outcome_added_is_one_change_and_claims_no_direction ... ok
test reordering_an_enums_variants_is_reported_without_claiming_a_direction ... ok
test reordering_a_views_fields_is_reported_once ... ok
test reordering_an_entitys_fields_is_reported_once ... ok
test an_events_wire_name_moving_is_not_the_event_moving ... ok
test an_input_that_changed_type_is_reported ... ok
test reordering_an_event_payload_is_reported_once_and_not_as_a_field_change ... ok
test the_error_a_branch_reports_is_compared ... ok
test writing_out_a_naming_default_is_not_a_change ... ok
test the_paragraph_saying_what_the_system_is_is_compared ... ok
test the_specifications_version_moving_is_reported_and_is_not_the_identity ... ok
test what_an_outcome_emits_is_compared_in_order ... ok
test what_an_error_tells_the_caller_is_compared ... ok
test review_outcome_refusal_is_independent_of_its_error ... ok
test review_reach_is_a_change_without_an_unrelated_surface_edit ... ok
test review_view_parameter_naming_is_compared_without_a_filter_edit ... ok
test review_cli_top_level_grouped_views_and_binary_are_changes ... ok
test review_view_ranking_is_compared_without_a_filter_edit ... ok
test review_outcome_sets_are_independent_of_event_payload ... ok
test review_residual_refs_cannot_hide_beside_a_classified_change ... ok
test review_unclassified_transition_order_cannot_hide_beside_a_classified_edit ... ok
test review_relation_cardinality_name_and_removal_are_changes ... ok

test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.60s

     Running tests/graph.rs (target/debug/deps/graph-746a5caf03f88bea)

running 10 tests
test correction2_row_shape_is_a_distinct_dependency_and_survives_graph_union ... ok
test the_graph_records_the_reference_an_author_wrote_and_not_its_reverse ... ok
test a_type_is_reached_through_the_declarations_that_hold_it_and_not_by_name ... ok
test a_component_is_reached_through_what_it_accepts_and_publishes ... ok
test a_closure_over_the_whole_model_terminates_and_stays_inside_it ... ok
test building_the_same_graph_twice_produces_the_same_edges_in_the_same_order ... ok
test correction2_network_exposure_matches_actual_routes_and_owned_domains ... ok
test review_cli_views_and_parameter_types_are_forward_slice_dependencies ... ok
test review_relation_edges_include_the_reverse_owns_carrier_and_old_graph_union ... ok
test every_relation_in_the_vocabulary_is_minted_by_a_specification_this_repository_ships ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/impact.rs (target/debug/deps/impact-1ac5d4ffb424fb86)

running 17 tests
test a_suite_produced_from_the_later_revision_is_refused_rather_than_narrowed ... ok
test two_specifications_of_different_systems_are_refused_here_too ... ok
test raw_legacy_impact_cannot_discard_a_coverage_inventory ... ok
test a_suite_whose_contract_digest_its_model_does_not_compute_is_refused ... ok
test a_suite_for_another_system_is_refused ... ok
test a_variant_removed_from_an_enum_reaches_the_entity_that_holds_it_transitively ... ok
test a_narrowed_answer_never_reports_more_scenarios_than_the_suite_holds ... ok
test the_suite_the_fixture_obliges_is_ten_scenarios_and_the_delta_is_six_changes ... ok
test an_edited_outcome_guard_owes_every_scenario_because_every_scenario_creates_through_it ... ok
test an_edited_entity_invariant_owes_every_scenario_that_rests_on_the_entity_and_no_other ... ok
test every_scenario_resting_directly_on_a_changed_construct_is_owed_again ... ok
test a_suite_resting_on_a_construct_no_graph_has_a_node_for_owes_the_whole_suite ... ok
test taking_a_grant_from_an_actor_owes_only_the_scenarios_that_act_as_that_actor ... ok
test analysing_the_same_pair_twice_produces_byte_identical_json ... ok
test a_domains_naming_moving_owes_the_whole_suite_because_no_family_compares_a_domain ... ok
test a_change_in_a_family_the_delta_still_does_not_compare_owes_the_whole_suite ... ok
test coverage_impact_keeps_exact_selection_context_out_of_persisted_impact3 ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s

     Running tests/review_adversary_f01.rs (target/debug/deps/review_adversary_f01-233b78233bca1148)

running 7 tests
test explicit_domain_naming_defaults_remain_semantically_equivalent ... ok
test complete_generated_and_authored_suite_four_bytes_remain_frozen ... ok
test moved_outcome_reference_retains_its_owner ... ok
test relation_delta_versions_refuse_relabeling_and_public_serialize_bypasses ... ok
test moved_outcome_reference_is_independent_of_a_classified_edit ... ok
test ownership_cardinality_invalidates_both_emitted_schema_ends ... ok
test incomplete_schema_stamp_is_owed_by_the_real_impact_reader ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.64s

     Running tests/review_adversary_f01_pass2.rs (target/debug/deps/review_adversary_f01_pass2-46209d7fd7b87abf)

running 5 tests
test reusable_row_type_belongs_to_the_view_slice_it_supplies ... ok
test ranking_precedence_survives_the_checked_delta_roundtrip ... ok
test switching_equal_row_shapes_retains_independent_residual_coverage ... ok
test reusable_row_invariant_change_reaches_its_openapi_artifact ... ok
test served_view_change_reaches_its_openapi_artifact ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/revision_pair.rs (target/debug/deps/revision_pair-ba032edad7953c23)

running 11 tests
test granting_a_command_to_an_actor_widens_what_the_system_permits ... ok
test two_different_systems_are_refused_rather_than_reported_as_a_rewrite ... ok
test the_fixture_pair_differs_by_exactly_six_changes ... ok
test a_revision_compared_with_itself_reports_nothing ... ok
test removing_an_enum_variant_narrows_the_type_that_accepted_it ... ok
test nothing_the_after_revision_only_rewrote_reaches_the_delta ... ok
test taking_a_command_from_an_actor_narrows_what_the_system_permits ... ok
test adding_an_enum_variant_widens_the_type_that_accepts_it ... ok
test rewriting_an_outcomes_when_is_changed_and_renders_both_guards_canonically ... ok
test rewriting_an_entitys_invariant_is_changed_and_quotes_both_statements ... ok
test the_delta_survives_being_written_and_read_back ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests ess_conformance

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ess_diff

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p ess-cli --test coverage_writer_adversary_pass2`
```

4. Findings and the explicit scope disposition

No implementation finding is returned. The findings table has no rows, and the final machine-readable list is []. The current red is reported as a test scope issue; no unsupported runtime failure is promoted to an INFEASIBLE or confirmed product defect.

What was measured: the first Firefox assertion at the initial unformatted test line 114, now final line 102, fails because returned view-expectation fields at owner indices 6 (contains), 7 (excludes) and 9 (at) retain NumberToken objects for their numeric leaves. The actual inner original suite bytes contain finite numeric values; the returned observational JSON exposes {"raw":"9.0"} under constructor.prototype and {"raw":"2.0"} at the second element of the NUL-key array. Arbitrary key identities and string/boolean/null values remain exact. browser-scope-final.json records all four actual browser observations and confirms that the same three indices differ on every execution. Their actual complete payload is reproduced below, not inferred from the implementation:

```json
{
  "\u0000": [
    null,
    {
      "raw": "2.0"
    }
  ],
  "__proto__": {
    "sentinel": "retained"
  },
  "constructor": {
    "prototype": {
      "raw": "9.0"
    }
  },
  "hasOwnProperty": false,
  "toString": "ordinary data",
  "\ue000": "BMP key",
  "\ud83d\ude00": "astral key"
}
```

What reaches the real contract: coverage-admission.js:268–284 normalizes contains/excludes/at field values through values/node for typed semantic meaning; step at :308–333 invokes it, and parentPair compares full surviving meaning before the return. The twelve actual negative controls refuse for changed full scenario definitions/dependencies, including these three owners. That is the required lossless admission/lineage boundary. The final reduced return at :505–512 rebuilds from the original parsed document and normalizes top-level input/params/payload/fields only. Its only product consumer found, coverage-player.js:73–102, does not execute expect_view/eventually_view and discards these steps while grouping acts. No supported caller was found that treats those three unused nested returned fields as normalized execution values.

The main binding at docs/design/review-conformance-coverage.md:91 keeps arbitrary Node keys as payload and adds no execution semantics; :169 specifies finite Number payload meaning. The current coordinator transport at :190–192 explicitly preserves reduced view behavior, and :196–199 binds full pairing and separate finite payload-number semantics. These do not establish a normalized public execution consumer for unused returned view objects. The scope conclusion preserves story:review-browser-replay-fidelity/F15; it does not claim that story is fixed or that a stronger view consumer already exists. The bounded caller search is not a whole JavaScript/Rust callgraph proof.

Root separately reviewed the same owners and supplied the explicit scope disposition retained byte-exact as root-scope-decision.md, SHA256 40a8a9ec24bec9c962af73ae3cfcd20f6f1e0647dd18b766bc8c0c16662ead77. It requires recording this complete red report unchanged before routing a new-test-only correction: retain exact normalized equality for indices 0,1,2,3,4,5,8,10,11; assert the exact observed token-bearing payload at 6,7,9; retain all seven keys, all twelve full-lineage mutations, own-property assertions and the F15 reference. I did not apply that correction. It is root-owned later verification, not a third attack and not a production behavior decision. All current red test bytes, exact argv and outputs remain retained.

The earlier D1 finding's projection class has two production renderers: generated refusal.to_string captured before field moves at coverage_build.rs:213, and authored refusal.to_string at :467. Authored compilation and final batch rejection both use the latter. The real six-permutation case and unchanged first-review regressions establish the exercised generated/authored/final-merge controls. Their occurrence/ownership/effect/source rules were not altered. The Go diagnostic's sole strict status emission at runtime.go:3692–3698 selects the old explanation only when coverage is absent; all four original source-review regressions now execute green. No new finding signature is carried or invented here.

5. Boundaries attacked without an implementation finding

- Actual original-byte parent/child admission at twelve Node owners preserves arbitrary keys and rejects every changed surviving nested value with an accurately rehashed child identity.
- Original checked relative-source ordering controls six independently compiled batch permutations and preserves full original D1 messages/occurrences through explicit empty selection and carrier re-admission.
- All inherited paired generic Firefox, retained old-player, exact lineage/integer/default, D7 selected-int versus omitted-parent, strict/count/terminal/clock, conservative impact, legacy bytes and producer-export cases remain green in the actual full package execution.
- The existing 95 lineage and 314 closed-model vectors remain internal loops within their Rust cases. The new 12 mutation vectors and six merge permutations are likewise not separate Rust test counts.

Intake reused the complete prior same-thread read of the unchanged original implementation and cumulative source diff at 874962d3, plus the complete correction delta and both previously authored test files. prior-read-source-reconciliation.json rehashes the prior 1,100 input files: only the three reviewed corrected owners differ; the new exact D1 fixture and two frozen pass-1 tests account for the three added tracked files. The current cumulative-source.diff contains the complete base-to-corrected subject; the four-path correction.diff is retained separately. Current AGENTS/story, accepted coordinator bindings including D1/D7, the original report, pass-1 report and corrected report are pinned as authority snapshots. The correction report's full narrative, correction class, failed helper assumption/lint, retained regressions and command totals were inspected; repetitive raw suite output is retained verbatim in its unchanged full report. No claim of a new whole-program proof or fresh baseline run is made.

All 1,103 frozen source hashes and snapshots, both inherited adversary files, 35 original implementor seal entries, 16 first-review seal entries, 21 correction seal entries and 13 authority originals/snapshots were rechecked. These are direct payload/seal checks, not a claim to have freshly rehashed every payload in root's 14,879-entry correction census. The original plan/catalog/legacy oracles are unchanged tracked fixtures. Exact original reviewed reports remain untouched.

The actual existing producer-export tests wrote fresh exports beneath this review's assigned exports-package-suite* directories; their source-driven controls are part of the package lane. No mapper or AEP helper ran and no report was parsed to derive new producer semantic expectations. Their outputs do not establish root's still-unexecuted final AEP correspondence. No Git/store/lifecycle/gate/publication/integration/cleanup or third attack ran. No files in the active coordinator were changed.

Tool versions were measured: stable Rust/Cargo 1.98.0, Node 24.20.0, Go 1.26.5-X:nodwarf5 linux/amd64, Firefox 153.0. Every test/lint command receipt records original argv, assigned working directory/cache/TMP variables, start, elapsed time, exit, free space and new test source hashes. The unit target is used with two jobs, locked/offline, debug/incremental off and wrappers/target override unset. All measured free space remains above 8 GiB. Scratch scripts only assemble command/evidence records; no product Python/shell checker was introduced.

All final CLI/new-test Rust binaries are copied under retained-binaries with hashes. The pre-refactor binaries and source copies preserve earlier producing states. The first unformatted source is retained, but its exact first browser-test executable was not separately copied before the later formatting rebuild; no separate first-executable hash is invented. All raw Firefox command/profile/PID/BiDi receipts and actual outputs are retained. Initial large read outputs were truncated in the tool display and followed by bounded source/prose reads; no truncated output is substituted for a retained raw runner lane.

6. Every retained path written outside the worktree

The only assigned external root is home-path:sha256:8e39b53721833efacf3fb8c451d6094cf53737a1be914958acfbb4efc39d2f88 The catalog records native-byte names, lstat mode/UID/GID/inode/device/time, file hashes and literal symlink targets without following them. Paths created and internally retired by Firefox are limited to its retained command/profile/BiDi observations; this post-run census is not a syscall trace. No external path was cleaned up. The complete retained path list, including the root, follows:

```text
home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea
home-path:sha256:f904e609188ecd64e79c1f76e8df2f002d943b9d2547cc25cdd8c6419d48ef10
home-path:sha256:172d509d312fb15fb8cd7e0288124cd3733b64ffcd41e558c07d2b237e03d87c
home-path:sha256:ac1f567aae18505a08193637a0a4f5c5f7e6f78057a9aed9fcfae666cdd981b7
home-path:sha256:f53fce6aaf0c585fc860774facfa18f90c4118326f25ac8b6f8862274ec56b0f
home-path:sha256:df5f6eca8095b4613b9b1280f0c557da0833803b4bb4a5eb4dd18505c99e58ed
home-path:sha256:a4d5f55db7e06000fac05e165e4c265a76e86e62e242563373946258541fe464
home-path:sha256:6d712b5b39c83d648ff0cef8cf6257464bdfab027e79b3f05c934b872b1390ee
home-path:sha256:6104bc094084ddd780475350b6495e0a356c1cc3f50841141fd7bf08d9c76c7b
home-path:sha256:b7d6d51f333fbacb5accc3f08a6d09052e1fae8adc732b2f519e4348abb87e08
home-path:sha256:f4bd133d6f0b813edf2408ca3fd795fab0fa5eabc6d30cd4cde84c7143f42524
home-path:sha256:b5de1c5cbd79ba0e9e415b2da5762216551e17fd72ce0e3cdccb845bd65cff8a
home-path:sha256:3f0519593be09d6685b50e57dda6768c99973727a5c720bb701df7c875e75d36
home-path:sha256:f1401b86e5c9cd39b80dbff6873d27ab267da3d126b18ac620c5da0bb0c12c2d
home-path:sha256:136220d17c4dd0bcc268cb8fe3a6460c262a3955d6a690bc386bba8074254b1c
home-path:sha256:f51fea69668bbd216e5fae4474d0354d890e4100e3af39eee5c3efa10d6bdadc
home-path:sha256:824b71ec7a9bec34c5c514ebe60c72eb2592a14ad42fbfd2188a14a9a778a8f6
home-path:sha256:66abce5657e36a0cc04ae2bfbd1fed03d58ad854f7abe0fed12d6de40951030d
home-path:sha256:1985fdbf91f4ee4283f0fa15e9dfe407aa6a1b4bec418e6c3935949fd18f2569
home-path:sha256:81ac3dd4e3b8d537c46702b5ced5be14262463ddccbda01933194229f03b5ace
home-path:sha256:2eb6507379c2c21e7d418649923a8cc22934f6654029ce4ce8d9b7ff13816e11
home-path:sha256:d46d4cb9f225fc8947006ef5f698aefaf1f27c568525c05a329f1ebcb86689ee
home-path:sha256:ce927269455dc6ad3e922a52606a01ff72b193c33bc5eb486e86468f3745d7e4
home-path:sha256:f4907dd39a54fdcaef020181bf4cde259ff1ef6e3f78507016c48c7c5cc268da
home-path:sha256:17c0255dd99498475659a76259f810531c261d4f525b5d427b80c608d8e3760f
home-path:sha256:847a3fb6c3f8c2fb975661bf59a712da1e3b1594f4d6f1244a9e33fd48b48378
home-path:sha256:dd31535059a19a272e77bb71d1da7c1dbdc8b0960f636f7a8e58d6ddb54ede26
home-path:sha256:97fec8b569bec7e0999e10afaf6822fdf535eea8460cbe069a17713869db0600
home-path:sha256:4f735397e15255be50c27b95560504b9c3425d4797f23d24aaea5634134f9b0a
home-path:sha256:98c94364454abd34474d1aff700c122829e33669a59bc967900b01b16bd6f767
home-path:sha256:cb845b998a7d038f1eda850967ff30436dd77884b146f5cfda75b5d9a7798334
home-path:sha256:202f8fd3d1056a6148b98982fe0ca835a7e7f41afa87eb00b4d7ccecf1b4a568
home-path:sha256:48437166fa729fe0e6a6cd96d2261e066865edefd3a12ae4a04ac62260c78452
home-path:sha256:31f17c41b7e80ae007ed5f95515b6c6164f0f96e2767b082c661e5aa5d48172e
home-path:sha256:7ecc6a1a7014eef6d966b2c29a29d80ebd3622a7adfb2f67dbb15e01a9677ebc
home-path:sha256:1d48108a2af3b3cd751800ddcf46f33cdcdb91c247aecac95271ec8799186927
home-path:sha256:63579ceac6237727b479cc22eaacca949c4bd6e21aecb6056e8db87e5b9862c5
home-path:sha256:d4cd59f487373836217021b2d5a95deb5a5a4d8ab50aa1c176eb132332e51cb3
home-path:sha256:c29d7f7eb0fc2d8d5ab719e3f2c753c38a628eae2017cb8ca36334445a39bc00
home-path:sha256:37a0c94e821dfb6fcc478e4679ed50da05ed160c775ad3b7c1160789f03bfaec
home-path:sha256:57d7fcdeacdf75b1e330fabf4ffad3505151f525ab1091b03d572bb4bda8c7bf
home-path:sha256:48412fde322290ed40eaa96c3365c63a73dcfbf01971fd7bc7af1c3205aaff20
home-path:sha256:ee9f56a8d09596c80c67cfbd194dee6a6b06c700862b372edbcf6d49516d3388
home-path:sha256:d7acf03c71b36243c42ba8c4067b6e06974d3008599ca7e9524c8067d207ecc9
home-path:sha256:8d79e3dbe4e7fb0ff187de0088076f8f0a3c090df8b9c6310f3a8d7b976f913e
home-path:sha256:3ef6fad1bf8976f942506f2c0048cb2578f3d66f633babcc5664b927f62ef2d4
home-path:sha256:12a30f715f7a255193aca4d4e3f97f37864e7a395f22890fa52933810663034d
home-path:sha256:0a5433c301e8b3795c770626152ca47c831e3401fe3b042ca2856a93b745ed84
home-path:sha256:bf7665466b1437d7efd6b1ee90edf9c4499c6bea43b39fcda0fb9d3f919ffb82
home-path:sha256:662a46d9a3f7e9533e90df471a0e1e95dbba12f723fe18f22dbf394418eaac6b
home-path:sha256:c714a78290e48ae973a4f8b95f6c00a56e881ee220074774536dee74ff55d309
home-path:sha256:0ae54e4c243650255a8537725588efd0833fcb67b020fc691287e2989d1944f3
home-path:sha256:4b54f4cc72ce053d0482fb740a4998d06ad6f00233664583d09255ff4ebd4415
home-path:sha256:07b8551bf6145360afa81087e08a2c4c649d36e61185e30528eb6f8e9a220bbf
home-path:sha256:bd3973031ee44d46251c4ef351605682ab850f4411916bc11d12cb3dba8249e7
home-path:sha256:baa8000efc8efdd565f6ba9cc0a52f58d66b4775e12d1a0a8045bc742d4d3025
home-path:sha256:6285b8c1120ef60ebf680033d4cbba146b23f466e685b968fcd4f8b8e215dd8a
home-path:sha256:4c67408d3e2cdfaf4fe246441fd0c8c54182d34227acc7b95a8748078459bdb8
home-path:sha256:6f4be23e15d4755aa1c5ec8bfa59f806f20d2573af8247cb8175f3c6d61c9703
home-path:sha256:6898d6b80b541ae9078d04b3ee3b4648cfcb37851a38d3562ff4ad13f40c51e0
home-path:sha256:05c28d9e1725a07a72018307101d95496d031d8636162311e2059f41048d5e58
home-path:sha256:d0fd79407392b9460ff9b2355939f8540877661f087fd858b1c9b5efed01c35f
home-path:sha256:99b2dc2260d167be6a1f9377d189bcbe60fd386cb19fb3e67d8f806c3e7efa16
home-path:sha256:b06903b48b88e627a3b5081738a70585fd8393e8342d539391b10e7623072aef
home-path:sha256:ad0e21b74c0f876e4e867d9608ce0d6e21dd2e5e138a9db2719b0ed68e4d3ca2
home-path:sha256:54b37a9443b6e8573210cd2adecff959ebc14b8e3934ac52c980e2752166e31c
home-path:sha256:da47b7abf0ffa6088b093a2f54a7d6200586c204f9a89bb27d4a3119c4666862
home-path:sha256:8ba49bc130a0c05465659f04842c582b67bf4e8b347dd21713e83825ab388df7
home-path:sha256:0f2c10a35296eb9cb62b9626ccfd5314ddcac286f5e43c9955064ac5e3a9d91b
home-path:sha256:0858bff3f7b93b65d2aade244e3a4e86f22fa27dacee59fa432b72b6bbc0340a
home-path:sha256:e1ac34497a806a241dc9fac7b98ec55d9fb886d45ae2ff1d246b6004b0fd2b36
home-path:sha256:98afd953ef4c2086d56ac73dec47bcc235ca27b5d180571e2c91c269a746eb94
home-path:sha256:e04b766d8f183d42a5ce2ebe286ab3926812ebae9df4b95ed978c6af4c9e9990
home-path:sha256:24109892ec6c6cf156f67ff0216aa1ababcb2cce112851ae86f80fbc3db4b6b4
home-path:sha256:f41052558fc617202e91565d6a05b4f43bcfaae56ef323b8526246c29535e0ed
home-path:sha256:076c13a92736c4e7f1b2966085d40ee8d767194cab4d8de6f7f461dec231104b
home-path:sha256:6c23719315c1f7922fbcce3f6d393ce661784c7519a0bf94ac4f97d4531a4be8
home-path:sha256:b108f235023658f60842a11b1ee39ea8c3408fd54bc42b4edd6f4ea8bea09fa2
home-path:sha256:75da792da69b387d74050134315836ddd9a5428c642135c48511e9408f3f6cfc
home-path:sha256:4707686e3d415000557a0a93dcd1c334313f9f09540f873ca513c48dc7559cf4
home-path:sha256:54a444b49f217dcaa6973924f912e6485c2e5ac2b0dc97ebf293e5d3150cfdc0
home-path:sha256:f74a8fb6a5505b061ff7427b561765ef5f8d7ca3b491def74311e4dea6f20675
home-path:sha256:b65ca0beb99fbac10750675a5b9c3e5498db53e0c5c8e5b6abe555ed31c27b1e
home-path:sha256:b7a2ca1d6b6c5af330a91f80f9246f0ff7f6fa6ee4a0dc99622c71b7c0524c36
home-path:sha256:9401771793ac0f08653877c2f3005c58642cb3aa240994bb99ad35a9722aa102
home-path:sha256:ed8af349126cf6d455006761dcdacdaab275a3ba5519929f3d462aedd3993ed2
home-path:sha256:9ac185953f781f88e8284e05a356faaa77fdb43dd1a00827dbaa237029989d0b
home-path:sha256:a70aa9d1c299717fd4f745d704db2f8c748175f91252acd1e77bc361a453f940
home-path:sha256:0389f00ad9d7682914c7b44dfdbfd5ca5236c9f2583491e331ef3dc2381c466d
home-path:sha256:b5478d4239eb392fcd8499ba3ba0a696df31a799758d00beb58df8429f22efcd
home-path:sha256:f32fae36981224cde521810b016a2c35788783e00747d7ac36b9619cd3a7eefb
home-path:sha256:0a70ac74174180c9caa7fc52838120c1dc2eff03231ecf687a50c231a1e08708
home-path:sha256:b08169484d4b5e32a5326aeb58b4b8fe166e6538fce21b37c18fe4c7f8567ef2
home-path:sha256:f978b9cb5846457c996a7c9f5a0e3aa83064075f258a28bbada719e954878e05
home-path:sha256:dfc454324218121b20f1fa2d38514d93683ae9147f42f177a06a204b31ae4b30
home-path:sha256:6ea15be9f76bc2e1599f8d035f1df63404fc6fc1a648e4293a484fa059cbef82
home-path:sha256:f7b46afa72f299970fe15f1b3bdef89ee250cd80d92b2c7ed9ace04ce641613b
home-path:sha256:3bf95dc5c32bec92c487470d81a73b369020a90830c1328c5a9e7b0fa4d03165
home-path:sha256:4c0ac1fa9243c20c7cf623378598c026fd9ed09e6e6c88bed81c977d2ba33a9c
home-path:sha256:0875dc2e1717e27116f4c43130705e609f638b002083736a557f81a95a2b1a8b
home-path:sha256:089434898f2063baed67b0fd05a0aa9ec5962690abf77812e61504bd3806fd27
home-path:sha256:08d453d06b35b292e73f62cf2afafdc7042441b7ff1f250ced1569232c5946c0
home-path:sha256:15e0280f798a3a5fc85119338d201f10a3fd9066ee1f4ed6ecca06c689710eb5
home-path:sha256:bb3cc2ae4b89fc315d3d1ed7d6146f0cb06708e53ccc8ab83422516d7a538ea2
home-path:sha256:a2d57c22b3f5a0b5dcf632dc8a7e8201c387c0de967ed0b252b7e1a7e2621a45
home-path:sha256:9c12e2dbcea7ee7be1874dfdee4e36122a067bb9bf59d680d93aedfa893f6b34
home-path:sha256:1efd9920f452425cbb86d92df705192d47db628476e62ec66fc6133ed74fa5ee
home-path:sha256:030373244ad51b8b3734b4bae48ab09e847e8a52203f8d3088cd7c5f25848f78
home-path:sha256:1873288b0b79fa5dc8c347243c316c6fb19a75ce004d0f189ea6d56b50e17c2c
home-path:sha256:593b4b47b09e779575fc13e97ef46ff06841684c1b7af31f4369c90f94cab9d5
home-path:sha256:bc64a047dc02f745556187ade07f9fa08e15e672cf7870a0f0a50e4aa2786d64
home-path:sha256:e603c6006bff75f64e57e6d74635cd75717ebf7618d8e0e7c9a5877281d2a3a8
home-path:sha256:0d6288bdba68cca0230a0cd79bfc0acdfb6d972310d77d28b5d3e227d92a3f25
home-path:sha256:6b47191e275004b021dc97584e8139b4e43dc63f6ae3d7fec2320f94bffd9f7b
home-path:sha256:26615e8f655abcc5678fb1df701293a769105fe345445cda1debeb6493490bcf
home-path:sha256:dfa404715a99a19ae94a61f3eed442bde433c618e6a33df66ed144250c507d3c
home-path:sha256:7ad286208cf1a7e97fd4bd6fc932f4812e2193b34034dd8d267122ba1bf7b8e8
home-path:sha256:193e18147e4b12cacadeedf9c74f1de287e46dfc48fd8487b52664e98f969e5a
home-path:sha256:808567ba0ab295fdb61c6d29fc5a9f6e74da544aba0515ba39a7352549ece810
home-path:sha256:b25428472f0978b6f58e8a9bde70366e853d1335c1c3aa28b335edd15d4e64f7
home-path:sha256:28144c30e2f2b7d9303cc46b77fad3e56c773e295adf9936605f3783d4fa14b2
home-path:sha256:e897bb887859dc4fc849b83333a80a3e1f7d7271090d9829698c608c9e3fecf2
home-path:sha256:7ec8e22f9f39ab05cc0d0096e5d8c3b983151f82ae612a36f433df30be34b060
home-path:sha256:8bcf8364da4cca0d49ba8904615fa47248da089031808fe609cb810af1d82b89
home-path:sha256:f54a8375088c322773288a8b0440ecef6c91dbd49c8e8375a72b54a567325115
home-path:sha256:862cf4529ead9bf726c4ee5e3adb48c95f8c9ccc1a67d3a3d7ff7edfd2a4b416
home-path:sha256:3598b6c8f626303f07e1228d2a7bd7f293c018367297dfac8e00f948512d2387
home-path:sha256:c43071dde56f483d7cc5e30c553af2f884c53571141643d12dc4f446afdd0a01
home-path:sha256:c0f5d5a3fc3c60b440e2d1c57c5b526ae463f2fe78430469ad97b701e318912e
home-path:sha256:4a0fc0e6198bc68d53a6b1ea5dca640085615a21625e121d22cd209b481a8312
home-path:sha256:7b2a5be0972186c6faf2685117768892144c7db242a8d6d0076a85f06c1eec2c
home-path:sha256:07721bdc660bd726cac390a5437abb7b43a1ef21ff24e761421fa6beb713e194
home-path:sha256:17b9844002cfdc9411747ac759de14adffb6616f56a002f4f476c6172bf548d4
home-path:sha256:bdebe278f5c3bc41ae573309be6818b98ceacdf5dd3db342644b7a875f1288df
home-path:sha256:be25e76de88c06b000a36a6ba91363212b41360d718abfaf8132c9713eb4b6ad
home-path:sha256:6406d5ae6e0c4e54656ac033d3b866e293d58421fc98e4dad77f15e2bf1a96aa
home-path:sha256:d441b68c1c1e510d56f27506f757de5058b3b6df75be55e2756d904302cae89a
home-path:sha256:3c77e0bf42ef5d0598a16f5af94e4e98a2e28feadbff26f3e0f5cfb5e75a45a3
home-path:sha256:fa36349f56c083ac705fe0f7f8f047dfea1e4f70a4091fdefaec5cd1fed7cfbc
home-path:sha256:116011030c1b8ccab314889e02f15577834235877868e059b3cff0e40e018a19
home-path:sha256:7b78ece2bac9f6e88c25544fe030a79654af3bee95bff7f929bf2b0240fa52cc
home-path:sha256:bfdd5547669792f4dbb784dcf39dbe8ab74349147191fabf1fcdfe526e4b5288
home-path:sha256:f57248fa87c4b7264ed5a7f028fd2ab1439436d4e2ff5ffbe197cba664c7d4cc
home-path:sha256:954cdc214ba32ee1b8fc9b5bc10068136ed5a31c6f7ac69ab6977fdd4358c219
home-path:sha256:7999dd83986e396f0b9062bd0be2265dab1950f660ba6da3bec2b8b6b7a60fa6
home-path:sha256:1d7d30f9d7d54ac7703bc5504da2ac64f4c2f9972c7908077a7cbc2c155c0c55
home-path:sha256:7b80a9dc970c7a3a017402033af7d30b9a1cb634490fbdf817ad34c2dbcdb006
home-path:sha256:05597961bcb4845b05daadf73392762aeb7d5a085e8cb8d81a6a775939a7900b
home-path:sha256:3e2c67241baca1c2f63559d4dbe6075e4d8d20f16703a292fde8208c706b1e29
home-path:sha256:f5a154eb87269967549b3f07fd8613089fb2fc06d9507b8d2de9a7db62146180
home-path:sha256:1def8ca3c8db3e75f77a1001236445625b96431043a3a04c01fde820aba49ef0
home-path:sha256:249e43bfc504d002af525d90bc4c22428f6544e0497fd63382fca76709f98a9d
home-path:sha256:798fcec431cb73dfe1c2b29963a0a28db2c4f1b68e6886b61d9a88e21b9fa373
home-path:sha256:0272cab090ec6603ae56b79095e6da9af9f429e1d7dcf7eecd8474734c313862
home-path:sha256:e2020f783e40c37791ebf04d1eced68dec8e540fa3271311fd7e14c092bad1b7
home-path:sha256:6a3146bd81e50742c3b19baa8c24cb41f8a0490b721120470c88bcd69946cb40
home-path:sha256:48f4883878a79a53cfc7356cee17b23a4a3991dc8ed98b514534a79db3ca8f70
home-path:sha256:11d1bd0e9face7ad8ecaca4998900301c9e0d3e4347ecdef79fbf4a515d81957
home-path:sha256:de139a7a341aa46a950ea7ddd5b080483ef099fcc612065b0c714392061bfa5d
home-path:sha256:854d00b9ac44cd923c36b3532bef8526fa8bacd1afb37ee3645ae2437d96dc5c
home-path:sha256:63eca27d06cbfdbe2794426b0251d9b3539e262009ba9eb0b1adb4689b43ebed
home-path:sha256:6cdaee790845920b4d34a477615d76d1a9ef4317ddf4bb790fb0f4ba83917f1c
home-path:sha256:d1236b3b49d1cc57f7773f24641589516a601f392f7a01ad8e943ce77562e94a
home-path:sha256:94c139d4e51b8948891d90a5efd83219cee83a91a4b4c58f522d8b6d860410c0
home-path:sha256:aa69474a7c5cbec48a6700052d57e5ec904701d088ea6368dad6e371043bf170
home-path:sha256:57c5355ac7c3e9b40f7a7995ced6874cf0a03322536fd647c2144bc4ccd30865
home-path:sha256:e6bd0635fba414bbde16a6102db7fc87f838b417de4fff1df43f6c8aed533db3
home-path:sha256:870434f37b1831a8afccccea9bbfa93f5e4155a03b493bac3dedb8c63822068d
home-path:sha256:0369514dbcd821cf01a389b4b3f3a97a7fa0555ca3d39343d790984f7b3a06e4
home-path:sha256:a9f060f0c2b25d4a9973eb65282b297190df4da334795327039228a8b3643fb2
home-path:sha256:eca88468c458e3cc4b2eb948b3f980c7616c1dd5e9b0c2a7483d17bf549ec0be
home-path:sha256:056f1be657ce21000f50c89463edd26e1a95e163ae8c79959d7cb6b863fea7f6
home-path:sha256:8b214dd3482e41a7e05823a8ba95fc2e67f51078d40a36926a2baabbf3b74a26
home-path:sha256:5e68c6efe59429b09565a2f181330cb94ae520f5744db3326d05124f2d9fe680
home-path:sha256:45c83e37b05c0b6fa92d2e0f97b323ce86aad9afdab909494f0f73a4d30657c0
home-path:sha256:e5a779ecfb5f68a854b2fc88d43747314fa18b302f94a62e3c57941a62fa1d11
home-path:sha256:bd4bbe30419673d9d661b13ea17da55b1b08f9712272dfb9c0aeec6dd810f974
home-path:sha256:966ea537ffd2f8b7178ded542871ab95a5b830d0b580b58c2491696359ed43f9
home-path:sha256:60e599380dd0a7eb8acfa528ccdd17557db31a305674ec53c5c1ebd05aa6135d
home-path:sha256:faa169a07eba2fbc6ce3f5c27dba6ad91af680cf48dec511a22748f61ac38091
home-path:sha256:6820360d048b36dc3b73df1ba6c901ad504e559312ef36f1960229cb3fbce4e8
home-path:sha256:81785f35636810afa28b02d3e054a0239afc4be27318e6de677c26c09dbaddc5
home-path:sha256:957d41ba4f4c1417e7f4a81ef50166737bff183c8b5a760da8c0b3341d887784
home-path:sha256:70838fac9e27cfa4e85745ab74f6c300babfd4d6a806744341354c35753ff027
home-path:sha256:0302c76396f68d7a85c34ad7132d16a0ab465b82ea7be7375b3d6cb6097f15b1
home-path:sha256:e26d496aaf5dc4b1bd97a3649be46442598dd514b126391ae927f9f9cd48e941
home-path:sha256:0ca52536faca0f30b6e140e17f9f81ca3fb0fc239cf0a5022cfa29b430485ade
home-path:sha256:1b874570f540cd3d1205608f1d6ca528427937811203ae5507f46a3f1ffd4131
home-path:sha256:9b8a571e44f695f80d283dd351c46044e7e08bf9e88b4f09cc011c82cf36b6a8
home-path:sha256:9c0a0fcb033e2f69287dd6ce15d4b761cca116ee4db7f5e079fa07d0ea9609a3
home-path:sha256:e86c1db292eda814b1b929702cb178ca39b39f2999726365175d4d023da998d6
home-path:sha256:3c85f59f03754e5d263ebfdaae6ad3ef5fd0eca46ff9992d23d94c25bc542c16
home-path:sha256:f1cd52cf4d55ccd05bc228680911c51bf45624f7e6583592025a347b7a2c387f
home-path:sha256:19e6ace9a3be7108aa52615abbd37e070d8e1d3af6db6bb1e9ed0ef5778f7c2d
home-path:sha256:081ea9bbcc4dfc600159e212cf39f7c69b2057f8021c2d4a1ab45f1911b86fc9
home-path:sha256:afdf8e6e069dd46eac24a761bd095351198e9f5ddbe0d0b8c6dfdb86750f54dc
home-path:sha256:27da452b27060befd21484f007ecf2a2038d4a7552918c125aee291cf95daa9d
home-path:sha256:ae48a37f2e30b6d28d9a29c3f1dad567b9164fc7ccba83c353ae493358924ece
home-path:sha256:45f87dc2ec8069a6f87fe09ffc848a60f9ab99db8927c65a0e46d85cef0cd450
home-path:sha256:20096974024fa0275b6fa12848215d5992f5370e192f9922bad031ab52c57a1b
home-path:sha256:f8763d238fa99447e2efa7caec343d2e426606b02ec28a76547cb12233953ce5
home-path:sha256:552f9128955a7fdc78a1584515f20ced3e9ecf67b5b68ba4f824b61621853b62
home-path:sha256:be20fad1e4e2b97adacc91e405b9c5d227160514b24148a2c2d5a161836ebe08
home-path:sha256:a5224edae3223f63f44c2aabba54fe796c13e1eaef919c70d8f84942482c61a8
home-path:sha256:968a8e91a86141a0cc2472e06f739bb46398c643777aa5ea19f1f860c87ff4b0
home-path:sha256:3a7c2b8becb10154f3d957374a63bc061e4b9ca2d46e2cb4e3f1e34d9587aa51
home-path:sha256:39d14b7391a2ed7b3510a0c5b76f49363d38ae23364f7695bb2f628545475697
home-path:sha256:e6f826f7eb4c61c832f7fefb107467abe75682123e9532bb774c12af4c42998a
home-path:sha256:f45a6a020f1a4fafb921dc22cae144046448d3691b4e63846fabbd47171668a7
home-path:sha256:a4f15b35dc596776591609eebd285d95b36a89838c52dd3ca1af307b3ad086c8
home-path:sha256:9a34377f3ed87016e206d7717c12d102f99bec0ee2c91db5aa240c23e67f5fc8
home-path:sha256:7df63b8dd3de5dc8d8122a45a468e39a85478a0f10e9e351c3327b81ce95a786
home-path:sha256:cdf0ea7470ee928acfb09b15f96a93ce28404d7c76e3f4668ec4b12e5d9047e5
home-path:sha256:1cf0431d7c126e47d61217c32c3c383b3b875370a563109f6a060850f4112433
home-path:sha256:f89a4e05957bef8ba3136d9cd85e751c18a3201617a6c980974d129db28e7043
home-path:sha256:fd38499ab21f9a7094d5ea85c9e1451ac07fc9b0afa0d0f1b54512c749e42026
home-path:sha256:5055063fabe6ad21aa33913b5df4af28f96c51a1c2080374b821d95dec68f27a
home-path:sha256:85b82130f8fcb2d42b50307ff85efd4b37a5aaf2f4f76bc1db0487b100c9a48c
home-path:sha256:c7930d805eaba16e96f6ec64c7ba55a3df13b0d478d1b0015813782581686c52
home-path:sha256:6f74a37d9b66fe64efa91ef6425417cd8a35e81b390368fe9ea5eae90729d71d
home-path:sha256:b740028c1aeba45f289364d5188872f9d567a3947b9ef44cd83ddabde848b196
home-path:sha256:33242c233bd9f9621179d683f987e26933393c8f1395f0be4b14cbe5f3f0eaf6
home-path:sha256:b87f4b50b4407ca28406bc63e91faa161d36d4ea47c99f4954a92fe5d4d73e4d
home-path:sha256:58fbc5c1a3337ad09cc643f7ba8066329ec8d4959b73b04214524f98a81c4180
home-path:sha256:5bb8cbedef20931920d3f28c4aab4d6fd43247629ff989c2429d7b3510f6a67a
home-path:sha256:d47c6ec158b03a8eff630e2900101fef983466000a54b55d8a6484f6bfbd6228
home-path:sha256:05a0c2647466ac93252364d093ccdb156af0b052954cad1d7d685e41cd75450c
home-path:sha256:1d685e6452f84bd13b1b6015c9bf4d7da9c7ab49c5682a0a0683409a5fc768f7
home-path:sha256:161a2bc297cfffb0c3b2fdd2ef01c7b29f3b1a2a1200f9950be020c4d1d74303
home-path:sha256:03a885c0344e3590b34995bfb21f06068a7feffc94d6ab48ee3f6ed9477fa890
home-path:sha256:fcf442bca5dacb922257794857fa5252a9b07aa48c7688f88d848a4b9d6fb543
home-path:sha256:8df24d14bf115194fc70fbeae3ead07d481895d89dab7a7932e16d999b840d31
home-path:sha256:a854ac6a2082f78e699456d6dbb2f7247fca56212e1337d35510a56e42545bb6
home-path:sha256:6f9cded9793c327333d2f1f6799d7d96209219d127078c643b9b219f79007bfc
home-path:sha256:2ce084525652b7e5e84e806b0d2015437f821f7d5fe06a414ee5f47206a513fe
home-path:sha256:e1500178e1c982cac8f6097c8f03d64ade52f376504027ffac7df12d9f540922
home-path:sha256:f9e8eafbe799ab2ac9f8138fbf5e24a1445ac74a83586460bae12755c706acc2
home-path:sha256:5c251ae8fd954994ece6668982823d9b343f93e269111a3916490a8139639374
home-path:sha256:1c3749cf859d90493358092a7b63682b8931bf78562f9988be7f610cafd62aa0
home-path:sha256:859ca93e02b9d0686a380fa7edde1ae1e6215d30f40e69021c0b75808bf0c385
home-path:sha256:9871ae8a0521a5d9030c1f8e1e1d317767a1d40dfe5693c1084fbcfa946aff3e
home-path:sha256:c7544fc9acaabde09882e57c65bffeaed14366afb4fb755d02a70078aaef293e
home-path:sha256:0c66e84333d2c6cccdccae847a63728b4911e26f80e62bc221e02f9ef4bcc7d1
home-path:sha256:ccae912bf2d917f755109c00328ba41b0827ff7861da5ddc59b5e3723a3d47fb
home-path:sha256:1d4b221f9f8f07df58e34bfdf49b0f85e87935d1e84613aa6677ce5a4d2e8aa1
home-path:sha256:59d427fe4082f3e97be84adfb40287b8f89cb143a2b00b67074cb94dae541a30
home-path:sha256:d62cf6bedc3417533276e6ede709a3a857edfc45f44284ebc4f3a439d074face
home-path:sha256:488c7d1b45af72d11f35fcab7c285d65410a8bbd5f181e30e87db5be742f8a9b
home-path:sha256:5b49a4d0225368c1c1f71356a95617bf9589c1582e34929670881177c3611125
home-path:sha256:d4d787b303d377d5b82619462e8b0c5a4101ac937ce3336ff421ca6252b23449
home-path:sha256:c6e50a6c582663554327c44390cc3fc74e1f56f3113821e0a774deed836850d1
home-path:sha256:36beecc39318b314fa47d39ddc1cbbf7ad70fc496614df11d1b754cf491a1938
home-path:sha256:eaab89469ec41fddbe38cf8898fdab9ac783f0b47c51f6248369bb1670624ca6
home-path:sha256:65fbd18772881c2046dece8274e99c6a4fa89108a366b97b6d86ab79fad1907d
home-path:sha256:9d9f8517aba96d940a503488e8ed8711c3dbc82f953fa1231ccae967245ae45e
home-path:sha256:d929a1f40aa6ed3bc1fb4404d27e8bcae6130d143caa7ec9fc4c2e29f03356e6
home-path:sha256:762bcd4b062cc5daa184a5f15b5b56bb6980a874c214caae3e486641eac6d1bf
home-path:sha256:877348ac9f38c2faf26dd549359053d60b686edb094d2b9b8db6ae7d0c149e41
home-path:sha256:565136c1576711272bbd8d81ffc81b71d0a0f0129a4775c8aab70f42d93d61db
home-path:sha256:5008fed03b0decff02a2bf9c142301e222013f18eedc8b8938e6334fe0bcccd3
home-path:sha256:8e4d5987148cb9b895ece9a21d82409ff693e02a561816e3d6cf3cf5241ab4b2
home-path:sha256:e95eeb35aa0ec85decab316deceae690f9d6ede76d407fafb1a6891e50918ed4
home-path:sha256:490eacfb49cfadf7073e7f52543f4351d6bed4ff667eb9cef68f974437d9b819
home-path:sha256:fcd5a46357b395695f31c5d7c4abe6c9dc75cd0a3750e6f708aa6ab9b1cb21cb
home-path:sha256:e34ce24d7e4332855eb12990d80c4f1938c8afc1eef917d70cbb7034e9b9959a
home-path:sha256:0657bd146601eb20d13c14f437e79adf7f6a417b73ff6f66451a79718e4bc2bd
home-path:sha256:1e643d0c91e166f2f4d38dff23992cb743b02c2ad00f63e643908618a76c597c
home-path:sha256:12bdc9ab414d82da14a3be7d8ed10c5320349b015b677a8b8f2b0971e4ab5dbb
home-path:sha256:b74c2e0d4690a7d7ad3fb22d55d29ebb7362531a20031e3d0618f37657cba4e6
home-path:sha256:f084fc6254c6d9f84d882461f3239142e279f3f7b0e3db179a23509aae01d7cd
home-path:sha256:d6aab6cdfd76b0c6ec8b246586a3c1021fe9a6e07ae44817a3327f4c74c2bb2e
home-path:sha256:cd653b28fae315a8625123ecabf4f4b2005a9ac5310ed5da7807c9e8c9a7091f
home-path:sha256:aa0f6152513e0e1bef84615567b79baa3917e58d03ef39f80f79d3c112b656c3
home-path:sha256:7e099875262c03600a3c0e2d1b4f3afa1611fbbd62fd402a32aee7a9c3b2047a
home-path:sha256:f54ebd8574768b54228359f86b68626d4043bebd05ec98b885fbb0156ca44afb
home-path:sha256:48cb31e15279e79430d037b61a3cb6bd7f1b274ef6f1e286089a00557551005f
home-path:sha256:f8ae4b51c6310346ccccb83de6b004495e29a3e4ae6319c99a273778b7fda2d6
home-path:sha256:ffca8d422b16c74eb11b6c8c7423e5b9359ef632c971a557166b0affe27a274d
home-path:sha256:fb0be319e38cc5bdd7536f0c287a34cf56bb6272211190eeb20705ea6767bb2d
home-path:sha256:79e10491d6ae175368df6ce771f21b5697b5758342e6cbd44a9cd1f8d9c10c66
home-path:sha256:3ab191bd8c4c775977c219505c9f63f948d56e31ac0e3fd885d141027e388f2f
home-path:sha256:2fe9fb22d23b6e70607bbe1c6365dd5eea2b37d5338a4e39bc3f257806a0fe02
home-path:sha256:6a9663b43e9907205de9ae23bb030e5972d4af12ccd009cee19a517394eb4247
home-path:sha256:681835ea66231193f246ef951fe8682dd5c57938fa521f72fe2bcfdbbfb1363a
home-path:sha256:2ad004165b42f3357e0da7803888f0f02b2aaa7fafb09e1c38f04288179bbea1
home-path:sha256:d362a01f863dc58ac31b0b3f46952961322156f40790430f940fb495a284b83e
home-path:sha256:216375ee2e6c3d0eb60e271c261b909ea5513476f47c798c51b98f689033e78f
home-path:sha256:56c982a02ad4d2cc30113a28ba0f574dc5a6f3b7170b94186eaf598b0ade9a77
home-path:sha256:17008b85bea2ea4dfdd9b170d1b5dcaffce663db46615779d662b8f7f55f21d1
home-path:sha256:20736554cc4b33badfad7c66634200755336bf0c2b18d4d3894a9173234a5952
home-path:sha256:53943fb7b01d467e0b54393690e4e13ffb3a2163e8ff1e16303cdaf950ca5c7f
home-path:sha256:af2d2ce375362b9c466b3c7e647bd11b5cbf2b862839d2d78d9433a7b034ae9b
home-path:sha256:d65b6ec4b1ad1d83a3b2560d26fd4fc91485e8f2b38c6ec4bc2e32b6cb62a84f
home-path:sha256:789dbb3288d432f25dd1dbd85a2919695c9d9cb2efb2f3c6d08f5ecb573c854f
home-path:sha256:439acbcb3ca197ece2145e6009fbb8cfde76d36031258974597ce54bf246a3e6
home-path:sha256:2cc949a7d43447ecea6464da927900ca322071695e79cc656a4a43cf30dd30ce
home-path:sha256:4dae7cea98111cfb86967d140b0a6f1d757789f30545ee35850c71278bcdda3a
home-path:sha256:1df4fc1f735580abf0352e7caf2619d75bbbeaa0c09f9ff72542d394e0a0c701
home-path:sha256:b12a417318471b7b9d8e9fdbef286c61dff13d75a1751d6323244627c07439ae
home-path:sha256:f2353d103a35f43e40886c41ca46546e8d165c53cc86c79064b86e9b171e960e
home-path:sha256:656041f2d743d47e92d0937cc873e8f16f5fa5078b809866774aa2dd5fc48fd1
home-path:sha256:48457423d12deb3ddd49c4355197462bacc3604b1609ca006f85c76feba814be
home-path:sha256:5a86579bdcebbf5659563a399d798d9c2925d9c5ebdf0590f1bb440e054eb811
home-path:sha256:f533faa980bab7dccc8a92a0e5946ff6c806d5fe6e6c7f60b1aff5735260f02f
home-path:sha256:9a2f48a1798b87b0d843fb05d8efc19749c1d6e2b4ce2afa9d276fe9dab69a95
home-path:sha256:01aa0cb852e6b375528804147e51fdcf3607e85509b746823cc33b534004d61a
home-path:sha256:74d567f9499eaae3fbf647bce90df765fa53c3af6b8a10d879674d648d47fba0
home-path:sha256:2244f22e8c1d12b32b6bfb2bff043ee12b96b495ca3d9f681fddb68dfcb2130b
home-path:sha256:5ede794fcba2fc97401e034cc87b8f37eb623362c53a6dceced982bee73b6159
home-path:sha256:c9e51e74b5b7b89d6c06b5c1a7e904e3d3d1f538d1292a9cd47a274c195ed844
home-path:sha256:b1e23dc48ba41dfb4ebbecd1f1741c0e2c26b27bbcae5ed955b735534a7efd79
home-path:sha256:655510455edb056aef89085504597f56c909fa6b9b44a251b5e72aac735e6791
home-path:sha256:61d4a80c8b81f271c4b58adde845138076509f58117af4728bf491ed275a12b1
home-path:sha256:cf795f1214ac002cab12df4b15ad56731cfe65dda5f2dac081e7a2a3d4278c8b
home-path:sha256:27b1ec242f051c469793e7bfdc407731786235afaff5c6790235d85a01a8ffa7
home-path:sha256:fe63edc516b2cd2c49c71543e6d4d9c44332e47abe8d303824527f546862bfee
home-path:sha256:d6a8abd7d5c5044d5fd19ea258a33e7d73e1d5d43296e925c5ce2b115853fba4
home-path:sha256:f88ba0cb535027815c157fd9ad2ca23292e9f3ae13c53554cb0a7d817ca929cb
home-path:sha256:3f2af3c2b65bbcc5ea3afd80bff1d832902b531911b6894b450e6fa28f20064b
home-path:sha256:d933a440511d10d8e1bde7871ad2d06246bbe2a1c79f713b3abd78113b5fc319
home-path:sha256:7323d065ae6d7d76cc218edb831ea05de163c81064278651f9c4cdcfe786c5c3
home-path:sha256:262d1d7426b2e4e787e56ea7ded5f4902153854e6e6dd218b37a4917fb9d533e
home-path:sha256:c73b24f10a163c9c2986cb832357276072043e101465583285c14dd00091004d
home-path:sha256:9b96086c3166cb26b9ccd96e5c1f2113cb00b85c45c636bf4a7d99c56e6ed45b
home-path:sha256:85d6fbf9293309e084b3f2f78ca4fd780f5433719dcd2c1dce22aabb7d16ac4c
home-path:sha256:b863fcb52b8aab3fb23c3f5e6e2594606620be4152e2f7248af7fd622068bd61
home-path:sha256:1a2fad7c30dc03d201c574498ca36a8bb97adce28cca35c097d9a9380354ea52
home-path:sha256:016e70405baa0f5fac4fd14c4ceda200d041032c8f0f831694723d02dfcf0231
home-path:sha256:5b877a8f475534504a078fc5fa8cc4239fafc00fda78a32d1de428ac2586b280
home-path:sha256:e5b36bff0df3c6eb0d6e9a31caa45e4f641fe8dac6697076a7a8ce71a60fbeb3
home-path:sha256:367cc64041d7ff3b0074c0f3e561ce6ca7f0ff2e00eb489dd8eb835293d516c0
home-path:sha256:e7d07aebe5e4aa9260d27af03fb741af728b7c21242a650a6e3b2ae2418812a4
home-path:sha256:9c9e94fb79b812e172220e56df058a1b570e56ae56a9dc9201db424e51bd1118
home-path:sha256:8dd44572294f5170984c8732eae49a8008615ebcbfa3474100aed3440033ae7f
home-path:sha256:08147cfd9a5d7fe66c5d87e780c3b741d3f89bd4bcda9ffcc9fcef57e99bb463
home-path:sha256:fea525b63d0948603e78fc6ab53636aab8999f562195148bc940d3c26e3c29a1
home-path:sha256:c347b63cc61f884429f61837c37ffbdb059b0acfccad1371c4bb6b441f29b168
home-path:sha256:2c580ca21bc5389ba421a52c23ba3e0e01024211a62e0205abc8e1c42ab147d1
home-path:sha256:c48687d76e570fa71ad343eadd8c2586b7275ef03e49b7a01e072878bf609407
home-path:sha256:1611a50a3a647de1d33dc4a2b4a05cd9abc1e9359c6d6e58eaa3dc84067b9991
home-path:sha256:5519573e570ea83092785405ae83db4156f22fba0cd1c29d337a874774d2cb73
home-path:sha256:df9901abacab89509180c96897edd9756bad015b3f5083f08b1d12153290e91a
home-path:sha256:6293375efb7e6b0c47286fc37a660717f569f148bb66540580d9e10350b343a1
home-path:sha256:6d2813bcbb0e37d6680d98287f12557d530f256098d6bfa6cf05e9ed46867064
home-path:sha256:003ae3ec59ffc5f4f2dad58b9a5275ca2070b461d1217f139a65f9acb670a731
home-path:sha256:ba7500451a46b69a1989c8a22bfc35aa49716dd9c82c145cbcda9dcf8b97c873
home-path:sha256:ae3ac37d09592760eab8cca0b95111b6b08a979f5e021835cd60e6d98004cd57
home-path:sha256:9141c7ab6bd541ab8b954df87e10cd3d844b551e78a056f6c2856db7ffee66e5
home-path:sha256:73f3fc0003b6bd4a138b0644f7ba1fb114a668d4153eed81b3887ce65024810c
home-path:sha256:adc16be3bb07d4a51a2aab4bad4182d82f87934d7fa61e3f459927e459111b41
home-path:sha256:d16a096780ff84a816006f9e98fc6d56ae176a756d286155e335774ee1216475
home-path:sha256:c296a5dd00f154c1594e56d3c692b663a4686f78838a757adb49c3ff26cf4d30
home-path:sha256:65486691dabae564379e6e78d61678e0a6a8238e4985a8799222e6ca776838f3
home-path:sha256:5a57ffd1a300d8cd636000f9b677ba29473237e5f4180d1b1a43a1aa44ab37ee
home-path:sha256:4b693ae26868aa09c955fee75dd9c68d62c8b182f2d6e2fdfd4133277a2fecfc
home-path:sha256:2183ec7aa93b62a2743daa6887f1b0ec03e650e7cfe0674fcad5e583ca092994
home-path:sha256:495687777f2f60a8062e26bcc50945bada34bf6b8dbf5c68c2f3011f0b5f1846
home-path:sha256:fab34253acf0f9f4197fa8a014dffffc8d9c63a46f9cc9dabf164b1e4caa1a07
home-path:sha256:056fdb0ddd743f2a0e47156c5a9fe2dd947f1b147f88aad20d6850a386cae4a2
home-path:sha256:58ee219ca01da1e56c4ef2a767891ca828b528bb8f2a247c82bf293638b26d08
home-path:sha256:5b896619151c66fe92b2bfdce69465e10995f90ae9ef5d64b274761fe0d6fd22
home-path:sha256:3264364996fa8412f1f79298a0fd6a4183cd8acae5b4780afb65c120ab3ef94c
home-path:sha256:3d945146abf51812a7ee03d3eac965a5f3999a9ef607d696d7bfe5b0504611e4
home-path:sha256:6e32cec9a10d8b5daab14f3fc386d7d2b088a579fbb9c100dd98f71aba2c7280
home-path:sha256:17afd7ba875fa6b3332a404c617083888b7799f339e186d30240bf7710a10136
home-path:sha256:f691e5c00c060941601112646174b93d179fc8df1e7f539dac740094bbef750b
home-path:sha256:491e295931b0ce6dafbece72aca42743f202fd71dd9b473a4f7df4783a451997
home-path:sha256:15ed6b41d75f3dbfc13168f3001ceb95876f9a7c4d57f23b40aba0da86ffc596
home-path:sha256:54efd2b164958d1686c3da2be1bd8f6d404c4a0a34be6e4ed678e6bde5edad83
home-path:sha256:630625091044668afbbce27c2c24451bd53d21c0f92c896f36e23502d4a6d5f2
home-path:sha256:091d4e8ffe2400b46f6b9654ab08eb58069fdf7857bbf6f8d9347ad8eb7e4576
home-path:sha256:1897eccdd6f222b1bf12add9d812c89e65bb5f79280dfaac55d849a50934d441
home-path:sha256:02e6644f3b133db2ab5fc39ca3e9d608769b9cc228fafb965bcb18569197c53a
home-path:sha256:6ea13f7b6ce881085e99ec38565a682009b954478f30ba6629e1053132867500
home-path:sha256:ed58048d1834487aba023e710d30bc34435d2128fdbd3f9c10f98c57d3fe9309
home-path:sha256:69cafc84096f773acf8ee561bc6144735193fbb8fc6942032d8fcdb7625f181a
home-path:sha256:43f05c1568fdb40c92e67538ef75e6722ff5903e305b7fb15e0fe2852b470804
home-path:sha256:58cefcd50d41ee5f1081a00e77bac906b4897ab0688d836d2ab19d365ff29284
home-path:sha256:a893a364016acfb45215a3431457522a4bcbd09b4b646aacac55e68400cc0876
home-path:sha256:569f0a600da742b5d33875bc5dc3f70f6b6ec24603b21201bc70c0f67653d887
home-path:sha256:985a1b1764d5e68b24fa13a8315ba97aaf6ce9e2d6372b26f31243a0a173da7a
home-path:sha256:3513ca2c073d9d59b3302f39271e0fcc7b4a81dd7b8066db332c9914218e682a
home-path:sha256:8dcdfd7a7e2798bb4d12493bb6c54934aefda0cdc153cd8f8b42c8b95c42105f
home-path:sha256:5772acd68a0668fe85a9a8cb94d52edab71877db8e9a2d6b57795ee51248db4a
home-path:sha256:57d3003f51389517c178de2d29c31f935c1859b7f96a21edf5f93d54cf31e8c1
home-path:sha256:e5d8fd87f28d27f8568c1fe8cc23c44cfe66d6aa8a982ba3f1f8782f8b182871
home-path:sha256:bd310eb217e4db179c91f3161ed0ec07ab6502eb9fa5fd7553cdf6fbf7d3356a
home-path:sha256:dda8b46e8e3ad950eae63b5a1e06f1730b2cd3f57ac344a644e3754d14ba1453
home-path:sha256:6cc544dc4ba458b2d37c60cb30f7d8596cd7d2d2323374a75790a7b2fa5fd642
home-path:sha256:6f87106f95bdbb757d7627fa3d5d75e4726b15628bca0f1d2ff66e10996532c0
home-path:sha256:53ed653db212049280239e57b785411435f029754fea6702ed56d4b4e4d5561c
home-path:sha256:059e299f1da4be46826f3de8f4e1d6e757e8278f97be96c15b2badc6dd1dd6a3
home-path:sha256:cd9f590a0e99550e09eafac4c2adedaaa383f861b17cdd0340950d0d7e14827b
home-path:sha256:d5a8412b0df8b16621451dfd9a0a5bbca8c445fedb1e07e861be3a030e8826ee
home-path:sha256:eaf103c3037c4938a94922e5692a7db5b9c336525784b794cd066ecc431546f6
home-path:sha256:7d18267bb335664fe2d9b83b9fe679808cf1ccab3a703c770e368d2ce01120bd
home-path:sha256:5e815ca4ddb23cbf3b67d9418c1af0aa16d78f886f9d4fdd8ac0e553460810f6
home-path:sha256:e7204b42854dc0dc5f584257d9fb0362c26dab350923919a1cfb49ab46ed5388
home-path:sha256:81339451c60daf43fe2c9297dbac1920b940b3572a975db4a66a55d3f3c8a926
home-path:sha256:2c8d10d1a277256981a77bbaa87323a8d7f83ca7e7b77925de92b13e77521bde
home-path:sha256:4aa3c58f1df2d7eca80ee575dd032925a9e295d316de60817a5223c00c11dbf0
home-path:sha256:79a2d18cd11190d7b8ac0c6d4a6b5d02f39b20496e7fb3382339b9ee76505596
home-path:sha256:b2fc91c88342f1d39a24a002e9a83715c34acd73e8ff3455f8809fc326a3b396
home-path:sha256:e42efdff3afae6adcca9301b94ee64ae1df6a2c44c4760b18336ee9a499da880
home-path:sha256:400ff2a8572e65fd385c34ce13fa9d7604e9bce1785d883a6a0588abadbfccb0
home-path:sha256:f5746076afcd5c303a345caeaa92de8655ef1ffac48f4dd53d8a2d89abc74725
home-path:sha256:134e24d95a9ddce0bccc7faf676541260eb30337d29818a140eaaa4a19f3a2ae
home-path:sha256:24c208ecbbc1e562cb0983a8ec11196eccb278d58653c0b7a53b1d1af4515b6f
home-path:sha256:834e6c8a6170e29b2da7e8810b26cdbe13e7d675cb08007a3bbd7e50c4e9f09e
home-path:sha256:4c5ad5e4be5b1a532302e7e839319386b499bdb4e2822f60e1c8fc82a34a9990
home-path:sha256:cb91b61c95ce4ff70eec43d976f7c06f8a6cc0f471543faa2cb6775a5ca394a0
home-path:sha256:3efb45405833455169002613eab11aabe087fbc409a87597e2af48c2a6900da5
home-path:sha256:07176a892648cf6e1a5fc554574739e4adbbfa096eeab00daa9943939577c6a6
home-path:sha256:6f76709afd7e0780305d8319c077757e44addc679b453d8c1ab48d541ffecdba
home-path:sha256:c311f2bef498e6a1f3dcdaf2a0fe3e7a75ff3fa083fca3b19b21c009667dba97
home-path:sha256:519699727fbe12f4ba0eb20cc9b13a20fb3229272a9eb7f3e8e9bc2890e87fb4
home-path:sha256:525d27bde066a17cf0be76e701b3bd0b1cef2d311118fc9c129028f8ba2603b3
home-path:sha256:2bd2b21e5c21a275ac3c32675b2dfd56a2ac3796d3734d3fd3e5b3209c49082b
home-path:sha256:e43df742af69fdd4e524519d540aa151155361153455133163cbfdc26a5784e9
home-path:sha256:6289b215e11c1cbdab9a31eb5a8002ad9ed49055435959dce8a1f7673496e834
home-path:sha256:ea33a3d9454a299f5f878ded411953caf34246111bd39b28029284fdd619dda4
home-path:sha256:11593f194588f32280289b6f35cad8e80647640409186ece08928e91adf48254
home-path:sha256:021428adb97ed5ad59855fe9625ac5e975dcb55f144eaa08f10f166b477a42ca
home-path:sha256:535de7f8bca9550e78f86ed79dbf34a7d6bf2c5ec13a6cc0121bc6cf30eeef36
home-path:sha256:9cf82bab5b459f3d834dbebc86f795ec8c0cfd88bc1899142d663b24d818413c
home-path:sha256:32df613963b33f6e2ec1b4ec7b47a81bcbd87c8961a1909d20f7c7f8e09232c9
home-path:sha256:b4e322a552f750265a4ddcaf8298dbdf4bf61f7bf8c74df4804c7b767bf37af6
home-path:sha256:e47cea24eff12dd1474e82123ea231f3c45ff3bb835c660cd22e157fd42d011c
home-path:sha256:b43d1f48afd16a17b6617fdcd38bc45ce8aa66b8b9a8aeadf6791e2c4e6de175
home-path:sha256:2e73ca1039147298b29763b59d03658b4048ee7b1b85afd5361c4de2882ac80d
home-path:sha256:66a0c6c459c16e34e1570f3830873a7c473716665fbf8d730113ebebdbf96259
home-path:sha256:aa4e4de8d12142e445a4496ca7362876b15a899a8cbca435b37c7e831eb2a4a1
home-path:sha256:97337f995b2f0b68074e1e56d4b6ee38059ab24cbe2f430cb76c66ae7ee67801
home-path:sha256:b66eea7326f42c50aef7813c061e10251359aa4283e8e2aab58a6f2c349c3e70
home-path:sha256:2a7d3f264224469203f2048619a1913c36a675e030458397edeed1d4fa163e6e
home-path:sha256:63cf5dd826f936050f5a0996e46ed7400016734f0db749527ef33f40fb398c98
home-path:sha256:1ffdf38739478e8bb0292ca4fa02545118b980d8f39bf4b4a75f540b432da166
home-path:sha256:0479fdf3069000daa0579a9c9638bb6780f9c84279088993de13111a148fb320
home-path:sha256:055d3a101580733c4eb0c481834c56b52ffc2b007aa7091efc271a422d087d28
home-path:sha256:1905284e8b28f22c045b1e3539cc07a0277f396a2c779b1e21cba4882c1d12f2
home-path:sha256:2f38d73db688eb172e8ce994565c2545dbb0c9121f36ad1134c860d8743412ff
home-path:sha256:2d8f60ef7f2b8c6927e97cebd46fb06a85ae52511189748f2afddf817a91fb2b
home-path:sha256:787d3744f33401bd9e8395170620d747ce5999694d4b9999a16aa8aa54256aa8
home-path:sha256:9d59dec5621d4659f0fe8b7307a077ea8a28917ba24b603bf6de7f6ceb692202
home-path:sha256:d035b38d79c930b0a26f72b3dd524db93af11e1467e055fe2a056486702e7d0b
home-path:sha256:72886315d8f3d7fcb54c628552d0738d1b97eaa6bbf8d25bb0ee9bd371531673
home-path:sha256:0f6fdb0476774d5e41c8812276a07fedf6fb4039327921474aead100931eb390
home-path:sha256:732092f844c9961c9ee26857c85eed9f130d1156912305ec06ff4b5f647a1bf7
home-path:sha256:2b0f9fcc809bb30e3476a6d5f43c11a8e30d91d1cdaadc8a04b33c986538af0e
home-path:sha256:4449879180b9a5b091c408fb41481ff2327bcc1bafb6c27a5ffc9537717890bf
home-path:sha256:09ee1ed28d8d7cd980b3f24d58abbcb21371247443041b95f77619cf8af3a94a
home-path:sha256:e124a4e71f55a8ff5818a780831e0696d4bf039babdb37bbaa8ede1d2c34dd70
home-path:sha256:af67206cf61be72841cf837acccbb2a824a6fce9f86b56ad8b1cbead947f8d2e
home-path:sha256:6a61d26ed6645d07951bdd082423f6bbcabd1236364cd2b3597f079aef387f4f
home-path:sha256:95600d4b7055cd160db262c50fa6f6a4e197f22365f59f94bbedf04fb36b71bd
home-path:sha256:db512ab5e8c7cae1ae6c51e34e65aab748cf761a1b5839b857d51c1836763248
home-path:sha256:ffb2555c08f690d63ce0d87d66f3b851d28c7edd35546301c3abe60566f56118
home-path:sha256:2eadda1901c0f8a6db8f2d121cad8041d627e51e79f7c9927bc4037caa549fc5
home-path:sha256:0895a250092ac57104ef9849e647e7d2002817cde9675e500f5e8a80471a2fdf
home-path:sha256:fa01b8a399d7006eef2ae22ee89f5d5a51c7403dc191b3d16e72ef327f34af3a
home-path:sha256:a59d8beb308e9dd14ee0fe1324f7c8ca5f8ce7a665c9d34434fa4cdcc6dc4db2
home-path:sha256:5ca20217b0f007775c5c9862b4ad714e1a4ff61ce61d7feb0488d2231ca01b39
home-path:sha256:b248d8b7ba6ee63259100dde02e048001b84409b292e3f6251ed9b5692193732
home-path:sha256:8302578866a2eca7c09a8039947a3756db50612b3218b15c3e0ef249477e90d2
home-path:sha256:3668803b4cf57b6e221298af3fc1ba2116d14cd1371ce8e51a5e3f876d31eb4b
home-path:sha256:e28f54b2809cb0d25bf9447e82f6695257f32e98c5c0d31184b09e102dc33ea5
home-path:sha256:c1af15e1a8deab1f82c37162f2d98b36f178a694b7083a8f7c208cdc5d0edc13
home-path:sha256:bbf0f107ab733094551b452e1e1bc4729f465ac32e541ca0a04e230aa3ca96db
home-path:sha256:5aa06d706f60d9ea9edd8ab3b6c627ebd611b53219bc39d23b1404df52cca839
home-path:sha256:cfffbad3c340aa8376893cb953d44526261f0fecf2b9eecc610fd33a3593925a
home-path:sha256:a38ba8df0e7f4b81ab37aacdc70fbf6508297440a7219c2e223d1665dee49951
home-path:sha256:f0597766edf0eddf69709fb580cb9a50dce80d9130f1408e70d039e978be60d8
home-path:sha256:dbdfdf9cf3da87a595f00f38c570fc4c2e8039739053403d30d3f4367ec4c65c
home-path:sha256:3adfbc1b98593aae73ab4eff8615e9ddc814cce97540442be5a83451cbbb359f
home-path:sha256:9657cadb135e7555fb2e6d5b5283aaa9edd7fd87a306ea3c19b270368c713d97
home-path:sha256:70b82b90629911dd8b791391becf1856c6595f662b06192746d3646d5cee1b91
home-path:sha256:ce759fdf821a672f21ac1b00f11e26e5ab62be5a91fe696df8aa69e0b909a7a1
home-path:sha256:684ca16a2880d8e5980e8d8418265045f9c6093e2950d6bdd15617ec405c7fcc
home-path:sha256:0761497cc74006bbc6d205ad8dc540fb02ddc9336fedca4a7f12611a7d74cd23
home-path:sha256:eff617260257c84dc15213c51b351764c9b9fbee514e0774e1e8668b47082eef
home-path:sha256:b2ca2f814c445af453edb8992851382a59ca83080d005dfb05139a3360fdfa5f
home-path:sha256:5738ad48b37ec32664edeaf82eb1cf7978dfcd46fd613f81785fb3a1222c1359
home-path:sha256:c6e79991c7c05f4d64588da27eb55f98072a498ad6c13fe881bf8632d09e791d
home-path:sha256:dfb937703e12f8e2dd825bce462304f7c1f3310736cf5e8e6e9fecbdbde6bd08
home-path:sha256:a393fc09738545eb7682956f39d1189131f77ae864b3a51dc93fe98878b538a4
home-path:sha256:f63cf01aae4af39e96e3bd2be858a76bf3bdcf5ced06cecdb7e28000865e7f59
home-path:sha256:59ab75f6dbb6e9511305a1bfb4292996825393bb08d952746b2bd5de8457fd68
home-path:sha256:522c3a080f63a5c161022a7ddff7f60a475cd85b7de681783152e7137f31d509
home-path:sha256:6def317743b8ddaec0e77514b2c794b9eac0531a2faab734331dea99ce5fea8a
home-path:sha256:e45dfa9d5cb147a5f51879b5ce9d63de5c6821b50d9bc68e80a51b872d6ff999
home-path:sha256:508b2d7882c929fe9ea316347a9b87714527547ef50ad783a83e8a5d5c9bd430
home-path:sha256:c0abe1df55b9424460667cdbab34a3c3a7ec158e8c3df9b8d5d25e357314c8a2
home-path:sha256:4cdccfb15e62a31199e98f9c10baba9f87a792297c34ec73a1f35b7e32e4ef6e
home-path:sha256:829c55fa4274e49e24ff72c6fe7b5684aeae1414056d92152a95179313eef27a
home-path:sha256:552ac3ec482357a4f3fd4de1bf44979f56122345bf9128e7f1a65d72ba300f6f
home-path:sha256:f0b99e2f5cf7e35146c78222f214c6fc83c04f7978c0775e130faa4811226294
home-path:sha256:3eb264046be2c640079c50f8a18ba630d3a95ec9bb961d2c37937db3279f3b16
home-path:sha256:81a9162295ea63a7a853834a851445880e1c27da08b85a8d04822b263150d2e0
home-path:sha256:8610c9f90cf3a4062aa5270e4794f7ffc98b49cbd4390dec2ebe5dd7b1f26f0d
home-path:sha256:24d1cf05108b5e81f539fb55e976e25610e28b43a6bee9c8703be429acac7f5a
home-path:sha256:eb785ee1c4c0feb2c9165ab56daf9683d7bbb52b38ce25c2fcc980b173d7a058
home-path:sha256:1e967fba03f3036448b31727ef6eace43eb9851e4c54cc3b031a3f0622560dd3
home-path:sha256:be7ef25ae49a2c42807b55f62ccb0de33ad60555a88d0c5c91d5be4f1187db21
home-path:sha256:25c814ab49bc206cb3542a332277742be0e403adeb9a63c6281e57306fd8d231
home-path:sha256:3802c5ad049107204c91db337dbb47271fa8b80d2654fbe24170a8f5c4d78866
home-path:sha256:277ad0d09282f3615298b3789dbfb9dbecc1e780803d8aff383040e6df9ff6cc
home-path:sha256:e10791836ba708f46a91b64b1dc989a51a954d0628d1a56b83a9325d7189e499
home-path:sha256:a05b6f9d2de3294777eb8d573fe9a6d0e00c49b350b6a8f6aa54ca9e395e67e8
home-path:sha256:48c701ffc8a601407a63b631df88b4336682b2d13d6f0fd4e20053c98b0d558d
home-path:sha256:66e5d9ed2f6c24a68dc7eac5f5cdb525ee897eddab9b7705c2b476332e3cf6c0
home-path:sha256:9531fca6c71d5bcf10574c9f6a13d8a83c6271c8ecc5e33c7067c5d916f4e132
home-path:sha256:0d599689655326f31dc45635fc10066a61f52f6d52170110b3f29f1982979016
home-path:sha256:2c2ee72e5dfcb653c29c4eb2f5d5008cdd57fcbe47b39a724ce114741564bb12
home-path:sha256:1ec84bbfc463ffaf5084f36e0ed6901145f3b26cf464985e4823a5a8a7c4f37a
home-path:sha256:587aca18c4326ae5eabbe5235fb3d94a4f9147c5cb7fe25f5c21217d13bc1ae9
home-path:sha256:27ab5815f5eb5f9395023f4da5fffdcd466d698e6a0699fdf01fd77b05028206
home-path:sha256:8b119e3deee2c5f1c2ba409b6234f6013fc15bb0b7236cde6eb8801cfa9aed50
home-path:sha256:64535a4624588f8e9fe9f3685ec7c154293d6b45d95bc04923becf1c662fcbd7
home-path:sha256:203d0b021ecba11b0a2d272bfb9430a29909c79d4cb39a8bef14c00872ede382
home-path:sha256:6e01283764033da1c9aaa4702e7b83c9ccf752b2bd15402c56b561910c7603c3
home-path:sha256:ca27a407ba1d1f49cb9279f4551fdfa128f21dfb3185724dad81b68bd32ae117
home-path:sha256:a3f5dda4ae2e7c25f39bfd7d35f4f894327cd0db4313b2a48b59359919b98d10
home-path:sha256:a7384e09b62fc88128a190fce1ed9259ee07ebee6ba6a10a1a1cae51754782d3
home-path:sha256:babb908a0dd389e320413ef16e4d54a1591a1c1129aadc3e1fe9907c68312f00
home-path:sha256:54f28ec7e2cd23a76f58ce2475045d6e77fd53f8df32bb57c26cfcdb61b6c16b
home-path:sha256:42a9f963b5148d8ef3f441c55cde22a00e58ced1e45b0802675e467c997ebf69
home-path:sha256:3292f71f8d4a07f9a60c979e27e724a811146582821d792859855d8a190381e4
home-path:sha256:099151730a6fda22198ad78e591e3e871d9cbfc7ab6b1afd1e624f35e835e7be
home-path:sha256:f1913accf16f73b6c277769f866269cde359e34c5a30928bad71da78a42dae91
home-path:sha256:f0ecd074e74668970701d944219ca302ab057136d67cd2f95c32811796f332f6
home-path:sha256:bea2a5d186f8e3e4b8b9cf65c1df805054f9e007d5f1c4b760b7c42cf4647eba
home-path:sha256:49d648583528285c3c3e6a99935342194062ca4c74f08bdf09c04721303bebcd
home-path:sha256:e8cd127069df3c8802c0f358b1f55d530c871705de1c651e2fbeefbdac717e0b
home-path:sha256:07e90b9985d3908cd3523e6a7e4a1db002806966e8c6fabfcfc6d5159b3e5a73
home-path:sha256:1fd49127e6ccad4c43804b9025fb5a3bd65571fb92df82cca546e23e746547b5
home-path:sha256:7412cc679ce42d700fc77b765e678fbdb6c44333e87a3b6f24ec6c85e46124f4
home-path:sha256:9d65a660b5c46ecbcec6e42994d8b7cecace9c47aa706c0218e620db726fa9cf
home-path:sha256:94e44d295b7b4c1c0bdbab653c5099d66acf83c0dc712ac6d81e989d330bf5e2
home-path:sha256:b127e3c3ef93eab7ceb16222d3e04d5c8415f899d8c2d29c061a57fc5d1f8471
home-path:sha256:94e82384f8f9483d01653fa8ca1004797f5ca72529a244f66f93ad6d5a7a5fcd
home-path:sha256:a12e44e3145488c64634c324a6d123f0e4ddc91955138e10e203fccc47451800
home-path:sha256:73c1399fa54e69d91ef5a6ac3d8c3a8b9602e68de227db58183e28de7ffbf0ab
home-path:sha256:9641235d0d997f2f2e0a34080fb3bcfd5cf443097b63a7ed9f0415023f792ba1
home-path:sha256:52c9a6208b68edc4d30f1c10fcf1d4c3ab448c42b39ba43b2966b0b1547d73ad
home-path:sha256:04ba674f24ff617d9a37058666c2b93380ddfd85c6e5a3a33db22bdb957ddf6d
home-path:sha256:78c8d5c13a0f8cb7753993a856ca9649980b1fb92d2f756d0c688313d3c02f61
home-path:sha256:901c528e4174082032f200b17111f89ef14567e14707869578ee214918b1b09e
home-path:sha256:36b376afb5f6c91cc79b52628aae83dc4fef60b69da9c89f926298188805d0a2
home-path:sha256:4d99b051b06a6c14621eb05831ef857c5f8db989c3068aff5c971c5d52e6b008
home-path:sha256:eca2b8e57a44d0bd3f9e791d05ab09f503caebd06a019d2778bc16e70dd574ff
home-path:sha256:90b63368c82446573a807256c84013096807e98c29c31b62a5684157d8827a1f
home-path:sha256:78c2f35b0418fa02704606db96f0406822034fd8972dacc1de545c9295a93240
home-path:sha256:c999b93a6b1d96c0050a514e29a2273072b5e153823ea06e7a53c402e3f0955e
home-path:sha256:51da11ce96c2dbe2afb85b3d31f6f8540f32c64d4751e6ce9d0957d5455a8e55
home-path:sha256:4a0bc14ba234ef619a796f79f8d6a315bd72f74c7e538e1f1bc91dda13ae26ee
home-path:sha256:d61042da8fa4915506bafc421fb6c296d4e6989a234c5a85152dd303582b512b
home-path:sha256:05767afec9078cc0be89ff93c83f313fcd84ef90b77c993178da79e53e526f0c
home-path:sha256:7458fd83b465f4c204a7050e7bdcea1f175205a6a47aa1a96cbad018a80a0bf5
home-path:sha256:f5280894541510ccb62ae80473bdd148ff48f48e07a9576ce188f3d7be07f113
home-path:sha256:ae0c69af604873653f36a7604edb4e23e4d0696716e55ea240d3c59c796cc8e9
home-path:sha256:2d3a5eff92a61651a2483cd13e887158ff0e98f70482ac588a7c4ebeb19ee11c
home-path:sha256:67e830f4af4bd55e9d3e049fe5ff48142b9e321fa341e9262af31f7bdb1b6ad4
home-path:sha256:e85745eee29ec91f470f1abd408a7de20ba7fed3cb42325b7d95d779e25bf312
home-path:sha256:2db96fc25c7d1fe421000db8d05339bbeef2b1704aebac266fe3c70f6d7eb521
home-path:sha256:bd93891be9c29517a9d6907346e8118b0f04d10cfbdb7bcaece920629f1da7c8
home-path:sha256:54850f32f43cb8daa46e283730ed9569df5b62497ef320274d0b54c992a1d76d
home-path:sha256:270cd13c8a5d024574de23a51b7d77a16789cbf151e54b756831c3e5aa0f01d6
home-path:sha256:314ad6d639757cf9760969ce443fee2b6c63e7432b52b746da233751bae40dc0
home-path:sha256:38e29440a84ff262c26808f19a56fec11379dcd38c4ba9f8d59c0064b91da03f
home-path:sha256:66f8db5e58289d983b81ec34b383f59ffdc4f5fb8dfb72c692985defb639afd6
home-path:sha256:5de2971c327423144ad903fccb109e23146056cc3e59e58ed2a65bae8bac45a6
home-path:sha256:2ea09c145ed8777fea296ffacd0a1dc5fe192fc7887876e27629633d739adb0a
home-path:sha256:cb6a90d5d82f32c179d78a66c42db6cde2c72686152690c24d04669fe4d5896a
home-path:sha256:91af5ee6ea8ea15beec24bc39248977671660b61a19513eca7b3566a36f9ec15
home-path:sha256:d5a081e25c3fe94294cb5854ab3ec4b8797af2ae37b9689c6817ec06c7bb90ff
home-path:sha256:72e78ebc26dba7d97f19bff467916b8b90bfa022f3cc305ea3e67563ee34a15a
home-path:sha256:b89206b762c05fc938c4d6daa03e8d44a07b5426dd1804e0984febf13b7eebcb
home-path:sha256:e75ac564b302c48bea5227183e6681f639df489f3fef864c31b04b984c30a3e4
home-path:sha256:c1f408d4157c223d8e84ee1614c6b1b0937175a92e690ee0b482077f3b4464e2
home-path:sha256:adeea268e18cefc95c1fb515bd66a90ef973e1c0dec5824c83b29b9e6d074628
home-path:sha256:6c8f02a9218e7e6e678cfe4659ab610ab1117cd16463c2b3a939aaca06911bc0
home-path:sha256:e0d2ce175028761cc7789c118cad4653480a45ca6c50e9abcb63593a8e01af37
home-path:sha256:98be20a355e225732553b6c490074d87a09a26b5812fa6f44c73fb3dcf805d3a
home-path:sha256:beb2639700b43b97a07466e7f31421233b91805ce21494bd062f4d0b54ff62fa
home-path:sha256:72d513b3e7a22a361a1157920467a5339ae60ce3c099621403906ed15255d500
home-path:sha256:d05b80c4a45fa3a5ef23f70e38188cefac14303562abe102f20b3c79b14ccbc3
home-path:sha256:c77db799121700e83f8cb3bebe07c4433ad7ce9e5d35e2488ca34b37fe85bb81
home-path:sha256:89350adc8bad85e70582c85deb37127aeeca1a6eeb358456853c965c111d1e34
home-path:sha256:6dfe0173aee3d9e812a2af6a02809d618496e7a5793bce9a4b7641a064f5528f
home-path:sha256:9642ceb549f46e34a966619f34af3230eb2a5116f99f2d539c9d36e787da342d
home-path:sha256:d1938cf1b656794548ba31c55616f9b1558eef0e97a6e3b2baefd5e331e84b71
home-path:sha256:324d2f0882bfb427b79c58898c2a6396a29a11b73154e5692c74964b290a1d1d
home-path:sha256:90a80692834cf084969480784007cdd3bec6da445842035843617c212da09b6a
home-path:sha256:10e3dc765c4485dfee76c462291bab8cf437fd537e5ae6f7cd0b415a1fa5ac08
home-path:sha256:4edebe4db67c2a84e9a05be8e90ef5fce09c505d0cd3fb846e4b84c23356b2da
home-path:sha256:0793ddb6c35a5fd9e7f7ffd4f77977b66d0c54d3012095268692e4b4fbf87483
home-path:sha256:74225be4eac5a373d87d8c06c820e50761664f04a8cf6fc139c243e2f93c1953
home-path:sha256:e159fa73f481d9bc48082df5623840f69b4278d3b3f358034659300b62f4254f
home-path:sha256:31f638c258d8b17765bba33845a698d0aa31c838c5beb851af40dce7750f9083
home-path:sha256:65b915254c22a43b1505b37f6a72dec4a07d8db6d5d862d4a9a0006a9be1ed22
home-path:sha256:da87dcc1e1f1253d3e5d3cc332d7d50cc94caa4497687e7dca16dbd0a9ba0eb7
home-path:sha256:3b9c8d640719ebb09719b96e34e1856b09b0462c399efab137cc9149435bcbb6
home-path:sha256:f38d8a03747e28414d418d9df2b5483d2a9ec5e24cf9dcd41ed64827759ce574
home-path:sha256:c93ce124853dee7c87c20d8f0136a80c1df465c077546de55040809818a47bde
home-path:sha256:afcab8d0c96669be18a8befa145477d10438b39d7e8baec044ecbc4234d76049
home-path:sha256:3b26e58bd37164786335890813a3f65305c36770b046024ab30babcbca664dd5
home-path:sha256:717a1319ef3aecd31e06c372482511fcb8c4f59fd629e780337fa989121322fd
home-path:sha256:9a604b7bec54913c964bb0292695428e06f141ee453bc5d5459c27c33ffa29e9
home-path:sha256:8f3f4dbb5bb09fb34c40aca28f40106197fb89f61e7f900e5d98e76dfab3305b
home-path:sha256:cc650fee4903ff1cb2238ec5afa15d9ab26031087fc6842c62af26c8a80a6608
home-path:sha256:811173e8cce9ae8f50539a97c0a4177e28ae1cf105b4ad6846234ea9b75071ae
home-path:sha256:942ab8bef0715bff1a8f54dd8c105d0e728ad750c78bb76fb0c228a82cfd80e1
home-path:sha256:ab3e9dc1a98ac7b0917c924ebd17039d6076de287c33497ee1104a02b9eeb957
home-path:sha256:078e5ec70c3f1f22037f2d1c7be4964ea94d5ed716ba0ddbbb8a3505ce995e8c
home-path:sha256:a7ddfdcb020805d965f315296af199d50f33a587bd165101c70b514cf54d770a
home-path:sha256:c3b147f2325fa185fc651c811786258dca6d3ff20ff4c2575f3eaca5c5ecfe1f
home-path:sha256:366f76e0e40dcc7df83d949e7aaea6c63f7f5f6cf098c4725e191c61cb3d7667
home-path:sha256:380ff458cd4862c30d55e8b75db77f47613d5ae51138342ae527bc1c6cba9a8f
home-path:sha256:65ca17b43ac6561cbb4dc88964bb364e7cb7f4cd254e238617579085bf9ce37f
home-path:sha256:cbdc359204110bbd99c76869c1e259a4d1b24635b2211d6e04d1e5dbb2a86426
home-path:sha256:7d9732c490b7ac57ed7cbf8c3706bfe2c980ca19b3e8a3caaca66b70e88e55e6
home-path:sha256:3c753f633807bba2988310e945d3fdf1cbe9083aaa9e7b410d8c41ac6c5b3aa2
home-path:sha256:257c72e29c7c0658ecad190ce75da6f4fa31abdef751ab2ab6e731ac2e108d79
home-path:sha256:fd66c9a961031a3b15c8a1dd593df2cd5e6cd081a698be581ec51d6d3fdfd718
home-path:sha256:ab1873a6bd11f5153b0b4e296d8276de4d9d04e2e5fcee0bfad655794764ce55
home-path:sha256:ee1417aa50da6256f8b9b05ed7e661e89262f7287a10f1e21d4f7b5403f88cae
home-path:sha256:612fbdb2105c52345b10af897a2f7d39f4e72c11ac4196c2f5b0ea4acbb7ca09
home-path:sha256:07425bc80603548a307be69c87a26237c230095265d57e62ec0c84016e3192a6
home-path:sha256:c66582aa85bca1e37d6034a815001b45ad3540554dee50f6058107bcb93c8429
home-path:sha256:0812573785880d9924154c9ad2297955ef5c1a6be67e48bab154d885e08b9e15
home-path:sha256:215be0f5d1652c61af292cb9579973398242e3ef76d9a87cc3a885e746cded0d
home-path:sha256:da5157dcf2f467a02822efde6436c7db8da87140eef1a776aeb0666964d777f6
home-path:sha256:02cd0ddd093b146208cc9b3ed8b0dca7fa81466c2160efcb2cb338f439a0a5a5
home-path:sha256:80a60217fa00ca2ea918935b9e8c03a8cd64f295a6a43873e4f7a57ef30eb68b
home-path:sha256:d735d275429221e451d8f991e25cddf05984dec6a95fcd2ad05c353b5363762a
home-path:sha256:08f2077bc6097e1f99aefcf0d70c3f28ee3ad7a11cca81fc3169b8a85ac27c0e
home-path:sha256:45feaad171611d90af2b7ffb84d102e28b8e0294b399bfd91e030b4f7cde33c7
home-path:sha256:d4c0bbbae24e1825a3baf4d485db286b7c9bdeb779b4568d51f24e81570e1d55
home-path:sha256:e02d26139bf6650edf2255b6bf244843b9bf689b47efdf406ef9eb4f55d388cb
home-path:sha256:956b51fbf7ff07acc85f873bf6eadb989e30c5f1db82800c82c71bd23d990bda
home-path:sha256:020602b896dd7a440481defc5d657b97dea4830dfc285374df170c0fd78b5092
home-path:sha256:5ead665810c88d1a180c7cdd17b0d16dc73acb2114201b8af3932e1c469712e9
home-path:sha256:6a4ad9f7902bae5427e8023f5ff0a22aaff3f09941ac0f333b69914753b6b845
home-path:sha256:3e8d157c538817b71447ba1326b8466933da95977096f6cbc05950f64627187e
home-path:sha256:3d4bfce997139605deb3bc11373ebfd784fb2214a301b288b05b6f5fb4e89292
home-path:sha256:8903eb744129292871de0703aa0790c8662977c3915a446a99f71c860d377019
home-path:sha256:55966b03d1ce1fe49b39e6c096615792118840895bb94f2b478f4fac5a9aa164
home-path:sha256:869326f7afd80bb8f95f24e3a14903e06958b6fba746ad669e4c9632b7b6fdfb
home-path:sha256:94b9f3ee2ffd576dfbe80a08b4f3a28d881094c57efec836762d25a2c6d0c69a
home-path:sha256:366bca9ee118638db43bdeb9252355f8b6868c3232fe44b3292d0666f9ef9fe0
home-path:sha256:dea56945a191c09d5d1d4a73085c050b3263feb5701adb408e1c797841cb683b
home-path:sha256:4fff94dd31b97c266fea1afae27496105367069f515a0c12f0047305f3e511f1
home-path:sha256:d9d93b87a133b0272f86a0d312596171a4abebb531c69a4f8d276cbca76e6a2c
home-path:sha256:69161baffdd8b264b8fac5149f541f137e00f265c54cdc2ad6b8a9241fcb5f54
home-path:sha256:3e477fb361d32a7177386ae2eec6fde72ca5dad348f54f061afbc2ccff862055
home-path:sha256:f7274580bbdf6f5cfd9879cda819a2587e23f1639df99b0644ccf90fddd65a3f
home-path:sha256:1e97a74c906b7142447dc0898da03f962a48c6bb6cb5d9673410d10accea19dd
home-path:sha256:09557306780720aef484e013c5692e81b971ab9278f58258792b02d506516793
home-path:sha256:279bc25be6513b4cd4a19bcfe00dbc1ff449a7ee3321ad5310df1e7fab35385a
home-path:sha256:36a18a662f3f28048a403ad913a99ed817efe1c59755e20b6b49df0547926762
home-path:sha256:b6121e427d383f2c5572615a1623c23b9ceb912ed773cb7b61dd7ce82fe231f6
home-path:sha256:18c891e57574f3ff0b4128f8ba53e6d27a205aca4d22d511ebd6c567d760abcf
home-path:sha256:18f765235db26a82c8614ee5863b13c9e0341d0c14931a3bbc16297c1ca841fa
home-path:sha256:90bc1ab91ad673cdc2044d0b67b6f133a46783e3dbb33ce2d7a043001c19e68d
home-path:sha256:5696b0248f9c74c71789b385a61c9e8f46d26f49760acf4a5be9081f0263473a
home-path:sha256:7aae66fffe88eed9cf6b6895017b9ce6d31fab8d04f6d2fee20d020ba1ed0cf9
home-path:sha256:97382665b4ed7a1664fb4a4f23d7563dfbff45ad5a246e4c7c06a2b1e848fa4d
home-path:sha256:19872d710a717f6db1cace38e64abe4adeeb4ccb5054f446f30af0a062894289
home-path:sha256:5c7d42d978b0ab5f6a445e6ed78c2ddc08c05c4fab37f724267830ad54d15adc
home-path:sha256:014c95870a83925c2e47e88bd20fec5d0beddafd4ae8769c3edc2358896561da
home-path:sha256:d26dcb987ac3c642cdd99e87123b2409320b61f5d01f3f754878e84c3995fd33
home-path:sha256:228fb3988ce8dd432cf0ebf1c03df7566075003af73f59e66709050a23844173
home-path:sha256:1c07a7505b9b79095eef9742e8bd973ae2dc51a98d4b9eb51ecd4f659188b34d
home-path:sha256:94fb96d3c4e3fad62ebe303f7abde0dcbe5400571202361120e48fd7bac69215
home-path:sha256:b117b5fc0149e81d84d7266ded22d8f1ea40db382a07ed3ce05461bd0ca40199
home-path:sha256:02a2fc55c7c2c11c0c310447bbb1a68c6ee8e22482fb8af22386489ae306ed3a
home-path:sha256:d7ff4ce58f070f6e4dcfae912e81797e106b58cd2ac8ffea6816a3b6f7d8de67
home-path:sha256:dc557213c03e75042250b296b5b9ed9a27c964b0a268ce21047309cd7eb82f7a
home-path:sha256:1b60de5d2700bc1f79d0ede1ade222f5dda68bab2bf91b992662eb5297b476f8
home-path:sha256:9cc8684a591d58cba9a3d13cc7c0e5fdbbb034f481006e39014d9a298893325e
home-path:sha256:deed93bb0fb6c3f14fa8885d9161208cebc8fe5fb429d953d047e1ed727893f3
home-path:sha256:3896c821127563554bfb8045178e42356a988f39c7c8ecb87d95cfc12043cd21
home-path:sha256:aeec02d283d0d9f93c7c152aaac8479c9851da76181aa48db179ac6aa5d54a85
home-path:sha256:cf9b9642444299ae4ea4e6bf43795459e1bf7c987a0a7411ef5d9aa985473106
home-path:sha256:fc6d1150ef5b3251f5a87b76180cde7011fdd43a6528ddb1c4b1ca9689bfffe4
home-path:sha256:0c88cf9ab175d72ab9f555133bf2025dbcc2ad03e7e7f198f365d67973c2b3c0
home-path:sha256:5c6597f566c472ab2f2999af704d95baf8c03654029568562a98bb4eedfb9a91
home-path:sha256:0d76c1040b88b5d8374f8fb50c0906fd8a3ab231bc11b7310050c0c1ff6d3add
home-path:sha256:24955b4bee47432ab3044c7cdf4e3d4f863891e9e1adc116597a599a8449950e
home-path:sha256:8fb46df8891248b589c39b31a5711b03d565fdb646eddabbe46069e0725fa52c
home-path:sha256:30f3f310cdf2af33e1ccaec503b124a513ce6972f64a9d5531be9a1b1b184dea
home-path:sha256:e857e869326f14a67964a424f5ed14562f5ad499048c90661397cbb4365355af
home-path:sha256:0ed38d3a97342b4009b94daf6499c8a2782896528d819888225a3d416bc77e66
home-path:sha256:a46f29511e3fc4c7e2e8a6e026dd643c60389ed1585b9cf2d727901f189d01fe
home-path:sha256:24235e11c895f0b57119288b17a3a2388808fd749d018bcde722c211ab158893
home-path:sha256:e852648aae903fc88495db667fc6a01f2cf9dd5cd7d39159babba0a8c0be54ef
home-path:sha256:1c1ac81f76f6b9d6084f53561992b0ec5c7075c2b0ce0dc46d3f40120e517b08
home-path:sha256:15dd8044eea812435447ae22246db84a44bd32750900cc7bd01f112441be3457
home-path:sha256:fc5a199992ea341d3e67711203d82b7bea0a3342c23dd75b4b73d2eaa7bec132
home-path:sha256:a0f2056256d0795bc25585ab764d8e2e2c1ab3f1c2590a3c11a7c8d984bad544
home-path:sha256:eb49ef727997ff2e1c537c7768db70c89271581c84d56feacd6ca8d7b29ced3e
home-path:sha256:e494092a53737f65fc9d9f0041c4ad812324a893995e033d87e1e2bc14aae1df
home-path:sha256:0f06f6e54ce2ab64c0d4d553a23c01dd084e8a7bf956d8b96ad1b74249f0226b
home-path:sha256:413bfd44fd5d9d819dc5b93aa307283e6a6f298967d74f63a54b94f58402884a
home-path:sha256:8cb7403dbea93dba004cef1af9c8ce72bed071bc79bcbad5ae0293b6d248cc5b
home-path:sha256:0524dc421236f77fb24f87f046fb5259d29580b77cb45a57e9113cbae27f96b2
home-path:sha256:0aa7ddd5cd3f9f543a6bb75c17596d050638b0d60cbdd15b83ac2c4e614dfcaf
home-path:sha256:a7c407ab30e08868c6536c0f5eed6737df63a458a82713996b5d399e50190028
home-path:sha256:1320b2f70ec9bec1904d895f5c39b970a24d8c710407d62f9ffbeb1d370b3bcd
home-path:sha256:9fd680d2d147148ca93b110556b8486a2e71c5a57e50f7ce36bca9715e30bfd9
home-path:sha256:ba5e26b65aafcaf5f8927b8d32c3d852ef4cdf84b2938a7af974da1432ec6868
home-path:sha256:f4a55130ee3deaf288bba50646946c82c6bd6143a61276abf04cbe53692c0931
home-path:sha256:5b2f717040ab93fc386f7d9918a2093fcbc2683b071dc75d9c87d97ac4ba5c1a
home-path:sha256:3b7d8f626ba81a5a37e961e36f8674fbad0ca05693fc57d42269d997b1cdaafe
home-path:sha256:62ebfe86fc18564252ef4e3c99ff491521d6c5ee495bd732f60f47f610aa6b8c
home-path:sha256:b3645c5b1d83427ceacfc65de62da428bc9ab5d18f891d65acf0d1bd85846aa5
home-path:sha256:a4b5b82471989a5cfb4ed2201ef791f63ab759bbc58a201bf33a91be1dac6719
home-path:sha256:08885f0d093154997a57c2a90248d664a354a2d670978d5bfd2a9c97ad6cce66
home-path:sha256:77dea8d73f5cd82f9d655d94c7cd8ce4f6ad87a9575f4114ae5c10a4e8acf4c4
home-path:sha256:b216d886ff8f930b4696a47a8106608e57b0f26dad2940fb5ab92dbd36aeac80
home-path:sha256:f45d5401e2beb5a5b527758916c60766500815fa3e3481f654aca70f5eec095e
home-path:sha256:4afd5d663540bbb9a45f43dc56433767d393ad67f4378420498784aa982e0d07
home-path:sha256:f64f683ab5abc0e04b07ca7d9f0186bbe2619de1b19c96ee70b01451ff84bae2
home-path:sha256:ebfffa98be26645f2bba9f2008695510b5e93a530fb65b96f446650645474291
home-path:sha256:fdac78da5be12bf55d584b4724273496dc7b1097048a8ed7bfe1754b9e3be4f9
home-path:sha256:5b973699a385261cba411bb20a934da2bcfc448508a91bc2a9e41064a504634e
home-path:sha256:53c30f27f4fbc10081929feaebff171c39c0e6b0cc3861822d619a5739104bf1
home-path:sha256:56d0304c7d91f0708ceb8d9426d96618885b8fd8e4b841fc00176346f80eeccd
home-path:sha256:1571522cfeba0a9263b4597c2c54e69086b29c495a1f913adac61fdc4b2236ac
home-path:sha256:d51dfb7f3620ac8fb3196fda5c1dee712ee05f123ebb7e17a72636a7da467f28
home-path:sha256:aa058bec353266e9507c50da7e9043a071c4c1c243a3d1ee1d243366af4f18dc
home-path:sha256:9a9d006e4d21d22e56957e4ed97a8af9834bf8445091dd2dd4ba436afda4ce22
home-path:sha256:07741027c2e3ee9d8b8839107a75e59884674b012cf101c856f37892f9e1a8c9
home-path:sha256:7351582c2b7a2cdecf37c22aebf0d9903b313ef3925cb00b48c63c3928a90439
home-path:sha256:4c8052a74f27bc48294c72b324b51b3fced0744224cd6d6180472ff30151919b
home-path:sha256:8f6ae7ffe0ea3fcbba61004e9339352651e83701b1da9b87cb30ea3f45dccabb
home-path:sha256:399860c782467fa1d380122fcf7e5b9ab195f377dff1b88abe915007dde436e2
home-path:sha256:92ab4ab035d093cc44abb30fb3d716048fc490c9dea3d7c3234556f3c4f9253f
home-path:sha256:7625a66d109a0751bb65962ac6ac98b84550784fd68b98efc385d98011927731
home-path:sha256:0b3a1a1a8590a3f2b038dde4783f04002f4821f9bd25290d91eef9c4981a3687
home-path:sha256:bf414b1b096869749c674fab83277614ac12767245a7938674331357c057e4d6
home-path:sha256:532a8b27e4469e1089daefb25c06ab7afbef255fea150be2c1c38a44c5bea620
home-path:sha256:c488ccfd4b85f611b0d111fd2d9db4458649504fe74c0d68b3d247a5e16aa203
home-path:sha256:162b8c631651d2ee84446e21e519779b16a50a8ee6afd1b9513e40dede1fa5d1
home-path:sha256:6b933bdce652d9644268eca29ef9d271105b042b4c20e8c3ca812c0c2ddabb80
home-path:sha256:e64e9b502c4817e996c77cb3431d45f2910df099363a56defa4d02d594ac0114
home-path:sha256:3f0a14d6d05c3829a44b816d6d3faf3394d08490ccb66799efa4302d136e9287
home-path:sha256:f5c1dfc16607239046177f49031d11fb36b5a7c45790000e6765847a47991dd9
home-path:sha256:e4c84b5d924241f2cb1e560f341cee71b0de78ff86317c5c78ec50afd70b68be
home-path:sha256:a9955cb4cfd74ababf53b8b6ebe740c294707918a3c4e9268150646ed51af564
home-path:sha256:7ba48e9a08f84c2e873356ac8d7e0cd5a4255a5dcd9a02ddf0372d1e77fb69a4
home-path:sha256:dd73001af881343f487f4608328c594149b25f43a70910e754e0cf097e2f8891
home-path:sha256:6cdab27ebc52c22cee4f81fb5c3a3cfa1c2614a01e0044089b6b6349be7b4656
home-path:sha256:6b00073609483788b8ce99d91eb19935e4013ef7470cfded9044dcfd041d7ebf
home-path:sha256:fc2d97bf51eeccae838e8d0a077276827ae8d808dc551cdc8587c3935da8fa90
home-path:sha256:4df34762541f3858d2dc460b29441241e22a97929e47ef7538e8bce05f231f3e
home-path:sha256:1c535cd045b0bbdf16b4a5e59053ada348e4bf4d617249718a1e8f20f59ee629
home-path:sha256:63e2aea102bd8132bb836f67c735f1c4f0ff869ffc0f7f79de3b725a959a641a
home-path:sha256:3f73b4c7678638f33df708518465f65313fc3fd15e23476770bb694205832677
home-path:sha256:e87f2ef21c323730fa1da97eb3d7ab374f8f226655dac07faee8e1033842d0c4
home-path:sha256:a46d709fb98e4cb6d1b06eb11082b56ed64c10d6bb88e2f9f88535f7914f5ba7
home-path:sha256:d5d1f8a84b783b65ff82065b4262f7f2828c022d44ba6f3bf93d31ce7723f38c
home-path:sha256:c6f074efe9809b5d218601c07f875fd63a3cea94b9a1a0d7208685bcb1f42799
home-path:sha256:3af9286538f02c9bed62ad58e2b5452e230cba453be5fcd2401b808c723f4fc2
home-path:sha256:50068c7216e82021c490584255ded38f68e50b753f4b60d6011566e3a2a7ffc4
home-path:sha256:433e950e1af2f3e5aabfc10d6aa511a12ebfb61f4df5bbabfb0652cea1f5f21e
home-path:sha256:f0e9cd4c8ff73bfbc4f8ce10649921239d0e635184663f85fabcd6fe77c38c61
home-path:sha256:d7575370d49fd0a4b98e1b114ab03da6230129669761004c16a244e05021933b
home-path:sha256:ed5fe90dda2ec100fca914a6e0679ff428d05be82ce96bc7f6a9a8cdcc86cab1
home-path:sha256:fe7789196bc5460bd56b415eaf8a16e8d6d5017fb54b992092f9f8116157f739
home-path:sha256:ca53d2f2325a22eb65aadf12252fd5e57fd801e8c3d50776e153556517d4b268
home-path:sha256:369bdef296529e456bb2af37677e4a29a1fbbd6cadc813927dec5767eef8032d
home-path:sha256:6aba38e37c45ae5190391c14106887daf5e86a2048836226d4f253d852ddedef
home-path:sha256:5bf77cfe357e1c8861ddb416a46104d66ed3b9911225060ca720120aac4515bb
home-path:sha256:904ee9f6d1404a5d39750ddd3168a8adb807ff602ee00c4b5632d8a409e59b8b
home-path:sha256:f4e01133d6796fcd65d669ffe91e44d50a5b3e990b6e3f4cb18336b171e19a9b
home-path:sha256:8510d412138fe344edc58c9ffd2a08d02e7f6ce5eb4fb34478b10e66093237bb
home-path:sha256:ea8c30ca91b8dc8600cca40b4d552e038dc5e1957604a614d09a61f6e6a03b94
home-path:sha256:82fc26ab8f73f9e87e5811ef4b91c300538b318749da9f16e0f86599fb716315
home-path:sha256:7c05da7e2708156a07fad967f579000fd757dd0141fb8e873179627b4a831c7d
home-path:sha256:d812b0dc890e242dd111313d3ddae0a0c9250568c333eabdbdeeb26e0f19cfb7
home-path:sha256:1c70c14c35af16db3e8935ee6a2b71891d6bd82f425c5a151d6e0435d58f7e5b
home-path:sha256:605f1c8606dda8be849d00c992aa1cc3fa60facb28183543e91719019fb0daf9
home-path:sha256:bbf79cb4f9f3f7a8c80778c50cb8bb6fc013a2285500aca236ea4522a2671003
home-path:sha256:974f046a5641be952de92267c0c19d6e74af194dac959de87168f9e9e04a854d
home-path:sha256:ab17aa42f53fcafca540a5ac6f138e11d17d080ba2c8e674f7728a6c1b6f630e
home-path:sha256:9aff5a541e559fb23698e288f0400af2af66d8878c7cd1fb275418be36c3ccca
home-path:sha256:f620ebb3b7c74b68227cd2f079b21b0fb57216c8608107f4b45b7b9d0765b6ef
home-path:sha256:003a193284d73856a5b1f91257ca19c01483015bee58ed76a82882206d3e4bb1
home-path:sha256:58f743aac8401387ed46eb2f7c83cbaccd4682b2d965a0f093105cff4375dbd8
home-path:sha256:7ccfbe90c3c83778b363bc0c803b0e82fb4f1b5b9538af1278b3a9940c8a27be
home-path:sha256:709d86e9958bbe010f61549f5a146339477e57c4cef5bbfe5e793987aab54b5d
home-path:sha256:d3e0eff82fcdaedfa300f4639ab87d65ebaaaad77a38e8e302ef96a36d3cd36b
home-path:sha256:2e1f7483dc8740d60441959242f4a40a528181d31602f2d86bb4ea836efa4237
home-path:sha256:aaf54fdb51605d96c08c1ce7ea5817f67d0ee814ecd082b639c3a64787433064
home-path:sha256:ce2f541374ac86079333bdf890898a1d939aedb5777a8f27a3e705b7d389f4f4
home-path:sha256:860f4d7727bb62424c3d13d35c737c739d73700d081e4a44aeafd87e08deb8df
home-path:sha256:04a0a92317aadb64624cabd9f39e55b012f75591421eee3392bf382f114c7801
home-path:sha256:afe29eefae5181ad9c03a8744a0dda16433c72253a5d0370c76e10c84ad3c3f8
home-path:sha256:ef023bfe359b1d56af8d9e777f91e636d03a7858ee74355f27ac340bb33970d1
home-path:sha256:7a85a7339a05b3b6f03f9e33ee25d1b022ca1e26fd4c4dde834505a9c93d55f1
home-path:sha256:1273af862715f500dc9dfc3c0d6cb6531e5acd15034a0f6cd187eac2c0d21e44
home-path:sha256:da2c45a3cfdcc77854f374699ea4dc081e1f46f6c31ba74391d35c0cf220ef25
home-path:sha256:e5a70404bf4700ec7e43ec93925846692cccda0e2082ca66d957bd2484d5f76b
home-path:sha256:cf964d6210994b7b5e50ef56ee27f8c24990a7fad992e249b0a089c72d00b80a
home-path:sha256:027dae654f354c5fa2294a05df750438b9f26507d582e39c33d9c40f8bb08172
home-path:sha256:864c0c926867a339e8f8557e0fc5ff6b1bca47fef5126e74ae99d0da51350280
home-path:sha256:476543f18c4f7d240ba60320d23fdd56681746eced486b54c9f7b801f85d4638
home-path:sha256:80e6141b0e3f0db95d2917961bbe03725479115ad81499a392eabe5d55ea3f40
home-path:sha256:5172f2e194bacb08bdb96eb52e2bd3b15db00b81465575f7b0adf9c806043680
home-path:sha256:274d4956553402615fe7b1d8761d45de9be89ccdc4562bdd24bac52467900484
home-path:sha256:a4e7027fb86a58dc6adb7c4cefc02395ca7cc95d20652a382346503fc17134ec
home-path:sha256:8638c99a0a8fe8cfc0a5edf1a037c8921e3a4e33d1fca4ef746cbe939c324912
home-path:sha256:6c75a1160554b0796a2105a5a19112868ad789a59089aa87cb296c13c79e68af
home-path:sha256:aec0968892f112fab7b3be06322fd35d9773f14079876495321fed1c4db337cc
home-path:sha256:45339b0e3ac68c29f6bfdbae47628255b74ba027f9b4a68516a7f1711751a3eb
home-path:sha256:bde97b4a3cbf531dd293034041aaa8c5ce6a678e0f25a1fa9d4efe1690c2db84
home-path:sha256:e05318f67bbfa90f99dce709f600afa5dd16247d28f0df5a870a1d52dafe73d4
home-path:sha256:65f383994a446a0c00279281fbd6d215bb800f78cab342a5a9656554e2c1bad6
home-path:sha256:72c8b72949e1de0d9f9f7d3d0f9a113eaaa3e65faa54e1d6acd62aeae1ed34e7
home-path:sha256:1f27dfd90c8f74abc7e1b98e8f0b08c47d49576653756147260e13f3ffea1c05
home-path:sha256:f7e75e8e4e490670262cbc135cc2c4d1ee5914853ba57670a27c467ffbfcbde1
home-path:sha256:4e1bda186d10720c0f489010ef74902aaacd80f487b1321987100a14f9d65dbf
home-path:sha256:2e772fb7cd9d876656a27233253df00dc175480aadcab5b28af906fb93cff07b
home-path:sha256:ecd3c11cf8e97c6877e42733af8089c1c974d9def6bb0bddfa9f3aeb220a7826
home-path:sha256:113dedc9e598082e8f82a5d525a1b6d18890994aa82a39f24b2d716d682b1566
home-path:sha256:13dafea6e847a9777a50a485455e877294b2e0a437380742b99c4696bb21d1cb
home-path:sha256:f05fb0a42f37a0aa8c33c0ec534df2ab98f26bff2853bfe070bf4ee3e37bc8e2
home-path:sha256:c3f191371bf407f8bb779f185e152045302f33970bec5a4259b99fd0455a59c2
home-path:sha256:583d7492fdfc93ebaef6c89bd7437cadbe0a6220b090391d9277f1e2658ff82a
home-path:sha256:9f13a93e1b1fc148616f5f5b295007072340446104a1404c47a7d69c0d0fbf96
home-path:sha256:8ffbcdc84f30c9d331d490daf818c6ed258b7a82c8ac9e1a1b548a87b2937a58
home-path:sha256:6b2a1a7c52f2cfeeaecbcb2a11eb5c952298ba0499cb9f933d319c0a6ecff342
home-path:sha256:73eca95790cb95bf5e37afb86ed744548b1a89d80700aa9423119ca374e9f0b9
home-path:sha256:ab679cf9549d9e67c9d623b98274a996f07a29e8c984b828573e94541c612bca
home-path:sha256:0f0cea0d5abb20ec4f6cdf33bed7ea2055e49ee8a44c5e6040c04965372553bb
home-path:sha256:2d8018fd214486f6b1b868809b2f5f22bf91e392af16755ed2e1b9ba4bc5472c
home-path:sha256:c7ea8747b90283d025184bc16ed6b3da42bef2151375a7975e09b06ef5225d79
home-path:sha256:d14a6498da158b7673bdf3efa5b3158495bd1bfde0e87b574429079c5eef9987
home-path:sha256:c353b8b67c7b47584db7f374e0383156325d3a14ec9c8f9a1450eabeb386aea6
home-path:sha256:915c65acd608591880ee40ebc6e68d4ffdfd65d82925d9fa54c089736db91a1f
home-path:sha256:7298a078201f9e2bb26a21bfc2d37391479621f5388c87ca05ed145749d33575
home-path:sha256:f41b24ba613a324e6a01c7189a3197038ea9de1bc2b7626b861692c2fade0c06
home-path:sha256:e9604fa60a629ccda594b59e5a560f7b6a9a50c9d60b834ec4e1dee3cd9c759d
home-path:sha256:d8983813fd8ece6eb13e272e68050d79a9ddfc21ebe0ae87f3cb155e4a263ea2
home-path:sha256:1d7c65a752711d5653256a7882f047bba05f944ab392bff0bd87a54c122765e0
home-path:sha256:24ce8c01b4ee794ebd0f0def44c220343e2f31b20704aec4006e0789d8173a91
home-path:sha256:c18813da8a4dcc014dd3a468e3633ab435a568ed19839683fa54eee84d64df38
home-path:sha256:30ee52527c4aa61001bfd166de916a13c241b8017cfb954773a0c4aa3d0df9a5
home-path:sha256:7441854cca0b88be22e1d04d998ebc7c52d0e42b2cdc59a27de6df7445f879c5
home-path:sha256:e4163afecaf26f2c802c41a5978db538b4118e08703110da7edab471b835c84a
home-path:sha256:2e541bbcab610f2013afdbaa470a1af4cbc60ecfa87e93ce0ce5a2ee0ac3e304
home-path:sha256:29e908c05b5442929e69d7e1bcd24331b915d5cabdf18ff25249610cd4634e8b
home-path:sha256:7ecf26e3849ca65eb1883720a4c9ea38b5b6be6ae53254e0e4dd1cb991779efa
home-path:sha256:1132842886a5305304e9e0187f5d20ef1fc40ca4f992c98a3f7995acef8b9941
home-path:sha256:1580da2d697029f12f5e7677a5f5bb3a032e1fb1b3e847fc28e7757aab88383e
home-path:sha256:0120b77c6ad11771e47f692eb180f25cf6fb2e5562960035fee34f2e4833bec2
home-path:sha256:e48c2cb2abe6a20452c8fd9bd6b6eb26b77e7cb2e7a600e3c313221c66460c86
home-path:sha256:d38786a63e7f0c634afbe61df6f0ee81d43a4b1489dde4457c54673788b7310b
home-path:sha256:d466b6a98b6165dd3cc7bf41d3ddaeefd3f63e27f940cb5501806d8305512b54
home-path:sha256:617cc9096d408c921d55f7d0f9626de8ee0685ad30d20b0c086c400e781a4597
home-path:sha256:dd821d91fd7a15a14b8a049ba2440e8490e539ce46b42fb69dc09ec170a9774b
home-path:sha256:1e302e63d700f99572a9c8ee77efdf2dd37f6929624d787b381115744aba1f77
home-path:sha256:a347ba9e339a8336ddcf4c8fec93a7d914c0147e601e7498767a060f6cd81c9f
home-path:sha256:3f3f44eea09ab28e8caa87d3740158dc4df7eb788029e8c045070f4464d82f90
home-path:sha256:5346ef44e3497ba7ff59f5149252a501784f9b55ae53b0c048d813a20e890aac
home-path:sha256:c1d48c5b7eb2c52e81dd12d588f9964700d0bc4cc151824b40d798f78fd95b17
home-path:sha256:129c2decd82583d839ff6cfc4278f522e194fd24cffc884bfa31fce78312432b
home-path:sha256:f5d7ca53490010f91e950463003dd505537c62cc450c74632b054fa276e99df6
home-path:sha256:a075492bcec7c47d9b4cc9aadf09df42ee61fed8df20796f31dce30698abe686
home-path:sha256:97cb070c7d6767aae0f347c4edb8eb68d82bdb8a01f92de6f1072dbe3e0de330
home-path:sha256:85409e689920eb1d3fecefb25f6e09a2919609f8e28d663f1517d1cbc1c4ca8e
home-path:sha256:279aa24c19265ee091868732ebf1ee608c324f2d4cf519134ce4faed28717919
home-path:sha256:3d05d5a655dfe0a5d3abd372262fa5cdf916c1a9e77b3596da333ad47acb7526
home-path:sha256:1d9edb4d73bb4d7370805db969507a0f16d8b12169a7f97f502e5274df11f401
home-path:sha256:5e51ccf5d6ba10a4a1e635187c4da1e263efde8a41211adbae3f4227fd717961
home-path:sha256:2b80072091e984e65c2e11e2d536e834e5834a96c3d15da7c7b8be610648f814
home-path:sha256:1ed18f4be9aade4f44e83950cde507c87ea691a5ccba002b4e5318e39d8bf890
home-path:sha256:5a103c30606aceb827399c044b314fdfe634152c9e463d86f185526380090ea1
home-path:sha256:7848a0e3ab79093aebc3f77f3662c09915cd964ac5ed1b351ce14f6f897682b7
home-path:sha256:868290d793cdb2420863777f9ba2fdd722a4b7eac58062719916f8670427a305
home-path:sha256:6608d2f60f5520dea638879dc4785be3d53aa940ec80e5718648a506bbb90583
home-path:sha256:1da257b5049baa3cc5b6f3e98427c4b429cd1c4cb39b77f8d04b2dfd9145b414
home-path:sha256:314cb253a101812790c64f1b15e8667ae9e755e021b9d15e1f8f9c4d1ace5009
home-path:sha256:795d43fea7975ce1647b6ad47a2f0d7764a20f73c5643f6d29b3e337b0ffb5ca
home-path:sha256:e575f67560204680acacd73cf9b7bf3b62471630a58edde4dfafa2f0bd229bc5
home-path:sha256:b07acff1025637d0bf7328c7ff728a0d185af0395dac4465a86ba5000266981a
home-path:sha256:af38a85a3527b9910809134413c85d3aad4502a475c0c2fdd75cfc2c6908f243
home-path:sha256:4ec94f9e6a7bdead4690c97b7f87e9edd18450f58d43dd11a4f389bff6f1c0b0
home-path:sha256:44bf7314ca963419cff4a4c31715d1794751a771e630071c6dba32c9b3574037
home-path:sha256:8095a504f7e60b76c4eed2c01ff00a5c6722776db2d79684532d0e028d6f6ba7
home-path:sha256:4ed0e97059423a30d172258f1163cabc46e33d8df82321b88b9decc18e3a1d38
home-path:sha256:595a144f4c55849cf256df2be40a250dd55f91b560922279430d444f733f869f
home-path:sha256:e0dbcbf68e1d28102e8cb27404041cb8909a3bacbf48378bcba705655755c7f6
home-path:sha256:0ea20fb1d42a91a48eacaeebf2b989b905d0e306e23a9803ed5dba680f5f4756
home-path:sha256:b00e63f8bce735c9635d88b1ac69af029f8309dab89fb1c98677e1c744ad6966
home-path:sha256:f8dd6fe6570184065792f7b9d89b87b04df5d87d29c51284b4e62f3aa950b313
home-path:sha256:39d487401299acde08f8aec5fb05af1ce9703fa96822dafdb28570929b9ff7dd
home-path:sha256:b52efd4bb32e17d740b22723616e0895cece27a1aecf2947588cd6a373905833
home-path:sha256:bd814587e16d950438b3f78eecfe0c17a69ed4ee17ee6dd6b88b6fbe5df50c07
home-path:sha256:210245eaaf8a45533254720b46c6d53e89adde8637181b1008b1a6ff1dd57064
home-path:sha256:95d2cdcc6b02f40530da18b3adc85c06db4a3d4272219989e591c92ca439765a
home-path:sha256:3980e584f86a6de2e0ebdb6bf46b667b277368163620e80df2d6a526b3d9131f
home-path:sha256:fa36f8b37af8ce90c3b50af10e44320745dbac4533a9140d30703309cc4980b5
home-path:sha256:816bcc8d1f5a944e00312d3d606076f8abd5b8cd61018ad24a56bb1a43f6c450
home-path:sha256:a22012a1cddfad69581a5232afccfaa018fcc2d8f732ef9a50d8fc1e5656b8ff
home-path:sha256:3ae47beb0bf2875234972224255684a59e2effba995dd683e6719d7eefcd1478
home-path:sha256:2f6f49426d216c5e2f402171e36a5629a8f4396af65211f628626f2e41ed317b
home-path:sha256:0e0d275125a99b71b7cfaf1a975a19867d200fe2b37a30e6fd62a34a53dbd51f
home-path:sha256:ae79b9dbb6a02e2ac9377561ee2c1f0af18d601b0b73c9b83f185f2547b6690d
home-path:sha256:fe139675a7ffbd5b2762bf92c8ab374900e6d8853f881e29bf0590315de95430
home-path:sha256:d96e7e48ed8c579190f2720318071e8eda5ca708f70337e117a2600cdf9be6c2
home-path:sha256:205b2f340397022e1a3b2c5080f6b0be365e3c293f27e5f2f2490d2522f3dfb7
home-path:sha256:332f3bf32908deb94c448134ad44d00818cc5e9576269f8bd712b0b6161776f5
home-path:sha256:c522b7bff3606e73f2d075a51d5e924fdae8e1c875358331651850311e8e1fa3
home-path:sha256:61779904882edbad9d355f83b12e3bbc6a2b7a216a85c8b0a38fe2273be0fbd4
home-path:sha256:3fe461f3d2ad48b697d4d71602c2fc9b63cdb19df9728a0fe7bacb88d50d1aaa
home-path:sha256:92aedf3f8f5f4b0d0e41b0a65be4b0b8ab4c5698c094f673e1c39aa944e55c34
home-path:sha256:f2a343fb0a1df072bf85a64adf240b981780c37549a73e7ff5a9290809a6c022
home-path:sha256:39829ab2f6172bbfef50506d176c7f8999da874e56a4bf6c3d21ae11765b2734
home-path:sha256:c33ff02baabb94ad55039a66a87c3e8024ee026a15600b2eb6034c26e6210c3c
home-path:sha256:7d1e55a053741b4cd896cb566f2938f2188b5ed704a5c4d5b5b9927a9d0045d7
home-path:sha256:4b2d2c48f6682f9b6b6651912082a93c8984cf3febe0268941b1dc12bbba6da2
home-path:sha256:c10fa81688cc0ccdc4bbc27a63cbde2b07f0c155e5a607fd5c04c69a0b478fa3
home-path:sha256:76501d0026957519e690bd15a7e231521eab909c00346671cf4e6d68cefa5d00
home-path:sha256:9591b211c18f20a212bc2af51882ed7f9b2c1bcee32ed9109980b9d036462047
home-path:sha256:23a9215e082bd1d9af0436f46b35f146a8f9549bbd28021e437c8fb6dacbde11
home-path:sha256:bfa9cad8fe159c4d10634a12decdeeddb31b1a1ec1eece3d190190da81e93b2d
home-path:sha256:67e03974b042bb8cf86f0f895dd74ce856b0879172372920bbd3d396e4ff9fe5
home-path:sha256:3499b429c747e43ed104763c069c706fff09f9ac069c86e94efcd16be80804fa
home-path:sha256:ddf479797699e34f8b8e589fbccc749937f67342ec937b504da43c0d1c4d536a
home-path:sha256:661b7e8c2a6d4b537322f089b1a7a1ecc3d7897e08b0d1b98d3675ac726dca81
home-path:sha256:097d36959e836c6ac07596cf763c4fc05d323af99e5c8cb7d1864c69ba3e6e77
home-path:sha256:5b8fdc8c7d4fcbd330e9975580184fbee447a53978ee56c48f68fdc44f8387d0
home-path:sha256:2e98b86824333e24a04352578222aa4cb64b16fd00838e3a07dc84add3757bd7
home-path:sha256:550864ee435a9c8f0b0f20cc41d677aae1c68f74ef7bc5853c28332278d507dd
home-path:sha256:aab1e170026406393d6c0776c382e4bc5f09d76e9cf21b179c85c63ef81515b4
home-path:sha256:99f8d84614d2851bc0d6ee90867fbe06c3028f8b4599a92dfe94d7aeae828bad
home-path:sha256:fd205b3d2a2c8a3dfe906357b26f416bd9bc963699855d84b368dbb02b4cc8ca
home-path:sha256:243c116fd29b379b54548af19430a1040492a7a6b93f770f3e1b802e860102c2
home-path:sha256:ae97c4e187a5b0f74227d9a337706a7639fa036085ca7934a09a6209e4f1ec43
home-path:sha256:d2f2b1c982513af2de859b7fcfc7fcd93841e454fb304a0e4b8d28872450ae1d
home-path:sha256:1569ce4d45a62cee3ebcb0c8b945df3a18c62c60ea62b01e08c264cf02c4ea2d
home-path:sha256:b7d692623b71ff4ee3c84bd4409e394172f48d7cb7febb7998924d7e500582ed
home-path:sha256:970ddccfea59e0ae19716cfa01279cf3ef3e813a8378540cb6e2a686d06d8950
home-path:sha256:9dba1391e63a906809c6f38df9b5863f692a9536c42fc55f31d23a0f1167a421
home-path:sha256:47cf13f57cb6a345d5f807c35c70dbd6a70f3489d0e3a6a44495088f554e7ec7
home-path:sha256:b80788327b233a366edcb97852acf955f1a1ac3a51e589410d7acffc6af22752
home-path:sha256:69eae45b9cac27771179ba78658f4ac025f9345f18e125d347a3b8998ee9317f
home-path:sha256:54e47897c1716b98474a96d85e538a0ef6da1514d73a722ec67ae88987eaf983
home-path:sha256:905a94841dcfbac81b15de146e88c38913c015d535d2066a29e09f1a17ef8f14
home-path:sha256:0759abc028030f695e822a38a3f821e606b3c996ef48d18e73cd5a1d9d617c1f
home-path:sha256:695df54261b24a3e4e087542cd58a6cfa86e3d94e7d518a52ed466c2c10e1767
home-path:sha256:a4a7b061a35f23d2735368d76fe55b7fb0becdb7b4898cb157e5656551bbb53a
home-path:sha256:510ad174544d41d8f5903576ffd6127a219995138e23ad8552625b274c7c2bd9
home-path:sha256:f2c0503e4242efdb2aacdb43e49f4f34d1ffe6f9ff38b7eb44bc8d1306a67b7b
home-path:sha256:d95e270bf82a9756b382f8f02ffb20e37d25021cae4cc762c6eaa63a74593ac4
home-path:sha256:0313d2fae82f5db6d8aefcff53fa8284434e507776a26008c274a2ae6ecdd747
home-path:sha256:cfb44ca14e4fab469ab8a96f65044f339f23d4df3c4794c1f5f026d599da52df
home-path:sha256:c02ab089524ba574f1b027c2dad26f0853df0ce20b950a09db64822d846d838e
home-path:sha256:b08a9a15f946401221ef4388b6ae5f938a2ccaa6c96e0601f4aef0833e7131e9
home-path:sha256:c090ae5e2862daa1c97c7feeaa5549f5bc92c55fc91c6eb9f28fb1e1749a7e11
home-path:sha256:38b46b77c481990e36f17db9e50099791339688e2014933aef908b14fb7e6535
home-path:sha256:c06cfd0296ac464a909ece3aefb503a555032bec5d89d405d4d082fcc180ec14
home-path:sha256:194ff16e5918d2acbd5dd8aedc76efa9628e23f3f1416e981ab9c22c788ef208
home-path:sha256:00c3a9309088f7397fb4296000af3417e1056131294113759e5cdd87bd09f492
home-path:sha256:c49ec369320e3c5041d07e8791bb77408fe34fb912d8b05a303181d3c1426ce8
home-path:sha256:2eff499844500cc41e267801634aa347b32b88d3324eab0862a76d6531792d3a
home-path:sha256:e2534c700a1362c1a0daf4fcdfd4f6aa1ecd54c33af32508223bd4f12193f4ad
home-path:sha256:d4830eb1ec4ebe1826b782b033350087a7fbdaabeccf8d9c496a86620939e7fc
home-path:sha256:cc143fa53da714698f13973bc2c25993d8138188214e2c87e8a233fb57c0ff29
home-path:sha256:01941c983580b23a88307c19de23cd558020cdf759145b3626443d1e0bd1cc16
home-path:sha256:e190b5afaa0210a4ffdd1007828b0ba87be3e3cb52effae8d5dce6a7ece26fb8
home-path:sha256:cd042c4f8adbd61a6f0c6cb6b2e5220e2ede9f1a5dd608d4fa10d2ca77e2e387
home-path:sha256:6eed87b0bc2abf706ab1871f96a54e883081187eea5cca156e7f9d93131bb89e
home-path:sha256:99dc02bdb16268a965630a8e3ca4c31b3da89181e7d804d0a6eac8b03093a4e8
home-path:sha256:ca75782a2caf22535a75054b1d62b5b0a70053141c0a46e262fc1b12c6df18c7
home-path:sha256:3e1fa54e03be5233c777e5b952e328935442e743641036884eb0a05b409b9971
home-path:sha256:f063b587aab4a4d13dbef213b102d87a2c88022925cb2822d7a50c3266bc6b06
home-path:sha256:9557e3cc3d61b94fad565957b12ebcaa2a2ce88a967688523369b0746b053e4b
home-path:sha256:49786ad734d698a2e392688b396820b70405bf8fc4fb5f1ea6dafd153fabeb9a
home-path:sha256:63190bedc694bf05a1d4f1790ae63a458e3430d258cbf1abefcc5706d4d63d90
home-path:sha256:c10d91c286c95da9d044a7fdc9c35968a5dae93ef78da3ac737ecf3e9fa8fb46
home-path:sha256:8ea77674d6cd265974e585550f8f5cdcaec239b1127b13d5f08b8791eff53023
home-path:sha256:13237d7bd606413832f5ecc9ef132b43c5e308916a1dd05b02529937d40036c0
home-path:sha256:4008f99dbd49f89ee847597a44b5d9940f8f7bb191d21524babd3a815babe1dd
home-path:sha256:1e946599197baa492321d46c9ee35a57d548fcc02951805c3234ec8b0e14ccaf
home-path:sha256:306d24fc3cc1d4018204c12f36e6376280568fff282ef4a7b0d381d535e10ee4
home-path:sha256:8ccb73276a860027baa50a81bdae84706f094d2cd1640f3ae95bb9631fc72e30
home-path:sha256:3509ad622cc08d9a49dce0595db40d9a09ffe0228a750781e2d15b418a08369a
home-path:sha256:146eb1dae2f234b1cae02add58ce2bbfb5955aa28f67d47cc4b833deadbc9f7a
home-path:sha256:8c9c66773c122629056c41342f99b17ae52db0f67e24828da1170ee67948dd6b
home-path:sha256:9b66ac702d00f6f754b33e46fff7aa8c0c445acf5e652d2359da79ecd4c85a32
home-path:sha256:e5a2fede1a759bd96b595fd7661eb9896181897b465d8920bea50ecc2524c958
home-path:sha256:a4185685fcd409270b59f34c82f61ae81e8f1ad928f489a143b44c0bcc27baae
home-path:sha256:988955ba589565299c905bfa9220a24b29ba7640a724e4feac6c61850d6d7ead
home-path:sha256:415a02f932b18e1513a9ab156ba712121a7b764b40c65697072079e858446911
home-path:sha256:c80f9863c0bc99e6f1dff1de7ee970777c8958e9704b5eb6754ad5e9249427d2
home-path:sha256:07c45481b9329af4c7207a4f736ec86cbb66a6a3b6c1b4c14024fedd7fe778b7
home-path:sha256:ab7de0258c0bdede66f038576a9118aaeb2ee3db3836c39aa1aca30a1f37b82d
home-path:sha256:3eac5491ac7a4709e140f0ab20396204f16d32ab5f31aa37e33a569bd3667e05
home-path:sha256:e4fa29969e5f157cd3176f421c7a2dfd120c2c46e417419d5130c8bd96a102ba
home-path:sha256:5c646f375355070bf79987e114a93d2110dff6b0e62c6250e63dfafc3dc8e902
home-path:sha256:68aa0e9071409174a64c71488a77e7be55268570f0c0a430a5cd96703f8a9890
home-path:sha256:25b9c320a276055dab64480170f59d08bf7b00998878f970f014940b8171a189
home-path:sha256:41e35f4ac95fc1f12540fc2c88f3c04f0f3d46c7381d07b6ef5e8557b5b349e4
home-path:sha256:d6594e67f472f622eafdc220e64b19d7bc0bcb2d6c543f152eff0aadbe6e9194
home-path:sha256:272925283d4aaeda31b99888369e70b110ab2ca93280f72848442b6e4d0deb8c
home-path:sha256:e731e06d6581a7c0c72a5cec5a87f38e6b88901082a145194bcb3e3d37c1e3cd
home-path:sha256:6fc8794908f413f1af7c1d55ee4ec62d34c742b490770a7d17e91c6d18ef467f
home-path:sha256:2e32d6bb02c5ea322c4e25720150ed0c086d9fabb1f4aa1905e7db187320095a
home-path:sha256:c893dc5be2d80f86fd2835ad27ebf79b46a5192caee57e6629e7017d52603331
home-path:sha256:1f6d11b9492040f7f05536acb88fff5af9e3d5117c4b476637e8a15e34bffb21
home-path:sha256:19e0da6c591c1504de3858c5f42557dcf34cc2fbdc28615b248e05eccaae18ba
home-path:sha256:dff01cdb2f50acf965641c53f0fee1d26e05b908cf4f3379f63d5c21778336d6
home-path:sha256:a46145ad2606cf65f859e0303dd0f63aa9d6dd131b4307e7448aa23c6b5d184d
home-path:sha256:f69a62d84d1d06155ddfdafc98c47c5553cf7449df4cc121c2cc7da1ba5da996
home-path:sha256:2a006368b671c92a324450e93beaf797f1c7ab719f0e50f1802c2c922a6ea15f
home-path:sha256:49ca4c61981e11dfc4c557f97030615542d09101f5512a7339364edb136a1b5b
home-path:sha256:8e72f44b1a268e5aa170e6570f7adcbeac09518489726e57d45512e5fd32010a
home-path:sha256:335f38091f85ff1799169c2fc537f97ae91c5be14bafa7495b2c917a6b89e149
home-path:sha256:1b2549b9b188f7f5bd22542f0aebc3bcf3cfbcbd21cf062ddb8adf4bb96b643e
home-path:sha256:1e8d41fa66102148a28b42e6c84b7a33dfed25eb6763ccbc1010647572265035
home-path:sha256:e8a3d93baf96e0c5a4586fc9502645d2891aeb93b847981935a94773e3520207
home-path:sha256:17d320eb9bffe0a56be0b8b5b72bd8257c3b822991905deca9e0db2e4b9e9840
home-path:sha256:65dab149f76c2652958867c0d7019cb9dd494787a91d0d0b81bcf78fb2f5f248
home-path:sha256:80a39c2029bb79d62d0a8e2b78bdba9d3199b950aa939784957b9f4177c95636
home-path:sha256:54d40a45a442bc816aa5de6d1a6bc56ca010dcd1c28dd4a5da5eb423b3566794
home-path:sha256:413b9c962cf75429f3b9862e25c9bdc3772901c1e4c45b5abd429c88019dfdaf
home-path:sha256:ce796b45536cc5d92b0ec98d64f23e7af0c615a8cfe003e4417488d85f35427c
home-path:sha256:96ddac7d61a761d875905da42a72df5c38471d7b36628d790f7a2c4977df99ad
home-path:sha256:dc40ecc8ee5be19ebb814949017fc1ff2fd435f07dd97466b141c9f06839698a
home-path:sha256:19f7495a8c2448559298d02bdffc21b2ab6f3337b8c6fb06581c7f79d77a6334
home-path:sha256:ec74e52abfd46e92900e03c414407dd6b61182e62b4e4b918a5ffb2dbc8e3509
home-path:sha256:36dd144d62b61adedcc3c206eb0983d0988205883c272d9fa2e82830be70c0ee
home-path:sha256:30a2ebc4c42e81b7caa6b2e5be08c20e560d33e1fe07c2076306417504895a3b
home-path:sha256:d139a7c939a08ebe28faef1bb5a3f3737b9de0c5a9456113b75d7fe0c556f939
home-path:sha256:e1f5d6456d274d88d16251edd09ec40a3c2474ce55a1919e80999090821d8f2d
home-path:sha256:10f4495cd2a902273a0ced6c93fbb6120c891e60ff1d1ee00d74e99644bae70c
home-path:sha256:11a9b6b96a76d2107d789889ad7951f5a04e8e6a9ef556b12744a8252753d3c4
home-path:sha256:5e87cfc3424eb2c58317b8a93680e91bd0f666a68cf0d1898763b5f7e114c588
home-path:sha256:a5d71f57c59397142c1e2ce8b3054687d4bf973ffcfbbfd56e93075b44e8d374
home-path:sha256:95d204e9e47252010ff2193996764b5c65d2feecaee8bc63b670b74753927620
home-path:sha256:a1ae4d5d5d6c4ab2b2067c5ff76142ce255c6900a662c00a54054711e164f7ea
home-path:sha256:336be635bfe7fc06a7c479aeb50a440b676c455dac102efb12e6f5e31b0f94a9
home-path:sha256:9f4df23f68a2d8db1b61545d92123bddc4b8d250ff9b7d17e52fbe744295574a
home-path:sha256:23d4d2a37b6df8257817b097233565375774c87cc1727ea9abb78e5b48ecaddc
home-path:sha256:3516d231448d57ff8b4acf0cf1fcbc13422984effa07dde072d4afe5cfa1c358
home-path:sha256:3999e16cb95b4e18e9cbfbb8252dbfaed8d27be67af0bc67327c7c287689277d
home-path:sha256:fe9ef781d8c933a11886d517b85f25846f366e851fb34c9b8830b2944ac1f554
home-path:sha256:75775b7f26a6a341182f6015d39eab09b56384128a8d4a74e2c0486fd8e389f1
home-path:sha256:61736865a136e63cb51356ae51b97660b35c33e79390f1ff0f0accae20108cea
home-path:sha256:b7ec9b93bf00f4aa9bf1c8535f0195c3ef57a21d28e1ee0f50e93ec138b9a85b
home-path:sha256:9f0651c9ae66ada45096e2ec6853347d0d4e961a8fa63b035803e54f014dcf94
home-path:sha256:3047b48f69badcd217be9efec8464cf26dd505d4a484620f5c1c3fb1f48d631f
home-path:sha256:3115f5921483b8e289eafa5c7d3301e37bed76e65ca883d8d4aede9398f48bf4
home-path:sha256:8a97b8681651586638d197544eb65359c38019b943aa1b79caf4ce424ca9aae9
home-path:sha256:d222004202c3c2d6dc53f619b6cf56799284c81efcf22d5029c03728784a3f12
home-path:sha256:376326a040bc78db5d73bcc38947e5aed724e59d60076e3a8cfd215f1abe67ea
home-path:sha256:0380c7c84e5b7b3ab66a042eec1459cd81a0110d330e3373afec168493982051
home-path:sha256:d963b57a2c6d0d9bcd64a2f39db8acde04ae35e0330523b5fd85200645775f4e
home-path:sha256:ee6bdc6d2354c3d804751c30974e3392687eba531d9690be0c0400f774994293
home-path:sha256:001e1b3a5b0585aa0bd91701982f3f91eb7754dd2148580a9cc962334ce28f1b
home-path:sha256:367cec4c893d83fb6c44406728b4c51aac6ef8ffc422842016d9ed7d7d60cdde
home-path:sha256:bab9fae320315f28a5052ff906f6dfe1f6b3e4ed5f2e0b2be78c831265ddcc24
home-path:sha256:a0ddcd5f0573257d67849bf215b8fb68ba727cec4ff47af6ec78b1550b83bd54
home-path:sha256:ffa07ceaf7388d2bab9fea87edbd1910dd3d7f04bbb9ade9d7c066ef646d369b
home-path:sha256:b71cb20a1f6f2e0962d200a75f052d66bfd5bdcca06522cfb31fd59e7cfb0508
home-path:sha256:5eeece6c9936c4e03e36e49fa29964a6465ece9197108a4078313a44f1ba2063
home-path:sha256:e75efaaa3c18dc33c18a3833d959bd1d97d0875873c2426ebc3e2ff51dc3b265
home-path:sha256:ceb86ff6e85d13967400ddc243bc94a58d02045e1c904b6944daa49981dd28f5
home-path:sha256:25b617fabef065a019ac1275ab11444e5f2f083a9d917142a7276ec66e007a2a
home-path:sha256:32798ed8c30b301604adb75e27067379533cb3c399dafbaf03789056eb421f97
home-path:sha256:7c62ac4669b1c8a6f375671e3323509a4cfa86756e31d12abb574c4398b8c744
home-path:sha256:326d1853e4f12e8c5977065dab20f6752efcce6e6c37707046ccc107e01e1d09
home-path:sha256:917bd8e90fd6b3d6a37db2118376695d79092691e3b0cf562c2c4865af479445
home-path:sha256:97e8dffebd201727458b2a28803357113f34070c20a85d1235b574f721ce0992
home-path:sha256:c3be573217f8046315480f539378dfb1dc6fa6fb846c0ea6f264c4d3802c4dd3
home-path:sha256:d6d6f71120d9f89ceaf6fdc667e2235f4acf3144eccf62f78878ecf6edc43bb1
home-path:sha256:6203379b1201fa078a4857067e9097ff21c24d567fdc692c3b4ae740a7244b46
home-path:sha256:0490c1cb7ca4fa6cb420e92b7006d16917df1d548767ecd4650a8d7825b88fe1
home-path:sha256:b7e190c575ba65264fea3411dcdb08bbb4bc7e3b44212b2cfaf89662b5ba2ba3
home-path:sha256:0f4af676ae01e8445159f961069c63cce71de8072e2587dd5b1af56cc94537d4
home-path:sha256:3b1765e26d426cf564997bc7d8d43c489e879a7898523fb1a83425d1024e7b8c
home-path:sha256:3dd22cdf8858c30491ed7ec3b1fc26742ff210f34234789ffbc69757c30385a0
home-path:sha256:d1bc7d1ccfcdfced7b7fbde119aa9cf2c486d07aa5dcbcbaabdce0e830efd907
home-path:sha256:bbe32147bda5087c6a0dec16af4ba9441a3b7ee12d8c03c42aa9726e3dd8a906
home-path:sha256:96ac6a50103ee0b5fa5d6e1f18787b67e3c43b40e84234b3fe70da0e613b0e83
home-path:sha256:9dbd617fe67d79cb85810a2910dbd1a022dd19aca131f8e63c561858b025991b
home-path:sha256:cae0bc2c328cf8cf41b45815869f052683974bddee77267041ed3c0ff69646a6
home-path:sha256:0196dbb2b12db97e4ffa0987aad4a781d35b7ce4c37aebc72172de1b125fac18
home-path:sha256:ae910d5e3bbf0e30c95ecca2053ce276ad26cc0ec7bacde2d6ef0e557d4f7c99
home-path:sha256:c28913bd710995acee26215ebb62a65a75ca0b5c1be1f001ff5f8e65cfdd48c2
home-path:sha256:887a6fdfb70e0a80e764208cbd95f6b7c3cbd81a0d9a4acbb5a349d62a87178a
home-path:sha256:0591092dfa9b68f2a9476cc8fe33c58ce4037bfe1dedce0d50d371cb94f724b5
home-path:sha256:e247ab70ff262c0f7cf4276e00a3ed19fd0a2ee435abce0d5631043788a9c42b
home-path:sha256:3e5c09f4588390638fb21a6e89cae2b9f95edf0066a377e18604c751eb060019
home-path:sha256:d7a6d5ebe4c751c75eb3250a1c998eefffeece2f3a8c8ec20c94a88fc5f3612d
home-path:sha256:57b5b5cc07e350b4afe5e33cd3ef9f81eb8bb4c9c8f4c325f33a0cd74abbe18f
home-path:sha256:6a1417086356a330906749a829b82a8908e79a4e213180c8a208a71d39908dc7
home-path:sha256:56387a29f26bff15456adf3790108214a95afc75cd1d01e2f8bc8d1d3613a61d
home-path:sha256:ebe5af7dd71acc173d32a09304b1878b7d64fad8bd6f6e681107cb672bcfd689
home-path:sha256:766f6b97160d366ab3ef627eab03a8f9ab79369bcfb8fd87fbc3e56e46368e75
home-path:sha256:7df1e9888978dbb89fae2c7d978856a6a494c066e872e9b7cde4afbe1f4b20a4
home-path:sha256:f7cd1e5a0720d130f386f6fadc0299678e05591f97e5a80b2c787eb20b70ac1f
home-path:sha256:f07c449e49b869b5e596811895a208061297023162e454c11adb7b060913d38f
home-path:sha256:1bbe08f391db5defcd951b5b521ae12ac86c671a1ab8cbcd5c56fa61ea0d96ac
home-path:sha256:4a0f8748328d331060cb01b8de3d3c6824313822ba5a24080dda7d94c2586a42
home-path:sha256:64e4abcf11322f59e69324a449241ad52611e18780a1369152adbb2e4cfb0edf
home-path:sha256:3e30b682f98e71121c91798bda28707e05c7b2ad4135f4dfb6b874d2d8991b77
home-path:sha256:9f695a686942a002a3ef43fc528711717420b807e2b81d465ea44d4e95f56f6e
home-path:sha256:1a514552d8e95c42e7b216ffb284c40874ab2dceae10e56dce3efedb609852e2
home-path:sha256:9258db551776a3b650cc3c1795987d77591720b859f51bcc02836d4f26c3027d
home-path:sha256:a587e4564fbe26dc2549cdaf5f7ad3f11b2f4da9f5bf3aa583f979154bb6a4e8
home-path:sha256:4eaf966cd8c8b7352815a637832ab8971bcbdf0dceef8115a3c7fa04d390b689
home-path:sha256:dae4e02a6a8de87b1a835d6550927e1570f9455cb61618e922866c70a6916482
home-path:sha256:000b650d860a44781208c103f15a42719b6985d44cb481d2ae0d23496c9bf369
home-path:sha256:625ef41169f173d342388f94b12ec503591fd5948c4925a5738094f43ff12c2b
home-path:sha256:b9e792261fd6456024107281d88f8ed3a4d5fabe373f6ab49bf7ff10feb85686
home-path:sha256:e8a92ba9115bade105d8c27952325429f79838c3409ac7b4bffee54436e018a4
home-path:sha256:d555dc78ada2360356fc5266dff718567bb68fd83f4d6f386642c3ab88042e36
home-path:sha256:10dcc205a5f2afb18c218cd92a4a167ac2fd3d1fd0ba4a28d81c32ce35f97589
home-path:sha256:ce3675d776ae3113bbc491d13b0ea8f18d56334db76494d0279c1795a5206c29
home-path:sha256:2fe0a2dca8d0f93c8fcee0c61796b15dc25361b9518b86f5cb56c4f8a264f35b
home-path:sha256:571c55bc94bbd210a8b7e572f2c0522c70d73ecddc5fdc432f28e35d95faa76d
home-path:sha256:c8ab2fee8b6686a2d433908453bb249edf046905a9aabd57dc92f636291c408c
home-path:sha256:bf66a9b6c7a6a2da81090c1b14c5fb7f8761e4affbc670b9077d50a22c795308
home-path:sha256:4aae70c0e538031dc34dbed85723cb397888d85b3f3da15a8811722bbc2fc6f9
home-path:sha256:0fae82508f41d4401163d64648fe1e9bc46793556b40eea4ce4050203e498556
home-path:sha256:89d9aad4e10e63c398483881946c3feb57ba16a40abf9a270c8b934017eb2bbc
home-path:sha256:6e568070e69ce7934194a867561064fbbfab33062969da499ecd300f80facf4b
home-path:sha256:a4dfe8cf982656b03d2b87e680928df4ec92189188ec9ef9f375163d95b644dd
home-path:sha256:2f64142e07cdee184536169e5c03965419400735b0d8b8661231af91d6afa477
home-path:sha256:9431df6f8bc6ffd2fb18de6ec30db18f15b666c2270bb4eef904f012d567982d
home-path:sha256:c1170e452ec7019fe9c987680e7d6de4a9bff7817a2c3e49f587d2d8a400662e
home-path:sha256:06c51a0fc63ac7caa652d16a079d644a9856470b12dfbeb6aaa98983f6be37de
home-path:sha256:599aa6c49a22ab8c7aae3f1947ce7f08ce5c09b89a1c341d600813de1adc3d4e
home-path:sha256:37bdc262658df4f8802cfa8beeb5fe80cfd88a3907877c41dd81bc5ad6e4b661
home-path:sha256:9c2dce8da8a08482fd0061a951d1a2f806973dafe7ec3a695c54c04da959c63b
home-path:sha256:5f8542b8a127289e5934c04a4cd770d5799ad3b8a6b15f9b438455b2638ccd58
home-path:sha256:7a157fbd9939048ca3d43a609c4d4b15c985b39bfb624e61ee0e4bc92289def8
home-path:sha256:f7f965af28d9659c2f7323574a314b96fcd732578c7e187ff781b5e7ef70f4af
home-path:sha256:a81bd432d082345fd0939e1605ecef36c6d14b332ad5bfb9ba2adcf6fa952cdb
home-path:sha256:e7487b7c034ac07fa136f134cba9e1294e0fead2f185897fe063cbfeb29000dd
home-path:sha256:fd74fd2d5d14258415edd369705b172e5bbc59ad4a73932014432a09fe8c0af2
home-path:sha256:af3fd3a3e92821d0c82cff8bb354e21a28ed995baf784663ca829d8744eded10
home-path:sha256:17835161420765e291817ba3c9d3ffff5eeddb175453504856f0b9bdfe190d6d
home-path:sha256:e24d348142bd0de26ac80094be966df66e4f2329bb70ccd062ddc35971d59c70
home-path:sha256:639cce1365807bad23367440dbc21f50a94941268066f299fa1f0d00df10b10c
home-path:sha256:746b7b367b1507c613e0da825ccab62fccad34073f1bc5b7f0f05666d7e60b14
home-path:sha256:ce75f1c25572067f49915bfca6a44033d83197d2e004743067e86fb8c2b7d816
home-path:sha256:d3e374a9b0dc7c6c378cc3e37a5f6ae06a0f36855239dd5cc7703c50e385449c
home-path:sha256:525547cb4d5019215fccf58c733a393d0cbb27dea030e3cc59dd7d59ac7394d0
home-path:sha256:88d62b2a858f95465a478bf878386f1e73309241780a029ee9caecf5b21373b1
home-path:sha256:39029d095244cbb7060af07b7bff3ed5879a0f3a414dad7e7457aa7f7403c48e
home-path:sha256:95abb20c4b4a84b6f05b52a83bb06c43bba16660534f4b5875ce764a73c5f77b
home-path:sha256:af007697ab3fceff907c607a987247438f46029975ac4c3bc938722c3a6de771
home-path:sha256:6a0a497df545869007a7717f0531ef350c3ff70e1c12921f42ade5e07487f953
home-path:sha256:db5459367f10fac24f44927799b69d53ff72d58a9b8f672d689f391473c6e7d5
home-path:sha256:c81359ce5941b69b6c3f51f6102f06d9b476c912ca6d2a518e426f7a498c93bd
home-path:sha256:f5bed8949aed13016e3b325db2f140c2349928fded68faebec563732ef7a1c5c
home-path:sha256:15f2b44462e449f7c675e707ed58bd9cfefc8556c89228cb074ce90ec952f65f
home-path:sha256:4254aaf5821b84fe0ed905cd3767c7c7f0c4a6a555511c202b7c389ae1b9ab94
home-path:sha256:cecf2cd71bafeb35641eedfbf4a236caef5fe4761e46ad47c917412f0f2f958b
home-path:sha256:4c774be97575df7cde32648da071bf50ad673ed32046fe9ca6d11768b156a292
home-path:sha256:d789d2ba2ba37690cbb474a6ea4396272be0a021f9d4169d113b797e387d58c9
home-path:sha256:6ff2ede6f0220ec30ac257f2ca4178de0a5e4409aedfa4fd6a618554ab86b551
home-path:sha256:6630f8a73f78b6a40d6bff4c83326e8956306433aeab802220c7435480bd9ec0
home-path:sha256:c3b2b2ab6053a5be5e5e18829d4b98b355c93d172836a3bcc3251fd4f9bd2de5
home-path:sha256:ff469a7fbf0f0604b836a379ef6a09bc8460ffe77b7fa07a1b9f0ff173a8019b
home-path:sha256:5aff9f1a4f12e925f2531eb25f6b9cce8b5004de17be5891b1afa51ca53b416c
home-path:sha256:901b36078cd23b0ac07e23951d5a34aac7f9415403562cda30261d18706a7a3c
home-path:sha256:77f544a06ba32177768710558bf17e7c1b8ac4577ebfb9213c6fb88c6cdfe803
home-path:sha256:dcc2ed4ddad4e8e031142c59dab97aa5028602a81037a1fc9aaa549f14c9e2fd
home-path:sha256:69fb10a0c77da835c35f2f53e17e53f71a9006aef581ff830091095ebccce483
home-path:sha256:8d2fd6f3b5d640fb2671e3e0eb27fad842c898305f2c4009f448b41477427924
home-path:sha256:e070e579cd5ca67c69669aa0c8d3cc5bc5c0c1f1f6376aeeba587e429fafcdd5
home-path:sha256:0b742b17be6288d5cc8695bbcc998c401b6984b6c2b0a7621cc3403274393418
home-path:sha256:7eae3533ea12272738a42ec38b16ee641a9ac3171e132a230fa74b847dfe443b
home-path:sha256:7267852e99a368be2239fad3e4be09130a624c07d4f1d7e7539d9c54f5df558b
home-path:sha256:48b60d014020382dcbf25a8fb2dbb2195435c452df6d907ec570dbf50101a3f7
home-path:sha256:04733a3620a5a5392bf5adcf1687e9dcdb5e72beef5e90e64dc36403ad416d34
home-path:sha256:cf9db0744a2278fb3119fc6122148bb7ce5a6f8cfed9e8836f6bd65cdbfc7a24
home-path:sha256:79500db51d23afdc4cb5aba04440b33942473df7d3fbfb66ed234517f1dbae7b
home-path:sha256:80879349f9c40246ec269f2c156455408f5bc6d4d838582e4704ec4921f0d425
home-path:sha256:426838766ee7ca9a7b2d1895024936dcee65d41a72803f6961fab54c68f0ffcb
home-path:sha256:3e0f67ea9ab691a52a96c916900b492899ef63c1f15db2bc45d44e51265a8e6f
home-path:sha256:4f70976ac9be4b85c64d1093d19458e99c04e10eb58cd5ccf07948fc893167ad
home-path:sha256:1dbf919f1d1f0b81d7f419b86c834477937fa0d210d7a1ffad0bec13c2c7df4b
home-path:sha256:fdbe7d7d1aab4461bf3b7637db93d231aed47b43c00c760388d3232a5e9de176
home-path:sha256:bfbe63a1637c1e4ed3737d745f6101fd0784b4c0a82c3af88ad1e0830c232bf4
home-path:sha256:8229d916effff48ebf7053a70bbbb2d3365ef2803aaed9e2f2546c2bdb49e96c
home-path:sha256:f4356b6bc455efbcb3dd1501a28178e3e4b4348dfb0f037b99d5917a37b872ba
home-path:sha256:5a4b642c07b03eaa05bba2ada2bfc07b36d6a174b53e667d39e78e2939f7ed62
home-path:sha256:43edf9a0cd84392729b81bb941c221c5b9b718b8e8bf2afeb66c9f84f3bf0534
home-path:sha256:f97db59e9bacd224c932749e2ed878ded6c8eb55705cc72484ab88654dc8834a
home-path:sha256:9e4f0d2402f978b745cfa8c96ab55da55183ca20f04233f5f5c17b447291664b
home-path:sha256:081819190c47a10a98dd003c8eeb9680dab250c57234b191479f79a7b71bd0be
home-path:sha256:295c9faaac3ff46c894bc6ab993613ab488155ce38d4b8643964519adbab9379
home-path:sha256:c615a7adc6a3538bece124575eaad6e2e1a32d33aafc8ceb7bad5f3b67b8d9e1
home-path:sha256:a1f2cf46804aa29d2fde912156d229e2724feff3751451e5cef3954ac6dd4042
home-path:sha256:36cad2ee088bc9d6987b2cd4a602f9140193b5bf00a00bce49a02ddd80f26c83
home-path:sha256:7fbafed130bb6769fd80322dcd63e23e875a04e2023c4ef67ea9041035d72b8c
home-path:sha256:fd6ccb8fe2bc666666a42d0a5a277ee2d19be0234ab5e9fba214b2a2df1ee9ae
home-path:sha256:161878cb920f0ba2a22bbc511ee802dfc6ff4a62cc360ee52acc0464cce5a435
home-path:sha256:873a0aa727101a9b525036bf6eac0fafc743564b33b692543a90847f4a7587ae
home-path:sha256:0dc41f4e2a69edf1521bd5d4149dd077f9e09f47a2ce6f1b78a08b15db313562
home-path:sha256:6d9e36a6bfc25a5635e30f0aa6773f001d2d4d5fa0ef1fbbe685d9c307929ae5
home-path:sha256:da7a45ad6c939fae63500cf6c6285aa36971b21323988ab884936ad1c26c55c4
home-path:sha256:58aa089645e7ff196b24d064f412eae336b3a64d29a4b717c93636b24bc28b64
home-path:sha256:fee82ad2edb0de6d42802113aa8efc015fc2314246a16d1fd10f1856222602f0
home-path:sha256:eec44095f7934c86866a6bc9ee136aa946b7c5a2a66505b66514c7548dc003ec
home-path:sha256:1cfea01bcef411098c454e1aa39f5e110fa622d3c4e423c4c365018eab95c02e
home-path:sha256:8ff735e81fa2353ed76ab51bc966add3c0195acd8a86ab30b8502d9147dd4dd9
home-path:sha256:c31f7ae9a5adc93eb203101ea6cd90953789ea63f563adb9f521c6c3cf183337
home-path:sha256:d9a89a49e33a8451882005a651ae878c8bf5eece077063ec988d15d27f46c5de
home-path:sha256:9761a6a4cbbba3b906de7d388b3f957c447ab89779be4e6d8c6ec01838f7facf
home-path:sha256:4016307fa00a0f131836f866b5feab439aa76dfef1618abfa6a2443527bd2c52
home-path:sha256:45bf0e4edce2532977e67654268f33ccd98bcb0a7ddec4bc2e2744cc61558c8c
home-path:sha256:1012b4beb2797da9803b5ea9c41d5c33746cbbfe1e94069974b298cf88f3d4b0
home-path:sha256:df6f6be96c01c4c99214575b17ddd963ae0a08e722dbeab9d171d49e73148000
home-path:sha256:c1e1fcc85980703bf986add0f9c2f9d0cbcb2c525bbf8066ef8a107945940c62
home-path:sha256:56a53357a3660d680d46fbc2eadaef0e31b03bb6b5fffe5b6e315d3ea7ac1ef1
home-path:sha256:8063ba0cc52e42c061f887206ac222c6d1d29a200dacd90f1c1561125fb94f1f
home-path:sha256:98bdb1ac5eb54502809391f4ba830718dcdd7dac8ed1979a944baf6f6a18b62c
home-path:sha256:1e6f7c3489825a030e72219564c11dd3f42cf8e3ef1a444f4b102ec00b8f50df
home-path:sha256:1775496bdc15dda4575fddbe68c2485b55c1f912740e1dcaafd0fb9b6bcc43eb
home-path:sha256:bf1cb3b486063615844caea04e917f9eab95dc61d537d1f37813bcd67d0756a7
home-path:sha256:ee4835fc467d3c8128d5273db159d4344678bffac9cbfce80da6a0fc76e039cc
home-path:sha256:ff43a6fee1f90ff0e41d3586a1917d0fed2be746f612a688a137725cab73090a
home-path:sha256:fe778738f3e6064a3e75cc337cf48d20c93f68460ecb7b72c151059c2097e1fa
home-path:sha256:38c35d240c10a360bfa7e62b9c442a258f04a45c73f225dd79648b2d409bb701
home-path:sha256:3e18a3d455243fdec67156ee6afe56def2af68d7e26d5727140b142c5175a6b6
home-path:sha256:24f6d143ad4b08cb4a44c3fc594779c3dc07a30707d27a2298c9b68744ca4c1e
home-path:sha256:4ee8cc395bc8a139d25d7a70db89bf82ece4e92f5f742374a9adaa7ae407c0e9
home-path:sha256:bc361e34df50c4cab31b4da18cd47e2d0ded15525baf6ac430b131e3029e2bc2
home-path:sha256:97aa829bc2547e3b6d29b32bf1864a1a624d0f8e4f4060f36a3e885629ed137c
home-path:sha256:d5e8572ead06b7eb2f7126130ee7d4ed6c989fd39f0f1ed5c48513f47282e46d
home-path:sha256:2877d96709d8f926f0d4aec3a957c65cc86f152716218e03416057ea1f04d9b6
home-path:sha256:73b0f1b7119c1829abdcecc7164542d9908573c2df46e1de64187b4b2ba5ee95
home-path:sha256:1ece9ff546654093588fd1caaf59428a26e7c9f037847db369efe78caf63780f
home-path:sha256:06f91b8f26d7284d6f963092fcfd7e551677f603167ff0054a9a5bf8edb849e0
home-path:sha256:b910a117953c0278df4476c24b240af5a32041c3fd85bcf5fc60a3d854475f68
home-path:sha256:7627d2c48f122b78b7164c9d0585e8696dd6def15b11ac8fa595666f93af9eb9
home-path:sha256:ef24fc6e3a8e55fa15a80b0a1e6e6556d752911a130cf070e9f8b64ba40c127a
home-path:sha256:6c1286e0ec4e3e9cb197d84aac8914e7984623fce7171c2e31c1a3097f4e579a
home-path:sha256:ffa57a00900c42f42ecc456460fb7b8a0039bcd481056739cb07d096d0279e4a
home-path:sha256:ab4c0526c2d18f5066398a4bb5f35e5d3941abfc9019b52a49eeb657168c4225
home-path:sha256:f912b9d038b50808aadc5fe1d6fd0695d95e366744db95130dbafbf4a3753cad
home-path:sha256:38d2e034b76a7649ab5745d1b31a16c1348aa9c9c9ff041d88dc5e5d09764ff1
home-path:sha256:f3d26e9673db6811bd2e50ac8d6c8b264a0bd124c62a961c872ef93285775c9f
home-path:sha256:d61991e55dd22d9f5a6f50e56ca8339b3cd90a497d2c3d481f90375b2280a7a6
home-path:sha256:86f1c8ce6e1711b346b176d44539608ce0aeb00205e386eec43dc37812c273aa
home-path:sha256:7a0b3d833488d7099e745154282c5ef89ce43f952717f2803ad7c0914a5b141f
home-path:sha256:5829b9821006d212391562024a57e0f3d7539230badc343f27899d61d3b9e052
home-path:sha256:59c4a07fa931aec9291d984c7b7330a7a34096e04ced2df36ad6c89225f7da01
home-path:sha256:63c77bf9920ef723663405c5ef92c0efaed9cc8704744a979e15cf6dd2a7545b
home-path:sha256:5737fc9a16fff5e399277a5a60b90ab352ae50a5ff3c6c70588c10d331b19546
home-path:sha256:d717fe09ee7f80bc164218dfe6e9f95f47f116a593cc1b4e3b84e2852c51c6db
home-path:sha256:c2f660532d539c558736a0179a7a9cd893781d74e3990c19dc64de54c99067b1
home-path:sha256:13b312692e7f06799d4a6d940e9f8f04038f3ad598d58e8792f93553b0cc0ac3
home-path:sha256:e2666275f76f924e268961f0b8a23f2a3b9018f54778fff20265be53cd563972
home-path:sha256:ab05c80e6b023e556ff96d810c32c340bc187835ae8121a90c3f62a9f0864308
home-path:sha256:725b1d516f531579fef0dd06b0aaa52d8ba4575b9d572919eecf30ac58ed0327
home-path:sha256:333b1a352b5470bf7b1e7997af4a648c7e46b9a88ca95467fe5b6ca8ffd1a450
home-path:sha256:bface3bba7de758932e4d5916cc4e07f38350e9724662ab2689eb91373e20d9f
home-path:sha256:d859d4da6055b69b27d9d318dac3a5a486ce73269395b241182e28e0c1eaad67
home-path:sha256:ef77e832774bd6e7141bce627e4c9f2d1218e9248aa9245720046f72fec9fd02
home-path:sha256:4ef650545346f5dae9703989ff8680274fb7641dbfaf0e9d942dbfa3ce363fa9
home-path:sha256:0eb62147577c2543a1c98654573faf33feea4764b20993530895ceb4609fec96
home-path:sha256:e91fc6ff53874395a8214cdd65023e0f1677509af4320ba433df967633b6607a
home-path:sha256:99f4e1eea87dd8ee100da3ce7baa1640b75d88841020be05df34743922303565
home-path:sha256:1f70e3271eb6034b0e100ec7f3ca7cc0973b08d984cef5464119af528e5200e2
home-path:sha256:c818792c50b9bcf4e593d03e00c4c1a57f7ff661b0a8ed80bc2abd7f0fc74af1
home-path:sha256:22fd42b28b21c6c40e01457f4c4f68fbb51fffafbd5df5e27d554ac5f093fce5
home-path:sha256:10a21a8cecf924d5dcfb2cf9627da76fa3f5b39519efaf6759cbaef8c41de4a2
home-path:sha256:9b6b863c42abfc984d3c2fd89a8f879478fd92e66802f5959e119434f14966e8
home-path:sha256:5ca31cea9698481e6dce50bf2063509fb231067ad4f6e42acabf60757ce0dc75
home-path:sha256:d2882e01bb004e3fce5feceee342e3e583a3b3093e2a21e4b998f4dd16c2ef5f
home-path:sha256:3efb555356113d71dc568106a23cc91a0b1f92532f57c2a97ea074367b0db77e
home-path:sha256:dd68d120930c382ada8280e336512974c76d64197b2b2ab5c62b1b61b4e40892
home-path:sha256:cea7083056546fe630c28f08c8c777e3e9e157182ff8c6117ea9513b9efcc508
home-path:sha256:e3256a454ce3fcd63bdb5cc077b65c15bf5b050fa76b1b9d6f96fa910edaabba
home-path:sha256:89eaf18ca45e23c2b755ca6c88bf1a21cea15455a3bb0cc098725d28bdfdd418
home-path:sha256:d40d49d48ebf3d6a5e2d5983622c3f04c535c73a40c2d784e90a5bf152895e8f
home-path:sha256:a81b6ff45484cbe2b3b8de0ccc922af1594894e18eace65f2b0a2a5aa2b86174
home-path:sha256:29ab31b793830b37ee56a847f3172938bf2648e9cc520ae9b39121a720a7668c
home-path:sha256:72060b9cbf2f3aa43cb38b6f61438236c57a223b5591a3072649e9d2ccf18cf2
home-path:sha256:abb6cc3cfd7698b1b80e0ac15f6297481d6a6d499e2dc8c56f060328b79f3862
home-path:sha256:b6050edc4b7b98ff953e0d601c31245453a56344226a48cc80ef8cd4b09819f0
home-path:sha256:395a0487ef7102ed8d373c61f20f0a344edb83766a5db27263e1363903235474
home-path:sha256:388561e351d4c5c373dfac249561524abde30a9338eca19fec6006d8db476235
home-path:sha256:dab7d3e522c58e5015fdade1a8c74f211c8b93ef647003c89e5a0e85aa307485
home-path:sha256:c01b95d36e78fa35b65a3f18df7d575149a3588a6ff920a25cddbc6f9931a564
home-path:sha256:83960ddbeb28b82b90a8b74e2f38c9c67421cabc7926873510166c5f3bbc588c
home-path:sha256:d1fdabf3acd0205aaaee4ca02c3801a34d716fcd44ea6c3351878e0020874e2a
home-path:sha256:c0c2bfea191efec9d253d06d59c24005ebaa2fbad8bb23262cbbf2cf5f5a6703
home-path:sha256:1a27df742e6dbb6f6e2e539b92be7ee2c8401f512a859483acea35b79fff4c52
home-path:sha256:b71f59b683223a245290da5cd49f9df264bd2c5727a1c1e4594795360aecb5b4
home-path:sha256:45841e92720d54007b4597a7f5054de235f56f4f0d92bf9790dd306e81674f10
home-path:sha256:b194df578e9fb55066b69ee13180852162ad43099b6a6c686e0f6dfb9169305e
home-path:sha256:707f7d6cedfbbf10a7fe7e53b42df99f3f9b33859b06b5cd1ad6b009a8fa651f
home-path:sha256:10f76fed857fe50bf7248e459dce6a0e08e79cb7af3a0c98ccfd7333e3eb5bd4
home-path:sha256:a770916c8bd4ec85c7ddcd6d3b433c2542b4cfff2b3432c9dd804357cd386d18
home-path:sha256:594df82481e10b57f8717575126a0d0ace371453293a3ff100f0b3f735b04d21
home-path:sha256:9549e0e8a59b699a10b24bf5c35a56053e140e8ddfd6503bd50a244579be40dc
home-path:sha256:11178e662607894f3395ffd2dad682338d82599c7bfe2cbd238ac14406dd51a1
home-path:sha256:221046a8de8715e45f4c53d9815e60f2cccebd95557d4dbcb27c1cb32f659678
home-path:sha256:54870cd71ff9a0a315ad1a69cd695c52ad1f83da77eebae350061aead5eaea30
home-path:sha256:99b4bc64fd4d9dcfc2ab2d017d972bbb9db847db5a34e9e5b9fd86a90f33631d
home-path:sha256:e6dcd0783b8bb324339a40cc3c0591aa15e697f50ccbcf68914dbdc15117c9d8
home-path:sha256:293bf5d805ffcb72e540bef4e4f625ce718998a857a01c0a69314cf727d5b5df
home-path:sha256:136a770ca96851266d79e2c9c1e25476897e4fd7b846f2fd7e2b359bc8b8360c
home-path:sha256:d87437739a64d08547b545f7e4c90d1332cd4130fbb2d2fb5510eddc252d89ae
home-path:sha256:67b01bb19ae8683eddfe1f501536ebc7ef717069ff8b594ea504c839b607f0b2
home-path:sha256:a601ad9f5395b38fe92d9066ca23b6ecd9242b47821db14d6d07c3c018c87489
home-path:sha256:66306738129900d9a846df08a76ab6cd0f997b8e61911495b8aa77e47813bfd6
home-path:sha256:c68e3cf0c2362dc643d17f498fbb5fdc02ff1a976a1bd0dcc210a8f36ba001ee
home-path:sha256:419124540661573bf5f9dbe897c46a6435aaa9f4c7e7aa3e76919a5f453b54f9
home-path:sha256:4d0736f7ca1d81c60225d2c5c0ce50f59e3c63fafca8f7146fa0f878c446b4ed
home-path:sha256:e63a38ad9866af7807468ee77a1d7ac7f453f163e9442b6fca819199ce55097e
home-path:sha256:28a8a3c4646f5b6a3c25b13c68335ccf7334fb81f010a6230bf42504fd78e3d9
home-path:sha256:f68d96f84c7faf4efc3e00e8161cd67ce594e36d0d1365e8be9f6ec9ffd7ee14
home-path:sha256:b8dda82e14bfc6ae0925de652c86c724b74793667d8b169f169c6cd7161666a3
home-path:sha256:f2bd532750f895766222c592dfa42fa1ee47b517fa263296154ec5720de632eb
home-path:sha256:f77872ee1142274fefb1b97d3e2119a879fa5ab4d6ea644c467b46a3e75248a0
home-path:sha256:7a966b5956a805ffe9862d1669acd50f7070b3a192c4af06c49669e6196b6606
home-path:sha256:abde361b653273c7201a6a4bae3438886cf80979bf9e7e80710209c22a98c080
home-path:sha256:4cd2acc5115261e59786e8729c330169e4cc04f66854e9a16eb6198d6f2e2afc
home-path:sha256:e10ad9de2f9c51128cbf15ede6495e22f2d4b6d10376ebd45ce7e7a075eaa9a5
home-path:sha256:c57c9490c5a06c5f0ad9c85d1d85dcda3eae3793f7ea2c0c0432a53254b796db
home-path:sha256:1387bb8a11669bbeb4501165069118e80472634628fe10c262317b59d91d2850
home-path:sha256:a706aa8cc030ad2903a82eedc78e0af9e3e611d664e2d5caea1a3b40d40328d1
home-path:sha256:6a4986a37a66d111e27ab5ed9f5c50916359af947e5ed8bb9e1c0096af9bd43c
home-path:sha256:a93399b9fced766b560f0816878c599333e7737ebaf3786e8e5ba7a0ba8911b3
home-path:sha256:e6cbf24179a57d6f4e02396334fe50dcea004185712c0ff961e1fe73fedef37d
home-path:sha256:707e97b318272a41a3fd52b631f45046e28ef443d55ba603341bb3c6efb12c4a
home-path:sha256:1700f9c98af55f77146ca56f8399b5241e003673cfd122d8e2d6c172c0056558
home-path:sha256:994269b4af463a1a99d44edc56df9bcebb282efc09176bd3b65f4cf42019fe6b
home-path:sha256:730423e83dba24c94bbefb1d44aeb59347cb15b2a504a86cf9593ef85fc4344d
home-path:sha256:e9b10090dddb7d8ff6144b6ced53c66e5efa7ba8034c2a2a437582cb0324c53e
home-path:sha256:e49a78c2eca352a3eeb7e1122a090f8ada86f903945cf535e5b84db6d5edaa5f
home-path:sha256:b37e8ed9c52d822141f11fa8dc8b0dec1b5afa90cf8f976a93a8e46c73870c6b
home-path:sha256:6daa6cd468355c6e79b5298a2a6d3ee8ab401f2873da929b90c4f2de65f23c51
home-path:sha256:7a8dfa2dd9e5ab64999d67ee3640c40bc2ca1eb783a950ed0021942ae5dd556a
home-path:sha256:943c182cb22645e2fb1734df16bd0e6c028bd3aa0bec9b5badb498d27eca4667
home-path:sha256:77c3ba877eb1f5c78b27c6157bcab1f6b7cf1021870a28263855aa5cfd928109
home-path:sha256:fb315860c623296f5827017d3fedbfb0fc767942ef57b571a3889d9f82e51aaf
home-path:sha256:4b119ad39fa2e1b241da184fc5d6d6ec60b4d9ce7389b3365604315f333654c3
home-path:sha256:bc5582a5a6b0db9a7cda072ad8f61b42ccc4caf29445bd4cfbe51ae4c67b0af1
home-path:sha256:82a6bdd20e2ed342b782792e36f5994074ddbafd9331c3aef319e7b44c5f911a
home-path:sha256:37a14b7e71e9192049bc3c5ce12357aa0bd2c699bbd0234ebff525104bdc7660
home-path:sha256:a3a0107f0d539ae8b4639eb7bad5ded590bd42b3be316218b1fce200938fcd04
home-path:sha256:5d7ac672315b9599706b2b501018c26a0fe9033a87eb0bb86b823cbeb34cbb33
home-path:sha256:207bc1ed75c43c270d4b77f82a381b07ae4ee000ba59253a8ca4a236c35665c5
home-path:sha256:a16f351428eda4e46e21df9f235e2ba603b42579971f9b1336506cdfe4c48ed9
home-path:sha256:bb4b27e30be187f26875cfe93d9178bd425347bf6841df629f8513721bae1361
home-path:sha256:7a819d46269c3d67abd35a7164c1af7e8378dc036a776e93b81bd7f0531b59bd
home-path:sha256:227a611dbbdbe8b0c27f17825385499de4ce078b85d6f7d693d24b669083743f
home-path:sha256:8b58102e78c3ee16db55885d78b7e5480cac355f0e135ede3fc9f44aeb00799d
home-path:sha256:5469aecab198c0325596283ad106ff97f24f14b99db7ed011e5dc6a12c62614f
home-path:sha256:0974c9edbf49a250f0454b2fb35190ffab321f59e7bea13791640934cd829456
home-path:sha256:4ce7b2d3a361e7e61d003873f86de186fa0433fe1a4dd5ea16ec4a38d9aeb54e
home-path:sha256:64fab9350963f35653f119195ff3745afc4efe7bb061e7b73fbc8a89c2cd5bfb
home-path:sha256:c5d7d3d1e48a3655d2f3e2c59b98ff3889411c427f40e0f21b40541f4b9258ee
home-path:sha256:158d428764fca2a1706bf2d69e9e7a9ddd6825f5b073980501cd2bd400644187
home-path:sha256:96a3790796945bafd23405ac7a6d4730bc5d7721585e62a289aa78cc76cc1509
home-path:sha256:834265a79f03364745c5aea2e888adf88dee72049d27109dab412430a8e4375f
home-path:sha256:f521c13feb26a3959f791fe88b8fb4999c06920f3781f35fc2355af5774a0558
home-path:sha256:8ebd02e30a5dd9a363eb5f76ba6a20862e5dc76d5aba10b170049fdea9aa6309
home-path:sha256:0fec51a4882f5f471811cc671b4c23bda797917983b6d89e3e781d2ce90bba4b
home-path:sha256:b32831deef783cb6386b27005f9a1ba5c5012f9f65a0b4172a1e83060b084ba1
home-path:sha256:0e9d57a52e8463340578a81e2f0457d8494a8ee71960efd1ddb6a8955e74a8a2
home-path:sha256:7f5a2033631224b040f9d105f6c10a345877c9c0442442bb75818ab7a7ba6a60
home-path:sha256:760ca0d2e46bee628023614e68f53b21a3ac44a92ca556248542365276765c5c
home-path:sha256:a823793e9bac9d6e4b22b73032bbc1c02fec3923202fad9d0e8f8d681e7f351b
home-path:sha256:1ac628a0f154d0948fabb6479ed73e48c458903bae139a1b96da7d704489f097
home-path:sha256:dce6ca13bde5c39492d53721b93bec2c6be0781c33bf3f90f5d8cb831d0748d8
home-path:sha256:7f7a46ecfe218638044beb10d6381f6a2713f61e3d003af7fa3b51cd59c078fe
home-path:sha256:49dd77836ed7cbe948841bed54890cf864cf20a0fa7833176c0b1f08e5f813e0
home-path:sha256:3e6478cdc69b49283a6a80ab2e1b209c63db3df851c1b60b9bc209d06b3ea944
home-path:sha256:2d0f81ac9535a3a7f5394d2f3ad4fd9a1b2a82933850e6cdb523c87495e55895
home-path:sha256:3347520f80b9c2f526cfc3a445741809fd0d97253a09f9a6398c74b5cd3f7176
home-path:sha256:2684ae2a074a3a645417a56a4138f5808f65e6c9df653f4f3e171de82a8ffe61
home-path:sha256:48b5513c52432936ac048c28ad5919ba5ef2ec7a8fd8213a4adcb4bd001c0b47
home-path:sha256:66ddf8770bea02598e52d73938f0641e2b4fb58e071c34c3bf02bc488f33b93c
home-path:sha256:4c6c9b9b2bdd8067c50c8981b9af7755ef3b30ac00ecc78a49b527c5874e922c
home-path:sha256:ccc35fa57ee6f91ae3457636afd633f0fcb56fb411358a8157af2301cf042a3d
home-path:sha256:7923db042e3775e05a9a5ac38c6bff76fbbc3ce70f5096eb97247799c134ac9d
home-path:sha256:c45daeaf6c18221a28c3a8652f986a639afb9e67bc45d189bf48a720a2e58095
home-path:sha256:a006144df094d30fb93be7f2d07a0a1d6dd9518dc411901ad2e5ced6c99b7bac
home-path:sha256:e959009e5b0b36c0eac563a686f6d9ae954f54726f31c223eed97a106818e1f6
home-path:sha256:121683eea3b6a96a89022d39cc612922482dc86af7a6f39c8dab0576649a335f
home-path:sha256:c076f27a9f553c28cf6589d160353e492045dd0f47fc67f876e809d9bd5b8081
home-path:sha256:b3603a6534dd63d4900c6a8eb600da899dc10625653517a5f27acb472197cf05
home-path:sha256:92c61dc42e180c596e26b87853e562deaf425ed960ef40c1c8c068235d0abe9d
home-path:sha256:25a928574e42ce6c19b363957d812b1b94030be6c62e72df301e05b836de822d
home-path:sha256:569699330bbc3396c6e21da0a6d07f1b797230147f9f8184e0db34f82b4b38f7
home-path:sha256:dfbaf1c014c3a8bafb68b6d578867a70dfbe7f730aaa7c748c03abae25c5076c
home-path:sha256:1bf48a8a282af4510b1ab2861a657fd1a80481205e0668bec74169f811ef6e9d
home-path:sha256:b64516bc984d2fa231dfc96c0b4e9968e74782996e30aac5bc28e0a678b316ed
home-path:sha256:630377c575dcc7fd87696f8de59428f20a94f8098e467e1845cc2c296b6bbf5f
home-path:sha256:dfceb8ddfbf9eefa86f9ed1e0b82c6260b43e6c7ae9b29d0471c3a162181a789
home-path:sha256:7b48a7915d1cdf9ba1407b5aa263f8214697bcdd2efe19ec0ce96a4a6a5ddb00
home-path:sha256:c3dcd3bdebfdfb274be153994950bec7f236ee0588f01cb04c3a00d84abc399c
home-path:sha256:0ab545bbd6619cf7c36dd151aa5fb38ac7b5f92a6bb5a9377be4f32ecbc14d08
home-path:sha256:fcfdf9c11c9a47acb09e98990fc8a3b0721f36835f50575a97608b7506abc5c4
home-path:sha256:0a129e23ede00e0bd889c446ceb3bdfc8a0cae805923abfb1f284c521fe25896
home-path:sha256:89a13d79723ab737e40425171d7b51cfa9b110241e72124efaf504b92679be63
home-path:sha256:67330b0743a65327dfe0bb112675614bfa4cea3496104d7f20a73b0851b6f07f
home-path:sha256:968b5fef28f1ee522f7e79d8ec10d193b94df11e9251e5420695f95b5e2a49f1
home-path:sha256:6d4be3de08b0db490e2122d1243949cfb7ada5f12a30a76012ce400817471ef5
home-path:sha256:8ff02761ccfe3a521ecd52cd490a89057c619201d4d39158c6362cd0f8b963d9
home-path:sha256:8f053791d74ee68ce585dffb0c0f0d287b46611b1929a5f9f530ca2840e207ce
home-path:sha256:ad05e2ad22405d80710701a41302e72261677413211f371da201e4bee351c0f3
home-path:sha256:9b1648a4dee57bc5ec1a36448962be8b0f79d414ab5b34cdefa8091025d7914f
home-path:sha256:3ab4509a230d69644d64905b94bbbf0f5dc3965256afbfaddc37d0a5f45a7c79
home-path:sha256:544c6e66846bac087905ba6d49f27eb692b09d59c621fd8e999e572e8871a39d
home-path:sha256:f14c6a2e0747a528c9d3e2d63bc932c47efee0286f65527dcb5b0b182d4e5d3f
home-path:sha256:961face5146dc99fb3b5615ec46e1d315befc95d879a7b5a17ae5fe9daf57099
home-path:sha256:66a243f0201388367047feff4218dbc067d12f5d44ad33d668e476b717150cba
home-path:sha256:5219b1c4e7b5229e0672ed053d4b6bb446d5b783f927b4ca6ed5b8f9ef43ff46
home-path:sha256:8512518ebf50e72b14b066342cb9dcc7e1db2086067920133bae5f726912ee40
home-path:sha256:05264b6301083064f6050df0c52c92c237a99863c85a152eb895cba018c071b7
home-path:sha256:896f1663c6c3d4827eeff1a2154ccfa074a35059ac4d95d87a0250345368ffb8
home-path:sha256:6df7ec1d8aa685d5cca7a7ca907b9e17d9dd879d52f49a741ab55cdf48147157
home-path:sha256:8dd4b0f8232af2dd17fee760b6d63f90a53d5ce452aa5ea8f07d37921ceedd06
home-path:sha256:ab9d1d580635b4a6a3e7eeb353e6241d263c98021987dae6fe3bc32c2c572dc5
home-path:sha256:d3eed182d51ddc4e2b361f2efdd9aa329b3784a205d36f76b27139ddf491cfef
home-path:sha256:d4b6f0025fdc3bea91ab0aee02c39a52fdffcb4398d45ed69ae41ad5241408a3
home-path:sha256:a6896826f6f8f56df4c681a9be0d9a01f2c66a2df81ab1005cf2fbc8cd7e4b62
home-path:sha256:860d3799e8d1ec9abf4802f784d25b8eaa45714e11cb5eb7e40bbe66cd0201e7
home-path:sha256:aae75050b6587791b242fbb79c68ef64ccf841fecb550906cd5509406629c02b
home-path:sha256:1d0ba6be7894a5080670e857c5af19653501839af7d86132e5360bcd95ff0163
home-path:sha256:8a55b059924a0cba09de614c58378062b77ee8534c7c860c488b678bfd143e4c
home-path:sha256:7027b23845ba689f7d34f9f1505c0446b54e1e40e4a2f7217fa3709055caf681
home-path:sha256:9c182351ad202fadaa6cf9b44d723a07747a1cd4f0594919806f7e8afd37dca3
home-path:sha256:174e3cb13cf0461d1a4cc0150d93b44718ffc808c17e1147cadeea59cba733ea
home-path:sha256:a888219a162b79eae756bcfbcd52433332b295097cbdc6bcc76317db26a6776e
home-path:sha256:4b3e3dc6f15e113becbc3a93f447188d46137a38a79b8eb5370090e68c78a08f
home-path:sha256:acd819baff142a24e06d594c4e78951f4bcd1242486fb10dd09d9b3a2a4f17ca
home-path:sha256:e089618fbcd61c5c66f5c07479f209784adcd1d9792ba52f8ad8f8e67efa4ce8
home-path:sha256:73ce238e6c3271e3796480a5925f286eb4001e672d2b2d42ef6ded378d4bcf62
home-path:sha256:29e356e0cecddc32bff16b4ea482776c0410e970b6e67f957eb8083b1432fc94
home-path:sha256:0296d4e4a89c080633a858d96f1e962ffe9fc69923b772e99bc79b7c309e3d38
home-path:sha256:d85b0554f49a8103f0bfdc92c42855fa1ecf9000afd3d2924cb5aa84628f1695
home-path:sha256:e4863d12ebda2e4388d2b13b77c71340fda543458a0beb8f76dfde1192089a7c
home-path:sha256:508c8735e8824c86e1809fcbe01c81155df5fcb0e931df29ca25547e1d4cf249
home-path:sha256:45b8991be529644b1d2fa1e183f051c82c261e7bb84f093ebc6d2fff9e26df5a
home-path:sha256:e546d2374e044dd61da9d8c1b417a50ba9670c7f043396ebf2fc2629dc11122f
home-path:sha256:bb30054281d815c2966b4ed526df57ae2bdd11743c84112efa9b20cc17634184
home-path:sha256:e69557f32af52af2ade2190a5988b36f795d6c4ff85d263a23410bfa9db7e0b1
home-path:sha256:5ac721bbda0ff1f9c13b65d915bab81a6e917c9e5c00c35ccfbe2328606241d5
home-path:sha256:575727eb3159a3b0aa07c673daea3a0dbf954996247629205d8656e082329903
home-path:sha256:f7087b608deb8264db6d201c4cfd51a3e4099701b58be184c676f9998f7b9a9e
home-path:sha256:4ae0ddeac855bfa2372efa7947b579b6ebf712002994f6845c433d8d6ff8ddd7
home-path:sha256:36529efbf91891f9573094092f5cf90602e45b74edacf4b5cc794e4b6a11e291
home-path:sha256:129bed1e01308f3362c7b511fa7c1f131adfe58f908f398e017e52773addae77
home-path:sha256:04030df211d41543f0b4ba6f4fcb8c10893579e7e713d7d3e583c1fe397c75d9
home-path:sha256:0d1e3ec6a8b3139c8802ac98de5c854c6301740727a8a6d3bd4bb6ba39920f27
home-path:sha256:82f55944781dd7721c2048bb32110534c241d3c243d6054682b6aab7e6cd9a4e
home-path:sha256:dc9adca3fe1af9de31522cb0dddc19b781ee800b727627a0ee2ce77320f03d1f
home-path:sha256:6b3fb5ed070ee83a24966c8e6a3deb73468cf44de8c8fe992f9c071e73b0e480
home-path:sha256:9ad7b9d60d3b93a9b7f71355d2a8bf941e58e8742905723e151c28128a3187c5
home-path:sha256:8abbf31511fc3e1d017a14e0de7e6da70b2b1aa8a8482e5ede6da3307063aa2e
home-path:sha256:bbbab16ff4b710029a6b06341ee391a505b004ee5f125ffbc3dec9e34297b4d9
home-path:sha256:1d181dc567aaaad2b60d7719901e9bd60bf274cf440d623c10ab05a8b1526a05
home-path:sha256:fc726f127e19ccb6e867f6a66784c95d8bcd553c6a36bf47b78274eb0d2a7414
home-path:sha256:b3baad8edde2ee1241386c8f8ee4a49806372283d389b5bc159b517180b92277
home-path:sha256:06b09e221aa6e481f6ed59cc9e447a307c7c0beb281199c86e09db4e88c96696
home-path:sha256:a7dafb22b4bf0f9ce5c198794a3b5add8cb88a1310893497d285f46371631f75
home-path:sha256:9db29717d7289e9afbe117e078a4ae17ca708b487c7cb68dcf2af71490ccffc3
home-path:sha256:5c18f9b2a862e53321ae7af8047bf16b2ac6b3ecfa359fdd0925cf65ca84d8ed
home-path:sha256:8c8045d20cdc117b7afc27ab512b13683bb63d149f2beb6a5d8e5f841ccafeeb
home-path:sha256:9fe032f52f1a88cde1a4ac2a64a99ba2a52bf726ae0408d1b995d67db774c8ac
home-path:sha256:d0ad29aa081ef57c667bfcc14ef209971f5a26a0030fce6dbfdcbe3d51156c9a
home-path:sha256:770ed5c8c88d081a9766f7e48682780b156b17cec0cf469380c05351fce4ee2b
home-path:sha256:20292db309bf0bd051ef350eeeb848795133b58aacd18e5d537d975182712c00
home-path:sha256:98def6f0d5756c8d037d6d428a1694e9e639a23dacee8eeb4d398b924935fee2
home-path:sha256:557c4fe5c35b365ddd8bdc187417479aad255468a33c48f041fbb1a90a2eda9d
home-path:sha256:59c1e35b98d56cbcba96c0b0de95d7d6fb00341b4d8840d0d87d5f3c34ad535b
home-path:sha256:a1d8aa62f287354d6a2d7a40e53c213c9183fb5d97215575c8cd95bfae0afad7
home-path:sha256:2464f728ae7ad90cf05ee584edb0d354092ec817b57a11afbe5d096c02462be6
home-path:sha256:064dd23cd4a4ac145e380fd5c2612ae55acefc9791259f28d97df2b9cbb54876
home-path:sha256:9794f113655a20429b21341aed4286ce7a39d3e68a2f0e1c38558c98cbcac0ee
home-path:sha256:c26040b5a53b031cdaadc159eedbfbd7ec3fab8454d49df73ed71f511246e400
home-path:sha256:a8936bcfa52286c3ea103927061aaf796b5532fb16e035f0b9358d13a373dff2
home-path:sha256:9a7c35aa1ee51617ab6175e8ae38ddbaa19b66f38c5fd7d5d804b8ab5d7fa920
home-path:sha256:009bbcef2a0d0c418f2e13d7b44deff7e23f1d3f6c4c6fc91c9b188735070a7a
home-path:sha256:f9407648ae54dd1103dfd571e0adc83a8c87ce53e08364d16d971fe2456e457d
home-path:sha256:bb74b30693d85ec9d4a667d31385818f7ed46c5a43dbad99b7db1101cb4d50a7
home-path:sha256:0f9245afcdae294ac231701c95743e0300dd09c76218475ec6d2976633363a93
home-path:sha256:871aafae5099951d2737efeaa2bd792424148b084a7817626d2ed796e5adc6b5
home-path:sha256:5f0c81b37005f840cae739224a26639157ff3fa2afcb374a9a92395659015d64
home-path:sha256:59d788280d3497a4c0bc97a65bc846ea32577b5b6e02103e55efb661724aa619
home-path:sha256:93f2f6dd3c7fc9e986b1a4564f2f3c88354cf80c6dcbd9551ea4968b174bf96d
home-path:sha256:d81b86f804813a1717073846d60420e2ae9c0ca362a6a0baa8ca398a0e58dcfc
home-path:sha256:debfa03af4548cb5b170c50d3f0e56c88dabe601a125b7b77e153224da689593
home-path:sha256:ac28c6628c3385025371dbbd3bf4301eb11b9f21fb11a2f900a85827de652aab
home-path:sha256:370e3ab8d44ce65ca9045b81630ce90f2dab4b01eb30aaa5c37392f16df01b06
home-path:sha256:244ab2e04c2580524f139e802a6fa8f66132c9374249fa1717163d6ba317e016
home-path:sha256:1fca0e32739944cc3c054b5f77e453bf13520e7f776e81db32a8ce6e726c8c22
home-path:sha256:663dbcbb2aa74ce0e7c1796917fa573f3af39484368191415a0c98cba020433b
home-path:sha256:dff74eaac5d23896f4f9e6cd7e40814d15f63ca8a65559eff4cc96aeedf3dc75
home-path:sha256:27b87491dde3c353eeb0a105a63935ee0463c88b08092a33fee7ce6b6cb2e531
home-path:sha256:52a29bbdc669974dd8919f5eac0120a77a2aa9e19d71c0dfdfe260a20a54931e
home-path:sha256:9253930c0bb838ca3f32595ca29d8bcefcd9d1ec9409b6df9945ee3ad548f88f
home-path:sha256:9b51e2ac26b4e97ea9507a828fc821b79a879a696968e48aac2827164c4724da
home-path:sha256:00ec181a5a4f9c81befe1f0096265ea44bf208fa6fa23be46c5c640be1f574fc
home-path:sha256:b5a2c9f3f3383d1fae5f9260533c5e49561695e65052b837339059c1c5eb88c0
home-path:sha256:430329e31b540429ec58cffc47e7700ea18dc8afe4309f77e8638818f5978aca
home-path:sha256:38e0f59f60b075769eae2b465353daab2248a5abac5ae38897a513b6a94ac8e5
home-path:sha256:a62ea49ff57a044727c2c3efaeeef682963185a76358ff1c008f1e5af38d1344
home-path:sha256:b95c734944199e9de444a8a5ffb6551bd3c6a2471037ae7465387f9856d4d1a4
home-path:sha256:4dddea9c621a92a0407a56fb9186b3d363fd2d7840eb195dfc56ab08dfcad797
home-path:sha256:de6d8f36530150541d7160536470953b91f60b7658bf2e724c47f2df9f7e8402
home-path:sha256:4b66bb5fe53d6b039ea9b0add343097ef88ddc5a679a73ebee188118442d2368
home-path:sha256:8215d821be59ba3e493e525a21476af71bf4c26eb6516a9b1a730ed60c61a07c
home-path:sha256:9d39c2d82e76e22e9189aa6c25015ccb4c0be69acbe7740c9dd0ca29a65cf798
home-path:sha256:cafc22eb98b0b6cae170239e67c6221da03cf6e25f79925a1e085e39874aa8b3
home-path:sha256:f349891fa7ba0154e426ab807801c82c75429016a17362558f619722efe37409
home-path:sha256:ca827aa1bd3f8721183b012a67aed63e0aea2188d9c7edfea3731c64754df8b0
home-path:sha256:e6d301ed2cd24c938f4333b9580ed3e893a68f33a39f8faaab79a093d1de9a1a
home-path:sha256:1426d7f719deb42c3026ae4fdf4cf961d63952c6ecff8e9f156bb9c87767f5c3
home-path:sha256:9938075c82259f39a6e31e1ca0523440ddd5546c7dcb48efd39fb8dc70d42eeb
home-path:sha256:9f7b973f8bc016bdb2876dce74e3ee43e8f7e97424aba09ed9da63434633681a
home-path:sha256:1e70650085658549b9f3f5f529a56a57023cb09a882f5c25215f8aaba9c50ef7
home-path:sha256:ba64bc26b36f7d97aba6028e0dc487a9eb3804833f47b1f92097f7f8e630eb7e
home-path:sha256:cfc6a9789e488139403a54bd46c1215254fe15f1cf65ef210f62c92c42ad926c
home-path:sha256:ddfe40e1db50b423c4419efe77efbe2b06349c1e394626ab9ba6d10798138ca6
home-path:sha256:690de01a2c5597e431b99c04c5cb39d0fc7101fdf24dfd4101eddb61358aeec3
home-path:sha256:dad27458d9fe0e37dd315fe20ad99285abf3f96a28d0e0c681511e9048e358cc
home-path:sha256:f447128f522ddf20f3cd9e3fee09d23bb59db69fa681e9ef7cc8956b6d9efefc
home-path:sha256:930c7885b271596a1fa4a32857b9879fc3d253b18c7a9bf0c17ef69fc6f8b937
home-path:sha256:d988eb1877fba120e39b6b38f25e6354315d5424d7daf4ed97842abab4a18c19
home-path:sha256:aa59d67a7ff4468f6af9ab6c4f2f60df2dc37db9fbdfa79b187e0de8113d37dd
home-path:sha256:4335c4dc0b9a6997c5e2bafa4057bf157b0999d4470d3139c75fb804632e6eab
home-path:sha256:5bb55af72fdef42b1c4da285af3dad9fcd7e28f9489672c7ae79c389542e5701
home-path:sha256:4e5ba39c1f5735123ca3044dd8a9ed5ab676c68190db1a675cc01a0d91df3b5b
home-path:sha256:b6b194d46d32522ab69a45e4bf2a380b33a1a37c6a2b53ab2cae03a05a288c59
home-path:sha256:fa314c66db34104ffa58922e15ca37ed48a48056c2742ab51d4a8fa15e5564b6
home-path:sha256:7ae60b1b6a02addfe31f6fa263598c7f0522c79d6a95bb6a3da2ee6a252a1386
home-path:sha256:b348c283f18a346bd5983d5eee427a7b482b07e20214c2a7b988c6d0e14f928b
home-path:sha256:2b928f66ce07954788fc2a885f731a36f8e0f1b68ed07d2cb9af2647d88ebe3d
home-path:sha256:44eff4c502600ebd422072810acca82711db90469de4a0bb4785bb359c8bf5fd
home-path:sha256:094806efbdae02375949668f8170abc8504bfda9fc75db4cd4314c84065ca7fe
home-path:sha256:3167c7dad8c9fdc113cd904981389958499341e666f555a71530fdda93ffa9c3
home-path:sha256:945c21e6e8a97242c0494e4839d76b6d77fb39e1a7d21bef4ac0aeacc57df8d3
home-path:sha256:4805ffbded2ed0ccd2309084401a2a1fefc646982a08186dd4fffc5d261cf9ab
home-path:sha256:09bca1f9a9938f9d3d653817c496dff55b5350731ad247323c719b3ab60ccc27
home-path:sha256:117849ef8d6e8436de9f3c043afeabec8d3c81918ebe9c6d28d28b2cc7e61c9c
home-path:sha256:afdb8126d048d734da00009d19246e82364cae95769606af28180910b89d555a
home-path:sha256:c748721a82577e91c3de9c9feba47439cfd154224244786aee13c64c6bb61d4d
home-path:sha256:288ba0ff2bf982504184d41b35d0c30407c71b7f21ca4d975170e38aabd43c4f
home-path:sha256:1a61dbebba3e63e3a5f101c78948ee136fb34c6b68cc3a2120fd81e3e09688af
home-path:sha256:c5fa07d13758206c73387982a8afbb260b2333102d0655537a2e0bb7322731ad
home-path:sha256:80d3747daf7e1dba42e5e152dbc7e4dcb96a47878e740f96f716301e4a6d1e59
home-path:sha256:e132373692c0c7debe7c600393a35fa57b7a05a526c569ec6343f9b6c91d4459
home-path:sha256:1f311c75d379d7ee7b6e786fc8f32b7ce215dc017d19b1b342256bf86c341389
home-path:sha256:8efdc621b85fc76c043fef3f4aab2af8d92bcf68d354c1b42de9adbd0b1404c2
home-path:sha256:d6dccebd17b550945a91de3ef891e7b30f36afd5204dda202ab46157cf341d4e
home-path:sha256:fcfab22294e4903a9244a48149076ede6a1fc2479b39a8aa771bfb8074f347e0
home-path:sha256:17f1db747dd6884891fbba789de608a11ac9ab9684c6dcd8f0863466d8ac7601
home-path:sha256:135c0721611b89e20b97f64c786a93304a8f7aa27d5509a277e1b2cb25b767c5
home-path:sha256:0b279c6b26718b551920f48b9242161edda512954c340cac5fc1b79117a4eea4
home-path:sha256:0b9d5dd18366921eb07692acf7f28315eb8712debab1e7aec7cc1b840ecf524f
home-path:sha256:c14ce30b14d1cdc6a739b90258cceebe1ac20bad1089fd73f89193f8959b1cf9
home-path:sha256:33ce1c820d3652d410ff6935bbadb3dc6f618c4f8acffc8750a090b9bf31c373
home-path:sha256:5000b44cbd35322503a61e073eff54539a9aa3ff893572787db9e8a8566eabe6
home-path:sha256:c19282c45942ec44f1756202e219c972897f07287033c326d4fe1e396d60a455
home-path:sha256:4e3dd629158346f1aec1a7058e23b0746bbe0558360a4c301505524d121e994e
home-path:sha256:0a447748bd1a0960a6063b1e5458ce444f96d6e2713fe901436f36da9877f677
home-path:sha256:655e9699907b12e07d7e6eb403ccdc89280a39b9f5a5f66292a273552953bf21
home-path:sha256:e5518217f572b7edee29abee82bd3043bc751e77e267161478c0d925f42c758b
home-path:sha256:bdc962c667102fd1ba517dbb7b5ee40316ce1d09b2d0db21fa7f586682402044
home-path:sha256:5b361b57a305a380df50a385c782da3832ba45ad3a21ebe609e224548413e4ae
home-path:sha256:f7834ab992daa9fe315206abf42194a748c077c99c4b7aa1e16234e5df8d1409
home-path:sha256:67c099b57694fe7b595852bdd8d7828c1e522c8f946d6a3d58b2c1d977233846
home-path:sha256:4568ec55996e1bd3b9d490a9dd5a0948930cf6ff2284d24c9c099a24293586bf
home-path:sha256:fe32b1f493811a5a6a150dd092a45ff775ad9e7aefea551807ba8a9c600906c5
home-path:sha256:804c8e8221af64246b1bf12588590043468872c0a67ca063c578943e73e6e038
home-path:sha256:3a70b964a928c2bcd2098422abcf9111d0e2d14914e261c16a4cc42b774ac6d8
home-path:sha256:c801c51ccf3fb55567a1885e1b967d02f02abdf8b5e9f8b85bc4009349360596
home-path:sha256:90d0e28b36b38112d1f6b1acb192b06f788b3932013a80a283f4d4ed0568229f
home-path:sha256:e44a8df67bbfe65132be402ca47f1a29ec66a1b68ab8535fb3a55f2850cc7ee3
home-path:sha256:9f260df9b3b1cde389748860f4fc1c01b8199c105c9283fdd6b5ac2f5f3b10b8
home-path:sha256:21ae2c2e9d4f3565a59e67037b00b0c95f4e28ace47e48998466637310c21220
home-path:sha256:7d4b047d00cbd3f6447083c193c39b587734f38f263f30c7d35df8d906318faf
home-path:sha256:8a4bd144420de18054def140815a946ec22c9b706320aebe828c488adcbfa830
home-path:sha256:08eb054274c38cc444ae8a6e1897d7ae853f45056added68c36ff2c83943fb22
home-path:sha256:34ab7b5cbf56342395d3596a1199c86b73ee2c2d80fe9e6709ae889f69caca5f
home-path:sha256:7872b79e95dd152374172495a4dd1674b94408d31d68c7b662996f7765860f0d
home-path:sha256:5de9250d47be77fe3818d50035a7ceb7001b01590ba19dd96adfeefcb17bfba0
home-path:sha256:0a77254757199af788748294dbabd73b444bb8a71701f547e08a3aff718f9659
home-path:sha256:996c14c69aa060b6e79b8bc3e15971919f7507b8b5ce24d6c18081845c44189d
home-path:sha256:52a400df791f19ec1fcc41f850468e673c3ab82dba69f55372cab68a45cb1692
home-path:sha256:13e8b6e3f1853a36cf1bf52ba5338be8dc577df3e3e313b80d5c18db7443be6d
home-path:sha256:66fe394e07dc893933ead5464b413035f3386f492c2e1bf4c40a2922507f15c0
home-path:sha256:c382c3c9c2d68b4bb998ab6aa20c16db8cbca3c608c461b4e66f6d613de260e7
home-path:sha256:2d2c3261a283d1ea9aea83f99dcadac4fcca2697f1daf8b735addcf3f78d6a31
home-path:sha256:f3f5e77ed11d4b7fc0047db1daf8f76e890dfc42d92591e40239f822d27025a4
home-path:sha256:29816550c3f8f989da984277004a94e17b5f221069320494ea720dacee014c8a
home-path:sha256:f85c6a4010fa22a3e8b73973329b3041ef7191bfe66b675835a64e16080d625f
home-path:sha256:8bbd05eb9f9739d4f9ef2caba23bb5b028c2d03f1d4dfa817a610a2222849cd7
home-path:sha256:0536f6068e0ddff8684add180631d2707cf11bdcfe6535d39e6d5ee694876a82
home-path:sha256:edb219059134d2ee0eb89833feb97d6749c137bfcd6ec25dbfa38e06b869f19e
home-path:sha256:b035a80dbe391ec54a9c3c563f7c11e9bc49298d27c82494b0a7ec99b7b940e3
home-path:sha256:fd9b9cfcf3d09749e1ee42509c27f25f30fa2f8e892b8836fbf8a932804f1e03
home-path:sha256:f9cf2f6c7663f39dc57e7c37c1d79302687eda2ad67d396a8669dca6869238d3
home-path:sha256:e387a3a12e5bba361427add35a28d04cca53c0f07b8d21e2ecea9019724091b7
home-path:sha256:853b32c1990b00b24308b46bf458ca841ca48ee5bcebee8d7450699ae188fb09
home-path:sha256:2a45ff40135aea9a6d55ced6446740b007d0d7723df0e5a46a8c9c9ad29be790
home-path:sha256:4514d3a68c0743bc3f48b5b5a0e3b507a3dfe6bd8738c3b1b4a5e818a8fc7c01
home-path:sha256:ff975cdcd98a10d317988301696b14568f2f7992c7dd445cda62aee94dabc1ab
home-path:sha256:e02aaca809334899833a0c2307be72bbbc0bdd0af66839abb5f566e0d8e0b00f
home-path:sha256:5a59a94e8f178a0c86542210a2925fd5ade07e2e971d7f1f97a4f96057932fec
home-path:sha256:014714421f05ca5afd60629dcafd6e7f667ec2ea5a15c45c4ab104f78a51704f
home-path:sha256:8b49363a68f71788e40cc9c0bda9c83f43f948bedcddbfdf816645483f3341bf
home-path:sha256:e963288e36fb960c47f70d1817d05f1755ae2471d9c028f822d396500b95e98f
home-path:sha256:98074802c42d11bb325327a7be7bdc3d07042bbc01e4812b7779d8948a1aef32
home-path:sha256:fd37bfc17e94bbf9c018163c06a6777182a8d6b2134611bc63634ab19707db31
home-path:sha256:6f7a16b1ac5ac91b1c16b007a860576676bb4990a3a41fd2535e04c0a640e267
home-path:sha256:89673a9ea70018641b0e7b8f77ad78fd63f5d67fc2361cf9079beb78fc170b61
home-path:sha256:6569887bbaa3f90507dcca6b7e17b30c67fc5328b27fdd57524c725941d9103c
home-path:sha256:891053aa7e97021c7a77850a575994c916b6474a5898efeb396825a1cf54dd20
home-path:sha256:4079db6ee6188c7ef4b2e927488e4bb0e98af09bf1d8c36777dd446e1bf44710
home-path:sha256:a1f090816fc4f5b179ea5403cc9946323407ec74413fd1bad502ffca9bfb7a29
home-path:sha256:277071d6b5b6d6ccb6fd67fb738941d725ea583bd122ac31b41413ead7e229b5
home-path:sha256:792faf2efed1bbda1c906d4b4a19980a2d6f8889e86d76d46af7effa0c3639b2
home-path:sha256:dbdf4c33b05a3e390ba6941ce1ec6df566b4171a945fa96fcfca0c20fc61bde7
home-path:sha256:addac1524adb18bef3f065b2ade953fe2e75ccd600fdfcb849e14763e1e0e9b2
home-path:sha256:07fb043704e85587cedd6a3cb3eac4aa7d41942ed04b87b6a6d3b0752d5ed014
home-path:sha256:094c8bc57766d70ca68b2417a4737d9dc0023d0209089501501be50293c14010
home-path:sha256:8bafa88f048692e027475ab71d7666caeddb81dfe9b74722b6e52ea2720288d0
home-path:sha256:02926826c97d706b13a38cb04a84497a6c80eb38c82931e1702f185ed3b36086
home-path:sha256:de141f8aefb656d9503d771d3a4e2abf26028c50365dd48fc9f556000da0a16e
home-path:sha256:cc2bfa0db6cfb935393a02c7317c91c8fd9c70e5f40eb26ec2c61f08e8def35a
home-path:sha256:47e36db34bb96e0dd7fbf434b24bad5c4ff87fa426be2be7278f84d47ac7683d
home-path:sha256:4441d2d3ccda57ff30b45d4abfe7b4b34eb0a459f40b8f6bcdeef0e2cf0334b4
home-path:sha256:81df5b1b87b923743608a976256f16f7fc90804ade99793c87e55cdc915a3268
home-path:sha256:bfbd34458e913545aa8ef9475fb8deac1dba4a0ca4ca3f159decb510d9f09877
home-path:sha256:911eea5513fadd77dffeae919c380f2574bbc6df9b1cfc55439bcd67b83dacac
home-path:sha256:77a359d03fdc25c7e6e582def9e729494d248eae2a2e84c1a64277d11bfb2c54
home-path:sha256:419d7070a5d98aac5571dcd2ae688cf05232d590f274b8b0cd094b3e65f78de6
home-path:sha256:b83597989217ea90522ac8a5b0915759f767f0459730d5c6b33ca586c8845b78
home-path:sha256:122ca34c1bcb0805a6fb1e9368b4625887d87dc592d6f72e665381ea4edc1b2b
home-path:sha256:6b9492c3749cdd453820e3f827112c86bf4ab76548c29f26cd6d2df43aba0f32
home-path:sha256:f544b16a7c48f847c339dc518bb41dcd7ccbe6b68a722c5b7643e5c1f0e95542
home-path:sha256:ea625ec388d80dce29d32a27777a640610207fc920334c3e8c8c2de451bd1436
home-path:sha256:a6f4546545c88b554b453f565efb8d976c5e79806f435ad2cd9a3cf22c0559f2
home-path:sha256:122a92a41a24525936823f54ce0b25a1a546be40349a7f113b334738d172f5d2
home-path:sha256:56df18db1e40ba56c3da69c407356607cd2ada68d8108a89906205d56ed675cd
home-path:sha256:beb91c25cfe8da517be331c205b1f71d33a9571cb73c8cfc8507269979fda814
home-path:sha256:257e09c9b83a4a485fb0c1d62a8ff23522478dbd018b82fc85640d3e32b4d015
```

External census:

```json
{
  "root": "home-path:sha256:6a7bc68a68f3089a2737ac2f45b8979eb68b28577a1782d41c27a4ebba2c82ea",
  "entries": 1653,
  "by_kind": {
    "directory": 625,
    "file": 1009,
    "symlink": 19
  },
  "regular_file_bytes": 529339291,
  "links_followed": false
}
```

evidence-manifest.jsonl is the complete native-byte census of this review scratch, including caches, before snapshots, exports, raw logs and orchestration sources. It excludes only adversary-report.md, evidence-manifest.jsonl, manifest-summary.json and SEAL-SHA256SUMS to avoid self-reference. Borrowed cache links remain literal read-only references; no referent is traversed. External browser files have the separately pinned complete catalog. Directory metadata is an observation because creating excluded seal files changes parent directory metadata. All retained file bytes and literal links are reread during sealing. No original preparation, implementor or earlier review evidence was overwritten.

All source, test, scratch and external-browser writes are relinquished with this immutable report/seal. Root owns the explicitly recorded new-test scope disposition and all later actions. No further source attack is authorized or launched.

7. Structured findings

```findings
[]
```
