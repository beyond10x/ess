---
format: aep.planning-md/1
id: review-result:ess-count-writer-adversary-pass-1
kind: review-result
status: active
title: ESS count writer first adversarial pass
relations:
- reviews: story:a-skipped-scenario-is-not-a-failed-one
revision: 1
---
unit: ESS count writer first tests-only pass; a46bd7ff46ec8553bef4f48d4021514c8f175e82 plus three new test paths; base bd6d82f0d551fcf1cc2ec2eab65aab2fe7539947
verdict: CONFIRMED
cases: executed 401→405, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 shared Cargo infrastructure metadata path reported conservatively; no authored external files
needs-coordinator: route the three retained regressions for production correction
git --no-pager diff --stat

The tracked diff is empty because all additions are new untracked tests. Supplementary `git --no-pager diff --no-index --stat /dev/null <path>` output follows (each command exits 1 for the added file):

```text
 .../ess-conformance/tests/count_writer_pass1.rs    | 72 ++++++++++++++++++++++
 1 file changed, 72 insertions(+)
 .../edge/ess-cli/tests/count_writer_pass1.rs       | 211 +++++++++++++++++++++
 1 file changed, 211 insertions(+)
 .../fixtures/count-writer-pass1/target_test.go     | 44 ++++++++++++++++++++++
 1 file changed, 44 insertions(+)
```

Complete `git ls-files --others --exclude-standard` inventory:

```text
crates/edge/ess-cli/tests/count_writer_pass1.rs
crates/edge/ess-cli/tests/fixtures/count-writer-pass1/target_test.go
crates/verify/ess-conformance/tests/count_writer_pass1.rs
```

2. Focused cases and first execution evidence

All four new cases existed before their respective focused invocations. The first authored case ran alone before any other test execution in this pass. Every focused command selected exactly one Rust case. No compilation, fixture-construction or zero-selection failure occurred. Formatting afterward changed only the new test files, not their assertions.

| Case | Focused result | Measurement |
|---|---|---|
| `crates/verify/ess-conformance/tests/count_writer_pass1.rs`: `a_completed_run_cannot_be_rebound_to_different_admitted_original_bytes` | 0 passed, 1 failed; exit 101 | An untouched actual run of suite A is accepted by both new producers against suite B with the same provenance and IDs but a different final newline or an additional assertion that actually fails. Both new readers then accept the incorrectly rebound output against B. |
| `crates/edge/ess-cli/tests/count_writer_pass1.rs`: `generated_go_rejects_closed_predicate_metadata_before_any_target` | 0 passed, 1 failed; exit 101 | Three malformed original predicates rejected by Rust admission are accepted by generated Go: an unknown quantifier field, a dotted quantifier binding, and depth 33. Each Go process exits 0, creates the target marker and publishes report/2. A valid quantified predicate is the positive control. |
| Same CLI test file: `generated_go_abnormal_teardown_cannot_publish_a_completed_skip` | 0 passed, 1 failed; exit 101 | Ordinary unsupported execution produces one skip and a returned teardown error produces one actual failure. But EndScenario calling runtime.Goexit after the skip exits 0; it writes a complete skipped report when a destination exists and still returns success without a destination. |
| Same CLI test file: `generated_go_skip_counts_preserve_opaque_ids_and_strictness_without_a_destination` | 1 passed, 0 failed; exit 0 | Original CRLF bytes, newline/quote/combining-Unicode opaque IDs and colliding Go display names preserve three distinct report IDs. Failed count stays zero, skipped is three, both verdicts remain inconclusive, and strict mode fails without a report destination. |

The Go fixture is a newly added test under `crates/edge/ess-cli/tests/fixtures/count-writer-pass1/target_test.go`. It uses actual generated runtime/emitter bytes and public Run/Target callbacks; it does not replace runtime production source. The injected fixed clock returns 0 through the established countReportNow test seam. All generated packages, marker/report files and child-process logs are in assigned scratch. Each child log records exact command/environment additions/removals, cwd, exit, stdout and stderr; those records are printed in the focused Rust output below. Ten actual focused Go invocations ran (eight exit 0, two expected control invocations exit 1).

Common execution environment: RUSTC_WRAPPER, SCCACHE_SERVER_UDS and CARGO_TARGET_DIR unset; CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, CARGO_CACHE_RUSTC_INFO=0, CARGO_BUILD_JOBS=4 and CARGO_NET_OFFLINE=true. TMPDIR is `/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1`; GOCACHE and GOMODCACHE are its `go-cache` and `go-mod-cache` children. Cargo uses this worktree's existing target. No shared compiler daemon was started.

The complete package suite has not run when this focused section is assembled. Original focused output follows in execution order:

```text
cargo test -p ess-conformance --locked --test count_writer_pass1 a_completed_run_cannot_be_rebound_to_different_admitted_original_bytes -- --exact --nocapture
   Compiling syn v3.0.4
   Compiling serde_core v1.0.229
   Compiling serde v1.0.229
   Compiling serde_json v1.0.151
   Compiling hashbrown v0.17.1
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.20
   Compiling indexmap v2.14.1
   Compiling thiserror v2.0.20
   Compiling serde_yaml v0.9.34+deprecated
   Compiling schemars v0.8.22
   Compiling ess-primitives v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/crates/specify/ess-primitives)
   Compiling ess-domain v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/crates/specify/ess-domain)
   Compiling ess-compiler v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/crates/specify/ess-compiler)
   Compiling ess-gen v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/crates/generate/ess-gen)
   Compiling ess-conformance v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/crates/verify/ess-conformance)
    Finished `test` profile [unoptimized] target(s) in 12.05s
     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-74769b81d848fde9)

running 1 test

thread 'a_completed_run_cannot_be_rebound_to_different_admitted_original_bytes' (474978) panicked at crates/verify/ess-conformance/tests/count_writer_pass1.rs:54:5:
a paired writer must retain the identity actually issued to Runner::run_admitted:
standalone different final newline: executed=sha256:cd6e819192262cc91c566cf7bc2e854b298a68d7bae5192d23ba5328e17f5cce claimed=sha256:c671631f9932fd763d1a4fa1fbcea560c11560574ccfc19f5904ac03f81b0cea
{
  "completed_at": 1700000000300,
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
  "implementation": "billing-reference 0.19.0",
  "outcomes": {
    "error": [],
    "failed": [],
    "passed": [
      "review.count/authored/one"
    ],
    "skipped": [],
    "unsupported": []
  },
  "policy": "complete-selection/1",
  "producer_profile": "rust-scenario-status/1",
  "spec_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "specification": "review/v1",
  "suite": {
    "digest": "sha256:c671631f9932fd763d1a4fa1fbcea560c11560574ccfc19f5904ac03f81b0cea",
    "digest_profile": "sha256-json-bytes/1",
    "version": "ess-conformance/4"
  }
}

detailed different final newline: executed=sha256:cd6e819192262cc91c566cf7bc2e854b298a68d7bae5192d23ba5328e17f5cce claimed=sha256:c671631f9932fd763d1a4fa1fbcea560c11560574ccfc19f5904ac03f81b0cea
{
  "format": "ess-conformance-run/2",
  "scenarios": [
    {
      "checks": [],
      "duration_ms": 100,
      "purpose": "An exact original suite",
      "scenario": "review.count/authored/one",
      "status": "passed"
    }
  ],
  "started_at": 1700000000000,
  "summary": {
    "completed_at": 1700000000300,
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
    "implementation": "billing-reference 0.19.0",
    "outcomes": {
      "error": [],
      "failed": [],
      "passed": [
        "review.count/authored/one"
      ],
      "skipped": [],
      "unsupported": []
    },
    "policy": "complete-selection/1",
    "producer_profile": "rust-scenario-status/1",
    "spec_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "specification": "review/v1",
    "suite": {
      "digest": "sha256:c671631f9932fd763d1a4fa1fbcea560c11560574ccfc19f5904ac03f81b0cea",
      "digest_profile": "sha256-json-bytes/1",
      "version": "ess-conformance/4"
    }
  }
}

standalone different executed assertion: executed=sha256:cd6e819192262cc91c566cf7bc2e854b298a68d7bae5192d23ba5328e17f5cce claimed=sha256:76816b29b7baf967ddd3be19267a326556c9b345ca5f46ebd749f1ffac1f7d82
{
  "completed_at": 1700000000300,
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
  "implementation": "billing-reference 0.19.0",
  "outcomes": {
    "error": [],
    "failed": [],
    "passed": [
      "review.count/authored/one"
    ],
    "skipped": [],
    "unsupported": []
  },
  "policy": "complete-selection/1",
  "producer_profile": "rust-scenario-status/1",
  "spec_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "specification": "review/v1",
  "suite": {
    "digest": "sha256:76816b29b7baf967ddd3be19267a326556c9b345ca5f46ebd749f1ffac1f7d82",
    "digest_profile": "sha256-json-bytes/1",
    "version": "ess-conformance/4"
  }
}

detailed different executed assertion: executed=sha256:cd6e819192262cc91c566cf7bc2e854b298a68d7bae5192d23ba5328e17f5cce claimed=sha256:76816b29b7baf967ddd3be19267a326556c9b345ca5f46ebd749f1ffac1f7d82
{
  "format": "ess-conformance-run/2",
  "scenarios": [
    {
      "checks": [],
      "duration_ms": 100,
      "purpose": "An exact original suite",
      "scenario": "review.count/authored/one",
      "status": "passed"
    }
  ],
  "started_at": 1700000000000,
  "summary": {
    "completed_at": 1700000000300,
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
    "implementation": "billing-reference 0.19.0",
    "outcomes": {
      "error": [],
      "failed": [],
      "passed": [
        "review.count/authored/one"
      ],
      "skipped": [],
      "unsupported": []
    },
    "policy": "complete-selection/1",
    "producer_profile": "rust-scenario-status/1",
    "spec_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "specification": "review/v1",
    "suite": {
      "digest": "sha256:76816b29b7baf967ddd3be19267a326556c9b345ca5f46ebd749f1ffac1f7d82",
      "digest_profile": "sha256-json-bytes/1",
      "version": "ess-conformance/4"
    }
  }
}

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test a_completed_run_cannot_be_rebound_to_different_admitted_original_bytes ... FAILED

failures:

failures:
    a_completed_run_cannot_be_rebound_to_different_admitted_original_bytes

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-conformance --test count_writer_pass1`
exit: 101
```

```text
cargo test -p ess-cli --locked --test count_writer_pass1 generated_go_rejects_closed_predicate_metadata_before_any_target -- --exact --nocapture
   Compiling ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.28s
     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-d029e2dedf54b30d)

