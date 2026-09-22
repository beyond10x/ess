unit:                   story:retained-command-result-replay — complete subject observation and reachable retry correction
verdict:                green
cases:                  executed 453→463, red 0 in the frozen whole-package run
origin:                 n/a
wrote-outside-worktree: assigned scratch, dedicated target, TMPDIR, Go cache and managed lease registry; full paths in private-inventory.md
needs-coordinator:      yes — independent correction review, integration and repository/release qualification remain coordinator-owned

## 1. Unit, acceptance and ownership

Close F1 by validating the actual selected subject row against complete typed authority before original capture and before post-command comparison. Close F2 by proving immediate replay eligibility on the original input, actor and post-origin subject. Preserve all four independent regression assertions, the adversary Go fixture and legacy source/suite semantics.

Opening commit is `89cab6479075efa37f2578a86913f8248db96872`. The correction follows the adopted complete-observation/reachable-retry design addendum and correction brief. Owners: the primary implementor owns the Rust conformance protocol, admission, synthesis, typed helper and Rust tests. The separately dispatched Go implementor owns only `src/go/runtime.go`, `src/go/replay.go` and `tests/fixtures/retained-replay-runtime.go`. The coordinator owns representation adoption, design/docs, AEP, integration, commits and release. Neither worker changed the four independent assertion bodies or their Go fixture.

The initial independent conformance package contained 449 passing and four failing cases, extracted from its retained six-package runner output. This correction's first own command reran the four actual failures. The package adds ten Rust cases; the retained-replay target grows from 19 to 29 executed cases. Projection packages add no cases; their unchanged count is regression coverage, not new acceptance evidence.

## 2. Change and correction classes

`source-stat.txt` and `source.patch` capture the actual complete diff, including the new `src/subject.rs` which ordinary unstaged `git diff --stat` omits until tracked. The complete source manifest is `source-final.sha256`.

```text
13 tracked files changed, 1598 insertions(+), 106 deletions(-)
new src/subject.rs: 181 insertions(+)
```

F1's class is a complete preservation witness whose declared projection coverage does not constrain actual observed rows. The separate `SnapshotCompleteSubject`/`ExpectCompleteSubjectUnchanged` pair carries a required closed `SubjectShape { identity_field, fields, declarations }`. It reuses the finite Field/Declaration type machinery. Both original and subsequent selected rows are validated against the retained descriptor; then all keys, including allowed extras, are compared exactly. Optional absence remains distinct from present null. Complementary views retain the existing coverage proof and each gains its own actual typed observation. Legacy snapshots retain their previous wire shape and behavior.

Closed admission rejects missing/unknown descriptor fields, unused declarations, unsupported recursive/floating observers, missing or optional identity fields, unbound or overwritten identities, wrong views, one-sided substitutions, missing pairs and old envelopes. Queries must be fresh at each command boundary. Retained replay requires the complete pair. Generic error/no-events assertions remain independent and do not imply that a subject exists. Replacing both complete steps with legacy assertions is a changed witness, not an authenticity claim; a source7 generation mutation tests the missing obligation.

F2's class is generating a retry without proving that the same originating invocation can select it immediately. Synthesis evaluates every currently admitted retry condition against the same original input and post-state, accounts for competing ordinary/default branches, observes relevant state/facts before retry and returns a named refusal for unknown or unreachable eligibility. Existing explicit unsupported replay-condition admission remains intact: this correction does not add SubjectField, StateChange or WrongState replay declarations. It evaluates those existing competing selector forms where relevant. It does not inject the replay outcome, mutate later state, invent input values or create a replacement subject.

The Rust source/test paths are `src/{admission,input,lib,replay,runner,scenario,subject,synthesize}.rs`, `src/synthesize/subject_fact.rs`, `tests/retained_replay.rs` and `tests/synthesis.rs`, all under the existing conformance crate. No inferred cross-crate implementation scope was needed. The Go split is separately described in its report and compatibility addendum.

## 3. Observed red evidence

```sh
cargo test --offline --locked -p ess-conformance --test retained_replay adversary_ -- --nocapture
```

`baseline-red.log`, exit **101**, ends verbatim:

```text
failures:
    adversary_go_replay_requires_complete_actual_subject_rows
    adversary_replay_must_reach_its_own_state_guard_after_the_origin
    adversary_replay_requires_complete_actual_subject_rows_not_only_complete_view_declarations
    adversary_source7_wrong_state_refusal_requires_actual_complete_subject_observation

test result: FAILED. 0 passed; 4 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.75s
```

The full log preserves the actual invalid Passed scenarios and unreachable retry diagnostic. These are behavior failures, not compile errors. `shape-compile.log` is an intermediate new-step admission failure, and `predicate-red.log` was an invalid new source fixture missing an exhaustive branch. Neither is counted as a defect witness. The corrected input-condition test subsequently passed and was independently made red by disabling eligibility.

The first full package run additionally caught a correction-introduced legacy Go decoding regression: the custom step decoder unconditionally chose exact numbers, changing old suite numeric representation. `package-final.log` records **462 passed, one failed**, exit **101**, in the unchanged authored entity-setup target. The actual exported suite is **suite/6**. Removing that override exposed a second compatibility boundary: direct `json.Unmarshal` of an existing Step must still decode optional event-shape leaves. `package-corrected-final.log` records **462 passed, one failed**, exit **101**, in the unchanged optional-shape target. Both failures were routed to the Go owner; the original reports and failing exports remain retained. No existing assertion was relaxed.

