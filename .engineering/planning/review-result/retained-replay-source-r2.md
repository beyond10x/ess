---
format: aep.planning-md/1
id: review-result:retained-replay-source-r2
kind: review-result
status: active
title: Independent verification of corrected retained-result boundaries
relations:
- reviews: story:retained-command-result-replay
revision: 1
---
Owners: 0 findings, 0 coordinator, 0 implementor. The appended independent report is quoted verbatim as the source; its findings array is empty.

unit: story:retained-command-result-replay correction, immutable candidate 21f2da5dbcbdf235442966df196590d0f494a727
verdict: nothing found
cases: executed 463→465, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 output roots; full paths and retained-file inventory recorded privately
needs-coordinator: record F1/F2 as addressed in this bounded pass; repository, consumer and release qualification remain coordinator-owned

```text
$ git --no-pager diff --stat
 .../ess-conformance/tests/retained_replay.rs | 152 +++++++++++++++++++++
 1 file changed, 152 insertions(+)
```

Only that appended test path was changed. No production, Go, fixture, document, manifest, planning or inherited assertion was edited. The candidate test file is an exact byte prefix of the final file. All tracked opening SHA-256 values match except the intentionally appended test file. The original four first-pass cases remain an identical 7,822-byte block compared with commit `89cab6479075efa37f2578a86913f8248db96872`; their Go fixture also matches exactly.

## 1. Original regressions, then new probes

I read the correction brief, original brief and charter, adopted addendum, source changes and the three committed correction reports before executing the bounded correction review. Each of the original four cases was rerun alone before new probes or the package suite:

```sh
cargo test --offline --locked -p ess-conformance --test retained_replay CASE -- --exact --nocapture
```

| Original case | Actual result | Exit |
|---|---|---:|
| `adversary_replay_requires_complete_actual_subject_rows_not_only_complete_view_declarations` | 1 passed; 0 failed; 28 filtered out | 0 |
| `adversary_replay_must_reach_its_own_state_guard_after_the_origin` | 1 passed; 0 failed; 28 filtered out | 0 |
| `adversary_source7_wrong_state_refusal_requires_actual_complete_subject_observation` | 1 passed; 0 failed; 28 filtered out | 0 |
| `adversary_go_replay_requires_complete_actual_subject_rows` | 1 passed; 0 failed; 28 filtered out | 0 |

Raw logs are `original-CASE.log`. The original Rust replay loop now reaches its missing-field and wrong-type entries as well as the identity-only entry that formerly stopped it. The Go positive and negative child controls execute through the emitted runner; a passing Rust wrapper means the invalid child is rejected.

Two additional cases were written before their isolated runs:

- `adversary_r2_replay_checks_actual_guard_state_before_retry`, `crates/verify/ess-conformance/tests/retained_replay.rs:1751`, runs the generated Commit replay against the existing state backend while its fresh query reports valid enum value Rejected after the original commit. The normal backend passes; the altered observation produces a failed expected-Committed check before the replay-result check. Its final isolated run (`actual-guard-behavior.log`) exits 0, 1 passed/0 failed. My first version additionally required the runner to halt before calling retry. That assumption was stronger than the adopted contract and contradicted existing `expect_view` behavior, which records failure and continues. The first run therefore failed only its call-count assertion (`left: 2`, `right: 1`) **after already observing the correct Failed scenario**. That refuted hypothesis is retained in `first-adversary_r2_replay_checks_actual_guard_state_before_retry.log` and `probe-with-refuted-stop-assumption.rs`; it is not a product finding. I revised only this newly authored assertion to require the actual failed state diagnostic in the proper check order.
- `adversary_r2_complete_rows_validate_union_payload_and_exact_integer_bounds`, `retained_replay.rs:1811`, exercises the new public complete-row authority with List→Union→Map<Integer> and a named Enum alternative. Exact i64 endpoints and 9007199254740993 are accepted; unsigned overflow, a Boolean Integer, missing Union payload, unknown tag and unknown Enum member refuse. The first isolated run exits 0, 1 passed/0 failed (`first-adversary_r2_complete_rows_validate_union_payload_and_exact_integer_bounds.log`). This is a Rust typed-row boundary test, not a claim about arbitrary raw JSON transport or a new Go Union test.

The final two-case run, after qualifying two new-test `RefCell::default()` calls for Clippy, is `new-cases-final.log`, exit 0:

```text
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 29 filtered out; finished in 0.01s
```

## 2. Legacy Go and scoped package execution

The same provisioned Go **1.25.10** was used with `GOTOOLCHAIN=local`, assigned JavaScript tools, external TMPDIR and Cargo target, two jobs, no incremental/dev/test debug info. Full environment paths are private. Neither legacy test nor its fixture was changed.

```sh
cargo test --offline --locked -p ess-conformance --test authored entity_setup_execution::entity_setup_generated_go_reads_established_backend_rows_and_rejects_faults -- --exact --nocapture
cargo test --offline --locked -p ess-conformance --test optional_shape_go go_shape_leaves_honour_optional_without_going_unchecked -- --exact --nocapture
cargo test --offline --locked -p ess-conformance --test retained_replay generated_go_replay_runtime_executes_actual_results_and_strict_admission -- --exact --nocapture
```