running 1 test
cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316/valid.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316/valid.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:1380: the target does not support this scenario: the target does not expose this
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316/unknown-quantifier-field.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316/unknown-quantifier-field.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:1380: the target does not support this scenario: the target does not expose this
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316/invalid-quantifier-binding.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316/invalid-quantifier-binding.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:1380: the target does not support this scenario: the target does not expose this
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316/excess-predicate-depth.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-500316/excess-predicate-depth.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:1380: the target does not support this scenario: the target does not expose this
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:


thread 'generated_go_rejects_closed_predicate_metadata_before_any_target' (500317) panicked at crates/edge/ess-cli/tests/count_writer_pass1.rs:125:5:
new admitted execution must validate the frozen predicate envelope before targets:
unknown-quantifier-field: exit=Some(0), target=true, report=true
invalid-quantifier-binding: exit=Some(0), target=true, report=true
excess-predicate-depth: exit=Some(0), target=true, report=true
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test generated_go_rejects_closed_predicate_metadata_before_any_target ... FAILED

failures:

failures:
    generated_go_rejects_closed_predicate_metadata_before_any_target

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 4.01s

error: test failed, to rerun pass `-p ess-cli --test count_writer_pass1`
exit: 101
```

```text
cargo test -p ess-cli --locked --test count_writer_pass1 generated_go_abnormal_teardown_cannot_publish_a_completed_skip -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.09s
     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-d029e2dedf54b30d)

running 1 test
cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071/skip.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071/skip.marker" REVIEW_MODE="skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:727: Review completion and original admission
    runtime.go:1380: step 0: the target does not expose `review.count.Do`
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071/skip-end-error.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071/skip-end-error.marker" REVIEW_MODE="skip-end-error" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(1)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:727: Review completion and original admission
    runtime.go:1380: step 0: the target does not expose `review.count.Do`
    runtime.go:722: end: teardown did return an error
--- FAIL: TestReview (0.00s)
    --- FAIL: TestReview/review.count/authored/one (0.00s)
FAIL
FAIL	countreview/essconform	0.002s
FAIL

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT -u ESS_REPORT_OUT ESS_REPORT_FORMAT="2" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071/abnormal-false.marker" REVIEW_MODE="skip-end-goexit" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:727: Review completion and original admission
    runtime.go:1380: step 0: the target does not expose `review.count.Do`
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071/abnormal-true.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-504071/abnormal-true.marker" REVIEW_MODE="skip-end-goexit" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:727: Review completion and original admission
    runtime.go:1380: step 0: the target does not expose `review.count.Do`
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:


thread 'generated_go_abnormal_teardown_cannot_publish_a_completed_skip' (504072) panicked at crates/edge/ess-cli/tests/count_writer_pass1.rs:166:5:
an EndScenario that never returns cannot finish a report/2 invocation:
destination=false: exit=Some(0), report=false
destination=true: exit=Some(0), report=true
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test generated_go_abnormal_teardown_cannot_publish_a_completed_skip ... FAILED

failures:

failures:
    generated_go_abnormal_teardown_cannot_publish_a_completed_skip

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.57s

error: test failed, to rerun pass `-p ess-cli --test count_writer_pass1`
exit: 101
```

```text
cargo test -p ess-cli --locked --test count_writer_pass1 generated_go_skip_counts_preserve_opaque_ids_and_strictness_without_a_destination -- --exact --nocapture
    Finished `test` profile [unoptimized] target(s) in 0.08s
     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-d029e2dedf54b30d)

running 1 test
cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-opaque-ids-504427
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-opaque-ids-504427" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-opaque-ids-504427/diagnostic.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-opaque-ids-504427/diagnostic.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 3 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count.Type/invariant/at/review.count.Rows/line_one
    runtime.go:1380: the target does not support this scenario: the target does not expose this
=== RUN   TestReview/review.count.Type/invariant/at/review.count.Rows/line_one#01
    runtime.go:1380: the target does not support this scenario: the target does not expose this
=== RUN   TestReview/review.count.Type/invariant/at/review.count.Rows/quote"_é
    runtime.go:1380: the target does not support this scenario: the target does not expose this
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count.Type/invariant/at/review.count.Rows/line_one (0.00s)
    --- SKIP: TestReview/review.count.Type/invariant/at/review.count.Rows/line_one#01 (0.00s)
    --- SKIP: TestReview/review.count.Type/invariant/at/review.count.Rows/quote"_é (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-opaque-ids-504427
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-opaque-ids-504427" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_REPORT_OUT ESS_CONFORMANCE_STRICT="1" ESS_REPORT_FORMAT="2" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-opaque-ids-504427/strict.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(1)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 3 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count.Type/invariant/at/review.count.Rows/line_one
    runtime.go:1380: the target does not support this scenario: the target does not expose this
=== RUN   TestReview/review.count.Type/invariant/at/review.count.Rows/line_one#01
    runtime.go:1380: the target does not support this scenario: the target does not expose this
=== RUN   TestReview/review.count.Type/invariant/at/review.count.Rows/quote"_é
    runtime.go:1380: the target does not support this scenario: the target does not expose this
=== NAME  TestReview
    review_test.go:38: strict conformance: inconclusive (legacy suite coverage is unknown)
--- FAIL: TestReview (0.00s)
    --- SKIP: TestReview/review.count.Type/invariant/at/review.count.Rows/line_one (0.00s)
    --- SKIP: TestReview/review.count.Type/invariant/at/review.count.Rows/line_one#01 (0.00s)
    --- SKIP: TestReview/review.count.Type/invariant/at/review.count.Rows/quote"_é (0.00s)
FAIL
FAIL	countreview/essconform	0.002s
FAIL

stderr:

test generated_go_skip_counts_preserve_opaque_ids_and_strictness_without_a_destination ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.36s

exit: 0
```

3. Package execution

The before count is the supplied final implementor run: 401 passed, zero failed/ignored, 31 runner summaries, documented in target/review-boundaries-8/implementor-report.md and final-suite.log. No preemptive baseline was run. The literal assigned command below stopped at the first failing test binary: 69 executed, 67 passed, 2 failed, 8 summaries, exit 101. It is retained in full and is not labeled complete package coverage.

```text
cargo test -p ess-conformance -p ess-cli --locked
   Compiling ess-conformance v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/crates/verify/ess-conformance)
    Finished `test` profile [unoptimized] target(s) in 0.30s
     Running unittests src/main.rs (target/debug/deps/ess-d3edf4c0ecde1ae7)

running 11 tests
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/authored_scenarios.rs (target/debug/deps/authored_scenarios-665b0ff538fabc92)

running 25 tests
test author_nested_only ... ok
test author_nonmatching_only ... ok
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test author_empty ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test web_empty ... ok
test go_nonmatching_only ... ok
test ir_nested_only ... ok
test go_empty ... ok
test run_nested_only ... ok
test run_empty ... ok
test web_nested_only ... ok
test run_nonmatching_only ... ok
test web_nonmatching_only ... ok
test ir_nonmatching_only ... ok
test ir_empty ... ok
test go_nested_only ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s

     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-ddf0cc8492bb33b1)

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

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

     Running tests/authored_site.rs (target/debug/deps/authored_site-21399ca4863fa792)

