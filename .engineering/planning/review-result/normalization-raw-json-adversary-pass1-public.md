---
format: aep.planning-md/1
id: review-result:normalization-raw-json-adversary-pass1-public
kind: review-result
status: active
title: Raw JSON normalization adversary pass 1
owner: aep-drive:adversary
relations:
- reviews: story:raw-json-normalization-provenance
revision: 1
---
unit: story:raw-json-normalization-provenance; pass 1; base/HEAD 60ffcb2238ffef3a48d0db9555b6f2ca709ca2f7; working-tree manifest SHA-256 a8415a4ff0d688840ec384000a5393b1d6ccb6d980e1ad4ae66bf83d23a4a6e9
verdict: nothing found
cases: executed 9→13, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 23 paths (19 named scratch files, scratch/TMPDIR and 3 build/tool cache roots; descendants listed below)
needs-coordinator: none

1. `git --no-pager diff --stat`

```text
 CHANGELOG.md                                       |  8 +++
 Cargo.lock                                         |  7 ++
 crates/edge/ess-cli/tests/normalization.rs         | 53 ++++++++++++++++
 crates/generate/schema-contract/Cargo.toml         |  1 +
 .../schema-contract/src/realize/normalize.rs       | 51 +++++++++++++--
 .../schema-contract/src/realize/normalize/check.rs | 74 ++++++++++++++++++++++
 .../src/realize/normalize/go_input.go.txt          | 74 +++++++++++++++++++---
 .../src/realize/normalize/go_runtime.go.txt        |  2 +-
 .../src/realize/normalize/go_target.rs             | 65 +++++++++++++++----
 .../schema-contract/src/realize/normalize/input.rs | 29 +++++++--
 .../src/realize/normalize/recipe.rs                | 58 ++++++++++++++++-
 .../src/realize/normalize/rust_runtime.rs.txt      | 11 +++-
 .../src/realize/normalize/target.rs                | 34 ++++++++--
 docs/design/source-pinned-data-normalization.md    |  7 +-
 website/docs/guides/generate-artifacts.md          | 45 ++++++++++++-
 website/docs/reference/formats.md                  |  3 +
 16 files changed, 481 insertions(+), 41 deletions(-)
```

This literal tracked diff is the implementation handed to this pass, not the adversary delta. The brief explicitly assigned an already-dirty worktree. All 32 inherited changed/untracked files match the starting SHA-256 snapshot; the final status differs only by the new test below. No implementation/design/existing test/Git-state/planning edit was made. No charter-scope violation was observed.

Adversary-only stat, from `git --no-pager diff --no-index --stat /dev/null crates/generate/schema-contract/tests/normalization_raw_adversary.rs` (exit 1 means the added file differs):

```text
 .../tests/normalization_raw_adversary.rs           | 266 +++++++++++++++++++++
 1 file changed, 266 insertions(+)
```

The corresponding full no-index diff also exited 1 and is retained in $SCRATCH/adversary.patch.
Starting tracked patch SHA-256: 6cf216fb7b3e8b24d56ffa60625bbf4c87e452bfa1db19ab54e371334b4aee36.
Adversary patch SHA-256: b5d7af26ae82bb54f1b1c4c28a2ef09dcef8ac69efdd6345cd3a33d57ac2b207.

The exact observed working tree is base/HEAD above plus the full relative-path file-content manifest $SCRATCH/final-all-files.sha256, whose SHA-256 is a8415a4ff0d688840ec384000a5393b1d6ccb6d980e1ad4ae66bf83d23a4a6e9. It was produced by `git ls-files -c -o --exclude-standard -z | sort -zu | xargs -0 sha256sum`; ignored build files and Git metadata are outside that content fingerprint. The starting/final status snapshots separately retain Git working-tree state.

Public report path normalization: $WORKTREE names the assigned ESS worktree; $BUILD names the explicitly leased Cargo/native build root; $SCRATCH names the assigned pass-1 scratch root. $CARGO_CACHE, $GO_BUILD_CACHE and $GO_MOD_CACHE name the ordinary Cargo, Go compiler and offline Go module caches. Those filesystem substitutions are the only transformation used to produce public-report.md; quoted output in that public report is explicitly path-normalized, not verbatim. Original verbatim private logs remain under the assigned scratch root, including isolated-lexical.log, isolated-selectors.log, isolated-rust.log, isolated-go.log, focused-suite.log and clippy.log.