The final correction preserves ordinary direct Step decoding, including legacy numbers and event shapes. Only admitted suite12/13 raw steps use the exact decoder, shared with new-protocol preflight. Both legacy regressions now pass in the final package. The Go owner extended an existing integer input test to execute both actual callbacks and added an explicit ordinary Step numeric-decode control; the final fixture executes186 outcomes. The split's compatibility addendum retains the actual red/green paths and exact-input mutation.

## 4. Verification and mutations

Exact expanded environment/commands are in `commands-private.txt`. All runs use the assigned target, two jobs, no incremental/dev/test debug info, external TMPDIR, exact CI **Go1.25.10**, local toolchain selection, offline Go modules and provisioned JavaScript tools. Rust wrapper counts and Go terminal test/subtest outcomes are reported separately.

The first combined corrected retained target passed **29/29** (`combined-first.log`, exit0). This includes the four unchanged independent regressions, stage-sensitive original and post row controls, descriptor/pair refusal, optional/null/extras, nested collections/enums, complementary view execution, mixed legacy/complete admission and supported/unsupported eligibility. Its emitted Go fixture passed **185** terminal test/subtest outcomes; the split separately reran its original111 outcomes.

```sh
cargo test --offline --locked -p ess-conformance --no-fail-fast
```

The final frozen run is `package-frozen-final.log`, exit **0**, with **463 passed, zero failed/ignored**, summed from its actual runner summaries. This includes the final fixture and both unchanged legacy targets; no selection filter or skip was used. The retained target prints:

```text
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.34s
```

The complete unabridged output, including durations, is retained in the log. The opening independent package lane was453 executed (449passed/4failed); final463 are allpassing. The final emitted Go fixture executes **186** terminal test/subtest outcomes, zero failed/skipped, counted separately from its Rust wrapper.

```sh
cargo test --offline --locked -p ess-gen -p ess-diff -p ess-synth --no-fail-fast
```

`projection-regression.log`: **598 passed, zero failed/ignored**, exit0. These unchanged projection, deterministic-byte and diff tests are regressions, not new cases. `cargo fmt --check -p ess-conformance`, package `cargo clippy --all-targets -- -D warnings`, and package rustdoc with warnings denied each exited0. Early lint failures (dispatch length and test semicolons) were corrected normally and remain in their logs.

The final frozen-source lint is `clippy-frozen-final.log`, exit0. The requested `cargo build --locked --offline --bin ess` is `cli-build-final.log`, exit0; its binary SHA-256 is `61d8e008594e69763b07c824d32e6d647bc442f56bbd3bad5b4e8d0ba68558b6`. The corrected CLI re-synthesized the unchanged source6 subject-history model to suite10, and exact `cmp` against the pre-correction CLI's retained bytes exited0 (`legacy-before.log`, `legacy-after.log`, `legacy-byte-compare.log`). Existing canonical fixture tests also remain green. This checks observed legacy bytes, not every possible legacy model.

| Source mutation | Deciding case/filter | Runner summary | Exit |
|---|---|---|---:|
| Disable original row admission only | original_complete_rows_refuse_before_any_retry_callback | 0 passed; 1 failed | 101 |
| Disable post row admission only | post_command_complete_rows_execute_typed_admission_before_equality | 0 passed; 1 failed | 101 |
| Source7 complete pair replaced by legacy pair | source7_ | 0 passed; 2 failed | 101 |
| Disable immediate eligibility refusal | unchanged adversary terminal-state case | 0 passed; 1 failed | 101 |
| Disable immediate eligibility refusal | identical original-input conditions | 0 passed; 1 failed | 101 |

Original-row tests require failure before another command (`calls=1`). Post-row tests require the typed actual-row diagnostic after the retry (`calls=2`); generic inequality is explicitly insufficient. Disabling post admission produces `generic inequality is not evidence of typed admission`. Disabling original admission produces the wrong call count, so the post check cannot hide it. Each mutation compiled and reached behavior. Exact original source hashes were restored and checked after every mutation (`mutation-before.sha256`); no mutation remains.

## 5. Limits and remaining ownership

No full repository, consumer, release or adopter-runtime qualification is claimed. The coordinator owns the real EKR draft preflight, repository gate, inherited consumer classification/path issues, release compiler and later EKR activation. The requested development CLI build is compatibility evidence only.

Complete-row recursive tests directly exercise List/Map values, named state enums, required/optional fields and extra keys; the Go split also exercises nested Struct. A separate valid complete-row Union payload case was not added. Existing retained-result recursion/type cases exercise shared Union/newtype machinery. Floating, reading and invariant observer rejection follows the shared finite declaration collector; not every unsupported family has a new source-level test. Raw arbitrary JSON transport parity is outside the adopted lossless native-target profile; global numeric primitives and legacy matchers are unchanged.

The original four adversary assertion bodies remain byte-identical. Their entire original appended suffix was extracted from the opening commit and compared against the corresponding current prefix (`adversary-original.rs`, `adversary-unchanged.rs`, cmp0). The independent Go fixture is unchanged. The inherited first15 retained tests were updated only where the promised source7/replay protocol now requires the explicit complete pair. No legacy generated artifacts were regenerated or formatted.

## 6. Outside writes and handback

`private-inventory.md` names every external writable root in full, and `written-paths.txt` lists retained correction scratch files. Source/stat/hash, exact commands, raw red/green/mutation logs, legacy byte captures and reports are retained under the assigned correction scratch. Target output, test TMPDIR modules, existing Go cache and own managed lease bookkeeping are separately identified. No cleanup, commit, dependency install or publication was performed. Source is frozen, all final hash checks pass, and no compiler remains owned by this unit. `lease-release.log` records release of only the primary's own lease; the Go split records its lease separately.