running 9 tests
test binary_downloads_are_not_silently_decoded ... ok
test an_explicit_missing_front_page_is_an_error ... ok
test asset_symlink_output_is_refused_before_any_page_changes ... ok
test a_page_identity_can_itself_end_in_html ... ok
test publication_without_strict_mode_preserves_unpublished_links ... ok
test strict_links_check_local_and_cross_page_fragments ... ok
test selected_sources_resolve_after_relocation_with_queries_fragments_and_downloads ... ok
test assets_cannot_collide_with_generated_output_or_escape_it ... ok
test undeclared_existing_and_escaping_targets_are_refused_with_source_lines ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/command_surface.rs (target/debug/deps/command_surface-9ab8f7e62cc8cbe3)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-2d5107ca256f28e2)

running 4 tests
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/count_reports.rs (target/debug/deps/count_reports-f089e70daf74a676)

running 3 tests
test count_report_opt_in_has_a_distinct_detailed_surface_and_unknown_coverage ... ok
test count_cli_configuration_and_original_suite_refusals_preserve_destinations ... ok
test count_cli_preserves_default_bytes_and_standalone_detailed_pairing ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s

     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-d029e2dedf54b30d)

running 3 tests
test generated_go_skip_counts_preserve_opaque_ids_and_strictness_without_a_destination ... ok
test generated_go_abnormal_teardown_cannot_publish_a_completed_skip ... FAILED
test generated_go_rejects_closed_predicate_metadata_before_any_target ... FAILED

failures:

---- generated_go_abnormal_teardown_cannot_publish_a_completed_skip stdout ----
cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804/skip.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804/skip.marker" REVIEW_MODE="skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:727: Review completion and original admission
    runtime.go:1380: step 0: the target does not expose `review.count.Do`
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804/skip-end-error.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804/skip-end-error.marker" REVIEW_MODE="skip-end-error" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(1)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:727: Review completion and original admission
    runtime.go:1380: step 0: the target does not expose `review.count.Do`
    runtime.go:722: end: teardown did return an error
--- FAIL: TestReview (0.00s)
    --- FAIL: TestReview/review.count/authored/one (0.00s)
FAIL
FAIL	countreview/essconform	0.003s
FAIL

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT -u ESS_REPORT_OUT ESS_REPORT_FORMAT="2" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804/abnormal-false.marker" REVIEW_MODE="skip-end-goexit" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:727: Review completion and original admission
    runtime.go:1380: step 0: the target does not expose `review.count.Do`
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804/abnormal-true.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-516804/abnormal-true.marker" REVIEW_MODE="skip-end-goexit" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:727: Review completion and original admission
    runtime.go:1380: step 0: the target does not expose `review.count.Do`
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:


thread 'generated_go_abnormal_teardown_cannot_publish_a_completed_skip' (516805) panicked at crates/edge/ess-cli/tests/count_writer_pass1.rs:166:5:
an EndScenario that never returns cannot finish a report/2 invocation:
destination=false: exit=Some(0), report=false
destination=true: exit=Some(0), report=true
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- generated_go_rejects_closed_predicate_metadata_before_any_target stdout ----
cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804/valid.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804/valid.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:1380: the target does not support this scenario: the target does not expose this
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804/unknown-quantifier-field.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804/unknown-quantifier-field.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:1380: the target does not support this scenario: the target does not expose this
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804/invalid-quantifier-binding.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804/invalid-quantifier-binding.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:1380: the target does not support this scenario: the target does not expose this
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804/excess-predicate-depth.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-516804/excess-predicate-depth.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:1380: the target does not support this scenario: the target does not expose this
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:


thread 'generated_go_rejects_closed_predicate_metadata_before_any_target' (516806) panicked at crates/edge/ess-cli/tests/count_writer_pass1.rs:125:5:
new admitted execution must validate the frozen predicate envelope before targets:
unknown-quantifier-field: exit=Some(0), target=true, report=true
invalid-quantifier-binding: exit=Some(0), target=true, report=true
excess-predicate-depth: exit=Some(0), target=true, report=true


failures:
    generated_go_abnormal_teardown_cannot_publish_a_completed_skip
    generated_go_rejects_closed_predicate_metadata_before_any_target

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.13s

error: test failed, to rerun pass `-p ess-cli --test count_writer_pass1`
exit: 101
```

The identical package selection was then completed with --no-fail-fast, so the retained regressions could not prevent inherited cases from executing. This ran 405 cases: 402 passed, 3 failed, zero ignored, 33 runner summaries, exit 101. All 401 inherited cases pass; the four additions supply one pass and three failures. Header counts describe this complete invocation, not the sum of repeated runs. New-test Rust formatting check exited 0; the new Go fixture was formatted without touching runtime source. No broader repository gate, site build or extra attack was run.

```text
cargo test -p ess-conformance -p ess-cli --locked --no-fail-fast
    Finished `test` profile [unoptimized] target(s) in 0.10s
     Running unittests src/main.rs (target/debug/deps/ess-d3edf4c0ecde1ae7)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/authored_scenarios.rs (target/debug/deps/authored_scenarios-665b0ff538fabc92)

running 25 tests
test author_nonmatching_only ... ok
test author_nested_only ... ok
test author_empty ... ok
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test ir_nested_only ... ok
test web_empty ... ok
test web_nested_only ... ok
test go_empty ... ok
test ir_empty ... ok
test ir_nonmatching_only ... ok
test run_empty ... ok
test run_nested_only ... ok
test go_nested_only ... ok
test web_nonmatching_only ... ok
test go_nonmatching_only ... ok
test run_nonmatching_only ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-ddf0cc8492bb33b1)

running 9 tests
test non_utf8_matching_content_is_refused_before_any_output_write ... ok
test selected_directory_symlink_reports_the_requested_path_and_stays_shallow ... ok
test empty_selection_cannot_follow_or_replace_an_output_symlink ... ok
test matching_broken_links_are_read_errors_even_beside_a_valid_source ... ok
test lexical_selection_order_decides_the_first_read_error ... ok
test committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners ... ok
test omitted_scenarios_ignore_a_poisoned_working_directory_default ... ok
test empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner ... ok
test valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s

     Running tests/authored_site.rs (target/debug/deps/authored_site-21399ca4863fa792)

running 9 tests
test an_explicit_missing_front_page_is_an_error ... ok
test binary_downloads_are_not_silently_decoded ... ok
test asset_symlink_output_is_refused_before_any_page_changes ... ok
test a_page_identity_can_itself_end_in_html ... ok
test publication_without_strict_mode_preserves_unpublished_links ... ok
test strict_links_check_local_and_cross_page_fragments ... ok
test assets_cannot_collide_with_generated_output_or_escape_it ... ok
test undeclared_existing_and_escaping_targets_are_refused_with_source_lines ... ok
test selected_sources_resolve_after_relocation_with_queries_fragments_and_downloads ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/command_surface.rs (target/debug/deps/command_surface-9ab8f7e62cc8cbe3)

running 5 tests
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test the_help_offers_exactly_the_four_areas ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-2d5107ca256f28e2)

running 4 tests
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/count_reports.rs (target/debug/deps/count_reports-f089e70daf74a676)

running 3 tests
test count_report_opt_in_has_a_distinct_detailed_surface_and_unknown_coverage ... ok
test count_cli_configuration_and_original_suite_refusals_preserve_destinations ... ok
test count_cli_preserves_default_bytes_and_standalone_detailed_pairing ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s

     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-d029e2dedf54b30d)

running 3 tests
test generated_go_skip_counts_preserve_opaque_ids_and_strictness_without_a_destination ... ok
test generated_go_abnormal_teardown_cannot_publish_a_completed_skip ... FAILED
test generated_go_rejects_closed_predicate_metadata_before_any_target ... FAILED

failures:

---- generated_go_abnormal_teardown_cannot_publish_a_completed_skip stdout ----
cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190/skip.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190/skip.marker" REVIEW_MODE="skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:727: Review completion and original admission
    runtime.go:1380: step 0: the target does not expose `review.count.Do`
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190/skip-end-error.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190/skip-end-error.marker" REVIEW_MODE="skip-end-error" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(1)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:727: Review completion and original admission
    runtime.go:1380: step 0: the target does not expose `review.count.Do`
    runtime.go:722: end: teardown did return an error
--- FAIL: TestReview (0.00s)
    --- FAIL: TestReview/review.count/authored/one (0.00s)
FAIL
FAIL	countreview/essconform	0.003s
FAIL

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT -u ESS_REPORT_OUT ESS_REPORT_FORMAT="2" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190/abnormal-false.marker" REVIEW_MODE="skip-end-goexit" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:727: Review completion and original admission
    runtime.go:1380: step 0: the target does not expose `review.count.Do`
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190/abnormal-true.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-teardown-522190/abnormal-true.marker" REVIEW_MODE="skip-end-goexit" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:727: Review completion and original admission
    runtime.go:1380: step 0: the target does not expose `review.count.Do`
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.006s