Report handling: this private report quotes tool output verbatim. The separate public-report.md changes only explicitly declared filesystem path spellings; it is not a verbatim private log. Original tool output remains in the isolated, focused-suite and clippy logs listed below.

2. Cases written before execution

All four cases are in crates/generate/schema-contract/tests/normalization_raw_adversary.rs; no existing case was changed. Every new case first ran alone, in the following order, before the focused suite. Each is green now. There was no compile failure or behavioral red output to preserve.

All commands ran in $WORKTREE. Native builds were serial under the exclusive cache lease.

Case `captured_lexemes_and_mixed_error_order_obey_the_document`: 42 document-derived text/helper cases: exact escapes and Unicode bytes, 1000-digit exponent lexemes, recipe-shaped opaque data, object/array depth, grammar-first failure, parent duplicates, outside decoded-key order and captured source order.

Command (exit 0):

```console
CARGO_TARGET_DIR=$BUILD CARGO_BUILD_JOBS=4 TMPDIR=$SCRATCH cargo test --offline --locked -p schema-contract --features go-typecheck --test normalization_raw_adversary captured_lexemes_and_mixed_error_order_obey_the_document -- --exact --nocapture
```

Verbatim first isolated output ($SCRATCH/isolated-lexical.log):

```text
   Compiling schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.77s
     Running tests/normalization_raw_adversary.rs ($BUILD/debug/deps/normalization_raw_adversary-ad973ada859df05e)

running 1 test
adversary lexical cases executed 42
test captured_lexemes_and_mixed_error_order_obey_the_document ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.04s
```

Case `selectors_use_declared_empty_and_escaped_wire_names_and_exact_conflict_order`: Empty/escaped declared wire names, optional nullable string capture, disjoint numeric decoding, open-key refusal, exact escaped pointers, ordered ancestor/numeric/duplicate findings and escape-equivalent selector identity.

Command (exit 0):

```console
CARGO_TARGET_DIR=$BUILD CARGO_BUILD_JOBS=4 TMPDIR=$SCRATCH cargo test --offline --locked -p schema-contract --features go-typecheck --test normalization_raw_adversary selectors_use_declared_empty_and_escaped_wire_names_and_exact_conflict_order -- --exact --nocapture
```

Verbatim first isolated output ($SCRATCH/isolated-selectors.log):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running tests/normalization_raw_adversary.rs ($BUILD/debug/deps/normalization_raw_adversary-ad973ada859df05e)

running 1 test
test selectors_use_declared_empty_and_escaped_wire_names_and_exact_conflict_order ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.02s
```

Case `generated_rust_runs_adversarial_lexical_cases`: The same 42 cases through actual generated Rust normalize/normalize_base64_json APIs in default and arbitrary_precision builds; retained-document composition also executes.

Command (exit 0):

```console
CARGO_TARGET_DIR=$BUILD CARGO_BUILD_JOBS=4 TMPDIR=$SCRATCH cargo test --offline --locked -p schema-contract --features go-typecheck --test normalization_raw_adversary generated_rust_runs_adversarial_lexical_cases -- --exact --nocapture
```

Verbatim first isolated output ($SCRATCH/isolated-rust.log):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/normalization_raw_adversary.rs ($BUILD/debug/deps/normalization_raw_adversary-ad973ada859df05e)

running 1 test
native Rust None; cases 42; exit exit status: 0

running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s



native Rust Some("serde_json/arbitrary_precision"); cases 42; exit exit status: 0

running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s



test generated_rust_runs_adversarial_lexical_cases ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 3.09s
```

Case `generated_go_runs_adversarial_lexical_cases`: The same 42 cases through actual generated Go Normalize/NormalizeBase64JSON APIs with -race; reused native harness additionally executes invalid UTF-8 and retained-document composition.

Command (exit 0):

