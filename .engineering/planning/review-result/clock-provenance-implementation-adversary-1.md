---
format: aep.planning-md/1
id: review-result:clock-provenance-implementation-adversary-1
kind: review-result
status: active
title: Clock provenance implementation adversary, round 1
relations:
- reviews: story:timestamp-clock-provenance-contract
revision: 1
---
unit: story:timestamp-clock-provenance-contract working tree wt-6b9e1be0a97b at base 32c765bc8535d2bf56d5a7fe56871208d5e8d830, candidate manifest fb00580a0d1ddd9697437194332669427fe96f2e70c5b1d8f64065907c84abd3
verdict: CONFIRMED
cases: executed 0→1, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 22 retained file or storage-root paths, listed below
needs-coordinator: record this immutable finding, route correction and retain the regression; no approval or adoption claim
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 b989e18ba5d68edaddd8ed27f8411dce7448000e8a1fc3f5bf1866434831bb17, retained as local-evidence:runtime-gaps/publication-replay/snapshots/b989e18ba5d68edaddd8ed27f8411dce7448000e8a1fc3f5bf1866434831bb17.md. Source creation recorded at 2026-09-11T03:45:24Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 65a4ad0004435974e12d59fb43aeab5cbe124c52a5057a12e80a0d8eb25a503d, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/65a4ad0004435974e12d59fb43aeab5cbe124c52a5057a12e80a0d8eb25a503d-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

```text
$ git --no-pager diff --stat
 crates/generate/ess-gen/src/types.rs               |  4 ++
 crates/generate/ess-synth/src/failure.rs           |  4 ++
 crates/generate/ess-synth/src/go/items.rs          |  1 +
 crates/generate/ess-synth/src/go/mod.rs            |  4 ++
 crates/generate/ess-synth/src/rust/items.rs        |  1 +
 crates/generate/ess-synth/src/rust/mod.rs          |  6 ++-
 crates/specify/ess-compiler/src/expression.rs      |  3 ++
 crates/specify/ess-compiler/src/ir.rs              |  3 ++
 crates/specify/ess-compiler/src/resolve.rs         |  1 +
 crates/specify/ess-domain/src/binding.rs           |  1 +
 crates/specify/ess-domain/src/command.rs           |  2 +
 crates/specify/ess-domain/src/domain.rs            |  1 +
 crates/specify/ess-domain/src/entity.rs            |  4 ++
 crates/specify/ess-domain/src/expression.rs        | 31 +++++++++++++-
 crates/specify/ess-domain/src/lib.rs               |  1 +
 .../specify/ess-domain/src/primitive_admission.rs  |  7 ++++
 crates/specify/ess-domain/src/system.rs            |  1 +
 crates/specify/ess-domain/src/types.rs             | 39 ++++++++++++++++++
 crates/specify/ess-domain/src/view.rs              |  2 +
 crates/specify/ess-domain/tests/expression.rs      | 47 ++++++++++++++++++++++
 crates/verify/ess-conformance/src/admission.rs     | 28 ++++++++++++-
 crates/verify/ess-conformance/src/go/mod.rs        | 21 ++++++++--
 crates/verify/ess-conformance/src/go/runtime.go    | 38 ++++++++++++-----
 crates/verify/ess-conformance/src/lib.rs           |  1 +
 crates/verify/ess-conformance/src/report.rs        |  9 ++++-
 crates/verify/ess-conformance/src/runner.rs        | 28 ++++++++++++-
 crates/verify/ess-conformance/src/scenario.rs      | 10 +++++
 crates/verify/ess-conformance/src/synthesize.rs    | 13 +++++-
 crates/verify/ess-conformance/src/target.rs        | 11 +++++
 .../ess-conformance/tests/bounded_accessor.rs      |  8 ++--
 crates/verify/ess-conformance/tests/synthesis.rs   |  1 +
 crates/verify/ess-diff/src/change.rs               | 17 +++++++-
 crates/verify/ess-diff/src/delta.rs                | 12 +++++-
 crates/verify/ess-diff/src/diff.rs                 |  6 +++
 34 files changed, 337 insertions(+), 29 deletions(-)
```

That stat is the candidate's pre-existing tracked diff, unchanged by the reviewer; Git omits its untracked source files and the new reviewer fixture. All 52 candidate files retain their pre-review SHA256 values (`candidate-integrity.json`). The sole reviewer addition in the checkout is `crates/verify/ess-conformance/tests/fixtures/clock-authority-adversary.go`, a test fixture. No implementation, existing test, AEP or Git state was changed. The candidate was uncommitted when assigned; the finding covers the manifest identified above, not the base alone.

This public report uses `local-evidence:` for the cache root and `worktree-state:` for managed checkout paths. Exact local paths are retained privately in `paths-private.md`. No private consumer identity is included.

## 1. Added case and first execution

