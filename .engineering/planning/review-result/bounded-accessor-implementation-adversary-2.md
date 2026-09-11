---
format: aep.planning-md/1
id: review-result:bounded-accessor-implementation-adversary-2
kind: review-result
status: active
title: 'Accessor final adversary: escape limits agree, helper-only allocation probe unreachable'
relations:
- reviews: story:binding-mapping-bounded-accessor
revision: 1
---
unit: story:binding-mapping-bounded-accessor candidate d513145de085777160ca2b9d2e848a2b50bc53be plus test-only additions
verdict: INFEASIBLE
cases: executed 12→2, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 24 paths, full inventory below
needs-coordinator: disposition of unreachable red allocation probe; affected suite not run at coordinator instruction
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 35b13036b35f882d374afef89d71daa8bf14096984111e58191fc1b5472fd8b3, retained as local-evidence:runtime-gaps/publication-replay/snapshots/35b13036b35f882d374afef89d71daa8bf14096984111e58191fc1b5472fd8b3.md. Source creation recorded at 2026-09-11T03:34:08Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 5f90bec5d77212c2dd57ca3aeee4b9a9bcc7f417c814fe823e845d4daa8629fe, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/5f90bec5d77212c2dd57ca3aeee4b9a9bcc7f417c814fe823e845d4daa8629fe-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

```text
 .../ess-conformance/tests/bounded_accessor.rs      | 94 ++++++++++++++++++++++
 1 file changed, 94 insertions(+)
```

The diff changes only the test file. No implementation, Git state, planning artifact, or previous test was changed. Candidate base is ac369db9a152811019bdb840e919c571b1a719f3. The before count 12 is the coordinator's reported complete bounded-accessor run; the after count 2 is two isolated Rust tests actually executed here, each invoking one Go test. The affected 14-test file was not executed: the coordinator's final instruction explicitly limited this handback to saved tests and logs. Thus 12→2 is a difference in selected coverage, not a claim that a 14-test suite ran. No full gate or unchanged baseline was run. A raw-string placement typo was corrected before compilation; it is not a finding.

1. Cases added

- `crates/verify/ess-conformance/tests/bounded_accessor.rs:480`, `adversary_second_go_exact_canonical_escape_boundaries`: green. Ten boundary examples compare Rust serialization/admission with Go canonical counting/admission: HTML characters, actual U+2028/U+2029, literal backslash-u sequences, backslash followed by those Unicode runes, and mixed quotes/backslashes/control characters/HTML/Unicode. Every pattern is accepted at exactly 1,048,576 canonical bytes and refused at 1,048,577. The seeded observation comes from the compiled bounded-accessor fixture, with only legal summary metadata changed. Prior 12 tests are intact.
- `crates/verify/ess-conformance/tests/bounded_accessor.rs:543`, `adversary_second_go_refuses_oversize_before_full_encoding_allocation`: red at assertion :565, reflected through Go wrapper :413. Direct helper admission of an already-decoded 8 MiB summary allocates 16,809,904 bytes before returning the correct resource refusal. **This is not demonstrated reachable through the public suite reader:** the earlier raw-observation preflight rejects this size. Retained as an explicitly infeasible probe, not a shipping blocker. The four-MiB allocation assertion is conservative test headroom, not a published exact allocation limit. Only its explanatory comment was corrected after the run to acknowledge the earlier guard; test inputs/assertions are unchanged.

Exact common invocation prefix for both isolated runs:

```console
env -u CARGO_ENCODED_RUSTFLAGS RUSTUP_TOOLCHAIN=1.98.1 CARGO_TARGET_DIR=local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/accessor CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_INCREMENTAL=false CARGO_PROFILE_TEST_INCREMENTAL=false RUSTFLAGS='-C link-arg=-fuse-ld=lld' RUSTC_WRAPPER=sccache TMPDIR=local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp cargo test --offline --locked -p ess-conformance --test bounded_accessor <case> -- --exact --nocapture
```

First executed case: `adversary_second_go_refuses_oversize_before_full_encoding_allocation`. Rust exit 101, nested Go exit 1. Verbatim retained output:

```text
   Compiling ess-conformance v0.22.2 (worktree-state:/trees/b10x/ess/wt-30254233b1b4/crates/verify/ess-conformance)
    Finished `test` profile [unoptimized] target(s) in 0.45s
     Running tests/bounded_accessor.rs (local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/accessor/debug/deps/bounded_accessor-48342fe9a880cf72)

running 1 test

thread 'adversary_second_go_refuses_oversize_before_full_encoding_allocation' (2612439) panicked at crates/verify/ess-conformance/tests/bounded_accessor.rs:413:5:
exit: Some(1)
=== RUN   TestAdversary
    adversary_test.go:17: already-decoded summary bytes=8388608, admission allocation=16809904, refusal=AccessorResource: observation bytes
    adversary_test.go:20: resource refusal buffered oversized observation: allocated 16809904 bytes after input was decoded
--- FAIL: TestAdversary (0.01s)
FAIL
FAIL	example.invalid/accessoradversary/essconform	0.015s
FAIL


note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test adversary_second_go_refuses_oversize_before_full_encoding_allocation ... FAILED

failures:

failures:
    adversary_second_go_refuses_oversize_before_full_encoding_allocation

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.33s

error: test failed, to rerun pass `-p ess-conformance --test bounded_accessor`
```

Second executed case: `adversary_second_go_exact_canonical_escape_boundaries`. Rust exit 0, nested Go exit 0. Verbatim retained outputs:

