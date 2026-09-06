---
format: aep.planning-md/1
id: review-result:ess-count-writer-adversary-pass-2
kind: review-result
status: active
title: ESS count writer adversary pass 2
relations:
- reviews: story:a-skipped-scenario-is-not-a-failed-one
revision: 1
---
unit: ess-conformance-count-writer pass 2, frozen 1be4dbd999b20e44ba0f04e87d1c4ddf81c1b4ab
verdict: CONFIRMED
cases: executed 405→408, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 shared infrastructure metadata path; no authored external file
needs-coordinator: record immutable pass and route the remaining predicate-admission finding
git --no-pager diff --stat
```text
```
Tracked diff is empty. New additive tests are untracked; the exact supplemental listings are:
```text
git ls-files --others --exclude-standard
crates/edge/ess-cli/tests/count_writer_pass2.rs
crates/edge/ess-cli/tests/fixtures/count-writer-pass2/target_test.go
crates/verify/ess-conformance/tests/count_writer_pass2.rs
```
For each listed path: git --no-pager diff --no-index --stat /dev/null PATH (exit 1 denotes an added file):
```text
 .../edge/ess-cli/tests/count_writer_pass2.rs       | 171 +++++++++++++++++++++
 1 file changed, 171 insertions(+)
exit: 1
 .../fixtures/count-writer-pass2/target_test.go     | 41 ++++++++++++++++++++++
 1 file changed, 41 insertions(+)
exit: 1
 .../ess-conformance/tests/count_writer_pass2.rs    | 58 ++++++++++++++++++++++
 1 file changed, 58 insertions(+)
exit: 1
```

2. New cases and first focused executions, captured before package-suite execution

The first authored test is `crates/edge/ess-cli/tests/count_writer_pass2.rs::generated_go_admits_only_typed_predicate_paths_and_operator_envelopes`. It selected one case and failed behaviorally, exit 101, before any other execution. Rust direct original admission refuses all three invalid predicates: mapping path `ready..done`, the unknown constraint operator `future` beside `eq`, and expression `ready..done == true`. The actual generated Go package constructs the target and publishes report/2 with exit 0 for all three. The otherwise identical valid `ready: {eq: true}` original is a positive skipped-report control.

The second case, `generated_go_abnormal_unsupported_error_formatting_cannot_complete`, selected one and passed. Its error implements `Is(ErrUnsupported)` and exits from `Error()` before SkipNow. Go testing detects the unfinished skip and exits 1 without report output, with and without a destination. The ordinary returned unsupported error remains a complete skipped report. This rejects the coordinator's offered hypothesis; it is not a finding.

The third case, `crates/verify/ess-conformance/tests/count_writer_pass2.rs::cloned_execution_binding_survives_mutation_of_extracted_legacy_diagnostics`, initially failed to compile because this test assigned scenario Status to aggregate ConformanceStatus. That setup attempt selected no cases and is preserved below, separately from semantic results. Correcting only that new test's enum type selected one case, which passed: cloning ExecutedRun retains exact CRLF/Unicode original identity despite mutation of separately extracted legacy diagnostics; decoded-equal escaped originals have a different digest and cannot bind either count producer or paired reader.

The final formatted assertion for the first failure is CLI test line 129; first raw output precedes formatting and identifies the same assertion at line 76. New authored totals: 3 Rust cases, 1 red and 2 green; 7 real Go invocations inside the first two focused cases, five exit 0 and two expected exit 1. No inherited test was changed. No preemptive baseline suite ran. Raw records follow in actual execution order.

```text
cargo test -p ess-cli --locked --test count_writer_pass2 generated_go_admits_only_typed_predicate_paths_and_operator_envelopes -- --exact --nocapture
   Compiling ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.27s
     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-a7133ca202cf1f1f)

running 1 test
cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308/valid.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308/valid.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(0)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
    runtime.go:1386: the target does not support this scenario: the target does not expose this
--- PASS: TestReviewSecond (0.00s)
    --- SKIP: TestReviewSecond/review.count/authored/second (0.00s)
PASS
ok  	countreviewsecond/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308/invalid-fact-path.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308/invalid-fact-path.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(0)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
    runtime.go:1386: the target does not support this scenario: the target does not expose this
--- PASS: TestReviewSecond (0.00s)
    --- SKIP: TestReviewSecond/review.count/authored/second (0.00s)
PASS
ok  	countreviewsecond/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308/unknown-constraint-operator.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308/unknown-constraint-operator.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(0)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
    runtime.go:1386: the target does not support this scenario: the target does not expose this
--- PASS: TestReviewSecond (0.00s)
    --- SKIP: TestReviewSecond/review.count/authored/second (0.00s)
PASS
ok  	countreviewsecond/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308/invalid-expression-path.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-821308/invalid-expression-path.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(0)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
    runtime.go:1386: the target does not support this scenario: the target does not expose this
--- PASS: TestReviewSecond (0.00s)
    --- SKIP: TestReviewSecond/review.count/authored/second (0.00s)
PASS
ok  	countreviewsecond/essconform	0.002s

stderr:


thread 'generated_go_admits_only_typed_predicate_paths_and_operator_envelopes' (821309) panicked at crates/edge/ess-cli/tests/count_writer_pass2.rs:76:5:
invalid original predicate envelopes must refuse before target construction:
invalid-fact-path: exit=Some(0), target=true, report=true
unknown-constraint-operator: exit=Some(0), target=true, report=true
invalid-expression-path: exit=Some(0), target=true, report=true
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test generated_go_admits_only_typed_predicate_paths_and_operator_envelopes ... FAILED

failures:

failures:
    generated_go_admits_only_typed_predicate_paths_and_operator_envelopes

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.10s

error: test failed, to rerun pass `-p ess-cli --test count_writer_pass2`
exit: 101
```

