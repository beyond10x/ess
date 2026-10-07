---
format: aep.planning-md/3
id: review-result:arithmetic-completeness-413a-20261004-r1
kind: review-result
status: active
title: 'First whole arithmetic review: package acceptance blocked by retained replay failures'
relations:
- reviews: story:counter-reachability-arithmetic-completeness
revision: 1
---
unit: ESS413A refreshed base2255315a48a947f363d46cb3c691a57b4879f3b1, exact two-path candidate
verdict: blocked — whole-unit independent verification and acceptance incomplete
cases: independent executed0; owning package aggregate2609passed/2failed/8ignored/0filtered across337summaries
origin: introduced0 / pre-existing0 / undecided2 package failures
wrote-outside-worktree: this report in assigned private review scratch
needs-coordinator: trace and resolve retained_replay failures, green required package evidence, then separately admit unexecuted independent probe

```text
 .../ess-conformance/src/synthesize/subject_fact.rs |  78 ++++++-
 .../verify/ess-conformance/tests/counter_limit.rs  | 243 +++++++++++++++++++++
 2 files changed, 317 insertions(+), 4 deletions(-)
```

This is the frozen author diff, not reviewer production edits. No concrete defect was found in the arithmetic source review. The two required-package failures nevertheless prevent whole-unit acceptance. This report does not prescribe an arithmetic source change, classify either failure as pre-existing, or treat focused GREEN results as a waiver. The coordinator owns origin tracing and any corrective scope.

## Exact reviewed source

HEAD is2255315a48a947f363d46cb3c691a57b4879f3b1. Refresh record SHA256098efff4777ac170c101fdf05b224f6c8e4aa1f6f5b333dac14613d15319e805 binds the unchanged two-path patch to that base. Original source-handoff freeze SHA256bbc9d2f51fb3a3f7fbadb539e833143c2e8a8c36d08a3433566fb9514c14833e remains retained.

- subject_fact.rs SHA256cb3b9fb0d1b4acd7aabf8a43f813e3920133f480ce0fd55dc2d88eef08d9dc20.
- counter_limit.rs SHA25626e00ff65a9ab39c453bd60ad2da24433e567cc61115a0fc094ae65b923cbc44.

Both hashes were independently re-read after the terminal package result and still match. Review covered the complete two-path diff, the accepted story and refreshed-base delta. The production correction propagates unavailable required bounds/magnitudes through the existing incomplete result. Number, CAS grounding, related guards, seeds, formats and runtime adapters remain unchanged by this unit. The refreshed base changes Go/TypeScript runtime parity and related fixtures; that is provenance context, not a reproduced cause of the failures.

## Required checks actually observed

The owning external coordinator (not the ESS integrator) executed package-validation-plan4. results.json records strict all-target Clippy exit0 in59.909571seconds; owning cargo fmt exit0 in2.116964seconds; repository task fmt-check exit0 in6.124133seconds. The unfiltered command was:

```text
cargo test -p ess-conformance --locked --offline --no-fail-fast -- --test-threads=1 --nocapture
```

It terminated exit101 after1128.553625seconds. terminal-audit.json totals337 summaries,2609passed,2failed,8ignored and0filtered. These are the retained audit's aggregate runner counts, not a new claim of that many distinct independently reviewed cases. No missing-runtime skip notices were recorded; the eight explicit ignored cases remain listed in the audit. No tests were rerun by this reviewer.

Only retained_replay has failed Rust cases. Its target summary is34passed/2failed/0ignored/0filtered:

- adversary_go_replay_requires_complete_actual_subject_rows: assertion at crates/verify/ess-conformance/tests/retained_replay.rs:1241, recorded in raw4.log at7763.
- generated_go_replay_runtime_executes_actual_results_and_strict_admission: assertion at the same file:570, recorded at8836.

The actual generated Go output includes `complete subject requires a fresh query of retained.core.Records`, `retained.core.Seed returned no consistency token`, and `no consistent view query preceded the subject snapshot`. These are observed diagnostic messages. No baseline reproduction was admitted, so neither issue is attributed to this arithmetic patch or declared pre-existing. A failed fixture/runtime assertion is not by itself the root cause.

Evidence under the owning private evidence root:

- package-run-1/4.log SHA256b4ff23325419aaf72d5b84c5247c5b1bc28a007106612e1c80c469ad171c1b0b; complete raw package output.
- package-run-1/terminal-audit.json SHA256b5847852be5cfa29f81b6f652491891817a6f02b0bcf8a35043475ab87ec343e; counts, ignored lines, runtime skips, hashes and failed cases.
- package-run-1/results.json SHA25622d0464e17aefd28950b48e796211770b9df883f81da838bbe805255c80a090b; exact commands, exits, durations, disk observations and every check-log hash.

## Arithmetic evidence and honest limits

The author baseline had3passing/2failing arithmetic cases at MAX/MIN; focused treatment had5/0 and counter_limit17/0. The refreshed package preserves the ordinary-two canonical hash6bf91d8f43da8103fa8e0f0a9898dc9002b292934f8a60d494769c72d0826c5e and shared-related-three hash8f56c982d3834b1eb7a3290a7a460d79fd2ca8c7fc4d821ad6a5df1748c96958, equal to the retained baseline. The finite CAS trace and faulty-target controls remain unchanged.

The mixed-MIN test was already GREEN on baseline and can refuse through padding. negated reconstructs text with Number::decimal_literal, which can carry integers wider than i64, whereas checked_add bounds integral sums to i64. It is not independent proof that a failed negative-magnitude Option was omitted by filter_map; no Number defect is inferred. The checked collection addresses that omission structurally, with this branch-coverage limitation retained.

The independently prepared public-synthesis MAX/MIN refusal probe remains entirely UNEXECUTED. Probe SHA25696346e1e50da2c3297a19f17ca154efb40eb2fdabae1d3cc5f994921ab767436; its seventeen author cases are an unchanged prefix. The grant was conditional on package success and therefore did not admit execution. Its proposed completeness-message/ordinary-execution oracle has not reached executable safety-proof level4. No fresh rlib admission, compile or probe result is claimed.

## Handoff

Hold whole-unit acceptance and integration. Resolve the two package failures through the coordinator's owner process, bind any later base/source changes, restore required green package evidence and obtain a separate probe grant before completing independent verification. Preserve this failed run and every earlier report/fixture. No extra private-helper infrastructure, speculative correction, scope expansion or test weakening was performed.

The exact outside-worktree write for this reporting turn is adversary-review-1/report-final-blocked-2255315.md. Earlier preparation paths remain in their inventories. The reviewer lease was already released after preparation; no reviewer lease, process or compiler lane remains active. No source, store, host configuration, runtime data or integration state was changed.

```findings
- file: crates/verify/ess-conformance/tests/retained_replay.rs
  line: 1241
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: undecided
  message: The required unfiltered package run fails adversary_go_replay_requires_complete_actual_subject_rows at this assertion; origin and corrective scope remain undecided.
- file: crates/verify/ess-conformance/tests/retained_replay.rs
  line: 570
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: undecided
  message: The required unfiltered package run fails generated_go_replay_runtime_executes_actual_results_and_strict_admission at this assertion; origin and corrective scope remain undecided.
```