```console
CARGO_TARGET_DIR=$BUILD CARGO_BUILD_JOBS=4 TMPDIR=$SCRATCH ESS_GO_COMPILER=/usr/bin/go cargo test --offline --locked -p schema-contract --features go-typecheck --test normalization_raw_adversary generated_go_runs_adversarial_lexical_cases -- --exact --nocapture
```

Verbatim first isolated output ($SCRATCH/isolated-go.log):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running tests/normalization_raw_adversary.rs ($BUILD/debug/deps/normalization_raw_adversary-ad973ada859df05e)

running 1 test
native Go; cases 42; exit exit status: 0
=== RUN   TestCorpus
=== RUN   TestCorpus/0
=== RUN   TestCorpus/1
=== RUN   TestCorpus/2
=== RUN   TestCorpus/3
=== RUN   TestCorpus/4
=== RUN   TestCorpus/5
=== RUN   TestCorpus/6
=== RUN   TestCorpus/7
=== RUN   TestCorpus/8
=== RUN   TestCorpus/9
=== RUN   TestCorpus/10
=== RUN   TestCorpus/11
=== RUN   TestCorpus/12
=== RUN   TestCorpus/13
=== RUN   TestCorpus/14
=== RUN   TestCorpus/15
=== RUN   TestCorpus/16
=== RUN   TestCorpus/17
=== RUN   TestCorpus/18
=== RUN   TestCorpus/19
=== RUN   TestCorpus/20
=== RUN   TestCorpus/21
=== RUN   TestCorpus/22
=== RUN   TestCorpus/23
=== RUN   TestCorpus/24
=== RUN   TestCorpus/25
=== RUN   TestCorpus/26
=== RUN   TestCorpus/27
=== RUN   TestCorpus/28
=== RUN   TestCorpus/29
=== RUN   TestCorpus/30
=== RUN   TestCorpus/31
=== RUN   TestCorpus/32
=== RUN   TestCorpus/33
=== RUN   TestCorpus/34
=== RUN   TestCorpus/35
=== RUN   TestCorpus/36
=== RUN   TestCorpus/37
=== RUN   TestCorpus/38
=== RUN   TestCorpus/39
=== RUN   TestCorpus/40
=== RUN   TestCorpus/41
--- PASS: TestCorpus (0.05s)
    --- PASS: TestCorpus/0 (0.00s)
    --- PASS: TestCorpus/1 (0.00s)
    --- PASS: TestCorpus/2 (0.00s)
    --- PASS: TestCorpus/3 (0.00s)
    --- PASS: TestCorpus/4 (0.00s)
    --- PASS: TestCorpus/5 (0.00s)
    --- PASS: TestCorpus/6 (0.00s)
    --- PASS: TestCorpus/7 (0.00s)
    --- PASS: TestCorpus/8 (0.00s)
    --- PASS: TestCorpus/9 (0.00s)
    --- PASS: TestCorpus/10 (0.00s)
    --- PASS: TestCorpus/11 (0.00s)
    --- PASS: TestCorpus/12 (0.00s)
    --- PASS: TestCorpus/13 (0.00s)
    --- PASS: TestCorpus/14 (0.00s)
    --- PASS: TestCorpus/15 (0.00s)
    --- PASS: TestCorpus/16 (0.00s)
    --- PASS: TestCorpus/17 (0.00s)
    --- PASS: TestCorpus/18 (0.00s)
    --- PASS: TestCorpus/19 (0.00s)
    --- PASS: TestCorpus/20 (0.00s)
    --- PASS: TestCorpus/21 (0.00s)
    --- PASS: TestCorpus/22 (0.00s)
    --- PASS: TestCorpus/23 (0.00s)
    --- PASS: TestCorpus/24 (0.00s)
    --- PASS: TestCorpus/25 (0.00s)
    --- PASS: TestCorpus/26 (0.00s)
    --- PASS: TestCorpus/27 (0.00s)
    --- PASS: TestCorpus/28 (0.00s)
    --- PASS: TestCorpus/29 (0.00s)
    --- PASS: TestCorpus/30 (0.00s)
    --- PASS: TestCorpus/31 (0.00s)
    --- PASS: TestCorpus/32 (0.00s)
    --- PASS: TestCorpus/33 (0.00s)
    --- PASS: TestCorpus/34 (0.00s)
    --- PASS: TestCorpus/35 (0.00s)
    --- PASS: TestCorpus/36 (0.00s)
    --- PASS: TestCorpus/37 (0.00s)
    --- PASS: TestCorpus/38 (0.00s)
    --- PASS: TestCorpus/39 (0.00s)
    --- PASS: TestCorpus/40 (0.00s)
    --- PASS: TestCorpus/41 (0.00s)