```text
cargo test -p ess-cli --locked --test count_writer_pass2 generated_go_abnormal_unsupported_error_formatting_cannot_complete -- --exact --nocapture
   Compiling ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.30s
     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-a7133ca202cf1f1f)

running 1 test
cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231/ordinary-skip.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231/ordinary-skip.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(0)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
    runtime.go:1386: the target does not support this scenario: the target does not expose this
--- PASS: TestReviewSecond (0.00s)
    --- SKIP: TestReviewSecond/review.count/authored/second (0.00s)
PASS
ok  	countreviewsecond/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT -u ESS_REPORT_OUT ESS_REPORT_FORMAT="2" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231/format-goexit-false.marker" REVIEW_MODE="begin-error-format-goexit" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(1)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
--- FAIL: TestReviewSecond (0.00s)
    --- FAIL: TestReviewSecond/review.count/authored/second (0.00s)
panic: test executed panic(nil) or runtime.Goexit

goroutine 7 [running]:
testing.tRunner.func1.2({0x5a9120, 0x750ea0})
	/usr/lib/go/src/testing/testing.go:1974 +0x232
testing.tRunner.func1()
	/usr/lib/go/src/testing/testing.go:1977 +0x349
runtime.Goexit()
	/usr/lib/go/src/runtime/panic.go:694 +0x5e
countreviewsecond/essconform.exitError.Error(...)
	/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231/essconform/review_test.go:14
fmt.(*pp).handleMethods(0x1fc989da2410, 0x4?)
	/usr/lib/go/src/fmt/print.go:668 +0x394
fmt.(*pp).printArg(0x1fc989da2410, {0x5b2700, 0x77c440}, 0x76)
	/usr/lib/go/src/fmt/print.go:757 +0x42e
fmt.(*pp).doPrintf(0x1fc989da2410, {0x5e7fe5, 0x2d}, {0x1fc989dd1d28, 0x1, 0x1})
	/usr/lib/go/src/fmt/print.go:1075 +0x3ec
fmt.Sprintf({0x5e7fe5, 0x2d}, {0x1fc989d98d28, 0x1, 0x1})
	/usr/lib/go/src/fmt/print.go:239 +0x53
testing.(*common).Skipf(0x1fc989de6488, {0x5e7fe5?, 0x5eef68?}, {0x1fc989d98d28?, 0x499b5a?, 0x1fc989da2410?})
	/usr/lib/go/src/testing/testing.go:1241 +0x3f
countreviewsecond/essconform.(*run).skip(...)
	/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231/essconform/runtime.go:1386
countreviewsecond/essconform.(*run).execute(0x1fc989dd1e50, {0x1fc989cee620, 0x1c}, {{0x1fc989cf4a80, 0x2c}, {0x1fc989cc8200, 0x1, 0x1}})
	/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231/essconform/runtime.go:718 +0x171
countreviewsecond/essconform.Run.func1(0x1fc989de6488)
	/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231/essconform/runtime.go:568 +0x2cd
testing.tRunner(0x1fc989de6488, 0x1fc989dc6380)
	/usr/lib/go/src/testing/testing.go:2036 +0xea
created by testing.(*T).Run in goroutine 6
	/usr/lib/go/src/testing/testing.go:2101 +0x4c5
FAIL	countreviewsecond/essconform	0.004s
FAIL

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231/format-goexit-true.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231/format-goexit-true.marker" REVIEW_MODE="begin-error-format-goexit" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(1)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
--- FAIL: TestReviewSecond (0.00s)
    --- FAIL: TestReviewSecond/review.count/authored/second (0.00s)
panic: test executed panic(nil) or runtime.Goexit

goroutine 19 [running]:
testing.tRunner.func1.2({0x5a9120, 0x750ea0})
	/usr/lib/go/src/testing/testing.go:1974 +0x232
testing.tRunner.func1()
	/usr/lib/go/src/testing/testing.go:1977 +0x349
runtime.Goexit()
	/usr/lib/go/src/runtime/panic.go:694 +0x5e
countreviewsecond/essconform.exitError.Error(...)
	/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231/essconform/review_test.go:14
fmt.(*pp).handleMethods(0x3a1877f36410, 0x4?)
	/usr/lib/go/src/fmt/print.go:668 +0x394
fmt.(*pp).printArg(0x3a1877f36410, {0x5b2700, 0x77c440}, 0x76)
	/usr/lib/go/src/fmt/print.go:757 +0x42e
fmt.(*pp).doPrintf(0x3a1877f36410, {0x5e7fe5, 0x2d}, {0x3a1877f75d28, 0x1, 0x1})
	/usr/lib/go/src/fmt/print.go:1075 +0x3ec
fmt.Sprintf({0x5e7fe5, 0x2d}, {0x3a1877f20d28, 0x1, 0x1})
	/usr/lib/go/src/fmt/print.go:239 +0x53
testing.(*common).Skipf(0x3a1877f92488, {0x5e7fe5?, 0x5eef68?}, {0x3a1877f20d28?, 0x499b5a?, 0x3a1877f36410?})
	/usr/lib/go/src/testing/testing.go:1241 +0x3f
countreviewsecond/essconform.(*run).skip(...)
	/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231/essconform/runtime.go:1386
countreviewsecond/essconform.(*run).execute(0x3a1877f75e50, {0x3a1877f5a360, 0x1c}, {{0x3a1877f64630, 0x2c}, {0x3a1877f80100, 0x1, 0x1}})
	/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231/essconform/runtime.go:718 +0x171
countreviewsecond/essconform.Run.func1(0x3a1877f92488)
	/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-error-formatting-838231/essconform/runtime.go:568 +0x2cd
testing.tRunner(0x3a1877f92488, 0x3a1877f6a380)
	/usr/lib/go/src/testing/testing.go:2036 +0xea
created by testing.(*T).Run in goroutine 18
	/usr/lib/go/src/testing/testing.go:2101 +0x4c5
FAIL	countreviewsecond/essconform	0.004s
FAIL

stderr:

test generated_go_abnormal_unsupported_error_formatting_cannot_complete ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.46s

exit: 0
```

```text
cargo test -p ess-conformance --locked --test count_writer_pass2 cloned_execution_binding_survives_mutation_of_extracted_legacy_diagnostics -- --exact --nocapture
   Compiling ess-conformance v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/crates/verify/ess-conformance)
error[E0308]: mismatched types
  --> crates/verify/ess-conformance/tests/count_writer_pass2.rs:23:21
   |
23 |     legacy.status = Status::Failed;
   |     -------------   ^^^^^^^^^^^^^^ expected `ConformanceStatus`, found `Status`
   |     |
   |     expected due to the type of this binding

For more information about this error, try `rustc --explain E0308`.
error: could not compile `ess-conformance` (test "count_writer_pass2") due to 1 previous error
exit: 101
```

```text
cargo test -p ess-conformance --locked --test count_writer_pass2 cloned_execution_binding_survives_mutation_of_extracted_legacy_diagnostics -- --exact --nocapture
   Compiling ess-conformance v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/crates/verify/ess-conformance)
    Finished `test` profile [unoptimized] target(s) in 0.26s
     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-934fcf0bb27c1cc4)

running 1 test
original digest: sha256:272282e3693f614c1f5b8c3655f63cca06c4fe19230fdf299ba2c35dbc1a10ad; escaped digest: sha256:d0138164a373c3082ac3c159aef72d8df71774dfe9b1e74dfc799f99ce1e365b; immutable retained run remains passed
test cloned_execution_binding_survives_mutation_of_extracted_legacy_diagnostics ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

exit: 0
```

3. Package execution, after all new focused cases and focused-evidence.md existed

Baseline is the correction's final actual command, `cargo test -p ess-conformance -p ess-cli --locked --no-fail-fast`: 405 passed, 0 failed/ignored across 33 runner summaries. This is inherited baseline evidence, not a preemptive run. The initially requested command below stopped at the new CLI failure: 71 executed, 70 passed, 1 failed, 0 ignored across 9 summaries, exit 101. The subsequent complete `--no-fail-fast` command executed 408, with 407 passed, 1 failed, 0 ignored across 35 summaries, exit 101. All 405 inherited names remain selected and passing; exactly the three added names account for the increase. measured-counts.json retains the counts and added/removed/nonpassing inherited name comparison.

All Cargo test commands ran from `/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer` with the following explicit environment. Rust compiler and Go tests used the assigned local build/scratch paths; no sccache server or shared target was used.

```text
unset RUSTC_WRAPPER SCCACHE_SERVER_UDS CARGO_TARGET_DIR
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true
export TMPDIR="$PWD/target/review-boundaries-8/adversary-pass-2/tmp" GOCACHE="$PWD/target/review-boundaries-8/adversary-pass-2/go-cache" GOMODCACHE="$PWD/target/review-boundaries-8/adversary-pass-2/go-mod-cache"
```

