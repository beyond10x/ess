---
format: aep.planning-md/1
id: review-result:bounded-accessor-implementation-adversary-1
kind: review-result
status: active
title: 'Accessor implementation adversary: nullable payload and Unicode byte parity'
relations:
- reviews: story:binding-mapping-bounded-accessor
revision: 1
---
unit: story:binding-mapping-bounded-accessor, candidate 873267401916de71948fffff06aa668dfdf543d2
verdict: CONFIRMED
cases: executed 9→12, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 52 retained file/cache paths, enumerated below
needs-coordinator: return both findings to implementor; root retains CLI/generated integration and consumer-adoption obligations
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 e8c71e67019d44df06a7e1e85fbdef1c192a1cfe5b9549d003457c9e5806a59c, retained as local-evidence:runtime-gaps/publication-replay/snapshots/e8c71e67019d44df06a7e1e85fbdef1c192a1cfe5b9549d003457c9e5806a59c.md. Source creation recorded at 2026-09-11T03:34:05Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 1f95fd4b8a39bc17cacc5d66d8ee467a787e71604256d3a9d21b9bbf270b2fde, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/1f95fd4b8a39bc17cacc5d66d8ee467a787e71604256d3a9d21b9bbf270b2fde-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

```text
 .../ess-conformance/tests/bounded_accessor.rs      | 153 +++++++++++++++++++++
 1 file changed, 153 insertions(+)
```

This is the actual final test-only worktree diff stat; tests.patch retains the full diff. No implementation, planning, Git, or pre-existing test assertion was changed. The reviewed source stayed at the exact candidate. The reviewer lease ess-accessor-adversary-01a089ee was acquired before additions and released after execution; the managed tree remains for its coordinator. No full gate, ownership gate, base checkout, or source repair ran.

1. New cases and first isolated executions

Commands ran in worktree-state:/trees/b10x/ess/wt-30254233b1b4, with this exact bounded build and assigned-scratch environment:

```sh
env -u CARGO_ENCODED_RUSTFLAGS RUSTUP_TOOLCHAIN=1.98.1 CARGO_TARGET_DIR=local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/accessor CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_INCREMENTAL=false CARGO_PROFILE_TEST_INCREMENTAL=false RUSTFLAGS='-C link-arg=-fuse-ld=lld' RUSTC_WRAPPER=sccache TMPDIR=local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp
```

The environment prefixes each cargo command below. Output was captured with pipefail and tee; quoted exits are Cargo statuses. Cases existed and compiled before the first execution; there were no fixture-compile failures.

Rust observation of an authored union whose ready payload is NullableBody = Optional<Body> must return Unavailable for explicit null. The source compiles, the generated accessor suite admits, and malformed missing/scalar controls refuse before the deciding assertion. RED at tests/bounded_accessor.rs:372; exit 101.

```sh
cargo test --offline --locked -p ess-conformance --test bounded_accessor adversary_nullable_newtype_union_payload_is_unavailable_not_malformed -- --exact --nocapture
```

Verbatim nullable-rust-first.log:

```text
   Compiling ess-conformance v0.22.2 (worktree-state:/trees/b10x/ess/wt-30254233b1b4/crates/verify/ess-conformance)
    Finished `test` profile [unoptimized] target(s) in 0.41s
     Running tests/bounded_accessor.rs (local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/accessor/debug/deps/bounded_accessor-48342fe9a880cf72)

running 1 test

thread 'adversary_nullable_newtype_union_payload_is_unavailable_not_malformed' (2538492) panicked at crates/verify/ess-conformance/tests/bounded_accessor.rs:372:5:
assertion `left == right` failed
  left: Err("required union payload is null")
 right: Ok(Absent)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test adversary_nullable_newtype_union_payload_is_unavailable_not_malformed ... FAILED

failures:

failures:
    adversary_nullable_newtype_union_payload_is_unavailable_not_malformed

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p ess-conformance --test bounded_accessor`
```

Generated Go observation of the same authored nullable-newtype union must return absence. GREEN; Cargo exit 0, one generated Go TestAdversary passes. This is a separate positive control, not a claim that the Rust runner passed.

```sh
cargo test --offline --locked -p ess-conformance --test bounded_accessor adversary_go_nullable_newtype_union_payload_is_unavailable -- --exact --nocapture
```

Verbatim nullable-go-first.log:

```text
    Finished `test` profile [unoptimized] target(s) in 0.04s
     Running tests/bounded_accessor.rs (local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/accessor/debug/deps/bounded_accessor-48342fe9a880cf72)