```text
    Finished `test` profile [unoptimized] target(s) in 0.04s
     Running tests/bounded_accessor.rs (local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/accessor/debug/deps/bounded_accessor-48342fe9a880cf72)

running 1 test
test adversary_second_go_exact_canonical_escape_boundaries ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 1.75s

exit: Some(0)
=== RUN   TestAdversary
    adversary_test.go:30: case 0 canonical=1048576 admitted=true
    adversary_test.go:30: case 1 canonical=1048577 admitted=false
    adversary_test.go:30: case 2 canonical=1048576 admitted=true
    adversary_test.go:30: case 3 canonical=1048577 admitted=false
    adversary_test.go:30: case 4 canonical=1048576 admitted=true
    adversary_test.go:30: case 5 canonical=1048577 admitted=false
    adversary_test.go:30: case 6 canonical=1048576 admitted=true
    adversary_test.go:30: case 7 canonical=1048577 admitted=false
    adversary_test.go:30: case 8 canonical=1048576 admitted=true
    adversary_test.go:30: case 9 canonical=1048577 admitted=false
--- PASS: TestAdversary (0.05s)
PASS
ok  	example.invalid/accessoradversary/essconform	0.047s

```

2. Affected suite

Not run. Coordinator ordered no further probes/testing and a handback from retained evidence. Two new isolated cases were executed first; all twelve prior cases are preserved, but their current result is coordinator-reported rather than remeasured in this review. No approval or complete-suite verification is claimed.

3. Findings

| File:line | Category / severity | Verdict / origin | Finding | What was measured | What reaches it |
| --- | --- | --- | --- | --- | --- |
| crates/verify/ess-conformance/src/go/runtime.go:4331 | boundary / note | INFEASIBLE / introduced | Direct Go accessor admission buffers an entire oversized observation before refusal, but the measured 8 MiB input is blocked by public suite preflight. | Test :543, assertion :565; nested Go exit 1 / Rust exit 101; 16,809,904 newly allocated bytes for an already-decoded 8,388,608-byte summary. | The direct helper is called by `admitValues` at :3330; `admitSuiteDocument` first calls `accessorPreflight` at :3061, and `accessorRawSize.UnmarshalJSON` rejects over 2 MiB at :4012. The measured 8 MiB helper invocation bypasses that guard. No public caller reaching this measured condition was found. |

Origin is introduced because the entire accessor admission capability and the buffered encoder at :4331 are additions in the candidate diff against ac369db9; the base has no `admitAccessor` or `accessorPreflight`. This is not a claim of base reproduction, nor a claim that the bounded public reader has unbounded accessor encoding. The design's bounded-sink wording at `docs/design/binding-mapping-bounded-accessor.md:101` motivated the attack, but the larger helper-only probe does not establish a blocker against the public workflow. The coordinator can discard or explicitly retain the infeasible probe; do not treat its process exit as evidence the public path fails.

4. Attacks that did not break

- Corrected canonical byte handling accepts/rejects exact thresholds for all ten escaped/raw Unicode, HTML, quote, backslash, and control-character examples.
- Literal `\\u2028` / `\\u2029` are not mistaken for actual three-byte Unicode runes.
- An actual rune after an escaped backslash is counted correctly.
- Existing optional-newtype/union and Unicode regression tests remain unchanged; their twelve-case green result is credited only to the coordinator.

5. Outside-worktree inventory

Full files are listed below. Three reproducible shared cache directories are named as directories rather than enumerating compiler-managed internal objects. No cache/worktree was removed. The assigned tmp directory contains only the two generated Go probes. Managed worktree lease metadata was also updated by the required worktree CLI; it is manager-owned state, not an untracked source artifact.

```text
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/allocation-isolated.log
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/escaping-isolated.log
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/outside-paths.txt
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/report.md
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tests.patch
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-bounded-allocation-2612438/essconform/README.md
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-bounded-allocation-2612438/essconform/adversary_test.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-bounded-allocation-2612438/essconform/predicate.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-bounded-allocation-2612438/essconform/runtime.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-bounded-allocation-2612438/essconform/suite.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-bounded-allocation-2612438/essconform/suite.json
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-bounded-allocation-2612438/go.log
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-bounded-allocation-2612438/go.mod
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-escape-boundaries-2613570/essconform/README.md
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-escape-boundaries-2613570/essconform/adversary_test.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-escape-boundaries-2613570/essconform/predicate.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-escape-boundaries-2613570/essconform/runtime.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-escape-boundaries-2613570/essconform/suite.go
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-escape-boundaries-2613570/essconform/suite.json
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-escape-boundaries-2613570/go.log
local-evidence:/cache/ess-evolution-20260910/priority-wave/accessor/implementation-adversary-2/tmp/ess-accessor-adversary-second-escape-boundaries-2613570/go.mod
local-evidence:/cache/ess-evolution-20260910/priority-wave/build-targets/accessor
local-evidence:/cache/go-build
local-evidence:/cache/sccache
```

**Publication path-redaction notice.** This public copy replaces only exact local home-path prefixes: the managed-worktree state root is `worktree-state:/`, and the local cache root is `local-evidence:/cache/`. Remaining path suffixes are preserved. These aliases are evidence labels, not executable filesystem paths. Original logs and the original report remain unmodified; quoted output here differs only in those path prefixes. Review findings, verdicts, test counts, and test outcomes are unchanged. This copy records no additional review or execution.

Original report SHA256: `3d0c76fe539a189d82e4e23547789d669c88371e23877329d26d2471597b6297`.

```findings
- file: crates/verify/ess-conformance/src/go/runtime.go
  line: 4331
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: Direct Go accessor admission buffers an entire oversized observation before refusal, but the measured 8 MiB input is blocked by public suite preflight.
```