The exact argv, combined Cargo stdout/stderr and actual exit follow for both package executions. Focused Go child stdout/stderr are separately delimited in section 2. Formatter-only changes occurred before the package commands and no source changed after them.

```text
cargo test -p ess-conformance -p ess-cli --locked
   Compiling ess-cli v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/crates/edge/ess-cli)
   Compiling ess-conformance v0.19.0 (/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/crates/verify/ess-conformance)
    Finished `test` profile [unoptimized] target(s) in 0.35s
     Running unittests src/main.rs (target/debug/deps/ess-d3edf4c0ecde1ae7)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/authored_scenarios.rs (target/debug/deps/authored_scenarios-665b0ff538fabc92)

running 25 tests
test author_empty ... ok
test author_nonmatching_only ... ok
test author_nested_only ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test ir_nested_only ... ok
test ir_empty ... ok
test ir_nonmatching_only ... ok
test go_nested_only ... ok
test go_empty ... ok
test go_nonmatching_only ... ok
test web_nested_only ... ok
test web_nonmatching_only ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test run_nonmatching_only ... ok
test web_empty ... ok
test run_empty ... ok
test run_nested_only ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.84s

     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-ddf0cc8492bb33b1)

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

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s

     Running tests/authored_site.rs (target/debug/deps/authored_site-21399ca4863fa792)

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

     Running tests/command_surface.rs (target/debug/deps/command_surface-9ab8f7e62cc8cbe3)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s

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

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s

     Running tests/count_writer_pass1.rs (target/debug/deps/count_writer_pass1-d029e2dedf54b30d)

running 3 tests
test generated_go_skip_counts_preserve_opaque_ids_and_strictness_without_a_destination ... ok
test generated_go_abnormal_teardown_cannot_publish_a_completed_skip ... ok
test generated_go_rejects_closed_predicate_metadata_before_any_target ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.09s

     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-a7133ca202cf1f1f)

running 2 tests
test generated_go_abnormal_unsupported_error_formatting_cannot_complete ... ok
test generated_go_admits_only_typed_predicate_paths_and_operator_envelopes ... FAILED

failures:

---- generated_go_admits_only_typed_predicate_paths_and_operator_envelopes stdout ----
cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313/valid.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313/valid.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(0)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
    runtime.go:1386: the target does not support this scenario: the target does not expose this
--- PASS: TestReviewSecond (0.00s)
    --- SKIP: TestReviewSecond/review.count/authored/second (0.00s)
PASS
ok  	countreviewsecond/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313/invalid-fact-path.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313/invalid-fact-path.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(0)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
    runtime.go:1386: the target does not support this scenario: the target does not expose this
--- PASS: TestReviewSecond (0.00s)
    --- SKIP: TestReviewSecond/review.count/authored/second (0.00s)
PASS
ok  	countreviewsecond/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313/unknown-constraint-operator.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313/unknown-constraint-operator.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(0)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
    runtime.go:1386: the target does not support this scenario: the target does not expose this
--- PASS: TestReviewSecond (0.00s)
    --- SKIP: TestReviewSecond/review.count/authored/second (0.00s)
PASS
ok  	countreviewsecond/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313/invalid-expression-path.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-860313/invalid-expression-path.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(0)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
    runtime.go:1386: the target does not support this scenario: the target does not expose this
--- PASS: TestReviewSecond (0.00s)
    --- SKIP: TestReviewSecond/review.count/authored/second (0.00s)
PASS
ok  	countreviewsecond/essconform	0.002s

stderr:


thread 'generated_go_admits_only_typed_predicate_paths_and_operator_envelopes' (860315) panicked at crates/edge/ess-cli/tests/count_writer_pass2.rs:129:5:
invalid original predicate envelopes must refuse before target construction:
invalid-fact-path: exit=Some(0), target=true, report=true
unknown-constraint-operator: exit=Some(0), target=true, report=true
invalid-expression-path: exit=Some(0), target=true, report=true
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    generated_go_admits_only_typed_predicate_paths_and_operator_envelopes

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s

error: test failed, to rerun pass `-p ess-cli --test count_writer_pass2`
exit: 101
```

