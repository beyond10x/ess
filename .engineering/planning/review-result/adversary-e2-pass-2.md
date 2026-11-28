---
format: aep.planning-md/3
id: review-result:adversary-e2-pass-2
kind: review-result
status: active
title: Adversary pass 2 on story:the-rust-dispatch-selects-a-binding-failure-policy-per-refusal-of-the-invoked-command-and-counts-attempts
relations:
- reviews: story:the-rust-dispatch-selects-a-binding-failure-policy-per-refusal-of-the-invoked-command-and-counts-attempts
revision: 1
---
unit: E2 per-refusal Rust dispatch after correction 1, uncommitted tree ess-wr-e2 on base f989692194
verdict: NEEDS-CHANGE
cases: executed 267→270, red 3
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 files in S/wave-1127/e2/scratch-adv2/: system.diff, red-alone.log, red-alone-3.log, suite.log
needs-coordinator: a decision on what the pump does with a failure on a retried attempt that is no longer held: report it or drop it

New file tests/adversary_rust_refusal_policy_pass2.rs.
- an_escalation_builder_failure_on_a_retried_attempt_is_reported (busy at-limit): expected …2 at-limit unmet -, got …2 at-limit drop -.
- a_port_unmet_obligation_at_the_bound_is_reported (busy busy unmet): expected attempt 3 unmet, got drop.
- a_bounded_retry_whose_refusals_are_all_final_still_bounds_the_port_unmet_obligation: Err(unmet) arm pushes held_archive_order with no count and no bound.
Cause of the first two: pump discards a retried attempt's error (let _ =, system.rs:1472), relying on "a held-back attempt that fails again stays held", now false.
Suite: 270 tests, 267 passed, 3 failed, 62.7 s.
Judgement, no case: a failure before the port is called is held again every pump with its count frozen (system.rs:1630, design page :55), against :262-264.
Could not break: invocation-record test = port called; failed selection and Unknown condition not held; made reserved only for counted bindings (no model 0.99.8 accepted is refused); other new names bound after the transformation; one-policy byte pins.

```findings
- file: crates/generate/ess-synth/src/rust/system.rs
  line: 1472
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: an escalation builder failure on a retried attempt is discarded by pump and not held, so the occurrence and its obligation vanish unreported, against design page :266 and CHANGELOG:20
- file: crates/generate/ess-synth/src/rust/system.rs
  line: 1929
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a port unmet obligation at the bound is reached only on a retried attempt and is then dropped with nothing reported, while CHANGELOG:20 says it propagates out of pump
- file: crates/generate/ess-synth/src/rust/system.rs
  line: 202
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: a bounded retry whose every refusal is final counts no attempts, so a port unmet obligation is held without bound against attempts; the model is legal but no user of it was found
- file: crates/generate/ess-synth/src/rust/system.rs
  line: 1630
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a failure before the port is called is held again every pump with its count frozen, while the design page :262-264 forbids automatic retry and a bounded retry whose count cannot advance
```
