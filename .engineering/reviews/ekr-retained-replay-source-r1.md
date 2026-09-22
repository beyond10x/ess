unit: story:retained-command-result-replay, candidate 5ae0d322b5b9d2cd878b136db14352ca0cca79c8 against 16aa8c7617214420d7d7f2108d0a896a5ed14eb0
verdict: NEEDS-CHANGE
cases: executed 1696→1700, red 4
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 output roots; exact paths and retained files in private-paths.md
needs-coordinator: route both blockers to implementation, preserve the four regression cases, then obtain correction verification and the repository gate

```text
$ git --no-pager diff --stat
 .../ess-conformance/tests/retained_replay.rs | 221 +++++++++++++++++++++
 1 file changed, 221 insertions(+)
```

The additional untracked test fixture is `crates/verify/ess-conformance/tests/fixtures/adversary-retained-projection.go` (46 lines). These are the only two authored worktree paths. There are no implementation, document, manifest, planning, or existing-assertion edits. The original tracked test file is an exact byte prefix of its final version (`cmp` exit 0). All tracked SHA-256 values match the opening manifest except that appended test file. The immutable candidate, including its production Rust/Go and inherited tests, is unchanged.

## Cases written, then executed individually

Every command below used the assigned external Cargo target, two jobs, no incremental/dev/test debug info, and the assigned external TMPDIR. Go commands used the exact CI Go 1.25.10 binary with `GOTOOLCHAIN=local`; the scoped suite also used the assigned JavaScript tools. Full absolute expansions are retained privately.

Each isolated Rust command had this form:

```sh
cargo test --offline --locked -p ess-conformance --test retained_replay CASE -- --exact --nocapture
```

1. `adversary_replay_requires_complete_actual_subject_rows_not_only_complete_view_declarations`, at `crates/verify/ess-conformance/tests/retained_replay.rs:1053`, drives the real generated Seed replay witness. Complete actual rows pass the positive control. Actual rows containing only the identity still produce a passing scenario, so the negative assertion fails. The subsequent missing-stamp and wrong-type loop entries are authored but **not reached** because identity-only fails first. No execution claim is made for those entries.

   The first invocation (`first-complete-rows.log`, exit 101) encountered my missing required `redeliver_event` delegation in the new test wrapper. This was a test setup mistake, not a product finding. I added the delegation only to my wrapper. The first executable behavior run (`first-complete-rows-behavior.log`, exit 101) then recorded:

   ```text
     left: Passed
    right: Failed
   test adversary_replay_requires_complete_actual_subject_rows_not_only_complete_view_declarations ... FAILED
   test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.01s
   ```

2. `adversary_replay_must_reach_its_own_state_guard_after_the_origin`, at `retained_replay.rs:1080`, changes only the fixture replay guard from Committed to Rejected. The model validates and compiles. Its originating commit reaches terminal Committed, but synthesis emits the replay scenario with zero refusals. `first-replay-eligibility.log`, exit 101:

   ```text
   the original commit ends in terminal Committed, so the same subject cannot satisfy replay's Rejected guard; generation must name the unavailable witness, not invoke it in Committed: []
   test adversary_replay_must_reach_its_own_state_guard_after_the_origin ... FAILED
   test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 16 filtered out; finished in 0.01s
   ```

3. `adversary_source7_wrong_state_refusal_requires_actual_complete_subject_observation`, at `retained_replay.rs:1155`, selects the real generated external-Stale → Validate/refused scenario and delegates its existing backend while removing the actual `note` field from both fresh queries. Identity and state remain present. The new source7 complete-refusal witness still passes. `first-refusal-rows.log`, exit 101:

   ```text
     left: Passed
    right: Failed
   test adversary_source7_wrong_state_refusal_requires_actual_complete_subject_observation ... FAILED
   test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.01s
   ```

4. `adversary_go_replay_requires_complete_actual_subject_rows`, at `retained_replay.rs:1177`, emits the ordinary Go runtime, then runs the added fixture via `go test -json -count=1 ./essconform -run ^TestAdversarySnapshotProjection$`. Its complete positive child passes. Its identity-only negative child incorrectly passes the generated replay, causing the wrapper assertion to fail. `first-go-complete-rows.log`, Rust exit 101 / nested Go exit 1:

   ```text
   incomplete: identity-only rows must not certify complete original subject preservation: <nil>
   --- PASS: TestAdversarySnapshotProjection (0.00s)
       --- PASS: TestAdversarySnapshotProjection/retained.core.Seed/outcome/replayed (0.00s)
   PASS
   ```

   Those are the invalid child execution's lines preserved inside the Go negative-control diagnostic, not a passing verdict for the outer test. The Rust wrapper reports:

   ```text
   Go must reject incomplete actual replay subject rows
   test adversary_go_replay_requires_complete_actual_subject_rows ... FAILED
   test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.86s
   ```

The isolated logs preserve complete output, including diagnostics omitted from these excerpts. Earlier line numbers in those logs precede final formatting and later additions. The references above identify the handed-back test file.

## Scoped suite after the isolated cases

The implementing report supplies the baseline: 1,696 passed, 0 failed, 7 existing ignored for these same six packages. I did not run a pre-addition baseline.

```sh
cargo test --offline --locked -p ess-domain -p ess-compiler -p ess-conformance -p ess-synth -p ess-gen -p ess-diff --no-fail-fast
```

`six-packages.log` contains the full runner output, exit **101**. Summing its 148 actual `test result:` summaries gives **1,696 passed, 4 failed, 7 ignored**. Only the four added cases fail. The terminal output is:

