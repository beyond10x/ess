---
format: aep.planning-md/3
id: review-result:adversary-rp-go-pass-2
kind: review-result
status: active
title: Adversary pass 2, wave runtime-parity unit go
relations:
- reviews: story:generated-runtimes-run-every-emitted-suite-version
revision: 1
---
unit: story:generated-runtimes-run-every-emitted-suite-version (go), head 77339115e + untracked adversary_runtime_parity_go_pass2.rs
verdict: red
cases: executed 1189→1195, red 3
origin: introduced 1, pre-existing 0, undecided 2
wrote-outside-worktree: ~/.cache/ess-wave-rp/go/adv2/case-run.log, suite-run.log
needs-coordinator: no

Adversary pass 2 (`aep:adversary`), 2026-09-28. Pass-1 F1 and F2 fixed (4/4 pass-1 tests green); no older Go lane regressed.

| test | now |
|---|---|
| `go_requires_a_null_leaf_on_expect_event_values_to_be_carried_as_the_reference_runner_does` | red (Go passed, Rust failed) |
| `go_waits_for_an_eventual_event_as_long_as_the_reference_runner_does` | red (Go not observed within budget; Rust passed) |
| `go_reads_only_the_last_commands_events_after_an_eventual_observation` | red (Go failed expect_no_event; Rust passed) |
| `go_selects_the_same_eventual_occurrence_as_the_reference_runner_in_either_order` | green |
| `go_compares_named_event_values_exactly_as_the_reference_runner_does` | green |
| `go_holds_container_leaves_as_the_reference_runner_does_in_every_form` | green |

`cargo test -p ess-conformance --no-fail-fast`: EXIT=101, 1191 passed, 4 failed (3 above + known TS admission half).

Held: eventual occurrence order both ways; number equality 1/1.0, -0/0, 2^53+1; nested Optional null vs omitted; container leaves in every form; r.observed reset per command (runtime.go:7588).

```findings
[{"file":"crates/verify/ess-conformance/src/go/fixtures.go","line":231,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"expectEventValues still compares values through matches, so a dotted leaf required to be null passes when left out where the Rust expect_event_values fails it; only a hand-edited suite reaches it"},{"file":"crates/verify/ess-conformance/src/go/runtime.go","line":1466,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"undecided","message":"the Go harness gives an eventually step 8 attempts where the Rust runner gives about 50 asks, so an event observed on the 9th to 50th ask passes in Rust and fails in Go"},{"file":"crates/verify/ess-conformance/src/go/runtime.go","line":2213,"category":"concurrency","severity":"note","verdict":"INFEASIBLE","origin":"undecided","message":"eventuallyEvent appends to r.observed, so an expect_no_event or expect_event after it reads eventually-observed events where Rust reads only the last command direct events; neither the synthesizer nor authored scenarios write that step order"}]
```

Trend: pass 1 → 2, pass 2 → 3, carried 0 (the fixtures.go finding is pass-1 F2 on a sibling path). Coordinator routing: correction 2 (last) for all three — they are local; undecided ones decided in scope for parity. Coordinator verifies by diff read.
