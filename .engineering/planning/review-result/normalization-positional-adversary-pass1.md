---
format: aep.planning-md/1
id: review-result:normalization-positional-adversary-pass1
kind: review-result
status: active
title: Independent positional normalization review, pass 1
relations:
- reviews: story:normalize-positional-array-input
revision: 1
---
unit: story:normalize-positional-array-input, adversary pass 1, eb2e5d60e9e803993417df39563bc744dbcd36fc
verdict: nothing found
cases: executed 30→38 focused outer tests, red 0 implementation findings
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 authorized roots (assigned evidence/TMPDIR, Cargo cache, sccache cache)
needs-coordinator: none
git --no-pager diff --stat
```text
(empty output; exit 0)
```

All 979 coordinator-frozen tracked files retain their exact SHA256 and filesystem modes. The worktree has four new test-only files; there are no modified tracked files. The no-index addition statistics are:

```text
crates/edge/ess-cli/tests/normalization_positional_adversary.rs                         | 181 insertions
crates/generate/schema-contract/tests/fixtures/positional_adversary_go.go.txt         | 124 insertions
crates/generate/schema-contract/tests/fixtures/positional_adversary_rust.rs.txt       |  98 insertions
crates/generate/schema-contract/tests/normalization_positional_adversary.rs           | 447 insertions
4 new files, 850 insertions; all mode 100644
```

The first test was authored before any test command ran. Its isolated execution passed. No implementation defect was demonstrated by this pass; this report is agent review, not approval or independent verifier attestation.

1. **Attacks and reachability.** The binding is `docs/design/positional-array-normalization.md` at the frozen commit and the local story revision 12. The functional production diff, caller plumbing, emitted runtimes and implementation tests were read; the ten frozen templates were verified against their pre-edit hashes. The new tests use public `Plan::read`, `Plan::run_json`, `Plan::run`, generated `Normalizer` entrypoints and the actual CLI binary. Their schemas, recipes and 19 literal token cases are authored in the new test; the expected values/errors do not call the implementation under attack to construct an oracle.

   - `schema-contract/tests/normalization_positional_adversary.rs:419`: source-order discarded object traversal, depth 64/65, later invalid key precedence, duplicate escape-equivalent keys, huge grammatical exponents, and U+0000. Three explicit cases; the first isolated command is recorded below.
   - `schema-contract/tests/normalization_positional_adversary.rs:176`: 19 literal cases cross escaped `a/~` fields, nullable items/intermediate lists, missing terminal, whole-null/short arrays, selected-versus-discarded wrong kinds, grammar-first errors, duplicate containing fields and raw/Binary64/positional siblings. Every case is followed by a fresh empty-array call and unchanged-recipe assertion. Two decoded-value provenance refusals and a signed-zero bit assertion are additional controls.
   - `schema-contract/tests/normalization_positional_adversary.rs:220`: source tuple unions remain refused through `find`, a nested `position`, and nullable record alternatives. A separate exact nested heterogeneous tuple succeeds through pure decoded-value mapping. Existing tests also exercise open/variable tails, zero/huge arity, out-of-range indices, old formats, closed DTOs, overlaps and homogeneous-operation refusals.
   - `schema-contract/tests/normalization_positional_adversary.rs:262`: retained `uniqueItems` rejects duplicated zero padding, discarded duplicates do not enter validation, and a mismatched next-stage identity cannot smuggle a prepared root into a second decoder.
   - Generated Rust uses these same 19 literal cases under default and `serde_json/arbitrary_precision`; the Go lane uses the same cases with `-race`. Both exercise fresh state and signed zero. Additional native cases remove/change checked arity metadata in **test-owned memory** and call the public normalizer; the promised runtime guard refuses. Private map mutation is explicit fault injection, not a claim that an ordinary caller has authority to edit sealed metadata. Missing operands still propagate. Go additionally supplies three invalid-UTF-8 byte inputs and counts a missing operand's single evaluation. Retained-base64 success and base64-before-dispatch refusal are exercised.
   - `ess-cli/tests/normalization_positional_adversary.rs:71,112`: one successful source input and three runtime refusals preserve input bytes and an existing output sentinel. A huge declared arity blocks both target publications and recipe-output replacement. Valid Rust/Go emissions retain target report `/3`, pass drift checks, and later metadata-file drift is detected without rewriting the modified file. These CLI tests do not invoke Go or depend on `ESS_GO_COMPILER`.

   The abbreviated crate paths above are under `crates/generate/` for schema-contract and `crates/edge/` for ess-cli. The new native fixtures are `schema-contract/tests/fixtures/positional_adversary_{rust.rs,go.go}.txt`.

2. **First execution and intermediate failures.** `01-first-case.log` is the actual first isolated execution, exit 0, one test. `02-reference-cases.log` is an authored-test lifetime/temporary compile typo, exit 101, plus an unused assignment; both were corrected only in the new test. `03-reference-cases.log` contains one failed test expectation: a constructed cross-stage case was refused during checking with `stage_identity`, so the hypothesized runtime re-decoding path was unreachable. The final test asserts that admission refusal; it is not a source defect. `06-clippy.log` contains seven style findings in new test helpers/raw literals, corrected without lint allowances. No existing test or production file was changed for any correction.

