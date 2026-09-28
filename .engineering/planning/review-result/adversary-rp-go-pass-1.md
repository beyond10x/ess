---
format: aep.planning-md/3
id: review-result:adversary-rp-go-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave runtime-parity unit go
relations:
- reviews: story:generated-runtimes-run-every-emitted-suite-version
revision: 1
---
unit: story:generated-runtimes-run-every-emitted-suite-version (go)
verdict: red
cases: executed 1184→1188, red 3 (2 new; the third is the known TS half of runtime_suite_admission)
origin: introduced 2, pre-existing 0, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-rp/go/adv1/case-run.log, suite-run.log
needs-coordinator: no

Adversary pass 1 (`aep:adversary`), 2026-09-28, head 7b21e59d1 + `crates/verify/ess-conformance/tests/adversary_runtime_parity_go.rs`.

| test | asserts | now |
|---|---|---|
| `go_holds_an_eventually_observed_event_to_its_shape_as_the_reference_runner_does` | bounded-retry fixture /26, target publishes Recorded with order_id as number or omitted; Rust fails binding/flow and binding/delivery | red |
| `issue_188_go_fails_every_wrong_generated_leaf_the_reference_runner_fails` | #188 spec, wrong rank / lead variants fail lead-set on both | green |
| `go_requires_a_null_leaf_value_on_an_event_to_be_carried_as_the_reference_runner_does` | hand-edited suite requiring lead.data: null; target omits it; Rust fails | red |
| `the_emitted_suite_26_package_is_gofmt_clean_and_passes_go_vet` | gofmt -l empty, go vet clean for dialer and ledger /26 | green |

`cargo test -p ess-conformance --no-fail-fast`: EXIT=101, 1185 passed, 3 failed.

Not broken: #188 wrong generated leaves; gofmt/go vet; skip path reported as skipped (inconclusive), Rust reports unsupported; bounded-retry unsupported observation; read-compare of changed_by, page, expect_subject_absent, snapshot_view/expect_view_unchanged, now_offset, presence.

```findings
[{"file":"crates/verify/ess-conformance/src/go/runtime.go","line":2187,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"eventuallyEvent ignores the step payload and shape, so Go passes /26 binding flow and delivery scenarios whose observed event is malformed and which the Rust runner fails"},{"file":"crates/verify/ess-conformance/src/go/runtime.go","line":2882,"category":"contract-drift","severity":"warning","verdict":"INFEASIBLE","origin":"introduced","message":"expectEvent reads event payload values through matches, so a dotted leaf required to be null passes when left out, while the Rust expect_payload requires it to be carried; shown only on a hand-edited suite"}]
```

Coordinator routing: both to the implementor (correction 1); F2 is fixed for parity although only a hand-edited suite reaches it today.
