---
format: aep.planning-md/3
id: review-result:wave-2026-11-22-whole-diff-security
kind: review-result
status: active
title: Security review of the whole wave of 2026-11-22
relations:
- reviews: story:each-code-target-admits-binary64-through-a-switch-of-its-own
- reviews: story:the-rust-target-carries-a-finite-binary64
- reviews: story:the-go-target-carries-a-finite-binary64
- reviews: story:the-web-target-carries-a-finite-binary64
- reviews: story:command-format-hints-name-the-source-that-declares-the-system
- reviews: story:type-view-binding-format-hints-name-the-source-that-declares-the-system
- reviews: story:expression-format-hints-and-the-header-refusal-name-the-system-source
- reviews: story:report-out-writers-replace-their-file-through-one-helper
- reviews: story:one-pinned-rustc-in-rust-toolchain-toml-read-by-every-workflow
- reviews: story:cargo-xtask-pr-watch-exits-non-zero-when-a-pull-requests-checks-end-red
- reviews: story:validation-refuses-an-ordering-comparison-over-an-enum
- reviews: story:validation-refuses-an-exists-false-branch-declared-below-a-refusal-it-answers-before
- reviews: story:delivery-trust-fixture-race
revision: 1
---
unit: wave of 2026-11-22, `origin/main...a3e75dfd8d`; my cases are on `review/w1122-sec` at commit 7c9fc9b813
verdict: NEEDS-CHANGE
cases: executed 5→8, red 3 (count_reports target)
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 6 paths (listed below)
needs-coordinator: Binary64 release note and doc edits (F4)

1. Diff stat: `crates/edge/ess-cli/tests/count_reports.rs | 62 +` (one test file, nothing else).
2. Cases added, each run alone and red for the reason named (build 9m48s, run 2.3 s):
   - `review_a_report_out_of_dev_stdout_writes_the_report_to_standard_output`: fails with `error: writing /dev/stdout: No such file or directory (os error 2)`.
   - `review_a_report_out_of_dev_null_succeeds`: fails with `error: writing /dev/null: Permission denied (os error 13)`.
   - `review_a_report_out_of_a_non_regular_file_does_not_replace_it`: fails with `the socket at the destination was replaced: is_file=true`.
3. Suite run (`cargo test -p ess-cli --locked --test count_reports`): 5 passed, 3 failed, exit 101, 9.4 s. Run with `--skip review_`: 5 passed, exit 0.
4. Findings:
   - **F1 S2, rest (report writers), `crates/edge/ess-cli/src/replace_file.rs:55-58,35-47`, red (the three cases above).** The helper follows a link with `canonicalize`, makes its temporary file in the destination's folder and renames it over whatever is there.
     - `--report-out /dev/stdout` (piped output) and `--report-out /dev/null` now fail. 0.97.4 wrote them with `fs::write` (`main.rs:3984,5000,5087,5104`).
     - A destination that is neither a regular file nor a folder gets replaced by a regular file. A root user in a container would replace `/dev/null` itself.
     - Fix: replace only a destination that is absent or a regular file; write anything else in place. CHANGELOG has no entry for this change in behaviour.
   - **F2 S3, rest, `replace_file.rs:39-45`, read, not run.** The temporary file is created with default permissions and written before the old file's permissions are copied onto it. An earlier report with mode 0600 is readable by others while the write runs. Fix: set the permissions at creation.
   - **F3 S2, rest, `crates/edge/ess-xtask/src/pr_watch.rs:115-134`, read, not run.**
     - `decide` returns Passed once every check run that exists has completed. There is no list of required checks.
     - Between the last shard finishing and `Gate`'s check run being created, a poll exits 0 before `Gate` has run.
     - `skipped` counts as passing, while `Gate` fails a skipped job that was not plan-only. So a run that ends red can be reported green.
     - Input: `[{"name":"test (1/20)","status":"completed","conclusion":"success"}]` gives exit 0.
   - **F4 S3, binary64 (B), `CHANGELOG.md:3`, read, not run.** The `### Added` entry for Binary64 in rust, go and web, which `the-web-target-carries-a-finite-binary64.md:193` asks for, is missing. These pages still say every target refuses Binary64:
     - `website/docs/concepts/ess.md:200`, `concepts/adoption-modes.md:24`, `guides/synthesize.md:79`
     - `docs/design/model-binary64.md:125,226,283`, `binary64-structural-codecs.md:5`
     - The new detail text for positions the targets still refuse is not mentioned either.
   - **F5 S3, VR, `CHANGELOG.md:9` against `crates/specify/ess-domain/src/command/precedence.rs:491-495`, read, not run.** The **Breaking** entry says "from `ess/27`". The code also refuses an `exists: false` over a stored reference at any format that has one (`command.rs` `validate_lead_below_present_related_refusal`). I did not show that such a model at ess/22–26 can actually be written.
   - Per-group verdicts: B no code finding, F4 is docs. F no finding. VR F5. S not reviewed (other reviewer). FX no finding. Rest F1, F2, F3.
5. Checked and could not fault:
   - `compiled_fixture.rs`: publishes by hard link, accepts AlreadyExists, removes the staging file, never replaces a published file.
   - Binary64: Rust `token` and Go `Token` give the same bytes for 0, -0, 1e15, 1e16, 2^53+1, 5e-324 and the largest finite value. NaN and the infinities are refused by `from_token` / `NewBinary64`, and Go's discarded ParseFloat error can only be an overflow to infinity, which is still refused.
   - Every addition is gated on `used(ir)`, so a model without Binary64 generates the same bytes. Guard literals are quoted with `{:?}` and `go_string`, so a number's spelling cannot inject code.
   - The toolchain step in the workflows only accepts digits. No HashMap or HashSet added, no `.py` or `.sh`, no new format version needed, no downstream name.
   - `pr_watch` is bounded by `polls`. Each `gh` call has no timeout of its own (note only).
6. Paths outside the worktree (under `close/sec/`): `STATE.md`, `commit-msg.txt`, `run-review-cases.log`, `run-suite.log`, `run-suite-before.log`. The sixth is the build folder `target/` inside the tree, already cleared with `worktree discard-cache`.
   Builds: one ess-cli test build, 9m48s wall, plus two re-runs of about 15 s each.
   Not done: leads 1 and 2. Lead 3's false positives and negatives beyond the scope check.

```findings
- file: crates/edge/ess-cli/src/replace_file.rs
  line: 55
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: report writers now refuse /dev/stdout and /dev/null and rename a regular file over any non-regular destination, where fs::write wrote through it
- file: crates/edge/ess-cli/src/replace_file.rs
  line: 39
  category: integrity
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the temporary file holds the report under default permissions before the earlier file's restrictive mode is applied
- file: crates/edge/ess-xtask/src/pr_watch.rs
  line: 115
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: pr-watch exits 0 when every existing check run passed even though Gate has not been created yet, and counts skipped jobs Gate fails as passing
- file: CHANGELOG.md
  line: 3
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Unreleased has no Binary64 code-target entry and four public pages still say every synthesis target refuses Binary64
- file: CHANGELOG.md
  line: 9
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the breaking entry scopes the exists-false refusal to ess/27 while the code applies it to a stored reference at every format, a model not shown to be reachable below ess/27
```