3. **Counts and final commands.** The focused baseline is 13 existing schema-contract outer tests (12 positional plus one complete legacy-map test) and 17 existing CLI normalization tests, as identified in the implementor handoff and re-observed in the final focused logs. The additions are six schema-contract outer tests with the explicit Go feature and two CLI tests: 30→38. This is a selected-test count, not a new whole-package or workspace count. There was no preemptive baseline suite before the isolated case. The final schema command separately reports the existing groups and new group; generated tests are nested and not double-counted in the 38.

   Native Rust executes six tests per feature configuration: three newly authored native tests plus three inherited runtime unit tests; each run includes all 19 new literal vectors. Native Go executes three top-level tests and 19 literal subtests, plus three UTF-8 vectors and metadata/missing/retained controls. The final inherited reference positional test executes its existing 99 vectors and all 70 Binary64 vectors, including the existing three-stage model-root composition test. This pass does not claim another full inherited native corpus run or a whole-system runtime/conformance qualification.

   Final focused schema and CLI commands, scoped strict Clippy, package formatting check and all hash comparisons exit 0. No `task check`, site build, Git mutation, planning command, lifecycle action, integration call, process kill or tracked-source edit occurred. Tool versions: rustc 1.98.0 (88d9e12ae 2026-08-18); cargo 1.98.0 (797e8a9bc 2026-08-05); Go go1.26.5-X:nodwarf5 linux/amd64; sccache 0.16.0.

4. **Frozen bytes and limits.** `root-frozen-files.json` SHA256 `192b33351bcf95ced2805603fe38b43b25ae18e2b4bf582e39aec9736f1b6169` binds all 979 original files. `11-frozen-byte-check.log` and `14-frozen-mode-check.log` are empty successful comparisons. `12-legacy-raw-check.log` verifies all **173 raw files**, with no path or byte normalization, against the implementation's pre-edit baseline; the current output directory has exactly 173 files. This includes ten complete generated maps and their report/manifest/source files plus `canonical-maps.json`. The inherited committed map test normalizes only its producer-version field internally, so this separate same-generator raw comparison is the stronger byte witness. `13-template-check.log` verifies all ten frozen templates. `diff-check.log` is empty. These checks do not prove every possible decoder permissiveness or future release metadata equality.

5. **Final additions.** `test-manifest.json` SHA256 `2f7e6f70bf8852a0d006f600add619cd81924f692cf09c99387bdfcb5a7b1c45`; `additions.patch` SHA256 `26d1b52e2ed7b3a97c645357c254d035beabb77e49fc1312b76138a45f7c135c`.

   | New path | SHA256 |
   | --- | --- |
   | crates/edge/ess-cli/tests/normalization_positional_adversary.rs | 7d50badefbcc17afe8e3e6262c37394f0b691ffc3ecb098747630c0d37d12398 |
   | crates/generate/schema-contract/tests/fixtures/positional_adversary_go.go.txt | cadf38e33595080e0dc25ffde41ba75a80b4d34ae65e3bdffa5c4c64427108ee |
   | crates/generate/schema-contract/tests/fixtures/positional_adversary_rust.rs.txt | 239bd739f2c15b8597c4719a06530488ebbe00cfbe5b42e01ad6de3f616ed6cf |
   | crates/generate/schema-contract/tests/normalization_positional_adversary.rs | 5f58f52bfbfe2783ab60acbaeaa2218eec8d48e12a89bbbe4710b21a8b5adf2a |

6. **Writes and terminal state.** Repository additions are exactly the four files above. Generated targets and native caches remain within `$WORKTREE/target/`, including `target/tmp/positional-adversary-*`, the existing local native cache directories and `target/tmp/normalization-legacy-current`. All agent-authored writes outside the tree are beneath `$SCRATCH`, including `tmp/` as every build's TMPDIR. The full persistent evidence-path inventory appears below. The mandated sccache wrapper uses `$SCCACHE_DIR`; Cargo uses `$CARGO_HOME` for its tool-managed shared cache/locks. Individual shared cache entries were not attributed to this agent; that inventory limit is explicit. No unassigned scratch or `/tmp` was selected.

   Builds used `RUSTC_WRAPPER=/usr/bin/sccache CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 ESS_GO_COMPILER=/usr/bin/go`, with offline/locked workspace and native test commands. Native dependency locking used the existing offline `cargo generate-lockfile` convention before locked tests. Workspace `CARGO_TARGET_DIR` was not overridden; generated native Rust uses only the established cache inside this tree. Start free space was 19,656,175,616 bytes, the observed native checkpoint was 18,985,033,728 bytes, and final recorded target usage was 2,477,226,176 bytes; all checkpoints exceeded the 8 GiB reserve. All this agent's commands and native children are terminal. The lane was explicitly released to the coordinator before report writing. `15-terminal-processes.log` has no matching build/test process. No source changes occurred after the final test manifest was frozen.

7. **Exact commands and outputs.** The following blocks preserve the private raw log text. The separately publishable `report-public.md` changes only these explicitly declared path prefixes throughout the report: the assigned worktree becomes `$WORKTREE`, the assigned evidence directory becomes `$SCRATCH`, the Rust toolchain root becomes `$RUST_TOOLCHAIN`, and Cargo/sccache cache roots become `$CARGO_HOME`/`$SCCACHE_DIR`. No test result, diagnostic, count or source byte was normalized for a verification claim. `report-private.md` and the raw-log hash manifest retain the exact private paths and bytes.

01-first-case.log — exit 0

```console
cargo test --offline --locked -p schema-contract --test normalization_positional_adversary discarded_source_order_depth_precedes_later_invalid_key -- --exact --nocapture
```

```text
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.31s
     Running tests/normalization_positional_adversary.rs (target/debug/deps/normalization_positional_adversary-a03f124226f15b31)

running 1 test
test discarded_source_order_depth_precedes_later_invalid_key ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


```

02-reference-cases.log — exit 101

```console
cargo test --offline --locked -p schema-contract --test normalization_positional_adversary -- --nocapture
```