```text
error: 1 target failed:
    `-p ess-conformance --test retained_replay`
```

No repository-wide, release, or xtask gate is claimed here. `cargo clippy --offline --locked -p ess-conformance --tests -- -D warnings` exited 0 (`clippy.log`). Single-file `rustfmt --edition 2021 --check`, `gofmt -l` on the added fixture, and `git diff --check` succeeded; gofmt printed no paths. No source correction or mutation was performed.

## Findings

| ID | Source | Category | Severity | Verdict | Origin | Finding |
|---|---|---|---|---|---|---|
| F1 | `crates/verify/ess-conformance/src/synthesize.rs:1677` | acceptance | blocker | NEEDS-CHANGE | introduced | New complete-subject witnesses validate view declarations but carry no required row-shape authority, so incomplete actual rows can certify preservation. |
| F2 | `crates/verify/ess-conformance/src/synthesize.rs:1643` | acceptance | blocker | NEEDS-CHANGE | introduced | Replay synthesis reuses the origin's resulting subject and input without establishing the replay branch's own guard, generating an unreachable Rejected replay after a terminal Committed origin. |

**F1 — measured:** three isolated red cases above: Rust replay, Rust source7 ordinary wrong-state refusal, and Go replay. The real generated scenario reports Passed when actual subject fields are absent; the added negative assertion turns red. **Reachability:** public `ConformanceTarget::query_view` / Go `Target.QueryView` results feed the normal admitted-suite runner. The fixture changes an ordinary target result, not internal runner state or suite admission. A projection defect in a real adapter can therefore receive a passing complete-preservation witness.

The new replay call at `synthesize.rs:1677` and source7 complete-refusal path reuse `synthesize/subject_fact.rs:392–423`. That helper checks declared view fields, then emits identity-only SnapshotSubject metadata. Rust `runner.rs:1923` and Go `src/go/runtime.go:2139` snapshot and compare actual matching rows without required field/type authority. This contradicts `docs/design/retained-command-results.md:38–40,94–100,145–146`, which requires complete actual subject observation. The helper's old untyped behavior is present at the base; the finding is classified introduced for the **new replay/source7 complete-subject guarantees and their new call sites**, not as a claim that the old helper was authored by this unit. No legacy behavior was changed or executed against a substituted base tree. Correction must retain legacy suite behavior while supplying enough authority for the new complete observation.

**F2 — measured:** the public source parser, specification assembler, compiler and synthesizer admit the changed fixture; synthesis generates `retained.core.Commit/outcome/replayed` and has an empty refusal list, although the original success leaves its subject terminal Committed and the replay requires Rejected. The isolated test fails at `retained_replay.rs:1098`. **Reachability:** this is ordinary documented source7 `when_subject_state` plus `replays` input. The default correct EKR guard is not alleged to be wrong; the newly supported generic compiler path admits an impossible witness for another validly parsed model.

`run_replay` starts with `run(... original ...)` at `synthesize.rs:1643` and invokes the retry at `:1689–1697` with the original input. It does not establish the replay's own eligibility on that exact post-origin subject. A correction needs a provable applicable immediate retry with actual observation, or a named refusal; forcing the replay branch or silently changing the subject/input does not satisfy the adopted contract. If a correction instead rejects this model earlier, it must deliberately preserve the test's refusal intent rather than allowing an early unwrap panic to masquerade as acceptance.

Owners: the implementor owns both new synthesis guarantees and the Rust/Go execution authority needed for complete observations. F1 exposes an inherited helper limitation through new promised semantics; this is not a request to rewrite legacy suite bytes. The adopted coordinator design already requires full actual subject observation and held-state eligibility, so these findings do not require a new product decision. My first test-wrapper compilation mistake is mine and is excluded from the findings. Earlier coordinator numeric-contract findings remain in their separate review; this pass does not relabel them or claim their work as new source defects.

## Bounds of this pass

- Existing selected identity, retained-response, direct-event/error, legacy-byte, native-adapter, projection and diff cases remained green in the executed six-package suite. These are inherited cases, not newly authored adversarial coverage.
- The complete-row positive controls passed in Rust and Go before their corresponding negative assertions failed.
- All inherited production and test bytes remain unchanged; the only modified tracked test file has an exact unchanged original prefix.
- Actual EKR draft activation was inspected as read-only context. This pass does not execute an EKR writer, restart, later-head retry, physical persistence, or transaction parsing.
- No claim is made that generic raw JSON number decoding is a certified exact-integer adapter; no numeric format, legacy serializer, or source≤6 contract was changed.
- The added source7 omission probe executed in Rust; Go corroboration here covers replay. Missing-stamp/wrong-type entries after the first Rust omission failure remain unexecuted.

## External outputs and handback

The five output roots are assigned review scratch, assigned Cargo target, assigned temporary fixture directory, the standard Go build cache used by existing fixture commands, and managed-worktree lease state. `private-paths.md` records their full host paths, the retained generated fixture roots, and every retained scratch file. Raw logs remain private because tool diagnostics contain host paths. The public report uses relative source and log names. No commit, planning command, source edit, cleanup, or publication was performed. Lease release and process status are recorded in `lease-release.log` and `handback-status.md`.

```findings
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 1677
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: New complete-subject witnesses validate view declarations but carry no required row-shape authority, so incomplete actual rows can certify preservation.
- file: crates/verify/ess-conformance/src/synthesize.rs
  line: 1643
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Replay synthesis reuses the origin's resulting subject and input without establishing the replay branch's own guard, generating an unreachable Rejected replay after a terminal Committed origin.
```