=== RUN   TestInvalidUTF8AndComposition
--- PASS: TestInvalidUTF8AndComposition (0.00s)
PASS
ok  	example.invalid/normalization-adapter	1.078s


test generated_go_runs_adversarial_lexical_cases ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.51s
```

3. Focused suite after the cases existed

The before count is the implementor's recorded final 8 reference tests plus 1 legacy-map test. This pass did not run a preemptive suite. The after count is the selected suite's 1 legacy-map + 8 reference + 4 adversary tests = 13, all actually executed. Corpus entries and nested native runner tests are separate counts, not additions to this 9→13 header.

Command (exit 0):

```console
CARGO_TARGET_DIR=$BUILD CARGO_BUILD_JOBS=4 TMPDIR=$SCRATCH ESS_GO_COMPILER=/usr/bin/go cargo test --offline --locked -p schema-contract --features go-typecheck --test normalization_raw --test normalization_legacy_bytes --test normalization_raw_adversary -- --test-threads=1 --nocapture
```

Verbatim output ($SCRATCH/focused-suite.log):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running tests/normalization_legacy_bytes.rs ($BUILD/debug/deps/normalization_legacy_bytes-0c0191c231334777)

running 1 test
test complete_legacy_file_maps_are_preserved ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/normalization_raw.rs ($BUILD/debug/deps/normalization_raw-ef025960bdcd6c03)

running 8 tests
test absent_null_and_wrong_intermediates_reach_the_declared_schema ... ok
test capture_envelope_is_closed_and_legacy_readers_refuse ... ok
test capture_is_explicit_and_refinements_apply_to_the_encoded_representation ... ok
test capture_paths_are_qualified_and_conflicts_are_explicit ... ok
test explicit_capture_preserves_tokens_before_the_first_schema ... ok
test lexical_corpus_matches_independent_expected_values_and_findings ... raw JSON corpus executed 156 cases
ok
test old_formats_refuse_even_empty_capture_declarations ... ok
test root_paths_leaf_types_and_both_overlap_directions_are_checked ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s

     Running tests/normalization_raw_adversary.rs ($BUILD/debug/deps/normalization_raw_adversary-ad973ada859df05e)

running 4 tests
test captured_lexemes_and_mixed_error_order_obey_the_document ... adversary lexical cases executed 42
ok
test generated_go_runs_adversarial_lexical_cases ... native Go; cases 42; exit exit status: 0
=== RUN   TestCorpus
=== RUN   TestCorpus/0
=== RUN   TestCorpus/1
=== RUN   TestCorpus/2
=== RUN   TestCorpus/3
=== RUN   TestCorpus/4
=== RUN   TestCorpus/5
=== RUN   TestCorpus/6
=== RUN   TestCorpus/7
=== RUN   TestCorpus/8
=== RUN   TestCorpus/9
=== RUN   TestCorpus/10
=== RUN   TestCorpus/11
=== RUN   TestCorpus/12
=== RUN   TestCorpus/13
=== RUN   TestCorpus/14
=== RUN   TestCorpus/15
=== RUN   TestCorpus/16
=== RUN   TestCorpus/17
=== RUN   TestCorpus/18
=== RUN   TestCorpus/19
=== RUN   TestCorpus/20
=== RUN   TestCorpus/21
=== RUN   TestCorpus/22
=== RUN   TestCorpus/23
=== RUN   TestCorpus/24
=== RUN   TestCorpus/25
=== RUN   TestCorpus/26
=== RUN   TestCorpus/27
=== RUN   TestCorpus/28
=== RUN   TestCorpus/29
=== RUN   TestCorpus/30
=== RUN   TestCorpus/31
=== RUN   TestCorpus/32
=== RUN   TestCorpus/33
=== RUN   TestCorpus/34
=== RUN   TestCorpus/35
=== RUN   TestCorpus/36
=== RUN   TestCorpus/37
=== RUN   TestCorpus/38
=== RUN   TestCorpus/39
=== RUN   TestCorpus/40
=== RUN   TestCorpus/41
--- PASS: TestCorpus (0.10s)
    --- PASS: TestCorpus/0 (0.00s)
    --- PASS: TestCorpus/1 (0.00s)
    --- PASS: TestCorpus/2 (0.00s)
    --- PASS: TestCorpus/3 (0.00s)
    --- PASS: TestCorpus/4 (0.00s)
    --- PASS: TestCorpus/5 (0.01s)
    --- PASS: TestCorpus/6 (0.00s)
    --- PASS: TestCorpus/7 (0.00s)
    --- PASS: TestCorpus/8 (0.01s)
    --- PASS: TestCorpus/9 (0.00s)
    --- PASS: TestCorpus/10 (0.00s)
    --- PASS: TestCorpus/11 (0.00s)
    --- PASS: TestCorpus/12 (0.00s)
    --- PASS: TestCorpus/13 (0.01s)
    --- PASS: TestCorpus/14 (0.00s)
    --- PASS: TestCorpus/15 (0.00s)
    --- PASS: TestCorpus/16 (0.00s)
    --- PASS: TestCorpus/17 (0.00s)
    --- PASS: TestCorpus/18 (0.00s)
    --- PASS: TestCorpus/19 (0.01s)
    --- PASS: TestCorpus/20 (0.00s)
    --- PASS: TestCorpus/21 (0.00s)
    --- PASS: TestCorpus/22 (0.00s)
    --- PASS: TestCorpus/23 (0.00s)
    --- PASS: TestCorpus/24 (0.01s)
    --- PASS: TestCorpus/25 (0.00s)
    --- PASS: TestCorpus/26 (0.01s)
    --- PASS: TestCorpus/27 (0.00s)
    --- PASS: TestCorpus/28 (0.00s)
    --- PASS: TestCorpus/29 (0.01s)
    --- PASS: TestCorpus/30 (0.00s)
    --- PASS: TestCorpus/31 (0.00s)
    --- PASS: TestCorpus/32 (0.00s)
    --- PASS: TestCorpus/33 (0.00s)
    --- PASS: TestCorpus/34 (0.01s)
    --- PASS: TestCorpus/35 (0.00s)
    --- PASS: TestCorpus/36 (0.00s)
    --- PASS: TestCorpus/37 (0.00s)
    --- PASS: TestCorpus/38 (0.00s)
    --- PASS: TestCorpus/39 (0.00s)
    --- PASS: TestCorpus/40 (0.01s)
    --- PASS: TestCorpus/41 (0.00s)
=== RUN   TestInvalidUTF8AndComposition
--- PASS: TestInvalidUTF8AndComposition (0.00s)
PASS
ok  	example.invalid/normalization-adapter	1.144s


ok
test generated_rust_runs_adversarial_lexical_cases ... native Rust None; cases 42; exit exit status: 0

running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s



native Rust Some("serde_json/arbitrary_precision"); cases 42; exit exit status: 0

running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 1 test
.
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s



ok
test selectors_use_declared_empty_and_escaped_wire_names_and_exact_conflict_order ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.60s
```