```text
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
warning: value assigned to `bundle` is never read
  --> crates/generate/schema-contract/tests/normalization_positional_adversary.rs:50:10
   |
50 |     let (mut bundle, _) = fixture(record.clone(), read(&[]), vec![]);
   |          ^^^^^^^^^^ this value is reassigned later and never used
51 |     let source = json!({"components":{"schemas":{"Input":record,"Pair":tuple(),"Output":true}}});
52 |     bundle = import(&source.to_string(), &["Input".to_owned(),"Pair".to_owned(),"Output".to_owned()].into_iter().collect(), Dialect:...
   |     ------ `bundle` is overwritten here before the previous value is read
   |
   = note: `#[warn(unused_assignments)]` (part of `#[warn(unused)]`) on by default

error[E0716]: temporary value dropped while borrowed
  --> crates/generate/schema-contract/tests/normalization_positional_adversary.rs:89:33
   |
89 | ...sh(failure("run/~",&format!(r#"[null,null,{{"z":{deep},"\ud800":0}}]"#),"/input/2","input_depth","JSON input exceeds 64 levels"));
   |                        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^                                                          - temporary value is freed at the end of this statement
   |                        |
   |                        creates a temporary value which is freed while still in use
90 | ...sh(failure("run/~",&format!(r#"[null,null,{{"\ud800":0,"z":{deep}}}]"#),"/input/2","input_syntax","discarded positional token contain...
   |       ------- borrow later used here
   |
   = note: consider using a `let` binding to create a longer lived value

For more information about this error, try `rustc --explain E0716`.
warning: `schema-contract` (test "normalization_positional_adversary") generated 1 warning
error: could not compile `schema-contract` (test "normalization_positional_adversary") due to 1 previous error; 1 warning emitted

```

03-reference-cases.log — exit 101

```console
cargo test --offline --locked -p schema-contract --test normalization_positional_adversary -- --nocapture
```

```text
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.37s
     Running tests/normalization_positional_adversary.rs (target/debug/deps/normalization_positional_adversary-a03f124226f15b31)

running 4 tests
test discarded_source_order_depth_precedes_later_invalid_key ... ok

thread 'source_refinements_and_second_stage_are_not_decoder_defaults' (2321290) panicked at crates/generate/schema-contract/tests/normalization_positional_adversary.rs:147:57:
called `Result::unwrap()` on an `Err` value: Refused([Finding { pointer: "/branches/run~1~0/1/input", rule: "stage_identity", detail: "input identity differs from the previous output" }])
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test source_refinements_and_second_stage_are_not_decoder_defaults ... FAILED
test source_union_proofs_survive_computed_collection_and_nested_position ... ok
independent reference: 19 literal vectors, repeated fresh-state controls, two value-API refusals, signed-zero bits
test literal_cross_boundary_cases_and_repeated_calls_are_atomic ... ok

failures:

failures:
    source_refinements_and_second_stage_are_not_decoder_defaults

test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p schema-contract --test normalization_positional_adversary`

```

04-native-cases.log — exit 0

```console
cargo test --offline --locked -p schema-contract --features go-typecheck --test normalization_positional_adversary -- --nocapture --test-threads=1
```

```text
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 0.47s
     Running tests/normalization_positional_adversary.rs (target/debug/deps/normalization_positional_adversary-18b9fb164c81917f)

running 6 tests
test discarded_source_order_depth_precedes_later_invalid_key ... ok
test literal_cross_boundary_cases_and_repeated_calls_are_atomic ... independent reference: 19 literal vectors, repeated fresh-state controls, two value-API refusals, signed-zero bits
ok
test native_go_preserves_independent_token_cases_and_checked_metadata ... command: cd "$WORKTREE/target/tmp/positional-adversary-go-2335893" && GOCACHE="$WORKTREE/target/tmp/positional-go-cache" GOFLAGS="" GOMAXPROCS="4" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" "/usr/bin/go" "test" "-v" "-count=1" "-race" "-mod=readonly" "-p" "1" "./..."
exit: exit status: 0
=== RUN   TestIndependentLiteralTokens
=== RUN   TestIndependentLiteralTokens/0
=== RUN   TestIndependentLiteralTokens/1
=== RUN   TestIndependentLiteralTokens/2
=== RUN   TestIndependentLiteralTokens/3
=== RUN   TestIndependentLiteralTokens/4
=== RUN   TestIndependentLiteralTokens/5
=== RUN   TestIndependentLiteralTokens/6
=== RUN   TestIndependentLiteralTokens/7
=== RUN   TestIndependentLiteralTokens/8
=== RUN   TestIndependentLiteralTokens/9
=== RUN   TestIndependentLiteralTokens/10
=== RUN   TestIndependentLiteralTokens/11
=== RUN   TestIndependentLiteralTokens/12
=== RUN   TestIndependentLiteralTokens/13
=== RUN   TestIndependentLiteralTokens/14
=== RUN   TestIndependentLiteralTokens/15
=== RUN   TestIndependentLiteralTokens/16
=== RUN   TestIndependentLiteralTokens/17
=== RUN   TestIndependentLiteralTokens/18
--- PASS: TestIndependentLiteralTokens (0.01s)
    --- PASS: TestIndependentLiteralTokens/0 (0.00s)
    --- PASS: TestIndependentLiteralTokens/1 (0.00s)
    --- PASS: TestIndependentLiteralTokens/2 (0.00s)
    --- PASS: TestIndependentLiteralTokens/3 (0.00s)
    --- PASS: TestIndependentLiteralTokens/4 (0.00s)
    --- PASS: TestIndependentLiteralTokens/5 (0.00s)
    --- PASS: TestIndependentLiteralTokens/6 (0.00s)
    --- PASS: TestIndependentLiteralTokens/7 (0.00s)
    --- PASS: TestIndependentLiteralTokens/8 (0.00s)
    --- PASS: TestIndependentLiteralTokens/9 (0.00s)
    --- PASS: TestIndependentLiteralTokens/10 (0.00s)
    --- PASS: TestIndependentLiteralTokens/11 (0.00s)
    --- PASS: TestIndependentLiteralTokens/12 (0.00s)
    --- PASS: TestIndependentLiteralTokens/13 (0.00s)
    --- PASS: TestIndependentLiteralTokens/14 (0.00s)
    --- PASS: TestIndependentLiteralTokens/15 (0.00s)
    --- PASS: TestIndependentLiteralTokens/16 (0.00s)
    --- PASS: TestIndependentLiteralTokens/17 (0.00s)
    --- PASS: TestIndependentLiteralTokens/18 (0.00s)
=== RUN   TestCheckedMetadataFaultAndMissingOperand
--- PASS: TestCheckedMetadataFaultAndMissingOperand (0.00s)
=== RUN   TestInvalidUtf8KindsAndRetainedOrder
--- PASS: TestInvalidUtf8KindsAndRetainedOrder (0.00s)
PASS
ok  	example.invalid/positional-adversary	1.035s


ok
test native_rust_preserves_independent_token_cases_and_checked_metadata ... command: "$RUST_TOOLCHAIN/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline" "--manifest-path" "$WORKTREE/target/tmp/positional-adversary-rust-2335893/Cargo.toml"
exit: exit status: 0

     Locking 80 packages to latest compatible versions
      Adding base64 v0.22.1 (available: v0.23.1)
      Adding jsonschema v0.52.1 (available: v0.53.0)

command: CARGO_TARGET_DIR="$WORKTREE/target/tmp/normalization-rust-target" "$RUST_TOOLCHAIN/stable-x86_64-unknown-linux-gnu/bin/cargo" "test" "--offline" "--locked" "--manifest-path" "$WORKTREE/target/tmp/positional-adversary-rust-2335893/Cargo.toml" "--" "--nocapture" "--test-threads=1"
exit: exit status: 0

running 6 tests
test eval::positional_defense::checked_arity_is_required_for_present_values_and_missing_propagates ... ok
test independent_adversary::caller_reaches_guard_after_private_metadata_fault_injection ... ok
test independent_adversary::decoded_value_provenance_and_retained_entry_order ... ok
test independent_adversary::literal_tokens_and_fresh_state ... ok
test numeric::tests::rounding_matches_independently_observed_binary64_bits ... ok
test numeric::tests::zero_clamps_are_sign_stable_in_either_operand_order ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


   Compiling positional_adversary v0.0.0 ($WORKTREE/target/tmp/positional-adversary-rust-2335893)
    Finished `test` profile [unoptimized] target(s) in 1.45s
     Running unittests src/lib.rs ($WORKTREE/target/tmp/normalization-rust-target/debug/deps/positional_adversary-e27a05ad123051e0)
independent native Rust literal vectors: 19
   Doc-tests positional_adversary

command: CARGO_TARGET_DIR="$WORKTREE/target/tmp/normalization-rust-target" "$RUST_TOOLCHAIN/stable-x86_64-unknown-linux-gnu/bin/cargo" "test" "--offline" "--locked" "--manifest-path" "$WORKTREE/target/tmp/positional-adversary-rust-2335893/Cargo.toml" "--features" "serde_json/arbitrary_precision" "--" "--nocapture" "--test-threads=1"
exit: exit status: 0

running 6 tests
test eval::positional_defense::checked_arity_is_required_for_present_values_and_missing_propagates ... ok
test independent_adversary::caller_reaches_guard_after_private_metadata_fault_injection ... ok
test independent_adversary::decoded_value_provenance_and_retained_entry_order ... ok
test independent_adversary::literal_tokens_and_fresh_state ... ok
test numeric::tests::rounding_matches_independently_observed_binary64_bits ... ok
test numeric::tests::zero_clamps_are_sign_stable_in_either_operand_order ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


   Compiling positional_adversary v0.0.0 ($WORKTREE/target/tmp/positional-adversary-rust-2335893)
    Finished `test` profile [unoptimized] target(s) in 1.55s
     Running unittests src/lib.rs ($WORKTREE/target/tmp/normalization-rust-target/debug/deps/positional_adversary-4f6db6c6241a9f39)
independent native Rust literal vectors: 19
   Doc-tests positional_adversary

ok
test source_refinements_and_stage_identity_remain_checked ... ok
test source_union_proofs_survive_computed_collection_and_nested_position ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.02s


```

05-cli-cases.log — exit 0

```console
cargo test --offline --locked -p ess-cli --test normalization_positional_adversary -- --nocapture --test-threads=1
```

```text
   Compiling ess-cli v0.19.0 ($WORKTREE/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.36s
     Running tests/normalization_positional_adversary.rs (target/debug/deps/normalization_positional_adversary-04c8fc26cdea59df)

running 2 tests
test cli_invalid_policy_blocks_all_publication_and_valid_metadata_is_drift_checked ... normalize-generate ["--target", "rust", "--package", "adversary", "--out", "rust"]: exit exit status: 1
stdout: 
stderr: error: checking normalization recipe recipe.json: /positional_inputs/pick~1~0/0/path: positional_schema: fixed_string_array requires an exact closed tuple of the declared length with string-shaped positions


normalize-generate ["--target", "go", "--package", "adversary", "--out", "go", "--module", "example.invalid/adversary"]: exit exit status: 1
stdout: 
stderr: error: checking normalization recipe recipe.json: /positional_inputs/pick~1~0/0/path: positional_schema: fixed_string_array requires an exact closed tuple of the declared length with string-shaped positions


normalize-check ["--out", "sentinel.json"]: exit exit status: 1
stdout: 
stderr: error: checking normalization recipe recipe.json: /positional_inputs/pick~1~0/0/path: positional_schema: fixed_string_array requires an exact closed tuple of the declared length with string-shaped positions


normalize-generate ["--target", "rust", "--package", "adversary", "--out", "rust"]: exit exit status: 0
stdout: 15 file(s), written to rust; provenance in normalization-report.json

stderr: 
normalize-generate ["--target", "rust", "--package", "adversary", "--out", "rust", "--check"]: exit exit status: 0
stdout: 15 file(s): current

stderr: 
normalize-generate ["--target", "rust", "--package", "adversary", "--out", "rust", "--check"]: exit exit status: 1
stdout: src/schemas.rs: stale

stderr: 
normalize-generate ["--target", "go", "--package", "adversary", "--out", "go", "--module", "example.invalid/adversary"]: exit exit status: 0
stdout: 15 file(s), written to go; provenance in normalization-report.json

stderr: 
normalize-generate ["--target", "go", "--package", "adversary", "--out", "go", "--module", "example.invalid/adversary", "--check"]: exit exit status: 0
stdout: 15 file(s): current

stderr: 
normalize-generate ["--target", "go", "--package", "adversary", "--out", "go", "--module", "example.invalid/adversary", "--check"]: exit exit status: 1
stdout: bindings.go: stale

stderr: 
ok
test cli_runtime_grammar_controls_preserve_existing_output ... normalize-run ["--branch", "pick/~", "--input", "input.json"]: exit exit status: 0
stdout: "literal"

stderr: 
normalize-run ["--branch", "pick/~", "--input", "input.json", "--out", "sentinel.json"]: exit exit status: 1
stdout: 
stderr: error: /input: input_syntax: expected one complete JSON value


normalize-run ["--branch", "pick/~", "--input", "input.json", "--out", "sentinel.json"]: exit exit status: 1
stdout: 
stderr: error: /input/1: positional_element_type: fixed_string_array element must be a string or null


normalize-run ["--branch", "pick/~", "--input", "input.json", "--out", "sentinel.json"]: exit exit status: 1
stdout: 
stderr: error: /input/2: input_syntax: discarded positional token contains invalid Unicode


ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s


```

06-clippy.log — exit 101

```console
cargo clippy --offline --locked -p schema-contract -p ess-cli --test normalization_positional_adversary --features schema-contract/go-typecheck -- -D warnings
```

```text
    Checking schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Checking ess-cli v0.19.0 ($WORKTREE/crates/edge/ess-cli)
error: unnecessary hashes around raw string literal
   --> crates/generate/schema-contract/tests/normalization_positional_adversary.rs:116:13
    |
116 |             r#"[false,null] []"#,
    |             ^^^^^^^^^^^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#needless_raw_string_hashes
    = note: `-D clippy::needless-raw-string-hashes` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::needless_raw_string_hashes)]`
help: remove all the hashes around the string literal
    |
116 -             r#"[false,null] []"#,
116 +             r"[false,null] []",
    |

error: unnecessary hashes around raw string literal
   --> crates/generate/schema-contract/tests/normalization_positional_adversary.rs:123:13
    |
123 |             r#"[false,null,]"#,
    |             ^^^^^^^^^^^^^^^^^^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#needless_raw_string_hashes
help: remove all the hashes around the string literal
    |
123 -             r#"[false,null,]"#,
123 +             r"[false,null,]",
    |

error: this argument is passed by value, but not consumed in the function body
  --> crates/generate/schema-contract/tests/normalization_positional_adversary.rs:15:17
   |
15 | fn policy(path: Value) -> Value {
   |                 ^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#needless_pass_by_value
   = note: `-D clippy::needless-pass-by-value` implied by `-D warnings`
   = help: to override `-D warnings` add `#[allow(clippy::needless_pass_by_value)]`
help: consider taking a reference instead
   |
15 | fn policy(path: &Value) -> Value {
   |                 +

error: this argument is passed by value, but not consumed in the function body
  --> crates/generate/schema-contract/tests/normalization_positional_adversary.rs:19:19
   |
19 | fn fixture(input: Value, value: Value, policies: Vec<Value>) -> (Bundle, Value) {
   |                   ^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#needless_pass_by_value
help: consider taking a reference instead
   |
19 | fn fixture(input: &Value, value: Value, policies: Vec<Value>) -> (Bundle, Value) {
   |                   +

error: this argument is passed by value, but not consumed in the function body
  --> crates/generate/schema-contract/tests/normalization_positional_adversary.rs:19:33
   |
19 | fn fixture(input: Value, value: Value, policies: Vec<Value>) -> (Bundle, Value) {
   |                                 ^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#needless_pass_by_value
help: consider taking a reference instead
   |
19 | fn fixture(input: Value, value: &Value, policies: Vec<Value>) -> (Bundle, Value) {
   |                                 +

error: this argument is passed by value, but not consumed in the function body
  --> crates/generate/schema-contract/tests/normalization_positional_adversary.rs:19:50
   |
19 | fn fixture(input: Value, value: Value, policies: Vec<Value>) -> (Bundle, Value) {
   |                                                  ^^^^^^^^^^ help: consider changing the type to: `&[Value]`
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#needless_pass_by_value

error: this argument is passed by value, but not consumed in the function body
  --> crates/generate/schema-contract/tests/normalization_positional_adversary.rs:42:20
   |
42 | fn position(value: Value, index: u64) -> Value {
   |                    ^^^^^
   |
   = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.98.0/index.html#needless_pass_by_value
help: consider taking a reference instead
   |
42 | fn position(value: &Value, index: u64) -> Value {
   |                    +

error: could not compile `schema-contract` (test "normalization_positional_adversary") due to 7 previous errors

```

07-clippy-final.log — exit 0

```console
cargo clippy --offline --locked -p schema-contract -p ess-cli --test normalization_positional_adversary --features schema-contract/go-typecheck -- -D warnings
```

```text
    Checking schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `dev` profile [unoptimized] target(s) in 0.16s

```

08-focused-schema-final.log — exit 0

```console
cargo test --offline --locked -p schema-contract --features go-typecheck --test normalization_positional --test normalization_legacy_bytes --test normalization_positional_adversary -- --nocapture --test-threads=1
```

```text
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized] target(s) in 1.38s
     Running tests/normalization_legacy_bytes.rs (target/debug/deps/normalization_legacy_bytes-d72e49b21daca1f9)

running 1 test
test complete_legacy_file_maps_are_preserved ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/normalization_positional.rs (target/debug/deps/normalization_positional-b9796849258de23f)

running 12 tests
test format_six_inherits_all_existing_binary64_vectors ... format6 inherited Binary64 corpus executed 70 cases
ok
test lexical_preparation_composes_with_three_modeled_binary64_stages ... ok
test old_formats_refuse_even_empty_positional_declarations ... ok
test paths_keep_exact_conflict_order ... ok
test policy_and_position_require_exact_tuples_without_guessing ... ok
test positional_corpus_has_independent_expected_values_and_findings ... positional corpus executed 99 cases
ok
test positional_declarations_are_closed ... ok
test pure_tuple_mapping_does_not_require_original_text ... ok
test refinements_and_requiredness_run_after_preparation ... ok
test source_tuple_alternatives_are_not_erased_by_member_or_fallback_typing ... ok
test tuple_reads_preserve_type_and_do_not_join_homogeneous_operations ... ok
test typed_construction_and_old_readers_cannot_bypass_version_or_path_checks ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s

     Running tests/normalization_positional_adversary.rs (target/debug/deps/normalization_positional_adversary-18b9fb164c81917f)

running 6 tests
test discarded_source_order_depth_precedes_later_invalid_key ... ok
test literal_cross_boundary_cases_and_repeated_calls_are_atomic ... independent reference: 19 literal vectors, repeated fresh-state controls, two value-API refusals, signed-zero bits
ok
test native_go_preserves_independent_token_cases_and_checked_metadata ... command: cd "$WORKTREE/target/tmp/positional-adversary-go-2364257" && GOCACHE="$WORKTREE/target/tmp/positional-go-cache" GOFLAGS="" GOMAXPROCS="4" GOPROXY="off" GOSUMDB="off" GOTOOLCHAIN="local" "/usr/bin/go" "test" "-v" "-count=1" "-race" "-mod=readonly" "-p" "1" "./..."
exit: exit status: 0
=== RUN   TestIndependentLiteralTokens
=== RUN   TestIndependentLiteralTokens/0
=== RUN   TestIndependentLiteralTokens/1
=== RUN   TestIndependentLiteralTokens/2
=== RUN   TestIndependentLiteralTokens/3
=== RUN   TestIndependentLiteralTokens/4
=== RUN   TestIndependentLiteralTokens/5
=== RUN   TestIndependentLiteralTokens/6
=== RUN   TestIndependentLiteralTokens/7
=== RUN   TestIndependentLiteralTokens/8
=== RUN   TestIndependentLiteralTokens/9
=== RUN   TestIndependentLiteralTokens/10
=== RUN   TestIndependentLiteralTokens/11
=== RUN   TestIndependentLiteralTokens/12
=== RUN   TestIndependentLiteralTokens/13
=== RUN   TestIndependentLiteralTokens/14
=== RUN   TestIndependentLiteralTokens/15
=== RUN   TestIndependentLiteralTokens/16
=== RUN   TestIndependentLiteralTokens/17
=== RUN   TestIndependentLiteralTokens/18
--- PASS: TestIndependentLiteralTokens (0.01s)
    --- PASS: TestIndependentLiteralTokens/0 (0.00s)
    --- PASS: TestIndependentLiteralTokens/1 (0.00s)
    --- PASS: TestIndependentLiteralTokens/2 (0.00s)
    --- PASS: TestIndependentLiteralTokens/3 (0.00s)
    --- PASS: TestIndependentLiteralTokens/4 (0.00s)
    --- PASS: TestIndependentLiteralTokens/5 (0.00s)
    --- PASS: TestIndependentLiteralTokens/6 (0.00s)
    --- PASS: TestIndependentLiteralTokens/7 (0.00s)
    --- PASS: TestIndependentLiteralTokens/8 (0.00s)
    --- PASS: TestIndependentLiteralTokens/9 (0.00s)
    --- PASS: TestIndependentLiteralTokens/10 (0.00s)
    --- PASS: TestIndependentLiteralTokens/11 (0.00s)
    --- PASS: TestIndependentLiteralTokens/12 (0.00s)
    --- PASS: TestIndependentLiteralTokens/13 (0.00s)
    --- PASS: TestIndependentLiteralTokens/14 (0.00s)
    --- PASS: TestIndependentLiteralTokens/15 (0.00s)
    --- PASS: TestIndependentLiteralTokens/16 (0.00s)
    --- PASS: TestIndependentLiteralTokens/17 (0.00s)
    --- PASS: TestIndependentLiteralTokens/18 (0.00s)
=== RUN   TestCheckedMetadataFaultAndMissingOperand
--- PASS: TestCheckedMetadataFaultAndMissingOperand (0.00s)
=== RUN   TestInvalidUtf8KindsAndRetainedOrder
--- PASS: TestInvalidUtf8KindsAndRetainedOrder (0.00s)
PASS
ok  	example.invalid/positional-adversary	1.036s


ok
test native_rust_preserves_independent_token_cases_and_checked_metadata ... command: "$RUST_TOOLCHAIN/stable-x86_64-unknown-linux-gnu/bin/cargo" "generate-lockfile" "--offline" "--manifest-path" "$WORKTREE/target/tmp/positional-adversary-rust-2364257/Cargo.toml"
exit: exit status: 0

     Locking 80 packages to latest compatible versions
      Adding base64 v0.22.1 (available: v0.23.1)
      Adding jsonschema v0.52.1 (available: v0.53.0)

command: CARGO_TARGET_DIR="$WORKTREE/target/tmp/normalization-rust-target" "$RUST_TOOLCHAIN/stable-x86_64-unknown-linux-gnu/bin/cargo" "test" "--offline" "--locked" "--manifest-path" "$WORKTREE/target/tmp/positional-adversary-rust-2364257/Cargo.toml" "--" "--nocapture" "--test-threads=1"
exit: exit status: 0

running 6 tests
test eval::positional_defense::checked_arity_is_required_for_present_values_and_missing_propagates ... ok
test independent_adversary::caller_reaches_guard_after_private_metadata_fault_injection ... ok
test independent_adversary::decoded_value_provenance_and_retained_entry_order ... ok
test independent_adversary::literal_tokens_and_fresh_state ... ok
test numeric::tests::rounding_matches_independently_observed_binary64_bits ... ok
test numeric::tests::zero_clamps_are_sign_stable_in_either_operand_order ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


   Compiling positional_adversary v0.0.0 ($WORKTREE/target/tmp/positional-adversary-rust-2364257)
    Finished `test` profile [unoptimized] target(s) in 1.23s
     Running unittests src/lib.rs ($WORKTREE/target/tmp/normalization-rust-target/debug/deps/positional_adversary-e27a05ad123051e0)
independent native Rust literal vectors: 19
   Doc-tests positional_adversary

command: CARGO_TARGET_DIR="$WORKTREE/target/tmp/normalization-rust-target" "$RUST_TOOLCHAIN/stable-x86_64-unknown-linux-gnu/bin/cargo" "test" "--offline" "--locked" "--manifest-path" "$WORKTREE/target/tmp/positional-adversary-rust-2364257/Cargo.toml" "--features" "serde_json/arbitrary_precision" "--" "--nocapture" "--test-threads=1"
exit: exit status: 0

running 6 tests
test eval::positional_defense::checked_arity_is_required_for_present_values_and_missing_propagates ... ok
test independent_adversary::caller_reaches_guard_after_private_metadata_fault_injection ... ok
test independent_adversary::decoded_value_provenance_and_retained_entry_order ... ok
test independent_adversary::literal_tokens_and_fresh_state ... ok
test numeric::tests::rounding_matches_independently_observed_binary64_bits ... ok
test numeric::tests::zero_clamps_are_sign_stable_in_either_operand_order ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


   Compiling positional_adversary v0.0.0 ($WORKTREE/target/tmp/positional-adversary-rust-2364257)
    Finished `test` profile [unoptimized] target(s) in 1.28s
     Running unittests src/lib.rs ($WORKTREE/target/tmp/normalization-rust-target/debug/deps/positional_adversary-4f6db6c6241a9f39)
independent native Rust literal vectors: 19
   Doc-tests positional_adversary

ok
test source_refinements_and_stage_identity_remain_checked ... ok
test source_union_proofs_survive_computed_collection_and_nested_position ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.47s


```

09-focused-cli-final.log — exit 0

```console
cargo test --offline --locked -p ess-cli --test normalization --test normalization_positional_adversary -- --nocapture --test-threads=1
```

```text
   Compiling ess-cli v0.19.0 ($WORKTREE/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.34s
     Running tests/normalization.rs (target/debug/deps/normalization-ac90639e6bf9bab8)

running 17 tests
test binary64_cli_keeps_numeric_identity_and_emits_checked_format_five ... CLI executed 70 independent Binary64 vectors and both target publication/drift checks
ok
test checked_recipe_and_run_are_deterministic_with_json_only_stdout ... ok
test explicit_binary64_inputs_run_through_the_cli_without_weakening_version_one ... ok
test failed_check_or_execution_preserves_output_and_does_not_emit_a_result ... ok
test generated_libraries_match_the_api_and_drift_check_never_repairs_files ... ok
test generated_paths_cannot_replace_canonical_source_inputs_or_follow_links ... ok
test generation_checks_refuse_before_creating_or_changing_destinations ... ok
test incompatible_later_destination_preserves_the_entire_existing_output ... ok
test model_sources_are_compiled_pinned_and_protected_by_the_cli ... ok
test output_cannot_replace_any_declared_input ... ok
test output_symlinks_and_hardlinks_are_refused_without_mutation ... ok
test positional_cli_prepares_text_and_refuses_before_publication ... CLI executed 8 positional text vectors and both publication/drift targets
ok
test positional_cli_refuses_unused_branches_before_publication ... CLI executed 4 unused-branch pre-publication refusals
ok
test qualified_go_base64_pattern_is_published_without_decoding_the_value ... ok
test raw_json_capture_runs_before_schema_and_keeps_failure_output_untouched ... ok
test stale_bundle_missing_source_and_unknown_dispatch_refuse ... ok
test unsupported_go_pattern_has_no_successful_partial_artifact ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.92s

     Running tests/normalization_positional_adversary.rs (target/debug/deps/normalization_positional_adversary-04c8fc26cdea59df)

running 2 tests
test cli_invalid_policy_blocks_all_publication_and_valid_metadata_is_drift_checked ... normalize-generate ["--target", "rust", "--package", "adversary", "--out", "rust"]: exit exit status: 1
stdout: 
stderr: error: checking normalization recipe recipe.json: /positional_inputs/pick~1~0/0/path: positional_schema: fixed_string_array requires an exact closed tuple of the declared length with string-shaped positions


normalize-generate ["--target", "go", "--package", "adversary", "--out", "go", "--module", "example.invalid/adversary"]: exit exit status: 1
stdout: 
stderr: error: checking normalization recipe recipe.json: /positional_inputs/pick~1~0/0/path: positional_schema: fixed_string_array requires an exact closed tuple of the declared length with string-shaped positions


normalize-check ["--out", "sentinel.json"]: exit exit status: 1
stdout: 
stderr: error: checking normalization recipe recipe.json: /positional_inputs/pick~1~0/0/path: positional_schema: fixed_string_array requires an exact closed tuple of the declared length with string-shaped positions


normalize-generate ["--target", "rust", "--package", "adversary", "--out", "rust"]: exit exit status: 0
stdout: 15 file(s), written to rust; provenance in normalization-report.json

stderr: 
normalize-generate ["--target", "rust", "--package", "adversary", "--out", "rust", "--check"]: exit exit status: 0
stdout: 15 file(s): current

stderr: 
normalize-generate ["--target", "rust", "--package", "adversary", "--out", "rust", "--check"]: exit exit status: 1
stdout: src/schemas.rs: stale

stderr: 
normalize-generate ["--target", "go", "--package", "adversary", "--out", "go", "--module", "example.invalid/adversary"]: exit exit status: 0
stdout: 15 file(s), written to go; provenance in normalization-report.json

stderr: 
normalize-generate ["--target", "go", "--package", "adversary", "--out", "go", "--module", "example.invalid/adversary", "--check"]: exit exit status: 0
stdout: 15 file(s): current

stderr: 
normalize-generate ["--target", "go", "--package", "adversary", "--out", "go", "--module", "example.invalid/adversary", "--check"]: exit exit status: 1
stdout: bindings.go: stale

stderr: 
ok
test cli_runtime_grammar_controls_preserve_existing_output ... normalize-run ["--branch", "pick/~", "--input", "input.json"]: exit exit status: 0
stdout: "literal"

stderr: 
normalize-run ["--branch", "pick/~", "--input", "input.json", "--out", "sentinel.json"]: exit exit status: 1
stdout: 
stderr: error: /input: input_syntax: expected one complete JSON value


normalize-run ["--branch", "pick/~", "--input", "input.json", "--out", "sentinel.json"]: exit exit status: 1
stdout: 
stderr: error: /input/1: positional_element_type: fixed_string_array element must be a string or null


normalize-run ["--branch", "pick/~", "--input", "input.json", "--out", "sentinel.json"]: exit exit status: 1
stdout: 
stderr: error: /input/2: input_syntax: discarded positional token contains invalid Unicode


ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s


```

10-format-final.log — exit 0

```console
cargo fmt --package schema-contract --package ess-cli -- --check
```

```text
(empty output)

```

11-frozen-byte-check.log — exit 0

```console
sha256sum --check --quiet $SCRATCH/frozen-check.sha256
```

```text
(empty output)

```

12-legacy-raw-check.log — exit 0

```console
sha256sum --check --quiet $SCRATCH/legacy-check.sha256
```

```text
(empty output)

```

13-template-check.log — exit 0

```console
sha256sum --check --quiet $SCRATCH/template-check.sha256
```

```text
(empty output)

```

14-frozen-mode-check.log — exit 0

```console
For every root-frozen-files.json entry: stat -c %a, convert octal to decimal, compare to its recorded mode; no mismatches
```

```text
(empty output)

```

15-terminal-processes.log — exit 0

```console
ps -eo pid,ppid,args filtered to assigned-worktree cargo/rustc/generated-test/go-test processes, excluding the observation shell; no matches
```

```text
(empty output)

```

Persistent evidence paths (the coordinator-created root-frozen-files.json was read, not rewritten):

```text
$SCRATCH/01-first-case.log
$SCRATCH/02-reference-cases.log
$SCRATCH/03-reference-cases.log
$SCRATCH/04-native-cases.log
$SCRATCH/05-cli-cases.log
$SCRATCH/06-clippy.log
$SCRATCH/07-clippy-final.log
$SCRATCH/08-focused-schema-final.log
$SCRATCH/09-focused-cli-final.log
$SCRATCH/10-format-final.log
$SCRATCH/11-frozen-byte-check.log
$SCRATCH/12-legacy-raw-check.log
$SCRATCH/13-template-check.log
$SCRATCH/14-frozen-mode-check.log
$SCRATCH/15-terminal-processes.log
$SCRATCH/additions-stat.log
$SCRATCH/additions.patch
$SCRATCH/commands.json
$SCRATCH/diff-check.log
$SCRATCH/diff-stat.log
$SCRATCH/final-artifacts.sha256
$SCRATCH/frozen-check.sha256
$SCRATCH/legacy-check.sha256
$SCRATCH/outside-writes.txt
$SCRATCH/raw-log-manifest.json
$SCRATCH/report-private.md
$SCRATCH/report-public.md
$SCRATCH/root-frozen-files.json
$SCRATCH/status.log
$SCRATCH/template-check.sha256
$SCRATCH/test-manifest.json
$SCRATCH/tool-and-disk.log
```

```findings
[]
```