stderr:


thread 'generated_go_abnormal_teardown_cannot_publish_a_completed_skip' (522191) panicked at crates/edge/ess-cli/tests/count_writer_pass1.rs:166:5:
an EndScenario that never returns cannot finish a report/2 invocation:
destination=false: exit=Some(0), report=false
destination=true: exit=Some(0), report=true
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- generated_go_rejects_closed_predicate_metadata_before_any_target stdout ----
cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190/valid.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190/valid.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:1380: the target does not support this scenario: the target does not expose this
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190/unknown-quantifier-field.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190/unknown-quantifier-field.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:1380: the target does not support this scenario: the target does not expose this
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190/invalid-quantifier-binding.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190/invalid-quantifier-binding.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:1380: the target does not support this scenario: the target does not expose this
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190/excess-predicate-depth.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1/go-predicate-522190/excess-predicate-depth.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReview$"
exit: Some(0)
stdout:
=== RUN   TestReview
    review_test.go:38: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReview/review.count/authored/one
    runtime.go:1380: the target does not support this scenario: the target does not expose this
--- PASS: TestReview (0.00s)
    --- SKIP: TestReview/review.count/authored/one (0.00s)
PASS
ok  	countreview/essconform	0.002s

stderr:


thread 'generated_go_rejects_closed_predicate_metadata_before_any_target' (522192) panicked at crates/edge/ess-cli/tests/count_writer_pass1.rs:125:5:
new admitted execution must validate the frozen predicate envelope before targets:
unknown-quantifier-field: exit=Some(0), target=true, report=true
invalid-quantifier-binding: exit=Some(0), target=true, report=true
excess-predicate-depth: exit=Some(0), target=true, report=true


failures:
    generated_go_abnormal_teardown_cannot_publish_a_completed_skip
    generated_go_rejects_closed_predicate_metadata_before_any_target

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.06s

error: test failed, to rerun pass `-p ess-cli --test count_writer_pass1`
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-d32c13ce3b168b5c)

running 4 tests
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_rust_cli_writes ... ok
test a_binding_function_capture_is_refused_before_web_cli_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-49c13e90a6599768)

running 12 tests
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test count_report_skip_only_is_inconclusive_without_actual_failures ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test count_retained_runtime_preserves_legacy_behavior_and_does_not_gain_version_checks ... ok
test count_go_empty_selection_is_inconclusive_and_clock_conversion_is_checked ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test count_go_actual_producers_keep_skip_error_and_teardown_categories ... ok
test count_go_refusals_precede_targets_and_incomplete_runs_never_publish ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.30s

     Running tests/model_types.rs (target/debug/deps/model_types-81eb3a002a8b3064)

running 2 tests
test root_selection_and_output_refusals_preserve_existing_files ... ok
test each_target_retains_the_same_model_selection_and_distinct_input_provenance ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/normalization.rs (target/debug/deps/normalization-9776ecd4b47f8e04)

running 13 tests
test unsupported_go_pattern_has_no_successful_partial_artifact ... ok
test incompatible_later_destination_preserves_the_entire_existing_output ... ok
test output_symlinks_and_hardlinks_are_refused_without_mutation ... ok
test generated_paths_cannot_replace_canonical_source_inputs_or_follow_links ... ok
test stale_bundle_missing_source_and_unknown_dispatch_refuse ... ok
test qualified_go_base64_pattern_is_published_without_decoding_the_value ... ok
test checked_recipe_and_run_are_deterministic_with_json_only_stdout ... ok
test explicit_binary64_inputs_run_through_the_cli_without_weakening_version_one ... ok
test output_cannot_replace_any_declared_input ... ok
test generation_checks_refuse_before_creating_or_changing_destinations ... ok
test failed_check_or_execution_preserves_output_and_does_not_emit_a_result ... ok
test generated_libraries_match_the_api_and_drift_check_never_repairs_files ... ok
test model_sources_are_compiled_pinned_and_protected_by_the_cli ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s

     Running tests/openapi_accounting.rs (target/debug/deps/openapi_accounting-3ce279f6417aad27)

running 6 tests
test complete_projection_preserves_supported_yaml_through_both_spellings ... ok
test both_import_spellings_write_the_accounted_envelope_for_every_presentation ... ok
test a_refused_import_keeps_existing_output_and_creates_no_new_file ... ok
test legacy_projection_refuses_before_destination_mutation ... ok
test partial_projection_reports_the_durable_gap_and_preserves_destinations ... ok
test unresolved_projection_reports_the_exact_reference_and_preserves_destinations ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running tests/openapi_adversary_pass1.rs (target/debug/deps/openapi_adversary_pass1-d92f800d3110935f)

running 2 tests
test cli_rejects_duplicate_keys_and_tampered_accounting_before_output ... ok
test cli_rejects_unknown_unit_fields_before_any_projection_output ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/openapi_adversary_pass2.rs (target/debug/deps/openapi_adversary_pass2-38d06ca5d76d5677)

running 1 test
test same_path_projection_refusal_preserves_the_unadmitted_input ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/output_containment.rs (target/debug/deps/output_containment-e1e5a2958a09cb3e)

running 19 tests
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-a5eb934fe97baa0e)

running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/schema_bundle.rs (target/debug/deps/schema_bundle-0d0e1ff01b95e144)

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

     Running tests/target_failure.rs (target/debug/deps/target_failure-243770058fcedcec)

running 1 test
test fatal_synthesis_preserves_destinations_and_has_a_typed_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

     Running unittests src/lib.rs (target/debug/deps/ess_conformance-842414d0dd52814a)

