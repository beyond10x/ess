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
wrote-outside-worktree: 513 retained paths under /home/timo/.cache/ess-w11-review1-tmp
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
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer",
  "environment": {
    "PATH": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:/home/timo/.nvm/versions/node/v24.20.0/bin:/home/timo/.local/bin:/home/timo/.deno/bin:/home/linuxbrew/.linuxbrew/Caskroom/codex/0.153.4/codex-path:/home/timo/.codex/tmp/arg0/codex-arg0jTsWIw:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.opencode/bin:/home/timo/.fly/bin:/home/timo/.cargo/bin:/home/timo:/home/timo/anaconda/bin:/usr/bin:/home/timo/.rbenv:/usr/local/go/bin:/home/timo/go/bin:/home/timo/go:/home/timo/.local/share/gem/ruby/3.0.0/bin:/home/timo/.deno/bin:/home/timo/.yarn/bin:/home/timo/.pulumi/bin:/opt/rocm/bin:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.sdkman/candidates/scala/current/bin:/home/timo/.sdkman/candidates/maven/current/bin:/home/timo/.sdkman/candidates/java/current/bin:/home/timo/.sdkman/candidates/groovy/current/bin:/home/timo/.sdkman/candidates/grails/current/bin:/home/timo/.sdkman/candidates/gradle/current/bin:/home/timo/.local/bin:/usr/local/bin:/bin:/usr/local/sbin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems:/usr/lib/jvm/default/bin:/usr/bin/site_perl:/usr/bin/vendor_perl:/usr/bin/core_perl:/usr/lib/rustup/bin:/var/lib/snapd/snap/bin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems",
    "RUSTC": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc",
    "RUSTDOC": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc",
    "TMPDIR": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/tmp",
    "GOCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/go-cache",
    "GOMODCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/go-mod",
    "CARGO_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/cargo-home",
    "ESS_BROWSER_TMPDIR": "/home/timo/.cache/ess-w11-review1-tmp",
    "ESS_COVERAGE_EXPORT_ROOT": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/exports-focused-first",
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
   Compiling ess-primitives v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/specify/ess-primitives)
   Compiling num-traits v0.2.19
   Compiling libc v0.2.189
   Compiling ess-domain v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/specify/ess-domain)
   Compiling num-integer v0.1.47
   Compiling getrandom v0.3.4
   Compiling zerocopy v0.8.56
   Compiling version_check v0.9.5
   Compiling heck v0.5.0
   Compiling pulldown-cmark v0.13.4
   Compiling ahash v0.8.12
   Compiling num-bigint v0.4.8
   Compiling ess-compiler v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/specify/ess-compiler)
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
   Compiling ess-gen v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/generate/ess-gen)
   Compiling infra-domain v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/infra/infra-domain)
   Compiling anstyle v1.0.14
   Compiling colorchoice v1.0.5
   Compiling anstyle-query v1.1.5
   Compiling unicode-general-category v1.1.0
   Compiling borrow-or-share v0.2.4
   Compiling bit-vec v0.8.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstream v1.0.0
   Compiling bit-set v0.8.0
   Compiling infra-compiler v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/infra/infra-compiler)
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
   Compiling infra-analyze v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/infra/infra-analyze)
   Compiling fancy-regex v0.19.0
   Compiling regex v1.13.1
   Compiling jsonschema-regex v0.52.1
   Compiling clap_derive v4.6.4
   Compiling email_address v0.2.9
   Compiling data-encoding v2.11.1
   Compiling anyhow v1.0.104
   Compiling jsonschema v0.52.1
   Compiling clap v4.6.6
   Compiling infra-spec v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/infra/infra-spec)
   Compiling ess-conformance v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/verify/ess-conformance)
   Compiling ess-realization v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/specify/ess-realization)
   Compiling semver v1.0.28
   Compiling base64 v0.22.1
   Compiling ess-deployment v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/generate/ess-deployment)
   Compiling schema-contract v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/generate/schema-contract)
   Compiling ess-diff v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/verify/ess-diff)
   Compiling infra-project v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/infra/infra-project)
   Compiling ess-kubernetes v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/infra/ess-kubernetes)
   Compiling ess-synth v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/generate/ess-synth)
   Compiling ess-composition v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/specify/ess-composition)
   Compiling ess-openapi v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/generate/ess-openapi)
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/edge/ess-cli)
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
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer",
  "environment": {
    "PATH": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:/home/timo/.nvm/versions/node/v24.20.0/bin:/home/timo/.local/bin:/home/timo/.deno/bin:/home/linuxbrew/.linuxbrew/Caskroom/codex/0.153.4/codex-path:/home/timo/.codex/tmp/arg0/codex-arg0jTsWIw:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.opencode/bin:/home/timo/.fly/bin:/home/timo/.cargo/bin:/home/timo:/home/timo/anaconda/bin:/usr/bin:/home/timo/.rbenv:/usr/local/go/bin:/home/timo/go/bin:/home/timo/go:/home/timo/.local/share/gem/ruby/3.0.0/bin:/home/timo/.deno/bin:/home/timo/.yarn/bin:/home/timo/.pulumi/bin:/opt/rocm/bin:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.sdkman/candidates/scala/current/bin:/home/timo/.sdkman/candidates/maven/current/bin:/home/timo/.sdkman/candidates/java/current/bin:/home/timo/.sdkman/candidates/groovy/current/bin:/home/timo/.sdkman/candidates/grails/current/bin:/home/timo/.sdkman/candidates/gradle/current/bin:/home/timo/.local/bin:/usr/local/bin:/bin:/usr/local/sbin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems:/usr/lib/jvm/default/bin:/usr/bin/site_perl:/usr/bin/vendor_perl:/usr/bin/core_perl:/usr/lib/rustup/bin:/var/lib/snapd/snap/bin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems",
    "RUSTC": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc",
    "RUSTDOC": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc",
    "TMPDIR": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/tmp",
    "GOCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/go-cache",
    "GOMODCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/go-mod",
    "CARGO_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/cargo-home",
    "ESS_BROWSER_TMPDIR": "/home/timo/.cache/ess-w11-review1-tmp",
    "ESS_COVERAGE_EXPORT_ROOT": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/exports-focused-d1-generated",
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
   Compiling ess-primitives v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/specify/ess-primitives)
   Compiling serde_yaml v0.9.34+deprecated
   Compiling ess-domain v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/specify/ess-domain)
   Compiling ess-compiler v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/specify/ess-compiler)
   Compiling ess-gen v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/generate/ess-gen)
   Compiling ess-conformance v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/verify/ess-conformance)
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
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer",
  "environment": {
    "PATH": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:/home/timo/.nvm/versions/node/v24.20.0/bin:/home/timo/.local/bin:/home/timo/.deno/bin:/home/linuxbrew/.linuxbrew/Caskroom/codex/0.153.4/codex-path:/home/timo/.codex/tmp/arg0/codex-arg0jTsWIw:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.opencode/bin:/home/timo/.fly/bin:/home/timo/.cargo/bin:/home/timo:/home/timo/anaconda/bin:/usr/bin:/home/timo/.rbenv:/usr/local/go/bin:/home/timo/go/bin:/home/timo/go:/home/timo/.local/share/gem/ruby/3.0.0/bin:/home/timo/.deno/bin:/home/timo/.yarn/bin:/home/timo/.pulumi/bin:/opt/rocm/bin:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.sdkman/candidates/scala/current/bin:/home/timo/.sdkman/candidates/maven/current/bin:/home/timo/.sdkman/candidates/java/current/bin:/home/timo/.sdkman/candidates/groovy/current/bin:/home/timo/.sdkman/candidates/grails/current/bin:/home/timo/.sdkman/candidates/gradle/current/bin:/home/timo/.local/bin:/usr/local/bin:/bin:/usr/local/sbin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems:/usr/lib/jvm/default/bin:/usr/bin/site_perl:/usr/bin/vendor_perl:/usr/bin/core_perl:/usr/lib/rustup/bin:/var/lib/snapd/snap/bin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems",
    "RUSTC": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc",
    "RUSTDOC": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc",
    "TMPDIR": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/tmp",
    "GOCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/go-cache",
    "GOMODCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/go-mod",
    "CARGO_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/cargo-home",
    "ESS_BROWSER_TMPDIR": "/home/timo/.cache/ess-w11-review1-tmp",
    "ESS_COVERAGE_EXPORT_ROOT": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/exports-focused-d1-authored",
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
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer",
  "environment": {
    "PATH": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:/home/timo/.nvm/versions/node/v24.20.0/bin:/home/timo/.local/bin:/home/timo/.deno/bin:/home/linuxbrew/.linuxbrew/Caskroom/codex/0.153.4/codex-path:/home/timo/.codex/tmp/arg0/codex-arg0jTsWIw:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.opencode/bin:/home/timo/.fly/bin:/home/timo/.cargo/bin:/home/timo:/home/timo/anaconda/bin:/usr/bin:/home/timo/.rbenv:/usr/local/go/bin:/home/timo/go/bin:/home/timo/go:/home/timo/.local/share/gem/ruby/3.0.0/bin:/home/timo/.deno/bin:/home/timo/.yarn/bin:/home/timo/.pulumi/bin:/opt/rocm/bin:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.sdkman/candidates/scala/current/bin:/home/timo/.sdkman/candidates/maven/current/bin:/home/timo/.sdkman/candidates/java/current/bin:/home/timo/.sdkman/candidates/groovy/current/bin:/home/timo/.sdkman/candidates/grails/current/bin:/home/timo/.sdkman/candidates/gradle/current/bin:/home/timo/.local/bin:/usr/local/bin:/bin:/usr/local/sbin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems:/usr/lib/jvm/default/bin:/usr/bin/site_perl:/usr/bin/vendor_perl:/usr/bin/core_perl:/usr/lib/rustup/bin:/var/lib/snapd/snap/bin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems",
    "RUSTC": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc",
    "RUSTDOC": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc",
    "TMPDIR": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/tmp",
    "GOCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/go-cache",
    "GOMODCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/go-mod",
    "CARGO_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/cargo-home",
    "ESS_BROWSER_TMPDIR": "/home/timo/.cache/ess-w11-review1-tmp",
    "ESS_COVERAGE_EXPORT_ROOT": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/exports-focused-go-diagnostic",
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
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/edge/ess-cli)
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
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer",
  "environment": {
    "PATH": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:/home/timo/.nvm/versions/node/v24.20.0/bin:/home/timo/.local/bin:/home/timo/.deno/bin:/home/linuxbrew/.linuxbrew/Caskroom/codex/0.153.4/codex-path:/home/timo/.codex/tmp/arg0/codex-arg0jTsWIw:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.opencode/bin:/home/timo/.fly/bin:/home/timo/.cargo/bin:/home/timo:/home/timo/anaconda/bin:/usr/bin:/home/timo/.rbenv:/usr/local/go/bin:/home/timo/go/bin:/home/timo/go:/home/timo/.local/share/gem/ruby/3.0.0/bin:/home/timo/.deno/bin:/home/timo/.yarn/bin:/home/timo/.pulumi/bin:/opt/rocm/bin:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.sdkman/candidates/scala/current/bin:/home/timo/.sdkman/candidates/maven/current/bin:/home/timo/.sdkman/candidates/java/current/bin:/home/timo/.sdkman/candidates/groovy/current/bin:/home/timo/.sdkman/candidates/grails/current/bin:/home/timo/.sdkman/candidates/gradle/current/bin:/home/timo/.local/bin:/usr/local/bin:/bin:/usr/local/sbin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems:/usr/lib/jvm/default/bin:/usr/bin/site_perl:/usr/bin/vendor_perl:/usr/bin/core_perl:/usr/lib/rustup/bin:/var/lib/snapd/snap/bin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems",
    "RUSTC": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc",
    "RUSTDOC": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc",
    "TMPDIR": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/tmp",
    "GOCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/go-cache",
    "GOMODCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/go-mod",
    "CARGO_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/cargo-home",
    "ESS_BROWSER_TMPDIR": "/home/timo/.cache/ess-w11-review1-tmp",
    "ESS_COVERAGE_EXPORT_ROOT": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/exports-package-suite",
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
   Compiling ess-diff v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/verify/ess-diff)
   Compiling ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/edge/ess-cli)
   Compiling ess-conformance v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/verify/ess-conformance)
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
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer",
  "environment": {
    "PATH": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:/home/timo/.nvm/versions/node/v24.20.0/bin:/home/timo/.local/bin:/home/timo/.deno/bin:/home/linuxbrew/.linuxbrew/Caskroom/codex/0.153.4/codex-path:/home/timo/.codex/tmp/arg0/codex-arg0jTsWIw:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.opencode/bin:/home/timo/.fly/bin:/home/timo/.cargo/bin:/home/timo:/home/timo/anaconda/bin:/usr/bin:/home/timo/.rbenv:/usr/local/go/bin:/home/timo/go/bin:/home/timo/go:/home/timo/.local/share/gem/ruby/3.0.0/bin:/home/timo/.deno/bin:/home/timo/.yarn/bin:/home/timo/.pulumi/bin:/opt/rocm/bin:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.sdkman/candidates/scala/current/bin:/home/timo/.sdkman/candidates/maven/current/bin:/home/timo/.sdkman/candidates/java/current/bin:/home/timo/.sdkman/candidates/groovy/current/bin:/home/timo/.sdkman/candidates/grails/current/bin:/home/timo/.sdkman/candidates/gradle/current/bin:/home/timo/.local/bin:/usr/local/bin:/bin:/usr/local/sbin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems:/usr/lib/jvm/default/bin:/usr/bin/site_perl:/usr/bin/vendor_perl:/usr/bin/core_perl:/usr/lib/rustup/bin:/var/lib/snapd/snap/bin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems",
    "RUSTC": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc",
    "RUSTDOC": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc",
    "TMPDIR": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/tmp",
    "GOCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/go-cache",
    "GOMODCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/go-mod",
    "CARGO_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/cargo-home",
    "ESS_BROWSER_TMPDIR": "/home/timo/.cache/ess-w11-review1-tmp",
    "ESS_COVERAGE_EXPORT_ROOT": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/exports-fmt-check",
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
  "cwd": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer",
  "environment": {
    "PATH": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:/home/timo/.nvm/versions/node/v24.20.0/bin:/home/timo/.local/bin:/home/timo/.deno/bin:/home/linuxbrew/.linuxbrew/Caskroom/codex/0.153.4/codex-path:/home/timo/.codex/tmp/arg0/codex-arg0jTsWIw:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.opencode/bin:/home/timo/.fly/bin:/home/timo/.cargo/bin:/home/timo:/home/timo/anaconda/bin:/usr/bin:/home/timo/.rbenv:/usr/local/go/bin:/home/timo/go/bin:/home/timo/go:/home/timo/.local/share/gem/ruby/3.0.0/bin:/home/timo/.deno/bin:/home/timo/.yarn/bin:/home/timo/.pulumi/bin:/opt/rocm/bin:/home/timo/.bun/bin:/home/linuxbrew/.linuxbrew/bin:/home/linuxbrew/.linuxbrew/sbin:/home/timo/.sdkman/candidates/scala/current/bin:/home/timo/.sdkman/candidates/maven/current/bin:/home/timo/.sdkman/candidates/java/current/bin:/home/timo/.sdkman/candidates/groovy/current/bin:/home/timo/.sdkman/candidates/grails/current/bin:/home/timo/.sdkman/candidates/gradle/current/bin:/home/timo/.local/bin:/usr/local/bin:/bin:/usr/local/sbin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems:/usr/lib/jvm/default/bin:/usr/bin/site_perl:/usr/bin/vendor_perl:/usr/bin/core_perl:/usr/lib/rustup/bin:/var/lib/snapd/snap/bin:/opt/cuda/bin:/opt/cuda/integration/nsight-compute:/opt/cuda/integration/nsight-systems",
    "RUSTC": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc",
    "RUSTDOC": "/home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustdoc",
    "TMPDIR": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/tmp",
    "GOCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/go-cache",
    "GOMODCACHE": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/go-mod",
    "CARGO_HOME": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/cargo-home",
    "ESS_BROWSER_TMPDIR": "/home/timo/.cache/ess-w11-review1-tmp",
    "ESS_COVERAGE_EXPORT_ROOT": "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/target/review-boundaries-11/adversary-pass-1/exports-strict-clippy",
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
    Checking ess-primitives v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/specify/ess-primitives)
    Checking bitflags v2.13.1
    Checking ess-domain v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/specify/ess-domain)
    Checking unicase v2.9.0
    Checking pulldown-cmark-escape v0.11.0
    Checking pulldown-cmark v0.13.4
    Checking num-traits v0.2.19
    Checking libc v0.2.189
    Checking ess-compiler v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/specify/ess-compiler)
    Checking num-integer v0.1.47
    Checking ess-gen v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/generate/ess-gen)
    Checking num-bigint v0.4.8
    Checking regex-syntax v0.8.11
    Checking ess-conformance v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/verify/ess-conformance)
    Checking num-rational v0.4.2
    Checking getrandom v0.3.4
    Checking zerocopy v0.8.56
    Checking ess-diff v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/verify/ess-diff)
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
    Checking infra-domain v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/infra/infra-domain)
    Checking anstyle v1.0.14
    Checking is_terminal_polyfill v1.70.2
    Checking bit-vec v0.8.0
    Checking colorchoice v1.0.5
    Checking borrow-or-share v0.2.4
    Checking anstyle-query v1.1.5
    Checking fluent-uri v0.4.1
    Checking anstream v1.0.0
    Checking bit-set v0.8.0
    Checking infra-compiler v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/infra/infra-compiler)
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
    Checking infra-analyze v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/infra/infra-analyze)
    Checking strum v0.28.0
    Checking unicode-general-category v1.1.0
    Checking fancy-regex v0.19.0
    Checking regex v1.13.1
    Checking jsonschema-regex v0.52.1
    Checking email_address v0.2.9
    Checking data-encoding v2.11.1
    Checking clap v4.6.6
    Checking infra-spec v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/infra/infra-spec)
    Checking jsonschema v0.52.1
    Checking ess-realization v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/specify/ess-realization)
    Checking semver v1.0.28
    Checking base64 v0.22.1
    Checking ess-deployment v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/generate/ess-deployment)
    Checking infra-project v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/infra/infra-project)
    Checking ess-kubernetes v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/infra/ess-kubernetes)
    Checking schema-contract v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/generate/schema-contract)
    Checking anyhow v1.0.104
    Checking ess-synth v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/generate/ess-synth)
    Checking ess-composition v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/specify/ess-composition)
    Checking ess-openapi v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/generate/ess-openapi)
    Checking ess-cli v0.20.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-coverage-writer/crates/edge/ess-cli)
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
- New scratch writes are under this report's directory, plus the two additive tests and the assigned unit target build output. CARGO_HOME uses the explicitly borrowed literal registry/git links to /home/timo/.cargo/registry and /home/timo/.cargo/git, without following them for ownership/census. No shared target or cache daemon was created. The measured free-space floor was 8,589,934,592 bytes; setup measured 23,678,291,968 and the final strict-Clippy receipt measured 20,485,578,752 free bytes. Every lane's before/after values are retained.
- All assertions remain intact. Four final Rust/CLI executable copies are retained and hashed for provenance; the first focused executable was not separately copied before later additive test compilation, so its exact binary hash is not invented. The first command records its test-source hashes and full build output.