running 1 test
test adversary_go_nullable_newtype_union_payload_is_unavailable ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.34s
```

An authored root-field summary containing 180,000 U+2028 characters produces an observation between 540,000 and 600,000 canonical Rust bytes. Both compile and Rust persisted admission succeed; the generated Go suite reader must also admit those emitted bytes. RED at tests/bounded_accessor.rs:413 called by :445; Cargo exit 101, generated Go exit 1 with AccessorResource: observation bytes.

```sh
cargo test --offline --locked -p ess-conformance --test bounded_accessor adversary_go_accepts_rust_admitted_unicode_below_observation_byte_limit -- --exact --nocapture
```

Verbatim unicode-go-first.log:

```text
    Finished `test` profile [unoptimized] target(s) in 0.04s
     Running tests/bounded_accessor.rs (local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/accessor/debug/deps/bounded_accessor-48342fe9a880cf72)

running 1 test

thread 'adversary_go_accepts_rust_admitted_unicode_below_observation_byte_limit' (2540751) panicked at crates/verify/ess-conformance/tests/bounded_accessor.rs:413:5:
exit: Some(1)
=== RUN   TestAdversary
    adversary_test.go:6: Rust-admitted, compiler-produced suite refused by Go: project/binding/mapping: AccessorResource: observation bytes
--- FAIL: TestAdversary (0.03s)
FAIL
FAIL	example.invalid/accessoradversary/essconform	0.029s
FAIL


note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test adversary_go_accepts_rust_admitted_unicode_below_observation_byte_limit ... FAILED

failures:

failures:
    adversary_go_accepts_rust_admitted_unicode_below_observation_byte_limit

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 11 filtered out; finished in 2.20s

error: test failed, to rerun pass `-p ess-conformance --test bounded_accessor`
```

2. Affected suite, after the isolated cases

The before count is the implementor's explicitly reported 9 executed bounded_accessor tests (implementation-report.md, final conformance lane), not a pre-emptive run. This pass selected all 12 once after writing and individually running the three additions. Ten passed and two failed; exit 101. Nested Go tests are not added a second time to the Rust case count. This is not a reexecution of the implementor's broader 1424-case union.

```sh
cargo test --offline --locked -p ess-conformance --test bounded_accessor -- --nocapture
```

Verbatim bounded-accessor-suite.log:

```text
    Finished `test` profile [unoptimized] target(s) in 0.04s
     Running tests/bounded_accessor.rs (local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/accessor/debug/deps/bounded_accessor-48342fe9a880cf72)

running 12 tests
test hidden_nested_optional_whole_leaf_refuses ... ok
test nominal_observation_facts_are_rechecked_at_the_persisted_boundary ... ok
test observations_use_declared_names_without_wire_alias_fallback ... ok
test absent_traversal_and_missing_terminal_are_different ... ok
test terminal_nested_optional_equal_target_refuses_but_deeper_target_is_observable ... ok
test unavailable_union_branch_still_checks_its_declared_payload_kind ... ok

thread 'adversary_nullable_newtype_union_payload_is_unavailable_not_malformed' (2544338) panicked at crates/verify/ess-conformance/tests/bounded_accessor.rs:372:5:
assertion `left == right` failed
  left: Err("required union payload is null")
 right: Ok(Absent)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test adversary_nullable_newtype_union_payload_is_unavailable_not_malformed ... FAILED