running 69 tests
test counts::tests::exact_unsigned_scalar_vectors_do_not_use_binary64 ... ok
test counts::tests::payload_number_and_utf8_canonical_profile_is_frozen_separately_from_scalars ... ok
test decision::tests::a_refusal_renders_the_predicate_the_command_and_every_reason ... ok
test decision::tests::exactly_one_reason_says_another_candidate_would_help ... ok
test decision::tests::a_decision_reads_its_two_other_cases_as_neither_satisfied_nor_the_other ... ok
test evidence::tests::a_standalone_report_carries_every_field_an_adapter_needs ... ok
test evidence::tests::report_readers_refuse_more_nonpasses_than_executed_scenarios ... ok
test evidence::tests::the_closed_report_round_trips_with_identical_canonical_bytes ... ok
test faulty::tests::every_fault_says_what_it_is_and_where_it_goes ... ok
test faulty::tests::a_fault_is_injected_into_the_system_that_declares_what_it_breaks ... ok
test evidence::tests::report_readers_refuse_nonpass_count_and_list_disagreement ... ok
test faulty::tests::no_two_faults_claim_the_same_scenario ... ok
test faulty::tests::only_the_two_faults_the_boundary_cannot_express_are_injected_in_the_implementation ... ok
test evidence::tests::unknown_report_fields_are_refused ... ok
test evidence::tests::report_readers_do_not_guess_a_producer_from_status_vocabulary ... ok
test evidence::tests::report_readers_preserve_go_producer_bytes_and_historical_nonpass_counts ... ok
test input::tests::every_primitive_projects_to_the_one_fact_value_that_can_hold_it ... ok
test go::tests::every_go_file_is_in_the_package_the_readme_names ... ok
test report::tests::a_diagnostic_answers_all_five_of_the_questions_a_failure_has_to_answer ... ok
test report::tests::an_unsupported_scenario_makes_the_run_fail_rather_than_look_like_a_pass ... ok
test report::tests::a_scenario_status_is_the_strongest_of_its_checks_and_a_contradiction_outranks_everything ... ok
test report::tests::a_quoted_input_reads_as_the_call_that_was_made ... ok
test input::tests::shape_errors_render_one_per_line_and_name_the_input_root_by_name ... ok
test input::tests::a_primitive_refuses_a_node_of_the_wrong_shape_rather_than_coercing_it ... ok
test report::tests::every_check_code_has_a_distinct_name_and_a_rule_sentence ... ok
test runner::tests::a_nested_row_binds_the_paths_a_predicate_spells ... ok
test runner::tests::a_position_in_a_view_that_declares_no_order_is_a_suite_defect ... ok
test runner::tests::a_declared_order_is_checked_on_adjacent_rows_and_the_next_key_breaks_a_tie ... ok
test runner::tests::a_count_with_neither_bound_is_a_suite_defect_and_not_a_satisfied_assertion ... ok
test runner::tests::a_ranking_key_a_row_does_not_publish_is_undecidable_rather_than_out_of_order ... ok
test runner::tests::a_position_names_both_ends_and_a_row_that_is_not_there_is_not_a_match ... ok
test runner::tests::an_order_over_fewer_than_two_rows_holds_and_does_not_double_as_a_non_emptiness_claim ... ok
test runner::tests::a_count_is_the_half_of_an_ordering_claim_that_says_the_rows_were_there ... ok
test runner::tests::a_view_that_holds_nothing_does_not_satisfy_an_invariant_by_being_empty ... ok
test runner::tests::ids_come_from_the_suite_and_from_nothing_ambient ... ok
test runner::tests::the_runners_clock_advances_on_every_read_so_a_deadline_can_bound_anything ... ok
test scenario::tests::a_declared_leaf_admits_what_its_type_admits_and_absence_only_where_the_type_permits_it ... ok
test go::tests::the_runner_is_a_constant_and_only_the_suite_moves ... ok
test evidence::tests::report_readers_refuse_unknown_report_formats ... ok
test scenario::tests::a_payload_shape_round_trips_through_the_form_a_suite_is_stored_in ... ok
test runner::tests::a_predicate_a_row_cannot_answer_is_reported_rather_than_retried ... ok
test scenario::tests::a_purpose_is_one_line_and_says_something ... ok
test scenario::tests::a_scenario_id_names_the_construct_it_exercises_rather_than_its_position ... ok
test runner::tests::an_empty_field_set_means_a_row_exists_and_not_that_anything_will_do ... ok
test scenario::tests::a_transition_ref_refuses_a_name_no_lifecycle_can_declare ... ok
test scenario::tests::a_suite_refuses_a_second_scenario_under_one_id ... ok
test scenario::tests::an_invariant_scenario_is_keyed_by_the_entity_and_the_branch_and_never_by_a_position ... ok
test scenario::tests::every_binding_aspect_is_in_the_list_that_is_walked_to_produce_them ... ok
test scenario::tests::a_semantic_reference_renders_the_way_the_design_writes_one ... ok
test scenario::tests::the_ids_of_a_suite_sort_the_way_a_reader_sorts_the_file ... ok
test scenario::tests::a_scenario_id_that_names_no_construct_is_refused ... ok
test synthesize::tests::a_refusal_names_the_construct_the_code_and_the_repair ... ok
test scenario::tests::two_scenarios_about_the_same_thing_in_the_same_way_are_one_id ... ok
test scenario::tests::every_scenario_id_reads_back_from_the_form_a_report_prints ... ok
test scenario::tests::a_suite_format_from_a_later_build_is_refused_rather_than_guessed ... ok
test synthesize::tests::every_refusal_carries_a_distinct_code_in_one_family ... ok
test witness::tests::a_text_witness_is_its_own_path_so_two_fields_of_one_type_never_agree ... ok
test witness::tests::an_enum_offers_every_variant_it_declares_and_the_first_one_only_once ... ok
test web::tests::no_comparison_sits_in_a_text_node_mustache ... ok
test witness::tests::an_integer_leaf_is_never_offered_a_fractional_candidate ... ok
test witness::tests::the_alternatives_for_a_number_are_the_guards_own_literals_either_side ... ok
test evidence::tests::report_readers_refuse_malformed_and_unsupported_suite_versions ... ok
test witness::tests::the_candidate_count_is_bounded_however_many_fields_a_guard_reads ... ok
test witness::tests::two_uuid_witnesses_differ_and_neither_moves_when_a_third_field_appears ... ok
test web::tests::the_page_calls_nothing_the_player_does_not_return ... ok
test evidence::tests::report_readers_refuse_nonpass_entries_without_a_known_nonpass_status ... ok
test web::tests::the_page_is_specification_neutral ... ok
test evidence::tests::report_readers_refuse_status_claims_that_contradict_the_list ... ok
test evidence::tests::report_readers_preserve_rust_producer_bytes_for_every_supported_suite ... ok

test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/adversary_expression_pass1.rs (target/debug/deps/adversary_expression_pass1-6656a5dcfd0ce7b4)

running 4 tests
test authored_semantic_depth_is_distinct_from_the_projection_limit ... ok
test malformed_bound_operands_refuse_before_the_collection_projection_gap ... ok
test authored_optional_enum_membership_rejects_later_invalid_values ... ok
test authored_rows_preserve_scalar_controls_and_reject_aggregate_and_unprojected_reads ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/authored.rs (target/debug/deps/authored-3bd103acda9b8c58)

running 49 tests
test a_halt_of_a_listing_the_model_calls_eventual_retries_because_the_model_said_so ... ok
test a_name_a_closed_set_does_not_have_is_refused_with_the_set ... ok
test a_declared_field_nothing_supplies_is_refused_by_name ... ok
test a_field_the_surface_does_not_declare_is_refused_by_name ... ok
test a_document_that_is_not_one_is_refused_rather_than_read_as_an_empty_scenario ... ok
test a_format_this_build_does_not_implement_is_refused_before_anything_is_read ... ok
test a_domain_the_model_does_not_declare_is_refused_by_name ... ok
test a_command_the_model_does_not_declare_is_refused_by_name ... ok
test a_halt_claimed_of_a_listing_with_no_declared_order_is_refused_by_the_code_that_already_says_so ... ok
test a_bounded_negative_that_forbids_no_event_is_refused ... ok
test a_halt_after_no_rows_at_all_is_refused_rather_than_compiled ... ok
test a_halt_compiles_to_a_step_of_its_own_and_not_to_a_claim_about_rows ... ok
test a_halt_stated_beside_another_claim_is_two_assertions_filed_as_one ... ok
test a_position_in_a_view_that_declares_no_order_is_refused ... ok
test a_claim_the_timelines_own_instants_contradict_is_refused ... ok
test a_predicate_reading_something_the_view_does_not_publish_is_refused ... ok
test a_positional_claim_takes_the_order_from_the_view_rather_than_from_the_author ... ok
test a_scenario_that_runs_nothing_is_refused_rather_than_counted_as_a_check ... ok
test a_scenario_compiles_to_the_id_the_domain_and_the_name_make ... ok
test a_reference_where_the_suite_compares_a_value_it_carries_is_refused ... ok
test a_state_the_lifecycle_does_not_declare_is_refused_as_a_state_and_not_as_a_variant ... ok
test a_value_read_off_an_event_nothing_required_is_refused ... ok
test a_timeline_whose_instants_do_not_ascend_is_refused ... ok
test a_value_the_declared_type_does_not_admit_is_refused_where_it_sits ... ok
test a_view_the_model_does_not_declare_is_refused_by_name ... ok
test a_window_measured_from_an_instant_nothing_marked_is_refused_with_the_ones_that_are ... ok
test a_window_of_no_seconds_is_refused_rather_than_compiled_into_a_check_that_cannot_fail ... ok
test a_window_that_states_other_than_one_bound_is_refused ... ok
test an_act_cannot_open_a_window_at_its_own_instant ... ok
test an_error_the_model_does_not_declare_is_refused_by_name ... ok
test an_assertion_that_states_other_than_one_claim_is_refused ... ok
test an_actor_the_specification_does_not_grant_the_command_is_refused ... ok
test an_actor_the_model_does_not_declare_is_refused_by_name ... ok
test an_entity_the_model_does_not_declare_is_refused_by_name ... ok
test an_elapsed_claim_compiles_to_the_four_steps_that_carry_it_and_they_come_before_the_act ... ok
test an_event_the_model_does_not_declare_is_refused_by_name ... ok
test an_instance_named_before_anything_binds_it_is_refused ... ok
test an_instance_the_arrangement_does_not_declare_is_refused_by_name ... ok
test an_outcome_the_command_does_not_declare_is_refused_with_the_ones_it_does ... ok
test an_instance_bound_to_a_field_that_cannot_hold_an_identity_is_refused ... ok
test the_steps_are_the_vocabulary_a_generated_scenario_already_uses ... ok
test the_order_the_files_are_handed_over_in_does_not_reach_the_result ... ok
test authored_aggregate_presence_keeps_026_and_valid_scalar_reads_keep_the_predicate ... ok
test the_committed_billing_suite_holds_the_authored_scenario_beside_the_generated_ones ... ok
test authored_predicate_operand_errors_have_their_own_refusal ... ok
test one_name_for_two_instants_is_refused_rather_than_read_as_the_later_one ... ok
test two_files_naming_one_scenario_are_refused_rather_than_one_displacing_the_other ... ok
test every_cause_is_reachable_from_a_document ... ok
test two_compilations_of_one_file_produce_identical_bytes ... ok

test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/count_reports.rs (target/debug/deps/count_reports-11d75868732b80a4)

running 6 tests
test legacy_dto_is_not_original_byte_admission_and_typed_execution_still_checks_versions ... ok
test suite_admission_closes_structural_variants_before_target_identity ... ok
test detailed_admission_checks_fields_outcome_order_and_checked_time ... ok
test exact_bytes_profiles_partition_and_identity_cannot_be_guessed ... ok
test new_scalar_tokens_are_exact_unsigned_in_both_surfaces ... ok
test actual_rust_producer_pairs_preserve_categories_precedence_empty_and_high_u64 ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-2df70a4c14376075)