Scoped new-test Clippy command (exit 0):

```console
CARGO_TARGET_DIR=$BUILD CARGO_BUILD_JOBS=4 TMPDIR=$SCRATCH cargo clippy --offline --locked -p schema-contract --features go-typecheck --test normalization_raw_adversary -- -D warnings
```

Verbatim output ($SCRATCH/clippy.log):

```text
    Checking ess-primitives v0.19.0 ($WORKTREE/crates/specify/ess-primitives)
    Checking ess-domain v0.19.0 ($WORKTREE/crates/specify/ess-domain)
    Checking ess-compiler v0.19.0 ($WORKTREE/crates/specify/ess-compiler)
    Checking ess-gen v0.19.0 ($WORKTREE/crates/generate/ess-gen)
    Checking schema-contract v0.19.0 ($WORKTREE/crates/generate/schema-contract)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.59s
```

Final `rustfmt --check --edition 2021 --config skip_children=true crates/generate/schema-contract/tests/normalization_raw_adversary.rs`: exit 0, no output.
Final `git diff --check`: exit 0, no output.
All 32 starting-file checks: exit 0, every file OK, original output in inherited-files-check.log and scope-audit.log.
Read-only SHA-256 comparisons show all five frozen templates exactly match their source bytes at the base commit; output is in frozen-template-audit.log. The executable six-map verifier above additionally compares all emitted paths and bytes with only its declared generator-version normalization.
The initial rustfmt invocation only formatted the new test (exit 0). One exploratory source search named a nonexistent historical CLI filename and exited 2 while also locating the actual public caller in crates/edge/ess-cli/src/normalize.rs:134; that read-only search was not a test finding.