6. Every retained path written outside the worktree

Only /home/timo/.cache/ess-w11-review1-tmp was used outside the assigned unit. The following is the complete post-run census, also in external-browser-paths.txt; external-browser-catalog.jsonl records full path/native bytes, lstat mode/owner/size/inode/device/mtime, literal symlink targets and SHA256 for regular files without following links. It contains 513 entries: 199 directories, 308 files (161,983,501 bytes), six symlinks and no sockets at capture. Per-case command/profile/PID/BiDi receipts retain the observed transient browser endpoints. This census is not a syscall trace of temporary files internally created and removed by tools. No reviewer cleanup was run.

```text
/home/timo/.cache/ess-w11-review1-tmp
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/.parentlock
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/.startup-incomplete
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/73fdb0b3.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/73fdb0b3.sqlite-shm
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/73fdb0b3.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/SiteSecurityServiceState.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/WebDriverBiDiServer.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/activity-stream.shortcut_cache.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/bookmarkbackups
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/bounce-tracking-protection.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/cache2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/cache2/doomed
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/cache2/entries
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/cache2/entries/360DE1F3E174E84794C36651DD126686164A2F9A
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/cache2/entries/5E43012191E7B1F510ACD5BF039D8BF1B9A277B2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/cache2/entries/BB95D0607349D05725D5FE01D4FB300E319072AD
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/cert9.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/compatibility.ini
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/content-prefs.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/cookies.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/crashes
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/crashes/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/datareporting
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/datareporting/archived
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/datareporting/archived/2026-09
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/datareporting/archived/2026-09/1788724886510.428114db-5dd3-4509-9c7a-dfb999106a33.deletion-request.jsonlz4
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/datareporting/glean
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/datareporting/glean/client_id.txt
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/datareporting/glean/db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/datareporting/glean/db/data.safe.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/datareporting/glean/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/datareporting/glean/events/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/datareporting/glean/pending_pings
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/datareporting/glean/tmp
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/datareporting/state.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/domain_to_categories.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/domain_to_categories.sqlite-journal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/extension-store
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/extension-store/data.safe.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/extensions.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/favicons.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/favicons.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/key4.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/lock
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/logins.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/minidumps
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/permissions.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/pkcs11.txt
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/places.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/places.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/prefs.js
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/safebrowsing
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/saved-telemetry-pings
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/saved-telemetry-pings/428114db-5dd3-4509-9c7a-dfb999106a33
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/security_state
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/sessionCheckpoints.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/startupCache
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/startupCache/startupCache.8.little
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/default
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/ls-archive.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/.metadata-v2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/2918063365piupsah.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/2918063365piupsah.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/2918063365piupsah.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/3561288849sdhlie.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/3561288849sdhlie.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/3561288849sdhlie.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage/temporary
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/storage.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/thumbnails
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1053388-0/times.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/.parentlock
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/.startup-incomplete
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/SiteSecurityServiceState.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/WebDriverBiDiServer.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/activity-stream.shortcut_cache.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/bookmarkbackups
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/bounce-tracking-protection.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/ca7f7af5.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/ca7f7af5.sqlite-shm
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/ca7f7af5.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/cache2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/cache2/doomed
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/cache2/entries
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/cache2/entries/360DE1F3E174E84794C36651DD126686164A2F9A
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/cache2/entries/5E43012191E7B1F510ACD5BF039D8BF1B9A277B2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/cache2/entries/BB95D0607349D05725D5FE01D4FB300E319072AD
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/cert9.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/compatibility.ini
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/content-prefs.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/cookies.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/crashes
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/crashes/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/datareporting
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/datareporting/archived
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/datareporting/archived/2026-09
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/datareporting/archived/2026-09/1788725412887.7d6d6474-df9c-4ee2-8639-a8f166bca399.deletion-request.jsonlz4
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/datareporting/glean
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/datareporting/glean/client_id.txt
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/datareporting/glean/db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/datareporting/glean/db/data.safe.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/datareporting/glean/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/datareporting/glean/events/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/datareporting/glean/pending_pings
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/datareporting/glean/tmp
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/datareporting/state.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/domain_to_categories.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/domain_to_categories.sqlite-journal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/extension-store
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/extension-store/data.safe.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/extensions.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/favicons.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/favicons.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/key4.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/lock
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/logins.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/minidumps
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/permissions.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/pkcs11.txt
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/places.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/places.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/prefs.js
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/safebrowsing
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/saved-telemetry-pings
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/saved-telemetry-pings/7d6d6474-df9c-4ee2-8639-a8f166bca399
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/security_state
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/sessionCheckpoints.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/startupCache
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/startupCache/startupCache.8.little
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/default
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/ls-archive.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/.metadata-v2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/2918063365piupsah.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/2918063365piupsah.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/2918063365piupsah.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/3561288849sdhlie.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/3561288849sdhlie.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/3561288849sdhlie.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage/temporary
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/storage.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/thumbnails
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-0/times.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/.parentlock
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/.startup-incomplete
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/4bcdcee8.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/4bcdcee8.sqlite-shm
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/4bcdcee8.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/SiteSecurityServiceState.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/WebDriverBiDiServer.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/activity-stream.shortcut_cache.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/addonStartup.json.lz4
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/bookmarkbackups
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/bounce-tracking-protection.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/cache2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/cache2/doomed
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/cache2/entries
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/cache2/entries/360DE1F3E174E84794C36651DD126686164A2F9A
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/cache2/entries/5E43012191E7B1F510ACD5BF039D8BF1B9A277B2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/cache2/entries/BB95D0607349D05725D5FE01D4FB300E319072AD
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/cert9.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/compatibility.ini
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/content-prefs.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/cookies.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/crashes
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/crashes/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/datareporting
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/datareporting/archived
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/datareporting/archived/2026-09
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/datareporting/archived/2026-09/1788725413191.d4b76b97-220b-45d7-a66f-0fa394e1cfa8.deletion-request.jsonlz4
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/datareporting/glean
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/datareporting/glean/client_id.txt
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/datareporting/glean/db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/datareporting/glean/db/data.safe.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/datareporting/glean/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/datareporting/glean/events/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/datareporting/glean/pending_pings
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/datareporting/glean/tmp
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/datareporting/state.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/domain_to_categories.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/domain_to_categories.sqlite-journal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/extension-preferences.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/extension-store
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/extension-store/data.safe.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/extensions.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/favicons.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/favicons.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/key4.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/lock
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/logins.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/minidumps
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/permissions.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/pkcs11.txt
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/places.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/places.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/prefs.js
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/safebrowsing
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/saved-telemetry-pings
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/saved-telemetry-pings/d4b76b97-220b-45d7-a66f-0fa394e1cfa8
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/security_state
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/sessionCheckpoints.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/startupCache
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/startupCache/startupCache.8.little
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/default
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/ls-archive.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/.metadata-v2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/2918063365piupsah.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/2918063365piupsah.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/2918063365piupsah.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/3561288849sdhlie.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/3561288849sdhlie.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/3561288849sdhlie.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage/temporary
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/storage.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/thumbnails
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-1/times.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/.parentlock
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/.startup-incomplete
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/SiteSecurityServiceState.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/WebDriverBiDiServer.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/activity-stream.shortcut_cache.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/addonStartup.json.lz4
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/b3ee102a.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/b3ee102a.sqlite-shm
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/b3ee102a.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/bookmarkbackups
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/bounce-tracking-protection.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/cache2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/cache2/doomed
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/cache2/entries
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/cache2/entries/360DE1F3E174E84794C36651DD126686164A2F9A
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/cache2/entries/BB95D0607349D05725D5FE01D4FB300E319072AD
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/cert9.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/compatibility.ini
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/content-prefs.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/cookies.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/crashes
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/crashes/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/datareporting
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/datareporting/archived
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/datareporting/archived/2026-09
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/datareporting/archived/2026-09/1788725413981.80719d95-6321-43dd-9aab-edb9ed0ef34e.deletion-request.jsonlz4
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/datareporting/glean
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/datareporting/glean/client_id.txt
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/datareporting/glean/db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/datareporting/glean/db/data.safe.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/datareporting/glean/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/datareporting/glean/events/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/datareporting/glean/pending_pings
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/datareporting/glean/tmp
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/datareporting/state.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/domain_to_categories.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/domain_to_categories.sqlite-journal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/extension-store
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/extension-store/data.safe.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/extensions.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/favicons.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/favicons.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/key4.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/lock
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/logins.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/minidumps
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/permissions.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/pkcs11.txt
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/places.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/places.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/prefs.js
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/safebrowsing
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/saved-telemetry-pings
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/saved-telemetry-pings/80719d95-6321-43dd-9aab-edb9ed0ef34e
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/security_state
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/sessionCheckpoints.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/startupCache
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/startupCache/startupCache.8.little
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/default
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/ls-archive.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/.metadata-v2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/2918063365piupsah.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/2918063365piupsah.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/2918063365piupsah.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/3561288849sdhlie.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/3561288849sdhlie.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/3561288849sdhlie.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage/temporary
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/storage.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/thumbnails
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-2/times.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/.parentlock
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/.startup-incomplete
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/612a1ed7.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/612a1ed7.sqlite-shm
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/612a1ed7.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/SiteSecurityServiceState.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/WebDriverBiDiServer.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/activity-stream.shortcut_cache.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/bookmarkbackups
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/bounce-tracking-protection.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/cache2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/cache2/doomed
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/cache2/entries
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/cache2/entries/360DE1F3E174E84794C36651DD126686164A2F9A
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/cache2/entries/5E43012191E7B1F510ACD5BF039D8BF1B9A277B2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/cache2/entries/BB95D0607349D05725D5FE01D4FB300E319072AD
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/cert9.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/compatibility.ini
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/content-prefs.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/cookies.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/crashes
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/crashes/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/datareporting
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/datareporting/archived
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/datareporting/archived/2026-09
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/datareporting/archived/2026-09/1788725414904.4f34c269-16c0-4970-9108-ea534609d77e.deletion-request.jsonlz4
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/datareporting/glean
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/datareporting/glean/client_id.txt
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/datareporting/glean/db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/datareporting/glean/db/data.safe.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/datareporting/glean/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/datareporting/glean/events/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/datareporting/glean/pending_pings
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/datareporting/glean/tmp
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/datareporting/state.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/domain_to_categories.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/domain_to_categories.sqlite-journal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/extension-store
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/extension-store/data.safe.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/extensions.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/favicons.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/favicons.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/key4.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/lock
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/logins.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/minidumps
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/permissions.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/pkcs11.txt
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/places.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/places.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/prefs.js
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/safebrowsing
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/saved-telemetry-pings
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/saved-telemetry-pings/4f34c269-16c0-4970-9108-ea534609d77e
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/security_state
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/sessionCheckpoints.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/startupCache
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/startupCache/startupCache.8.little
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/default
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/ls-archive.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/.metadata-v2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/2918063365piupsah.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/2918063365piupsah.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/2918063365piupsah.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/3561288849sdhlie.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/3561288849sdhlie.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/3561288849sdhlie.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage/temporary
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/storage.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/thumbnails
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1138007-3/times.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/.parentlock
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/.startup-incomplete
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/SiteSecurityServiceState.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/WebDriverBiDiServer.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/a0b5a4c8.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/a0b5a4c8.sqlite-shm
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/a0b5a4c8.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/activity-stream.shortcut_cache.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/bookmarkbackups
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/bounce-tracking-protection.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/cache2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/cache2/doomed
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/cache2/entries
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/cache2/entries/360DE1F3E174E84794C36651DD126686164A2F9A
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/cache2/entries/5E43012191E7B1F510ACD5BF039D8BF1B9A277B2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/cache2/entries/BB95D0607349D05725D5FE01D4FB300E319072AD
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/cert9.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/compatibility.ini
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/content-prefs.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/cookies.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/crashes
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/crashes/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/datareporting
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/datareporting/archived
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/datareporting/archived/2026-09
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/datareporting/archived/2026-09/1788725428602.71e44463-3290-4ac7-9f22-07652440f4a0.deletion-request.jsonlz4
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/datareporting/glean
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/datareporting/glean/client_id.txt
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/datareporting/glean/db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/datareporting/glean/db/data.safe.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/datareporting/glean/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/datareporting/glean/events/events
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/datareporting/glean/pending_pings
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/datareporting/glean/tmp
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/datareporting/state.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/domain_to_categories.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/domain_to_categories.sqlite-journal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/extension-store
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/extension-store/data.safe.bin
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/extensions.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/favicons.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/favicons.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/key4.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/lock
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/logins.db
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/minidumps
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/permissions.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/pkcs11.txt
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/places.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/places.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/prefs.js
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/safebrowsing
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/saved-telemetry-pings
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/saved-telemetry-pings/71e44463-3290-4ac7-9f22-07652440f4a0
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/security_state
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/sessionCheckpoints.json
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/startupCache
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/startupCache/startupCache.8.little
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/default
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/ls-archive.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/.metadata-v2
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/1451318868ntouromlalnodry--epcr.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/1657114595AmcateirvtiSty.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/2918063365piupsah.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/2918063365piupsah.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/2918063365piupsah.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/3561288849sdhlie.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/3561288849sdhlie.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/3561288849sdhlie.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.files
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/permanent/chrome/idb/3870112724rsegmnoittet-es.sqlite-wal
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage/temporary
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/storage.sqlite
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/thumbnails
/home/timo/.cache/ess-w11-review1-tmp/ess-bidi-1147781-0/times.json
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
