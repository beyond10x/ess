---
format: aep.planning-md/3
id: review-result:adversary-e2-pass-1
kind: review-result
status: active
title: Adversary pass 1 on story:the-rust-dispatch-selects-a-binding-failure-policy-per-refusal-of-the-invoked-command-and-counts-attempts
relations:
- reviews: story:the-rust-dispatch-selects-a-binding-failure-policy-per-refusal-of-the-invoked-command-and-counts-attempts
revision: 1
---
unit: E2 story:the-rust-dispatch-selects-a-binding-failure-policy-per-refusal-of-the-invoked-command-and-counts-attempts, uncommitted tree ess-wr-e2 on base f989692194
verdict: NEEDS-CHANGE
cases: executed 264→267, red 3
origin: introduced 3 / pre-existing 0 / undecided 2
wrote-outside-worktree: S/wave-1127/e2/scratch-adv/ (red-alone.log, suite.log, clippy.log)
needs-coordinator: two decisions (UnmetObligation counts against attempts:? size of the block)

Cases: tests/adversary_rust_refusal_policy_pass1.rs (new, 165 lines); fixture refusal-policy-harness/main.rs +4 lines (builder Err for ids starting nobuild).
- an_escalation_builder_failure_does_not_invoke_the_command_again: extra line "attempt demo.ledger.Record nobuild1 2 recorded accepted demo.ledger.Recorded".
- a_port_invocation_answered_unmet_counts_against_the_bound: script busy unmet busy recorded; expected drop at attempt 3, got attempt 4 recorded.
- a_binding_named_made_is_not_shadowed_by_the_attempt_count: fn deliver_made(..., made: u32) contains let input = made(event); E0618.
Suite: Summary [47.424s] 267 tests run: 264 passed, 3 failed (the 3 above). Slowest 11.5 s.
Not broken: per-refusal choice incl. fallback and final:, count across two refusals selecting one retry, retry bound 2, plain bounded retry, shared outcome names, refusal in two policies and attempts<2 refused by the domain, selecting and conditioned bindings, one-policy byte pins, Go/web refusal texts, harness built once, the story's 3 mutants.

```findings
- file: crates/generate/ess-synth/src/rust/system.rs
  line: 2009
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a failed escalation builder under a refusal-selected policy propagates with ? into attempt_<binding>, which holds the event, so the next pump re-invokes the command, against design page :266 and the interpreter
- file: crates/generate/ess-synth/src/rust/system.rs
  line: 1579
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: a port invocation answered UnmetObligation is held with made unchanged, so it does not count against attempts, while design page :240-241 counts every attempt before the port call
- file: crates/generate/ess-synth/src/rust/system.rs
  line: 1577
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the counted dispatch's parameter made shadows the generated transformation function of a binding named made, emitting made(event) on a u32 that does not compile, and rust/feasibility.rs does not reserve the name
- file: docs/design/conditional-binding-failure-policies.md
  line: 52
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: undecided
  message: the page says a restart may retry once more, but the held event is not persisted, so a restart loses the occurrence, and a host redeliver restarts the count at 0 for up to attempts further invocations; case not built
```