running 1 test
test a_completed_run_cannot_be_rebound_to_different_admitted_original_bytes ... FAILED

failures:

---- a_completed_run_cannot_be_rebound_to_different_admitted_original_bytes stdout ----

thread 'a_completed_run_cannot_be_rebound_to_different_admitted_original_bytes' (528769) panicked at crates/verify/ess-conformance/tests/count_writer_pass1.rs:67:5:
a paired writer must retain the identity actually issued to Runner::run_admitted:
standalone different final newline: executed=sha256:cd6e819192262cc91c566cf7bc2e854b298a68d7bae5192d23ba5328e17f5cce claimed=sha256:c671631f9932fd763d1a4fa1fbcea560c11560574ccfc19f5904ac03f81b0cea
{
  "completed_at": 1700000000300,
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
  "implementation": "billing-reference 0.19.0",
  "outcomes": {
    "error": [],
    "failed": [],
    "passed": [
      "review.count/authored/one"
    ],
    "skipped": [],
    "unsupported": []
  },
  "policy": "complete-selection/1",
  "producer_profile": "rust-scenario-status/1",
  "spec_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "specification": "review/v1",
  "suite": {
    "digest": "sha256:c671631f9932fd763d1a4fa1fbcea560c11560574ccfc19f5904ac03f81b0cea",
    "digest_profile": "sha256-json-bytes/1",
    "version": "ess-conformance/4"
  }
}

detailed different final newline: executed=sha256:cd6e819192262cc91c566cf7bc2e854b298a68d7bae5192d23ba5328e17f5cce claimed=sha256:c671631f9932fd763d1a4fa1fbcea560c11560574ccfc19f5904ac03f81b0cea
{
  "format": "ess-conformance-run/2",
  "scenarios": [
    {
      "checks": [],
      "duration_ms": 100,
      "purpose": "An exact original suite",
      "scenario": "review.count/authored/one",
      "status": "passed"
    }
  ],
  "started_at": 1700000000000,
  "summary": {
    "completed_at": 1700000000300,
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
    "implementation": "billing-reference 0.19.0",
    "outcomes": {
      "error": [],
      "failed": [],
      "passed": [
        "review.count/authored/one"
      ],
      "skipped": [],
      "unsupported": []
    },
    "policy": "complete-selection/1",
    "producer_profile": "rust-scenario-status/1",
    "spec_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "specification": "review/v1",
    "suite": {
      "digest": "sha256:c671631f9932fd763d1a4fa1fbcea560c11560574ccfc19f5904ac03f81b0cea",
      "digest_profile": "sha256-json-bytes/1",
      "version": "ess-conformance/4"
    }
  }
}

standalone different executed assertion: executed=sha256:cd6e819192262cc91c566cf7bc2e854b298a68d7bae5192d23ba5328e17f5cce claimed=sha256:76816b29b7baf967ddd3be19267a326556c9b345ca5f46ebd749f1ffac1f7d82
{
  "completed_at": 1700000000300,
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
  "implementation": "billing-reference 0.19.0",
  "outcomes": {
    "error": [],
    "failed": [],
    "passed": [
      "review.count/authored/one"
    ],
    "skipped": [],
    "unsupported": []
  },
  "policy": "complete-selection/1",
  "producer_profile": "rust-scenario-status/1",
  "spec_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "specification": "review/v1",
  "suite": {
    "digest": "sha256:76816b29b7baf967ddd3be19267a326556c9b345ca5f46ebd749f1ffac1f7d82",
    "digest_profile": "sha256-json-bytes/1",
    "version": "ess-conformance/4"
  }
}