Every Cargo/native command is terminal. The exclusive build slot was explicitly released to the coordinator after Clippy. No broader existing native corpus, workspace gate or site build was rerun in this pass.

4. Findings

Nothing found.

No findings table rows are returned. These results cover the observed worktree fingerprint above and do not assert approval or verifier independence. No suspected pre-existing runtime issue is promoted to a confirmed finding without a base reproduction.

5. Attacks that did not break

- Exact retained bytes survive escaped selector names, embedded control escapes, duplicate escaped-equivalent captured keys, canonically distinct Unicode strings and 1000-digit exponent lexemes.
- Recipe-shaped retained content remains ordinary data; it does not replace the sealed executable recipe.
- At original-root depths 64/65, Unicode-versus-depth ordering follows the binding contract for both array and object structure.
- Complete grammar errors precede located capture processing; outside parent duplicates precede child findings; outside objects use decoded-key order and captured objects preserve source-order validation.
- Empty wire names, nullable leaves, explicit disjoint numeric paths and escaped branch/field identities retain declared behavior; unknown open-object keys and all tested overlap classes refuse with exact ordered findings.
- Canonical helper composition retains lexical bytes and failure order across actual reference, generated Rust and generated Go entrypoints; existing focused cases also exercise strict base64/pad-bit/UTF-8 refusals, presence and provenance refusal.
- Old-reader admission fences and all six representative legacy emitted-file maps pass; the five frozen templates match the base bytes exactly.

6. Outside-worktree paths

Assigned scratch/TMPDIR root: $SCRATCH. Tool-created temporary descendants, if any, stay under this assigned root.
The 19 named scratch files written by this pass are:

- $SCRATCH/starting.patch
- $SCRATCH/starting-status.txt
- $SCRATCH/starting-files.sha256
- $SCRATCH/isolated-lexical.log
- $SCRATCH/isolated-selectors.log
- $SCRATCH/isolated-rust.log
- $SCRATCH/isolated-go.log
- $SCRATCH/focused-suite.log
- $SCRATCH/clippy.log
- $SCRATCH/final-tracked-stat.txt
- $SCRATCH/adversary-stat.txt
- $SCRATCH/adversary.patch
- $SCRATCH/inherited-files-check.log
- $SCRATCH/final-status.txt
- $SCRATCH/final-all-files.sha256
- $SCRATCH/scope-audit.log
- $SCRATCH/frozen-template-audit.log
- $SCRATCH/private-report.md
- $SCRATCH/public-report.md

Leased build root: $BUILD. Cargo compilation/check outputs and metadata are beneath that root. Generated native fixtures and their nested build outputs are specifically beneath:

- $BUILD/tmp/normalization-raw-adversary-rust
- $BUILD/tmp/normalization-raw-adversary-go
- $BUILD/tmp/normalization-rust-target

Ordinary tool-cache roots used, conservatively included as potentially written metadata/cache paths:

- $CARGO_CACHE
- $GO_BUILD_CACHE

Offline existing module cache read: $GO_MOD_CACHE; no new module pin or download was requested. No /tmp scratch, source fix, planning operation, commit, push or lifecycle action was performed.

```findings
[]
```