`crates/verify/ess-conformance/tests/fixtures/clock-authority-adversary.go` adds `TestClockAdversaryRequestMutationCannotAuthorizeOrigin`. A target first validates the original event/member/type contract through the existing fixture adapter. It returns the undeclared `consumer_process` origin for producer-only offset/Unix members. The control returns those facts without touching the request and must fail origin compatibility. The fault does one additional operation: changes `request.Reading.Contract.Origins[0].Role` through the by-value callback request. That mutation must not authorize the returned facts.

The case remains red. Its control passed and its mutation subcase failed. Two subprocesses each executed one real scenario: the control child exited 1 with `IncompatibleOrigin`; the mutation child exited 0 and wrote report/2 with `execution_status=passed`, one passed scenario and no failed/error/unsupported scenarios. Both use the unchanged suite digest. The report still says `conformance_status=inconclusive` because suite coverage is unknown; the defect is false assertion/scenario success, not a demonstrated complete-conformance success.

The test file existed before the first execution. It was copied byte-for-byte to `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/go/essconform/clock_authority_adversary_test.go`. The pre-existing generated module was copied without mutation. Its runtime is exactly the candidate's `go/runtime.go`, `go/reading.go`, and domain `reading/coordinate.go` joined by newlines: 166918 bytes, SHA256 `c6c61e2b38f9079cdd4c34f50b4d6ac334275f8d5c8888057c2398cf6a81d671`. The predicate and existing clock test fixture also match current source bytes exactly. `generated-correspondence.json` records every copied artifact digest before addition of the test.

Command in `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/go`, with GOMAXPROCS=2 and both GOTMPDIR/TMPDIR set to its sibling `gotmp`:

```text
go test -p 2 -count=1 -run '^TestClockAdversaryRequestMutationCannotAuthorizeOrigin$' -v ./essconform
=== RUN   TestClockAdversaryRequestMutationCannotAuthorizeOrigin
=== RUN   TestClockAdversaryRequestMutationCannotAuthorizeOrigin/control
    clock_authority_adversary_test.go:56: mode=control child exit=exit status 1 execution_status=failed
        === RUN   TestClockAdversaryRequestMutationCannotAuthorizeOrigin
            clock_authority_adversary_test.go:35: chronology v1, 1 scenario(s), spec digest 397d6fe654628b9c2d7bc396dd88e67210de577ce5d500cdd2cfeb8838d89bf6
        === RUN   TestClockAdversaryRequestMutationCannotAuthorizeOrigin/chronology.reading/authored/clock-reading
            runtime.go:1766: compare actual reading coordinates
            runtime.go:2459: step 1: clock reading: clock reading: IncompatibleOrigin
        --- FAIL: TestClockAdversaryRequestMutationCannotAuthorizeOrigin (0.00s)
            --- FAIL: TestClockAdversaryRequestMutationCannotAuthorizeOrigin/chronology.reading/authored/clock-reading (0.00s)
        FAIL
=== RUN   TestClockAdversaryRequestMutationCannotAuthorizeOrigin/mutate
    clock_authority_adversary_test.go:56: mode=mutate child exit=<nil> execution_status=passed
        === RUN   TestClockAdversaryRequestMutationCannotAuthorizeOrigin
            clock_authority_adversary_test.go:35: chronology v1, 1 scenario(s), spec digest 397d6fe654628b9c2d7bc396dd88e67210de577ce5d500cdd2cfeb8838d89bf6
        === RUN   TestClockAdversaryRequestMutationCannotAuthorizeOrigin/chronology.reading/authored/clock-reading
            runtime.go:1766: compare actual reading coordinates
        --- PASS: TestClockAdversaryRequestMutationCannotAuthorizeOrigin (0.00s)
            --- PASS: TestClockAdversaryRequestMutationCannotAuthorizeOrigin/chronology.reading/authored/clock-reading (0.00s)
        PASS
    clock_authority_adversary_test.go:61: request mutation authorized an undeclared origin; exact report:
        {
          "completed_at": 1789097754120,
          "conformance_status": "inconclusive",
          "counts": {
            "error": 0,
            "failed": 0,
            "passed": 1,
            "skipped": 0,
            "total": 1,
            "unsupported": 0
          },
          "coverage": {
            "knowledge": "unknown"
          },
          "execution_status": "passed",
          "format": "ess-conformance-report/2",
          "implementation": "clock-fixture 1",
          "outcomes": {
            "error": [],
            "failed": [],
            "passed": [
              "chronology.reading/authored/clock-reading"
            ],
            "skipped": [],
            "unsupported": []
          },
          "policy": "complete-selection/1",
          "producer_profile": "go-scenario-status/1",
          "spec_digest": "397d6fe654628b9c2d7bc396dd88e67210de577ce5d500cdd2cfeb8838d89bf6",
          "specification": "chronology/v1",
          "suite": {
            "digest": "sha256:1fd898a310798950b899a63f9d68309895ff50535b565e966229bff46428353a",
            "digest_profile": "sha256-json-bytes/1",
            "version": "ess-conformance/6"
          }
        }
--- FAIL: TestClockAdversaryRequestMutationCannotAuthorizeOrigin (0.00s)
    --- PASS: TestClockAdversaryRequestMutationCannotAuthorizeOrigin/control (0.00s)
    --- FAIL: TestClockAdversaryRequestMutationCannotAuthorizeOrigin/mutate (0.00s)
FAIL
FAIL	clock-conformance/essconform	0.006s
FAIL
exit: 1
```