detailed different executed assertion: executed=sha256:cd6e819192262cc91c566cf7bc2e854b298a68d7bae5192d23ba5328e17f5cce claimed=sha256:76816b29b7baf967ddd3be19267a326556c9b345ca5f46ebd749f1ffac1f7d82
{
  "format": "ess-conformance-run/2",
  "scenarios": [
    {
      "checks": [],
      "duration_ms": 100,
      "purpose": "An exact original suite",
      "scenario": "review.count/authored/one",
      "status": "passed"
    }
  ],
  "started_at": 1700000000000,
  "summary": {
    "completed_at": 1700000000300,
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
    "implementation": "billing-reference 0.19.0",
    "outcomes": {
      "error": [],
      "failed": [],
      "passed": [
        "review.count/authored/one"
      ],
      "skipped": [],
      "unsupported": []
    },
    "policy": "complete-selection/1",
    "producer_profile": "rust-scenario-status/1",
    "spec_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "specification": "review/v1",
    "suite": {
      "digest": "sha256:76816b29b7baf967ddd3be19267a326556c9b345ca5f46ebd749f1ffac1f7d82",
      "digest_profile": "sha256-json-bytes/1",
      "version": "ess-conformance/4"
    }
  }
}

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    a_completed_run_cannot_be_rebound_to_different_admitted_original_bytes

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-conformance --test count_writer_pass1`
     Running tests/elapsed.rs (target/debug/deps/elapsed-8c80e368032de232)

running 7 tests
test a_window_opened_at_an_instant_nothing_marked_is_a_suite_defect_and_not_a_failed_implementation ... ok
test a_deadline_the_target_ran_past_fails_the_within_claim ... ok
test a_target_with_no_clock_reports_unsupported_and_the_run_fails ... ok
test a_target_that_holds_the_window_and_reports_it_passes ... ok
test an_event_published_inside_the_window_fails_the_bounded_negative_and_nothing_else ... ok
test a_target_whose_clock_never_moves_fails_rather_than_being_read_as_having_waited ... ok
test two_runs_over_one_window_produce_byte_identical_reports ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/execution.rs (target/debug/deps/execution-e989a24739ec2595)

running 12 tests
test an_eventual_assertion_asks_again_within_a_deadline_and_never_sleeps ... ok
test an_eventual_view_is_read_again_and_a_read_your_writes_view_is_not ... ok
test a_view_assertion_names_the_instance_the_scenario_created_rather_than_any_row ... ok
test a_view_answered_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test a_value_of_the_wrong_declared_type_is_caught_by_the_same_check_as_a_missing_one ... ok
test a_scenario_whose_input_no_longer_reaches_its_branch_fails_with_a_diagnostic_naming_the_defect ... ok
test an_event_missing_a_field_it_declares_is_named_leaf_by_leaf_rather_than_reported_as_absent ... ok
test every_scenario_checked_something_and_no_family_of_them_was_silently_empty ... ok
test a_read_your_writes_view_is_not_quietly_read_at_current_when_no_token_came_back ... ok
test every_scenario_the_billing_specification_obliges_passes_against_the_reference_implementation ... ok
test a_target_that_cannot_expose_an_observation_fails_the_run_rather_than_skipping_it ... ok
test two_runs_of_one_suite_against_one_target_produce_byte_identical_reports ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

     Running tests/faults.rs (target/debug/deps/faults-e1b7998fd65b338f)

running 11 tests
test a_fault_nothing_catches_is_recorded_rather_than_quietly_dropped ... ok
test the_diagnostic_of_a_caught_fault_names_the_defect_rather_than_reporting_that_something_broke ... ok
test dropping_one_binding_leaves_the_other_two_green ... ok
test a_wrong_mapping_is_invisible_to_a_target_that_cannot_show_its_invocations ... ok
test every_fault_that_could_be_a_boundary_perturbation_is_one ... ok
test the_widest_blast_radius_is_scenarios_that_could_not_be_arranged_rather_than_extra_verdicts ... ok
test each_specification_is_passed_in_full_by_the_implementation_written_from_it ... ok
test two_runs_against_one_faulty_target_produce_byte_identical_reports ... ok
test a_fault_does_not_simply_break_everything ... ok
test each_fault_fails_the_scenario_that_exists_to_catch_it ... ok
test a_faults_blast_radius_is_accounted_for ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.77s

     Running tests/halt.rs (target/debug/deps/halt-998b2db32b9f7154)

running 7 tests
test a_halt_of_an_eventual_listing_is_asked_again_while_the_projection_catches_up ... ok
test a_listing_that_ran_out_before_the_reader_stopped_it_is_not_a_halt ... ok
test retrying_does_not_rescue_a_producer_that_never_stops ... ok
test a_target_whose_producer_stops_when_the_reader_does_passes ... ok
test a_target_that_cannot_read_a_row_at_a_time_reports_unsupported_and_the_run_fails ... ok
test a_target_that_reads_the_whole_listing_fails_rather_than_being_read_as_having_stopped ... ok
test two_runs_over_one_halt_claim_produce_byte_identical_reports ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/report_reader_adversary.rs (target/debug/deps/report_reader_adversary-16de184c885788e0)

running 4 tests
test duplicate_claims_cannot_hide_behind_a_valid_last_value ... ok
test count_extremes_refuse_contradictions_without_inventing_coverage ... ok
test closed_wire_fields_preserve_their_formats_scalar_contracts ... ok
test aggregate_status_does_not_depend_on_nonpass_order_or_multiplicity ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

     Running tests/suite.rs (target/debug/deps/suite-28a700b1745795e0)

running 14 tests
test a_suite_naming_something_that_is_not_an_ess_name_is_refused_while_it_is_read ... ok
test the_scan_for_a_clock_finds_one_and_does_not_find_a_word_that_merely_ends_in_a_banned_token ... ok
test a_suite_parses_from_text_alone_without_an_ir ... ok
test a_count_and_a_position_read_back_as_what_a_runner_in_another_language_must_read ... ok
test the_steps_a_binding_and_an_invariant_need_survive_being_read_back_from_text ... ok
test every_scenario_id_the_billing_model_can_produce_reads_back ... ok
test the_scenario_ids_appear_in_the_file_in_the_order_a_sorted_key_list_would_be ... ok
test a_suite_serialised_in_one_process_resolves_in_another ... ok
test the_suite_records_the_same_model_digest_the_projections_do ... ok
test the_step_vocabulary_expresses_the_worked_example_from_section_ten ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test the_dependency_set_names_a_type_no_derived_from_would_have_mentioned ... ok
test inserting_one_outcome_re_keys_nothing_around_it ... ok
test serialising_a_suite_twice_produces_byte_identical_json ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/synthesis.rs (target/debug/deps/synthesis-9c5f74c7568d3471)

running 50 tests
test a_guard_no_candidate_can_satisfy_is_refused_with_the_number_tried ... ok
test a_binding_whose_branch_the_event_decides_refuses_the_flow_and_still_checks_the_mapping ... ok
test a_command_that_accepts_a_wrong_state_is_asserted_as_accepting_rather_than_refusing ... ok
test a_command_that_declares_no_wrong_state_answer_is_refused_by_name_beside_its_scenario ... ok
test a_filter_reading_something_no_scenario_knows_refuses_rather_than_guessing ... ok
test a_parameterised_view_is_queried_with_the_value_the_scenario_put_in_the_row ... ok
test a_state_reached_only_through_a_branch_no_input_reaches_is_refused_rather_than_arranged ... ok
test a_component_nothing_declares_is_refused_by_name ... ok
test a_binding_that_drops_its_failures_refuses_that_check_and_names_the_reason ... ok
test a_binding_mapping_names_the_source_the_document_wrote_and_not_its_same_typed_sibling ... ok
test a_binding_that_retries_forces_one_failure_and_still_requires_the_consequence ... ok
test a_read_your_writes_view_filled_by_the_command_that_ran_is_asserted_to_hold_a_row ... ok
test an_entity_nothing_creates_cannot_be_acted_on_and_says_so ... ok
test a_scenario_that_moves_an_instance_names_the_one_an_earlier_step_created ... ok
test a_declared_error_is_asserted_by_name_and_never_by_an_invented_payload ... ok
test a_binding_flow_is_proved_through_the_event_the_invoked_command_publishes ... ok
test a_move_is_observed_through_the_view_the_state_it_left_is_filtered_on ... ok
test a_declared_order_is_asserted_against_two_rows_the_scenario_arranged_itself ... ok
test a_value_object_nothing_observable_holds_keeps_a_refusal_naming_what_would_close_it ... ok
test a_view_is_asserted_in_the_block_its_own_consistency_decides ... ok
test an_undecidable_guard_refuses_and_does_not_spend_the_candidate_budget ... ok
test an_order_the_specification_cannot_put_two_rows_under_is_refused_and_not_asserted ... ok
test a_move_that_is_illegal_in_a_state_is_attempted_with_the_input_that_would_have_worked ... ok
test a_view_the_entity_has_not_reached_yet_is_asserted_to_exclude_the_instance_by_name ... ok
test an_invariant_over_a_field_no_view_publishes_refuses_rather_than_being_dropped ... ok
test a_view_that_does_not_hold_the_instance_yet_is_not_asked_about_its_invariants ... ok
test an_actor_is_named_only_where_the_specification_grants_the_command ... ok
test an_invariant_is_asserted_against_every_view_that_publishes_what_it_reads ... ok
test an_at_least_once_binding_delivers_the_event_twice_and_requires_no_count ... ok
test a_binding_that_escalates_requires_the_event_the_escalation_declares ... ok
test an_outcome_that_updates_an_instance_acts_on_one_the_scenario_created ... ok
test a_value_objects_own_invariants_are_read_at_every_field_position_a_view_holds_one ... ok
test an_event_assertion_carries_the_declared_shape_and_exactly_the_values_the_payload_determines ... ok
test a_synthesised_count_is_a_floor_the_scenario_arranged_and_never_a_ceiling ... ok
test an_illegal_move_requires_the_branch_and_the_declared_error_rather_than_merely_failing ... ok
test a_whole_system_suite_does_not_mention_a_component ... ok
test a_synthesised_suite_survives_being_written_and_read_back ... ok
test the_failure_control_is_armed_after_the_arrangement_and_before_the_command_that_triggers_it ... ok
test a_suite_for_one_component_holds_only_what_that_component_realises ... ok
test every_command_names_an_instance_an_earlier_step_of_the_same_scenario_bound ... ok
test each_example_synthesises_the_families_its_specification_declares ... ok
test every_declared_transition_has_a_scenario_that_proves_it_can_occur ... ok
test the_dependency_set_names_the_types_the_scenario_is_made_of ... ok
test every_declared_outcome_is_either_a_scenario_or_a_named_refusal_or_asserted_by_the_state_family ... ok
test the_refusal_branch_asserts_that_no_event_the_specification_declares_occurred ... ok
test an_outcome_no_input_decides_is_reached_by_injection_and_by_nothing_else ... ok
test the_input_a_scenario_sends_is_re_decided_against_the_guard_it_claims_to_reach ... ok
test every_clause_of_every_binding_is_either_a_scenario_or_a_named_refusal ... ok
test synthesising_the_same_specification_twice_produces_byte_identical_output ... ok
test canonical_expression_compatibility_fixtures ... ok

test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s

     Running tests/witness.rs (target/debug/deps/witness-8d060cd948fd3c6d)

running 28 tests
test a_refusal_names_the_predicate_the_command_and_the_path ... ok
test a_candidate_missing_a_required_field_is_refused_before_any_guard_is_read ... ok
test a_conjunction_of_two_undecidable_leaves_reports_both ... ok
test a_candidate_input_projects_one_fact_per_scalar_leaf ... ok
test a_disjunction_one_of_whose_branches_holds_is_satisfied_despite_an_undecidable_branch ... ok
test a_newtype_is_transparent_so_a_deep_path_reaches_through_it_without_a_segment ... ok
test a_path_landing_on_an_aggregate_is_unevaluable_by_construction ... ok
test a_list_a_map_and_a_union_bind_no_fact_in_the_current_projection ... ok
test a_scalar_of_the_wrong_shape_is_refused_rather_than_coerced ... ok
test a_path_into_a_list_or_a_union_names_the_aggregate_rather_than_the_missing_element ... ok
test a_refuted_guard_carries_the_leaf_and_the_value_that_refuted_it ... ok
test an_absent_optional_binds_nothing_rather_than_binding_a_default ... ok
test an_absent_optional_is_unevaluable_but_says_a_candidate_could_repair_it ... ok
test equality_over_two_texts_is_decided_even_though_ordering_them_is_not ... ok
test a_long_recursive_read_validates_beyond_the_projection_limit ... ok
test a_newtype_is_transparent_when_a_path_is_resolved_as_well_as_when_it_is_projected ... ok
test only_an_absent_value_says_another_candidate_would_help ... ok
test resolved_adapter_keeps_semantics_separate_from_collection_projection ... ok
test the_same_text_ordering_is_decidable_once_a_scale_contains_both_values ... ok
test a_candidate_carrying_a_field_no_type_declares_is_refused ... ok
test the_normative_shape_of_guard_is_decidable_for_both_signs ... ok
test unclassified_is_a_drift_alarm_and_no_enumerated_source_trips_it ... ok
test legal_collection_cardinality_is_not_currently_projected ... ok
test otherwise_and_external_are_not_guards_over_the_input ... ok
test malformed_declarations_refuse_early_and_direct_bad_reads_remain_unknown ... ok
test ordering_two_texts_is_unevaluable_because_an_ess_specification_declares_no_scale ... ok
test ordering_across_two_types_is_unevaluable_not_false ... ok
test expression_search_limits_do_not_define_type_correctness ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests ess_conformance

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 2 targets failed:
    `-p ess-cli --test count_writer_pass1`
    `-p ess-conformance --test count_writer_pass1`
exit: 101
```

4. Findings against a46bd7ff46ec8553bef4f48d4021514c8f175e82