```text
cargo test -p ess-conformance -p ess-cli --locked --no-fail-fast
    Finished `test` profile [unoptimized] target(s) in 0.10s
     Running unittests src/main.rs (target/debug/deps/ess-d3edf4c0ecde1ae7)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/authored_scenarios.rs (target/debug/deps/authored_scenarios-665b0ff538fabc92)

running 25 tests
test author_empty ... ok
test author_nested_only ... ok
test author_nonmatching_only ... ok
test synthesize_help_describes_explicit_shallow_selection_without_a_default ... ok
test empty_selection_refusal_is_identical_through_flat_and_area_spellings ... ok
test web_empty ... ok
test web_nested_only ... ok
test ir_empty ... ok
test go_nonmatching_only ... ok
test ir_nested_only ... ok
test go_nested_only ... ok
test go_empty ... ok
test run_nested_only ... ok
test run_nonmatching_only ... ok
test ir_nonmatching_only ... ok
test run_empty ... ok
test web_nonmatching_only ... ok
test a_yaml_named_subdirectory_alone_retains_its_read_refusal ... ok
test a_matching_directory_symlink_beside_a_valid_scenario_retains_its_read_refusal ... ok
test a_matching_directory_beside_a_valid_scenario_retains_its_read_refusal ... ok
test supplied_suite_bypasses_empty_and_nonexistent_scenario_paths ... ok
test missing_paths_and_malformed_matching_sources_remain_failures ... ok
test omitted_scenarios_preserve_intentional_generated_and_authored_selections ... ok
test shallow_yaml_and_yml_selection_is_independent_of_creation_order ... ok
test explicit_files_and_shallow_directories_select_the_same_scenario ... ok

test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

     Running tests/authored_scenarios_adversary.rs (target/debug/deps/authored_scenarios_adversary-ddf0cc8492bb33b1)

running 9 tests
test non_utf8_matching_content_is_refused_before_any_output_write ... ok
test empty_selection_cannot_follow_or_replace_an_output_symlink ... ok
test selected_directory_symlink_reports_the_requested_path_and_stays_shallow ... ok
test matching_broken_links_are_read_errors_even_beside_a_valid_source ... ok
test lexical_selection_order_decides_the_first_read_error ... ok
test committed_suite_bypasses_poisoned_scenarios_and_missing_model_for_both_runners ... ok
test omitted_scenarios_ignore_a_poisoned_working_directory_default ... ok
test empty_selection_precedes_conflicting_output_for_every_alias_format_and_runner ... ok
test valid_file_links_keep_direct_extension_independence_and_ignore_nonmatching_links ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s

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

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-2d5107ca256f28e2)

running 4 tests
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
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
test generated_go_abnormal_teardown_cannot_publish_a_completed_skip ... ok
test generated_go_rejects_closed_predicate_metadata_before_any_target ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.11s

     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-a7133ca202cf1f1f)

running 2 tests
test generated_go_abnormal_unsupported_error_formatting_cannot_complete ... ok
test generated_go_admits_only_typed_predicate_paths_and_operator_envelopes ... FAILED

failures:

---- generated_go_admits_only_typed_predicate_paths_and_operator_envelopes stdout ----
cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947/valid.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947/valid.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(0)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
    runtime.go:1386: the target does not support this scenario: the target does not expose this
--- PASS: TestReviewSecond (0.00s)
    --- SKIP: TestReviewSecond/review.count/authored/second (0.00s)
PASS
ok  	countreviewsecond/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947/invalid-fact-path.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947/invalid-fact-path.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(0)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
    runtime.go:1386: the target does not support this scenario: the target does not expose this
--- PASS: TestReviewSecond (0.00s)
    --- SKIP: TestReviewSecond/review.count/authored/second (0.00s)
PASS
ok  	countreviewsecond/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947/unknown-constraint-operator.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947/unknown-constraint-operator.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(0)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
    runtime.go:1386: the target does not support this scenario: the target does not expose this
--- PASS: TestReviewSecond (0.00s)
    --- SKIP: TestReviewSecond/review.count/authored/second (0.00s)
PASS
ok  	countreviewsecond/essconform	0.002s

stderr:

cwd: /home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947
command: cd "/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947" && env -u ESS_CONFORMANCE_ALLOW_INCOMPLETE -u ESS_CONFORMANCE_STRICT ESS_REPORT_FORMAT="2" ESS_REPORT_OUT="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947/invalid-expression-path.report.json" REVIEW_MARKER="/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2/go-leaf-admission-869947/invalid-expression-path.marker" REVIEW_MODE="begin-skip" "go" "test" "-count=1" "-v" "./..." "-run" "^TestReviewSecond$"
exit: Some(0)
stdout:
=== RUN   TestReviewSecond
    review_test.go:35: review v1, 1 scenario(s), spec digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
=== RUN   TestReviewSecond/review.count/authored/second
    runtime.go:1386: the target does not support this scenario: the target does not expose this
--- PASS: TestReviewSecond (0.00s)
    --- SKIP: TestReviewSecond/review.count/authored/second (0.00s)
PASS
ok  	countreviewsecond/essconform	0.002s

stderr:


thread 'generated_go_admits_only_typed_predicate_paths_and_operator_envelopes' (869951) panicked at crates/edge/ess-cli/tests/count_writer_pass2.rs:129:5:
invalid original predicate envelopes must refuse before target construction:
invalid-fact-path: exit=Some(0), target=true, report=true
unknown-constraint-operator: exit=Some(0), target=true, report=true
invalid-expression-path: exit=Some(0), target=true, report=true
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    generated_go_admits_only_typed_predicate_paths_and_operator_envelopes

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s

error: test failed, to rerun pass `-p ess-cli --test count_writer_pass2`
     Running tests/feasibility_adversary.rs (target/debug/deps/feasibility_adversary-d32c13ce3b168b5c)

running 4 tests
test web_dependency_module_collision_is_refused_before_cli_output ... ok
test http_codec_local_collision_is_refused_before_cli_output ... ok
test a_binding_function_capture_is_refused_before_rust_cli_writes ... ok
test a_binding_function_capture_is_refused_before_web_cli_writes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-49c13e90a6599768)

running 12 tests
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test count_report_skip_only_is_inconclusive_without_actual_failures ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test count_retained_runtime_preserves_legacy_behavior_and_does_not_gain_version_checks ... ok
test count_go_empty_selection_is_inconclusive_and_clock_conversion_is_checked ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test count_go_actual_producers_keep_skip_error_and_teardown_categories ... ok
test count_go_refusals_precede_targets_and_incomplete_runs_never_publish ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.66s

     Running tests/model_types.rs (target/debug/deps/model_types-81eb3a002a8b3064)

running 2 tests
test root_selection_and_output_refusals_preserve_existing_files ... ok
test each_target_retains_the_same_model_selection_and_distinct_input_provenance ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/normalization.rs (target/debug/deps/normalization-9776ecd4b47f8e04)

running 13 tests
test incompatible_later_destination_preserves_the_entire_existing_output ... ok
test unsupported_go_pattern_has_no_successful_partial_artifact ... ok
test generated_paths_cannot_replace_canonical_source_inputs_or_follow_links ... ok
test output_symlinks_and_hardlinks_are_refused_without_mutation ... ok
test qualified_go_base64_pattern_is_published_without_decoding_the_value ... ok
test stale_bundle_missing_source_and_unknown_dispatch_refuse ... ok
test checked_recipe_and_run_are_deterministic_with_json_only_stdout ... ok
test output_cannot_replace_any_declared_input ... ok
test explicit_binary64_inputs_run_through_the_cli_without_weakening_version_one ... ok
test generation_checks_refuse_before_creating_or_changing_destinations ... ok
test failed_check_or_execution_preserves_output_and_does_not_emit_a_result ... ok
test generated_libraries_match_the_api_and_drift_check_never_repairs_files ... ok
test model_sources_are_compiled_pinned_and_protected_by_the_cli ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s

     Running tests/openapi_accounting.rs (target/debug/deps/openapi_accounting-3ce279f6417aad27)

running 6 tests
test complete_projection_preserves_supported_yaml_through_both_spellings ... ok
test both_import_spellings_write_the_accounted_envelope_for_every_presentation ... ok
test a_refused_import_keeps_existing_output_and_creates_no_new_file ... ok
test legacy_projection_refuses_before_destination_mutation ... ok
test unresolved_projection_reports_the_exact_reference_and_preserves_destinations ... ok
test partial_projection_reports_the_durable_gap_and_preserves_destinations ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/openapi_adversary_pass1.rs (target/debug/deps/openapi_adversary_pass1-d92f800d3110935f)

running 2 tests
test cli_rejects_duplicate_keys_and_tampered_accounting_before_output ... ok
test cli_rejects_unknown_unit_fields_before_any_projection_output ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/openapi_adversary_pass2.rs (target/debug/deps/openapi_adversary_pass2-38d06ca5d76d5677)

running 1 test
test same_path_projection_refusal_preserves_the_unadmitted_input ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/output_containment.rs (target/debug/deps/output_containment-e1e5a2958a09cb3e)

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
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-a5eb934fe97baa0e)

running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

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

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/target_failure.rs (target/debug/deps/target_failure-243770058fcedcec)

running 1 test
test fatal_synthesis_preserves_destinations_and_has_a_typed_envelope ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s

     Running unittests src/lib.rs (target/debug/deps/ess_conformance-842414d0dd52814a)

running 69 tests
test decision::tests::a_decision_reads_its_two_other_cases_as_neither_satisfied_nor_the_other ... ok
test counts::tests::exact_unsigned_scalar_vectors_do_not_use_binary64 ... ok
test counts::tests::payload_number_and_utf8_canonical_profile_is_frozen_separately_from_scalars ... ok
test decision::tests::a_refusal_renders_the_predicate_the_command_and_every_reason ... ok
test decision::tests::exactly_one_reason_says_another_candidate_would_help ... ok
test evidence::tests::a_standalone_report_carries_every_field_an_adapter_needs ... ok
test evidence::tests::report_readers_refuse_more_nonpasses_than_executed_scenarios ... ok
test faulty::tests::a_fault_is_injected_into_the_system_that_declares_what_it_breaks ... ok
test evidence::tests::the_closed_report_round_trips_with_identical_canonical_bytes ... ok
test evidence::tests::report_readers_do_not_guess_a_producer_from_status_vocabulary ... ok
test faulty::tests::no_two_faults_claim_the_same_scenario ... ok
test faulty::tests::every_fault_says_what_it_is_and_where_it_goes ... ok
test evidence::tests::unknown_report_fields_are_refused ... ok
test faulty::tests::only_the_two_faults_the_boundary_cannot_express_are_injected_in_the_implementation ... ok
test input::tests::every_primitive_projects_to_the_one_fact_value_that_can_hold_it ... ok
test input::tests::shape_errors_render_one_per_line_and_name_the_input_root_by_name ... ok
test evidence::tests::report_readers_refuse_nonpass_count_and_list_disagreement ... ok
test input::tests::a_primitive_refuses_a_node_of_the_wrong_shape_rather_than_coercing_it ... ok
test report::tests::a_diagnostic_answers_all_five_of_the_questions_a_failure_has_to_answer ... ok
test report::tests::a_quoted_input_reads_as_the_call_that_was_made ... ok
test report::tests::a_scenario_status_is_the_strongest_of_its_checks_and_a_contradiction_outranks_everything ... ok
test go::tests::every_go_file_is_in_the_package_the_readme_names ... ok
test report::tests::an_unsupported_scenario_makes_the_run_fail_rather_than_look_like_a_pass ... ok
test report::tests::every_check_code_has_a_distinct_name_and_a_rule_sentence ... ok
test runner::tests::a_count_with_neither_bound_is_a_suite_defect_and_not_a_satisfied_assertion ... ok
test go::tests::the_runner_is_a_constant_and_only_the_suite_moves ... ok
test runner::tests::a_count_is_the_half_of_an_ordering_claim_that_says_the_rows_were_there ... ok
test runner::tests::a_nested_row_binds_the_paths_a_predicate_spells ... ok
test runner::tests::a_declared_order_is_checked_on_adjacent_rows_and_the_next_key_breaks_a_tie ... ok
test runner::tests::a_position_in_a_view_that_declares_no_order_is_a_suite_defect ... ok
test evidence::tests::report_readers_preserve_go_producer_bytes_and_historical_nonpass_counts ... ok
test runner::tests::a_ranking_key_a_row_does_not_publish_is_undecidable_rather_than_out_of_order ... ok
test runner::tests::a_position_names_both_ends_and_a_row_that_is_not_there_is_not_a_match ... ok
test runner::tests::a_predicate_a_row_cannot_answer_is_reported_rather_than_retried ... ok
test runner::tests::a_view_that_holds_nothing_does_not_satisfy_an_invariant_by_being_empty ... ok
test runner::tests::an_order_over_fewer_than_two_rows_holds_and_does_not_double_as_a_non_emptiness_claim ... ok
test runner::tests::an_empty_field_set_means_a_row_exists_and_not_that_anything_will_do ... ok
test runner::tests::the_runners_clock_advances_on_every_read_so_a_deadline_can_bound_anything ... ok
test runner::tests::ids_come_from_the_suite_and_from_nothing_ambient ... ok
test scenario::tests::a_declared_leaf_admits_what_its_type_admits_and_absence_only_where_the_type_permits_it ... ok
test scenario::tests::a_purpose_is_one_line_and_says_something ... ok
test scenario::tests::a_payload_shape_round_trips_through_the_form_a_suite_is_stored_in ... ok
test scenario::tests::a_scenario_id_names_the_construct_it_exercises_rather_than_its_position ... ok
test scenario::tests::a_semantic_reference_renders_the_way_the_design_writes_one ... ok
test scenario::tests::a_scenario_id_that_names_no_construct_is_refused ... ok
test scenario::tests::a_suite_format_from_a_later_build_is_refused_rather_than_guessed ... ok
test scenario::tests::a_suite_refuses_a_second_scenario_under_one_id ... ok
test scenario::tests::a_transition_ref_refuses_a_name_no_lifecycle_can_declare ... ok
test scenario::tests::an_invariant_scenario_is_keyed_by_the_entity_and_the_branch_and_never_by_a_position ... ok
test scenario::tests::every_binding_aspect_is_in_the_list_that_is_walked_to_produce_them ... ok
test scenario::tests::the_ids_of_a_suite_sort_the_way_a_reader_sorts_the_file ... ok
test scenario::tests::two_scenarios_about_the_same_thing_in_the_same_way_are_one_id ... ok
test evidence::tests::report_readers_refuse_unknown_report_formats ... ok
test scenario::tests::every_scenario_id_reads_back_from_the_form_a_report_prints ... ok
test synthesize::tests::a_refusal_names_the_construct_the_code_and_the_repair ... ok
test synthesize::tests::every_refusal_carries_a_distinct_code_in_one_family ... ok
test web::tests::no_comparison_sits_in_a_text_node_mustache ... ok
test witness::tests::a_text_witness_is_its_own_path_so_two_fields_of_one_type_never_agree ... ok
test witness::tests::an_enum_offers_every_variant_it_declares_and_the_first_one_only_once ... ok
test witness::tests::an_integer_leaf_is_never_offered_a_fractional_candidate ... ok
test witness::tests::the_alternatives_for_a_number_are_the_guards_own_literals_either_side ... ok
test evidence::tests::report_readers_refuse_malformed_and_unsupported_suite_versions ... ok
test witness::tests::the_candidate_count_is_bounded_however_many_fields_a_guard_reads ... ok
test web::tests::the_page_calls_nothing_the_player_does_not_return ... ok
test witness::tests::two_uuid_witnesses_differ_and_neither_moves_when_a_third_field_appears ... ok
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
test a_positional_claim_takes_the_order_from_the_view_rather_than_from_the_author ... ok
test a_field_the_surface_does_not_declare_is_refused_by_name ... ok
test a_declared_field_nothing_supplies_is_refused_by_name ... ok
test a_document_that_is_not_one_is_refused_rather_than_read_as_an_empty_scenario ... ok
test a_command_the_model_does_not_declare_is_refused_by_name ... ok
test a_domain_the_model_does_not_declare_is_refused_by_name ... ok
test a_claim_the_timelines_own_instants_contradict_is_refused ... ok
test a_halt_stated_beside_another_claim_is_two_assertions_filed_as_one ... ok
test a_halt_compiles_to_a_step_of_its_own_and_not_to_a_claim_about_rows ... ok
test a_bounded_negative_that_forbids_no_event_is_refused ... ok
test a_position_in_a_view_that_declares_no_order_is_refused ... ok
test a_halt_after_no_rows_at_all_is_refused_rather_than_compiled ... ok
test a_reference_where_the_suite_compares_a_value_it_carries_is_refused ... ok
test a_format_this_build_does_not_implement_is_refused_before_anything_is_read ... ok
test a_scenario_compiles_to_the_id_the_domain_and_the_name_make ... ok
test a_predicate_reading_something_the_view_does_not_publish_is_refused ... ok
test a_halt_claimed_of_a_listing_with_no_declared_order_is_refused_by_the_code_that_already_says_so ... ok
test a_scenario_that_runs_nothing_is_refused_rather_than_counted_as_a_check ... ok
test a_timeline_whose_instants_do_not_ascend_is_refused ... ok
test a_value_read_off_an_event_nothing_required_is_refused ... ok
test a_window_that_states_other_than_one_bound_is_refused ... ok
test an_act_cannot_open_a_window_at_its_own_instant ... ok
test a_window_of_no_seconds_is_refused_rather_than_compiled_into_a_check_that_cannot_fail ... ok
test a_state_the_lifecycle_does_not_declare_is_refused_as_a_state_and_not_as_a_variant ... ok
test a_view_the_model_does_not_declare_is_refused_by_name ... ok
test a_value_the_declared_type_does_not_admit_is_refused_where_it_sits ... ok
test an_elapsed_claim_compiles_to_the_four_steps_that_carry_it_and_they_come_before_the_act ... ok
test an_entity_the_model_does_not_declare_is_refused_by_name ... ok
test an_event_the_model_does_not_declare_is_refused_by_name ... ok
test a_window_measured_from_an_instant_nothing_marked_is_refused_with_the_ones_that_are ... ok
test an_assertion_that_states_other_than_one_claim_is_refused ... ok
test an_actor_the_model_does_not_declare_is_refused_by_name ... ok
test an_actor_the_specification_does_not_grant_the_command_is_refused ... ok
test an_instance_the_arrangement_does_not_declare_is_refused_by_name ... ok
test an_outcome_the_command_does_not_declare_is_refused_with_the_ones_it_does ... ok
test an_instance_bound_to_a_field_that_cannot_hold_an_identity_is_refused ... ok
test an_instance_named_before_anything_binds_it_is_refused ... ok
test an_error_the_model_does_not_declare_is_refused_by_name ... ok
test authored_predicate_operand_errors_have_their_own_refusal ... ok
test authored_aggregate_presence_keeps_026_and_valid_scalar_reads_keep_the_predicate ... ok
test two_files_naming_one_scenario_are_refused_rather_than_one_displacing_the_other ... ok
test the_order_the_files_are_handed_over_in_does_not_reach_the_result ... ok
test the_committed_billing_suite_holds_the_authored_scenario_beside_the_generated_ones ... ok
test one_name_for_two_instants_is_refused_rather_than_read_as_the_later_one ... ok
test the_steps_are_the_vocabulary_a_generated_scenario_already_uses ... ok
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
test a_completed_run_cannot_be_rebound_to_different_admitted_original_bytes ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/count_writer_pass2.rs (target/debug/deps/count_writer_pass2-9ea0525aab5cc61e)

running 1 test
test cloned_execution_binding_survives_mutation_of_extracted_legacy_diagnostics ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/elapsed.rs (target/debug/deps/elapsed-8c80e368032de232)

running 7 tests
test a_window_opened_at_an_instant_nothing_marked_is_a_suite_defect_and_not_a_failed_implementation ... ok
test a_deadline_the_target_ran_past_fails_the_within_claim ... ok
test a_target_that_holds_the_window_and_reports_it_passes ... ok
test a_target_whose_clock_never_moves_fails_rather_than_being_read_as_having_waited ... ok
test an_event_published_inside_the_window_fails_the_bounded_negative_and_nothing_else ... ok
test a_target_with_no_clock_reports_unsupported_and_the_run_fails ... ok
test two_runs_over_one_window_produce_byte_identical_reports ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/execution.rs (target/debug/deps/execution-e989a24739ec2595)

running 12 tests
test an_eventual_assertion_asks_again_within_a_deadline_and_never_sleeps ... ok
test an_eventual_view_is_read_again_and_a_read_your_writes_view_is_not ... ok
test a_read_your_writes_view_is_not_quietly_read_at_current_when_no_token_came_back ... ok
test an_event_missing_a_field_it_declares_is_named_leaf_by_leaf_rather_than_reported_as_absent ... ok
test a_value_of_the_wrong_declared_type_is_caught_by_the_same_check_as_a_missing_one ... ok
test every_scenario_checked_something_and_no_family_of_them_was_silently_empty ... ok
test a_target_that_cannot_expose_an_observation_fails_the_run_rather_than_skipping_it ... ok
test a_view_answered_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test a_view_assertion_names_the_instance_the_scenario_created_rather_than_any_row ... ok
test every_scenario_the_billing_specification_obliges_passes_against_the_reference_implementation ... ok
test a_scenario_whose_input_no_longer_reaches_its_branch_fails_with_a_diagnostic_naming_the_defect ... ok
test two_runs_of_one_suite_against_one_target_produce_byte_identical_reports ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

     Running tests/faults.rs (target/debug/deps/faults-e1b7998fd65b338f)

running 11 tests
test a_fault_nothing_catches_is_recorded_rather_than_quietly_dropped ... ok
test dropping_one_binding_leaves_the_other_two_green ... ok
test the_diagnostic_of_a_caught_fault_names_the_defect_rather_than_reporting_that_something_broke ... ok
test a_wrong_mapping_is_invisible_to_a_target_that_cannot_show_its_invocations ... ok
test every_fault_that_could_be_a_boundary_perturbation_is_one ... ok
test the_widest_blast_radius_is_scenarios_that_could_not_be_arranged_rather_than_extra_verdicts ... ok
test each_specification_is_passed_in_full_by_the_implementation_written_from_it ... ok
test two_runs_against_one_faulty_target_produce_byte_identical_reports ... ok
test a_fault_does_not_simply_break_everything ... ok
test each_fault_fails_the_scenario_that_exists_to_catch_it ... ok
test a_faults_blast_radius_is_accounted_for ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.86s

     Running tests/halt.rs (target/debug/deps/halt-998b2db32b9f7154)

running 7 tests
test a_halt_of_an_eventual_listing_is_asked_again_while_the_projection_catches_up ... ok
test a_target_that_cannot_read_a_row_at_a_time_reports_unsupported_and_the_run_fails ... ok
test retrying_does_not_rescue_a_producer_that_never_stops ... ok
test a_listing_that_ran_out_before_the_reader_stopped_it_is_not_a_halt ... ok
test a_target_that_reads_the_whole_listing_fails_rather_than_being_read_as_having_stopped ... ok
test a_target_whose_producer_stops_when_the_reader_does_passes ... ok
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
test a_suite_parses_from_text_alone_without_an_ir ... ok
test a_suite_naming_something_that_is_not_an_ess_name_is_refused_while_it_is_read ... ok
test a_count_and_a_position_read_back_as_what_a_runner_in_another_language_must_read ... ok
test the_scan_for_a_clock_finds_one_and_does_not_find_a_word_that_merely_ends_in_a_banned_token ... ok
test the_steps_a_binding_and_an_invariant_need_survive_being_read_back_from_text ... ok
test every_scenario_id_the_billing_model_can_produce_reads_back ... ok
test the_step_vocabulary_expresses_the_worked_example_from_section_ten ... ok
test a_suite_serialised_in_one_process_resolves_in_another ... ok
test the_dependency_set_names_a_type_no_derived_from_would_have_mentioned ... ok
test no_source_file_in_this_crate_reads_a_clock_or_an_unordered_map ... ok
test inserting_one_outcome_re_keys_nothing_around_it ... ok
test the_scenario_ids_appear_in_the_file_in_the_order_a_sorted_key_list_would_be ... ok
test serialising_a_suite_twice_produces_byte_identical_json ... ok
test the_suite_records_the_same_model_digest_the_projections_do ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/synthesis.rs (target/debug/deps/synthesis-9c5f74c7568d3471)

running 50 tests
test a_guard_no_candidate_can_satisfy_is_refused_with_the_number_tried ... ok
test a_binding_whose_branch_the_event_decides_refuses_the_flow_and_still_checks_the_mapping ... ok
test a_command_that_declares_no_wrong_state_answer_is_refused_by_name_beside_its_scenario ... ok
test a_command_that_accepts_a_wrong_state_is_asserted_as_accepting_rather_than_refusing ... ok
test a_filter_reading_something_no_scenario_knows_refuses_rather_than_guessing ... ok
test a_parameterised_view_is_queried_with_the_value_the_scenario_put_in_the_row ... ok
test a_component_nothing_declares_is_refused_by_name ... ok
test a_state_reached_only_through_a_branch_no_input_reaches_is_refused_rather_than_arranged ... ok
test a_binding_that_retries_forces_one_failure_and_still_requires_the_consequence ... ok
test a_read_your_writes_view_filled_by_the_command_that_ran_is_asserted_to_hold_a_row ... ok
test a_declared_error_is_asserted_by_name_and_never_by_an_invented_payload ... ok
test a_value_object_nothing_observable_holds_keeps_a_refusal_naming_what_would_close_it ... ok
test an_entity_nothing_creates_cannot_be_acted_on_and_says_so ... ok
test a_binding_flow_is_proved_through_the_event_the_invoked_command_publishes ... ok
test a_value_objects_own_invariants_are_read_at_every_field_position_a_view_holds_one ... ok
test a_declared_order_is_asserted_against_two_rows_the_scenario_arranged_itself ... ok
test a_view_that_does_not_hold_the_instance_yet_is_not_asked_about_its_invariants ... ok
test a_binding_that_drops_its_failures_refuses_that_check_and_names_the_reason ... ok
test a_move_is_observed_through_the_view_the_state_it_left_is_filtered_on ... ok
test a_binding_mapping_names_the_source_the_document_wrote_and_not_its_same_typed_sibling ... ok
test a_view_the_entity_has_not_reached_yet_is_asserted_to_exclude_the_instance_by_name ... ok
test an_undecidable_guard_refuses_and_does_not_spend_the_candidate_budget ... ok
test a_move_that_is_illegal_in_a_state_is_attempted_with_the_input_that_would_have_worked ... ok
test a_binding_that_escalates_requires_the_event_the_escalation_declares ... ok
test an_order_the_specification_cannot_put_two_rows_under_is_refused_and_not_asserted ... ok
test a_synthesised_count_is_a_floor_the_scenario_arranged_and_never_a_ceiling ... ok
test an_at_least_once_binding_delivers_the_event_twice_and_requires_no_count ... ok
test a_scenario_that_moves_an_instance_names_the_one_an_earlier_step_created ... ok
test a_synthesised_suite_survives_being_written_and_read_back ... ok
test a_view_is_asserted_in_the_block_its_own_consistency_decides ... ok
test an_invariant_over_a_field_no_view_publishes_refuses_rather_than_being_dropped ... ok
test an_illegal_move_requires_the_branch_and_the_declared_error_rather_than_merely_failing ... ok
test a_suite_for_one_component_holds_only_what_that_component_realises ... ok
test an_event_assertion_carries_the_declared_shape_and_exactly_the_values_the_payload_determines ... ok
test an_outcome_that_updates_an_instance_acts_on_one_the_scenario_created ... ok
test a_whole_system_suite_does_not_mention_a_component ... ok
test an_actor_is_named_only_where_the_specification_grants_the_command ... ok
test every_command_names_an_instance_an_earlier_step_of_the_same_scenario_bound ... ok
test the_failure_control_is_armed_after_the_arrangement_and_before_the_command_that_triggers_it ... ok
test the_refusal_branch_asserts_that_no_event_the_specification_declares_occurred ... ok
test the_dependency_set_names_the_types_the_scenario_is_made_of ... ok
test an_invariant_is_asserted_against_every_view_that_publishes_what_it_reads ... ok
test an_outcome_no_input_decides_is_reached_by_injection_and_by_nothing_else ... ok
test the_input_a_scenario_sends_is_re_decided_against_the_guard_it_claims_to_reach ... ok
test every_declared_outcome_is_either_a_scenario_or_a_named_refusal_or_asserted_by_the_state_family ... ok
test every_declared_transition_has_a_scenario_that_proves_it_can_occur ... ok
test each_example_synthesises_the_families_its_specification_declares ... ok
test every_clause_of_every_binding_is_either_a_scenario_or_a_named_refusal ... ok
test synthesising_the_same_specification_twice_produces_byte_identical_output ... ok
test canonical_expression_compatibility_fixtures ... ok

test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s

     Running tests/witness.rs (target/debug/deps/witness-8d060cd948fd3c6d)

running 28 tests
test a_candidate_carrying_a_field_no_type_declares_is_refused ... ok
test a_candidate_input_projects_one_fact_per_scalar_leaf ... ok
test a_candidate_missing_a_required_field_is_refused_before_any_guard_is_read ... ok
test a_newtype_is_transparent_so_a_deep_path_reaches_through_it_without_a_segment ... ok
test a_conjunction_of_two_undecidable_leaves_reports_both ... ok
test a_path_into_a_list_or_a_union_names_the_aggregate_rather_than_the_missing_element ... ok
test a_newtype_is_transparent_when_a_path_is_resolved_as_well_as_when_it_is_projected ... ok
test a_disjunction_one_of_whose_branches_holds_is_satisfied_despite_an_undecidable_branch ... ok
test a_refusal_names_the_predicate_the_command_and_the_path ... ok
test a_scalar_of_the_wrong_shape_is_refused_rather_than_coerced ... ok
test an_absent_optional_binds_nothing_rather_than_binding_a_default ... ok
test an_absent_optional_is_unevaluable_but_says_a_candidate_could_repair_it ... ok
test a_path_landing_on_an_aggregate_is_unevaluable_by_construction ... ok
test a_refuted_guard_carries_the_leaf_and_the_value_that_refuted_it ... ok
test legal_collection_cardinality_is_not_currently_projected ... ok
test a_list_a_map_and_a_union_bind_no_fact_in_the_current_projection ... ok
test a_long_recursive_read_validates_beyond_the_projection_limit ... ok
test equality_over_two_texts_is_decided_even_though_ordering_them_is_not ... ok
test expression_search_limits_do_not_define_type_correctness ... ok
test the_same_text_ordering_is_decidable_once_a_scale_contains_both_values ... ok
test the_normative_shape_of_guard_is_decidable_for_both_signs ... ok
test resolved_adapter_keeps_semantics_separate_from_collection_projection ... ok
test otherwise_and_external_are_not_guards_over_the_input ... ok
test ordering_two_texts_is_unevaluable_because_an_ess_specification_declares_no_scale ... ok
test unclassified_is_a_drift_alarm_and_no_enumerated_source_trips_it ... ok
test malformed_declarations_refuse_early_and_direct_bad_reads_remain_unknown ... ok
test ordering_across_two_types_is_unevaluable_not_false ... ok
test only_an_absent_value_says_another_candidate_would_help ... ok

test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests ess_conformance

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p ess-cli --test count_writer_pass2`
exit: 101
```

4. Finding against frozen source 1be4dbd999b20e44ba0f04e87d1c4ddf81c1b4ab

| File:line | Category | Severity | Verdict | Origin | Finding |
|---|---|---|---|---|---|
| crates/verify/ess-conformance/src/go/runtime.go:2294 | acceptance | blocker | CONFIRMED | introduced | Generated Go admission still accepts invalid leaf fact paths and unknown constraint operators, allowing refused original predicates to construct targets and publish report/2. |

What was measured: the assertion at `crates/edge/ess-cli/tests/count_writer_pass2.rs:129` fails, focused exit 101 and complete-suite exit 101. Before formatter-only changes, the same assertion is line 76 in the first raw output. The exact predicate originals are `{"ready..done":{"eq":true}}`, `{"ready":{"eq":true,"future":true}}`, and `"ready..done == true"`. Each is refused by public Rust `AdmittedSuite::from_json`. Each actual generated-Go invocation exits 0, creates its target marker, and writes complete report/2. The valid `{"ready":{"eq":true}}` control admits and publishes exactly one skipped outcome. No setup failure is used to substantiate this finding.

What reaches it: generated public `Run` reads complete embedded original suite JSON, enters `admitSuite` / `admitExpectation`, then constructs its target. The test emits the actual Go package through public `go::emit` and changes only test-owned embedded originals. Its ordinary Target.BeginScenario returns ErrUnsupported, proving malformed original predicates reach a callback and complete reporting even when their assertion is never evaluated. No production runtime is edited. The emitted runtimes from both focused Go modules have SHA256 `0a6fa1e1025a34f27c52273acf21fdab8baa0a10d7fd32d6c5d3723beaa6f702`, identical to the frozen template.

Mechanism and ownership: the corrected `admitPredicateEnvelope` validates quantifiers and recursion but its default mapping/string leaves do not validate fact paths or close constraint operators. `admitExpectation` then treats the historical evaluator parser at runtime.go:2294 as admission. That parser's `parseConstraint` and `parseLeaf` accept the malformed leaves and ignore the extra operator once a known one is found. The base-to-current diff for predicate.go is empty: evaluator permissiveness itself is historical. The introduced defect is the new strict original-admission/report2 path accepting these originals; the original base has no such admitted count producer. This is not a request to rewrite the legacy raw Rust DTO contract. The coordinator binding's pre-callback closed vocabulary/semantic checks and P8 require refusal before target construction.

Previous signatures, retained without rewriting either report:
- Pass 1 `crates/verify/ess-conformance/src/counts.rs:171 / CONFIRMED / introduced`: original binding case now passes; the new clone/extracted-diagnostics control also passes.
- Pass 1 `crates/verify/ess-conformance/src/go/runtime.go:2284 / CONFIRMED / introduced`: the retained quantifier/invalid-binder/depth case passes, while this pass's new leaf/path/operator case demonstrates adjacent residue of the same original-admission class at line 2294. The new signature is not silently substituted into the old report.
- Pass 1 `crates/verify/ess-conformance/src/go/runtime.go:566 / CONFIRMED / introduced`: original abnormal-teardown case passes; the additional returned-error-formatting Goexit hypothesis was rejected by the actual host test process.

The finding table and final machine-readable block use the identical message and fields. No additional judgement-only finding is returned. Coordinator owns carried/new/resolved ledger interpretation and routing; this is the second and last full attack.

5. Coverage and limits

- Exact review basis is the assigned original-base-to-current change, correction diff, retained tests/callers, unit acceptance and coordinator binding at 9671340000fd15a60ef7d3ead59a73a14c0cf7ab. Old unit binding text was not substituted for its execution-capability clarification.
- Clone/extraction leaves the retained ExecutedRun and both canonical count outputs unchanged; CRLF original versus decoded-equal Unicode-escaped original remains separately bound, and both paired readers refuse the other original.
- Actual Go error formatting Goexit before SkipNow exits 1 without report, with and without a destination; ordinary unsupported begin remains skipped. This passing control is not claimed as an internal completion proof beyond its measured workflow.
- All 405 inherited cases passed, including the four prior adversary cases and retained full-u64/lexical, detailed closure/order/arithmetic, raw DTO/admitted capability, decoded duplicate keys, wrong-major vocabulary, opaque IDs, strictness, omitted subtest, abnormal callbacks and actual old-runtime controls. Inherited assertions are not relabeled as newly authored attacks.
- Review covers count-stage suites1–4/report2/run2 and frozen legacy controls only. It establishes no suite5 inventory, all75 matrix rows, browser/impact, default movement, installed-client or adopter readiness. Root's separate corrected producer→both AEP readers/replay exercise remains separate; no AEP write occurred.

6. Writes, retained source and relinquishment

The three new test/fixture paths immediately after the header are the only authored source additions, totaling 270 lines. Final tracked diff is empty. All 23 files in correction-pass-1/source-manifest.json verified unchanged before and after package execution; final-frozen-source-check.txt contains 23 OK records. This includes production, prior tests, manifest, Cargo.lock and changed public documentation. The pre-suite focused section and all three new test hashes verified unchanged after the complete suite. Package formatter check and git diff --check both exited 0. No strict Clippy or full repository/site gate was rerun in this tests-only review; those remain coordinator-owned.

Initial immutable report SHA256 remains `32563d1fcbcc00489f7d6f897a0c6f471d2774ef99e65ad4a861e69c959b84ff`; correction report remains `b5197b11aa13bf152a09dd03ea6d896ca1d19a485b3f2055f919c8db94796296`. No earlier report, production file, existing assertion, planning artifact, Git index/ref/object or managed-worktree lifecycle was edited.

All authored scratch, complete command/stdout/stderr/exit records, generated modules, markers/reports and caches are under:
`/home/timo/.local/state/worktree/trees/b10x/ess/ess-conformance-count-writer/target/review-boundaries-8/adversary-pass-2`.
TMPDIR, GOCACHE and GOMODCACHE are its `tmp`, `go-cache` and `go-mod-cache` children. Compilation uses only the assigned worktree's existing target. Inherited package tests also regenerate their established in-tree producer exports and test scratch, including pass1 CLI PID modules under correction-pass-1/go-tests; those helpers were preserved. No new test intentionally rewrites shared exports.

No authored file outside the worktree was written. Cargo's pre-existing shared infrastructure metadata `/home/timo/.cargo/.global-cache` is conservatively disclosed as one external path: 1,007,616 bytes, final observed mtime 2026-09-06 12:18:55.521773819 +0200. Cargo and coordinator activity share that file, so attribution to a specific process is not established. No external Go cache was selected and no daemon was launched.

Every owned command session has exited; no owned Cargo/test/Go process remains. Final observed free space was 23,233,638,400 bytes, above the 8 GiB floor. Report is immutable on return, with matching findings.yaml, and all source/scratch writes are relinquished to the coordinator. No third attack is launched.

```findings
- file: crates/verify/ess-conformance/src/go/runtime.go
  line: 2294
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Generated Go admission still accepts invalid leaf fact paths and unknown constraint operators, allowing refused original predicates to construct targets and publish report/2.
```