## 2. Affected suite execution boundary

No additional suite was run. The coordinator explicitly directed “only new probe until immutable verdict” and prohibited rerunning existing green broad Go cases before the fix. This narrower instruction superseded the default follow-up suite step. The header counts the selected new probe: zero such cases in the prior candidate, one actually executed after addition, with two control/mutation subcases. It does not claim the implementor's three existing Go functions or its 46 total bounded case functions were rerun. No Cargo build, full gate, ownership run or wall-clock sleep occurred. The compiler slot was released immediately after the 0.006-second Go package execution.

## 3. Finding

| File:line | Verdict | Severity | Origin | What was measured |
|---|---|---|---|---|
| `crates/verify/ess-conformance/src/go/reading.go:143` | CONFIRMED | blocker | introduced | Go observation request origins alias the runner contract, so mutating the callback DTO makes an otherwise incompatible returned origin pass the scenario. |

The callback receives a shallow struct copy: `Contract.Origins` retains the same backing array. Lines 147–151 construct the admissible origin list only after the callback, so the adapter can overwrite the declared requirement used by the evaluator. This contradicts the design's separation between declared producer/consumer alternatives and observed facts, and the requirement that the runner check compatibility against that declaration. The Rust path clones its `ReadingReference`, including the vector, before handing it to the target; the Go request does not provide that isolation.

The reachability witness uses an ordinarily admitted suite6, a valid existing observed occurrence, the real Go runner, a valid same-source/epoch observation, and a target that validates the original declaration before returning. No malformed suite, forged expected coordinate or comparison verdict is involved. A target remains responsible for truthful facts; that trust does not grant it mutable ownership of the test's requirements.

Origin is introduced: the affected reading module/operation is entirely new in this candidate and `git show` confirms that the module does not exist at the assigned base (captured in `origin.log`, exit128). The base cannot execute a clock-reading scenario; no claim of a base reproduction is made.

Required correction: isolate the target's nested request data from the runner-owned contract before the callback, including the origins backing array, and evaluate against preserved requirements. Retain both the non-mutating control and mutation regression. Correcting only a temporary comparison list may leave persisted scenario requirements mutable for later steps, so protect ownership as well as this immediate check. This report does not apply the fix.

## 4. Other attacked boundaries

- Source reading attachments, direct primitive representation restrictions, closed origin/formatter vocabulary, source3 admission and legacy omission were inspected; no additional finding.
- Exact bounded Gregorian normalization, offset extremes, Unix ranges and the shared native Rust/Go helpers were inspected against the written grammar; no additional finding. Existing implementation test results were read, not rerun.
- Scenario/occurrence correlation, source/epoch agreement, explicit unknown authority, old suite-format refusal and direct writer admission were traced; the reproduced defect is the Go mutable-origin handoff above.
- Standalone authored references explicitly require actual target-model re-admission; they are not claimed as certificates. Controlled fixtures do not establish production consumer instrumentation or cross-source calibration.
- Semantic attachment changes retain a typed diff3 record and legacy format refusal. No additional design judgement is raised.

## 5. Outside-worktree inventory and release

Every retained file and explicitly used storage root is listed below using the public aliases. Exact absolute paths are in the private inventory. The Go temporary directory was assigned scratch, not the global temporary filesystem. Go's ordinary shared compiler cache is listed as a storage root because individual cache entries are tool-managed. No cache or worktree was deleted. The own review lease `ess-clock-adversary-01a089ee` was released successfully; `lease-release.log` records exit0. Root owns correction, planning record and eventual integration/cleanup.

- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/authority-command.json`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/authority-first.exit`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/authority-first.log`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/candidate-files-private.json`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/candidate-integrity.json`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/candidate.diff`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/final-diff-stat.txt`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/generated-correspondence.json`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/go/essconform/README.md`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/go/essconform/clock_authority_adversary_test.go`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/go/essconform/clock_test.go`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/go/essconform/predicate.go`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/go/essconform/runtime.go`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/go/essconform/suite.go`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/go/essconform/suite.json`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/go/go.mod`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/lease-release.log`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/origin.log`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/paths-private.md`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/report-public.md`
- `local-evidence:ess-evolution-20260910/priority-wave/clock/adversary-1/gotmp`
- `local-evidence:go-build`

```findings
- file: crates/verify/ess-conformance/src/go/reading.go
  line: 143
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Go observation request origins alias the runner contract, so mutating the callback DTO makes an otherwise incompatible returned origin pass the scenario.
```