test coverage_refusals_select_seven_and_exact_filtered_lineage_cannot_downgrade ... ok
test executable_accessors_select_new_suite_and_roundtrip_closed_admission ... ok
test adversary_go_nullable_newtype_union_payload_is_unavailable ... ok
test generated_go_executes_accessor_admission_and_presence_faults ... ok

thread 'adversary_go_accepts_rust_admitted_unicode_below_observation_byte_limit' (2544336) panicked at crates/verify/ess-conformance/tests/bounded_accessor.rs:413:5:
exit: Some(1)
=== RUN   TestAdversary
    adversary_test.go:6: Rust-admitted, compiler-produced suite refused by Go: project/binding/mapping: AccessorResource: observation bytes
--- FAIL: TestAdversary (0.03s)
FAIL
FAIL	example.invalid/accessoradversary/essconform	0.032s
FAIL


test adversary_go_accepts_rust_admitted_unicode_below_observation_byte_limit ... FAILED

failures:

failures:
    adversary_go_accepts_rust_admitted_unicode_below_observation_byte_limit
    adversary_nullable_newtype_union_payload_is_unavailable_not_malformed

test result: FAILED. 10 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.30s

error: test failed, to rerun pass `-p ess-conformance --test bounded_accessor`
```

3. Findings

| Source | Verdict / origin / severity | What was measured | What reaches it |
| --- | --- | --- | --- |
| crates/verify/ess-conformance/src/accessor.rs:219 | CONFIRMED / introduced / blocker | A legal null union payload behind a newtype of Optional is rejected by Rust even though its checked traversal shape permits absence. New test :372 returns Err("required union payload is null") instead of Ok(Absent), exit 101; generated Go control passes. | Authored ess/3 Choice.ready: NullableBody with NullableBody a newtype of Optional<Body>, mapping event.choice.status to Optional<String>; compile → synthesize → admitted suite succeeds. Runner::resolve_expected at runner.rs:1729–1737 invokes this evaluator on observed events. |
| crates/verify/ess-conformance/src/go/runtime.go:4318 | CONFIRMED / introduced / blocker | Generated Go rejects a compiler-produced observation below Rust's 1 MiB canonical-byte limit because its JSON encoder expands U+2028 metadata. New test :445 passes source compilation, Observation::bytes bounds and Rust admission, then Go TestAdversary fails with AccessorResource: observation bytes, exit 1 (Cargo 101). | Authored event root Field.summary survives into the checked plan; synthesize → go::emit emits the exact admitted suite. Go Run at runtime.go:1506–1513 calls admitRunInput, whose closed observation admission invokes admitAccessor at :3330. No forged persisted plan is required. |

Both origins are introduced based on the reviewed base-to-candidate diff: accessor.rs is a new file and the failing Rust branch is newly added; admitAccessor and its Go canonical-count branch are newly added at this candidate. This is a direct new-path attribution, not a claim that these new-API tests executed against the base.

The nullable case contradicts the design's transparent-newtype traversal rule (docs/design/binding-mapping-bounded-accessor.md:55–56) and Unavailable semantics for an empty intermediate Optional (:165–167), plus acceptance's requirement that the conformance scenario reads the same value as the mapping. Correction: determine union-payload null eligibility from the checked transparent shape, or let the following checked node validate it, while preserving missing-content and required nonnullable/scalar refusals. Do not refuse this legal source construct to hide reader disagreement.

The Unicode case contradicts the shared canonical per-accessor byte contract (design :89–102) and same-suite reader compatibility. Correction: count the same canonical UTF-8/escape representation in Go and Rust, including U+2028/U+2029; keep the declared 1 MiB and aggregate bounds. SetEscapeHTML(false) alone does not give Go Rust's Unicode escape policy. Do not silently lower admission or raise only one reader's budget.

4. Attacked without breaking

- Existing nine accessor cases remain green: missing versus null, malformed unions, declared versus wire names, hidden Optional ambiguity, closed nominal certificates, suite/6 admission, coverage/7 exact lineage, and generated Go absence enforcement.
- The added nullable-newtype case still rejects missing union content and scalar payload before its deciding null assertion.
- Generated Go correctly handles the nullable-newtype null payload in its separate positive-control case.
- Source review covered the complete candidate delta, new tests, typed native accessor callers and conditional formats; this pass did not rerun native Rust/Go binding fixtures, CLI or generated schema projections and makes no fresh execution claim for them.
- The four consumer-adoption rows remain open; no arbitrary host conversion or downstream realization is claimed. Root owns those and the remaining integration surfaces.

5. Outside-worktree paths

The retained-file/cache inventory is also in outside-paths.txt. TMPDIR was assigned scratch for every test, including the pre-existing generated-Go fixture. Rust and Go transient build files lived under that assigned temporary subtree and were toolchain-cleaned normally. Persistent compiler/cache directories are listed as roots rather than falsely enumerating every cache internals file. The existing brief.md was read only. No manual source/worktree/cache deletion occurred; worktree CLI owns its lifecycle metadata.

```text
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/bounded-accessor-suite.log
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/nullable-go-first.log
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/nullable-rust-first.log
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/outside-paths.txt
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/report.md
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tests.patch
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2539485/essconform/README.md
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2539485/essconform/adversary_test.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2539485/essconform/predicate.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2539485/essconform/runtime.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2539485/essconform/suite.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2539485/essconform/suite.json
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2539485/go.log
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2539485/go.mod
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2544334/essconform/README.md
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2544334/essconform/adversary_test.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2544334/essconform/predicate.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2544334/essconform/runtime.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2544334/essconform/suite.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2544334/essconform/suite.json
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2544334/go.log
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-nullable-2544334/go.mod
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2540750/essconform/README.md
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2540750/essconform/adversary_test.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2540750/essconform/predicate.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2540750/essconform/runtime.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2540750/essconform/suite.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2540750/essconform/suite.json
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2540750/go.log
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2540750/go.mod
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2544334/essconform/README.md
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2544334/essconform/adversary_test.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2544334/essconform/predicate.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2544334/essconform/runtime.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2544334/essconform/suite.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2544334/essconform/suite.json
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2544334/go.log
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-adversary-unicode-2544334/go.mod
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-go-2544334/coverage-input.json
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-go-2544334/essconform/README.md
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-go-2544334/essconform/accessor_test.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-go-2544334/essconform/predicate.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-go-2544334/essconform/runtime.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-go-2544334/essconform/suite.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-go-2544334/essconform/suite.json
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-go-2544334/filtered-input.json
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-go-2544334/go.log
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/tmp/ess-accessor-go-2544334/go.mod
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary/unicode-go-first.log
local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/accessor
local-evidence:/cache/go-build
local-evidence:/cache/sccache
```

**Publication path-redaction notice.** This public copy replaces only exact local home-path prefixes: the managed-worktree state root is `worktree-state:/`, and the local cache root is `local-evidence:/cache/`. Remaining path suffixes are preserved. These aliases are evidence labels, not executable filesystem paths. Original logs and the original report remain unmodified; quoted output here differs only in those path prefixes. Review findings, verdicts, test counts, and test outcomes are unchanged. This copy records no additional review or execution.

Original report SHA256: `64baf3ec540f73252418aeab8205dbe4f79166c7f98c5a1b26c3e79a2eb84ef0`.

```findings
- file: crates/verify/ess-conformance/src/accessor.rs
  line: 219
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: A legal null union payload behind a newtype of Optional is rejected by Rust even though its checked traversal shape permits absence.
- file: crates/verify/ess-conformance/src/go/runtime.go
  line: 4318
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Generated Go rejects a compiler-produced observation below Rust's 1 MiB canonical-byte limit because its JSON encoder expands U+2028 metadata.
```