Each wrapper selected and passed exactly one Rust case, exit 0. `legacy-setup.log` preserves all seven original target modes: good exits 0; empty/wrong/reversed/borrowed exit 1; unsupported/absent-capability exit 0 with their expected inconclusive reports. Those unsupported controls are not counted as passing conformance. `legacy-optional.log` records the unchanged direct-Step optional-shape lane passing. `retained-go.log` contains the full ordinary fixture JSON output and this measured summary:

```text
Go retained-result runtime: 186 passed; 0 failed; 0 skipped
```

That fixture includes complete-row original/replacement admission, identity and pair authority, optional/null/extras, mixed legacy and complete steps, descriptor refusals, exact native Integer identity, and exact large Integer input reaching both original and retry callbacks. Go terminal test/subtest outcomes are counted separately from the enclosing Rust case, not added to the package count.

Only after the isolated probes and legacy lanes:

```sh
cargo test --offline --locked -p ess-conformance --no-fail-fast
```

`package.log`, exit **0**, contains 43 actual runner summaries totaling **465 passed, 0 failed, 0 ignored**. Its retained-replay target reports:

```text
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s
```

The baseline **463** is the primary's frozen package report; I did not run a pre-addition package baseline. The primary's reported 598 projection cases and the coordinator's 33 generated EKR scenarios were not rerun or counted here. This is not the full repository gate.

`cargo clippy --offline --locked -p ess-conformance --tests -- -D warnings` exits 0 in `clippy-final.log`. The initial Clippy invocation found only two `Default::default()` style errors in my new case; `clippy.log` retains that result. Replacing them with the equivalent qualified constructors changed no assertion; the final two-case run passes. Single-file rustfmt check and `git diff --check` pass. No broad suite was repeated for those equivalent test-initializer changes.

## 3. F1/F2 disposition

| Original signature | Disposition in this pass | Source and executed basis |
|---|---|---|
| F1, `synthesize.rs:1677`, NEEDS-CHANGE / introduced | Addressed | The new complete pair carries `SubjectShape`; `subject.rs:89` admits actual required typed values. Rust `runner.rs:1932,1942` validates both snapshots; Go `src/go/replay.go:170` does likewise. Original Rust replay, source7 refusal and Go projection regressions all pass unchanged. The scoped package exercises stage-specific original/replacement controls, complementary views and legacy coexistence. |
| F2, `synthesize.rs:1643`, NEEDS-CHANGE / introduced | Addressed | `synthesize.rs:1724` evaluates immediate eligibility using original input and post-origin subject, accounts for competing/default outcomes, and supplies actual state/fact observations. The original terminal-state guard case now refuses synthesis; the added actual-state probe verifies a wrong but well-typed state fails the executed witness. Existing identical-input and external eligibility cases pass. |

The Go compatibility correction selects exact raw-Step decoding only for suite12/13 (`src/go/replay.go:56`), preserving the legacy direct decoder's number and event-shape behavior. Both measured compatibility regressions pass through their unchanged wrappers. The existing callback test demonstrates 9007199254740993 arrives exactly at the actual origin and retry commands; it does not certify arbitrary numeric token spellings through unrelated adapters.

Owners: the primary implementor owns the Rust complete-observation and immediate-eligibility corrections; the Go implementor owns runner parity and both measured legacy decoder repairs. The coordinator owns their adopted contract, integration and release gates. My overstrong fail-fast assertion and new-test style errors are mine and excluded from product findings. No new product decision or unresolved implementation finding emerged from this pass.

## 4. Limits and handback

- No remaining blocker was reproduced in the corrected boundaries exercised here; this is not approval or human review authority.
- Legacy snapshot assertions remain distinct from complete assertions. Generic error/no-events assertions do not acquire an invented subject requirement; whole-pair relabeling is a different witness, as the adopted addendum specifies.
- Existing package checks for complete descriptors, fresh queries, identity binding, complementary views, exact integers, default held-state refusals and old-format compatibility pass. These are inherited cases, distinguished from the two newly authored probes above.
- EKR restart, later-head retry, persistent writer behavior, parser activation, full consumer coverage and release qualification are outside this review. The coordinator's CLI temporary-output placement issue was not encountered by this scoped package; no placement guard was changed.
- Five output roots were used: assigned review scratch, assigned Cargo target, assigned TMPDIR, standard Go build cache and managed lease state. `private-paths.md` gives full paths; `retained-scratch-files.txt` enumerates retained evidence. Old review/correction logs were preserved. No cleanup, AEP command, source mutation, commit or publication was performed.
- Source hashes, original-case preservation and lease/process handback are retained in `source-after-check.log`, `preservation.md` and `handback-status.md`. Root owns every subsequent source and integration action.

```findings
[]
```