| File:line | Verdict | Origin | Severity | Finding |
|---|---|---|---|---|
| crates/verify/ess-conformance/src/counts.rs:171 | CONFIRMED | introduced | blocker | The public count producers can bind an unchanged completed run to a different admitted suite with the same provenance and selected IDs, including a suite whose added assertion actually fails. |
| crates/verify/ess-conformance/src/go/runtime.go:2284 | CONFIRMED | introduced | blocker | Generated Go admission delegates predicates to a permissive evaluator parser, allowing unknown quantifier fields, invalid bindings and excess depth to reach target construction and complete report/2 output. |
| crates/verify/ess-conformance/src/go/runtime.go:566 | CONFIRMED | introduced | blocker | A skipped Go scenario whose EndScenario calls runtime.Goexit is counted as terminal, so report/2 can be published and the invocation can exit successfully even without a report destination. |

F1 measurement: the assertion at `crates/verify/ess-conformance/tests/count_writer_pass1.rs:69` fails, focused exit 101, after actual `Runner::run_admitted` execution. The first raw output predates formatting and locates the same assertion at line 54. Executed suite A has digest `sha256:cd6e819192262cc91c566cf7bc2e854b298a68d7bae5192d23ba5328e17f5cce`. The extra-final-newline suite has `sha256:c671631f9932fd763d1a4fa1fbcea560c11560574ccfc19f5904ac03f81b0cea`; the suite adding `expect_event review.count.Missing` has `sha256:76816b29b7baf967ddd3be19267a326556c9b345ca5f46ebd749f1ffac1f7d82`. Both new producers accept the untouched passing run of A against each other suite, and both readers accept the resulting report/run against the incorrectly claimed suite. Executing the added-assertion suite itself fails, as the positive contrast in the test asserts.

F1 reachable caller: public library clients compose `Runner::run_admitted`, `CountReport::from_run` and `CountRun::from_run`; no private state, fabricated run, altered result or dishonest custom reader is involved. Runner returns only legacy provenance at runner.rs:328–336, and the new producer takes the claimed digest from its separate admitted parameter. The current CLI at main.rs:2463–2489 passes the same admitted object to both operations and is a valid control; no CLI rebinding defect is claimed. The bound violated is exact identity of the actual issued suite, including every original byte, in the binding's Exact-suite digest section and M27–M29. This is introduced by the new paired producer API; base has no count-report producer. A correction must retain the executed original identity through a trustworthy new producer boundary while preserving the legacy persisted report shape.

F2 measurement: the assertion at `crates/edge/ess-cli/tests/count_writer_pass1.rs:127` fails, focused exit 101. Each original is rejected by Rust `AdmittedSuite::from_json`. Generated Go `Run` nevertheless exits 0 and writes both a target-construction marker and a complete skipped report for: `{"forall":{"in":"rows","as":"row","that":true,"future":true}}`; `{"forall":{"in":"rows","as":"row.part","that":true}}`; and 33 nested `not` mappings around `true`. An otherwise identical valid quantified predicate admits in both languages.

F2 reachable caller: an embedded original suite enters generated public `Run`, which invokes `admitSuite` and `admitExpectation` before `newTarget`. The test emits the actual package, replaces only its test-owned embedded input, and uses a target whose BeginScenario returns ErrUnsupported; invalid metadata therefore cannot hide behind an assertion that execution never reaches. The new admission function at runtime.go:2284 reuses `parsePredicate`; the unchanged evaluator's `parseQuantifier` ignores extra fields and accepts any nonempty binder, and its recursive parser supplies no frozen 32-level check. The existing parser's permissiveness is historical; the introduced defect is treating that evaluator as the new strict original-byte admission gate and issuing report/2 for the refused input. The binding's closed-vocabulary rule and M15/M16 require refusal before target activity. No claim about retroactively hardening raw Rust DTO parsing is made.

F3 measurement: the assertion at `crates/edge/ess-cli/tests/count_writer_pass1.rs:168` fails, focused exit 101. ExecuteCommand returns ErrUnsupported, initiating the normal skip. Deferred EndScenario then calls runtime.Goexit instead of returning. Both diagnostic invocations exit 0; the invocation with a destination also writes report/2 with one skipped terminal outcome. Controls show a returned ordinary teardown error becomes exactly one failed outcome and an ordinary skip stays exactly one skipped outcome.

F3 reachable caller: the generated public Target.EndScenario callback may terminate abnormally; the brief explicitly selects Goexit/abnormal-return controls. The new guard at runtime.go:566 treats any prior non-pass status as proof of terminal completion, even when teardown never returned. This is an introduced report/2 completion invariant failure; no change to the frozen report/1 contract is requested. The binding requires every selected scenario to terminate before a complete report is issued, including no-destination operation. Completion bookkeeping must distinguish the runner's intended skip/failure exit from an unfinished callback/teardown.

All three failures reproduced in the complete package run. The table and final machine-readable block carry identical findings. There are no additional judgement-only findings.

5. Coverage and limits

- The complete accepted binding at coordinator commit 06bb07182f51524ab6443c9080ba4b88687f9c7c, the active story and original unit brief, repository instructions, full base-to-subject diff, changed public documentation, added tests, retained legacy Go source and touched callers were read before the first authored test executed.
- Actual public Rust production and paired readers were exercised without modifying the completed report; direct CLI callers were inspected and identified separately from the public library counterexample.
- Actual generated Go sources ran in all new Go cases. Each focused generated runtime hashes to `ec6bde7ac44b4871c2466d714bbd46303998e026d82d24bb3f109a3bb56a41b2`, identical to frozen production. The retained legacy runtime hashes to `434a77d4044ebb7775e68b97e36c57cc26ac3c066ae5b678fe948cfbca10a124`, identical to the base Git object.
- Opaque newline, quote and combining-Unicode IDs, colliding Go display names, CRLF suite identity, skip-only zero actual failures, returned teardown error classification and strict inconclusive operation without a destination passed their new controls.
- All 401 inherited package cases passed, including existing lexical/full-u64 timestamp, detailed closure/order/duration, duplicate-key, raw DTO versus admitted capability, invalid configuration, omitted subtest, negative Go clock and retained-old-runtime controls. This does not turn each inherited assertion into a new adversarial case.
- The review is bounded to count-stage suite/1–4, report/2, run/2 and frozen legacy controls. It does not cover suite/5 inventory, all 75 matrix rows, browser/impact, deployment regeneration or default movement. Root's separate frozen-producer → AEP compatibility exercise is not replaced or claimed by this report. No AEP path was modified.

6. Writes and closure

All authored files are the three new test/test-fixture paths listed immediately after the header. No production source, existing test, manifest, lockfile, document, planning artifact, Git index/ref/object, managed worktree registry or lifecycle record was intentionally changed. All 20 source-manifest hashes were verified after the complete suite; `final-frozen-source-check.txt` contains 20 OK lines. The subject remains a46bd7ff46ec8553bef4f48d4021514c8f175e82. The saved focused report and all new test hashes also verified unchanged after the full suite.

All authored scratch, process logs, generated Go modules, marker/report outputs and caches remain under:
`/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-1`.
The exact TMPDIR is that directory; GOCACHE and GOMODCACHE are its `go-cache` and `go-mod-cache` children. Local compilation used only the assigned worktree's existing `target`. Existing package tests also regenerate their established in-tree fixture/export/log locations under `target/review-boundaries-8`; those are inherited test side effects, and the coordinator separately retained its frozen producer exports. No shared export was intentionally rewritten by a new test.

No authored path outside the worktree was written. Cargo uses the pre-existing shared infrastructure metadata file `/home/timo/.cargo/.global-cache` (1,007,616 bytes; an observed mtime during this review was 2026-09-06 11:42:19.375950944 +0200). Because Cargo and coordinator activity share that metadata, attribution of its refresh to one process is not established; it is reported conservatively as the one external infrastructure path. No external Go cache or module cache was selected, and no compiler/cache daemon was started.

All four focused command sessions and both package command sessions exited; final process inspection found no owned Cargo/test/Go child process. No service was launched. The final disk check reported 29 GiB available, above the 8 GiB floor. This is the first bounded attack only. This complete report is immutable on return, and all test/scratch writes are relinquished to the coordinator for routing and correction.

```findings
- file: crates/verify/ess-conformance/src/counts.rs
  line: 171
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: The public count producers can bind an unchanged completed run to a different admitted suite with the same provenance and selected IDs, including a suite whose added assertion actually fails.
- file: crates/verify/ess-conformance/src/go/runtime.go
  line: 2284
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Generated Go admission delegates predicates to a permissive evaluator parser, allowing unknown quantifier fields, invalid bindings and excess depth to reach target construction and complete report/2 output.
- file: crates/verify/ess-conformance/src/go/runtime.go
  line: 566
  category: boundary
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: A skipped Go scenario whose EndScenario calls runtime.Goexit is counted as terminal, so report/2 can be published and the invocation can exit successfully even without a report destination.
```
